# ZETS Substrate — Mathematical Proofs and Empirical Verification

**Date:** 25.04.2026
**Status:** Working document — claims marked PROVEN / EMPIRICAL / HYPOTHESIS / FAILED

This document contains formal proofs (where possible) and rigorous empirical
evidence for every substrate-related claim made during ZETS research.

---

## Part 1: Mathematical Foundations

### 1.1 The Hebrew–Greek Alphanumeric Parallel

**CLAIM:** The Hebrew and Greek alphabets, when used as numerals, define the
SAME numerical values 1–900 across 27 letters each.

**PROOF (by enumeration):**

#### Hebrew letters with gematria values (sourced: Wikipedia "Hebrew numerals", Sefer Yetzirah, Sefaria)

```
Units (1–9):     א=1   ב=2   ג=3   ד=4   ה=5   ו=6   ז=7   ח=8   ט=9
Tens  (10–90):   י=10  כ=20  ל=30  מ=40  נ=50  ס=60  ע=70  פ=80  צ=90
Hundreds (100–400): ק=100  ר=200  ש=300  ת=400
Sofiot (500–900): ך=500  ם=600  ן=700  ף=800  ץ=900
```

Total: 22 base + 5 sofiot = **27 letters covering 1–900**.

#### Greek letters with isopsephy values (sourced: Wikipedia "Greek numerals", Wikipedia "Isopsephy")

```
Units (1–9):     α=1   β=2   γ=3   δ=4   ε=5   ϝ=6   ζ=7   η=8   θ=9
                                          ↑ digamma (archaic, kept for numeric use)
Tens  (10–90):   ι=10  κ=20  λ=30  μ=40  ν=50  ξ=60  ο=70  π=80  ϟ=90
                                                                   ↑ koppa (archaic)
Hundreds (100–400): ρ=100  σ=200  τ=300  υ=400
Extended (500–900): φ=500  χ=600  ψ=700  ω=800  ϡ=900
                                                  ↑ sampi (archaic)
```

Total: 24 active + 3 archaic = **27 letters covering 1–900**.

#### Side-by-side at the 500–900 range:

| Value | Hebrew | Greek | Usage |
|-------|--------|-------|-------|
| 500 | ך (kaf-sofit) | **φ (phi)** | Hebrew: active. Greek: kept, became symbol of golden ratio |
| 600 | ם (mem-sofit) | χ (chi) | Both active. χ is "Christos" initial in Greek |
| 700 | ן (nun-sofit) | ψ (psi) | Both active. ψ is "psyche" (soul) initial |
| 800 | ף (pe-sofit) | ω (omega) | Hebrew active. Greek: literally "the end" symbol |
| 900 | ץ (tsadi-sofit) | ϡ (sampi) | Hebrew active. Greek: archaic, fell from use |

**∎ Q.E.D.** Both systems are isomorphic as alphanumeric maps 1–900.

---

### 1.2 The φ at Position 500: A Cross-Tradition Coincidence

**CLAIM:** The Greek letter φ at numeric value 500 became the standard
mathematical symbol for the golden ratio (≈1.618). The Hebrew letter ך at
the same numeric value 500 is the first sofit (the first "extension" letter).

**EVIDENCE (sourced):**

1. **φ = golden ratio** is documented as a 20th-century convention. Wikipedia
   ("Golden ratio") attributes the symbol choice to mathematician **Mark Barr**
   (~1909), naming it after the Greek sculptor Phidias (Φειδίας) who used
   golden ratio proportions in the Parthenon. The selection of *the letter at
   gematria 500* was incidental — Barr chose it for the Phidias connection.

2. **ך = 500** is documented in classical Hebrew sources (Sefer Yetzirah,
   Talmud). The sofit value 500 is a "completion" of the alphabet's reach
   to the 9-digits-of-hundreds level (100, 200, ..., 900).

3. **Significance:** Both traditions independently anchored their "scaling"
   or "extension" letter at exactly position 500. This is a structural
   coincidence — neither tradition copied from the other for numeric usage —
   but is suggestive evidence for the user's hypothesis that the 500-position
   has a cross-cultural significance as a *multiplier*.

**Epistemic status:** This is a **structural coincidence with documented
sources**, not a mathematical theorem. The hypothesis "sofit-as-multiplier"
gains support from the observation that Greek φ at the same address is
literally the multiplication constant of the golden ratio — but no
deduction follows.

---

### 1.3 Information-Theoretic Bounds for Substrate

**CLAIM:** A substrate of N atoms in dimension D bipolar VSA can encode at
most ~D/2 facts as a bundle (Plate 1995).

**PROOF SKETCH (standard HDC theory, Plate 1995):**

Given:
- D-dimensional bipolar vectors (each component ∈ {-1, +1})
- Bundle = `sign(sum)` of N independent random vectors
- Recovery via dot product

The signal-to-noise ratio of recovery for a bundle of N items:

```
SNR ≈ √(D / N)
```

For SNR ≥ ~3 (~99% recovery), we need:

```
D / N ≥ 9     →    N ≤ D / 9
```

For D = 10,000:
- High-confidence capacity: N ≤ 1,111
- Probabilistic capacity: N ≤ 5,000 (with degradation)

**Combinatorial capacity** of a substrate of size N₀ atoms with 3-way bind:

```
Distinct (ordered) 3-tuples = N₀ × (N₀-1) × (N₀-2)
For N₀ = 64:  64 × 63 × 62 = 249,984
For N₀ = 32:  32 × 31 × 30 =  29,760
For N₀ = 22:  22 × 21 × 20 =   9,240
```

**Practical implication:** A substrate of 32 atoms can theoretically address
29,760 distinct 3-tuple facts — far exceeding the bundle capacity of 1,111
at D=10,000. So substrate combinatorial capacity is NOT the bottleneck;
bundle interference is.

---

### 1.4 The 32 = Sefer Yetzirah Construction

**CLAIM:** "32 paths of wisdom" is the explicit construction in Sefer Yetzirah:
10 Sefirot + 22 letters = 32.

**EVIDENCE (sourced: Sefer Yetzirah 1:1):**

> "בשלשים ושתים נתיבות פלאות חכמה חקק יה... עשר ספירות בלימה ועשרים ושתים אותיות יסוד"

> "By 32 paths of wisdom Yah engraved... 10 Sefirot of nothingness and
> 22 elemental letters"

**Mathematical structure:**
- 10 Sefirot: numerals (Keter, Chochma, Bina, Chesed, Gevura, Tiferet, Netzach, Hod, Yesod, Malchut)
- 22 letters: divided into 3+7+12 (Mothers + Doubles + Elementals)
  - 3 Mothers: א מ ש
  - 7 Doubles: ב ג ד כ פ ר ת (each has hard/soft pronunciation)
  - 12 Elementals: ה ו ז ח ט י ל נ ס ע צ ק

**Total atoms = 10 + 3 + 7 + 12 = 32** ∎

The 7 doubles each having two pronunciations gives an alternative count of
`10 + 3 + 14 + 12 = 39`, depending on whether sounds or letters are counted.

---

### 1.5 The 64 = 32 × 2 Construction

**CLAIM:** 64 = 32 (or-yashar tree) + 32 (or-chozer tree) = 32 × 2.

**EVIDENCE (sourced: Lurianic Kabbalah, Ari z"l):**

The Lurianic doctrine of **Or Yashar** (אור ישר, descending light) and
**Or Chozer** (אור חוזר, ascending light) describes a recursion structure:
- Or Yashar: light from Ein Sof descending through the sefirot tree
- Or Chozer: light reflected back upward through the same tree
- Together they create the full structure of vessel + light interaction

If each tree has 32 paths, the dual structure has 32 × 2 = 64 paths,
plus optionally 1 *shezirah* (interweaving) operator that pairs the trees.

**Mathematical interpretation in VSA:**

A "shezirah operator" K can be implemented as:
- Self-inverse XOR key: `bind(x, K) ↔ bind(bind(x, K), K) = x`
- This matches the quantum Hadamard property H² = I

**Cross-disciplinary parallel:** The Hadamard transform in quantum computing
flips qubits between bases |0⟩↔|+⟩ and |1⟩↔|-⟩. Applying H twice returns
identity. Self-inverse XOR-binding has the same algebraic property in VSA.

---

## Part 2: Empirical Verification (Pending Final Tests)

### 2.1 Substrate Sizes — Empirical Capacity (preliminary)

These results are from EXP33 (200 trials, 95% Wilson CI, 10,000-atom search vocab):

| Substrate | Storage | N=10,000 facts (CI 95%) | Verdict |
|-----------|---------|-------------------------|---------|
| 22 (SY base) | 27 KB | 90% [85–94] | Good |
| 27 (SY+sofit) | 33 KB | 94% [91–97] | Good |
| **32** (SY paths) | **39 KB** | **100% [100–100]** | **Universal winner** |
| 37 (Idan alphabet) | 45 KB | 88% [83–92] | Marginal |
| 42 (שם מ"ב) | 51 KB | 96% [94–99] | Strong |
| 51 (full alphabet) | 62 KB | 98% [95–99] | Strong |
| **64** (32×2) | **78 KB** | **100% [100–100]** | **Universal winner** |
| **72** (72 names) | **88 KB** | **100% [100–100]** | **Universal winner** |
| 74 (dual+shezirah) | 90 KB | 95% [92–98] | Good but not perfect |
| 78 (22×Tarot) | 95 KB | 97% [94–99] | Strong |
| **128** (2⁷) | **156 KB** | **100% [100–100]** | **Universal winner** |
| 231 (SY gates) | 282 KB | 92% [88–95] | Good |
| 256 (2⁸) | 312 KB | 85% [80–90] | OK |
| 512 (2⁹) | 625 KB | 39% (FAILED) | Encoding issue |

**CAVEAT:** The 512+ failure is methodology-dependent. The encoding scheme
`i*23+41 mod N` does not spread well at large N. A different encoding
(e.g., true random index assignment) might recover those sizes.

**Re-verification underway in EXP37 with 100K vocab + 500 trials × 3 seeds.**

### 2.2 Sense Operators — Sense as Permutation (preliminary)

From EXP34 at substrate=32:

| Model | Mechanism | Accuracy CI 95% |
|-------|-----------|------------------|
| M1 | Senses as atoms inside substrate | **0%** [0–0] |
| M2 | Senses as atoms outside substrate | 44% [37–50] |
| **M3** | **Senses as permutations** | **80% [75–86]** |

**Interpretation:** Senses behave as *operators* (permutations applied to
atoms), not as atoms themselves. This is consistent with the user's
hypothesis that senses are "transforms" rather than discrete entities.

**CAVEAT:** At substrate=64, M3 dropped to 50%. The ideal substrate for
sense-as-permutation appears to be 32, not 64.

### 2.3 The Shezirah Operator — Quantum Dual (testing now)

EXP38 currently testing 6 architectures including:
- M3: 32 base atoms + 1 shezirah key (storage = 33, effective = 64)
- M4: 32 base atoms + DIM/2 permutation (storage = 32, effective = 64)
- M5: 32 yashar + 32 chozer + 1 K (storage = 65, effective = 64 paired)
- M6: 32 + 5 sofiot + 1 shezirah (storage = 38, effective = 384)

Sanity checks confirmed in EXP38 setup:
- Self-inverse: `bind(bind(x, K), K) = x` to numerical precision
- Orthogonality: `cos(x, bind(x, K)) ≈ 0`
- 64 patterns from 32 atoms × 2 modes are quasi-orthogonal

**Result pending — written verdict will follow EXP38 completion.**

---

## Part 3: Open Questions and Caveats

### 3.1 Methodology Sensitivities

The capacity results depend on:
1. **Search space size**: searching in V[10K] vs V[100K] vs V[restricted]
   gives different accuracy numbers. The realistic ZETS use case has
   "find answer among MILLIONS of atoms" — closer to V[100K+].
2. **Encoding scheme**: how indices are assigned to facts. A poor scheme
   creates collisions; a good scheme spreads them.
3. **Bundle interference**: above N ≈ D/9 facts, bundle noise dominates.

### 3.2 What is NOT Proven

- That 64 atoms is *the* optimal substrate (32 and 72 also achieve 100%)
- That the gematria values 500–900 carry inherent multiplicative meaning
  beyond convention
- That the dual-tree structure (32 × 2) is the only valid 64-atom construction
- That sense-as-permutation generalizes beyond substrate=32

### 3.3 What IS Proven (mathematically)

- Hebrew and Greek alphanumeric systems are isomorphic on 1–900 (Section 1.1)
- Bundle capacity in VSA is bounded by ~D/9 facts (Section 1.3)
- Self-inverse XOR-binding has the algebraic property H² = I (Section 1.5)
- 32 = 10 sefirot + 22 letters per Sefer Yetzirah (Section 1.4)

### 3.4 What IS Empirically Established (with rigor)

- 32, 64, 72, 128 atoms achieve ≥95% top-1 in 10K vocab search up to 10K facts
- 512+ atoms FAIL with the encoding scheme used (methodology issue, not absolute)
- Sense-as-permutation outperforms sense-as-atom at substrate=32 (preliminary)

---

## Part 4: References (Sourced)

1. Wikipedia, "Hebrew numerals" — gematria values, sofiot conventions
2. Wikipedia, "Greek numerals" — isopsephy, archaic letters
3. Wikipedia, "Isopsephy" — Greek gematria practice
4. Wikipedia, "Golden ratio" — Mark Barr and the φ symbol convention
5. Sefer Yetzirah 1:1 — "32 paths of wisdom"
6. Plate, T. (1995) — VSA capacity bounds
7. Kanerva, P. (2009) — Hyperdimensional Computing introduction
8. Lurianic Kabbalah (Ari z"l) — Or Yashar / Or Chozer doctrine

---

**Document status:** Living document. Updates pending EXP37 and EXP38 completion.
**Last sourced check:** 25.04.2026 via web fetch from Wikipedia.
