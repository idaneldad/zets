# Sprint B — Query Planner + Multi-Seed Retrieval

**Estimated time:** 1 week
**Branch:** `sprint-b-query`
**Risk level:** Medium — new code in hot path
**Dependencies:** Sprint A complete
**Blocks:** Sprint D

---

## Why this sprint exists

Currently: a query goes directly into `walk::forward_pass` with a single
seed synset. This works for trivial lookups but fails for:
- Multi-entity queries ("compare X and Y")
- Natural-language queries (not a direct synset reference)
- Context-dependent queries ("crown" meaning depends on session)

Sprint B adds a Query Planner that:
1. Parses query text
2. Extracts entities using surface → synset lookup
3. Resolves ambiguity using session context
4. Produces an ExecutionPlan with 1+ seeds

---

## Tasks

### Task B1 — Query struct
```rust
pub struct Query {
    pub text: String,
    pub language: LangCode,
    pub mode: ResponseMode,
    pub session_id: Option<SessionId>,
}
```

### Task B2 — Surface-to-synset lookup
Build a reverse map: surface form → list of candidate synsets.
Currently: no such structure. We have synset → surface via learning.

Add: `pub struct SurfaceIndex { map: HashMap<String, Vec<SynsetId>> }`
built at graph load time, updated on every add_entry.

### Task B3 — Entity extractor
```rust
pub fn extract_entities(query: &str, lang: LangCode, idx: &SurfaceIndex)
    -> Vec<EntityMatch>
{
    // Split query into tokens
    // For each token, lookup in SurfaceIndex
    // Return positions + candidate synsets
}

pub struct EntityMatch {
    pub surface: String,
    pub position: (usize, usize),
    pub candidates: Vec<SynsetId>,
}
```

### Task B4 — Ambiguity resolver (simple first)
If a token matches multiple synsets, pick one using:
- If session_id: use session context synsets (to be done properly in Sprint H)
- Else: pick highest-weight synset

For Sprint B: only implement the else-branch. Session-based disambig is
Sprint H.

### Task B5 — Multi-seed walk
Extend `walk::forward_pass` to accept multiple seeds:
```rust
pub fn multi_seed_walk(
    graph: &Graph,
    seeds: &[SynsetId],
    depth: u8,
) -> Vec<Candidate>
```

Strategy: run forward_pass from each seed, merge results, re-rank.

### Task B6 — ExecutionPlan struct
```rust
pub struct ExecutionPlan {
    pub entities: Vec<EntityMatch>,
    pub seeds: Vec<SynsetId>,
    pub walk_params: WalkParams,
    pub response_mode: ResponseMode,
}
```

### Task B7 — Tests
- Query with 1 entity → 1 seed plan
- Query with 2 entities → 2 seed plan
- Query with ambiguous word → deterministic resolution
- Multi-seed walk returns merged candidates

---

## Acceptance criteria

- [ ] 72 existing + at least 8 new tests pass
- [ ] `cargo build --release` succeeds
- [ ] Benchmark: plan+walk <10ms for typical queries
- [ ] No new dependencies

---

## Commit format

```
Sprint B: Query Planner + Multi-seed Retrieval

- Query, EntityMatch, ExecutionPlan data types
- SurfaceIndex: reverse lookup from surface form to synsets
- Entity extraction from natural-language text
- Basic ambiguity resolution (by weight; session-based deferred to Sprint H)
- multi_seed_walk extends walk engine for 2+ starting points

Tests: 77 -> 85 passing.
```
