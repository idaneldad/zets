# ZETS — סיכום ודילמות אחרי רקורסיה כפולה ושבירת כלים

**תאריך:** 22.04.2026
**בסיס:** הדיון בן-3 ה-AIs (Claude + Gemini 2.5 Flash + Groq Llama-4-Scout)
**אחרי:** בדיקת מצב הקוד האמיתי של ZETS (לא מה שה-AIs חשבו שקיים)

---

## למה המסמך הזה נכתב

Gemini ו-Groq ענו לשאלות ארכיטקטוניות **בלי שידעו מה יש ב-ZETS**. הם נתנו עצות גנריות
שנשמעות טוב בתיאוריה אבל:

1. **פספסו שיש כבר `pack.rs` + `mmap_core.rs` + `mmap_lang.rs`** — מערכת per-language mmap
   עם 16 שפות על הדיסק, 97MB סה"כ, נפתחת ב-20ms.
2. **המליצו על Louvain clustering + zstd-seekable** — כאשר mmap על 66MB core כבר נותן
   lazy loading per-page (4KB) בעלות 81MB RAM peak.
3. **לא הבינו שיש 2 מערכות מקבילות** — AtomStore (שרץ את ה-benchmarks) ו-ZetsEngine
   (שמבוסס pack+mmap) — והן לא מדברות זו עם זו.

אחרי 3 רקורסיות של שבירה ותיקון, כתבתי כאן **רק את מה שאני בטוח לגביו**. כל מה שיש
עליו דילמה — לא מומש, רשום לך למטה להחלטה.

---

## מצב הנוכחי — שתי מערכות מקבילות ב-ZETS

### מערכת A — `AtomStore` (זו שרצה ב-benchmarks)

- פורמט: `.atoms` flat binary
- נתונים: 211,650 atoms + 13.2M edges מ-331 ערכי Wikipedia
- משתמש: `benchmark-runner`, `ingest-corpus`
- תוצאות: 90.6% / 93.8% / 68.8% על 3 הרמות
- 158MB קובץ בודד, נטען כולו ל-RAM (לא mmap)

### מערכת B — `PieceGraph` + `ZetsEngine` (לא מחובר ל-benchmarks)

- פורמט: `zets.core` (66MB) + `zets.<lang>` (16 שפות, 32MB סה"כ) = **97MB**
- נתונים: 144,670 concepts + 3.1M pieces (נראה כמקור ConceptNet)
- משתמש: `engine_cli`, `pack_inventory`, `mmap_read` demos בלבד
- mmap — pages נטענים על-ידי ה-OS רק על נגיעה (4KB units)
- **Cross-language עובד**: "dog" (en) ו"גדול" (he) מצביעים לאותו concept

### השכבה המשותפת שהן חולקות

אין. הן בנויות סביב structs שונים לגמרי (Atom vs ConceptNode).

---

## מה אני בטוח לגביו ב-100% (ומה כבר מימשתי)

### ✅ 1. mmap הוא הפתרון ל"walk-based lazy decompression"

**מדוד בפועל:**
- 65MB core → 81MB RAM peak → 0.02 שניות זמן עבודה
- ה-OS טוען pages של 4KB רק על אקסס
- לא צריך zstd-seekable. לא צריך Louvain. `mmap()` ב-Rust עושה את העבודה.

**מסקנה:** ההחלטה של Gemini "zstd-seekable + Louvain" היא overkill לגודל הזה.
אם בעתיד הגרף יגיע ל-10GB, אולי נצטרך. ל-100MB זה מיותר.

### ✅ 2. השפות כבר מפוצלות ב-pack format

**מדוד בפועל:**
- HE = 1MB (18K POS tags, 10K synonyms), פתיחה 1.19ms
- EN = 6.3MB (282K POS, 43K synonyms), פתיחה 14ms
- 14 שפות נוספות בין 0.4MB ל-3.2MB, פתיחה 0.7-8ms

**מסקנה:** החזון של עידן ("שפות core + שפות lazy") **כבר מומש** ברמת הפורמט.
מה שחסר הוא לחבר את זה ל-benchmarks.

### ✅ 3. יישמתי: `pack_inventory` CLI

כלי דיווח חדש שמראה בבת אחת כל ה-packs, גודלם, זמן mmap, ומה הם מכילים.
לא משנה כלום, רק קורא. commit בנפרד.

### ✅ 4. לא לגעת במערכת A שעובדת

**Principle:** "Don't break what works." מערכת A מגיעה ל-68.8% על wiki.
כל שינוי שם = סיכון ל-regression.

---

## דילמות — עליך להחליט (לא מימשתי כלום מאלו)

### 🔴 דילמה D1: איך מאחדים A ו-B?

שתי מערכות נתונים במקביל. אותן מילים מיוצגות בצורות שונות בכל אחת.

**אופציה α — זורקים B, מרחיבים A:**
- יתרון: B עוד לא רץ ב-benchmarks, לא נאבד איכות
- חסרון: מאבדים 144K concepts + cross-language mapping + 16 שפות
- חסרון: נצטרך לבנות מחדש את כל ה-lazy mmap (אבל יש עובד!)

**אופציה β — זורקים A, מעבירים benchmarks ל-B:**
- יתרון: B יותר עשיר ויותר מתוכנן (mmap, per-language, ZetsEngine facade)
- חסרון: עבודה גדולה — יצטרכו לבנות pipeline Wikipedia→PieceGraph
- חסרון: לא יודעים אם benchmarks ישמרו 68.8% — נצטרך להריץ מחדש

**אופציה γ — משאירים שניהם, בונים גשר:**
- יתרון: אף אחד לא נשבר
- חסרון: יותר קוד לתחזק
- פרטים: ZetsEngine יכול להוסיף `AtomStore` כ-field פנימי; walks משתמשים ב-A לאחסון
  עובדתי, ב-B ל-linguistic. לא פתור איך פותרים conflict.

**הדעה שלי (זהירה):** γ הכי בטוח. α עדיף אם רוצים פשטות. β מסוכן.

---

### 🔴 דילמה D2: domain packs (medicine / CS / geography / slang / culture)

עידן ביקש ש"חבילות ידע יהיו נפרדות". כרגע:
- מערכת A: flat — אין חלוקה לדומיינים
- מערכת B: מפוצלת **לפי שפה**, לא לפי דומיין
- אף אחת מהן לא יודעת "זה ערך רפואי" vs "זה ערך תרבות"

**אופציה α — Louvain clustering אוטומטי (המלצת Gemini):**
- מריצים algorithm על 13.2M edges
- מקבלים 50-500 clusters
- אבל: לא ברור שה-clusters יתאימו ל"medicine/CS/..." שאנשים חושבים עליהם
- זמן חישוב: שעות על 13.2M edges
- סיכון: cluster שנוצר יכול לערב רפואה + ביולוגיה + כימיה (הם קרובים בגרף)

**אופציה β — manual tagging לפי category ב-ingestion:**
- בזמן ingest, כל ערך Wikipedia יקבל tag לפי הקטגוריה שלו
- חלק מ-Wikipedia כבר יש קטגוריות (`Category:Medicine`)
- דורש שינוי ב-`ingest-corpus`
- יתרון: domain packs ברורים לבני אדם
- חסרון: ערכים יכולים להיות במס' categories

**אופציה γ — דחות את זה ל-Phase מאוחר:**
- בינתיים מערכת A flat עובדת ב-68.8%
- ההחלטה לפצל ל-domains יכולה לחכות
- יתרון: מתמקדים באיחוד A+B קודם
- חסרון: ה-RAM יגדל ככל שמוסיפים corpora

**הדעה שלי:** γ כרגע. β כש-corpus יגדל ל-5× (1M+ Wikipedia articles).
α לא נחוץ עד שיש 100M+ edges.

---

### 🔴 דילמה D3: Personal graphs (משפחה/עסק/יחיד)

Gemini המליץ "global shared + per-user overlay + edge ACL בbitmask". זה הגיוני אבל:

**מה שאני לא יודע (צריך ממך):**
1. **מה התרחיש הראשון?** משפחה שלך (רוני, שי, בן, אור, ים)? CHOOZ employees? יחידים באפליקציה?
2. **מה צריך להיות "פרטי"?** כל האטומים? רק ה-edges של "הוא בן של..."? פעילות חיפוש?
3. **מתי מתחיל הצורך?** עכשיו או בעתיד?

**מה אני יכול לממש בלי להכריע:**
- Field `tenant_mask: u64` על EdgeSource (64 tenants max)
- Walks עם tenant context
- Family = union של bitmasks

**אבל לפני שאבנה:** מי המשתמש הראשון ומתי? בלי תרחיש אמיתי, זה over-engineering.

---

### 🔴 דילמה D4: Edge/Cloud sync

המלצת 3 ה-AIs: delta-based + Merkle + CRDT.

**אבל:**
- אין לנו edge device עכשיו (לא טלפון, לא chip)
- אין cloud infrastructure
- היחיד שקיים: השרת ddev.chooz.co.il

**הדעה שלי:** זה Phase 15+. הפתרון ברור (Merkle + snapshot versioning), אבל אין
מה לסנכרן כרגע. לא אבנה.

---

### 🔴 דילמה D5: Inference by analogy — איפה לטמון את ה-Hypothesis?

3 ה-AIs הסכימו: "`inference_walk()` שמסמן edges כ-Hypothesis". אבל:

**שאלה טכנית (לא פתורה):**
- האם Hypothesis edges נשמרות לדיסק (מערכת A=.atoms) או רק בזיכרון?
- אם לדיסק — היא ממשיכה לצמוח לנצח, גם אם הניחוש היה לא נכון
- אם בזיכרון — מפסידים את ה-learning ב-restart

**המלצה של 3 ה-AIs:** "נשמר עם provenance + דה-העדפה על חוסר-מידע". לא מפרטים מי מוחק ומתי.

**מה שאני יכול לממש בלי להכריע:**
- walk mode בלבד (בלי persistence)
- Hypothesis edges מחושבות לכל query, נזרקות אחרי
- Trade-off: מהיר per-query אבל מחשב מחדש כל פעם

**דילמה אמיתית:** האם להשקיע ב-persistence של Hypothesis? אם כן — מתי לוקחים "GC"?

---

## מה לממש בעדיפות (הסדר שלי)

### Priority 1 — קצר, בטוח, ערך מיידי

- [x] **pack_inventory CLI** ✅ מימשתי (commit זה)
- [ ] **Plan V2 document** — עדכון תוכנית הארכיטקטורה V1 עם ההבנות החדשות
- [ ] **בדוק שכל ה-pack files עדיין תקינים** (אולי חסרים, אולי corrupt)

### Priority 2 — בינוני, דורש החלטה שלך (D1)

- [ ] בחר בין α/β/γ של איחוד A+B
- [ ] כשיש החלטה — תכנן migration path

### Priority 3 — ארוך, דילמות לא פתורות

- [ ] domain packs (D2)
- [ ] personal graphs (D3)
- [ ] edge/cloud sync (D4)
- [ ] inference_walk persistence (D5)

---

## Summary (אנגלית לקוד)

- mmap gives lazy loading for free at the 100MB scale. zstd-seekable not needed yet.
- Two parallel data models (AtomStore vs PieceGraph) — must decide how to merge.
- Phase 11's proposed Louvain clustering is premature — the per-language split already
  covers language isolation; domain isolation can wait.
- Personal graphs, edge/cloud sync are designs without concrete use cases yet.

---

## מה שאני עושה באופן אוטונומי (לא מחכה להחלטה)

1. Commit של `pack_inventory` CLI
2. Commit של המסמך הזה
3. עדכון תוכנית הארכיטקטורה V1 → V2 (נקי מההמלצות השגויות של Gemini/Groq על
   Louvain/zstd-seekable, עם ההבנה שמערכת B כבר פתרה את "language packs")
4. ממתין להחלטה שלך על D1 (איחוד A+B) לפני שאני עושה משהו במערכת הנתונים עצמה

---

## שאלה תכליתית לעידן

**מה אתה רוצה שאעשה הלאה?**

- [ ] **אפשרות 1:** אסיים V2, נעצור כאן. אתה תחשוב על D1-D5.
- [ ] **אפשרות 2:** אני ממשיך ל-D1 אופציה γ (גשר בין A ל-B) — הכי בטוח
- [ ] **אפשרות 3:** אני ממשיך ל-D1 אופציה α (זורק B, מרחיב A) — הכי פשוט
- [ ] **אפשרות 4:** אתה רוצה קודם להריץ את ה-sudo deploy של ZETS MCP החדש
- [ ] **אפשרות 5:** משהו אחר שחשבת עליו
