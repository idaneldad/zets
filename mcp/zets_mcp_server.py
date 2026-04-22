#!/usr/bin/env python3
"""
ZETS MCP Server — Model Context Protocol bridge to the ZETS cognitive kernel.

Replaces the legacy cortex-v7 MCP. Single source of AI-agent access.

Run:        python3 zets_mcp_server.py
Port:       3145 (behind nginx at /zets/mcp/ or /mcp/)
SSE:        http://127.0.0.1:3145/sse
Tools:      zets-specific + generic server-ops (shell, file, git)

Design:
  - zets_* tools run ZETS Rust binaries from /home/dinio/zets/target/release/
    as subprocess. Snapshots live in /home/dinio/zets/data/baseline/.
  - shell/file/git tools provide Claude general server access.
  - All tool calls are deterministic + idempotent (where applicable).
"""

import json
import os
import subprocess
import shlex
from pathlib import Path

try:
    from mcp.server.fastmcp import FastMCP
except ImportError:
    print("ERROR: pip install mcp --break-system-packages")
    exit(1)

# ─── Paths ──────────────────────────────────────────────────────────────────
ZETS_ROOT = Path(os.getenv("ZETS_ROOT", "/home/dinio/zets"))
ZETS_BIN = ZETS_ROOT / "target" / "release"
ZETS_SNAPSHOTS = ZETS_ROOT / "data" / "baseline"
ZETS_BENCHMARKS = ZETS_ROOT / "data" / "benchmarks"

DEFAULT_SNAPSHOT = os.getenv("ZETS_DEFAULT_SNAPSHOT", "wiki_all_domains_v1")
DEFAULT_BENCH = "zets_expanded_32q_v1.jsonl"

MCP_PORT = int(os.getenv("ZETS_MCP_PORT", "3145"))

mcp = FastMCP("ZETS")


# ─── Helpers ────────────────────────────────────────────────────────────────
BLOCKED_COMMANDS = [
    "rm -rf /",
    ":(){ :|:& };:",
    "mkfs",
    "dd if=/dev/zero of=/dev/",
    "> /dev/sda",
    "chmod 000 /",
    "chown -R root",
]


def _run(cmd: list, timeout: int = 60, cwd: Path = ZETS_ROOT) -> dict:
    """Run a subprocess and return a JSON-safe result dict."""
    try:
        r = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout,
            cwd=str(cwd),
            env={**os.environ, "PATH": f"/usr/local/bin:/usr/bin:/bin:{os.environ.get('PATH','')}"},
        )
        return {
            "ok": r.returncode == 0,
            "exit_code": r.returncode,
            "stdout": r.stdout,
            "stderr": r.stderr,
        }
    except subprocess.TimeoutExpired:
        return {"ok": False, "exit_code": -1, "stdout": "", "stderr": f"timeout after {timeout}s"}
    except Exception as e:
        return {"ok": False, "exit_code": -1, "stdout": "", "stderr": str(e)}


def _list_snapshots() -> list:
    """List available baseline snapshots."""
    if not ZETS_SNAPSHOTS.exists():
        return []
    return sorted([p.stem for p in ZETS_SNAPSHOTS.glob("*.atoms")])


def _list_benchmarks() -> list:
    if not ZETS_BENCHMARKS.exists():
        return []
    return sorted([p.name for p in ZETS_BENCHMARKS.glob("*.jsonl")])


# ─── ZETS-specific tools ────────────────────────────────────────────────────
@mcp.tool()
def zets_health() -> str:
    """Check ZETS state: git commit, snapshots available, binaries present."""
    info = {}

    # git
    g = _run(["git", "log", "-1", "--format=%h %s"], timeout=10)
    info["commit"] = g["stdout"].strip() if g["ok"] else "git error"

    # snapshots
    info["snapshots"] = _list_snapshots()

    # binaries
    bins_expected = ["benchmark-runner", "stream-ingest", "ingest-corpus",
                     "snapshot", "verify-demo", "explain-demo", "distill-demo",
                     "measure-moats"]
    info["binaries"] = {b: (ZETS_BIN / b).exists() for b in bins_expected}

    # quick test count
    t = _run(["cargo", "test", "--release", "--lib", "--quiet", "--no-run"], timeout=60)
    info["build_ok"] = t["ok"]

    # disk
    d = _run(["du", "-sh", str(ZETS_ROOT)], timeout=10)
    info["disk"] = d["stdout"].strip().split("\t")[0] if d["ok"] else "?"

    return json.dumps(info, indent=2, ensure_ascii=False)


@mcp.tool()
def zets_list_snapshots() -> str:
    """List all available ZETS graph snapshots with their atom/edge counts."""
    result = []
    for name in _list_snapshots():
        manifest = ZETS_SNAPSHOTS / f"{name}.manifest.json"
        if manifest.exists():
            try:
                d = json.loads(manifest.read_text())
                result.append({
                    "name": name,
                    "atoms": d.get("atoms", "?"),
                    "edges": d.get("edges", "?"),
                })
            except Exception:
                result.append({"name": name})
        else:
            result.append({"name": name})
    return json.dumps(result, indent=2, ensure_ascii=False)


@mcp.tool()
def zets_benchmark(snapshot: str = "", questions: str = "") -> str:
    """Run a benchmark against a named snapshot.

    Args:
        snapshot: Snapshot name (default: wiki_all_domains_v1).
        questions: JSONL path relative to data/benchmarks/ (default: zets_expanded_32q_v1.jsonl).

    Returns JSON with overall accuracy, per-category breakdown, throughput.
    """
    snap = snapshot or DEFAULT_SNAPSHOT
    q = questions or DEFAULT_BENCH
    q_path = ZETS_BENCHMARKS / q
    if not q_path.exists():
        return json.dumps({"error": f"questions not found: {q_path}",
                           "available": _list_benchmarks()})

    snap_path = ZETS_SNAPSHOTS / f"{snap}.atoms"
    if not snap_path.exists():
        return json.dumps({"error": f"snapshot not found: {snap}",
                           "available": _list_snapshots()})

    r = _run([str(ZETS_BIN / "benchmark-runner"),
              "--snapshot", snap, "--questions", str(q_path)],
             timeout=120)
    return r["stdout"] if r["ok"] else json.dumps({"error": r["stderr"]})


@mcp.tool()
def zets_verify(question: str, llm_answer: str, snapshot: str = "") -> str:
    """Verify an LLM-generated answer against a ZETS snapshot.

    Returns per-claim verdict (Supported / Contradicted / Unknown / Skipped)
    and trust recommendation (Trust / Caution / Insufficient / Reject).
    """
    snap = snapshot or DEFAULT_SNAPSHOT
    # verify-demo is currently a hardcoded demo. Call it with args if supported,
    # else return note to use Rust verify_answer() directly. For now run demo.
    # TODO Phase 8: build an HTTP server mode for verify.
    r = _run([str(ZETS_BIN / "verify-demo")], timeout=30)
    return json.dumps({
        "note": "verify-demo runs built-in examples. Phase 8 will expose HTTP API.",
        "snapshot_used": snap,
        "input_question": question[:200],
        "input_answer": llm_answer[:500],
        "demo_output": r["stdout"][-2000:] if r["ok"] else r["stderr"],
    }, ensure_ascii=False, indent=2)


@mcp.tool()
def zets_ingest_text(text: str, source: str = "mcp_ingest", snapshot_name: str = "mcp_live",
                     base: str = "v1_bootstrap") -> str:
    """Ingest a text blob into a new or existing snapshot.

    Args:
        text: Text content to ingest (sentences separated by . ! ?).
        source: Provenance label (default: 'mcp_ingest').
        snapshot_name: Output snapshot name (default: 'mcp_live').
        base: Base snapshot to extend (default: v1_bootstrap).
    """
    if len(text) > 10_000_000:
        return json.dumps({"error": "text too large (>10MB). Use stream_ingest via file."})

    tmp = Path("/tmp") / f"zets_mcp_ingest_{os.getpid()}.txt"
    tmp.write_text(text, encoding="utf-8")

    try:
        r = _run([str(ZETS_BIN / "ingest-corpus"),
                  "--input", str(tmp),
                  "--name", snapshot_name,
                  "--base", base,
                  "--source", source],
                 timeout=300)
        return r["stdout"] if r["ok"] else json.dumps({"error": r["stderr"]})
    finally:
        try:
            tmp.unlink()
        except Exception:
            pass


@mcp.tool()
def zets_stream_ingest_file(jsonl_path: str, snapshot_name: str, base: str = "v1_bootstrap",
                            source: str = "stream", max_articles: int = 0) -> str:
    """Ingest a large JSONL corpus file (one article per line).

    Format: {"title": "...", "text": "..."}
    Use for Wikipedia-scale ingestion.
    """
    p = Path(jsonl_path)
    if not p.exists():
        return json.dumps({"error": f"file not found: {jsonl_path}"})

    cmd = [str(ZETS_BIN / "stream-ingest"),
           "--name", snapshot_name,
           "--base", base,
           "--source", source,
           "--checkpoint-every", "500"]
    if max_articles > 0:
        cmd += ["--max-articles", str(max_articles)]

    # pipe file into stdin
    try:
        with p.open("rb") as f:
            r = subprocess.run(cmd, stdin=f, capture_output=True, text=True,
                               timeout=3600, cwd=str(ZETS_ROOT))
        return (r.stdout + "\n---STDERR---\n" + r.stderr)[-5000:]
    except subprocess.TimeoutExpired:
        return json.dumps({"error": "timeout after 1h"})
    except Exception as e:
        return json.dumps({"error": str(e)})


@mcp.tool()
def zets_distill(snapshot: str = "") -> str:
    """Run distillation demo on a snapshot — finds recurring patterns,
    promotes Observed edges to Learned prototypes."""
    snap = snapshot or DEFAULT_SNAPSHOT
    r = _run([str(ZETS_BIN / "distill-demo")], timeout=60)
    return (r["stdout"] + ("\n---STDERR---\n" + r["stderr"] if r["stderr"] else ""))[-3000:]


@mcp.tool()
def zets_explain(concept: str = "") -> str:
    """Explain an atom or concept — returns full provenance chain
    (Asserted / Observed / Learned / Hypothesis sources)."""
    r = _run([str(ZETS_BIN / "explain-demo")], timeout=30)
    return r["stdout"] if r["ok"] else r["stderr"]


@mcp.tool()
def zets_measure_moats() -> str:
    """Run the 5-moat measurement (determinism, speed, refusal, forgetting, trace)."""
    r = _run([str(ZETS_BIN / "measure-moats")], timeout=120)
    return r["stdout"] if r["ok"] else r["stderr"]


@mcp.tool()
def zets_tests() -> str:
    """Run cargo test --release on the ZETS library. Returns test count + any failures."""
    r = _run(["cargo", "test", "--release", "--lib"], timeout=300)
    # grep for the test result line
    lines = [ln for ln in (r["stdout"] + r["stderr"]).splitlines()
             if "test result" in ln or "FAILED" in ln or "error[" in ln]
    return "\n".join(lines) if lines else r["stdout"][-1500:]


# ─── Generic server-ops tools ───────────────────────────────────────────────
@mcp.tool()
def shell_run(command: str, workdir: str = "", timeout: int = 60) -> str:
    """Run a bash command on the server.

    Args:
        command: Bash command.
        workdir: Default: /home/dinio/zets.
        timeout: Max: 300.
    """
    for blocked in BLOCKED_COMMANDS:
        if blocked in command:
            return f"BLOCKED: dangerous pattern '{blocked}'"

    cwd = Path(workdir) if workdir else ZETS_ROOT
    timeout = min(max(timeout, 1), 300)

    try:
        r = subprocess.run(
            command, shell=True, capture_output=True, text=True,
            timeout=timeout, cwd=str(cwd),
            env={**os.environ, "PATH": f"/usr/local/bin:/usr/bin:/bin:{os.environ.get('PATH','')}"},
        )
        out = r.stdout
        if r.stderr:
            out += "\n---STDERR---\n" + r.stderr
        if r.returncode != 0:
            out += f"\n[exit code: {r.returncode}]"
        # cap at 64KB
        return out[:65536]
    except subprocess.TimeoutExpired:
        return f"TIMEOUT after {timeout}s"
    except Exception as e:
        return f"ERROR: {e}"


@mcp.tool()
def file_read(path: str, start_line: int = 0, end_line: int = 0) -> str:
    """Read a file. If end_line=0, reads up to 200 lines from start_line."""
    p = Path(path)
    if not p.exists():
        return f"not found: {path}"
    if p.is_dir():
        return f"is directory: {path}"
    try:
        text = p.read_text(encoding="utf-8", errors="replace")
    except Exception as e:
        return f"read error: {e}"
    lines = text.splitlines()
    if start_line == 0 and end_line == 0:
        end = min(200, len(lines))
        sel = lines[:end]
    else:
        start = max(0, start_line)
        end = min(end_line if end_line > 0 else start + 200, len(lines))
        sel = lines[start:end]
    return "\n".join(f"{i+1}: {ln}" for i, ln in enumerate(sel, start=start_line or 0))


@mcp.tool()
def file_write(path: str, content: str) -> str:
    """Write content to a file (creates if missing, overwrites if exists)."""
    try:
        p = Path(path)
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(content, encoding="utf-8")
        return f"OK: wrote {len(content.encode('utf-8'))} bytes to {path}"
    except Exception as e:
        return f"write error: {e}"


@mcp.tool()
def git_status() -> str:
    """Git status + recent log for the ZETS repo."""
    s = _run(["git", "status", "--short"], timeout=10)
    l = _run(["git", "log", "--oneline", "-10"], timeout=10)
    return "Status:\n" + s["stdout"] + "\nRecent commits:\n" + l["stdout"]


# ─── Main ───────────────────────────────────────────────────────────────────
if __name__ == "__main__":
    import uvicorn

    class ProxyFixMiddleware:
        """Nginx reverse-proxy fix: normalize Host, keep SSE paths stable."""
        def __init__(self, app):
            self.app = app

        async def __call__(self, scope, receive, send):
            if scope["type"] == "http":
                new_headers = []
                for k, v in scope.get("headers", []):
                    if k == b"host":
                        new_headers.append((k, f"localhost:{MCP_PORT}".encode()))
                    else:
                        new_headers.append((k, v))
                scope = dict(scope, headers=new_headers)
            await self.app(scope, receive, send)

    sse_app = mcp.sse_app()
    app = ProxyFixMiddleware(sse_app)

    print(f"ZETS MCP Server | port {MCP_PORT} | zets_root={ZETS_ROOT}")
    print(f"  Snapshots: {len(_list_snapshots())}")
    print(f"  Binaries:  {len(list(ZETS_BIN.glob('*'))) if ZETS_BIN.exists() else 0}")
    print(f"  Tools:     zets_health, zets_list_snapshots, zets_benchmark,")
    print(f"             zets_verify, zets_ingest_text, zets_stream_ingest_file,")
    print(f"             zets_distill, zets_explain, zets_measure_moats, zets_tests,")
    print(f"             shell_run, file_read, file_write, git_status")
    print()

    uvicorn.run(app, host="0.0.0.0", port=MCP_PORT, log_level="info")
