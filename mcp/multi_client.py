#!/usr/bin/env python3
"""
Multi-instance ZETS prototype — launch N ZETS clients, each a distinct persona.

Idea: proves D3 (multi-tenant) + D4 (edge/cloud sync) without needing real
phones/chips. Each client is a separate Python HTTP process that:
  - Maintains its own persona atoms (name, interests, traits)
  - Has its own AtomStore-lite (just a JSON file for now)
  - Queries can hit its local store first, then fall back to the "cloud"
    (our existing zets_http_api.py on port 3147)
  - Keeps a sync log of (timestamp, query, hit_local, hit_cloud)

Each client listens on a port 3251..3260 by default.

This is NOT a full ZETS edge device — it's a testbed to exercise the
sync/discovery/persona protocol before committing it to Rust.

Run:
  python3 mcp/multi_client.py spawn 10          # launch 10 clients
  python3 mcp/multi_client.py status            # list running
  python3 mcp/multi_client.py ask 3251 "hello"  # send query to client 3251
  python3 mcp/multi_client.py stop              # stop all
"""

import argparse
import json
import os
import signal
import socket
import sys
import time
import urllib.request
import urllib.error
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from threading import Thread

# ─── Paths ──────────────────────────────────────────────────────────────
ZETS_ROOT = Path(os.environ.get('ZETS_ROOT', '/home/dinio/zets'))
CLIENTS_DIR = ZETS_ROOT / 'data' / 'clients'
PID_DIR = ZETS_ROOT / 'mcp' / 'logs' / 'clients'

CLOUD_URL = os.environ.get('ZETS_CLOUD_URL', 'http://127.0.0.1:3147/api')

# ─── 10 default personas ─────────────────────────────────────────────────
PERSONAS = [
    {"port": 3251, "name": "Shai",   "role": "student",   "interests": ["art", "literature"], "lang": "he"},
    {"port": 3252, "name": "Ben",    "role": "teenager",  "interests": ["gaming", "sports"],  "lang": "he"},
    {"port": 3253, "name": "Or",     "role": "child",     "interests": ["animals", "space"],  "lang": "he"},
    {"port": 3254, "name": "Yam",    "role": "toddler",   "interests": ["songs", "colors"],   "lang": "he"},
    {"port": 3255, "name": "Roni",   "role": "parent",    "interests": ["family", "music"],   "lang": "he"},
    {"port": 3256, "name": "Idan",   "role": "architect", "interests": ["zets", "kabbalah"],  "lang": "he"},
    {"port": 3257, "name": "Alex",   "role": "employee",  "interests": ["sales", "promo"],    "lang": "en"},
    {"port": 3258, "name": "Maya",   "role": "designer",  "interests": ["ui", "color"],       "lang": "he"},
    {"port": 3259, "name": "Noam",   "role": "developer", "interests": ["rust", "graphs"],    "lang": "en"},
    {"port": 3260, "name": "Tamar",  "role": "manager",   "interests": ["team", "product"],   "lang": "he"},
]


# ════════════════════════════════════════════════════════════════════════
# CLIENT — single-persona HTTP server
# ════════════════════════════════════════════════════════════════════════

class ClientState:
    def __init__(self, persona: dict):
        self.persona = persona
        self.name = persona['name']
        self.port = persona['port']
        self.store_file = CLIENTS_DIR / f"{self.name.lower()}.json"
        self.sync_log: list = []
        self.load()

    def load(self):
        if self.store_file.exists():
            try:
                d = json.loads(self.store_file.read_text(encoding='utf-8'))
                self.atoms = d.get('atoms', {})
                self.edges = d.get('edges', [])
                self.sync_log = d.get('sync_log', [])
                return
            except Exception:
                pass
        # bootstrap from persona
        self.atoms = {
            "name": self.persona['name'],
            "role": self.persona['role'],
            "lang": self.persona['lang'],
        }
        for i, interest in enumerate(self.persona['interests']):
            self.atoms[f"interest_{i}"] = interest
        self.edges = [
            {"from": "self", "to": f"interest_{i}", "rel": "likes"}
            for i in range(len(self.persona['interests']))
        ]
        self.save()

    def save(self):
        CLIENTS_DIR.mkdir(parents=True, exist_ok=True)
        self.store_file.write_text(
            json.dumps({
                'persona': self.persona,
                'atoms': self.atoms,
                'edges': self.edges,
                'sync_log': self.sync_log[-100:],  # keep last 100
            }, ensure_ascii=False, indent=2),
            encoding='utf-8',
        )

    def query_local(self, q: str) -> dict:
        """Try to answer from persona atoms + edges."""
        ql = q.lower()
        hits = []
        for k, v in self.atoms.items():
            if isinstance(v, str) and (ql in v.lower() or v.lower() in ql):
                hits.append({"atom": k, "value": v, "kind": "persona_match"})
        if ql in ("who are you", "מי אתה", "מה שמך", "what's your name"):
            hits.append({"atom": "name", "value": self.atoms['name'],
                        "kind": "identity"})
        return {"local_hits": hits}

    def query_cloud(self, q: str) -> dict:
        """Fall back to server."""
        try:
            body = json.dumps({"question": q}).encode('utf-8')
            req = urllib.request.Request(f"{CLOUD_URL}/query", data=body, method='POST')
            req.add_header('Content-Type', 'application/json')
            with urllib.request.urlopen(req, timeout=30) as r:
                return json.loads(r.read())
        except Exception as e:
            return {"error": str(e)}

    def ask(self, q: str) -> dict:
        t0 = time.time()
        local = self.query_local(q)
        used_cloud = False
        cloud = None
        if not local.get('local_hits'):
            cloud = self.query_cloud(q)
            used_cloud = True

        entry = {
            "ts": time.time(),
            "q": q,
            "hit_local": bool(local.get('local_hits')),
            "hit_cloud": used_cloud,
            "elapsed_ms": round((time.time() - t0) * 1000, 1),
        }
        self.sync_log.append(entry)
        self.save()

        return {
            "client": self.name,
            "question": q,
            "local": local,
            "cloud": cloud if used_cloud else None,
            "used": "cloud" if used_cloud else "local",
            "elapsed_ms": entry['elapsed_ms'],
        }


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
                    'persona': state.persona,
                    'atoms': len(state.atoms),
                    'edges': len(state.edges),
                    'queries_served': len(state.sync_log),
                })
            if self.path == '/state':
                return self._send(200, {
                    'atoms': state.atoms,
                    'edges': state.edges,
                    'sync_log': state.sync_log[-20:],
                })
            return self._send(404, {'error': 'unknown path'})

        def do_POST(self):
            length = int(self.headers.get('Content-Length', '0'))
            raw = self.rfile.read(length).decode('utf-8', errors='replace')
            try:
                body = json.loads(raw) if raw else {}
            except json.JSONDecodeError:
                return self._send(400, {'error': 'invalid JSON'})
            if self.path in ('/ask', '/api/ask'):
                q = body.get('q') or body.get('question', '')
                if not q:
                    return self._send(400, {'error': 'q required'})
                return self._send(200, state.ask(q))
            return self._send(404, {'error': 'unknown path'})

        def log_message(self, *a):  # quiet
            pass
    return H


def run_client(persona: dict):
    state = ClientState(persona)
    handler = make_handler(state)
    srv = HTTPServer(('127.0.0.1', persona['port']), handler)
    print(f"[{state.name}] listening on :{state.port}", flush=True)
    srv.serve_forever()


# ════════════════════════════════════════════════════════════════════════
# CLI
# ════════════════════════════════════════════════════════════════════════

def port_up(port: int) -> bool:
    try:
        with socket.create_connection(('127.0.0.1', port), timeout=0.3):
            return True
    except OSError:
        return False


def cmd_spawn(n: int):
    PID_DIR.mkdir(parents=True, exist_ok=True)
    count = 0
    for persona in PERSONAS[:n]:
        if port_up(persona['port']):
            print(f"  ✓ {persona['name']} :{persona['port']}  already up")
            continue
        pid = os.fork()
        if pid == 0:
            # child — serve forever
            try:
                run_client(persona)
            except KeyboardInterrupt:
                pass
            os._exit(0)
        else:
            (PID_DIR / f"{persona['name'].lower()}.pid").write_text(str(pid))
            time.sleep(0.1)
            if port_up(persona['port']):
                print(f"  ✓ {persona['name']} :{persona['port']}  (pid {pid})")
                count += 1
            else:
                print(f"  ✗ {persona['name']} :{persona['port']}  failed")
    print(f"\n  spawned {count} new clients")


def cmd_status():
    print(f"{'client':<10} {'port':<6} {'status':<10} {'atoms':<6} {'queries':<8}")
    print("─" * 52)
    for persona in PERSONAS:
        if port_up(persona['port']):
            try:
                with urllib.request.urlopen(f"http://127.0.0.1:{persona['port']}/health",
                                            timeout=2) as r:
                    d = json.loads(r.read())
                print(f"{persona['name']:<10} {persona['port']:<6} {'UP':<10} "
                      f"{d.get('atoms',0):<6} {d.get('queries_served',0):<8}")
            except Exception as e:
                print(f"{persona['name']:<10} {persona['port']:<6} {'ERR':<10}  ({e})")
        else:
            print(f"{persona['name']:<10} {persona['port']:<6} {'DOWN':<10}")


def cmd_ask(port: int, question: str):
    try:
        body = json.dumps({'q': question}, ensure_ascii=False).encode('utf-8')
        req = urllib.request.Request(f"http://127.0.0.1:{port}/ask",
                                     data=body, method='POST')
        req.add_header('Content-Type', 'application/json')
        with urllib.request.urlopen(req, timeout=60) as r:
            d = json.loads(r.read())
        print(json.dumps(d, ensure_ascii=False, indent=2))
    except Exception as e:
        print(f"ERR: {e}")


def cmd_stop():
    stopped = 0
    for persona in PERSONAS:
        pid_file = PID_DIR / f"{persona['name'].lower()}.pid"
        if pid_file.exists():
            try:
                pid = int(pid_file.read_text())
                os.kill(pid, signal.SIGTERM)
                print(f"  ✓ stopped {persona['name']} (pid {pid})")
                stopped += 1
            except ProcessLookupError:
                print(f"  - {persona['name']} already stopped")
            except Exception as e:
                print(f"  ✗ {persona['name']}: {e}")
            pid_file.unlink(missing_ok=True)
    print(f"\n  stopped {stopped} clients")


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest='cmd', required=True)

    p = sub.add_parser('spawn', help='launch N clients (default 10)')
    p.add_argument('n', type=int, nargs='?', default=10)

    sub.add_parser('status', help='list client status')

    p = sub.add_parser('ask', help='query a specific client')
    p.add_argument('port', type=int)
    p.add_argument('question')

    sub.add_parser('stop', help='stop all clients')

    args = ap.parse_args()
    if args.cmd == 'spawn':
        cmd_spawn(args.n)
    elif args.cmd == 'status':
        cmd_status()
    elif args.cmd == 'ask':
        cmd_ask(args.port, args.question)
    elif args.cmd == 'stop':
        cmd_stop()


if __name__ == '__main__':
    main()
