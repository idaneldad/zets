## Top 5 Critical Issues

### ISS-01: Atom ABI is not one source of truth
- **Section affected:** §0.2, §0.10, §0.11, §4.1, §5.1-5.4, §31
- **Severity:** critical
- **Confidence:** 98
- **Claim:** Multiple incompatible AtomKind enums, bit layouts, language widths, and graph-id reservations make federation impossible.
- **Proposed patch:** Replace §5.1 `AtomKind` with §0.3 enum exactly. Delete old HebrewWord/ArabicWord/etc. Add: “All variant layouts MUST be overlays of §0.2 unless explicitly defined per AtomKind in §0.x.” Remove §0.11 deferral or mark §0.2 non-binding.
- **Hidden assumption:** ABI v1 must be implementable immediately, not aspirational.
- **Strongest self-objection:** Layout B may be better for SDR reasoning.
- **Validation test:** Compile-time test: every section’s `AtomKind` names/values generated from one `atom_abi.rs`; no duplicate enum definitions.

### ISS-02: Edge ABI contradicts itself and cannot encode declared values
- **Section affected:** §0.4, §5.5, §10.2, §15.1, §18.4, §20.1
- **Severity:** critical
- **Confidence:** 100
- **Claim:** §0 says `EdgeKind=u16`, but `EdgeHot` stores u8; §18 uses `repr(u8)` with values 300/400.
- **Proposed patch:** Keep 6-byte edge by adding a dictionary:  
  ```rust
  #[repr(C, packed)]
  pub struct EdgeHot { target: u32, meta: u8, kind_idx: u8 }
  pub struct EdgeKindTable([u16; 256]); // per graph segment
  ```  
  Replace all `predicate as u8` with `edge_kind_table.intern(predicate_u16)`.
- **Hidden assumption:** 6B edge density is more important than direct u16 storage.
- **Strongest self-objection:** Per-segment dictionaries complicate mmap replay.
- **Validation test:** Insert edge kind `0x0100`, `0x012C`, and Tav=400; serialize/deserialize and recover exact u16.

### ISS-03: Determinism is violated by floats, time, maps, and external enrichment
- **Section affected:** §0.5, §2.2, §10, §11, §15.3, §16, §20, §29
- **Severity:** critical
- **Confidence:** 92
- **Claim:** `f32`, wall-clock `now()`, FxHashMap, partial sort ties, and LLM JSON create non-replayable graphs.
- **Proposed patch:** Add §0.12 Deterministic Numerics:  
  “All ranking/activation uses fixed-point `i32 Q16.16`; ties sorted by `(score desc, AtomId asc)`. Wall time stored only as Observation; replay uses Lamport clock. FxHashMap forbidden in replay path. External model outputs enter Sandbox as ObservationAtom only.”
- **Hidden assumption:** Bit-identical replay matters more than tiny score precision.
- **Strongest self-objection:** Fixed-point may weaken analog/vector scoring.
- **Validation test:** Same seed/query/graph on x86_64 and ARM64 produces byte-identical `ReasoningTrace` and answer atom set.

### ISS-04: CSR storage cannot support continuous online learning as specified
- **Section affected:** §5.6, §10.1-10.5, §14, §20, §21
- **Severity:** critical
- **Confidence:** 94
- **Claim:** The spec mutates/inserts edges constantly, but CSR is static and expensive to update.
- **Proposed patch:** Add LSM graph architecture:  
  ```text
  BaseCSR: immutable mmap, rebuilt NightMode.
  DeltaLog: append-only EdgeOp records, max 256MB RAM/disk active.
  TombstoneSet: deleted/overridden edges.
  Query neighbors = merge(BaseCSR row + Delta row - tombstones), stable-sorted.
  Compact when DeltaLog > 5% of BaseCSR or 256MB.
  ```
- **Hidden assumption:** NightMode compaction is acceptable on laptop.
- **Strongest self-objection:** Merge-on-read adds latency.
- **Validation test:** 1M inserts + 100K strength updates during session; P99 neighbor query <5µs for degree≤64, compaction restores <100ns base rows.

### ISS-05: “Zero hallucination” lacks an answer contract and trust threshold
- **Section affected:** §0.5, §14, §16, §19.3, §22, §29, §33
- **Severity:** critical
- **Confidence:** 90
- **Claim:** LM/enrichment outputs are sometimes inserted as properties without enforced provenance/trust/unknown behavior.
- **Proposed patch:** Add §19.5 Answer Contract:  
  “Every declarative sentence must cite ≥1 provenance path. If best support trust <0.65, answer ‘I don’t know’ or ‘Observed claim, unverified’. External LLM-enriched facts require two-source agreement or OwnerVerified promotion before entering Semantic graph.”
- **Hidden assumption:** Refusal is preferable to fluent unsupported answer.
- **Strongest self-objection:** This will make early ZETS feel less capable.
- **Validation test:** Ask 100 facts absent from graph; 100 must return unknown, not invented answers. Ask LM-enriched color fact; response must label provenance tier.

## Top 3 Strengths

1. **Beautiful core instinct:** graph-native, walk-traceable, Hebrew-canonical reasoning is coherent and genuinely differentiated.  
2. **Memory sovereignty:** PersonalVault + provenance + append log is the right moral architecture.  
3. **Mechanical ambition:** 8-byte atoms, mmap CSR, compact edges, and CPU-only budgets force healthy discipline.

## Open Question for Iter 2-7

What is the single canonical ABI v1 bit layout and edge serialization format that all code, tests, federation, and 30-year migration tooling must obey?

## Final Score

**6.5/10** — visionary and unusually rich, but not yet implementable because ABI, edge encoding, determinism, mutation strategy, and truth-contract contradictions must be resolved first.