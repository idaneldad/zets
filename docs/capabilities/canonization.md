# Canonization — Variant Detection + Epistemic Classification

**Module:** `src/canonization/`
**Status:** 🟢 0.70 / 1.00
**Landed:** 23.04.2026 (Agent D, Opus 4.7)
**Tests:** 32 passing
**LOC:** ~1,713

## מה המשימה

תובנה של עידן (23.04.2026):

> דתות, תרגומים, מאמרים מתורגמים, גרסאות — אלה **אותה בעיה**. "האם זה אותו חומר?"

Canonization הוא המודול הגנרי שמטפל בשאלה הזו. אחרי שהוא קיים, ZETS יכול **לבד** להתמודד עם:
- תרגומים בין שפות
- מסורות דתיות מקבילות (תורה ↔ קוראן ↔ ברית)
- גרסאות שונות של אותו מסמך
- מאמרים שמצטטים מאמרים אחרים
- סיכומים ועיבודים
- תרגום-של-תרגום (drift)

## הקריטריון להצלחה

- [x] Identity detection — האם שני טקסטים הם "אותו דבר"?
- [x] Fingerprint (structural + semantic) יציב
- [x] Variant kinds: Original / Translation / Citation / Derivative / ParallelTradition / Version
- [x] Epistemic classification: Fact / Tradition / Opinion / ReligiousNarrative / Historical / Fiction / Mythology / Theoretical
- [x] QuotePolicy derivation: FreelyQuotable / Paraphraseable / ConceptOnly / Private
- [x] **עידן's rule:** ReligiousNarrative → ConceptOnly (ZETS יודע, לא מצטט)
- [x] Parallel tradition marking (Creation_Myths: Genesis + Quran + Norse)

## איך בוחנים (32 tests)

### QA (נכונות classification)
- Epistemic patterns: tradition ("X said"), empirical ("measured"), religious, historical, fiction, mythology, opinion
- Quote policy: religious→ConceptOnly, empirical+PeerReviewed→FreelyQuotable
- Parallel tradition: Creation_Myths family with bidirectional siblings
- Variant thresholds: >0.95 = Translation, 0.70-0.95 = Derivative, 0.50-0.70 = ParallelTradition

### Scenario tests (real-world)
- **Hebrew/English Genesis** recognized via shared senses + religious patterns
- **Paper HE/EN translation** detected as Faithful translation
- **Creation_Myths:** Genesis + Norse Ginnungagap + Quran creation → parallel tradition
- **Retranslation drift** detected (HE→EN→FR)
- **Citation chain** preserved (A cites B cites C)

## באחריות

**גרף** (graph-native). Fingerprinting משתמש ב-`sense_graph.rs` (קיים) למיפוי cross-lingual.

## קוד

```
src/canonization/
├── mod.rs              (709 lines) — module root + 32 tests
├── engine.rs           (273 lines) — CanonizationEngine orchestrator
├── work.rs             (107 lines) — Work + WorkKind + Fidelity
├── fingerprint.rs      (210 lines) — structural + semantic fingerprint
├── variant.rs          (115 lines) — detect_variants + VariantMatch
├── epistemic.rs        (160 lines) — pattern-based classifier (no LLM)
├── policy.rs           (72 lines)  — QuotePolicy derivation rules
├── provenance.rs       (44 lines)  — Provenance + TrustTier
├── error.rs            (23 lines)  — CanonizationError
└── README.md           (83 lines)
```

## Interface

```rust
pub struct CanonizationEngine { /* ... */ }

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
}

pub enum WorkKind {
    Original,
    Translation { from: WorkId, fidelity: Fidelity },
    Citation { of: WorkId, scope: CitationScope },
    Derivative { of: WorkId, transform: DerivationKind },
    ParallelTradition { family: String, siblings: Vec<WorkId> },
    Version { of: WorkId, version_num: u32 },
}

pub enum EpistemicStatus {
    EmpiricalFact,
    HistoricalRecord,
    Tradition,
    ReligiousNarrative,
    Opinion,
    Theoretical,
    Fiction,
    Mythology,
    Speculation,
    Unknown,
}

pub enum QuotePolicy {
    FreelyQuotable,
    Paraphraseable,
    ConceptOnly,       // ← Idan's rule for religious
    Private,
}

impl CanonizationEngine {
    pub fn canonize(&mut self, text: &str, lang: LanguageId, prov: Provenance) -> Result<Work, CanonizationError>;
    pub fn detect_variants(&self, fp: &Fingerprint, lang: LanguageId) -> Vec<(WorkId, VariantMatch)>;
    pub fn classify_epistemic(&self, text: &str, prov: &Provenance) -> EpistemicStatus;
    pub fn derive_quote_policy(&self, epistemic: EpistemicStatus, trust: TrustTier, license: Option<&str>) -> QuotePolicy;
    pub fn mark_parallel_tradition(&mut self, works: &[WorkId], family: String);
}
```

## Pipeline

```
Input: text + language + provenance
  ↓
1. compute_fingerprint(text)  ← structural + semantic
  ↓
2. detect_variants(fp, lang) ← cross-reference existing works
  ↓
3. If similarity > 0.95 → Translation of existing
  If 0.70-0.95 → Derivative
  If 0.50-0.70 → ParallelTradition
  If < 0.50 → Original
  ↓
4. classify_epistemic(text, prov) ← pattern-based
  ↓
5. derive_quote_policy(epistemic, trust, license)
  ↓
6. Store Work with all metadata
```

## Why this matters

**The generic solution.** Once this module exists, everything else is **data ingestion**:
- Feed תורה → ZETS auto-classifies as ReligiousNarrative → ConceptOnly policy
- Feed paper (EN) then paper (HE) → ZETS auto-links as Translation
- Feed Wikipedia (48 languages) → ZETS auto-canonicalizes across languages
- Feed scientific paper + summary → ZETS auto-links as Derivative

**No need to code per-religion or per-format logic.** The mechanism is universal.

## פער (מה חסר להגיע ל-1.00)

1. **Scale testing** — currently tested on small corpora. Scale to 17GB Wikipedia?
2. **Deeper semantic fingerprint** — today: concept-bag; target: ordered-concept-sequence
3. **LLM-assisted classification for edge cases** — some texts are hybrid (historical + religious)
4. **Confidence surfacing** — when variant match is borderline, surface uncertainty
5. **UI for manual override** — if ZETS misclassifies, user corrects, it learns
6. **Integration with ingestion pipeline** — currently standalone; wire to `src/ingestion.rs`

## Impact על HumannessScore

Meta category (new): 0.70
Unblocks future ingestion of:
- Religious texts (Tanakh, Quran, New Testament, Bhagavad Gita)
- 48-language Wikipedia corpus
- Scientific papers in multiple languages
- Multi-version documents

**Strategic value:** Samsung's RDFox doesn't do this. Neo4j doesn't do this. This is defensible IP.
