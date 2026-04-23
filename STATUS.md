# STATUS.md — ZETS live state

> **Read-after:** [`CLAUDE.md`](./CLAUDE.md)
> **Update-policy:** append to session log, update the live sections. Don't delete history.

---

## 🎯 North Star (90 days)

Prove autonomy: Zetson (the "infant" ZETS) launches with an empty graph and,
within 90 days, reaches 20K+ HE atoms, 20K+ EN atoms, 5K+ cross-lingual links,
500+ learned procedures — **with zero LLM calls and no parent-injected knowledge.**

Full criteria: `docs/ZETS_LEARNING_HOW_IT_LEARNS.md` §success-criteria.

---

## 📦 Current build state

**Branch:** `main`
**Last commit:** `5ed3d3a` (23.04.2026) — *docs: How ZETS Learns + Zetson infant spec + 7 agent missions*
**Version tag:** (none yet — suggest `v0.1-spec-complete` once Zetson primitives land)

### What compiles / passes today
- [x] 223 Rust modules, ~45K LoC (claim from PRODUCT.md — re-verify)
- [x] **1,278 tests passing** (claim from PRODUCT.md — needs re-verify via `cargo test` next session)
- [x] Procedure-atom infrastructure (`src/procedure_atom.rs`, `src/procedure_template/`)
- [x] Learning layer with provenance (`src/learning_layer.rs`)
- [x] Capability runtime (`src/capability_runtime/`)
- [x] Morphology for HE/AR/EN + 12 other languages (`src/morphology/`)
- [x] Canonization engine (`src/canonization/`)
- [x] Calibration harness (`src/benchmark/calibration/`)
- [x] Preference store (`src/preferences/`)
- [x] MCP server + HTTP API (`mcp/`)

### What's spec-only, not yet implemented
- [ ] **Zetson infant binary** — YAML seed exists (`config/zetson-infant.yaml`),
      code does not. 12 agent missions queued (P-A..P-L, P-M for code quality).
- [ ] **HTTP fetch primitive (P-A)** — mission written, not built.
- [ ] **HTML parser primitive (P-B)** — mission written, not built.
- [ ] **20 initial procedure atoms (P-C)** — mission written. Procedures in
      graph form are ~5% of what's needed.
- [ ] **Learning loop executor (P-D)** — mission written, not built.
- [ ] **Seed loader (P-E)** — mission written, not built.
- [ ] **Observability dashboard (P-F)** — mission written, not built.
- [ ] **Zetson first-day demo (P-G)** — mission written, not built.
- [ ] **Math procedures pack (P-H)** — audited, not specced yet.
- [ ] **NL I/O pack (P-I)** — audited, not specced yet.
- [ ] **Cross-lingual pack (P-J)** — audited, not specced yet.
- [ ] **Benchmarks integration (P-K)** — audited, not specced yet.
- [ ] **Source registry & trust (P-L)** — audited, not specced yet.
- [ ] **Code quality audit (P-M)** — new, queued for Claude Code on ddev.
- [ ] **Image understanding without LLM (P-N)** — added 23.04.2026 session #2.
- [ ] **Speech without LLM (P-O)** — added 23.04.2026 session #2.
- [ ] **Provenance database (P-P)** — added 23.04.2026 session #2 (client recovery).
- [ ] **Video understanding (P-Q)** — added 23.04.2026 session #2.

---

## 🔧 In-flight work

| Who | Branch | Scope | Started | Status |
|---|---|---|---|---|
| Claude (chat via Cortex MCP) | `main` (docs only, low-risk) | Memory infra + audit + new missions | 23.04.2026 18:00 IDT | **active** |

---

## ⏭️ Next actions queue (priority order)

1. **[this session]** Land memory infrastructure (5 files) + AGI_ROADMAP update
   + PRODUCT.md storage-engine addendum + 5 new missions (P-M..P-Q). **in progress**
2. **[Claude Code, non-interactive, this session]** Run mission **P-M**
   (code quality audit: `cargo check`, `cargo clippy`, dead-code detection,
   duplication → graph-gap mapping). Produces `docs/CODE_QUALITY_REPORT.md`.
3. **[Claude Code, next]** Run **P-A** + **P-B** in parallel (primitives).
4. **[Claude Code, then]** **P-C**: procedure loader + 20 initial procedures.
5. **[Claude Code, then]** **P-D..P-G**: loop, seed, observability, demo.
6. **[research slot]** P-N (images), P-O (speech), P-P (provenance DB), P-Q (video).
7. **[later]** P-K benchmarks vs LLM suite.

---

## 🚧 Blockers

- **Zetson cannot be built yet** — depends on P-A..P-E.
- **PRODUCT.md rendering error on GitHub** — verified file is valid
  (HTTP 200 via GitHub markdown API). Transient. Revisit if persists.

---

## 📊 Claim-vs-measured ledger

Numbers quoted for ZETS — with honest status:

| Claim | Source | Status |
|---|---|---|
| 1,278 tests passing, 0 failures | `docs/PRODUCT.md` | [claimed] — re-verify with `cargo test` (mission P-M) |
| Rust ~45K LoC | PRODUCT.md | [claimed] — `tokei` or `cloc` to measure |
| 2.6 MB RAM | PRODUCT.md | [claimed] — no reproducible methodology in repo |
| 80.8 µs latency | PRODUCT.md | [claimed] — unclear which op, which hardware |
| HumannessScore 0.48 | PRODUCT.md + PARALLEL_WORK.md | [claimed] — metric spec unclear |
| 144,670 concepts loaded | AGI_ROADMAP.md | [claimed] — mission P-M to verify |

**Rule:** unverified claims stay `[claimed]` until reproducible. No investor brief
ships a `[claimed]` number stripped of its tag.

---

## 📋 Session log

Every session appends here. Most recent on top. Never delete.

### Session 2026-04-23 #2 — Memory & context infrastructure + missions expansion
- **Agent:** Claude (Anthropic chat via Cortex MCP shell_run on ddev)
- **Started:** 23.04.2026 ~18:00 IDT
- **Scope:**
  - Added 5 memory-infrastructure files (CLAUDE.md, STATUS.md, VISION_VS_REALITY, CLAUDE_ACTIONS_AUDIT, DECISIONS_LOG)
  - Cleaned repo (.gitignore for data/autonomous/, data/wikipedia_dumps/ [17GB])
  - Committed 5 unused-import lint fixes
  - Updated AGI_ROADMAP.md with status-as-of-today
  - Added storage-engine addendum to PRODUCT.md (dense graph DB, client-recovery)
  - Added missions P-M (code quality), P-N (images), P-O (speech), P-P (provenance DB), P-Q (video)
  - Invoked Claude Code (via `claude -p` on server) to execute P-M
- **Branch:** `main` (docs + cleanup only — low risk)
- **Status:** **active**

### Session 2026-04-23 #1 — ZETS Learning spec + Zetson seed + 7 missions
- **Agent:** Claude (Cortex MCP, server-side)
- **Ended:** 23.04.2026 ~17:36 IDT with commit `5ed3d3a`
- **Output:** 3 docs (ZETS_LEARNING_HOW_IT_LEARNS.md, ZETSON_AGENT_MISSIONS.md,
  config/zetson-infant.yaml).
- **Status:** **handed off to session #2**

### Earlier sessions
See `docs/DECISIONS_LOG.md` for per-decision history before 23.04.2026.

---

*Last edit: 23.04.2026 · Next mandatory update: end of current session.*
