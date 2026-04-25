# ZETS AGI Specification — Iteration 1 Architecture Review

## Top 5 Critical Issues

---

**ISS-01: Canonical Layout vs Implementation Bit-Smash (Severity: CRITICAL)**
- **Section affected:** §0.2 [BINDING] vs §5.2 (implementation)
- **Severity:** critical | **Confidence:** 95
- **Claim:** §0.2 defines `semantic_id` as 19 bits (bit 18..0) while §5.2 allocates 27 bits (bit 26..0). Root field position differs (bit 49..32 vs bit 55..38). These are mutually exclusive — the same 64-bit word cannot satisfy both.
- **Proposed patch:** Designate §5.2 implementation as authoritative. Revise §0.2 to match: `semantic_id: u27` (bits 26..0), `root_encoded: u18` (bits 55..38), `definite: u1` (bit 27), `pgn: u4` (bits 31..28). Add migration note: "ABI v1 atom used 19-bit semantic_id; v1.1+ uses 27-bit."
- **Hidden assumption:** §0.2 is the "binding" canonical while §5.2 is "implementation detail." But binding implies code conformance, not override.
- **Strongest self-objection:** Idan may have intentionally kept §0.2 as abstract interface with §5.2 as concrete optimization. But a 64-bit atom has only one layout.
- **Validation test:** Compile-time assertion: `const _: () = assert!(std::mem::size_of::<Atom>() == 8);` plus `Atom::SEM_MASK == 0x7FFF_FFFF` (27 bits). Verify against §0.2 claim `0x7FFFF` (19 bits).

---

**ISS-02: AtomKind Enum Divergence (Severity: CRITICAL)**
- **Section affected:** §0.3 [BINDING] vs §5.1 (implementation)
- **Severity:** critical | **Confidence:** 90
- **Claim:** §0.3 specifies 16 AtomKind values (4 bits, hex 0x0–0xF). §5.1 defines only 12 variants with different assignments (HebrewWord=0x0, ArabicWord=0x1, AramaicWord=0x2, ForeignWord=0x3, Logographic=0x4, Concept=0x5, PhraseLemma=0x6, Procedure=0x7, Action=0x8, Media=0x9, Numeral=0xA, Rule=0xB, Personal=0xC, Meta=0xD, Reserved_E=0xE, Reserved_F=0xF). Two values (SourceAtom, TrustAtom) exist in §0.3 but not §5.1. Two more (MotifAtom, ObservationAtom) exist in §0.3 with no §5.1 equivalent.
- **Proposed patch:** Reconcile into unified enum in `src/atoms.rs`:
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AtomKind {
    Lexical    = 0x0,  // Unified Hebrew/Arabic/Aramaic/foreign via language_id
    Concept    = 0x1,
    Edge       = 0x2,
    Radical    = 0x3,
    Procedure  = 0x4,
    Rule       = 0x5,
    Source     = 0x6,
    Sense      = 0x7,
    Context    = 0x8,
    Time       = 0x9,
    Parse      = 0xA,
    Observation= 0xB,
    Goal       = 0xC,
    Trust      = 0xD,
    Motif      = 0xE,
    Reserved   = 0xF,
}
```
- **Hidden assumption:** §0.3 reflects design-day thinking while §5.1 reflects implementation pivot toward unified Lexical. The canonical section wasn't updated.
- **Strongest self-objection:** §0.3 is marked [BINDING] and explicitly states "Any contradiction...must be resolved IN FAVOR OF THIS SECTION." My patch violates this binding.
- **Validation test:** Generate `AtomKind` exhaustive match coverage. Any unhandled variant = compile error. Verify all 16 values present and accounted for.

---

**ISS-03: EdgeKind u8 vs u16 Collision (Severity: CRITICAL)**
- **Section affected:** §0.4 [BINDING] vs §5.5 (implementation)
- **Severity:** critical | **Confidence:** 98
- **Claim:** §0.4 explicitly states "EdgeKind is u16 (2 bytes)" and defines ranges up to 0xFFFF. §5.5 implements `EdgeHot` with 8-bit edge_kind field (bit 47..40), capping at 256 types. Application-specific types (CHOOZ, etc.) at 0x200..0xFFFF cannot fit.
- **Proposed patch:** Either (a) change §5.5 EdgeHot to u16 edge_kind (expanding to 8 bytes, breaking 6-byte hot-path promise), or (b) change §0.4 to cap application types at 0xFF and defer higher values to "extended edge index." Recommend (b): "Primary semantics use 0x00..0xFF; 0x100..0xFFFF deferred to ABI v2."
```rust
// Revised §5.5 EdgeHot:
pub struct EdgeHot {
    pub packed: [u8; 8],  // u16 edge_kind + u16 strength+freshness + u32 target
}
```
- **Hidden assumption:** The 6-byte hot path is performance-critical. But §0.4's u16 claim predates the 6-byte decision.
- **Strongest self-objection:** Expanding to 8 bytes increases edge storage from 6GB to 8GB on 1B edges. Exceeds 6GB RAM target if mmap'd.
- **Validation test:** Count actual edge types in use. If <256, u8 is fine. If ≥256, must resolve conflict.

---

**ISS-04: Hebrew-First Canonical Undefined for Shared-Root Disambiguation (Severity: IMPORTANT)**
- **Section affected:** §4.4.3, §6.2
- **Severity:** important | **Confidence:** 85
- **Claim:** "Accept the merge" for Hebrew/Arabic when letters differ (ث/س→ש, ذ/ز→ז, ض/צ, etc.) loses 4+ consonant distinctions. Hebrew and Arabic morphologies differ substantially (binyan systems don't map 1:1, Hebrew has 7 binyanot vs Arabic 10+ forms). When Hebrew root "ש.ל.ם" (shalom) shares atom with Arabic "س.ل.م" (salam), how are morphological features (tense, voice) disambiguated without duplicating atoms?
- **Proposed patch:** Add explicit policy to §6.2:
```rust
/// Hebrew-First Canonical with Arabic Shared-Root Policy:
/// 
/// 1. Base3 letter encoding: shared consonants map to same base37 code.
///    Where Arabic distinguishes letters Hebrew merges, the Hebrew
///    encoding is canonical. Lossy but deterministic.
/// 2. Morphology disambiguation: binyan field discriminates
///    morphological differences. If Hebrew binyan Paal vs Arabic
///    Form-I share same root+features, they are SAME atom (canonical
///    uses Hebrew binyan numbering).
/// 3. Fine-grained distinctions preserved in semantic_id:
///    Arabic-specific roots get semantic_id offset +0x10000,
///    Hebrew get 0x00000. Same root, different semantic context.
/// 4. Language_id field enables rendering choice at realization.
```
- **Hidden assumption:** The morphological differences are shallow enough that shared atoms suffice. But Hebrew/Arabic verbal systems differ significantly.
- **Strongest self-objection:** Arabic linguists would reject Hebrew-binyan-numbering as canonical. This is Idan's design choice, but it should be explicit.
- **Validation test:** Ingest parallel Hebrew/Arabic parallel corpus (Quran, Bible). Measure: (a) atom count reduction vs separate languages, (b) accuracy of morphological parsing on held-out Arabic text.

---

**ISS-05: Gematria Hash Claim Not Validated (Severity: IMPORTANT)**
- **Section affected:** §13.4 (Gematria as structural hash), §2.4 (quantum disclosure)
- **Severity:** important | **Confidence:** 70
- **Claim:** Section presents three-independent-model convergence on Gematria-as-hash as breakthrough insight. But "convergence of independent models" is not empirical validation — it's confirmation bias signaling. No performance numbers, no ablation study, no collision rate analysis. Gematria has ~4,000 possible values (sum of standard gematria max ≈ 22+21+20 = ~400 max per 3-letter root; 400² collision rate?). If collision rate is high, gematria-as-hash fails.
- **Proposed patch:** Add validation section to §13.4:
```rust
/// Gematria Hash Validation:
/// 
/// Gematria of 3-letter root r: g(r) = Σ letter_value(c_i)
/// Letter values: א=1..ט=9, י=10, כ=20..צ=90, ק=100..ת=400
/// Max gematria for 3-letter root: 400+300+100 = 800
/// Collision rate: g(r₁) == g(r₂) for random roots ≈ 1/800
/// 
/// For analogy detection: r₁ ~ r₂ if g(r₁) == g(r₂)
/// Precision: high (true positives strongly correlated)
/// Recall: low (many same-meaning pairs have different gematria)
/// 
/// Proposed use: O(1) candidate pre-filter for analogy check.
/// Full analogy requires full walk. Gematria hash is quick reject only.
```
- **Hidden assumption:** The three independent models are truly independent. But they were all asked the same question by the same human (Idan).
- **Strongest self-objection:** The Gematria insight may still be correct and valuable — just as a quick pre-filter, not a full analogy engine. My objection is to the unvalidated magnitude of the claim.
- **Validation test:** Compute gematria for 1000 known Hebrew root pairs (synonyms, antonyms, cognates). Measure: (a) same-gematria rate for true analogies vs random pairs, (b) ROC AUC for gematria-only analogy detection. Target: AUC > 0.65 to justify inclusion.

---

## Top 3 Strengths (Briefly)

**S1 — Unified Atom ABI with Hebrew-First Canonical** (Confidence: 92)
The 8-byte atom with base37 encoding and Hebrew-first canonical is architecturally sound. 18 bits for root + 6-bit language_id + morphological fields gives excellent coverage. The 30-year ABI commitment with reserved bits is responsible forward planning.

**S2 — Determinism Boundary Clarity** (Confidence: 95)
§0.5 explicitly defines what ZETS does and does NOT guarantee