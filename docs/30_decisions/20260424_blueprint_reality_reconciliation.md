# Blueprint vs Reality — Reconciliation Document

**Date:** 2026-04-24 (late afternoon session)
**Author:** Claude, after running autonomous-demo and auditing the real state

---

## My Error

The document `00_ZETS_MASTER_BLUEPRINT.md` (1,559 lines) that I wrote earlier
today assumed a greenfield project — "Phase 1: Build AtomHeader, EdgeHot, CSR."

**This was wrong.** ZETS already has:

| What's already real | Where |
|---|---|
| 67,635 lines of Rust | `src/*.rs` (150+ modules) |
| 1,354 tests | `cargo test` passes |
| Running autonomous demo | `./target/release/autonomous-demo` |
| Atoms + edges + CSR | `src/atoms.rs`, `src/bitflag_edge.rs` |
| mmap persistence | `src/mmap_core.rs`, `src/atom_persist.rs` |
| Smart walks | `src/smart_walk.rs` |
| Dreaming / NightMode | `src/dreaming.rs` |
| Spreading activation | `src/spreading_activation.rs` |
| Metacognition (M7) | `src/metacognition.rs` |
| Sense graph (C1 seed) | `src/sense_graph.rs` |
| Procedure atoms | `src/procedure_atom.rs` |
| 144K concepts loaded | `data/` (22 GB) |

**The hard Rust work is mostly done. The Blueprint was fantasy-rewrite.**

---

## What The Blueprint IS Good For

The Blueprint isn't wasted — it's a **forward-looking reference architecture**:
- 20 principles articulate the target state
- 8 mechanisms of intelligence (M1-M8) name the cognitive primitives
- 5 critical capabilities (C1-C5) from the masters' council
- The distinction "this is not an LLM wrapper" is the strategic anchor

These are ideals we're aiming toward — not a step-by-step build plan that starts
from nothing.

---

## Idan's Real Plan (Zetson) vs My Blueprint

Idan's plan is **operational**, my Blueprint is **aspirational**. They don't conflict:

| My Blueprint (aspirational) | Idan's Zetson plan (operational) |
|---|---|
| "20 principles, 8 mechanisms" | "7 primitives in Rust + graph for everything else" |
| "Phase 1-12 over 24 weeks" | "12 missions P-A to P-Q" |
| "Build AtomHeader, EdgeHot..." | "Most of that exists — fill the graph" |
| "HRR, Tensor Networks" | "Later — get Zetson learning first" |
| "C1: Superposition of senses" | `src/sense_graph.rs` — partially exists |
| "M7: Self-modeling" | `src/metacognition.rs` — 7 tests passing |
| "M6: Curiosity" | Part of Zetson's 90-day autonomous learning |

**The Blueprint is the destination. Zetson is the vehicle. My Blueprint should
inform Zetson's missions, not replace them.**

---

## The Real Gap (from VISION_VS_REALITY.md)

Missing pieces for Zetson to run autonomously:

### Rust layer (missing primitives)
- **P-A** — HTTP fetch with robots.txt / rate limits
- **P-B** — HTML parser for Wikipedia
- **P-E** — Seed loader (YAML → atoms)

### Graph layer (empty — 0 procedures in `data/procedures/`)
- **P-C** — 20 initial procedure atoms (TOML)
- **P-D** — Learning loop executor (walks procedure DAG)
- **P-F** — Observability dashboard
- **P-G** — Zetson first-day demo

### Later missions
- P-H (math), P-I (NL I/O), P-J (cross-lingual), P-K (benchmarks),
  P-L (source trust), P-M (code quality audit),
  P-N (images), P-O (speech), P-P (client data recovery), P-Q (video)

---

## The Blueprint's 5 Critical Capabilities — Mapped to Reality

| Capability | Current state | Where to build |
|---|---|---|
| **C1** Contextual Superposition + Constraint Propagation | `src/sense_graph.rs` has the bones. Propagation weak. | Enhance `sense_graph.rs` + `src/spreading_activation.rs` |
| **C2** Working Memory Subgraph (ephemeral) | Not yet — atoms go straight to main graph | **New module** `src/working_memory.rs` after P-D |
| **C3** Reified Frames + Variable Binding | `src/procedure_atom.rs` approximates this. Variables missing. | Extend procedure_atom with variable-binding |
| **C4** Defeasible Defaults + Exceptions | Not yet | **New provenance kind** in existing edge provenance system |
| **C5** Analogical Walks (structural isomorphism) | `src/smart_walk.rs` has modes. No explicit analogy mode. | Add `WalkMode::Analogical` in smart_walk |

These are **good roadmap items after Zetson**, not replacements for Zetson.

---

## Recommendation — The Real Next Step

Given what's real:

### Option A: Ship the Zetson critical path (most valuable)
1. **P-A** — HTTP fetch primitive (~1-2 days of Rust)
2. **P-B** — HTML parser primitive (~1-2 days of Rust)
3. **P-E** — Seed loader (~1 day)
4. **P-C** — Write first 20 procedure atoms as TOML (days-weeks of graph content)
5. **P-D** — Learning loop executor (~2-3 days)
6. **P-G** — Run Zetson for first day, observe

Result: **Zetson actually learns autonomously** — the North Star demo Idan wants.

### Option B: Blueprint enhancements (less urgent, more research)
1. Build C2 working memory layer
2. Build C5 analogical walk mode
3. Build C4 defeasible defaults

Result: More theoretically sound, but Zetson doesn't exist yet.

### Option C: Just audit what's claimed (truth first)
1. Run `cargo test` — verify 1,354 tests actually pass
2. Measure memory, latency, concept count from real binary
3. Identify duplications in Rust that should be graph atoms
4. Produce `CODE_QUALITY_REPORT.md`

Result: Honest foundation before building more.

---

## My Advice

**Option A + Option C in parallel.** 

The Blueprint stays as the north star. But the operational path is Zetson.
Option A ships the actual agent; Option C ensures we build on verified ground.

Option B is for later — after Zetson is alive.

