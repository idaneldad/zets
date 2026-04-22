#!/usr/bin/env python3
"""
ZETS HTTP API — thin REST bridge for the zets-gui chat UI.

Exposes 4 endpoints that call ZETS Rust binaries:

  GET  /api/health          → status, snapshots count, binaries
  GET  /api/snapshots       → list of {name, atoms, edges}
  POST /api/query           → {question, snapshot?} → {seeds, top_atoms, trust}
  POST /api/ingest          → {text, name, base?} → ingestion stats

Port 3147. Behind nginx at /zets/api/.

Design: stateless subprocess calls to the Rust binaries. No long-lived
graph in memory — each request spawns a fresh process. Slower per-call
(200-500ms) but simpler and no need to keep a Rust server running.
Phase 8 will replace this with a native Rust HTTP server.
"""

import json
import os
import subprocess
from pathlib import Path
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse

ZETS_ROOT = Path(os.getenv("ZETS_ROOT", "/home/dinio/zets"))
ZETS_BIN = ZETS_ROOT / "target" / "release"
ZETS_SNAPSHOTS = ZETS_ROOT / "data" / "baseline"
ZETS_BENCHMARKS = ZETS_ROOT / "data" / "benchmarks"

DEFAULT_SNAPSHOT = os.getenv("ZETS_DEFAULT_SNAPSHOT", "wiki_all_domains_v1")
HTTP_PORT = int(os.getenv("ZETS_HTTP_PORT", "3147"))


def list_snapshots():
    out = []
    for atoms in sorted(ZETS_SNAPSHOTS.glob("*.atoms")):
        name = atoms.stem
        manifest = atoms.with_suffix(".manifest.json")
        info = {"name": name}
        if manifest.exists():
            try:
                d = json.loads(manifest.read_text())
                info["atoms"] = d.get("atoms")
                info["edges"] = d.get("edges")
            except Exception:
                pass
        info["size_bytes"] = atoms.stat().st_size
        out.append(info)
    return out


def health():
    try:
        g = subprocess.run(["git", "log", "-1", "--format=%h %s"],
                           cwd=str(ZETS_ROOT), capture_output=True,
                           text=True, timeout=10)
        commit = g.stdout.strip() if g.returncode == 0 else "?"
    except Exception:
        commit = "?"
    return {
        "status": "ok",
        "commit": commit,
        "snapshots": len(list(ZETS_SNAPSHOTS.glob("*.atoms"))),
        "default_snapshot": DEFAULT_SNAPSHOT,
    }


def query(question: str, snapshot: str = None):
    """Answer a free-text question by finding top activated atoms in the graph.

    Current implementation uses benchmark-runner with a single on-the-fly
    multiple-choice question. The response shows what ZETS 'thinks' without
    forcing a choice — as if the user asked a real question.
    """
    snap = snapshot or DEFAULT_SNAPSHOT
    # Build a temporary 1-question JSONL. Use placeholder choices to get the
    # scoring pipeline to run — and then we return the top candidates as the
    # actual answer content.
    tmp = Path("/tmp") / f"zets_q_{os.getpid()}.jsonl"
    one_q = {
        "id": "q",
        "text": question,
        "choices": ["_a", "_b", "_c", "_d"],
        "expected": "A",
        "category": "user_query",
    }
    tmp.write_text(json.dumps(one_q, ensure_ascii=False) + "\n")

    try:
        r = subprocess.run(
            [str(ZETS_BIN / "benchmark-runner"),
             "--snapshot", snap,
             "--questions", str(tmp)],
            cwd=str(ZETS_ROOT),
            capture_output=True, text=True, timeout=60,
        )
        # Parse stdout for top candidates. The benchmark-runner's own output
        # doesn't directly expose candidates; for now return the raw stdout
        # tail so the UI can show it. Phase 8 will replace this with a
        # proper /query endpoint in Rust.
        output = r.stdout[-3000:]
        return {
            "question": question,
            "snapshot_used": snap,
            "engine_output": output,
            "stderr": r.stderr[-500:] if r.stderr else "",
        }
    finally:
        try:
            tmp.unlink()
        except Exception:
            pass


def ingest(text: str, name: str = "mcp_live", base: str = "v1_bootstrap",
           source: str = "user_ingest"):
    if len(text) > 5_000_000:
        return {"error": "text too large (>5MB). Use stream-ingest for files."}
    tmp = Path("/tmp") / f"zets_ing_{os.getpid()}.txt"
    tmp.write_text(text, encoding="utf-8")
    try:
        r = subprocess.run(
            [str(ZETS_BIN / "ingest-corpus"),
             "--input", str(tmp),
             "--name", name,
             "--base", base,
             "--source", source],
            cwd=str(ZETS_ROOT),
            capture_output=True, text=True, timeout=300,
        )
        return {
            "ok": r.returncode == 0,
            "output": r.stdout[-2000:],
            "error": r.stderr[-500:] if r.stderr else None,
        }
    finally:
        try:
            tmp.unlink()
        except Exception:
            pass


class Handler(BaseHTTPRequestHandler):
    def _send(self, code: int, body: dict):
        payload = json.dumps(body, ensure_ascii=False).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(payload)))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()
        self.wfile.write(payload)

    def do_OPTIONS(self):
        self._send(204, {})

    def do_GET(self):
        path = urlparse(self.path).path
        if path in ("/health", "/api/health", "/"):
            return self._send(200, health())
        if path in ("/snapshots", "/api/snapshots"):
            return self._send(200, {"snapshots": list_snapshots()})
        self._send(404, {"error": f"unknown path: {path}"})

    def do_POST(self):
        path = urlparse(self.path).path
        length = int(self.headers.get("Content-Length", "0"))
        raw = self.rfile.read(length).decode("utf-8", errors="replace") if length else "{}"
        try:
            body = json.loads(raw) if raw else {}
        except json.JSONDecodeError:
            return self._send(400, {"error": "invalid JSON"})

        if path in ("/query", "/api/query"):
            q = body.get("question", "")
            if not q:
                return self._send(400, {"error": "question required"})
            return self._send(200, query(q, body.get("snapshot")))

        if path in ("/ingest", "/api/ingest"):
            t = body.get("text", "")
            if not t:
                return self._send(400, {"error": "text required"})
            return self._send(200, ingest(
                t,
                name=body.get("name", "mcp_live"),
                base=body.get("base", "v1_bootstrap"),
                source=body.get("source", "user_ingest"),
            ))

        self._send(404, {"error": f"unknown path: {path}"})

    def log_message(self, format, *args):
        # quieter log
        pass


if __name__ == "__main__":
    print(f"ZETS HTTP API | port {HTTP_PORT} | zets_root={ZETS_ROOT}")
    HTTPServer(("0.0.0.0", HTTP_PORT), Handler).serve_forever()
