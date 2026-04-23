# ZETS Architecture Plan V1 — Compressed Cognitive Kernel

**Date:** 22.04.2026
**Status:** Draft — synthesized from 3-AI shevirat-kelim discussion
**Basis:** Claude + Gemini 2.5 Flash + Groq Llama-4-Scout on 6 architectural questions
**Commit base:** 4dfca97 on main

---

## Executive summary

ZETS today is a **stateless monolithic Rust CLI** that loads a single 158MB snapshot into RAM per invocation. To run on phone/laptop/server/chip, to handle multi-tenancy, to stream languages and domain packs in and out of memory, it must become a **chunked, compressed, provenance-aware, multi-tenant system** — without losing its deterministic, auditable, LLM-free character.

The three AI reviewers (Claude, Gemini 2.5 Flash, Groq Llama-4-Scout) converged on **5 major design decisions** and surfaced **4 issues Idan is missing**.

---

## The 6 questions — consensus and disagreements

### Q1 — Language model

| | Claude | Gemini | Groq |
|---|---|---|---|
| **"pre-load grammar-rules" = ?** | Morphology tables + POS tagger data + script/alphabet recognizers | UD framework, morphological analyzers, tokenization rules | Morphology tables + script recognizers |
| **"real use" detection** | Token-frequency threshold + successful smart_walk hits | Sustained token-frequency + min successful smart_walk | Query-hit count in short time window |
| **concrete lib** | ICU + Morfessor | Universal Dependencies (UD) | Morfessor + ICU |

**Consensus:** Pre-load = tokenization + morphology + POS. Detection = hit-count threshold. Concrete = ICU for scripts, Morfessor for morphology, UD treebank format for grammar rules.

**Critical risk (Gemini):** "Grammar rules cannot be fully isolated from vocabulary" for agglutinative languages (Finnish, Turkish, Hungarian, Hebrew). **You cannot have Hebrew grammar without Hebrew vocabulary** — the two are entangled. Cost: Hebrew cannot be "frozen vocabulary + live grammar". Hebrew stays uncompressed always. English too.

### Q2 — Knowledge packs (compression)

| | Claude | Gemini | Groq |
|---|---|---|---|
| **compression unit** | Per-cluster, 10–100 atoms | Per-cluster, 100–1000 atoms | Per-cluster, 10–100 atoms |
| **cluster definition** | Co-occurring atoms (walk-neighbors) | Strongly connected component / cohesive subgraph | Generic grouping |
| **compression scheme** | zstd with graph-pattern dictionary | zstd with custom dict + seekable API | zstd + optional GraphZip |
| **activation trigger** | First edge-traversal into cluster | First edge-traversal into cluster | Query count threshold |
| **idle trigger** | 10 min no access + memory pressure | 5–10 min + memory pressure | Time since last access |

**Consensus:** Per-cluster compression with zstd + custom dictionary trained on graph patterns. Cluster size: **100–1000 atoms is the sweet spot** (Gemini's larger range is more defensible — overhead of managing millions of tiny blocks dominates otherwise). Decompress on first edge-traversal into the cluster. Re-compress after 5–10 min idle **plus** when memory pressure is high.

**Critical risk (Gemini + Claude):** The 0.2ms walk-step constraint WILL be violated by any non-trivial decompression. Even a 10KB zstd block takes ~1ms to decompress — 5× the step budget. **Resolution:** pre-warm heuristics + strict "do not decompress mid-walk" rule. A walk only "activates" a cluster's decompression as a side-effect — the walk itself continues on the already-loaded data, and the background thread warms the new cluster for the next walk.

### Q3 — Personal graphs (multi-tenancy)

| | Claude | Gemini | Groq |
|---|---|---|---|
| **data model** | Global shared + per-user overlay | Global shared + per-user overlay | Per-user overlay on global |
| **ACL granularity** | Per-edge | Per-edge | Per-edge or per-atom |
| **inheritance** | Read-only view with consent | Permissioned views with provenance | Hierarchical graph |
| **concrete** | Google Zanzibar ACL model | Row-Level Security (RLS) | RBAC + Neo4j |

**Consensus:** **Global shared base + per-user overlay graphs**, with per-edge ACLs and read-only family/group views requiring explicit consent.

**Critical risk (Gemini + Claude):** Per-edge ACL check on every walk step × millions of edges = 0.2ms latency dies. **Resolution:** compile ACL to an opaque tenant_id bitmask stored inline with the edge (1 byte). Check is a single bitwise AND per edge — ~0.5ns, lost in the noise. Family/group scopes = pre-computed tenant_id unions.

### Q4 — Edge/Cloud sync

| | Claude | Gemini | Groq |
|---|---|---|---|
| **protocol** | Delta-based per-atom/edge | Delta per-atom/edge + Merkle | Delta + per-cluster |
| **cloud "learn" trigger** | Aggregate miss-count across users | Threshold of miss queries from multiple devices | Miss-count on same topic |
| **concurrency** | CRDT for user updates | automerge-rs CRDT | Operational Transformation |
| **determinism concern** | Cloud "learning" must stay deterministic per-snapshot | Flagged as non-determinism risk | Not addressed |

**Consensus:** **Delta-based sync with Merkle-tree checksums, CRDTs for user annotations.** Cloud detects "need to learn" from aggregate miss-counts across multiple edge devices.

**Critical risk (Gemini):** Cloud-side learning + CRDT concurrency threatens determinism. **Resolution (not in reviewer responses — mine):** treat each new "learned" fact as adding a versioned layer to the snapshot. The snapshot at time T is still deterministic. Cloud learning = atomic snapshot publish with new version hash. Edge always pins to a specific snapshot hash.

### Q5 — Walk-based lazy decompression

| | Claude | Gemini | Groq |
|---|---|---|---|
| **is zstd-seekable real?** | Real | Real, needs careful impl | Real |
| **decompression unit** | 100-atom cluster | 100–1000 atoms per cluster | Chunks via mmap |
| **mmap + compression** | Not OS-native, custom needed | Not OS-supported | Possible via mmap |
| **alternative** | Graph-level split | Graph-level split with index | Graph-level split |

**Consensus:** **Graph-level split into clusters, each cluster compressed independently with zstd + custom dictionary, indexed by byte-range in a single archive file.** zstd-seekable is the pragmatic primitive. mmap of decompressed bytes works; mmap of compressed bytes does not.

**Critical risk (Gemini):** Overhead of managing many tiny blocks (file handles, metadata, dictionaries) can negate benefits. **Resolution:** one big archive file + in-memory byte-range index. Open archive once, seek + decompress specific ranges. No file-handle explosion.

### Q6 — Inference by analogy

| | Claude | Gemini | Groq |
|---|---|---|---|
| **is it automatic?** | Yes via existing walk | Yes via smart_walk with inheritance mode | Yes with modified scoring |
| **needs new module?** | Strategy within smart_walk | Strategy within smart_walk | Modified smart_walk |
| **hallucination control** | Tag inferred as Hypothesis | Tag Hypothesis with derivation path | Tag Hypothesis |
| **concrete** | Inheritance walk mode | Hypothesis(DerivedFrom(...)) provenance | GraphSAGE |

**Unanimous consensus:** **Inference by analogy requires no new module.** Add a walk mode `inference_walk()` that follows `is_a` edges upward then `has_attribute` downward. Every inferred edge is tagged `Hypothesis` with a `DerivedFrom(EdgeID_is_a, EdgeID_has_attribute)` provenance chain.

**Critical risk:** over-inference → hallucination. **Resolution:** bound inference depth to 3 hops max by default; require all intermediate edges to be `Asserted` or `Observed` (not `Hypothesis`) — no inference chaining on inferences. This prevents hypothesis creep.

---

## Cross-cutting concerns (all 3 reviewers flagged)

### C1. Latency vs. granularity — the fundamental tension

0.2ms walk target × lazy-load overhead = impossibility without tricks.

**Resolution strategy:**
- Never decompress mid-walk. Walks only run against already-resident data.
- Background "pre-warmer" thread reacts to walk patterns: if walk1 touched cluster X, pre-warm neighbors of X.
- "Cold walk" explicitly flagged: returns fewer candidates, marks answer with `requires_warming` provenance tag.
- Provenance never hides slow paths — user sees them.

### C2. Provenance propagation is a first-class concern

Every operation — inference, learning, multi-tenant sharing, CRDT merge — must propagate provenance. Current ZETS has 4 tags (Asserted / Observed / Learned / Hypothesis). This plan adds derivation chains.

**Resolution:** extend `EdgeSource` enum to carry full derivation path:

```rust
enum EdgeSource {
    Asserted(CorpusRef),
    Observed { corpus: CorpusRef, count: u32 },
    Learned { pattern: PatternId, from: Vec<EdgeId> },
    Hypothesis { derived_from: Vec<EdgeId>, reasoning: InferenceKind },
    Synced { origin_tenant: TenantId, at_version: SnapshotHash },
}
```

### C3. Memory eviction — LRU + pressure-aware

Phone = 4GB RAM, maybe 500MB for ZETS. Laptop = 2GB. Server = 20GB. Edge chip = 256MB.

**Resolution:** two-level eviction:
1. LRU across clusters (standard).
2. Memory-pressure callback re-compresses oldest-untouched clusters first, even within LRU window.

---

## What Idan is missing (blunt)

### M1. Graph schema and evolution (flagged by Gemini)

ZETS has atoms and edges with types but no formal schema. When adding Wikipedia + PubMed + Wiktionary + cultural corpora, type drift will create unmanageable chaos.

**Action:** formal schema spec. New edge types require a proposal + a semantics test (what walks use them, what's the inverse, what provenance is allowed). Store schema in git as part of the snapshot.

### M2. Conflict resolution for learning (flagged by Gemini)

Wikipedia says X, PubMed says not-X. When cloud "learns" new facts, what wins?

**Action:** provenance-weighted voting. Each corpus gets a reliability weight (Wikipedia = 0.9, PubMed for medicine = 1.0, arXiv = 0.95, random web = 0.3). Conflicting edges both persist; walks select by weighted sum + provenance metadata. This stays deterministic because the weights and corpora are snapshot-fixed.

### M3. Cold-start problem (flagged by Gemini)

First query to a new domain/language pack pays full decompression cost.

**Action:** per-user profile tracks 5 most-used domains. On process start, pre-warm those clusters in background before accepting queries. First query may be slow (flagged in response), subsequent are fast.

### M4. Query language beyond smart_walk (flagged by Gemini)

Today ZETS has `smart_walk()`, `find_relevant_atoms()`, `score_candidates()`. No user-facing query language. Can't express "find all X that are_a Y with attribute Z".

**Action:** define ZETS-QL — a minimal declarative query language that compiles to walk sequences. See future spec.

### M5. Evaluation plan across new dimensions (flagged by Groq)

Current 32-question MC benchmark doesn't measure: language coverage, compression overhead, multi-tenant isolation, edge-cloud sync correctness, inference accuracy.

**Action:** 5 new benchmark suites, one per cross-cutting concern. See Phase breakdown below.

---

## Phased implementation plan

### Phase 11 — Cluster-based compression format (core)

**Goal:** replace monolithic `.atoms` file with chunked, compressed `.zpak` archive.

- [ ] Define ClusterId (u32) and Cluster = contiguous range of atom IDs.
- [ ] Cluster detection: Louvain algorithm or connected-components on the co-occurrence graph.
- [ ] Format spec:
  ```
  .zpak = { header | cluster_index | zstd_dict | compressed_cluster_blocks }
  header = {version, n_clusters, dict_offset, dict_size, index_offset}
  cluster_index = [ (cluster_id, offset, compressed_size, n_atoms) ]
  ```
- [ ] `write_zpak()` CLI — migrate existing snapshots.
- [ ] `load_zpak()` with on-demand cluster decompression.
- [ ] LRU eviction + idle-timeout re-compression.
- [ ] Regression test: wiki_all_domains_v1 → .zpak round-trip preserves all 32 benchmark answers.

**Accept criteria:** zpak size ≤ 40% of raw; benchmark still 68.8%; memory peak ≤ 500MB on wiki query.

### Phase 12 — Language packs

- [ ] LanguagePack = { alphabet_ranges, script_id, tokenizer_type, morphology_table, pos_model }.
- [ ] HE + EN core packs always loaded.
- [ ] 5 other lang packs (AR, FR, ES, DE, RU) shipped as frozen .zpak.
- [ ] Hit-counter per language; unfreeze when N queries / hour exceeded.
- [ ] Regression: HE and EN benchmarks unchanged; FR/DE query triggers unfreeze; determinism preserved.

### Phase 13 — Inference walk + Hypothesis provenance

- [ ] `inference_walk(atom, max_hops=3)` — follows `is_a` up, `has_attribute` down.
- [ ] Produces Hypothesis edges with DerivedFrom chain.
- [ ] Hypothesis edges never chain on Hypothesis edges (cycle prevention).
- [ ] Benchmark: 20 analogy questions (capybara → four_legs, etc.).

### Phase 14 — Multi-tenant overlay + per-edge ACL

- [ ] TenantId = u32 (family-tree-friendly: children inherit parent's bitmask).
- [ ] Add tenant_mask: u64 to EdgeSource.
- [ ] Walks respect tenant context.
- [ ] Benchmark: privacy-leak test (user A query cannot return user B's private edges).

### Phase 15 — Edge/Cloud sync protocol

- [ ] Snapshot versioning with content hash.
- [ ] Delta format: (prev_hash, next_hash, added_edges, removed_edges).
- [ ] Edge device: local .zpak pinned to version V; cloud serves V+1 deltas.
- [ ] CRDT for user annotations (automerge-rs).
- [ ] Miss-query aggregation: cloud tracks top-N unanswered topics, flags for learning.
- [ ] Benchmark: offline mode works (edge has pinned V, no internet, queries still answer deterministically).

### Phase 16 — Query language (ZETS-QL)

- [ ] Minimal declarative grammar compiled to walk sequences.
- [ ] Example: `find X where X is_a mammal and X has_attribute aquatic` → walk plan.
- [ ] REPL + MCP tool.

### Phase 17 — Formal schema + provenance auditor

- [ ] Schema file (`schema.yml`) in the repo, versioned.
- [ ] `zets audit` CLI: checks every edge against schema, reports orphan types.
- [ ] CI test: schema validation on every ingestion.

---

## Decisions locked NOW

These are no longer open. Challenge only with hard data:

1. **Compression = zstd with custom graph-pattern dictionary. Per-cluster. Cluster size target 100–1000 atoms.**
2. **Cluster detection = Louvain on co-occurrence graph.** (Not random grouping.)
3. **Archive format = single file with byte-range index.** Not per-cluster files.
4. **ACL = tenant_mask bitwise stored inline with each edge.** No separate ACL table.
5. **Inference = walk mode, not separate module.** Tagged Hypothesis with DerivedFrom chain.
6. **Sync = Merkle-delta + snapshot versioning. CRDT only for user annotations.**
7. **HE + EN stay uncompressed core.** Hebrew cannot freeze vocabulary (agglutinative). English stays hot because of universality.
8. **Never decompress mid-walk.** Walks hit already-loaded clusters. Pre-warming is background.
9. **Evaluation = 5 new benchmark suites, one per cross-cutting concern.** Land with each phase.
10. **Schema is part of the snapshot.** No schema-less evolution.

---

## Next 3 sessions

**Session 1:** Phase 11 design doc in full (cluster detection algorithm + .zpak format spec + migration path) + prototype code.

**Session 2:** Phase 11 implementation against v1_world_facts_large; benchmark; regression.

**Session 3:** Phase 11 against wiki_all_domains_v1 (211K atoms); measure compression ratio, memory, latency; lock the format.

Only after Phase 11 ships does Phase 12+ begin. No premature abstraction.

---

## Honest caveats

- Reviewers (Gemini, Groq) gave broad-strokes consensus. None have actually built this system.
- "Concrete library" suggestions are starting points, not guarantees. zstd-seekable API has evolved; verify current Rust bindings before Phase 11.
- The Louvain community-detection algorithm assumes homogeneous graph weights; ZETS edges have provenance-weighted semantics. May need a weighted variant (e.g., weighted modularity optimization).
- The 0.2ms walk latency is aspirational, not measured on cold clusters. Phase 11 must re-measure honestly.

---

## References

- Zstd seekable format: https://github.com/facebook/zstd/tree/dev/contrib/seekable_format
- Louvain community detection: Blondel et al. 2008
- automerge-rs CRDT: https://github.com/automerge/automerge
- Google Zanzibar: https://research.google/pubs/zanzibar-googles-consistent-global-authorization-system/
- Universal Dependencies: https://universaldependencies.org/
- Morfessor: https://morfessor.readthedocs.io/
