## Top 5 Critical Issues

### ISS-01: AtomId u32 Scaling Contradiction
- **Section affected:** §0.7 vs §5.1
- **Severity:** critical
- **Confidence:** 95
- **Claim:** §0.7 claims "Gevurah pruning ensures active graph stays under 2B atoms" but no mechanism defined. §5.1 uses Vec<Atom> which hits 32GB at 4B atoms.
- **Proposed patch:** Add to §10.4: `fn gevurah_prune(graph: &mut Graph) { graph.atoms.retain(|a| a.freshness() > 0 || a.is_anchor()); }` + trigger at 1.5B threshold.
- **Hidden assumption:** Pruning won't delete atoms needed by later walks.
- **Strongest self-objection:** Aggressive pruning could break provenance chains.
- **Validation test:** Simulate 30-year growth at 100K atoms/day; verify <2B resident.

### ISS-02: EdgeKind u16 vs §5.5 EdgeHot u8
- **Section affected:** §0.4 vs §5.5
- **Severity:** critical
- **Confidence:** 100
- **Claim:** §0.4 declares EdgeKind as u16 (BINDING), but §5.5 EdgeHot uses `edge_kind: u8` in packed struct. Direct ABI violation.
- **Proposed patch:** In §5.5: `pub packed: [u8; 7]` with `[47..32] edge_kind: u16, [31..0] target`.
- **Hidden assumption:** 1 extra byte per edge is acceptable (1B edges = 1GB more).
- **Strongest self-objection:** 6GB edge budget violated; need 7GB.
- **Validation test:** `assert_eq!(std::mem::size_of::<EdgeHot>(), 7)`.

### ISS-03: Determinism Boundary Incomplete for Walk Seeds
- **Section affected:** §0.5, §11.2
- **Severity:** critical
- **Confidence:** 85
- **Claim:** §0.5 guarantees determinism for "Walk traversal given fixed (graph_version, query, seed)" but §11.2 QuantumWalker never consumes seed parameter.
- **Proposed patch:** Add to QuantumWalker: `pub seed: u64` and use `deterministic_rng(self.seed)` for neighbor selection order.
- **Hidden assumption:** BFS order is already deterministic via IndexMap.
- **Strongest self-objection:** IndexMap iteration order may depend on insertion order, not deterministic.
- **Validation test:** Run identical walk 1000x with same seed; hash outputs; all identical.

### ISS-04: Atom Layout A vs B Unresolved
- **Section affected:** §0.11
- **Severity:** important
- **Confidence:** 90
- **Claim:** "Decision deferred to Iter 1 council vote" but this IS Iter 1 and code in §5.2 implements Layout A. Ambiguity blocks implementation.
- **Proposed patch:** Resolve NOW: Layout A for structured access (v1), reserve FLAG_SDR_MODE bit for Layout B hybrid in v1.1.
- **Hidden assumption:** Layout A suffices for MVP; SDR optimization is deferrable.
- **Strongest self-objection:** SDR dot-product is core to interference scoring (§11.4).
- **Validation test:** Benchmark interference_score with Layout A; if >1ms, escalate.

### ISS-05: 6GB RAM Target vs Edge Budget
- **Section affected:** §0.6 vs §21.2
- **Severity:** critical
- **Confidence:** 88
- **Claim:** §0.6 says "6 GB RAM minimum", §21.2 says "Edge CSR 6 GB (mmap)". mmap doesn't eliminate RAM pressure on active working set. 1B edges with 10% hot = 600MB just for hot edges.
- **Proposed patch:** In §21.2: Explicit working set budget: "Hot edges (100M) = 600MB resident; cold edges paged."
- **Hidden assumption:** mmap paging won't thrash on typical queries.
- **Strongest self-objection:** Walking cold edges triggers page faults; latency >100ns.
- **Validation test:** Profile walk on 1B-edge graph with 6GB RAM limit; measure p99 latency.

---

## Top 3 Strengths

1. **Hebrew-canonical with base37 direct encoding** — eliminates pool lookup, enables O(1) gematria, genuinely novel.
2. **Explicit determinism boundary** — honest separation of guaranteed-deterministic vs LM-dependent is rare and valuable.
3. **13-subgraph topology with cryptographic boundaries** — solves federation, privacy, and trust in unified model.

---

## Open Question for Iter 2-7

**How does ZETS handle the cold-start bootstrap?** §13.5 mentions "Cold-start from zero atoms requires what minimum knowledge base?" but nothing concrete. A 6GB-RAM laptop with empty graph cannot answer "מה זה כלב?" — what's the seed corpus, how large, and how is it distributed?

---

## Final Score

**6.5/10** — Strong conceptual architecture with genuine Hebrew-native innovation, but critical ABI inconsistencies (EdgeKind size, atom layout ambiguity) and unvalidated RAM/edge scaling claims block implementation. Fix the five issues above and this becomes 8+/10.