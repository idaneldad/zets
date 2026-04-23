# py_testers — Disposable Python Prototypes

Per **CLAUDE_RULES_v1** §4: Every engineering proposal from Idan gets a Python
prototype FIRST, before any Rust implementation.

## Purpose

- Prove architectural concepts in minutes (not hours of `cargo build`)
- Exercise the logic on real data
- Catch design bugs early (Python is forgiving)
- Once green → plan Rust implementation

## NOT part of the ZETS package

These files are development aids. They are:
- Not included in `cargo build` / `cargo install`
- Not shipped with binaries
- Never run in production

## Current testers

| File | Proves | Tests |
|------|--------|-------|
| `test_unified_node_v1.py` | Server + Client use ONE class, different config | 6/6 |
| `test_multi_interface_v1.py` | GUI + CLI + API + MCP share rate limit + policy | 8/8 |

## How to run

```bash
python3 py_testers/test_unified_node_v1.py
python3 py_testers/test_multi_interface_v1.py
```

## Workflow

```
Idan proposes X
    ↓
Claude: "let me prototype in Python first"
    ↓
py_testers/test_X_v1.py (~200 lines)
    ↓
Run, verify, show Idan the output
    ↓
If green → docs/working/YYYYMMDD_X_design_V1.md
    ↓
If Idan approves → src/X.rs Rust implementation
```
