# Language Re-Learn — המערכת לומדת שפה דרך ה-meta-rules שלה

**תאריך:** 22.04.2026 (evening)
**בקשת עידן (earlier turn):** "כל שפה שוקחת מעט תלמד אותו את כל השפות שיש לך יכולת
לדעת גם אם זה מאות שפות... שהמערכת תלמד שוב וככה תוכל להכניס לגרף הוראות למידת שפה
ולראות שזה פרפקט"

---

## הרעיון בצורה הכי תמציתית

היום, ZETS יודע שפות (A = Wikipedia text, B = pack per-lang). אבל המערכת **לא יודעת
איך למדה** אותן — אין explicit meta-rules על "איך להכניס שפה חדשה לגרף".

**התובנה:** אם המערכת תלמד את **הכלל** ("ככה לומדים שפה"), לא רק את **התוכן**,
אז שפה #17 תלמד את עצמה מ-corpus קטן. שפה #100 כמעט חינם.

זה בדיוק עיקרון הhomoiconic של ZETS — "system rules as nodes in graph itself".

---

## למה זה שונה מ-zero-shot / few-shot learning?

- **Zero-shot LLM**: מודל שכבר למד 100 שפות מזיכרון בפרה-אימון. לא scalable לשפות
  נדירות, לא deterministic, לא auditable.
- **ZETS relearn**: meta-rules כ-atoms בגרף. הם **נקראים** בזמן ingestion של שפה
  חדשה, מפיקים atoms ו-edges, והכל נקי.

ההבדל הקריטי: ה-meta-rules עצמם **כבר חלק מהגרף**. אם הגרף זמין, המערכת יודעת
ללמוד. אם ה-rule לא עובד בשפה X — ZETS יכולה להוסיף rule מחודש (Hypothesis →
Learned אחרי אימות).

---

## מה אנחנו יודעים כבר (מתוך ZETS)

יש לנו:
- `morphology.rs` — per-language morphology rules, לא מוסבר כחלק מהגרף
- `ingestion.rs` — hard-coded pipeline
- `pack.rs` — per-language pack structure
- `bootstrap.rs` — seed atoms per language (manual)
- 16 language packs קיימים (`zets.core + zets.<lang>`)

**מה חסר** (וזו התובנה של עידן):
- ה-rules עצמם לא אטומים. `morphology.rs` = קוד Rust, לא nodes.
- ingestion-pipeline לא homoiconic — אי אפשר לשאול את ZETS "איך למדת?".
- אין דרך ל-ZETS **לשנות את ה-rules** שלה בסמך observation.

---

## העיצוב — meta-rules as atoms

### Atom kinds חדשים

```rust
pub enum AtomKind {
    // ... existing: Concept, Text, Template, Feature, ...
    MetaRule = 100,          // "When X, do Y" — procedural knowledge
    LearningObjective = 101, // "Learn how to detect noun plurals"
    Grammar = 102,           // Root for a language's grammar atoms
    Lexicon = 103,           // Root for a language's vocabulary atoms
}
```

### Meta-rule atoms (seed)

```
meta_rule_001 = "A language pack starts with ~100 most-frequent nouns"
meta_rule_002 = "Morphology clusters share a stem; detect via suffix alignment"
meta_rule_003 = "Cross-lang: concrete-object atoms (tree, water, hand) are universal"
meta_rule_004 = "Verb conjugation pattern = same suffix appearing on 100+ verbs"
meta_rule_005 = "Plural marker = most common suffix on known nouns in plural context"
meta_rule_006 = "Formality register detectable via surrounding lexicon"
meta_rule_007 = "When X is_a Y, X inherits Y's has_attribute edges (with Hypothesis tag)"
meta_rule_008 = "Unknown token preceded by article → likely noun"
meta_rule_009 = "Unknown token followed by object → likely verb"
meta_rule_010 = "Punctuation ordering differs per language (RTL vs LTR)"
```

Edges from each meta-rule:
- `is_a language_learning_rule`
- `has_attribute confidence` (starts high for seeded, earns it for Learned)
- `applies_to language_family:indo_european` (or universal)

### The re-learn pipeline

```
Input: corpus of language X (e.g., 10MB raw text of Swahili)
       + meta_rules (in the graph)

Process:
  1. Tokenize text via ICU (script detection auto)
  2. Build frequency table
  3. Consult meta_rule_001 → extract top-100 nouns (by co-occurrence pattern)
  4. Consult meta_rule_002 → cluster morphological variants
  5. Consult meta_rule_005 → identify plural marker
  6. Consult meta_rule_003 → try to link concrete nouns to existing cross-lang atoms
  7. Emit: zets.<lang> pack + hash_registry entries for each detected linking

Output:
  - New language pack
  - Updated hash_registry (more hashes shared across langs)
  - Evaluation: how many tokens understood after ingestion?
```

### Self-modifying loop (the learning-to-learn part)

```
If for language X:
   - A meta-rule fails (e.g., meta_rule_005 doesn't find a plural marker)
   - But the pack is still usable (e.g., language doesn't mark plural like English)

Then:
   - Tag meta_rule_005 with `Hypothesis(not_applicable_to_lang_X)`
   - Propose new meta-rule via pattern discovery (e.g., "This language marks
     plural via reduplication, not suffixing")
   - Add meta_rule_005b with `Hypothesis` provenance
   - After 3 more languages show the same pattern → promote to `Learned`
```

זה ה"ZETS is learning to learn" — איטרטיבי, ברור, reversible.

---

## מדידת הצלחה — language coverage benchmark

New test suite: `tests/language_coverage_bench.rs`

For each language:
- Load N MB of Wikipedia text
- Run re-learn pipeline
- Measure: what fraction of tokens in an unseen test corpus are:
  - (a) Recognized as known (in pack)
  - (b) Linked to cross-lang anchor (via hash_registry)
  - (c) Emitted as Hypothesis (for later verification)
  - (d) Unknown

Target per language:
- After 1 MB corpus: ≥ 50% coverage
- After 10 MB corpus: ≥ 80% coverage
- After 100 MB corpus: ≥ 95% coverage

**אם ZETS עובר את המדד על שפות 17, 18, 19 (שמעולם לא למדה), זה הוכחה חזקה שה-rules
עובדים.**

---

## סדר מימוש (הצעה)

### Phase 14.1 — Meta-rule atoms as first-class citizens

- [ ] הוסף `AtomKind::MetaRule` ל-atoms.rs
- [ ] הוסף 10 seed meta-rules ב-bootstrap.rs (המופיעים למעלה)
- [ ] שאילתה בסיסית: "איזה rules יש ל-noun detection?" → walks over `is_a
      language_learning_rule` ו-`applies_to:noun`

### Phase 14.2 — Re-learn English

- [ ] קח `zets.en` קיים + Wikipedia EN
- [ ] הרץ re-learn pipeline עם meta-rules
- [ ] השווה לתוצאה המקורית — אם coverage < 95% משתמש ב-pack הקיים, meta-rules
      חסרים. סמן את הפערים כ-Hypothesis.

### Phase 14.3 — Re-learn Hebrew

- [ ] `zets.he` + Hebrew Wikipedia
- [ ] מצפה לפחות כיסוי כי עברית מורפולוגית
- [ ] גילוי rules ייחודיים לעברית (prefix ו'/ב'/ל'/מ' → particles)
- [ ] cross-validate with English rules

### Phase 14.4 — Learn a NEW language (never seen)

- [ ] בחר שפה ללא pack קיים (נניח Swahili, Zulu, Finnish, Turkish)
- [ ] הרץ pipeline על Wikipedia dump שלה
- [ ] מדוד coverage
- [ ] אם < 80% — analyze מה meta-rules חסרים, הוסף, חזור

### Phase 14.5 — Overnight mode

- [ ] Cron-style job: every night, pick 1 untrained language from a queue
- [ ] Run re-learn
- [ ] Report to Idan via MCP next morning
- [ ] Over time: ZETS accumulates hundreds of languages

---

## הקשר לתובנות האחרות

| רכיב | איך זה משחק עם re-learn |
|------|-------------------------|
| **hash_registry** (שבניתי היום) | Cross-lang anchor — כש-re-learn של Swahili מוצא "maji" (מים), hash_registry אומר "matches English 'water' concept" → inherit the concept's edges |
| **inference_walk** (שבניתי) | Meta-rule #7 ("is_a inherits has_attribute") — כבר מומש! זו הדוגמה הראשונה של meta-rule-in-action |
| **Language meta-graph** (design doc) | ה-meta-graph נותן את ה-context — "איזה שפה דומה לאיזה?" → rules מ-family member אחד מועברים כ-starting guess לחדש |
| **16 personas** | פרסונות מגוונות שפות — Yam (child, Hebrew), Michel (senior, HE+EN-GB) — שכל אחד יכול לבדוק שה-re-learn נראה נכון לרמתו |
| **Dialect overlay** | כש-re-learn מזהה dialect variation (לא לשון שונה, גוון שלה), הוא יוצר overlay, לא pack חדש |
| **Formality register** | Meta-rule #6 — detect formality via lexicon. חלק מה-re-learn pipeline. |
| **Programming taxonomy** (שבניתי היום) | Proof שסוג אחר של taxonomy (שפות תכנות) הוא גם graph of categories. אותה טכניקה לשפות טבעיות. |

---

## למה זה פותר את D1 (merge AtomStore + PieceGraph) ללא merge

כל re-learn מייצר:
- AtomStore atoms חדשים (factual content)
- PieceGraph pack entries (linguistic structure)
- HashRegistry cross-refs

**זה לא merge. זה bridge דינמי.** A ו-B שניהם מושקעים על-ידי re-learn, אבל הם
ממשיכים לשמש תפקידים שונים. כל re-learn מחזק את ה-bridge.

אחרי 20 שפות של re-learn, yes hash_registry תכיל ~80% של cross-lang anchors.
הגישה היא הגשר ה-bigger-than-sum-of-parts.

---

## Summary

- **Idea**: system rules are atoms → system can learn-to-learn.
- **10 seed meta-rules** cover language learning at high level.
- **Re-learn pipeline** takes corpus + meta-rules → language pack + hash_registry entries.
- **Success metric**: coverage vs corpus size, measured per language.
- **Self-improves** via Hypothesis tagging of rules that don't fit.
- **Scales to hundreds of languages** because each new one reuses the rules.

**לא ממושף עכשיו** — זה design doc. Phase 14 כשנגיע. אבל זה מאפיין הכי-חזק של ZETS
ארכיטקטונית, והתובנה הזו היא שלך.
