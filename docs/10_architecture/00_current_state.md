# ZETS — Current Architecture Snapshot

**Date:** 21.04.2026
**Commit:** e0ab3ad
**Lines of code:** 5,112
**Tests:** 72/72 passing
**Binary size:** zets 356KB, tester 513KB

---

## System layers (top to bottom)

```
┌─────────────────────────────────────────────────────────────┐
│                   USER / APP LAYER                          │
│   zets CLI  │  tester  │  scale_probe  │  unp_bench         │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                   GRAPH FACADE                              │
│   pub mod graph — unified API                               │
│      Graph { store, index, bloom, doc_alloc, aux_alloc }    │
│      + generation counter for lazy index rebuild            │
└─────────────────────────────────────────────────────────────┘
         │               │              │             │
         ▼               ▼              ▼             ▼
┌────────────────┐ ┌──────────────┐ ┌──────────┐ ┌──────────┐
│  WALK ENGINE   │ │  MEMORY MGR  │ │ LEARNING │ │ DOCUMENT │
│  forward_pass  │ │ preload/evict│ │ ingest   │ │ add_seq  │
│  backward_pass │ │ co-access    │ │ synonym  │ │ citation │
│  multi_pass    │ │ 4 strategies │ │ resolve  │ │ reconstr │
│  ResponseMode  │ │              │ │          │ │          │
└────────────────┘ └──────────────┘ └──────────┘ └──────────┘
         │               │              │             │
         └───────────────┼──────────────┼─────────────┘
                         ▼              ▼
              ┌─────────────────┐ ┌──────────────┐
              │  EDGE_STORE     │ │  BLOOM       │
              │  SOA columns    │ │  8MB, 7 hash │
              │  10 bytes/edge  │ │  <1% FP rate │
              │  30 relations   │ │              │
              │  AdjacencyIndex │ │              │
              │  binary search  │ │              │
              │  O(log N)       │ │              │
              └─────────────────┘ └──────────────┘
                         │
                         ▼
              ┌─────────────────────────────┐
              │  META-GRAPH                 │
              │  Synsets 0..999 reserved    │
              │  Languages 10..29           │
              │  Relations 30..56           │
              │  Homoiconic 11th sphere     │
              └─────────────────────────────┘
                         │
                         ▼
              ┌─────────────────────────────┐
              │  UNP NORMALIZATION          │
              │  1. NFC                     │
              │  2. trim                    │
              │  3. collapse whitespace     │
              │  4. lang-specific           │
              │  5. lemmatize (partial)     │
              └─────────────────────────────┘
                         │
                         ▼
              ┌─────────────────────────────┐
              │  PER-LANGUAGE MODULES       │
              │  hebrew { niqud, finals,    │
              │           canonicalize,     │
              │           stemmer }         │
              └─────────────────────────────┘
                         │
                         ▼
              ┌─────────────────────────────┐
              │  DATA TABLES (compile-time) │
              │  build.rs reads data/*.tsv  │
              │  include! at compile time   │
              │  Zero runtime I/O for core  │
              └─────────────────────────────┘
```

---

## Identity / Equality mechanisms currently in ZETS

| Level | What it catches | Speed | Impl status |
|-------|-----------------|-------|-------------|
| **UNP byte-exact** | Niqud variants, final forms, spacing | 183ns per normalize | ✅ done |
| **fnv_128_with_lang** | Homographs cross-language (gift[EN] ≠ gift[DE]) | ~50ns | ✅ done |
| **Bloom existence** | "Have I seen this synset before?" | ~20ns | ✅ done |
| **AdjacencyIndex lookup** | "What edges exist from/to this synset?" | ~240ns (O(log N)) | ✅ done |
| **SAME_AS / NEAR_SYNONYM edges** | Translation / synonym membership | walk-dependent | ✅ done |
| **Lemmatization** | "כלבים" → "כלב" (surface → lemma) | N/A | ❌ stub only |
| **Chunk-level BLAKE3** | "Have I seen this paragraph in another document?" | N/A | ❌ not implemented |
| **Semantic embedding** | "gift" ≈ "present" without explicit edge | N/A | ❌ V2+ |

---

## What the server has available for POC

```
/home/dinio/cortex-v7/data/tanakh/     6.1 MB   39 books of Tanakh in Hebrew
/home/dinio/lev-knowledge/sources/wikipedia_he/   99 MB   69 TSV batches, Hebrew Wikipedia
/home/dinio/lev-knowledge/sources/wikipedia_en/   empty (only directory exists)
```

TSV format for wiki_he batches:
```
<word>\t<relation>\t<definition_text>\t<confidence>
```

Example rows from batch_0000:
```
April    IS_A   April (Apr.) is the fourth month of the year in the Julian and Gregorian calendars...  0.70
August   IS_A   August (Aug.) is the eighth month of the year in the Gregorian calendar...             0.70
Art      IS_A   thumb|300x300px|A painting by [[Renoir is a work of art.]] Art is a creative activity   0.70
```

**Note:** this is a Cortex-Lev-style extraction (subject, relation, definition, confidence).
Not the canonical Wikipedia dump. Good for testing ingestion — not for evaluating
semantic accuracy, because the extraction is heuristic.

---

## POC scope decision (next 4 hours)

Given:
- 99MB text corpus is available
- 72 tests pass, graph walks work
- Dedup mechanism not yet built

**Minimal POC we can do in 4 hours:**

1. **Wiki-batch ingestor** (1 hour) — parse one batch TSV, produce edges.
2. **BLAKE3 chunk hashing** (1 hour) — simplest possible. Add `blake3 = "1.5"` crate.
3. **Dedup measurement** (1 hour) — ingest all 69 batches, measure unique chunks vs total, dedup ratio.
4. **Query demo** (1 hour) — walk from a seed word, show retrieved content with provenance.

**Out of scope for 4 hours:**
- PDF/docx unwrappers (sprint 1 material)
- FastCDC content-defined chunking (sprint 1)
- Full pack file format (sprint 1-2)
- Android build (sprint 3)

---

## Performance baseline (measured on Oracle ARM x86_64)

| Operation | Time | Notes |
|-----------|------|-------|
| UNP normalize | 183ns | 5.46M ops/sec |
| EdgeStore push | 11.6ns | 86M edges/sec |
| AdjacencyIndex build (1M edges) | 80ms | one-time cost |
| AdjacencyIndex outgoing sparse | 196ns | O(log N), 5428× faster than scan |
| AdjacencyIndex outgoing dense | 754µs | bottleneck is Vec alloc, not search |
| Bloom insert | ~30ns | default 8MB filter |
| Bloom check (negative) | ~20ns | single bit read |
| Forward walk depth 3 | ~5ms | 1000 edges, 100 nodes |
| Disk serialize 100K edges | 320µs | 2675 MB/s on NVMe |
