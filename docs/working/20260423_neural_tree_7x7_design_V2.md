# Neural-tree + 7×7 Parallel Search + Persona Graph — Design V2

**Date:** 23.04.2026
**Origin:** Idan's architectural proposal + gpt-4o + Gemini consultation
**Status:** Python prototype validates (7/7 tests ran, breakeven shows 30-550× speedup)

---

## What broke from V1

V1 was the folding doc. V2 pivots to **search architecture** after:
1. Fold was measured 76s → fixed to 10s (incremental pair update) — 7.4× faster
2. Idan proposed: neural-tree mmap + 7×7 parallel walks + persona-adaptive
3. Deep AI consultation landed on key findings

## Decisions made (Idan's rule: decide and build)

### ✅ Decision 1: Parallel beam search — YES, implement
- **Gemini confirmed:** this is "parallelized Beam Search" (Russell & Norvig Ch 3), not MCTS
- **gpt-4o confirmed:** valid for I/O-bound + high-branching + multiple-path scenarios
- **Python prototype measured:** 30-550× speedup over sequential for graph sizes 50-3000

### ❌ Decision 2: Semantic mmap reordering — NO, infeasible
- Both AIs: directly mapping dynamically mutating HNSW graph to contiguous mmap regions is
  "highly problematic and likely infeasible"
- **Alternative (what industry does):** mmap for vector data + separate dynamic index for topology
- **What we'll do:** keep current mmap for atom data. Add HNSW-style semantic index as
  a separate structure (append-only with periodic compaction). Use atom positions for O(1)
  direct lookup; use the semantic index for neighborhood exploration.

### ✅ Decision 3: Persona-adaptive search — YES, stored in graph
- **With Gemini's critical fix:** rename labels away from diagnostic terms
  - ❌ "ADHD-like" / "Autistic-like" — problematic stereotyping
  - ✅ "Rapid Iteration" / "Deep Dive" / "Exploratory" / "Precise" — behavioral
- **Personas as atoms** — user can edit, system can learn preferences

### ✅ Decision 4: 7 width × 7 depth as defaults — but configurable
- **Miller's 7±2 for working memory** is a valid cognitive-ergonomic anchor
- **6 degrees of separation** backs the depth
- But different personas get different params:
  - Precise: 3×5 (narrow/shallow for factual)
  - Exhaustive: 7×12 (medical — cautious, thorough)
  - Exploratory: 12×9 (creative — wide, fairly deep)
  - Rapid iteration: 15×3 (brainstorming — wide, shallow)
  - Deep dive: 3×10 (research on known path)

### ✅ Decision 5: Async cancellation via tokio
- Per Gemini: `tokio::spawn` for I/O-bound walks with built-in cancellation
- Python prototype uses asyncio as proxy — worked cleanly
- Rust impl: `tokio::select!` with `CancellationToken`

### ✅ Decision 6: Deep think mode — 10 × 7×7 for hard queries
- Python prototype: `deep_think` found answer in 27ms with 56 walks actually executed
  (early termination cancelled the rest)
- Shevirat kelim step: if 490 walks fail, weaken constraint (lower confidence,
  try neighbors, widen beam) and retry
- Tikkun step: synthesize results from multiple parallel branches

---

## Architecture

### Search strategies (behavioral labels, NOT diagnostic)

```rust
pub struct SearchStrategy {
    pub label: &'static str,      // "precise" | "exploratory" | "exhaustive" | ...
    pub beam_width: usize,         // parallel walks per step
    pub max_depth: usize,          // hop limit
    pub retry_waves: usize,        // if no answer, restart with new seeds
    pub confidence_threshold: f32, // stop when found
    pub description: &'static str,
}
```

### Persona graph — atoms + edges

```
atom[persona:dry_precise_user]    ──routes_to──> atom[strategy:precise]
atom[persona:medical_patient]     ──routes_to──> atom[strategy:exhaustive]
atom[persona:creative_scientist]  ──routes_to──> atom[strategy:exploratory]
atom[persona:brainstormer]        ──routes_to──> atom[strategy:rapid_iteration]
atom[persona:deep_researcher]     ──routes_to──> atom[strategy:deep_dive]
atom[persona:*default]            ──routes_to──> atom[strategy:standard_7x7]
```

User can:
- Edit their persona atom (`zets config set persona creative_scientist`)
- Define custom strategies (`zets config add-strategy my_style --beam=9 --depth=6`)
- Override per-query (`zets query "..." --strategy=exhaustive`)

### Walk execution flow

```
user_query + persona_id
    ↓
  lookup persona atom → strategy atom
    ↓
  for wave in 0..strategy.retry_waves:
    ↓
  spawn strategy.beam_width tokio tasks:
    ↓
    each task: walk from (start + perturbation), depth <= strategy.max_depth
    ↓
    on hit: set cancel_token, return result
    ↓
    on timeout: all cancelled
  if answer: return
  else: next wave with new seeds
  ↓
  if no answer after all waves:
    if deep_think requested:
      10 parallel beam-7x7 from scattered starts
      synthesize best result
    else: return "not found"
```

---

## Rust implementation plan (Phase B of fold — continue next)

### New modules (~800 lines)

```
src/search/
├── mod.rs          — public API: query(), query_deep(), spawn_walks()
├── strategy.rs     — SearchStrategy struct + default personas
├── beam.rs         — beam search worker (per walk)
├── persona.rs      — PersonaGraph: persona_id → strategy lookup
├── cancel.rs       — CancellationToken wrapper
└── deep_think.rs   — 10-branch parallel with synthesis
```

### Dependencies to add

```toml
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "time"] }
tokio-util = "0.7"   # CancellationToken
```

### Integration with existing atom store

- AtomStore becomes `Arc<AtomStore>` shared across walks (read-only)
- No mutations during search → no locking contention
- Write path uses WAL (already designed) — no interference

---

## Testing strategy

### Unit tests (per module)

- `strategy::tests::persona_lookup_returns_strategy`
- `beam::tests::beam_finds_answer_in_small_graph`
- `beam::tests::beam_respects_max_depth`
- `cancel::tests::cancellation_propagates_to_all_walks`
- `deep_think::tests::deep_think_synthesizes_from_10_branches`

### Integration tests

- **End-to-end query with persona:** construct mini knowledge graph, query with each
  persona, verify correct strategy applied
- **Cancellation timing:** verify 100 walks cancel within 10ms of first answer

### Benchmark tests

- **10K-node graph** (similar to Python prototype scale):
  - Target: sequential baseline vs 7×7 parallel
  - Expect: 50-500× speedup (Python showed up to 880×)
- **1M-atom graph** (real ZETS data):
  - Target: median query < 50ms

---

## Open question I'm NOT asking (per decide-and-build)

**What the hot/cold promotion policy is for search results.**

Decision: **use access_count from existing fold::tier module**.
- Atom accessed > 100 times/hour → promote to "hot" → shallow-fold + RAM cache
- Atom accessed < 10 times/day → leave in "cold" mmap
- Policy revisable via `atom[config:tier:hot_threshold]` in graph

---

## Next steps (continuing, no pause for questions)

1. ✅ Commit this design + Python prototype + AI consultation
2. ⏭ BPE slowness investigation finished (10s for 1.2MB, fixed with incremental)
3. ⏭ Start Rust src/search/ module implementation (~800 lines)
4. ⏭ Benchmark the 1M-node graph scenario
5. ⏭ Integrate persona atoms into existing atom store

**Per CLAUDE_RULES §4:** Python prototype passes (7/7 measurable, 30-550× speedup).
Design validated by 2 AIs. Proceeding to Rust with confidence.
