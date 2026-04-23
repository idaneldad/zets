# ZETS — AI Consultation Primer

**ALWAYS prepend this file to any prompt sent to external AIs (Gemini, Groq,
GPT, Claude-via-API, etc.) before asking architectural or design questions.**

Without this primer, external AIs hallucinate solutions to problems that are
already solved, or recommend libraries/approaches that conflict with existing
code. Attaching this primer lets them recommend changes TO what exists, not
from zero.

When updating ZETS, this file MUST be updated too. If you change the module
structure, the data systems, or the invariants, update the primer first.

Last updated: 22.04.2026. Reflects commit 3341ab7 (main).

---

## 1. What ZETS is (one sentence)

A deterministic, symbolic, graph-based cognitive kernel written in Rust, with
provenance-tagged edges, no LLM in the loop, no neural nets, reproducible from
git, designed to run on mobile / laptop / server / edge chips.

## 2. What ZETS is NOT

- Not a neural net. Not an LLM. Not RAG-over-LLM.
- Not a graph database like Neo4j/Neptune. It's a compiled Rust binary that
  mmaps its own binary snapshot format.
- Not probabilistic. Same input → same output every time (verified: 10/10
  cross-process identical runs).
- Not trainable by gradient descent. "Learning" means ingesting text, extracting
  edges, tagging them with provenance. Deterministic and auditable.

## 3. The 5 provenance tags on every edge

```rust
enum EdgeSource {
    Asserted,     // taken as given from a corpus
    Observed,     // seen in data N times, aggregated
    Learned,      // distilled pattern promoted from Observed
    Hypothesis,   // inferred (e.g. by analogy), must have DerivedFrom chain
    Synced,       // came from another ZETS instance (future)
}
```

Every walk result must carry provenance. That's the core invariant.

## 4. Two parallel data systems (IMPORTANT)

ZETS currently has **two data models** that don't talk to each other.
Any recommendation that assumes "the graph" without naming which one is wrong.

### System A — `AtomStore` (production, runs benchmarks)

| | Value |
|---|---|
| Module | `src/atoms.rs`, `src/atom_persist.rs` |
| Core type | `Atom` + `AtomEdge` + `AtomStore` |
| File format | `.atoms` flat binary (custom, little-endian) |
| On-disk size | 158 MB for `wiki_all_domains_v1` (211,650 atoms, 13.2M edges) |
| RAM model | Full load into RAM at process start |
| Used by | `benchmark-runner`, `ingest-corpus`, `distill-demo` |
| Benchmarks | v1=90.6% / v2=93.8% / wiki=68.8% |
| Lazy/compressed | No |

### System B — `PieceGraph` / `ZetsEngine` (not hooked to benchmarks)

| | Value |
|---|---|
| Modules | `src/piece_graph.rs`, `src/pack.rs`, `src/mmap_core.rs`, `src/mmap_lang.rs`, `src/engine.rs` |
| Core types | `ConceptNode` + `PackedEdge` + `PieceGraph` + `ZetsEngine` |
| File format | `zets.core` (66 MB) + `zets.<lang>` × 16 languages (32 MB total) |
| On-disk size | 97 MB total |
| RAM model | mmap — OS loads 4KB pages only on access; 81 MB peak for full traversal |
| Used by | `engine_cli`, `mmap_read`, `pack_inventory` demos |
| Cross-language | YES — "dog" (en) and "גדול" (he) resolve to same concept |
| Lazy/compressed | Lazy YES (mmap). Compressed NO (raw bytes). |

### What this means for recommendations

Do NOT recommend:
- Compression schemes (zstd-seekable, etc.) — mmap + raw is fast enough for
  the ≤1GB scale we're at.
- Louvain-style clustering — per-language split already exists in System B;
  domain split not needed yet.
- "Build a per-language pack system" — already built in System B.

DO make recommendations about:
- How to merge A and B (the actual #1 problem).
- Whether to extend A's ingestion to emit System B packs.
- Whether to retire one in favor of the other.
- Specific algorithms for category tagging at ingestion time.

## 5. Existing module inventory (17K lines of Rust)

Production modules:
- `atoms.rs`, `atom_persist.rs` — System A data + serialization
- `piece_graph.rs`, `piece_graph_loader.rs` — System B data + loader
- `pack.rs` — Binary pack format (writer)
- `mmap_core.rs`, `mmap_lang.rs` — mmap-based readers
- `engine.rs` — `ZetsEngine` facade (MmapCore + MmapLangPack + WAL)
- `wal.rs` — Write-Ahead Log for learned updates
- `crypto.rs`, `encrypted_installer.rs` — At-rest encryption option
- `benchmarks.rs` — Benchmark runner + JSONL format + category scoring
- `smart_walk.rs`, `spreading_activation.rs` — The walk algorithms
- `ingestion.rs`, `edge_extraction.rs` — Text → graph pipeline
- `morphology.rs` — Per-language morphology rules
- `system_graph.rs` — Homoiconic routes + bytecode VM
- `ethics_core.rs` — Refusal / policy layer
- `verify.rs` — Claim verification against snapshot (Track C product)
- `dreaming.rs`, `distill.rs` — Observed → Learned pattern promotion
- `prototype.rs`, `hopfield.rs` — Associative memory layers
- `persona.rs`, `scenario.rs`, `skills.rs` — Higher-level cognitive
- `planner.rs`, `appraisal.rs`, `metacognition.rs` — Meta-cognition
- `learning_layer.rs`, `meta_learning.rs` — Learning-rate controls
- `llm_adapter.rs`, `gemini_http.rs` — External LLM calls (for verify only)
- `session.rs`, `state_persist.rs` — Session state
- `testing_sandbox.rs` — Eval harness

Do not recommend building anything that duplicates these. If the question is
"how do I verify a claim against the graph", the answer starts with "use the
existing `verify.rs` Track C product".

## 6. Performance invariants (measured, not aspirational)

- **Cross-process determinism**: 10/10 identical runs verified.
- **Speed vs LLMs**: ×6338 on the 32-question benchmark.
- **Memory**: System A loads 2.5GB peak on wiki; System B mmaps at 81MB peak.
- **Pack open time**: 20ms for core, 1-14ms per language.
- **Walk latency**: ~0.2ms typical (smart_walk, warm). Cold page faults slower.
- **Regression tests**: 484 lib + 8 regression. Zero warnings.
- **Benchmark floors locked in `tests/regression_baseline.rs`**: v1≥29/32,
  v2≥30/32, wiki≥22/32.

A design recommendation that would reduce any of these is rejected by default.

## 7. Constraints — environmental

- Must run on mobile phone (4 GB RAM, ARM).
- Must run on laptop / server / edge chip.
- Must work fully offline.
- Must stay deterministic under all environments.
- Snapshot files must be reproducible-from-git (no non-deterministic hashes).

## 8. What's being built NOW (priority order)

1. **D1 — Merging A and B**. Decision between α (drop B), β (drop A),
   γ (bridge). Idan has not decided yet.
2. `pack_inventory` CLI — shipped 22.04.
3. ZETS MCP server (port 3145) + HTTP API (port 3147) + web GUI — shipped 22.04,
   not deployed via sudo yet.
4. Architecture Plan V2 — shipped 22.04, supersedes V1.

## 9. What's NOT being built (deferred)

All of these are real needs but don't have an immediate concrete use case:
- Domain packs (medicine / CS / geography / slang)
- Personal graphs / multi-tenant ACL
- Edge/Cloud sync protocol (no edge devices yet)
- Hypothesis edge persistence
- ZETS query language

## 10. How to query this primer

When asking an external AI:
1. Send this primer first (copy the whole thing).
2. Then send the actual question.
3. Demand the AI's response reference specific files/types from section 5 when
   making recommendations.
4. If the AI recommends something that duplicates existing code (violates
   section 5), re-ask with that violation called out explicitly.

## 11. Hebrew / English

- Hebrew script across the graph is a first-class citizen, not an afterthought.
- The existing `morphology.rs` handles Hebrew morphology.
- HE+EN core always resident. Other 14 languages lazy.
- User interface is Hebrew-first; code, logs, and specs are English.

## 12. "Idan is wrong" rules

If the AI thinks the user is wrong, it should say so directly. Do not flatter.
Do not hedge. Back the claim with a reference to a specific file or invariant.

If the AI cannot answer without inventing facts, it says "I need more info
about X — please include the source of Y in next message."
