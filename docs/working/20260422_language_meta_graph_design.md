# ZETS — גרף-של-גרפים לניהול שפות (meta-graph)

**תאריך:** 22.04.2026
**סטטוס:** עיצוב בלבד — לא מימוש עדיין
**בקשת עידן:** "תראה אם ניתן לנהל את זה חכם בלי למחוק, אולי גרף של שפות או גרף
שנוצר פר שפה אבל יש גרף של גרפים לשפות וככה נאפשר להתנהל עם כל שפה,
אבל קטגנטיבית זה עדיין יהיה מנוהל חסכוני"

---

## המטרה

במקום לבחור איזו שפה "חיה" ואיזו "דחוסה", לבנות **meta-graph** שמתאר את
היחסים בין שפות ודיאלקטים:

- אילו שפות קיימות
- מי יורש ממי (דיאלקטים)
- איזה חלקי של שפה נטענים עכשיו
- איזה "distance" יש בין שפות (למעבר cognitive)

הוא עצמו **גרף ZETS** — atoms + edges — אבל על **שפות**, לא על concepts.

---

## מבנה ה-meta-graph

### Atoms
כל שפה/דיאלקט הוא atom:

```
lang:root          AtomKind::Concept, data="root"
lang:family:indo-european
lang:family:afro-asiatic
lang:family:sino-tibetan
lang:en            — English base
lang:en-US
lang:en-GB
lang:en-AU
lang:he            — Hebrew
lang:ar            — Arabic
lang:zh            — Chinese
lang:zh-Hans       — Simplified
lang:zh-Hant       — Traditional
...
```

### Edges
- `en-GB is_a en`       (דיאלקט יורש משפה)
- `en is_a indo-european`  (שפה שייכת למשפחה)
- `en has_attribute script:latin`
- `he has_attribute script:hebrew`
- `he has_attribute alignment:rtl`
- `he similar_to ar`    (שתיהן semitic)
- `en close_to de`      (germanic + similar grammar)
- `en distant_from zh`  (typologically far)
- `en-GB inherits_from en`  (identical to `is_a` here)

### Runtime state (ephemeral, לא שמור בדיסק)
- `en status:loaded`
- `en-GB status:loaded`
- `en-AU status:cold`
- `fr status:cold`
- `zh status:never-loaded`

### מידע שימוש
- `en last_used:<timestamp>`
- `en usage_count:<u32>`
- `en memory_footprint_kb:<u32>`

---

## איך זה חוסך זיכרון

### סצנריו: משתמש שואל בעברית על מונח טכני באנגלית

1. קונטקסט פעיל: `he` + `en` (core + לשתיהן).
2. שאילתה מגיעה עם מונח "database".
3. Meta-graph lookup:
   - האם "database" ב-`he`? לא.
   - חפש ב-parent של `he` — זה `afro-asiatic`? לא רלוונטי לטכנית.
   - חפש ב-שפות close_to — `en` קרוב (באמצעות edges של concepts משותפים).
   - `en` כבר loaded → מיד יש תשובה.
4. אם לא היה loaded — ה-meta-graph יגלה את הנתיב `he → en (close_to)` ויטען.

### חסכון:
- שפות לא-בשימוש: רק ה-atoms שלהן ב-meta-graph (kilobytes), לא ה-pack המלא.
- 16 שפות נוכחיות בצורת pack = 97MB. Meta-graph = ~50KB.
- שפה נפתחת רק כשמישהו צריך אותה.

---

## עיצוב טכני

### איפה ה-meta-graph גר?

**אופציה 1**: בתוך `zets.core` שכבר טעון תמיד.
- יתרון: חלק מה-graph המרכזי, cross-linking טבעי.
- חסרון: `zets.core` גדל.

**אופציה 2** (מומלץ): קובץ meta נפרד `zets.meta.langs` (~50KB).
- יתרון: טעינה סלקטיבית, אפשר לעדכן בלי ליצור core מחדש.
- חסרון: עוד קובץ לתחזוקה.

### API פרופוזציה

```rust
pub struct LangMetaGraph {
    atoms: AtomStore,       // small, ~50KB
    registry: HashMap<String, AtomId>,  // code → atom
}

impl LangMetaGraph {
    pub fn open(path: &Path) -> io::Result<Self>;
    
    /// Which languages should be loaded given current active set?
    /// Returns (must-load, should-warm-next, can-evict).
    pub fn loading_plan(&self, active: &[&str], recent_queries: &[&str])
        -> LoadingPlan;
    
    /// What's the "cognitive distance" between two langs?
    /// 0 = identical dialect. Higher = more different.
    pub fn distance(&self, from: &str, to: &str) -> u32;
    
    /// Given a query we don't know in lang X, which other lang is most
    /// likely to know it?
    pub fn fallback_suggestion(&self, lang: &str, topic_hints: &[&str])
        -> Option<String>;
    
    /// Find all dialects of a base language.
    pub fn dialects_of(&self, base: &str) -> Vec<String>;
}

pub struct LoadingPlan {
    pub must_load: Vec<String>,    // user is querying these
    pub should_warm: Vec<String>,   // likely needed soon
    pub can_evict: Vec<String>,     // safe to unload
    pub reasons: HashMap<String, String>,
}
```

### Integration עם pack + ZetsEngine

```rust
impl ZetsEngine {
    pub fn smart_load_for_session(&mut self, session: &SessionContext) -> io::Result<()> {
        let plan = self.meta.loading_plan(&session.active_langs(), &session.recent_queries());
        for lang in plan.must_load {
            self.ensure_lang(&lang)?;  // no-op if loaded
        }
        for lang in plan.can_evict {
            self.unload_lang(&lang)?;
        }
        Ok(())
    }
}
```

---

## מה זה נותן על פני ה-pack-only גישה הנוכחית

פיצ'ר שלא קיים היום אבל מתאפשר עם meta-graph:

1. **Auto-loading חכם:** במקום "כל pack שה-app ביקש במפורש", ה-system מחליט מה
   הכי יעיל לטעון עכשיו בסיס השיחה.
2. **Fallback logic:** אם HE לא יודע — ה-meta יודע איזה שפה קרובה יודעת.
3. **Hierarchy של דיאלקטים:** ה-meta יודע ש-en-AU יורש מ-en, מטפל ב-inheritance.
4. **משפחות שפות:** לחקור "איך מילה זו מתנהגת במשפחת Semitic" (he+ar+amharic).
5. **Cold-warm-hot lifecycle:** ברמה של השפה, לא רק ברמת ה-page (כמו היום
   דרך mmap).

---

## סדר מימוש (מוצע, לא חובה)

### Phase 12.1 — Meta-graph file format
- [ ] הגדר את ה-atoms וה-edges של meta-graph.
- [ ] כתוב `zets.meta.langs` קובץ (seed ידני: 16 השפות הקיימות + משפחות).
- [ ] CLI `meta_inspect` — מדפיס את ה-graph.

### Phase 12.2 — Loading plan API
- [ ] `LangMetaGraph::loading_plan()` לפי heuristics פשוטים:
  - Active langs must load.
  - Parents of active langs should load (base → dialect).
  - Untouched for 10 min → can evict.
- [ ] Unit tests לכל heuristic.

### Phase 12.3 — Integration
- [ ] הוסף `meta: Option<LangMetaGraph>` ל-`ZetsEngine`.
- [ ] `smart_load_for_session()` משתמש בזה.
- [ ] Benchmark: HE+EN query continues to pass, FR/DE query auto-loads on demand.

### Phase 12.4 — Distance + fallback
- [ ] `distance()` מחושב באמצעות graph walks (closer = fewer hops).
- [ ] `fallback_suggestion()` — אם שאילתה נכשלת ב-X, הצע Y.

---

## מחיר מול תועלת

| רכיב | מחיר (מורכבות) | תועלת |
|------|----------------|--------|
| Meta-graph file + API | בינוני (1 sprint) | רב — ניהול חכם של 16 שפות |
| Loading plan heuristics | נמוך | רב — חוסך memory אוטומטית |
| Dialect inheritance (עם מסמך dialects) | בינוני | גבוה — פותר polysemy |
| Family-level queries | נמוך | נמוך-בינוני (nice to have) |

**מסקנה:** שווה להשקיע ב-Phase 12.1 + 12.2 + 12.3. 12.4 אפשר לדחות.

---

## קשר למסמך dialects

Meta-graph מספק את ה-**אינפרה** לדיאלקטים:
- ה-`en-GB is_a en` edge ב-meta-graph = השתקפות של `zets.en-GB` להיות overlay על `zets.en`.
- ZetsEngine רואה את ה-edge ויודע איזה packs לטעון יחד.

לא צריך קוד נפרד לדיאלקטים אם meta-graph יוצר + לוגיקת lookup עוקבת
את ה-`is_a` edges.
