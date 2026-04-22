#!/usr/bin/env python3
"""
multi_client_v2.py — orchestrator for 16 ZETS clients.

Replaces multi_client.py. Each client now:
  - Has a real persona (see mcp/personas.py)
  - Supports peer-to-peer querying with TTL-based loop prevention
  - Learns from peers (confidence-weighted)
  - Has a daily learning plan (topics to explore)

CLI:
  python3 mcp/multi_client_v2.py spawn              — start all 16
  python3 mcp/multi_client_v2.py spawn 5            — start first 5
  python3 mcp/multi_client_v2.py status             — list
  python3 mcp/multi_client_v2.py ask 3251 "..."     — query one
  python3 mcp/multi_client_v2.py roundtable "..."   — ask everyone
  python3 mcp/multi_client_v2.py night              — run learning night cycle
  python3 mcp/multi_client_v2.py stop               — stop all
"""

import argparse
import json
import os
import random
import signal
import socket
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from personas import PERSONAS, get as get_persona  # noqa

ZETS_ROOT = Path(os.environ.get('ZETS_ROOT', '/home/dinio/zets'))
CLIENTS_LOG = ZETS_ROOT / 'mcp' / 'logs' / 'clients'
PID_DIR = CLIENTS_LOG / 'pids'

CLIENT_SCRIPT = ZETS_ROOT / 'mcp' / 'zets_client.py'


def port_up(port: int, timeout: float = 0.3) -> bool:
    try:
        with socket.create_connection(('127.0.0.1', port), timeout=timeout):
            return True
    except OSError:
        return False


def _post(url: str, body: dict, timeout: float = 30) -> dict:
    data = json.dumps(body, ensure_ascii=False).encode('utf-8')
    req = urllib.request.Request(url, data=data, method='POST')
    req.add_header('Content-Type', 'application/json')
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return json.loads(r.read())
    except Exception as e:
        return {'error': str(e)}


def _get(url: str, timeout: float = 5) -> dict:
    try:
        with urllib.request.urlopen(url, timeout=timeout) as r:
            return json.loads(r.read())
    except Exception as e:
        return {'error': str(e)}


# ═══════════════════════════════════════════════════════════════════════
# Spawn / stop
# ═══════════════════════════════════════════════════════════════════════

def cmd_spawn(n: int):
    PID_DIR.mkdir(parents=True, exist_ok=True)
    CLIENTS_LOG.mkdir(parents=True, exist_ok=True)
    spawned = 0
    for persona in PERSONAS[:n]:
        if port_up(persona.port):
            print(f"  ✓ {persona.name:<9} :{persona.port}  already up")
            continue
        log_out = CLIENTS_LOG / f"{persona.name.lower()}.out"
        log_err = CLIENTS_LOG / f"{persona.name.lower()}.err"
        with open(log_out, 'a') as lo, open(log_err, 'a') as le:
            proc = subprocess.Popen(
                ['python3', str(CLIENT_SCRIPT), 'serve', str(persona.port)],
                stdout=lo, stderr=le, stdin=subprocess.DEVNULL,
                start_new_session=True,
            )
        (PID_DIR / f"{persona.name.lower()}.pid").write_text(str(proc.pid))
        # Wait for port to come up
        for _ in range(20):
            if port_up(persona.port):
                break
            time.sleep(0.1)
        if port_up(persona.port):
            print(f"  ✓ {persona.name:<9} :{persona.port}  (pid {proc.pid})  — {persona.bio[:50]}")
            spawned += 1
        else:
            print(f"  ✗ {persona.name:<9} :{persona.port}  failed to start")
    print(f"\n  spawned {spawned} new clients")


def cmd_stop():
    stopped = 0
    for persona in PERSONAS:
        pid_file = PID_DIR / f"{persona.name.lower()}.pid"
        if pid_file.exists():
            try:
                pid = int(pid_file.read_text())
                os.kill(pid, signal.SIGTERM)
                stopped += 1
            except ProcessLookupError:
                pass
            except Exception as e:
                print(f"  ✗ {persona.name}: {e}")
            pid_file.unlink(missing_ok=True)
    print(f"  stopped {stopped} clients")


def cmd_status():
    print(f"  {'name':<9} {'he':<6} {'port':<6} {'status':<8} {'learned':<8} {'queries':<8} {'bio':<40}")
    print("  " + "─" * 95)
    for p in PERSONAS:
        if port_up(p.port):
            d = _get(f"http://127.0.0.1:{p.port}/health")
            learned = d.get('learned_items', 0)
            queries = d.get('queries_served', 0)
            status = 'UP'
        else:
            learned = '-'
            queries = '-'
            status = 'DOWN'
        print(f"  {p.name:<9} {p.name_he:<6} {p.port:<6} {status:<8} "
              f"{str(learned):<8} {str(queries):<8} {p.bio[:40]}")


def cmd_ask(port: int, question: str):
    d = _post(f"http://127.0.0.1:{port}/ask", {'q': question})
    print(json.dumps(d, ensure_ascii=False, indent=2))


# ═══════════════════════════════════════════════════════════════════════
# Roundtable — ask everyone the same question
# ═══════════════════════════════════════════════════════════════════════

def cmd_roundtable(question: str):
    print(f"\n━━━ Roundtable on: {question!r} ━━━\n")
    results = []
    for p in PERSONAS:
        if not port_up(p.port):
            print(f"  {p.name:<9} (down)")
            continue
        d = _post(f"http://127.0.0.1:{p.port}/ask", {'q': question}, timeout=60)
        ans = d.get('answer', '(no answer)')
        src = d.get('source', '?')
        conf = d.get('confidence', 0.0)
        peer = d.get('peer_hit', False)
        print(f"  {p.name:<9} [{src:<12} conf={conf:.2f} peer={'Y' if peer else 'N'}]")
        print(f"              {ans[:120]}")
        print()
        results.append({'name': p.name, 'port': p.port, 'answer': ans,
                        'source': src, 'confidence': conf})

    # Quality summary
    high = [r for r in results if r['confidence'] >= 0.7]
    low = [r for r in results if r['confidence'] < 0.3]
    print(f"\n  Summary: {len(high)} high-confidence, {len(low)} low-confidence")


# ═══════════════════════════════════════════════════════════════════════
# Night school — cross-learning session
# ═══════════════════════════════════════════════════════════════════════

def cmd_night(rounds: int = 3, seed: int = 0):
    """
    Night cycle. For a number of rounds:
      - Each client picks a topic from its plan
      - Asks a random peer about it
      - Records what comes back
    Loop prevention handled at ask() level (TTL + seen-set).
    """
    print(f"\n━━━ Night school ━━━")
    print(f"  rounds: {rounds}, seed: {seed}\n")
    rng = random.Random(seed or time.time())

    up_ports = [p.port for p in PERSONAS if port_up(p.port)]
    if len(up_ports) < 2:
        print("  need at least 2 clients up")
        return

    stats = {'queries': 0, 'answered': 0, 'peer_hits': 0,
             'cloud_hits': 0, 'loops_prevented': 0}

    for round_i in range(1, rounds + 1):
        print(f"  ─── Round {round_i} ───")
        for p in PERSONAS:
            if not port_up(p.port):
                continue
            # pick a topic from its plan
            plan = _get(f"http://127.0.0.1:{p.port}/plan").get('plan', [])
            if not plan:
                continue
            topic = rng.choice(plan)['topic']
            # formulate a question around that topic
            question = _make_question(topic, p, rng)
            # ask self (will auto-delegate to peer if needed)
            d = _post(f"http://127.0.0.1:{p.port}/ask",
                      {'q': question}, timeout=60)
            stats['queries'] += 1
            if d.get('answer'):
                stats['answered'] += 1
            if d.get('peer_hit'):
                stats['peer_hits'] += 1
            if d.get('cloud_hit'):
                stats['cloud_hits'] += 1
            if d.get('loop_detected'):
                stats['loops_prevented'] += 1
            conf = d.get('confidence', 0.0)
            src = d.get('source', '?')
            print(f"    {p.name:<9} → asked about '{topic:<15}' "
                  f"[src={src:<10} conf={conf:.2f}]")

        # Small pause between rounds to let logs flush
        time.sleep(0.3)

    print(f"\n  stats: {json.dumps(stats)}")


def _make_question(topic: str, persona, rng) -> str:
    """Compose a question in the persona's language about a topic."""
    if persona.lang.startswith('he'):
        templates = [
            f"מה זה {topic}?",
            f"ספר לי על {topic}",
            f"למה {topic} חשוב?",
            f"איך עובד {topic}?",
        ]
    else:
        templates = [
            f"What is {topic}?",
            f"Tell me about {topic}",
            f"Why does {topic} matter?",
            f"How does {topic} work?",
        ]
    return rng.choice(templates)


# ═══════════════════════════════════════════════════════════════════════
# CLI
# ═══════════════════════════════════════════════════════════════════════

def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest='cmd', required=True)

    p = sub.add_parser('spawn')
    p.add_argument('n', type=int, nargs='?', default=len(PERSONAS))

    sub.add_parser('status')

    p = sub.add_parser('ask')
    p.add_argument('port', type=int)
    p.add_argument('question')

    p = sub.add_parser('roundtable')
    p.add_argument('question')

    p = sub.add_parser('night')
    p.add_argument('--rounds', type=int, default=3)
    p.add_argument('--seed', type=int, default=0)

    sub.add_parser('stop')

    args = ap.parse_args()
    if args.cmd == 'spawn':
        cmd_spawn(args.n)
    elif args.cmd == 'status':
        cmd_status()
    elif args.cmd == 'ask':
        cmd_ask(args.port, args.question)
    elif args.cmd == 'roundtable':
        cmd_roundtable(args.question)
    elif args.cmd == 'night':
        cmd_night(args.rounds, args.seed)
    elif args.cmd == 'stop':
        cmd_stop()


if __name__ == '__main__':
    main()
