# דחייה הנדסית: אין מודל מתמטי של "שזירה + φ" במוח/בטקסט

**Date:** 24.04.2026  
**Status:** ❌ **Rejected with evidence** — לא להשתמש  
**Related:** `sim/math_tests/operator_tests.py`, `sim/math_tests/deep_probe.py`

---

## ההיפותזה שנבדקה

האם יש מודל מתמטי על המוח שמופעל דרך operators על טקסט?  
מבוצע באמצעות:
- שזירה (reverse)
- שזירה הפוכה (inverse interleave)
- אופרטורים של אותיות (at-bash, al-bam)
- Operators מבוססי-φ (phi_position, golden_split)
- פונקציות באותיות יווניות + גימטריה

**הטענה שנבדקה:** אם נריץ אותם על טקסט אמיתי, יופיעו "יחסים מיוחדים" (φ, π, אחרים) שלא מופיעים בטקסט random.

---

## מתודולוגיה

### Corpus
- **30 משפטים אמיתיים** מתנ"ך + קבלה + ברכות (מתפללים)
- **30 טקסטים רנדומליים** (אותיות עבריות random)
- **30 טקסטים מעורבבים** (אותן אותיות של real, סדר רנדומלי)

### Operators נבדקו (10)
```
identity, reverse, at_bash, al_bam,
phi_position, phi_inv_position,
interleave, inverse_interleave,
golden_split, mirror_halves
```

### Metrics
- Entropy (Shannon)
- Letter diversity
- Pair preservation (% זוגות שכנים שנשמרו)
- Gematria ratio (gematria(transformed) / gematria(original))

### Statistical Test
Welch's t-test approximation. Significance: `|t| > 2.0` → p < 0.05.

---

## תוצאות הבדיקות

### TEST A: Entropy — REAL vs RANDOM
**10/10 operators מבחינים** — אבל זה **טריוויאלי**. טקסט אמיתי בעברית משתמש בהתפלגות אותיות שונה מ-random (האותיות א, ו, י, ה, ל שכיחות יותר). כל operator מראה את זה.

**משמעות:** לא "גילוי". סתם העובדה שעברית ≠ random.

### TEST B: Pair Preservation — REAL vs SHUFFLED (הטסט הקריטי)
הבדיקה הזו היא הכי חשובה. אותן אותיות, סדר שונה.

**תוצאה: 1/9 operators מראים הבדל סטטיסטי.**

רק `inverse_interleave` מבחין (t=-3.67, p<0.01), וגם זה מראה שטקסט מסודר **פחות** preserves pairs אחרי inverse_interleave — הפוך ממה שהיינו מצפים מ"מודל קדוש".

**משמעות:** Operators לא תופסים "סדר משמעותי" בטקסט. השזירה, פאי, at-bash — כולם מתנהגים אותו דבר על טקסט מסודר וטקסט מעורבב.

### TEST C: Gematria Ratio — האם יש קרבה ל-φ?

```
operator            REAL ratio    RAND ratio
identity            1.000         1.000
reverse             1.000         1.000    (trivial — same letters)
at_bash             2.015         1.185    (artifact, ראה להלן)
al_bam              2.233         1.152    (artifact, ראה להלן)
phi_position        1.000         1.000    (trivial — same letters)
interleave          1.000         1.000    (trivial)
...
```

**הערה חשובה:** רוב ה-operators משמרים גימטריה (כי אותן אותיות = אותו סכום). רק at_bash ו-al_bam משנים גימטריה (כי מחליפים אותיות). אלה שלא משנים גימטריה — באופן trivial מחזירים 1.

### TEST D: קרבה ל-φ — האם ספציפית לטקסט אמיתי?

**תוצאה: 0/10 operators מראים קרבה מיוחדת ל-φ בטקסט אמיתי.**

אין **שום** operator שיחס הגימטריה שלו קרוב ל-φ באופן משמעותי יותר בטקסט אמיתי מאשר ב-random.

---

## בדיקות עמוקות של claims ספציפיים

### Claim 1: at_bash ratio=2 בטקסט אמיתי
**הסבר:** artifact של התפלגות אותיות.

האותיות הנפוצות בעברית (א=1, ו=6, י=10, ה=5, ל=30) כולן בעלות gematria נמוכה. at_bash שולח אותן לאותיות גבוהות (ת=400, ק=100, צ=90, פ=80, מ=40, כ=20).

ממוצע: high_values/low_values ≈ 2. זה מתמטי, לא "קדוש".

### Claim 2: גבריאל Heb/Greek = φ (מ-memory)
```
גבריאל    Heb=246, Greek=154    ratio=1.597    (1.3% from φ)
מיכאל     Heb=101, Greek=141    ratio=0.716    (closest: 1/φ=0.618, dist=0.10)
רפאל      Heb=311, Greek=128    ratio=2.430    (closest: 2, dist=0.43)
אוריאל    Heb=248, Greek=618    ratio=0.401    (closest: 0.5, dist=0.10)
```

**4/4 מלאכים: רק 1 קרוב ל-φ. אין pattern.** גבריאל היה cherry-pick.

### Null Hypothesis Test (10,000 random pairs)

אם ניקח 2 מספרים random בטווח 50-500:
- **2.3%** יפלו ב-±0.05 מ-φ
- **5.8%** יפלו ב-±0.05 מ-1/φ
- **6.3%** יפלו ב-±0.05 מ-1

**סיכוי שמתוך 5 זוגות, לפחות אחד יפול ליד φ = 11%.**

לכן, מציאת **מקרה בודד** של יחס φ (כמו גבריאל) = **לא ראיה**. זה תוצר טבעי של random sampling.

### 2,352 זוגות של מילים מהתנ"ך
```
REAL (תנ״ך):         1.53% קרובים ל-φ
RANDOM (אותה dist):  1.36% קרובים ל-φ
Expected (uniform):   2.00%
Chi-squared:          2.591 (not significant)
```

**אין over-representation של φ במילים תנ״כיות.**

---

## מסקנות הנדסיות

### מה ש**לא** קיים:

❌ **אין מודל מתמטי פשוט** של המוח דרך שזירה + φ  
❌ **אין "קוד נסתר"** בטקסט עברי שמתגלה דרך operators  
❌ **אין יחסי φ** שמופיעים ספציפית במילים קדושות/משמעותיות  
❌ **cherry-picking** של גבריאל או מילה בודדת = לא מהווה ראיה

### מה ש**כן** קיים:

✓ **Entropy differences** בין שפה טבעית ל-random (טריוויאלי)  
✓ **Artifacts סטטיסטיים** כמו at_bash bias (ניתן להסביר)  
✓ **Cherry-picked coincidences** מופיעים ב-~11% מכל 5 זוגות  
✓ **Mathematical operators** הם כלים כלליים, לא גילויים

---

## המלצות ל-ZETS

### אל תעשה:
- ❌ אל תבנה מודל ש"מזהה קדושה" דרך φ ב-gematria
- ❌ אל תקבע features מ-at_bash/al_bam בלי לבחון null hypothesis
- ❌ אל תטען "מלאכים משקפים יחסי φ" בלי בדיקה סטטיסטית

### כן תעשה:
- ✓ השתמש באותיות עבריות כ-edge_type identifiers (22 סוגי קשרים)
- ✓ השתמש בגימטריה כ-content hash (מזהה ייחודי, לא תחזית)
- ✓ כשטוענים מישהו על יחס מתמטי — בצע null hypothesis test לפני ציטוט

### הכלל הברזל

**כל טענה על "יחס מתמטי קדוש" חייבת לעבור:**

1. **Sample size:** לפחות 50 מקרים (לא 1-2)
2. **Null comparison:** vs random עם אותה התפלגות
3. **Significance threshold:** p < 0.05 (t-test או chi-squared)
4. **Effect size:** real rate ≥ 2× baseline rate

אם טענה לא עוברת את ה-4 — היא cherry-pick, לא discovery.

---

## סיכום למסמכי Doctrine

**נוסף ל-`brain_architecture_facts.md` תחת 7.X:**

```markdown
### ❌ 7.8 "יחסי φ מוסתרים בגימטריה" — נדחה בבדיקה סטטיסטית
- 2,352 זוגות של מילים תנ״כיות: 1.53% קרובים ל-φ
- 10,000 זוגות random: 2.34% קרובים ל-φ
- **Random רחוק יותר מ-φ מעט יותר ממילים אמיתיות** (ההפך ממה שטוענים)
- Chi-squared vs uniform: not significant
- Cherry-picked cases (גבריאל=246/154) — הסבר: b-random sampling, 11% 
  סיכוי למצוא 1 מקרה קרוב ל-φ מתוך 5 זוגות
```

---

## Evidence files

```
sim/math_tests/operator_tests.py    (10 operators × 4 metrics × 3 corpora)
sim/math_tests/deep_probe.py        (null hypothesis + cherry-pick analysis)
```

ניתן להריץ מחדש כל רגע:
```bash
python3 /home/dinio/zets/sim/math_tests/operator_tests.py
python3 /home/dinio/zets/sim/math_tests/deep_probe.py
```
