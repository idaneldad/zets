# MISSION: Agent D — Canonization Module

**Branch:** `feat/canonization`
**Estimated time:** 3-4 hours
**Priority:** STRATEGIC — solves meta-problem of variant content (religions, translations, papers, versions)
**Model:** Opus 4.7 (architectural decisions + epistemic classification)

---

## Context

Idan identified a meta-pattern: translations, religious parallels,
paper translations, versions, derivatives, citations — they're all
the same problem class.

**The problem:** Given two pieces of content, decide:
1. Is this the SAME content as something already in the graph?
2. If yes, at what level? (verbatim / semantic / conceptual / family)
3. What's the canonical reference?
4. What epistemic status does it have? (fact / tradition / opinion / fiction)
5. What's the quotation policy? (freely quotable / paraphrase only / concept only / private)

**The solution:** One generic module that classifies ANY incoming content.
After this exists, ingesting religions, translations, papers, Wikipedia
all becomes trivial — the module handles them correctly automatically.

---

## Rules of engagement

1. **Branch:** `feat/canonization` from main
2. **Scope:** NEW files in `src/canonization/` ONLY
3. You may NOT modify:
   - `src/lib.rs` (Idan will register)
   - `Cargo.toml` (no new deps without explicit permission)
   - `src/sense_graph.rs` (USE it, don't change)
   - `src/ingestion.rs` (reference its types, don't change)
   - `src/wisdom_engines/cross_tradition.rs` (reuse patterns, don't change)
   - ANY other existing file
4. **Tests:** 20+ passing, cargo test --lib clean
5. **No hallucinations:** Unclear → STOP, write to `QUESTIONS.md`

---

## Interface contract

```rust
use crate::sense_graph::{LanguageId, SenseStore};

pub struct CanonizationEngine {
    works: HashMap<WorkId, Work>,
    fingerprint_index: HashMap<Fingerprint, Vec<WorkId>>,
    sense_store: SenseStoreRef,  // uses existing sense_graph
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkId(pub u64);

#[derive(Debug, Clone)]
pub struct Work {
    pub id: WorkId,
    pub title: Option<String>,
    pub language: LanguageId,
    pub kind: WorkKind,
    pub canonical: Option<WorkId>,
    pub provenance: Provenance,
    pub epistemic: EpistemicStatus,
    pub quote_policy: QuotePolicy,
    pub fingerprint: Fingerprint,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone)]
pub enum WorkKind {
    Original,
    Translation {
        from: WorkId,
        fidelity: Fidelity,
    },
    Citation {
        of: WorkId,
        scope: CitationScope,
    },
    Derivative {
        of: WorkId,
        transform: DerivationKind,
    },
    ParallelTradition {
        family: String,          // e.g. "Abrahamic_Creation"
        siblings: Vec<WorkId>,
    },
    Version {
        of: WorkId,
        version_num: u32,
        change_summary: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fidelity {
    Faithful,      // tracks original tightly
    Loose,         // same ideas, different wording
    Adaptation,    // "inspired by"
    Retranslation, // translation of a translation (drift risk)
}

#[derive(Debug, Clone)]
pub enum DerivationKind {
    Summary,
    Paraphrase,
    Commentary,
    Critique,
    Adaptation,
}

#[derive(Debug, Clone, Copy)]
pub enum CitationScope {
    Quote { word_count: u32 },
    Reference,
    Allusion,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EpistemicStatus {
    EmpiricalFact,       // measured, reproducible
    HistoricalRecord,    // documented past event
    Tradition,           // culturally transmitted
    ReligiousNarrative,  // sacred text content
    Opinion,             // expressed viewpoint
    Theoretical,         // scientific theory
    Fiction,             // acknowledged as invented
    Mythology,           // mythic narrative
    Speculation,         // unverified claim
    Unknown,             // uncategorized
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuotePolicy {
    FreelyQuotable,       // factual, public domain, or explicit license
    Paraphraseable,       // can restate, not quote
    ConceptOnly,          // only discuss the idea, not the text
    Private,              // don't surface at all
}

#[derive(Debug, Clone)]
pub struct Provenance {
    pub source: Option<String>,
    pub author: Option<String>,
    pub published_year: Option<i32>,
    pub license: Option<String>,
    pub trust_tier: TrustTier,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrustTier {
    PeerReviewed,
    EditorialReview,
    Community,
    Anecdotal,
    Unknown,
}

// ─── The fingerprint — identity detection across languages/versions ───

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fingerprint {
    pub structural: Vec<u64>,  // sentence-structure hashes
    pub semantic: Vec<u64>,    // concept-level hashes (via sense graph)
    pub length: u32,           // word count tier
}

impl CanonizationEngine {
    pub fn new(sense_store: SenseStoreRef) -> Self;
    
    // ─── Core operations ───
    
    /// Register new content. Returns Work with variant detection + epistemic classification.
    pub fn canonize(
        &mut self,
        text: &str,
        language: LanguageId,
        provenance: Provenance,
    ) -> Result<Work, CanonizationError>;
    
    /// Detect if this text is a variant of existing works.
    pub fn detect_variants(
        &self,
        fingerprint: &Fingerprint,
        language: LanguageId,
    ) -> Vec<(WorkId, VariantMatch)>;
    
    /// Classify epistemic status from text patterns.
    pub fn classify_epistemic(
        &self,
        text: &str,
        provenance: &Provenance,
    ) -> EpistemicStatus;
    
    /// Derive quote policy from epistemic + trust + license.
    pub fn derive_quote_policy(
        &self,
        epistemic: EpistemicStatus,
        trust: TrustTier,
        license: Option<&str>,
    ) -> QuotePolicy;
    
    /// Retrieve canonical version.
    pub fn canonical_of(&self, id: WorkId) -> Option<&Work>;
    
    /// Get all variants (translations, versions, derivatives) of a work.
    pub fn variants_of(&self, id: WorkId) -> Vec<&Work>;
    
    /// Mark works as parallel tradition siblings.
    pub fn mark_parallel_tradition(
        &mut self,
        works: &[WorkId],
        family: String,
    );
}

#[derive(Debug, Clone)]
pub struct VariantMatch {
    pub similarity: f32,          // 0..1
    pub likely_kind: WorkKind,
    pub evidence: Vec<String>,    // human-readable reasons
}

#[derive(Debug)]
pub enum CanonizationError {
    InvalidInput(String),
    LanguageNotSupported(LanguageId),
    FingerprintFailed,
    ClassificationFailed,
}
```

---

## Files to create

```
src/canonization/
    mod.rs              ← module + re-exports + top-level docs
    work.rs             ← Work struct + WorkId + WorkKind + related enums
    fingerprint.rs      ← Fingerprint computation (structural + semantic)
    variant.rs          ← detect_variants logic + VariantMatch
    epistemic.rs        ← EpistemicStatus classifier (pattern-based + provenance)
    policy.rs           ← QuotePolicy derivation rules
    provenance.rs       ← Provenance + TrustTier
    engine.rs           ← CanonizationEngine main impl
    error.rs            ← CanonizationError
    README.md           ← Module documentation
```

---

## Features (in priority order)

### 1. Work storage (30min)
- `WorkId` generation (hash of title + language + provenance)
- `Work` struct with all fields
- In-memory `HashMap<WorkId, Work>`
- Retrieval + listing

### 2. Fingerprint computation (1h)
- **Structural fingerprint:** sentence boundaries + length ratios + punctuation pattern
- **Semantic fingerprint:** for each sentence, extract top-N concepts via `sense_graph`, hash them
- Length tier: word count bucketed (≤100, 100-1K, 1K-10K, 10K+)
- Stable across minor wording differences

### 3. Variant detection (1h)
- Compare new fingerprint against existing
- Similarity score: structural_match * 0.4 + semantic_match * 0.6
- Thresholds:
  - >0.95 = Translation or same content
  - 0.70-0.95 = Derivative or close paraphrase
  - 0.50-0.70 = ParallelTradition or loose translation
  - <0.50 = Unrelated
- Return `VariantMatch` with evidence

### 4. Epistemic classifier (1h)
Pattern-based rules (no LLM needed). Examples:
- `"X said"` / `"according to"` / `"tradition holds"` → Tradition
- `"measured"` / `"experiment"` / `"data shows"` → EmpiricalFact
- `"in year X"` / `"on date"` → HistoricalRecord
- `"should"` / `"ought"` / `"believe"` → Opinion
- `"created"` + religious markers → ReligiousNarrative
- `"once upon a time"` → Fiction
- `"myth"` / `"legend"` → Mythology
Plus provenance signal:
- `license == "CC-BY"` → likely fact/theory
- `source.contains("bible")` or `torah` or `quran` → ReligiousNarrative

### 5. Quote policy derivation (30min)
```
if epistemic == EmpiricalFact && trust == PeerReviewed:
    FreelyQuotable
if epistemic == ReligiousNarrative:
    ConceptOnly    // Idan's specific rule
if epistemic == Fiction && license == "copyright":
    Paraphraseable
if epistemic == Opinion && trust == Anecdotal:
    Paraphraseable
default:
    Paraphraseable
```

### 6. Parallel tradition marking (30min)
- Mark a group of works as siblings of same tradition
- Bidirectional links
- Example: Genesis creation, Quran creation, Norse Ginnungagap → `"Creation_Myths"` family

### 7. Canonize() — main pipeline (30min)
```
canonize(text, lang, prov):
    fp = compute_fingerprint(text)
    variants = detect_variants(fp, lang)
    if any variant with similarity > 0.95:
        return Work as Translation { from: variant.canonical }
    epistemic = classify_epistemic(text, prov)
    policy = derive_quote_policy(epistemic, prov.trust)
    return new Work { ... }
```

---

## Test requirements (20+ tests)

```rust
#[test] fn test_empty_engine() {}
#[test] fn test_register_first_work_is_original() {}
#[test] fn test_register_translation_detected_as_variant() {}
#[test] fn test_fingerprint_structural_stable() {}
#[test] fn test_fingerprint_semantic_via_sense() {}
#[test] fn test_variant_detection_above_threshold() {}
#[test] fn test_variant_detection_below_threshold_means_original() {}
#[test] fn test_epistemic_tradition_pattern() {}
#[test] fn test_epistemic_empirical_pattern() {}
#[test] fn test_epistemic_religious_narrative_pattern() {}
#[test] fn test_epistemic_historical_pattern() {}
#[test] fn test_epistemic_fiction_pattern() {}
#[test] fn test_policy_religious_is_concept_only() {}
#[test] fn test_policy_empirical_peer_reviewed_is_freely_quotable() {}
#[test] fn test_policy_opinion_is_paraphraseable() {}
#[test] fn test_parallel_tradition_creates_siblings() {}
#[test] fn test_canonical_of_returns_original() {}
#[test] fn test_variants_of_returns_all() {}
#[test] fn test_retranslation_drift_detected() {}
#[test] fn test_hebrew_english_translation_recognized() {}
#[test] fn test_citation_chain_preserved() {}
#[test] fn test_summary_is_derivative_not_translation() {}
```

---

## Scenario tests

```rust
#[test]
fn test_torah_verse_english_hebrew_recognized_as_variants() {
    // Insert Hebrew Genesis 1:1
    // Insert English KJV "In the beginning..."
    // Expect: variant detection above 0.90
    // Both: EpistemicStatus::ReligiousNarrative
    // Both: QuotePolicy::ConceptOnly
}

#[test]
fn test_scientific_paper_he_en_recognized() {
    // Insert English abstract
    // Insert Hebrew translation
    // Expect: Translation { from: english, fidelity: Faithful }
    // EpistemicStatus: EmpiricalFact or Theoretical
    // QuotePolicy: FreelyQuotable (if CC)
}

#[test]
fn test_creation_myths_parallel_tradition() {
    // Register Genesis creation
    // Register Norse Ginnungagap
    // Mark as parallel tradition "Creation_Myths"
    // Verify bidirectional links
    // Verify all have ReligiousNarrative or Mythology status
}
```

---

## Done criteria

1. ✅ 9 source files created in `src/canonization/`
2. ✅ 20+ tests, all passing
3. ✅ `cargo build --lib` clean
4. ✅ `cargo clippy --lib` no new warnings
5. ✅ README.md documenting the module
6. ✅ All commits on `feat/canonization`
7. ✅ `git push origin feat/canonization`
8. ✅ PR to main with clear description

---

## When blocked

Write to `/home/dinio/agent-logs/agent-d-questions.md`. Then STOP. Do not guess.

---

## Final instruction

After implementation:
```bash
cd /home/dinio/zets-agent-d
cargo test --lib canonization 2>&1 | tail -30
# Must show X passed, 0 failed

git push origin feat/canonization
```

Create PR titled "Add canonization module: variant detection + epistemic classification".
Tag Idan for review.

This module is strategic — it unblocks religious texts, translations, papers,
and versioning all at once. Take your time, get it right.

Start now.
