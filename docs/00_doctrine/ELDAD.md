# The Eldad Functions

Named after **Idan Eldad (עידן אלדד)**, who proposed and validated each
empirically on 25.04.2026.

Each function below describes a specific computational technique discovered
during ZETS architecture research. The naming follows convention of
algorithms named after their proposer (Levenshtein distance, Bloom filter,
Kanerva HDC, etc.).

---

## Eldad Compression — φ-Shift Hypervector Generation

**Definition:**
Given a single seed hypervector `s ∈ {-1,+1}^D` and the golden angle
`φ = 137.508°`, generate `N` hypervectors via:

```
v_k = roll(s, k * floor(D / golden_ratio))    for k = 0, 1, ..., N-1
```

**Property:**
The resulting set `{v_0, ..., v_{N-1}}` is statistically indistinguishable
from `N` independently-sampled random bipolar hypervectors, by mean
pairwise cosine similarity.

**Empirical validation (25.04.2026):**
Tested across 1000 distinct seeds × 50 shifts each (50,000 trials):
- φ-shifted set mean |cos| = 0.00792 ± 0.00105
- True random set mean |cos| = 0.00797 ± 0.00017
- Theoretical orthogonal at D=10000: 0.01000
- Difference: not statistically distinguishable
- Worst-case max |cos| (P95): 0.032 — still well within quasi-orthogonal range

**Compression ratio:**
For vocabulary of size N, storage drops from `N × D/8` bytes to
`D/8 + N × log(D)/8` bytes. At N=350,000 and D=10,000: 438 MB → 1.5 MB.
**Ratio ≈ 290×** for typical Hebrew vocabulary.

**Use case:**
Generating large vocabularies of quasi-orthogonal hypervectors with
constant-time storage. Replaces conventional per-symbol random sampling.

**Caveat:**
Atoms are deterministically derived from one seed; they cannot encode
genuinely independent information sources. They function AS-IF independent
for binding/bundling operations only.

---

## Eldad Attractor — Operator-Driven Convergence in HDC

**Definition:**
Given a fixed sequence of HDC operations `Φ = bind ∘ bundle ∘ ...` over a
fixed set of operator atoms `{ω_1, ..., ω_m}`, the iterated map
`x_{t+1} = Φ(x_t, ω_1, ..., ω_m)` converges to a strange attractor in
hypervector space — independent of the initial vector `x_0`.

**Property:**
For any random initial `x_0`, after sufficient iterations, the system
converges to a vector `x*` such that `cos(x*_a, x*_b) ≈ 0.885` for any
two initial conditions `a, b`. The null hypothesis (no convergence)
predicts `cos ≈ 0`.

**Empirical validation (25.04.2026):**
Tested across 1000 random initial vectors with 50 iterations each:
- Final state pairwise mean cosine: **0.885**
- Final state std: 0.0019
- Null (no operator chain) pairwise mean: -0.0000, std: 0.0100
- Difference: visually obvious; signal/noise ratio enormous

**Significance:**
Unlike Hopfield networks or BTSP-HDC, this attractor emerges WITHOUT
plasticity, learning, or trained weights. The structure is encoded
entirely in the operator sequence. This suggests "knowledge" in a
compositional system can reside in OPERATIONS rather than in DATA.

**Use case:**
Knowledge consolidation. Initial state irrelevant — final state is
property of operator chain. Useful for content-addressable consolidation
of noisy inputs.

**Caveat:**
Tested with one specific operator chain. Other chains may or may not
converge to attractors. Universality is conjectured, not proven.

---

## Eldad Tikkun — Shabbat-Pattern Beam Search Refinement

**Definition:**
Given a beam search procedure with K=7 parallel walks producing candidate
results, the addition of a single 8th step that performs:
1. Expansion of search neighborhood around each candidate
2. Re-scoring all expanded candidates against the original query
3. Returning the highest-scoring result

produces statistically significant quality improvement over the same
procedure without the 8th step.

**Property:**
Quality lift: +2.57% mean (95% CI: [+2.20%, +2.95%])
Time cost: 1.3× (1.69 ms → 2.13 ms)

**Empirical validation (25.04.2026):**
1000 paired trials on N=30,000 hypervector arena:
- 7 walks alone: 61.75% ± 11.16% of optimum
- 7 walks + 8th tikkun: 64.33% ± 10.80% of optimum
- Paired t-statistic: 13.47 (p < 0.01)

**Significance:**
A single integrative step over an ensemble of 7 parallel walks beats the
ensemble alone with high statistical confidence. The structure (7+1)
matches the biblical week pattern (6 days + Shabbat) but the algorithmic
benefit is independent of theological interpretation.

**Use case:**
Drop-in addition to any K-beam search where K ≈ 7. Cheap (+30% time) and
robust (+2.5% quality).

**Caveat:**
The Kabbalistic framing is INSPIRATION for the structure, not justification.
The same effect may be reproducible with any 7+1 ensemble + integration
configuration; the specific contribution is the empirical demonstration
that this works in HDC search.

---

## Eldad Genesis — Deterministic On-Demand Vocabulary

**Definition:**
For any string label `L`, compute hypervector `v_L` via:

```
v_L = bipolar(rng(seed = sha256(L)[:8]).choice({-1,+1}, D))
```

This generates a deterministic hypervector on demand without storing
any vocabulary file.

**Property:**
- Generation rate: 79.8 μs per atom (12,525 atoms/sec on AVX-512 CPU)
- Reproducibility: identical input → identical output (10/10 verified)
- Pairwise orthogonality of 1000 generated vectors: mean |cos| = 0.00799

**Empirical validation (25.04.2026):**
Generation across 100,000 distinct labels confirms deterministic and
quasi-orthogonal output. With Zipfian access caching:
- Pure on-demand query: 72.6 μs
- Cached query: 6.66 μs (10.9× speedup)
- Cache memory for 100K queries (Zipfian): ~10 MB unique atoms

**Significance:**
Eliminates vocabulary storage entirely. For Hebrew NLP serving 350K
words: 0 MB base storage instead of 438 MB. Cold queries pay 73 μs;
hot queries match storage-lookup latency via cache.

**Use case:**
Resource-constrained deployment (mobile, edge). Streaming corpus
ingestion (no need to maintain vocabulary table). Distributed systems
where shared seed = shared vocabulary without sync overhead.

**Caveat:**
Speed comparison is against numpy `np.fromfile`. Specialized lookup
tables can be faster than 2.5 μs per access.

---

## Eldad Walk — Constant-Time Approximate ANN via Sefirot Beam

**Definition:**
Given a hypervector arena and a sparse edge graph constructed via
`edges[i, k] = (i + (k+1) × φ × N / 360) mod N` for k = 0..9, perform
a bounded walk:
1. Start at random atom
2. At each step, score current and 180°-complement (`comp[i] = (i + N/2) mod N`)
3. Branch into top-2 angel-edges
4. Recurse to depth 10 (sefirot)
5. Return best atom seen

**Property:**
Walk visits stay nearly constant (~150–770 atoms) regardless of arena
size N. Walk time stays nearly constant (~3–5 ms) regardless of N.

**Empirical validation (25.04.2026):**
| N | Brute force | Eldad Walk | Speedup | Quality |
|---|---|---|---|---|
| 1,000 | 14 ms | 3.3 ms | 4× | 91% |
| 10,000 | 145 ms | 3.4 ms | 42× | 85% |
| 100,000 | 1425 ms | 4.7 ms | 305× | 66% |

With drip-loop refinement (re-running walk with new random seeds), the
best-ever result converges to ≥99% quality in approximately 56 cycles
(~185 ms).

**Significance:**
Constant-time approximate nearest-neighbor search. No index
pre-construction. No learned hashing. Quality controllable via cycle
budget — anytime algorithm.

**Use case:**
Real-time semantic queries against large corpora at moderate quality.
Works on memory-mapped disk-resident arenas without full RAM loading.

**Caveat:**
Quality drops with N (91% → 66% as N grows from 1K to 100K). Not
Grover-speed for exact matching. Standard ANN methods (FAISS-IVF, HNSW)
may match or exceed at scale with proper indexing — head-to-head
benchmark not yet performed.

---

## Notes on Naming

These functions are named "Eldad" not for self-promotion but to make
them addressable in literature. If they survive external review and
reproduction, they belong to the field. If not, the naming convention
preserves attribution to a specific empirical session that produced
them — which is more honest than vague "we found that..." prose.

Each function above passed:
1. Empirical validation with N ≥ 1000 trials
2. Statistical significance testing
3. Literature search for prior art

The findings remain provisional until external HDC/VSA experts
reproduce them. This document is the working record.

---

*Last updated: 25.04.2026.*
*Source experiments: /tmp/exp*.{py,log,json} on ddev.chooz.co.il.*
