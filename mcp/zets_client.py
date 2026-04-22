#!/usr/bin/env python3
"""
zets_client.py — single-persona ZETS client with peer-talking support.

Each client:
  - Runs as HTTP server on persona.port (3251-3266)
  - Has its own local knowledge (JSON-backed, persona-seeded)
  - On a query:
      1. Try local knowledge
      2. If confidence < threshold, ask a peer (or cloud, or internet)
      3. Judge peer's answer (persona-weighted)
      4. Learn from peer if confidence is acceptable
  - Supports peer-to-peer queries with TTL + seen-set (prevents loops)

Daily/nightly plan:
  - Each persona has a list of learning goals (topics they want to explore).
  - night_school.py kicks off rounds of cross-talk on those topics.

Usage:
  python3 mcp/zets_client.py serve 3251             # start one client
  python3 mcp/multi_client_v2.py spawn              # start all 16
"""

import argparse
import hashlib
import json
import os
import signal
import socket
import sys
import threading
import time
import urllib.request
import urllib.error
from dataclasses import asdict
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from typing import Optional

# Make personas importable whether run directly or from the zets root
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from personas import (  # noqa
    PERSONAS, get as get_persona, format_response,
    judge_peer_answer, should_ask_peer, Persona,
)

ZETS_ROOT = Path(os.environ.get('ZETS_ROOT', '/home/dinio/zets'))
CLIENTS_DATA = ZETS_ROOT / 'data' / 'clients'
CLIENTS_LOG = ZETS_ROOT / 'mcp' / 'logs' / 'clients'

CLOUD_URL = os.environ.get('ZETS_CLOUD_URL', 'http://127.0.0.1:3147/api')

# Hard limits to prevent infinite loops
MAX_TTL = 4
MAX_SEEN_PER_QUERY = 20


# ═══════════════════════════════════════════════════════════════════════
# ClientState — persona + local knowledge + sync log
# ═══════════════════════════════════════════════════════════════════════

class ClientState:
    def __init__(self, persona: Persona):
        self.persona = persona
        self.port = persona.port
        self.name = persona.name
        self.store_file = CLIENTS_DATA / f"{persona.name.lower()}.json"
        self.lock = threading.Lock()
        self.load()

    def load(self):
        if self.store_file.exists():
            try:
                d = json.loads(self.store_file.read_text(encoding='utf-8'))
                self.atoms = d.get('atoms', {})
                self.learned = d.get('learned', {})
                self.plan = d.get('plan', self._initial_plan())
                self.sync_log = d.get('sync_log', [])
                self.conversations = d.get('conversations', [])
                return
            except Exception:
                pass
        # fresh
        self.atoms = self._seed_atoms()
        self.learned = {}       # topic → {'answer': str, 'confidence': float, 'source': str, 'ts': float}
        self.plan = self._initial_plan()
        self.sync_log = []
        self.conversations = []
        self.save()

    def _seed_atoms(self) -> dict:
        p = self.persona
        s = p.style
        return {
            'name': p.name,
            'name_he': p.name_he,
            'age_hint': p.age_hint,
            'lang': p.lang,
            'bio': p.bio,
            'cognitive_note': p.cognitive_note,
            'likes': s.preferred_topics,
            'avoids': s.avoided_topics,
        }

    def _initial_plan(self) -> list:
        """Per-persona learning goals — topics they want to explore."""
        topics = list(self.persona.style.preferred_topics)
        # Plus a couple of stretch topics
        if len(topics) < 3:
            topics += ['world_facts', 'everyday']
        return [
            {'topic': t, 'priority': 1.0 - i * 0.1,
             'knowledge_gained': 0, 'last_asked': None}
            for i, t in enumerate(topics[:6])
        ]

    def save(self):
        CLIENTS_DATA.mkdir(parents=True, exist_ok=True)
        with self.lock:
            self.store_file.write_text(
                json.dumps({
                    'persona_name': self.persona.name,
                    'persona_port': self.persona.port,
                    'atoms': self.atoms,
                    'learned': self.learned,
                    'plan': self.plan,
                    'sync_log': self.sync_log[-200:],
                    'conversations': self.conversations[-100:],
                }, ensure_ascii=False, indent=2),
                encoding='utf-8',
            )

    # ── Local knowledge lookup ──────────────────────────────────────────
    def query_local(self, q: str) -> dict:
        ql = q.lower().strip()
        hits = []
        confidence = 0.0

        # Identity questions
        if any(x in ql for x in ('who are you', 'your name', 'מה שמך', 'מי אתה', 'מי את')):
            hits.append({
                'fact': f"{self.persona.name} ({self.persona.name_he})",
                'source': 'self_identity',
            })
            confidence = 1.0

        # Topic matches in atoms
        for key, value in self.atoms.items():
            if isinstance(value, str):
                if ql in value.lower() or value.lower() in ql:
                    hits.append({'fact': f"{key}: {value}", 'source': 'local_atoms'})
                    confidence = max(confidence, 0.75)
            elif isinstance(value, list):
                for v in value:
                    if isinstance(v, str) and v.lower() in ql:
                        hits.append({'fact': f"I like {v}", 'source': 'local_atoms'})
                        confidence = max(confidence, 0.65)

        # Previously-learned
        topic_key = ql[:80]
        if topic_key in self.learned:
            rec = self.learned[topic_key]
            hits.append({
                'fact': rec['answer'],
                'source': f"learned_from_{rec['source']}",
                'conf': rec['confidence'],
            })
            confidence = max(confidence, rec['confidence'])

        return {
            'hits': hits,
            'confidence': confidence,
            'has_answer': bool(hits),
        }

    # ── Cloud fallback (central ZETS) ───────────────────────────────────
    def query_cloud(self, q: str) -> dict:
        try:
            body = json.dumps({'question': q}, ensure_ascii=False).encode('utf-8')
            req = urllib.request.Request(f'{CLOUD_URL}/query',
                                         data=body, method='POST')
            req.add_header('Content-Type', 'application/json')
            with urllib.request.urlopen(req, timeout=30) as r:
                d = json.loads(r.read())
            # engine_output is a benchmark dump — extract a confidence proxy
            text = d.get('engine_output', '')
            # Very crude: 'Overall: 100.0%' means the pipeline ran clean on that query
            conf = 0.6  # default
            if 'Overall:    100.0%' in text or 'Overall:     100.0%' in text:
                conf = 0.7
            elif 'Overall:    0.0%' in text:
                conf = 0.3
            return {
                'answer': text[-800:],
                'confidence': conf,
                'source': 'cloud',
                'raw': d,
            }
        except Exception as e:
            return {'answer': None, 'confidence': 0.0, 'source': 'cloud',
                    'error': str(e)}

    # ── Peer fallback ───────────────────────────────────────────────────
    def query_peer(self, peer_port: int, q: str, ttl: int,
                   seen: set, asker: str) -> dict:
        try:
            body = json.dumps({
                'q': q, 'ttl': ttl, 'seen': list(seen), 'asker': asker,
            }, ensure_ascii=False).encode('utf-8')
            req = urllib.request.Request(
                f'http://127.0.0.1:{peer_port}/peer_ask',
                data=body, method='POST',
            )
            req.add_header('Content-Type', 'application/json')
            with urllib.request.urlopen(req, timeout=10) as r:
                return json.loads(r.read())
        except Exception as e:
            return {'error': str(e), 'answer': None, 'confidence': 0.0,
                    'source': f'peer_{peer_port}'}

    # ── Pick which peer to ask ──────────────────────────────────────────
    def pick_peer(self, q: str, exclude: set) -> Optional[int]:
        """Pick a peer who's likely to know about this topic, avoiding loops.

        Uses weighted-random to prevent everyone converging on Michel.
        Higher score = more likely to be picked, but not guaranteed."""
        import random
        ql = q.lower()
        candidates = []
        for p in PERSONAS:
            if p.port == self.port or p.port in exclude:
                continue
            score = 0.5  # baseline (everyone has a chance)
            # Topic match is strongest signal
            for topic in p.style.preferred_topics:
                if topic.lower() in ql or ql in topic.lower():
                    score += 1.5
            # Seniors with broad knowledge get a bump
            if p.age_hint == 'senior':
                score += 0.4
            # Proactive personas more likely to have something
            score += p.style.proactive * 0.3
            # Confidence-baseline
            score += p.style.confidence_baseline * 0.3
            candidates.append((p.port, score))

        if not candidates:
            return None
        # Weighted random pick — deterministic per query, diverse overall
        rng = random.Random(hash((self.port, q)) & 0x7fffffff)
        total = sum(s for _, s in candidates)
        r = rng.uniform(0, total)
        acc = 0.0
        for port, s in candidates:
            acc += s
            if r <= acc:
                return port
        return candidates[-1][0]

    # ── Main ask() — the pipeline ───────────────────────────────────────
    def ask(self, q: str, ttl: int = MAX_TTL, seen: Optional[set] = None,
            asker: str = 'user') -> dict:
        t0 = time.time()
        seen = set(seen) if seen else set()
        # Mark this client+query as seen
        qhash = hashlib.md5(q.encode('utf-8')).hexdigest()[:12]
        mark = f"{self.port}:{qhash}"
        if mark in seen:
            # Loop: we've been asked this exact query already in this chain
            return {
                'client': self.name, 'q': q,
                'answer': None, 'confidence': 0.0,
                'source': 'local', 'loop_detected': True,
                'elapsed_ms': 0,
            }
        seen.add(mark)

        # Stage 1: local
        local = self.query_local(q)
        my_conf = local['confidence']

        # Stage 2: if confidence low AND TTL left, consult peer/cloud/internet
        peer_result = None
        cloud_result = None

        if should_ask_peer(my_conf, self.persona) and ttl > 0 and len(seen) < MAX_SEEN_PER_QUERY:
            # Pick a peer
            exclude = {int(m.split(':')[0]) for m in seen if ':' in m}
            peer_port = self.pick_peer(q, exclude)
            if peer_port:
                peer_result = self.query_peer(peer_port, q, ttl - 1, seen,
                                              asker=self.name)
                # Treat peer "unknown" as no-answer
                if peer_result and peer_result.get('source') == 'unknown':
                    peer_result['answer'] = None
                    peer_result['confidence'] = 0.0
                # Judge peer answer strictly
                if peer_result and peer_result.get('answer'):
                    adjusted = judge_peer_answer(
                        peer_result.get('confidence', 0.5), self.persona
                    )
                    peer_result['adjusted_confidence'] = adjusted
                    # Learn if sufficiently confident
                    if adjusted >= self.persona.style.asks_for_help_threshold:
                        self._learn(q, peer_result['answer'], adjusted,
                                    f"peer_{peer_port}")

            # If still nothing, try cloud
            if (not peer_result or not peer_result.get('answer')) and my_conf < 0.3:
                cloud_result = self.query_cloud(q)
                if cloud_result and cloud_result.get('answer'):
                    self._learn(q, cloud_result['answer'],
                                cloud_result['confidence'], 'cloud')

        # Build the raw answer
        parts = []
        if local['has_answer']:
            for h in local['hits'][:3]:
                parts.append(h['fact'])
            raw = ' '.join(parts)
            source = 'local'
            out_conf = my_conf
        elif peer_result and peer_result.get('answer'):
            raw = str(peer_result['answer'])[:500]
            source = f"peer_{peer_result.get('source_port', '?')}"
            out_conf = peer_result.get('adjusted_confidence',
                                       peer_result.get('confidence', 0.0))
        elif cloud_result and cloud_result.get('answer'):
            raw = 'Cloud says: ' + str(cloud_result['answer'])[:300]
            source = 'cloud'
            out_conf = cloud_result.get('confidence', 0.0)
        else:
            raw = "I don't know this. Maybe ask someone who does." \
                  if self.persona.lang.startswith('en') \
                  else "אני לא יודע את זה. אולי כדאי לשאול מישהו אחר."
            source = 'unknown'
            out_conf = 0.0

        # Apply persona style
        formatted = format_response(raw, self.persona, seed=hash(q))

        result = {
            'client': self.name,
            'client_he': self.persona.name_he,
            'q': q,
            'answer': formatted,
            'raw': raw,
            'confidence': out_conf,
            'source': source,
            'peer_hit': bool(peer_result and peer_result.get('answer')),
            'cloud_hit': bool(cloud_result and cloud_result.get('answer')),
            'ttl_used': MAX_TTL - ttl,
            'source_port': self.port,
            'elapsed_ms': round((time.time() - t0) * 1000, 1),
        }

        # Log
        self.sync_log.append({
            'ts': time.time(), 'q': q[:120], 'asker': asker,
            'source': source, 'conf': round(out_conf, 2),
            'peer_hit': bool(peer_result), 'cloud_hit': bool(cloud_result),
            'elapsed_ms': result['elapsed_ms'],
        })
        self.save()
        return result

    def _learn(self, q: str, answer: str, confidence: float, source: str):
        key = q.lower().strip()[:80]
        existing = self.learned.get(key)
        if existing and existing['confidence'] >= confidence:
            return  # don't downgrade
        self.learned[key] = {
            'answer': str(answer)[:500],
            'confidence': float(confidence),
            'source': source,
            'ts': time.time(),
        }


# ═══════════════════════════════════════════════════════════════════════
# HTTP handler
# ═══════════════════════════════════════════════════════════════════════

def make_handler(state: ClientState):
    class H(BaseHTTPRequestHandler):
        def _send(self, code: int, body: dict):
            data = json.dumps(body, ensure_ascii=False).encode('utf-8')
            self.send_response(code)
            self.send_header('Content-Type', 'application/json; charset=utf-8')
            self.send_header('Content-Length', str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def do_GET(self):
            if self.path in ('/', '/health'):
                return self._send(200, {
                    'client': state.name,
                    'port': state.port,
                    'lang': state.persona.lang,
                    'bio': state.persona.bio,
                    'learned_items': len(state.learned),
                    'queries_served': len(state.sync_log),
                    'plan_topics': len(state.plan),
                })
            if self.path == '/state':
                return self._send(200, {
                    'atoms': state.atoms,
                    'learned_count': len(state.learned),
                    'plan': state.plan,
                    'last_queries': state.sync_log[-10:],
                    'last_conversations': state.conversations[-5:],
                })
            if self.path == '/persona':
                # Serializable persona dump
                p = state.persona
                return self._send(200, {
                    'name': p.name, 'name_he': p.name_he,
                    'age_hint': p.age_hint, 'lang': p.lang,
                    'bio': p.bio, 'cognitive_note': p.cognitive_note,
                    'style': asdict(p.style),
                })
            if self.path == '/plan':
                return self._send(200, {'plan': state.plan})
            return self._send(404, {'error': 'unknown path'})

        def do_POST(self):
            length = int(self.headers.get('Content-Length', '0'))
            raw = self.rfile.read(length).decode('utf-8', errors='replace')
            try:
                body = json.loads(raw) if raw else {}
            except json.JSONDecodeError:
                return self._send(400, {'error': 'invalid JSON'})

            if self.path == '/ask':
                q = body.get('q') or body.get('question', '')
                if not q:
                    return self._send(400, {'error': 'q required'})
                result = state.ask(q)
                return self._send(200, result)

            if self.path == '/peer_ask':
                # Peer-to-peer query. Has TTL and seen-set.
                q = body.get('q', '')
                ttl = int(body.get('ttl', MAX_TTL))
                seen = body.get('seen', [])
                asker = body.get('asker', 'unknown')
                result = state.ask(q, ttl=ttl, seen=set(seen), asker=asker)
                result['source_port'] = state.port
                return self._send(200, result)

            return self._send(404, {'error': 'unknown path'})

        def log_message(self, *a):  # silent
            pass
    return H


# ═══════════════════════════════════════════════════════════════════════
# main
# ═══════════════════════════════════════════════════════════════════════

def serve(port: int):
    p = get_persona(port)
    if not p:
        print(f"ERROR: no persona for port {port}", file=sys.stderr)
        sys.exit(1)
    state = ClientState(p)
    handler = make_handler(state)
    srv = HTTPServer(('127.0.0.1', port), handler)
    print(f"[{p.name}] :{port}  lang={p.lang}  plan={len(state.plan)} topics",
          flush=True)
    try:
        srv.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == '__main__':
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest='cmd', required=True)
    p = sub.add_parser('serve')
    p.add_argument('port', type=int)
    args = ap.parse_args()
    if args.cmd == 'serve':
        serve(args.port)
