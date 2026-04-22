#!/usr/bin/env python3
"""
multi_client_v3.py — orchestrator for 16 REAL Rust ZETS client-server instances.

Each client is a compiled Rust binary `zets_client_server` serving on its own port.
Replaces the Python prototype in multi_client_v2.py.

Usage:
  python3 mcp/multi_client_v3.py spawn               # start all 16
  python3 mcp/multi_client_v3.py status              # list
  python3 mcp/multi_client_v3.py ask 3251 "..."      # query one
  python3 mcp/multi_client_v3.py roundtable "..."    # ask everyone
  python3 mcp/multi_client_v3.py conversation 3251 3259 5  # Idan↔Michel, 5 turns
  python3 mcp/multi_client_v3.py quality-probe       # quality/depth metrics
  python3 mcp/multi_client_v3.py stop
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
from personas import PERSONAS  # noqa

ZETS_ROOT = Path(os.environ.get('ZETS_ROOT', '/home/dinio/zets'))
CLIENTS_DIR = ZETS_ROOT / 'data' / 'clients'
PERSONAS_DIR = ZETS_ROOT / 'data' / 'personas'
LOGS_DIR = ZETS_ROOT / 'mcp' / 'logs' / 'clients'
PIDS_DIR = LOGS_DIR / 'pids'

SERVER_BIN = ZETS_ROOT / 'target' / 'release' / 'zets_client_server'


def port_up(port: int, timeout: float = 0.3) -> bool:
    try:
        with socket.create_connection(('127.0.0.1', port), timeout=timeout):
            return True
    except OSError:
        return False


def _get(url: str, timeout: float = 5) -> dict:
    try:
        with urllib.request.urlopen(url, timeout=timeout) as r:
            return json.loads(r.read())
    except Exception as e:
        return {'error': str(e)}


def _post(url: str, body: dict, timeout: float = 30) -> dict:
    data = json.dumps(body, ensure_ascii=False).encode('utf-8')
    req = urllib.request.Request(url, data=data, method='POST')
    req.add_header('Content-Type', 'application/json')
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return json.loads(r.read())
    except Exception as e:
        return {'error': str(e)}


def build_peer_string(for_persona) -> str:
    """Comma-separated list of other personas, excluding self."""
    return ','.join(
        f"{p.port}:{p.name}" for p in PERSONAS if p.port != for_persona.port
    )


def cmd_spawn(n: int):
    if not SERVER_BIN.exists():
        print(f"  building {SERVER_BIN.name}...")
        r = subprocess.run(
            ['cargo', 'build', '--release', '--bin', 'zets_client_server'],
            cwd=str(ZETS_ROOT), capture_output=True, text=True,
        )
        if r.returncode != 0:
            print(f"  BUILD FAILED:\n{r.stderr[-500:]}")
            return

    PIDS_DIR.mkdir(parents=True, exist_ok=True)
    LOGS_DIR.mkdir(parents=True, exist_ok=True)

    spawned = 0
    for persona in PERSONAS[:n]:
        if port_up(persona.port):
            print(f"  ✓ {persona.name:<9} :{persona.port}  already up")
            continue

        atoms_path = CLIENTS_DIR / f"{persona.name.lower()}.atoms"
        persona_path = PERSONAS_DIR / f"{persona.name.lower()}.persona.json"

        if not atoms_path.exists():
            print(f"  ! {persona.name}: no atoms file {atoms_path}")
            continue

        log_out = LOGS_DIR / f"{persona.name.lower()}.out"
        log_err = LOGS_DIR / f"{persona.name.lower()}.err"

        with open(log_out, 'a') as lo, open(log_err, 'a') as le:
            proc = subprocess.Popen(
                [str(SERVER_BIN),
                 '--port', str(persona.port),
                 '--atoms', str(atoms_path),
                 '--persona', str(persona_path),
                 '--peers', build_peer_string(persona)],
                stdout=lo, stderr=le, stdin=subprocess.DEVNULL,
                start_new_session=True,
            )
        (PIDS_DIR / f"{persona.name.lower()}.pid").write_text(str(proc.pid))

        for _ in range(30):
            if port_up(persona.port):
                break
            time.sleep(0.1)
        if port_up(persona.port):
            print(f"  ✓ {persona.name:<9} :{persona.port}  pid {proc.pid}")
            spawned += 1
        else:
            print(f"  ✗ {persona.name:<9} :{persona.port}  FAILED")
    print(f"\n  spawned {spawned} new Rust clients")


def cmd_stop():
    stopped = 0
    for p in PERSONAS:
        pid_file = PIDS_DIR / f"{p.name.lower()}.pid"
        if pid_file.exists():
            try:
                pid = int(pid_file.read_text())
                os.kill(pid, signal.SIGTERM)
                stopped += 1
            except ProcessLookupError:
                pass
            except Exception as e:
                print(f"  ✗ {p.name}: {e}")
            pid_file.unlink(missing_ok=True)
    print(f"  stopped {stopped} Rust clients")


def cmd_status():
    print(f"  {'name':<9} {'he':<8} {'port':<6} {'status':<6} "
          f"{'atoms':<6} {'edges':<6} {'queries':<7} {'peer_hits':<9}")
    print("  " + "─" * 70)
    for p in PERSONAS:
        if port_up(p.port):
            d = _get(f"http://127.0.0.1:{p.port}/health")
            print(f"  {p.name:<9} {p.name_he:<8} {p.port:<6} {'UP':<6} "
                  f"{d.get('atoms',0):<6} {d.get('edges',0):<6} "
                  f"{d.get('queries_served',0):<7} {d.get('peer_hits',0):<9}")
        else:
            print(f"  {p.name:<9} {p.name_he:<8} {p.port:<6} {'DOWN':<6}")


def cmd_ask(port: int, question: str):
    r = _post(f"http://127.0.0.1:{port}/ask", {'q': question}, timeout=30)
    print(json.dumps(r, ensure_ascii=False, indent=2))


def cmd_roundtable(question: str):
    print(f"\n━━━ Roundtable: {question!r} ━━━\n")
    for p in PERSONAS:
        if not port_up(p.port):
            continue
        r = _post(f"http://127.0.0.1:{p.port}/ask", {'q': question}, timeout=30)
        ans = r.get('answer', '')
        src = r.get('source', '?')
        conf = r.get('confidence', 0.0)
        peer = r.get('peer_port')
        peer_str = f"→{peer}" if peer else "    "
        print(f"  {p.name:<9} [{src:<18} conf={conf:.2f} {peer_str}]")
        print(f"              {ans[:140]}")
        print()


def cmd_conversation(port_a: int, port_b: int, turns: int):
    """A ↔ B conversation over N turns. Each turn, one asks the other a question."""
    pa = next(p for p in PERSONAS if p.port == port_a)
    pb = next(p for p in PERSONAS if p.port == port_b)
    print(f"\n━━━ Conversation: {pa.name} ({port_a}) ↔ {pb.name} ({port_b}) ━━━\n")

    # Seed: A asks about something from its preferred topic list
    topic = (pa.style.preferred_topics or ['something interesting'])[0]
    templates_he = [f"מה אתה יודע על {topic}?", f"ספר לי על {topic}",
                    f"{topic} — זה מעניין אותך?"]
    templates_en = [f"What do you know about {topic}?",
                   f"Tell me about {topic}", f"{topic} — interested?"]
    q = random.choice(templates_he if pa.lang.startswith('he') else templates_en)
    speaker = pa
    listener = pb

    log = []
    for turn in range(1, turns + 1):
        # Speaker asks listener
        r = _post(f"http://127.0.0.1:{listener.port}/ask", {'q': q}, timeout=30)
        ans = r.get('answer', '')
        conf = r.get('confidence', 0.0)
        src = r.get('source', '?')
        print(f"  [T{turn}] {speaker.name} → {listener.name}: {q}")
        print(f"        {listener.name} says [{src} conf={conf:.2f}]: {ans[:200]}")
        print()
        log.append({'turn': turn, 'speaker': speaker.name, 'listener': listener.name,
                    'q': q, 'a': ans, 'conf': conf, 'src': src})
        # Listener generates follow-up in their preferred topic
        topic_l = (listener.style.preferred_topics or ['everyday'])[0]
        templates = [f"ו{topic_l}?", f"אבל מה עם {topic_l}?", f"what about {topic_l}?"]
        q = random.choice(templates)
        speaker, listener = listener, speaker

    # Quality summary
    hi = sum(1 for e in log if e['conf'] >= 0.6)
    lo = sum(1 for e in log if e['conf'] < 0.3)
    print(f"\n  Quality summary:")
    print(f"    high confidence ({hi}/{len(log)}), low ({lo}/{len(log)})")
    print(f"    avg confidence: {sum(e['conf'] for e in log) / max(1,len(log)):.2f}")


def cmd_quality_probe():
    """Run a suite of probe queries to measure conversation quality/depth."""
    probes = [
        ("identity_local", "מי אתה?",           "Testing self-identity"),
        ("identity_local_en", "Who are you?",    "Testing self-identity (EN)"),
        ("known_interest", "Tell me about music", "Testing topic match (Adrian should shine)"),
        ("unknown_deep", "מה זה מיכניקת הקוונטים?", "Testing unknown topic (all peers fail)"),
        ("peer_routing", "Tell me about football", "Testing peer routing (Ben should get it)"),
    ]
    print("\n━━━ Quality Probe ━━━\n")
    for pid, q, note in probes:
        print(f"  ── {pid}: {note} ──")
        print(f"     Q: {q!r}")
        confs = []
        sources = {'local': 0, 'peer': 0, 'miss': 0, 'loop': 0}
        for p in PERSONAS:
            if not port_up(p.port):
                continue
            r = _post(f"http://127.0.0.1:{p.port}/ask", {'q': q}, timeout=20)
            c = r.get('confidence', 0.0)
            src = r.get('source', '?')
            confs.append(c)
            if 'loop' in src: sources['loop'] += 1
            elif 'local' in src: sources['local'] += 1
            elif 'peer' in src: sources['peer'] += 1
            else: sources['miss'] += 1
        if confs:
            avg = sum(confs) / len(confs)
            hi = sum(1 for c in confs if c >= 0.6)
            print(f"     → avg_conf={avg:.2f}  high={hi}/{len(confs)}  "
                  f"sources={sources}")
        print()


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

    p = sub.add_parser('conversation')
    p.add_argument('port_a', type=int)
    p.add_argument('port_b', type=int)
    p.add_argument('turns', type=int, nargs='?', default=4)

    sub.add_parser('quality-probe')
    sub.add_parser('stop')

    args = ap.parse_args()
    globals()[f"cmd_{args.cmd.replace('-', '_')}"](**{
        k: v for k, v in vars(args).items() if k != 'cmd'
    })


if __name__ == '__main__':
    main()
