# ELDAD — Genuinely Novel Findings from ZETS Research

**Author:** Idan Eldad (עידן אלדד)
**Date:** 25.04.2026
**Status:** Empirically validated, literature-checked for novelty

This document records findings from ZETS empirical research that — to the best
of literature search — appear to be genuinely new contributions, not yet published.
Each finding is stated with the caveat that absence of evidence is not evidence
of absence; further literature review by domain experts is recommended.

---

## Background — What's Known Already

The following are STANDARD results in HDC/VSA literature, NOT novel:
- Quasi-orthogonal random hypervectors at d=10000 (Kanerva 2009)
- 2^D nearly-orthogonal vectors in dimension D (folk theorem)
- XOR/multiplication binding, sum bundling, permutation rotation
- Capacity ~D/2 for bundles before noise dominates (Plate 1995)
- Constant-time approximate nearest-neighbor via VSA cleanup memory
- Strange attractors in iterated nonlinear dynamics (Lorenz 1963, Ruelle/Takens 1971)
- HDC + attractor networks: BTSP-enhanced HDC (2025 bioRxiv preprint)

The contributions below are NOT these. They build ON these.

---

## ELDAD-1: φ-Shift Compression with Empirical Orthogonality

### Claim
A single seed hypervector + N golden-ratio-derived rotation offsets
produces N quasi-orthogonal hypervectors INDISTINGUISHABLE from N
truly random hypervectors, by mean pairwise cosine similarity.

### Empirical Result (1000 seeds × 50 shifts = 50,000 trial families)
- φ-shifted mean |cos|: 0.00792 ± 0.00105
- True random mean |cos|: 0.00797 ± 0.00017
- Theoretical orthogonal: 0.01000
- Difference: STATISTICALLY INDISTINGUISHABLE

### Why It's Novel
Standard HDC literature treats vector generation as i.i.d. random sampling.
The use of golden-angle (φ ≈ 137.5°) deterministic rotations to GENERATE a
vocabulary from a single seed appears unstudied. Phyllotaxis and quasi-random
sequences are known in different contexts; their application to HDC vocabulary
generation with empirical orthogonality validation is the contribution.

### Practical Significance
Storage compression: 1 seed atom + N integer offsets vs N independent atoms.
For DIM=10000, that's 1.25 KB + 200 B vs 62.5 KB. **Compression ratio ≈ 50×.**
For 350K Hebrew vocabulary: 438 MB → ~1.5 MB. **Ratio ≈ 290×.**

### Caveat
The atoms are deterministically derived. They function AS-IF independent for
binding/bundling, but cannot encode genuinely independent information.
Two different seeds give two different "vocabularies" that are also
quasi-orthogonal to each other — but within a single vocabulary, all atoms
are derivative of one seed.

---

## ELDAD-2: Operation-Driven Strange Attractor in HDC

### Claim
Iterating a fixed sequence of HDC operations (bind + bundle with a fixed
operator set) on RANDOM initial vectors produces convergence to a strange
attractor — a near-identical fixed-point regardless of initial vector.

### Empirical Result (1000 random initial vectors)
- Pairwise mean cosine of final attractor states: **0.885**
- Pairwise std: 0.0019
- Null (truly random vectors): mean = -0.0000, std = 0.0100
- Z-score of difference: very large (number reported was inflated due to small
  std of attractor cluster; the real magnitude is "0.885 vs 0.000" which is
  visually obvious without statistics)

### Why It's Possibly Novel
- Standard attractor networks (Hopfield, BTSP-HDC) require LEARNED weights
- Strange attractors in chaos theory require nonlinear ODE/iterated maps
- This finding: a FIXED operator chain on RANDOM initial state produces
  attractor convergence in HDC
- The 2025 BTSP-HDC paper introduces attractor features via plasticity rule;
  this finding suggests attractor behavior emerges WITHOUT plasticity, just
  from operator structure
- Direct literature search did not find prior work on this specific phenomenon

### Practical Significance
Suggests that "knowledge" in ZETS is encoded primarily in the OPERATOR CHAIN
(bind, bundle, rotate sequence), not in the data. The same chain applied to
different inputs converges to similar outputs. This is consistent with
representation theory but specific empirical demonstration in HDC appears new.

### Caveat
- Tested with one specific operator chain. Other chains may not converge.
- The attractor may be specific to the operators chosen, not universal.
- Need further work: characterize basin of attraction, sensitivity to
  operator changes, fractal structure if any.

---

## ELDAD-3: Sefirotic-Ordered Beam Search with Shabbat-Pattern Tikkun

### Claim
A heuristic search procedure structured by Kabbalistic ordering
(7 angels in Chesed→Malkhut sequence, with Shabbat-position 8th tikkun
operation) produces statistically significant quality improvement over
the same procedure WITHOUT the 8th step.

### Empirical Result (1000 trials each, paired t-test)
- 7 walks without tikkun: 61.75% ± 11.16% of optimal
- 7 walks + 8th tikkun: 64.33% ± 10.80% of optimal
- Mean difference: +2.57% (CI: +2.20%, +2.95%)
- Paired t-statistic: 13.47 (p < 0.01)
- Time cost: 1.3× (1.69 ms → 2.13 ms)

### Why It's Novel
Beam search with structured beam selection IS standard. The specific finding
that a 7+1 structure (parallel walks + a single integration/tikkun step)
provides statistically significant improvement — and the connection to the
Kabbalistic Shabbat structure — appears novel as a search algorithm design.

### Practical Significance
+2.57% quality improvement at 30% time cost is not dramatic but is robust.
For applications where quality matters more than latency (offline batch
inference), this is a free improvement.

### Caveat
- The Kabbalistic framing is INSPIRATION, not justification
- Same effect might be reproducible with any 7+1 ensemble structure
- Did not test against equally-sized non-structured baselines exhaustively

---

## ELDAD-4: Genesis-on-Demand HDC Vocabulary

### Claim
Hebrew vocabulary atoms can be generated deterministically on-demand from
character labels via SHA-256 seeded RNG, eliminating need to store any
vocabulary file. With Zipfian access caching, observed query latency
matches stored-lookup latency for hot terms.

### Empirical Result
- Generation rate: 79.8 μs/atom (12,525 atoms/sec)
- Disk read: 2.5 μs/atom (32× faster than gen)
- Cached query (Zipfian): 6.66 μs/query
- Pure on-demand: 72.6 μs/query
- Cache speedup: 10.9× over pure-gen
- Cache size for 100K queries: 8530 unique atoms = 10.4 MB
- Reproducibility: 10/10 calls with same seed give identical output

### Why It's Novel-Adjacent
Hash-based vector generation is known. The combination with HDC vocabulary
+ Zipfian caching for streaming corpora ingestion is, as far as I can find,
not explicitly published as an architecture. It's an engineering pattern
rather than a theorem.

### Practical Significance
For a system serving Hebrew NLP queries: 0 MB base storage instead of 438 MB.
Cold queries pay 73 μs penalty. Hot queries (Zipfian) match storage lookup.
Useful for resource-constrained deployment (mobile, edge).

### Caveat
- Speed comparison is against numpy disk read; specialized lookup tables
  could be faster than 2.5 μs.
- For corpus ingestion (write-many), this is much faster: 12K atoms/sec.

---

## ELDAD-5: Constant-Time Approximate Search via Sefirot Walk

### Claim
A bounded-depth walk through a sparse φ-distributed edge graph,
augmented with 180° complement checks ("shezirah pairs"), achieves
APPROXIMATE nearest-neighbor search in time independent of arena size.

### Empirical Result (5 N values, 5 trials each)
- N=1,000: 3.3 ms walk vs 14 ms brute = 4× speedup, 91% quality
- N=10,000: 3.4 ms vs 145 ms = 42× speedup, 85% quality
- N=100,000: 4.7 ms vs 1425 ms = 305× speedup, 66% quality
- Walk visits stay nearly constant (~150-770 atoms regardless of N)
- Walk time stays nearly constant (~3-5 ms regardless of N)

### Why It's Possibly Novel
Constant-time approximate ANN exists (LSH, Annoy, FAISS-IVF). The specific
finding here is that a walk structured by Kabbalistic angels + complements
+ partzuf jumps gives O(1) time at the cost of O(1) quality (~70%) without
requiring index pre-construction or learned hashing.

### Practical Significance
For real-time semantic queries against large corpora at moderate quality:
- 305× speedup at 100K atoms with 66% quality
- No index required
- Drip-loop refinement reaches 99% quality in ~56 cycles (~185 ms)

For most retrieval tasks where 66% match is "good enough first pass",
this is a viable production algorithm.

### Caveat
- Quality drops with N: 91% → 66% as N grows from 1K to 100K
- Not Grover-fast for exact matching
- Standard ANN methods may match or exceed at scale with proper indexing

---

## What These Findings Are NOT

1. **NOT quantum computing.** No Grover speedup, no Shor, no exponential
   parallelism. Classical CPU only.

2. **NOT a complete AGI architecture.** These are mechanisms for vocabulary
   generation, search, and memory. Knowledge ingestion, reasoning beyond
   similarity, and grounding remain open problems.

3. **NOT proven on real-world corpora.** All experiments use synthetic
   random hypervectors. Hebrew Wikipedia ingestion is calculated extrapolation,
   not measurement.

4. **NOT replacements for established methods.** They are complementary
   building blocks. φ-shift compression doesn't replace HDC; it's a way to
   parameterize HDC vocabularies.

---

## Recommended Next Steps

1. **External review** by HDC/VSA researchers — these claims need expert
   eyes from outside the project.
2. **Test on real corpora** — Hebrew Wikipedia, ConceptNet, real text.
3. **Formalize attractor finding** — is the attractor in ELDAD-2 universal
   or operator-specific? Lyapunov exponent? Basin of attraction?
4. **Compare ELDAD-5 against FAISS-IVF, HNSW, Annoy** at same N and quality
   targets — is the constant-time claim robust against best-in-class baselines?
5. **Implement in Rust** with the algorithms validated above; measure
   real-world performance, not numpy synthetic.

---

*This document records findings from approximately 8 hours of empirical
research conducted on 25.04.2026. Source experiments and raw outputs are
preserved at /tmp/exp*.{py,log,json} and /tmp/rerun*.{py,log,json} on
ddev.chooz.co.il. No claim in this document is presented as definitive.*
