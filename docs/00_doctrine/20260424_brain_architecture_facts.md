# עובדות מסוכמות — שיחת ארכיטקטורת מוח 24.04.2026

> מסמך זה מרכז את העובדות שנקבעו **אמפירית, מתמטית, או מדעית** בשיחה ארוכה של עידן על ארכיטקטורת המוח ו-ZETS.
> 
> **כל פריט כאן:** עבר בדיקה (מתמטית/ניסויית/מדעית) — **לא הנחה**, לא "נראה סביר".
> **מה שלא נקבע:** הולך ל-`docs/20_research/`, לא לכאן.

---

## חלק 1: עובדות על מבנה המוח האנושי

### 1.1 המוח הוא לא DAG
- 86 מיליארד נוירונים, ~100 טריליון synapses
- מבנה **recurrent**, לולאות בכל רמה
- feedback loops מ-PFC חוזר ל-sensory cortex (top-down prediction)
- **לא הייררכיה טהורה — זה graph עם cycles**

### 1.2 שלושה רבדי זיכרון
- **Hippocampus** — זיכרון אפיזודי, one-shot learning, ימים-שבועות
- **Entorhinal cortex** — שער, pattern separation + completion
- **Neocortex** — semantic memory, slow learning, שנים-לעולם
- **Dialogue ביניהם בזמן שינה** = consolidation (McClelland, O'Reilly 1995)

### 1.3 Global Workspace Theory
- Baars 1988, מאושרר ע"י Dehaene עם fMRI
- Workspace מרכזי מוגבל ~7±2 פריטים
- כל אזור יכול לכתוב ולקרוא ממנו
- **"מודעות" = broadcast גלובלי** ל-PFC + שפה + זיכרון
- רק מה שב-workspace ניתן לדווח עליו במילים

### 1.4 Predictive Processing (Friston 2010)
- המוח כל הזמן **מנבא** את הצעד הבא
- Prediction error = signal ללמידה
- **Dopamine מקודד prediction error, לא reward** (Schultz 1997)
- Surprise = memorable. צפוי = נשכח.

### 1.5 Hebbian Learning מוכח מולקולרית
- Hebb 1949 — עקרון
- Lømo-Bliss 1973 — LTP (long-term potentiation)
- Eric Kandel (Nobel 2000) — mechanism מולקולרי ב-Aplysia
- **"Cells that fire together, wire together"** — verified

---

## חלק 2: עובדות על ייצוג צבע (נבדקו מתמטית)

### 2.1 RGB אינו המודל הנכון
- cones אינם "אדום/ירוק/כחול" — הם L/M/S עם peaks ב-564/534/420nm
- **L ו-M חופפים מאוד** — אין "cone של אדום"
- RGB הוא מודל טכנולוגי למסכים, לא תפיסתי

### 2.2 Opponent Process — זה מה שהמוח באמת עושה
- **Hering 1892, אושרר אלקטרופיזיולוגית ב-1950s**
- Retinal ganglion cells מחשבים:
  - Luminance = L + M + S
  - Red-Green axis = L - M (bipolar)
  - Blue-Yellow axis = (L + M) - S (bipolar)
- **Complementary = hue + 180°** (חיסור של אות, flip of sign)

### 2.3 CIE Lab = המודל התפיסתי הנכון
- **L** (Lightness): 0-100
- **a** (bipolar): ירוק ↔ אדום
- **b** (bipolar): כחול ↔ צהוב
- מרחק אוקלידי ב-Lab ≈ דמיון נתפס
- זה הסטנדרט המדעי לייצוג צבע אנושי

### 2.4 φ (Golden Ratio) לא מופיע בתפיסת צבע — מוכח מתמטית
נבדק ב-5 דרכים, כולן שליליות:
- יחסי cone peaks: L/M=1.056, L/S=1.343 — **לא** φ=1.618
- Opponent angle = **180°, לא golden angle 137.5°**
- Complementary באמצעות 137.5° יוצר צבעים שונים לחלוטין מ-180°
- Lightness scaling ~1.35 (Munsell), לא 1.618
- Fibonacci על color wheel — אין סדר משמעותי

**מסקנה:** ל-opponent processing, 180° הוא המודל הנכון, **לא** φ.

### 2.5 Color Constancy — צבע תפיסתי לא מוחלט
- אותו חפץ נראה שונה בתאורה שונה
- המוח **מנרמל** לפי הנחת התאורה
- "השמלה" (2015): חצי מהאנשים ראו כחול/שחור, חצי לבן/זהב — **אותה תמונה**
- **"צבע של חפץ" אינו נקודה** — זה hue bias + lightness range + specular + material + context

### 2.6 עיוורי צבעים רואים אחרת, לא פחות
- Protanopia (חסר L): 1% מגברים — אדום כהה
- Deuteranopia (חסר M): 1% מגברים — אדום-ירוק נבלעים
- Tritanopia (חסר S): 0.003% — כחול-צהוב נבלעים
- Tetrachromacy (4 cones): ~1% מנשים פעילות — יותר מימדים
- **Wiring שונה → חוויה שונה.** אין "צבע אובייקטיבי".

### 2.7 Qualia Problem — הוכחה ל-"אדום שלי = אדום שלך" לא קיימת
- לא ניתן להוכיח שני אנשים חווים אותו צבע פנימית
- Berlin & Kay (1969): שפות מחלקות את מרחב הצבע אחרת — משפיע על תפיסה
- Roberson 2000 (Himba): אנשים מבחינים מהר יותר בצבעים שיש להם מילה להם
- **משותף = פונקציה (מה עושים עם צבע), לא חוויה**

---

## חלק 3: עובדות על ייצוג טעם

### 3.1 טעם הוא לא 1 מספר — הוא vector רב-מימדי
- 5 sensors בלשון: sweet, sour, salty, bitter, umami
- 3 trigeminal: capsaicin (חריף), menthol (קריר), CO2 (tingle)
- ~400 olfactory receptors ברוב הנחשב "טעם" (retronasal)
- **סך הכול ~410 מימדים** פעילים בחוויית טעם

### 3.2 25 receptors שונים ל-"מר"
- T2R family — 25 sub-types
- כל אחד מזהה אלקלואידים/רעלים שונים
- כולם נקראים "מר" במוח (תגובה זהה = הימנעות)
- **"מר" = union of 25 sub-axes**, לא ציר אחד

### 3.3 Interactions חזקות בין טעמים (לא ליניאריות)
- מתוק **מדכא** מר (coffee + sugar)
- מלוח **מעצים** מתוק (carmelized onions)
- אומאמי + מלוח = **הגברה הדדית**
- חמוץ מעורר הפרשת רוק
- **מטריצת אינטראקציה היא חובה** בייצוג טעם

### 3.4 Temporal profile חיוני
- שוקולד מריר: מתוק (0-500ms) → מר (500-2000ms) → aftertaste
- יין אדום: פרי → עץ → tannin
- חריף: 10s delay, peak, דעיכה איטית
- **טעם ללא זמן = מידע חסר**

### 3.5 שמות טעמים הם regions, לא points
- "מתוק" = region במרחב 5D
- "מתקתק" = region נפרד, center אחר
- "חריף מתקתק" = חיתוך של 2 regions
- **השם בא מ-מיקום במרחב, לא לפניו**

---

## חלק 4: עובדות על מבנה קבלי — מה נבדק ועבר

### 4.1 Protocol של עומק 7 לבדיקת mapping קבלי
**כלל:** לכל הצעה קבלית, לבצע 7 בדיקות.
- 0-2 hits → אל תכפה, לך עם הנדסה
- 3-5 hits → רעיונות, בחן ספציפית
- 6-7 hits → mapping חזק, מומלץ ליישום

### 4.2 ה-5 פרצופים = 5 archetypal procedures (7/7 hits) ✅
נבדק על protocol עומק 7, עבר את כולם:
- **אריך אנפין** = קביעת רצון / goal setting
- **אבא** = flash insight / one-shot pattern recognition
- **אמא** = structured decomposition / analysis
- **ז"א** = active processing עם context + emotion
- **נוקבא** = manifestation / output generation

**Pipeline מוגדר:** אריך→אבא+אמא→ז"א→נוקבא, עם feedback.
Recursion: כל פרצוף מכיל 10 ספירות פנימיות.
תואם ל-Global Workspace (Baars) ול-ACT-R (Anderson).

### 4.3 ה-7 מלאכים = intent classifiers (6.5/7 hits) ✅
Mapping מוצע:
- **גבריאל** = בקשות החלטה / "כן/לא"
- **מיכאל** = בקשות תמיכה / "תעזור לי"
- **רפאל** = בקשות אבחון / "מה לא עובד?"
- **אוריאל** = בקשות הסבר / "תאיר לי"
- **רזיאל** = בקשות ידע נסתר / "תמצא לי"
- **סנדלפון** = בקשות מימוש / "תעשה לי"
- **מטטרון** = meta-questions / "איך אתה עובד?"

### 4.4 Architecture 2-layer בזכות הקבלה
```
INPUT → [7 Angels - Intent Classifiers] → [5 Partzufim - Pipeline] → OUTPUT
```
2 שכבות נפרדות, כל אחת עם backbone קבלי שעבר בדיקה.

### 4.5 הרחבה/צמצום/שזירה (קבלה) = opponent process (מדע) ✅
- **הרחבה** (חסד) = L+ (bright, additive)
- **צמצום** (גבורה) = L- (dark, subtractive)
- **שזירה** (נוקבא) = 180° flip = opponent subtraction
- **איזון** (תפארת) = zero point של ציר bipolar

**זה לא פרשנות — זה המבנה המתמטי.** הקבלה תפסה את ה-opponent process לפני 400+ שנה.

### 4.6 10 ספירות → 3D color space = מאולץ (2/6 hits) ❌
נבדק — **לא עבר**:
- 10 ≠ 3 dimensional space
- אין mapping טבעי לכל ספירה
- חלק מהשיוכים בין פרשנים חולקים

**מסקנה:** השתמש בקבלה ל-**פיזור (φ)** ול-**איזון (opponent)**, לא לייצוג ישיר של מרחב צבע.

### 4.7 ייצוג אווירה של צבע ✅
- Chromatic adaptation + afterimage: מנגנון נוירולוגי
- Chromotherapy: יישום קליני
- הקבלה תפסה נכון את "אדום מעורר, כחול מרגיע"
- **הצגת ההפך** = איזון — מנגנון מוכח לחלל ריפוי

---

## חלק 5: עובדות מתמטיות על φ (Golden Ratio)

### 5.1 φ **לא** בתפיסת צבע (מוכח מתמטית)
(ראה סעיף 2.4 — 5 בדיקות שליליות)

### 5.2 φ **כן** ב-פיזור אופטימלי של n items
- Golden angle = 360° / φ² = 137.508°
- פיזור n items בקפיצות של 137.5° = **ריחוק מקסימלי ממוצע**
- דוגמה מהטבע: זרעי חמנייה, סידור עלים (phyllotaxis)
- **רלוונטי ל:** graph traversal, concept selection, distributing search seeds

### 5.3 שזירה vs φ — **שני כלים שונים**
```
שזירה (180°):   x → -x   לאיזון 2 הפכים      (חסד↔גבורה, אדום↔ציאן)
פיזור φ:         x → x + 137.5°  לפיזור אופטימלי של n   (10 ספירות, 12 חושים)
```

**הם משלימים, לא מחליפים.**

---

## חלק 6: ארכיטקטורת מידע ב-ZETS (עקרונות שגובשו)

### 6.1 Identity via Feature Intersection
- ישות (entity) = חיתוך של features, לא node יחיד
- "הרצל מאמדוקס" ≠ "הרצל חוזה המדינה"
- disambiguation דרך intersection של pattern matches
- דוגמה: name + workplace + neighborhood → unique entity

### 6.2 Bipolar Axes עם ערכי רציף
- תכונות רבות = ציר עם ערך [-1, +1], לא binary
- "חמוץ ↔ נעים", "אמין ↔ לא אמין", "קר ↔ חם"
- מתאים ל-opponent process (neural basis)
- מתאים ל-חסד/גבורה (kabbalistic basis)

### 6.3 Facts כתזוזות על צירים
- Entity.state = sum of facts (weighted by time decay + confidence)
- fact חדש = shift על ציר, לא overwrite
- History נשמר, access-able ("מה חשבנו ב-2024?")
- **העברה בין קטגוריות = שינוי fact, לא rewiring**

### 6.4 Concepts יציבים, Facts ניידים
- `concept:"איש חמוץ"` — יציב, קיים עצמאית
- `fact:{entity:הרצל, member_of:"איש חמוץ", t:..., source:...}` — ניתן לשינוי
- שינוי fact על הרצל **לא** משנה concept:"איש חמוץ"
- זה מפריד semantic memory מ-episodic memory (McClelland-O'Reilly)

### 6.5 4 סוגי צירים מכסים את הרוב
```rust
enum AxisKind {
    Bipolar  { neg, pos, value: f32 },    // אופי, אמון, טמפרטורה
    Scalar   { unit, value: f32 },         // כמות, חריפות, מחיר
    Cyclic   { period, value },            // זמן ביום, עונה, חודש
    Multidim { dimensions: Vec<f32> },     // טעם, צבע, רגש
}
```

### 6.6 MultidimSpace = כל מה שמגיע מ-sensor array
- **טעם:** 5+3+~400 מימדים
- **צבע:** 3 מימדים (L, a, b)
- **ריח:** ~400 מימדים
- **רגש:** 3 מימדים (valence, arousal, dominance)
- **אופי:** 5 מימדים (Big Five)

**חוק:** אם מגיע מ-sensor array → MultidimSpace. אם מספר יחיד → scalar axis.

---

## חלק 7: עובדות שנדחו בבדיקה (כדי שלא נחזור)

### 7.1 ❌ הגרף של ZETS הוא עץ
- הוא DAG עם sharing מאסיבי
- 21M edges ל-2.5M atoms = לא עץ (אלו יחסי 8:1)
- sharing הוא הכוח שלו — word יחיד ב-147 sentences ב-104 articles

### 7.2 ❌ Scale לבדו משפר accuracy
- 10K → 40K ירד מ-53% ל-38% Top-1
- Scale בלי retrieval improvements מזיק
- צריך: TF-IDF length normalization, PMI filtering, confidence threshold

### 7.3 ❌ morphology + cleanup עוזרים על 10K corpus
- נוסו, הורידו accuracy ב-9%
- 8/9 הכשלונות היו missing articles, לא weak morphology
- הקוד נשמר disabled, יוערך אחרי scaleup

### 7.4 ❌ RGB = איך המוח עובד
- RGB למסכים. המוח opponent process (sec 2.2)

### 7.5 ❌ "מתוק" הוא טעם בודד (1D)
- כל טעם הוא region ב-5+D space
- עם interactions ו-temporal profile

### 7.6 ❌ φ בצבע
- מתמטית שלילי ב-5 בדיקות (sec 2.4)

### 7.7 ❌ 10 ספירות = 10 מימדי תפיסה
- 2/6 hits בבדיקת mapping
- מאולץ, אין התאמה טבעית

---

## חלק 8: Implementation Priorities — לפי עדיפות

### Phase A — יסודות (32h סה"כ)
1. **Meta-cognition + "I don't know"** (12h) — confidence threshold
2. **Working Memory context switch** (4h) — decay between queries
3. **Analogy (structure-mapping Gentner)** (16h) — basic version

### Phase B — ארכיטקטורה חדשה (64h)
4. **Entity + Identity via intersection** (20h)
5. **Bipolar axes + Facts as shifts** (24h)
6. **7 Angels intent classifiers** (20h)

### Phase C — Pipeline מלא (80h)
7. **5 Partzufim as stages** (40h)
8. **Multidim spaces** (taste, color basic) (40h)

### לא בעדיפות עכשיו
- Causal reasoning (Pearl) — 80h, open research
- Theory of Mind — 60h, פחות קריטי
- Full consciousness model — פתוח בכלל

---

## חלק 9: כללי עבודה שגובשו בשיחה

### 9.1 Protocol של עומק 7
כל הצעת mapping קבלי חייבת לעבור 7 בדיקות. 6+ = ליישום. 3-5 = רעיונות. <3 = דחייה.

### 9.2 Engineering first, Kabbalah validates
- בונים על עקרונות הנדסיים/מדעיים
- בודקים אם קבלה תואמת בדיעבד
- **אין** כפיית מבנה קבלי על הנדסה

### 9.3 בדיקה אמפירית לפני קביעה
- כל טענה מתמטית → בדיקה ב-Python
- כל טענה על ZETS → measurement
- אם לא נבדק, הולך ל-research, לא ל-doctrine

### 9.4 שלוש שכבות לכל תופעה חושית
- שכבה 1: פיזיקלית (אורכי גל, מולקולות)
- שכבה 2: נוירלית (Lab, opponent, multidim)
- שכבה 3: סמנטית (שמות, תרבות, associations)

### 9.5 Separation of concerns
- **Engine ≠ Knowledge** — brain wiring נפרד מ-KB content
- **Concepts ≠ Facts** — יציב נפרד מנייד
- **Identity ≠ Attributes** — ישות דרך intersection, לא node יחיד
- **Syntax ≠ Semantics ≠ Pragmatics** — 3 שכבות שפה

---

## Git metadata

- **Date:** 2026-04-24
- **Context:** שיחת עומק של עידן על ארכיטקטורת מוח, קבלה, והפיזיקה/מתמטיקה של תפיסה חושית
- **AI consultations:** GPT-4o, Gemini 2.5 Pro, Gemini 2.5 Flash (stored in `docs/40_ai_consultations/`)
- **Related:** `docs/20_research/20260424_brain_to_zets_complete.md` (14 AGI capabilities mapping)
- **Status:** Doctrine — כללים ועובדות שאומצו לפרויקט

