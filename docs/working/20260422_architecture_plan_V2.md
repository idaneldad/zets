# ZETS Architecture Plan V2 — עם ההבנות החדשות מהקוד האמיתי

**Date:** 22.04.2026
**Status:** Supersedes V1. V1 was overconfident in Gemini/Groq recommendations.
**Basis:** Actual code review of `src/pack.rs`, `src/mmap_core.rs`, `src/mmap_lang.rs`,
          `src/engine.rs`, `src/atom_persist.rs` — plus `pack_inventory` measurements.
**Commit base:** 4dfca97 on main (plus pack_inventory + V2 docs).

---

## Why V2

V1 locked decisions based on Gemini 2.5 Flash + Groq Llama-4-Scout recommendations,
but those AIs didn't know ZETS's actual internals. Code review revealed:

1. **`pack.rs` + `mmap_core` + `mmap_lang` already exist** — 97MB per-language
   layered pack format with 16 languages, opens in 20ms via mmap.
2. **There are two parallel data systems**:
   - `AtomStore` (runs benchmarks, 211K atoms, flat .atoms)
   - `PieceGraph` + `ZetsEngine` (pack-based, 144K concepts, 16 langs, mmap)
3. **mmap at the 100MB scale already gives lazy-page loading for free** — zstd-seekable
   would be premature optimization.
4. **Louvain clustering is not needed yet** — domain isolation can wait; language
   isolation is already handled by per-lang packs.

V2 revises V1's locked decisions.

---

## Decisions REVISED (V1 → V2)

| # | V1 decision | V2 decision | Reason |
|---|-------------|-------------|--------|
| **1** | zstd + custom graph-pattern dictionary | **mmap as-is (no compression)** for files ≤ 1GB | mmap page-level lazy is enough for current scale |
| **2** | Louvain cluster detection on co-occurrence graph | **No cluster detection yet** — use existing per-language packs | Louvain is expensive (13.2M edges = hours) with unclear ROI |
| **3** | Single archive file with byte-range index | **Keep current pack format** (`zets.core` + `zets.<lang>`) | It already works; just connect it to benchmarks |
| **4** | tenant_mask bitwise on each edge (NOW) | **Defer** — wait for real multi-tenant use case | No users yet; over-engineering |
| **5** | Inference as walk mode with Hypothesis provenance | **Confirmed** — still correct, keep | Consensus across all 3 AIs + matches existing architecture |
| **6** | Merkle-delta + snapshot versioning + CRDT | **Defer** — no edge device yet, no cloud sync pipeline | Phase 15+. Design is clear but premature |
| **7** | HE+EN stay uncompressed core, others compressed | **Already done differently** — all 16 lang packs are mmap-lazy | mmap gives lazy without compression overhead |
| **8** | Never decompress mid-walk | **Irrelevant** — mmap doesn't decompress, OS handles pages | mmap fixes this trivially |
| **9** | 5 new benchmark suites | **Confirmed — still needed**, but ordering adjusted | Keep this plan |
| **10** | Schema is part of the snapshot | **Confirmed — critical**, but scoped smaller | Start with the existing `EdgeKind` enum as schema |

---

## What's actually locked in V2

1. **Two-system merge is the #1 problem.** AtomStore (production) ↔ PieceGraph (future).
   Must resolve before anything else.
2. **mmap is sufficient up to ~1GB**. Revisit compression only if snapshot > 1GB.
3. **Per-language packs already working** — extend them, don't rebuild.
4. **Inference walk with Hypothesis provenance** — still the right design.
5. **Pack_inventory CLI** (shipped in this commit) — visibility over existing packs.

---

## What's NOT yet locked (dilemmas — see hebrew_summary_and_dilemmas_V1.md)

- **D1**: How to merge AtomStore (A) with PieceGraph/ZetsEngine (B)? 3 options (α/β/γ).
- **D2**: Domain packs (medicine / CS / geography / slang)? Manual tagging vs Louvain vs defer.
- **D3**: Personal graphs — who's the first real user?
- **D4**: Edge/Cloud sync — no edge device exists yet.
- **D5**: Hypothesis edge persistence — ephemeral or on-disk with GC?

None of these are blocking current work. All can be revisited after D1 is decided.

---

## Revised phased plan

### Phase 11 — Two-system merge (replaces old Phase 11 "cluster compression")

**Goal:** make `ZetsEngine` (mmap-based) run the benchmarks at same accuracy as
`AtomStore` flat.

**3 sub-options to choose from:**
- α. Drop PieceGraph/ZetsEngine, keep AtomStore, add per-lang to it.
- β. Drop AtomStore, migrate benchmarks to ZetsEngine, rebuild ingestion pipeline.
- γ. Keep both, add a bridge layer where `ZetsEngine` holds an `AtomStore` internally.

Decision pending from Idan.

**Accept criteria (any option):** wiki benchmark stays ≥ 68.8%. Determinism unchanged.
Memory peak ≤ current 2.5GB.

### Phase 12 — Language hot/cold switch (smaller than V1's plan)

**Goal:** when a walk hits a language not yet mmap'd, auto-open it.

- [ ] `ZetsEngine::ensure_lang(code)` — idempotent, opens pack if needed.
- [ ] Track hit-count per language for telemetry.
- [ ] Cold-evict languages untouched for 10 min (but keep HE + EN always warm).

### Phase 13 — Inference walk + Hypothesis provenance

**Same as V1.** This was already well-designed.

- [ ] `inference_walk(atom, max_hops=3)` — `is_a` up, `has_attribute` down.
- [ ] Hypothesis edges tagged with `DerivedFrom(EdgeId)` chain.
- [ ] Max depth 3 to prevent Hypothesis-on-Hypothesis creep.
- [ ] **Ephemeral only** — computed per-query, not persisted (decision D5 deferred).
- [ ] Benchmark: 20 analogy questions.

### Phase 14 — Domain/category awareness (light version)

**Goal:** atoms carry a category tag (from Wikipedia's `Category:*`), but no clustering.

- [ ] Add `category: Option<SmallString>` to `Atom`.
- [ ] Ingestion extracts Wikipedia category.
- [ ] Walks can filter by category (optional param).
- [ ] Benchmark breakdown by category already exists — reuse.

No Louvain, no separate files, no compression. Just a tag.

### Phase 15 — [DEFERRED] Personal graphs + multi-tenant
### Phase 16 — [DEFERRED] Edge/Cloud sync
### Phase 17 — [DEFERRED] Query language

All deferred until a real use case surfaces.

---

## Next concrete steps

1. **Idan decides D1** (α / β / γ for A+B merge).
2. If γ chosen (my recommendation — safest): I build the bridge layer in Phase 11
   without breaking either system.
3. If α: quick path, some data loss (drop B's ConceptNet).
4. If β: biggest risk, highest long-term quality.

---

## Honest caveats

- I still haven't measured: does `ZetsEngine` handle 211K atoms? Current pack has 144K
  concepts — different scale. May fail at wiki scale until tested.
- `atom_persist` and `pack.rs` were built by different generations of ZETS — may have
  subtle incompatibilities.
- mmap's "RAM peak" = 81MB for 65MB file was on a ~idle system. Under load with
  many simultaneous walks, could be higher.
- The 0.2ms walk target in V1 wasn't measured on cold pages — only on warm. Needs
  honest remeasurement.
