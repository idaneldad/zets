# 0001 — Rust-only core, Python only as external tools

**Date:** 2026-04-22  
**Status:** Accepted  
**Supersedes:** —

## Context

ZETS started with a mix of Rust (core graph) and Python (mcp servers, autonomous crawlers, persona clients). Over time this created drift: knowledge logic appearing in Python lookup tables, dependencies on Python interpreters at runtime, deployment friction.

Idan's binding rule (recorded 2026-04-19): "Python is for external tools only that talk to Rust via HTTP. No core knowledge or core logic in Python."

## Decision

The ZETS **core** lives in `src/*.rs`. Python is permitted ONLY for:

1. **`py_testers/`** — prototypes that validate a design before Rust implementation (per CLAUDE_RULES rule 4).
2. **`mcp/`** — external service tools (HTTP API, MCP wrapper) that communicate with Rust via HTTP only.
3. **`scripts/`** — one-shot ingestion utilities (Wikipedia dump processors, etc).

The Python in `mcp/` is **scheduled for replacement** by Rust as the equivalent Rust modules become production-ready (`src/http_server.rs` for `zets_http_api.py`, `src/persona.rs` walker for `zets_client.py`).

## Consequences

**Positive:**
- Single binary deployment (Rust)
- No Python interpreter required at runtime
- Type safety end-to-end
- No risk of "knowledge in Python dict" anti-pattern

**Negative:**
- Slower prototyping than Python
- Tree-sitter / LLM client libraries require Rust crates (occasionally less mature)
- `py_testers/` rule must be enforced — easy to skip the Rust port

## Mitigation of negatives

- CLAUDE_RULES rule 4 enforces Python-prototype-first → Rust-second
- All `py_testers/*.py` must have a corresponding `src/*.rs` ticket
- Pre-commit hook (TBD) checks no new `mcp/*.py` files added without justification
