# ZETS — טיפול בדיאלקטים (אנגלית אוסטרלית/בריטית/אמריקאית וכו')

**תאריך:** 22.04.2026
**סטטוס:** עיצוב בלבד — לא מימוש עדיין
**בקשת עידן:** "איך אני מתייחס לאנגלית אוסטרלית או אנגליה — זה הרבה קשור לסלאנג, אוצר מילים ואחר כך לדיבור של מבטא ואולי עוד?"

---

## מה שונה בדיאלקטים

דיאלקטים של אותה שפה נבדלים ב-4 מימדים:

### 1. **אוצר מילים** (vocabulary)
אותו רעיון, מילים שונות. דוגמאות:
- `elevator` (US) ↔ `lift` (UK/AU)
- `truck` (US) ↔ `lorry` (UK)
- `cookie` (US) ↔ `biscuit` (UK) — **אבל "biscuit" באמריקאית = מוצר אחר (scone-like)**
- `sidewalk` (US) ↔ `pavement` (UK)
- `eggplant` (US) ↔ `aubergine` (UK)

**⚠ המלכודת:** אותה מילה (biscuit) → concepts שונים בדיאלקטים שונים. זה polysemy, לא רק synonym.

### 2. **סלאנג + דיבור יומיומי** (slang)
מאפיין דיאלקט הרבה יותר מאשר אוצר פורמלי:
- Australian: `arvo` (afternoon), `barbie` (BBQ), `servo` (service station), `heaps of` (lots of), `mate`
- British: `cheers` (thanks/goodbye), `chuffed` (pleased), `gutted` (disappointed), `knackered` (tired)
- American: `y'all`, `dude`, `awesome`, `ride` (car)

סלאנג משתנה במהירות. מילה שהייתה סלאנג ב-2010 יכולה להיות פורמלית ב-2026.

### 3. **איות** (spelling)
- `colour` (UK/AU) ↔ `color` (US)
- `travelled` ↔ `traveled`
- `centre` ↔ `center`
- `realise` ↔ `realize`

שני איותים, אותו concept. זו ה-trivial case.

### 4. **מבטא** (accent — **בדיבור בלבד**)
לא רלוונטי לטקסט. אם ZETS יום אחד יטפל בדיבור (TTS/STT) — זו שכבה נפרדת של `phonetic_representation` שמצביעה מאותו `concept` לאפשרויות הגיה שונות.

**החלטה לעכשיו:** מבטא לא חלק מ-graph data. נטפל בו אם/כאשר יהיה pipeline אודיו.

---

## מה ZETS עושה היום

**`LangId: u8`** (256 שפות אפשריות). אין sub-language. `pack/zets.en` אחד לכל ענף אנגלית.

זה לא מספיק — `cookie:en-US` ו-`biscuit:en-UK` לא יכולים להיות אותו concept אם `biscuit:en-US` הוא משהו אחר.

---

## עיצוב מוצע: hierarchical language IDs

### רעיון 1: "parent + overlay" (הבחירה שלי)

מבנה:
- **שפת-בסיס** = `en`, `he`, `fr`, `es`, etc.
- **דיאלקט** = `en-US`, `en-GB`, `en-AU`, `en-NZ`, `en-IN`
- **דיאלקט יורש מה-בסיס** — כל concept/piece של `en` זמין ב-`en-GB` **אלא אם יש override**

**מבנה הנתונים:**

```rust
// במקום LangId: u8, נעבור ל-LangCode שהוא string קצר (max 7 bytes → פרטי u64).
// "en"    = 2 bytes
// "en-GB" = 5 bytes
// "yue-Hant-HK" (Cantonese in Hong Kong) = 11 bytes — נשתמש ב-BCP 47

pub struct LangId {
    code: [u8; 8],  // null-terminated, BCP 47 subset
    parent: Option<u8>,  // index into LangRegistry of parent (if dialect)
}

pub struct LangRegistry {
    langs: Vec<LangId>,  // "en", "en-US", "en-GB", "en-AU", "he", ...
}
```

**פורמט pack:**
- `zets.en` = base English (כרגע כבר קיים — 6.3MB)
- `zets.en-GB` = **overlay** — רק ה-diffs מ-`en` (override + additions)
- `zets.en-AU` = overlay נוסף
- `zets.en-US` = overlay (חלק מהערכים של `en` ב-`zets.en` עשויים להיות US-ברירת-מחדל; יש לבדוק)

**כשstrongly-typed lookup**:
```rust
fn lookup(word: &str, context_dialect: LangId) -> Vec<ConceptId> {
    // First try the exact dialect
    if let Some(results) = lookup_in_pack("zets.en-AU", word) { return results; }
    // Fall back to parent
    if let Some(results) = lookup_in_pack("zets.en", word) { return results; }
    // Nothing
    vec![]
}
```

### יתרונות
- **חסכוני בזיכרון** — לא כופלים נתוני בסיס
- **כמעט-automatic** — מילים שזהות בין דיאלקטים נכתבות פעם אחת בבסיס
- **נשמר polysemy** — `biscuit:en-US` ≠ `biscuit:en-UK` כי הם atoms שונים ב-overlays שונים
- **פרגון הדרגתי** — אפשר להוסיף `en-IN`, `en-SG` בלי לגעת בבסיס

### חסרונות
- **מורכבות לוגיקה** — כל lookup עכשיו בודק hierarchy
- **overlay-conflict** — כששני דיאלקטים משנים את אותו piece, איך מכריעים? (תלוי קונטקסט של המשתמש)

---

### רעיון 2: "dialect-as-tag" (הדחיתי)

כל piece/concept נושא tag של dialect. Lookup בוחר לפי context.

**חסרונות:** לא חוסך אחסון; כל מילה-אותה-צורה-אותה-משמעות מופיעה 5 פעמים (US/UK/AU/NZ/IN).

### רעיון 3: "flat en with regional weight" (פחות טוב)

כל מילה ב-`zets.en` יש weight וקטור של regional preference.

**חסרונות:** לא עונה על polysemy (biscuit US vs UK).

---

## המלצה

**לאמץ רעיון 1 (parent + overlay).** זה:
- תואם את הpack format הקיים
- חוסך אחסון
- פותר polysemy
- פרגון הדרגתי

**סדר מימוש (לכשנגיע אליו):**

1. הרחב `LangId` מ-`u8` ל-struct של `{code: [u8;8], parent: Option<u8>}`.
2. הרחב `pack.rs` לתמוך ב-`parent` metadata ב-header של lang pack.
3. הוסף `ZetsEngine::open_dialect(code)` — טוען base + overlay.
4. כלל lookup: dialect → parent → fail.
5. התחל עם `zets.en` (base) + `zets.en-GB` (overlay עם 500 מילים ידועות) כ-POC.

**איך נבנה `zets.en-GB`?**
- Seed קטן: 500 מילים UK-specific מתוך רשימה ידועה (lift, lorry, biscuit-UK, ...).
- אוטומטית מאוחר יותר: אם ingest-corpus מקבל טקסט עם `-` איות וsemantic hints של UK
  (e.g., מ-BBC corpus), יפריש את ההבדלים ל-overlay.

---

## מבטא — deferred to audio pipeline

כש-ZETS יטפל בדיבור (TTS/STT — Phase 20+), כל dialect יכול להחזיק:

```rust
struct AccentProfile {
    ipa_transcriptions: HashMap<PieceId, Vec<String>>,  // multiple pronunciations
    phoneme_freq: FrequencyMap,
    typical_intonation: IntonationPattern,
}
```

לא רלוונטי לטקסט. לא מעצב את ה-knowledge graph.

---

## מסקנה

- דיאלקטים = בעיה אמיתית, ברת-פתרון אלגנטי דרך pack inheritance.
- לא לממש עכשיו — לחכות ל-Phase 12 (language hot/cold).
- מבטא = שכבה נפרדת לחלוטין, Phase 20+ (audio).
- 4 קבצים ייווספו ל-repo כשנתחיל: `zets.en-US`, `zets.en-GB`, `zets.en-AU`,
  `zets.en-IN`. כל אחד קטן (100-500KB).
