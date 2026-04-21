# ZETS — Master Architecture Compass

**Date:** 21.04.2026
**Status:** Authoritative. All future work aligns with this document.
**Size:** Short by design — this is a compass, not a manual.

---

## 1. The Four Elements

ZETS represents all information as one of four element types:

| Element | Purpose | Example |
|---------|---------|---------|
| **Piece** | Smallest raw unit | A byte, a pixel, an audio sample, an atomic token |
| **Node** | A concept with identity | "dog", "Albert Einstein", "sales-step-7" |
| **Bundle** | Collection of elements | "red dog", "John Smith", a paragraph |
| **Flow** | Ordered sequence with direction | a sentence, a song, a sales process, a movie |

Each element is **a-synchronous** — it does not know who references it.
References are always top-down: Flow references Bundles, Bundle references Nodes, Node references Pieces.

## 2. Identity — Content-Addressed

Every element has a **content hash**:
```
hash = BLAKE3(bytes_of_element)
```

Properties:
- Same content → same hash on any machine (deterministic).
- Different content → different hash (uniqueness).
- Short reference = first 8 bytes of hash (u64) for in-memory use.
- No allocated IDs. No central registry. No synchronization needed.

This is content-addressing, same as Git and IPFS.

## 3. Relationships — Edges

Two kinds of relationships:

**Simple edge** (cheap, 10 bytes):
```
[source: 3 bytes][target: 3 bytes][relation: 5 bits][weight: 3 bits][provenance: 3 bytes]
```
Used for: IsA, HasPart, MemberOf, NextInFlow, AppearsIn, Cites, SameAs — 30 relation types total.

**Rich relation** (when metadata is needed):
Create an intermediate Node. Example: "Drug A is equivalent to Drug B with 85% probability if patient is not allergic."
- 3 edges + 1 node instead of 1 edge
- Used only when metadata is essential

## 4. Static Shared Functions

Common operations are static functions, not inherited methods:
```rust
compute_hash(bytes: &[u8]) -> [u8; 32]
short_id(hash: &[u8; 32]) -> u64
serialize(object: &dyn Serializable) -> Vec<u8>
```

No inheritance. No traits with default implementations. Just functions.

## 5. Works On Every Medium

Same model serves all data types:

| Medium | Piece | Node | Bundle | Flow |
|--------|-------|------|--------|------|
| Text | char/byte | word | phrase | sentence/paragraph/document |
| Image | pixel | region | object | image with structure |
| Audio | sample | chunk | phrase | song/recording |
| Video | frame | keyframe | shot | scene/movie |
| Process | primitive action | named step | compound step | procedure |

## 6. Full Reconstruction

Every Flow has `content_hash`. Reconstruction:
1. Walk Flow, collect Pieces in order.
2. Assemble bytes.
3. Compute hash of assembled bytes.
4. Compare to stored hash. Match = faithful reconstruction.

## 7. Current Codebase Alignment

### What already matches the model
- `EdgeStore` (SOA, 10 bytes/edge) — matches the simple edge spec exactly.
- `document::add_sequence` — implements Flow for text.
- `HasPart` / `MemberOf` relations — support Bundle semantics.
- Content-addressing via FNV/BLAKE3 — partial; needs unification.

### What needs change
- **Rename concepts** for clarity: `SynsetId` → keep (it works), but document that it maps to Node.
- **Generalize `document`** module: rename to `flow` and extend to non-text media.
- **Add Piece primitive:** new struct for raw byte/pixel/sample content.
- **Short-ID helper:** extract u64 short IDs from hashes (already implicit, make explicit).
- **Rich-relation pattern:** documentation + examples, no new code needed (graph already supports it).

### What doesn't need to change
- EdgeStore, AdjacencyIndex, Bloom, UNP — all aligned with the model.
- Walk engine, Memory manager — work at the relationship level, media-agnostic.
- Learning module — treats inputs as Nodes; already matches.

## 8. Development Plan (Near-Term)

### Phase 1 — Alignment & Ingestion (this week)
**Goal:** make the existing codebase a useful knowledge graph at scale.

1. **Verify code health** — all 72 tests pass, no regressions.
2. **Full Hebrew Wikipedia ingest** — all 69 batches (670K entries) into one graph.
3. **Tanakh ingest** — 39 books, 1.2M Hebrew letters. Build letter-level + word-level views.
4. **Dictionary ingest** — any Hebrew/English lexicons available on server.
5. **Measurement** — total graph size, RAM on Oracle ARM, query latency.

### Phase 2 — Polish per Gemini/Perplexity feedback (Sprint A)
6. CLI iterator API, stdout lock, Result-based errors.
7. `EdgeStore::outgoing_iter()` — zero-allocation iterator.
8. `Graph` always-on index, no linear-scan fallback.

### Phase 3 — Flow generalization (next week)
9. Rename `document` module to `flow`, keep backwards-compat aliases.
10. Add `Piece` struct + storage for raw media bytes.
11. Proof-of-concept: ingest one song/image/audio file, reconstruct byte-identical.

### Phase 4 — Cognitive engine (future)
12. Query planner, beam tree, user model, response crafter.
(Already specified in COGNITIVE_TREE_SPEC, EMPATHIC_RESPONSE_SPEC.)

## 9. Efficiency Principles — Non-Negotiable

Any change must honor:
- **≤10 bytes per simple edge**
- **Content-addressed** — no allocated IDs
- **Static functions** — no inheritance chains
- **Zero-allocation on hot paths** — iterators, not Vec returns
- **Index always-on** — no linear fallbacks
- **Graph IS the storage** — no parallel caches

If a feature violates one of these, redesign the feature, not the principle.

## 10. What This Document Replaces

This is the compass. The following become historical references:
- `AGI_LITE_SPEC.md` — implementation plan still valid
- `COGNITIVE_TREE_SPEC.md` — beam tree design still valid
- `EMPATHIC_RESPONSE_SPEC.md` — user model design still valid
- `UNIVERSAL_INGESTION_ARCHITECTURE.md` — file ingest spec still valid
- `OPENCLAW_INTEGRATION.md` — nervous system spec still valid

All of the above now **inherit from this document**. Conflicts resolved in favor of this.

---

**End of compass.**
