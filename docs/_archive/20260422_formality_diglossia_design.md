# רמות formality ודיגלוסיה — עיצוב

**תאריך:** 22.04.2026
**בקשת עידן:** "אולי יש שפות שאסור לדבר רשמית במדוברת וברשמית מדוברת?"

---

## כן. הרבה שפות כאלה קיימות.

רמת ה-formality היא **לא תוספת לדיאלקט** — היא עצמה מה שקובע האם אמירה תקינה בהקשר נתון. זה לא overlay קל. זה **בעיה של תחביר ושימוש**, לא רק של אוצר מילים.

---

## הספקטרום

### 1. שפות עם הבדל קל (English, Modern Hebrew, French, German)

אפשר לערבב עם עלות קטנה:
- **עברית**: "אני הולך" ↔ "הנני הולך" — שתיהן תקינות, אחת פורמלית יותר
- **English**: "gonna" ↔ "going to" — שתיהן מובנות, אחת casual
- **French**: "nous allons" ↔ "on va" — שתיהן תקינות, אחת formal

אין "טאבו" — רק inappropriate register.

### 2. שפות עם diglossia חריפה (Arabic, Greek, Tamil, Czech)

**כללים שונים לחלוטין** בין מדוברת ורשמית — לא רק אוצר שונה:

- **Arabic**: فصحى (פסחה, MSA) ↔ عامية (עמיה, Egyptian/Levantine/Gulf)
  - דקדוק שונה (מערכת פועל שונה)
  - אוצר מילים חופף ~70% אבל שונה ~30%
  - **אסור לערבב**: לדבר פסחה בבית קפה = לקרוא מהתנ"ך. לכתוב עמיה בעיתון = לא-חינוכי.
  - רוב הערבים הם native בעמיה, **לומדים** פסחה בבית ספר.

- **Tamil**: Literary Tamil (செந்தமிழ்) ↔ Spoken Tamil (கொடுந்தமிழ்)
  - הבדלים פונטיים + גרמטיים שיטתיים
  - ספרות וטלוויזיה חדשותית — literary; מדוברת בשוק — spoken

- **Greek**: היסטורית Katharevousa ↔ Demotic. נפתר 1976 לטובת Demotic. **היסטורי, לא עכשווי.**

### 3. שפות עם register משמעותי אבל לא diglossic (Japanese, Korean)

- **Japanese**: 5+ רמות (敬語 keigo, 丁寧 teinei, 普通 futsū, タメ口 tameguchi, 幼児 childish)
  - כל רמה עם particles שונים, סיומות שונות, אוצר שונה
  - בחירת רמה שגויה = העלבון/אי-כבוד
  - זה לא diglossia (אין 2 שפות נפרדות), אבל זה כמעט

- **Korean**: 7 רמות דיבור (말투/존댓말)
- Thai, Vietnamese: דומה

### 4. עברית — יש יותר ממה שנראה

- **תנ"כית** (Biblical): שונה מאוד ממודרנית בדקדוק, בפועל, באוצר מילים
- **משנית** (Mishnaic): אקדמית, דומה לתנ"כית אבל עם השפעות יוון וארמית
- **ימי הביניים** (Medieval): ערבית-השפעה, פילוסופית (רמב"ם) או פיוטית (אבן גבירול)
- **מודרנית רשמית** (Modern Formal): עיתונות, אקדמיה, נאומים
- **מודרנית מדוברת** (Modern Spoken): בית, רחוב, WhatsApp
- **סלנג**: מתחדש כל שנה

בעברית **אין** טאבו (אפשר לערבב), אבל מעבר-ללא-הקשר הוא מוזר. "ויאמר יהוה אל משה" בשיחה יומיומית = צחוק.

---

## איך ZETS צריך לייצג את זה

### דחייה: register כ-dialect overlay

**לא עובד.** register לא מוסיף מילים חדשות — הוא משנה **את הגרמטיקה עצמה**. Dialect overlay (כמו שהצעתי בdialects_design.md) טוב לאוצר מילים, לא לגרמטיקה.

### הגישה הנכונה: register כ-edge attribute

כל edge של אוצר מילים או דקדוק מסומן עם `register: RegisterLevel`.

```rust
#[repr(u8)]
pub enum RegisterLevel {
    Sacred = 0,       // טקסטים קדושים, תפילה (בעברית: תנ"ך, סידור)
    Literary = 1,     // ספרות, שירה (עברית פיוטית)
    Formal = 2,       // עיתונות, אקדמיה, נאומים
    Neutral = 3,      // ברירת מחדל — תקין כמעט בכל מקום
    Colloquial = 4,   // מדוברת — בית, חברים
    Slang = 5,        // עכשווי, לא יציב
    Child = 6,        // שפת ילדים / חיבה
}
```

כל `PackedEdge` ב-PieceGraph מקבל field חדש: `register: u8`. זה +1 byte per edge (~13MB ל-wiki) — זניח.

### דיגלוסיה — graph-level flag

**שפה עם diglossia חריפה מקבלת 2 packs ב-pack.rs:**

```
zets.ar          → MSA (فصحى)          register_strict=true
zets.ar-eg       → Egyptian Arabic      register_strict=true, parent=ar
zets.ar-lev      → Levantine Arabic     register_strict=true, parent=ar
```

Meta-graph (מהמסמך הקודם) יודע על ה-`register_strict` flag. כש-walks מעבדים שפה כזו:

```rust
impl LangMetaGraph {
    /// Is this a strict-register language? (Arabic, Tamil, Classical Chinese)
    /// If true, walks must not mix register levels.
    pub fn is_strict_register(&self, lang: &str) -> bool { ... }
}

impl SmartWalk {
    fn walk_with_register(&self, query: &str, lang: &str, register: RegisterLevel) {
        let strict = self.meta.is_strict_register(lang);
        if strict {
            // Only traverse edges with matching register
            self.filter_edges(|e| e.register == register as u8)
        } else {
            // All registers allowed, but boost exact-match
            self.boost_edges(|e| e.register == register as u8, 1.5)
        }
    }
}
```

### מה זה מאפשר

1. **Arabic queries**: שואלים ב-MSA → answer ב-MSA. שואלים ב-Egyptian → answer ב-Egyptian. לא ערבוב.
2. **Hebrew queries**: קונטקסט דתי ("מה אומר הרמב"ם על X") → edges rabbinical/literary. קונטקסט יומיומי ("איך מזמינים פיצה") → colloquial.
3. **Child mode** (עידן ביקש על ים/בן): filter ל-register ≤ 4 (neutral-or-simpler). שפה ילדית בלבד.
4. **Formal output mode**: לפקידים ממשלתיים / מסמכים משפטיים — filter ל-formal+literary.

---

## בעיית דקדוק — מעבר לאוצר מילים

הכלים הקיימים: `morphology.rs` מטפל במורפולוגיה per-language. אבל **גרמטיקה פר register** זה חדש.

פתרון מוצע:

```rust
pub struct MorphologyRules {
    base: MorphologyTable,       // לכל השפה
    overrides: HashMap<RegisterLevel, MorphologyTable>,  // diff per register
}
```

לעברית: `overrides[Literary]` יוסיף את "ואמר" (הקדמת ו' היפוך) שאין ב-Neutral. `overrides[Sacred]` יוסיף vavs הפוכים מלאים.

לערבית: 2 tables נפרדים לגמרי — `ar_fusha.morph` ו-`ar_egy.morph` — כי זו **סוג של 2 שפות**.

---

## מה לממש עכשיו — לא הרבה

### Phase 13.1 — Register attribute
- [ ] הוסף `RegisterLevel` enum ל-piece_graph.rs
- [ ] הוסף `register: u8` ל-`PackedEdge`
- [ ] עדכן pack format (version bump)
- [ ] default = Neutral לכל edges קיימים (backward compat)

### Phase 13.2 — Strict-register flag
- [ ] הוסף `register_strict: bool` ל-`LangId`
- [ ] זהה שפות: ar, tam, gr-classical → strict. en, he, fr → not.
- [ ] walks בודקים את ה-flag

### Phase 13.3 — Child mode filter
- [ ] `SmartWalk::for_persona(Yam)` → auto-filter register ≤ 4
- [ ] Benchmark עם personas: child gets child-level, elder gets full range

### Phase 13.4+ — Arabic diglossia
- [ ] שני packs לערבית (fusha + egy), meta-graph parent relationship
- [ ] 200-query benchmark ב-Arabic עם register mix

---

## קשר ל-D1 (hash registry)

Hash registry פותר את זה **בנוסף**:
- אותו concept "water" בארבית MSA (ماء) ובמדוברת (موية / מיה) → שני atoms שונים
- אבל **אותו content_hash** של ה-concept_core "water" (abstract)
- edges ל-ماء מסומנים `register: Formal`, edges ל-موية מסומנים `register: Colloquial`
- cross-graph vote מראה: "both fusha and egy agree this concept is 'water'"

זה מפריד בין **הfact** (water = H2O = drinkable) לבין **ה-realization** (איך אומרים את זה).

---

## Summary

| רמת הבעיה | פתרון |
|-----------|--------|
| מילים שונות לאותו concept (formal/colloquial) | `register: u8` edge attribute |
| דקדוק שונה (עברית תנ"כית vs מודרנית) | `MorphologyRules.overrides[register]` |
| דיגלוסיה מלאה (Arabic) | 2 packs נפרדים + strict flag |
| Child-safe mode | register-level filter |
| Cross-register concept sharing | content_hash registry (כבר מימשנו) |

לא לממש הכל עכשיו. לממש **Phase 13.1 + 13.3** ראשונים (register attribute + child filter) — זה נותן ROI מיידי על 16 ה-personas שכבר יש לנו.
