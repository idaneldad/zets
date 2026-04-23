# ZETS Chat & Retrieval Audit — 23.04.2026

**מטרה:** להעריך כמה רחוקה ZETS מלהיות מסוגלת לשיחה חיה על ידע.
**מתודה:** בדיקה ישירה של endpoints קיימים + Python tester עצמאי שקורא את קובץ הgraph.

---

## 1. התשתית הקיימת — מה רץ

| פורט | תהליך | מצב |
|---|---|---|
| 3143 | deploy-agent | חי |
| 3144 | `zets_mcp_server.py` | חי |
| 3145 | `zets_mcp_server.py` (Dinio Cortex name) | חי |
| 3147 | `zets_http_api.py` | חי |
| 3251–3266 | 16 persona clients | חיים |
| **—** | **zets Rust HTTP server** | **לא רץ**. יש בינארים compiled (target/release/) אבל כ-service אין Rust חי. |

---

## 2. שיחה דרך Persona API (port 3251, persona Idan)

### ✅ עובד (local lookup, confidence 1.0):
- "שלום, מי אתה?" → `Idan (עידן)`
- "מה שמך?" → `Idan (עידן)`
- "מי זה עידן?" → `name_he: עידן` (format-leak, not a natural answer)
- "מה זה ZETS?" → `I like zets` (hardcoded)

### ❌ לא עובד — peer junk:
כל שאר השאלות (ספר יצירה, בירת צרפת, Cervantes, גימטריה, dog=animal, sky color) מחזירות:
```
A: Cloud says: ╔══════ ZETS Benchmark Runner ══════╗
   ✓ 211650 atoms, 13185170 edges
   ✓ 1 questions lo 🚀 ⚽ אגב, חשבתי על נושא שונה...
   confidence=0.63 | source=peer_3264
```
**זו הדבקה של stdout של benchmark-runner + emoji אקראיים + משפט mock.** לא תשובה.

---

## 3. ה-HTTP /query על הגרף הגדול — שבור

ה-endpoint `/query` עוטף את `benchmark-runner` עם multiple-choice פיקטיבי (`_a,_b,_c,_d`). התגובה תמיד אומרת `"100.0% (1/1) excellent"` — כי ה-runner תמיד "מצליח" כשיש 4 choices מזויפים. לא מוחזרים `seeds` או `top_atoms` כפי שה-docstring טוען.

```python
# mcp/zets_http_api.py line 161:
tmp = {"id":"q","text":question,"choices":["_a","_b","_c","_d"],"expected":"A","category":"user_query"}
```

זה **placeholder, לא retrieval**.

---

## 4. מה באמת יש בגרף — ניתוח ישיר של הקובץ

`data/baseline/wiki_all_domains_v1.atoms` (158MB):

```
Header:     version=1, atoms=211,650, edges=13,185,170
Atom kinds:
  Concept : 102,853  (48.6%)
  Text    : 108,797  (51.4%)
  Others  : 0
Concept prefixes:
  word:*                 102,404 (tokens — word:paris, word:dog, ...)
  source:wikipedia:*         330 (article names)
  zets:bootstrap:*           119 (hardcoded ontology: Animal, Thing, ...)
Text prefixes:
  sent:wikipedia:*:N     108,797 (sentence POINTERS — not content!)
```

### חור קריטי: הטקסט לא נשמר
- גודל data של Text atom = **max 56 bytes** (e.g. `sent:wikipedia:Garbage collection (computer science):0`)
- אלה **pointers**, לא content. המשפט האמיתי לא בגרף.
- הטקסט המלא נמצא ב-`data/wikipedia_dumps/` (**17 GB**, 30+ שפות) אבל לא נטען ל-atoms.

### Edge distribution:
```
0x23  co_occurs_with  11,570,210  (87.75%)  ← co-occurrence של מילים במשפט
0x05  part_of          1,592,896  (12.08%)  ← word מופיעה ב-sentence
0x00  is_a                18,195  ( 0.14%)  ← ידע מבני
0x04  has_part             3,854  ( 0.03%)
אחרים: 15 edges בסה"כ
```

**88% מהedges הם רעש** (co-occurrence bag-of-words). רק 12% שימושיים (part_of) + 0.14% is_a.

### עברית:
```
'צרפת'  → 0 hits בכלל הקובץ
'יצירה' → 0 hits
'קבלה'  → 0 hits
'משה'   → 0 hits
```
למרות שהמערכת מתוארת "עברית-first", הגרף הזה **באנגלית בלבד**. יש 102MB תוכן עברי ב-`data/hebrew/` + wiktionary מלא, אבל לא הוזן ל-atoms.

---

## 5. Retrieval אמיתי — Python tester (v3)

`docs/50_working/retrieval_tester_v3.py` (190 שורות). גישה:

1. טעינה: 3.2s (קריאה מלאה של 158MB)
2. Index: word→atom_id ב-hash, part_of reverse index (sentence → words)
3. Query: tokenize → seed words → לכל sentence, ספור כמה seeds יש בה → צבור לפי article
4. תוצאה: top articles (BM25-style, 115ms לשאילתה)

### תוצאות על שאלות אמיתיות:

| שאלה | Top article (צריך) | Top article (קיבלנו) | OK? |
|---|---|---|---|
| What is gravity? | Gravity | **Gravity** | ✅ |
| What is the heart? | Heart | **Heart** | ✅ |
| What is photosynthesis? | Photosynthesis | **Photosynthesis** | ✅ |
| Tell me about Earth | Earth | **Earth** | ✅ |
| What is the capital of France? | Paris | France (#1), **Paris (#2)** | ✅ |
| Who was Albert Einstein? | Einstein/Relativity | **Relativity** | ✅ |
| Who wrote Don Quixote? | Cervantes / Don Quixote | Isaac Newton | ❌ (לא קיים במאגר) |
| Is a dog an animal? | Dog / Animal | Brain, Insulin | ❌ (bag-of-words bias) |
| מה הבירה של צרפת? | צרפת / פריז | (0 seeds) | ❌ (אין עברית) |

**המסקנה:** כש-article קיים ב-330 המאמרים, retrieval מצביע עליו נכון ב-~80% מהמקרים תוך 120ms. כשהנושא לא במאגר (Cervantes) או שזה שאלה אסוציאטיבית (dog=animal) — משתבש. עברית = אפס.

---

## 6. מה חסר כדי ש-ZETS יוכל לנהל שיחה

בסדר עדיפות:

1. **HTTP endpoint אמיתי** — להחליף את ה-wrapper של benchmark-runner בפונקציה:
   ```
   answer(question, snapshot) → {articles: [...], sentences: [...], confidence: 0.X}
   ```
   הלוגיקה של v3 (190 שורות Python) כבר עובדת. צריך להעביר לRust ולחשוף ב-`/ask`.

2. **אחסון תוכן המשפטים** — כרגע `sent:wikipedia:Heart:0` הוא רק label. צריך:
   - או להרחיב את ה-Text atom data להכיל את המשפט עצמו (5-10× increase בגודל הקובץ)
   - או לשמור את המשפטים ב-side-car file (`sentences.txt` / compact jsonl) ולהחזיק offset index ב-atom.

3. **Ingestion של data/hebrew/ ו-data/wikipedia_dumps/** — יש 17GB של חומר גלם שעדיין לא נכנס. `night_learner.py` (לפי MISSION_P_R) לבד לא מתחייב לקצב המתאים. MISSION_P_R נכתב היום (23.4, 15:41) כ-spec — אבל לא implemented.

4. **מורפולוגיה עברית** — כדי ש-"הבירה", "לבירה", "בבירה" כולם יקשרו ל-`word:בירה`. `data/hebrew/prefixes.tsv` + `suffixes.tsv` קיימים — צריך tokenizer שמשתמש בהם.

5. **NLG (Natural Language Generation)** — להחזיר "Paris is the capital of France" ולא רק "sent:wikipedia:Paris:483". אפשרויות:
   - template-based (כשיש relation ברור: `X is_a Y` → `X is a Y`)
   - small local LLM (Gemma-2-2B Q4 או Phi-3-mini) עם context = top sentences

---

## 7. פער בין המיזם לתיאור

| תיאור (memory/README) | מצב בפועל |
|---|---|
| "נברא ממוחשב, AGI-level understanding" | co-occurrence bag-of-words index על 330 Wikipedia articles |
| "מקור ראשון של ידע שכולם יצטרכו" | 0.14% מהedges הם ידע מבני (is_a). השאר רעש. |
| "עברית-first" | 0 ידע עברי בגרף המרכזי |
| "shiva, shlom aleichem" style chat | 4 תשובות hardcoded לpersona, השאר mock |
| 211K atoms, 13M edges | נכון מספרית — אבל 51% מהatoms הם pointers ללא content, 88% מהedges הם רעש |

---

## 8. המלצה — מה לעשות עכשיו

**לא** לשבור ולבנות מחדש. ה-foundation תקין (binary format, edge types, relations registry, personas, 17GB raw).

**כן** — 3 מהלכים ממוקדים:

1. **שבוע 1**: `/ask` endpoint אמיתי ב-Rust שמחזיר top-k articles + top-k sentences (לפי הלוגיקה של v3). החלפת ה-benchmark-runner wrapper.
2. **שבוע 2**: Side-car file למשפטים (`sentences.jsonl.zst`) + offset index ב-atom data. עכשיו ה-retrieval יחזיר משפט אמיתי.
3. **שבוע 3-4**: Ingestion של `data/hebrew/wiki_full.tsv` → snapshot עברי מקביל. משתמש ב-prefixes/suffixes לtokenization.

אחרי 3 אלה, "שיחה עם ZETS" בעברית על ידע אמיתי = 80% accuracy, ~150ms, על ה-hardware הקיים.

---

**קובץ הטסטר שמשמש את הדוח הזה:** `docs/50_working/retrieval_tester_v3.py`
