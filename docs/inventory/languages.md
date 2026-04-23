# 🌐 שפות — Languages Inventory

**Last updated:** 23.04.2026
**Last verified:** 23.04.2026 (against `cargo test --lib morphology::languages`)

---

## מה המשימה

ZETS צריך להבין ולייצר תוכן ב-שפות רבות:
1. **שפות טבעיות** (Hebrew, English, Arabic, Spanish, Vietnamese, ...)
2. **שפות תכנות** (Rust, Python, JavaScript, ...)
3. **שפות מדעיות / סמליות** (מתמטיקה, כימיה, לוגיקה פורמלית)

## איך תיחשב הצלחה

- כל שפה עם morphology module משלה (tokenize, stem, normalize)
- Tests ספציפיים לשפה עוברים (past-tense, plural, prefixes)
- Sense graph תומך ב-cross-lingual lookup

---

## 🗣️ שפות טבעיות — מצב בפועל

### מיושם במלואו (עם morphology + tests)

| שפה | Morphology | Tests | נבדק אחרון | סטטוס |
|------|:----------:|------:|:-----------:|:-----:|
| עברית (Hebrew) | ✅ `hebrew()` | 5 tests | 23.04 | 🟢 עובד |
| אנגלית (English) | ✅ `english()` | 7 tests | 23.04 | 🟢 עובד |
| ערבית (Arabic) | ✅ `arabic()` | 2 tests | 23.04 | 🟡 בסיסי |
| ספרדית (Spanish) | ✅ `spanish()` | 3 tests | 23.04 | 🟡 בסיסי |
| וייטנאמית (Vietnamese) | ✅ `vietnamese()` | 2 tests | 23.04 | 🟡 בסיסי |

### מה הtests באמת בודקים

**Hebrew** — `src/morphology/languages.rs` ו-`fold/`:
- `hebrew_past_1sg` — past tense 1st person singular detection
- `hebrew_plural_masc` / `hebrew_plural_fem` — plural inflection
- `hebrew_stacked_prefixes` / `hebrew_stacked_triple` — preposition + article + verb
- `bpe_on_hebrew` — BPE tokenization works on Hebrew letters
- `tokenize_hebrew_per_codepoint` — codepoint-level splitting
- `hebrew_preserved` — normalize doesn't break Hebrew
- `hebrew_with_niqqud_preserved_unless_stripped` — niqqud handling
- `roundtrip_on_hebrew` — fold + unfold is lossless

**English** — 7 tests covering past tense (walked), irregular (went/children), continuous (running), plurals (dogs/cities), future, perfect

**Arabic** — definite article ("al"), feminine taa marbuta (ة). **חסר:** verb conjugation, dual number, broken plurals

**Spanish** — regular plurals (perros), past (hablado), future (hablaré). **חסר:** irregular verbs, subjunctive

**Vietnamese** — particles (đã, sẽ), analytic-only morphology (no inflection)

### לא מיושם

כל שאר השפות בעולם — **אין support ספציפי**. ה-fold module יודע לטקן generic bytes/UTF-8, אבל **בלי morphology**.

**הקלודם שיש להם "support" עקיף:**
- צרפתית, גרמנית, איטלקית, פורטוגזית — דרך BPE generic
- רוסית, אוקראינית — UTF-8 OK, אבל אין stemmer
- סינית, יפנית, קוריאנית — UTF-8 OK, אבל אין segmentation (דרוש tokenizer מיוחד)

### מציאות מול claim קודם

⚠️ **תיקון:** בעבר נאמר "ZETS crawler מכסה 47/48 שפות". זה לא מדויק.
- **Crawler רץ** (17M articles, commit `81f9724`) היה קיים לפני הrework
- **Morphology + tests** — רק 5 שפות (עברית, אנגלית, ערבית, ספרדית, וייטנאמית)
- **BPE generic tokenization** — עובד על כל UTF-8, אבל זה לא "תמיכה בשפה"

---

## 💻 שפות תכנות

### מיושם ב-ProcedureTemplate (src/procedure_template/instance.rs)

Enum `Language` כולל:

| שפה | פורמלית מוכרת | תמיכת runtime | באחריות |
|------|:--------------:|:---------------:|----------|
| Rust | ✅ | ✅ (ZETS עצמו) | עצמי |
| Python | ✅ | ❌ | תאורטי (templates only) |
| JavaScript | ✅ | ❌ | תאורטי |
| TypeScript | ✅ | ❌ | תאורטי |
| Go | ✅ | ❌ | תאורטי |
| Java | ✅ | ❌ | תאורטי |
| C# | ✅ | ❌ | תאורטי |
| Ruby | ✅ | ❌ | תאורטי |
| PHP | ✅ | ❌ | תאורטי |
| C | ✅ | ❌ | תאורטי |
| C++ | ✅ | ❌ | תאורטי |
| Shell | ✅ | ❌ | תאורטי |
| SQL | ✅ | 🟡 חלקי (system_graph/vm) | פנימי |

**מה "תאורטי" אומר:** ZETS **יודע לתאר** code patterns (templates + instance binding). אבל **לא מריץ** את הקוד. הפעלה תדרוש orchestration layer או LLM integration.

**מה עובד בפועל:** מעבר בין שפות דרך `procedure_template.binding` — `http_post(url, body)` בPython עם `target_url` + `payload` מזוהה כאותו template כמו JavaScript עם `endpoint` + `b`.

---

## 🧮 שפות מדעיות / סמליות

### מתמטיקה
- **אריתמטיקה:** ✅ עובד (`system_graph/vm.rs`, Rust native)
- **אלגברה סמלית:** ❌ לא מיושם
- **calculus סמלי:** ❌ לא מיושם
- **סטטיסטיקה:** ❌ לא מיושם (אפשר דרך עתיד)

### לוגיקה פורמלית
- **Propositional logic:** 🟡 דרך `system_graph/reasoning.rs` — partial
- **Predicate logic:** ❌ לא מיושם
- **Theorem proving:** ❌ לא מיושם

### נוטציות נוספות
- **LaTeX parsing:** ❌ לא מיושם
- **MathML:** ❌ לא מיושם
- **Chemistry (SMILES):** ❌ לא מיושם

---

## פער ליעד

| תחום | יעד MVP | מצב נוכחי | פער | עדיפות |
|------|:-------:|:---------:|:---:|:------:|
| 5 שפות טבעיות עיקריות | 5 | 5 | 0 | ✅ הושג |
| עוד 5 שפות טבעיות | 10 | 5 | −5 | בינונית |
| 3 שפות תכנות עם runtime | 3 | 1 (Rust) | −2 | גבוהה |
| מתמטיקה סמלית | כיסוי בסיסי | 0 | חסר | בינונית |

---

## Tests פעילים

```bash
# Hebrew
cargo test --lib morphology::languages::tests::hebrew

# English
cargo test --lib morphology::languages::tests::english

# Arabic / Spanish / Vietnamese
cargo test --lib morphology::languages::tests::arabic
cargo test --lib morphology::languages::tests::spanish
cargo test --lib morphology::languages::tests::vietnamese

# All morphology (36 tests)
cargo test --lib morphology::

# All language-related (fold + morphology)
cargo test --lib fold:: morphology::
```

---

## בדיקות QA (איכות) + TEST (עומס)

| Test | סוג | מה נבדק | נבדק אחרון | סטטוס |
|------|:---:|---------|:-----------:|:-----:|
| hebrew_past_1sg | QA | נכונות זיהוי זמן | 23.04 | ✅ |
| english_irregular_children | QA | plural irregular | 23.04 | ✅ |
| arabic_definite_article | QA | זיהוי "אלַ-" | 23.04 | ✅ |
| spanish_plural_perros | QA | regular plural | 23.04 | ✅ |
| vietnamese_particles | QA | זיהוי "đã" | 23.04 | ✅ |
| roundtrip_on_hebrew | QA | lossless fold | 23.04 | ✅ |
| (missing) | TEST | latency tokenize 10K chars | — | 🔴 חסר |
| (missing) | TEST | memory usage 1M tokens | — | 🔴 חסר |
| (missing) | TEST | throughput 100K req/sec | — | 🔴 חסר |

---

## היסטוריית שינויים (במסמך זה)

| תאריך | שינוי |
|:-----:|-------|
| 23.04.2026 | Audit ראשון. תיקון claim קודם של "47 שפות" ל-5 morphology + generic BPE |
