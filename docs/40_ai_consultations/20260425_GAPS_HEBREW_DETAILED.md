# 22 הפערים של ZETS — הסבר בעברית ברורה

**מטרת המסמך:** לכל פער — תיאור פשוט, פתרון, ערך, דוגמה, ומה ההבדל בין "טוב" ל-"מעולה".

**הסטטוס:**
- 5 נסגרו רעיונית
- 17 פתוחים

---

# ✅ הפערים שנסגרו (5)

## #1 — אחסון קשתות (Edge Storage)

**הכוונה:** איך לאחסן את כל הקשתות בגרף כך שגם נוכל להוסיף מהר וגם לקרוא מהר.

**הפתרון שנבחר:** שכבה תחתונה (CSR) קוראת מהר, שכבה עליונה (append-only log) כותבת מהר, ובלילה NightMode מאחד אותן.

**הערך:** ZETS יכול ללמוד תוך כדי שהוא עונה — בלי לעצור הכל לשחזור.

---

## #7 — חיזוי (Predictive Processing)

**הכוונה:** ZETS מנחש מה המשתמש הולך לכתוב/לבקש לפני שסיים.

**הפתרון:** 7 שכבות של חיזוי (מילה הבאה, ביטוי, שאלה, כוונה, צורת תשובה, שאלת המשך, דפוסים אישיים) — כולן מבוססות גרף, לא LLM.

**הערך:** ZETS מתחיל לעבוד מהר יותר, יכול ליזום פנייה, ומציע 3 כפתורי המשך כמו Perplexity.

---

## #8 — חלימה / יצירתיות (Idle Dreaming)

**הכוונה:** ZETS חושב על קשרים חדשים גם כשאף אחד לא שאל.

**הפתרון:** **רק לפי בקשה** (לא אוטונומי). User/procedure קוראים `dream_about(topic)`, ZETS מחזיר proposed edges, המשתמש מאשר.

**הערך:** יצירתיות בסגנון ADHD/דיסלקטים (קישורים cross-domain) בלי שהמשתמש יאבד שליטה.

---

## #10 — נרטיב עצמי (Self-Narrative)

**הכוונה:** ZETS זוכר מה הוא עצמו עשה — שיחות, החלטות, למידה.

**הפתרון:** PersonalVault[zets_self] — לוג עובדתי (לא narrative מומצא). שאלות אפשריות: "מה עשית בשעה האחרונה?", "מתי הגעת לרעיון X?".

**הערך:** ZETS הופך לישות עם זהות מתמשכת, לא רק chatbot stateless.

---

## #11a — TMS שלד (Cardinality Schema)

**הכוונה:** טיפול בעובדות סותרות / כפולות.

**הפתרון:** 6 קטגוריות (Single, Multi, TimeBound, Conflicting, Subjective, ContextMulti) + Conflict Disclosure ("יש 2 כתובות, איזו פרשנות נכונה?").

**הערך:** אין hallucinations. ZETS אומר "לא יודע" כשאין מספיק וודאות.

---

# 🔴 הפערים הקריטיים הפתוחים (6)

## #4 — מודל שפה קטן מקומי (Phi-3-mini)

### מה הפער?
ZETS הליבה היא גרף **מדויק ונוקשה**. אבל אנשים מדברים בצורה **מבולגנת** — "נו, היא אמרה לך מה שאני אמרתי?". בלי גשר שפתי, ZETS לא יבין.

### איך לפתור?
שיתוף Phi-3-mini (3.8B params, 1.8GB quantized) שעובד מקומית על ה-CPU:
- מקבל input → מוציא parsed atoms + intent
- מקבל graph result → מוציא תשובה בעברית טבעית
- **הוא לא קובע עובדות** — רק מתרגם

### הערך
ZETS נשמע כמו אדם, לא כמו רובוט. משתמשים סובלים אותו.

### דוגמה
**בלי LM:**
```
משתמש: "נו, מה אמרה שי על הפרויקט?"
ZETS: ERROR — pronoun "she" unresolved, "the project" ambiguous
```

**עם LM:**
```
משתמש: "נו, מה אמרה שי על הפרויקט?"
LM parses: subject=שי, predicate=said, object=opinion_about(current_project)
LM picks "current_project" from session context
ZETS walks graph → finds "שי שיבחה את העיצוב"
LM realizes: "שי שיבחה את העיצוב, במיוחד את הצבעים."
```

### מה צריך להיות שונה כדי שזה יהיה מעולה?
- **טוב:** Phi-3-mini standard, רץ ב-CPU, 30 tokens/sec
- **מעולה:** fine-tuned על שיחות עברית של עידן עצמו → register מתאים, מילים שעידן באמת משתמש בהן, latency 100ms

---

## #5 — Fuzzy Hopfield Fallback (חיפוש דמיון)

### מה הפער?
ZETS עובד על atoms ברורים. אם משתמש שואל על משהו שאין בגרף — dead end. LLMs "מרגישים" דרך embedding space, ZETS לא.

### איך לפתור?
HNSW index (Hierarchical Navigable Small World) על embeddings של atoms:
- שאלה → embedding → top-K nearest atoms
- אם exact match נכשל → walk מ-K הקרובים → תשובה עם confidence נמוכה + disclaimer

### הערך
תמיד יש תשובה (גם אם פחות בטוחה). אין "לא יודע" מוחלט.

### דוגמה
**בלי Fuzzy:**
```
משתמש: "מה זה CapybaraGPT?"
ZETS: לא נמצא בגרף.
```

**עם Fuzzy:**
```
משתמש: "מה זה CapybaraGPT?"
ZETS: לא מכיר ישירות. לפי דמיון לאטומים אחרים, נראה שזה LLM 
       (כמו ChatGPT) עם branding של חיה (capybara). 
       רוצה שאחפש מידע ספציפי?
```

### מעולה לעומת טוב
- **טוב:** HNSW עם 384-dim embeddings, top-10 neighbors
- **מעולה:** embeddings מותאמים לעברית + Kabbalistic similarity (גימטריה, צורת אותיות) כמדד נוסף

---

## #11b — TMS מלא (Truth Maintenance System)

### מה הפער?
מי אמין? מי סותר? מה היה נכון מתי? בלי TMS, ZETS מתבלבל בין מקורות, חוזר על שטויות, ולא יודע למתי הוא מכוון.

### איך לפתור?
4 שכבות:
1. **Provenance** על כל edge (מי המקור, מתי, איך)
2. **Trust scoring** לכל מקור (0-1, נלמד עם הזמן)
3. **Conflict resolution** — אם פער מצומצם → "לא יודע", אם פער ברור → trusted source נצח
4. **"לא יודע" כ-state ראשי** בכל מקום ב-API

### הערך
אנשים סומכים על ZETS. כשהוא אומר משהו — זה נכון. כשהוא לא בטוח — הוא אומר.

### דוגמה
**בלי TMS:**
```
2024: שי גר בתל אביב (מסמך A)
2025: שי גר בחיפה (פוסט פייסבוק)
שאלה: "איפה שי גר?"
ZETS: שי גר בתל אביב. (השני נדחה כי המבנה לא יודע איזה נכון)
```

**עם TMS:**
```
ZETS: לפי מסמך רשמי מ-2024, שי גר בתל אביב.
       יש פוסט פייסבוק מ-2025 שאומר חיפה — אבל זה מקור פחות אמין.
       רוצה שאבדוק או תאשר ידנית?
```

### מעולה לעומת טוב
- **טוב:** trust scoring סטטי, conflict detection בסיסי
- **מעולה:** trust הופך **למידה דינמית** (אם מקור התברר כשגוי → ה-trust שלו צונח), confidence propagation לאורך chains, time-aware queries ("מה היה נכון בינואר?")

---

## #13 — ידע כללי (Common-Sense / World Knowledge)

### מה הפער?
LLMs יודעים "אם גשם → רטוב" כי קראו 1B עמודי אינטרנט. ZETS מתחיל ריק — חייבים ללמד אותו edge by edge.

### איך לפתור?
Batch enrichment pipeline:
- Wikipedia dumps → ZETS atoms
- Wikidata structured facts → edges
- ConceptNet (commonsense) → edges
- Gemini Flash batch calls: "ל-atom X תן לי 10 commonsense facts"
- ~1000 concepts/hour, ~$0.10/hour

### הערך
ZETS עובר מ-"savant עם חורים" ל-"מומחה רחב". אחרי 3 חודשי enrichment = coverage מלא של Wikipedia.

### דוגמה
**בלי enrichment:**
```
משתמש: "אם יורד גשם בחוץ ויש לי כלב — מה כדאי לעשות?"
ZETS: רק יודע מה זה גשם וכלב. לא יודע שכלבים נרטבים, נצנפלים, צריך מטריה.
```

**עם enrichment:**
```
ZETS: כדאי לקחת מטריה. כלבים נרטבים בגשם ועלולים להצטנן. 
       עדיף לקצר את הטיול. אם הוא ירטב — לייבש עם מגבת.
```

### מעולה לעומת טוב
- **טוב:** ConceptNet + Wikipedia → 1M facts בגרף
- **מעולה:** **active learning** — ZETS מזהה איפה יש "חור ידע" (הרבה שאלות נכשלות באזור) → autonomous request להעשרה ספציפית של אותו תחום

---

## #14 — מתכנן (Planner Under Uncertainty)

### מה הפער?
ZETS עונה על שאלות. אבל "תעזור לי לקבוע פגישה עם שי לשבוע הבא" דורש **תכנון**: בדוק calendar → מצא חלון → שלח invite → אשר. אין מנגנון לזה.

### איך לפתור?
Classical AI planner + graph integration:
- A* / MCTS על מרחב procedures
- Cost estimation per action (משתמש ב-Affective State #9)
- Fallback strategies (אם X נכשל → נסה Y)
- Goal decomposition דרך motifs

### הערך
ZETS הופך מ-Q&A bot ל-**agent אמיתי**. יכול לבצע משימות מורכבות.

### דוגמה
**בלי planner:**
```
משתמש: "תקבע לי פגישה עם שי בשבוע הבא"
ZETS: אני יכול לחפש את האימייל של שי בלוח הקשרים שלך.
```

**עם planner:**
```
ZETS: בודק... שי פנוי שלישי 14:00, חמישי 10:00, שישי 9:00.
       Calendar שלך פנוי שלישי 14:00 וחמישי 10:00.
       שילחת לו לאחרונה 3 הצעות בבוקר — אולי עדיף שלישי 14:00?
       אשלח invite?
```

### מעולה לעומת טוב
- **טוב:** A* על procedures, מחזיר plan
- **מעולה:** **plan explanation** — ZETS מסביר למה בחר plan A על pan B, ו-replanning דינמי כשמשהו משתנה באמצע

---

## #18 — Cache Thrashing (טיפול בכשלי cache)

### מה הפער?
ZETS מתפאר ב-2 ננו-שניות לפעולה. אבל random walks על גרף 6GB → כל hop = cache miss = 100ns. **התוצאה: ZETS פי 50 לאט מהיעד.**

### איך לפתור?
Cache-aware graph layout:
- Vertex reordering (Rabbit Order, METIS) — atoms שכיחים-יחד, באותו memory page
- Hot atoms cluster — top 1000 בLevel 1 cache
- Prefetching — ZETS מבקש מה-CPU להביא מראש את ה-atoms הצפויים
- Workload-based reorganization — atoms שנקראו ביחד → יושבים ביחד

### הערך
ה-2ns target הופך מאמירה למציאות. ZETS באמת מהיר.

### דוגמה
**בלי mitigation:**
```
Walk: atom_3847 → atom_8392 → atom_1224 → atom_6651
Each hop: cache miss (100ns)
Total walk of 5 hops: 500ns (פי 250 מהיעד!)
```

**עם mitigation:**
```
Walk: atom_3847 → atom_8392 (same page, prefetched)
                → atom_1224 (cluster member, in L2)
                → atom_6651 (prefetched during atom_1224 read)
Each hop: 2-5ns
Total walk of 5 hops: 10-25ns ✓
```

### מעולה לעומת טוב
- **טוב:** vertex reordering one-time
- **מעולה:** **adaptive reordering** — ZETS מודד אילו walks הם hot, ומסדר את הגרף **בלילה** ב-NightMode כדי שהם יהיו cache-friendly. הביצועים משתפרים מעצמם עם השימוש.

---

## #20 — WASM Sandboxing (אבטחה לקוד עצמי)

### מה הפער?
אם ZETS יכול ליצור procedures חדשים (קוד שרץ), בלי sandbox הוא יכול:
- למחוק את עצמו (`rm edges_csr.bin`)
- לדלוף מידע פרטי
- להיכנס ל-infinite loop ולתקוע את המערכת

### איך לפתור?
Strict WASM sandbox:
- כל code חדש compiled to WASM bytecode
- רץ ב-wasmtime עם capability model:
  - אין filesystem access (ללא הרשאה מפורשת)
  - memory limit (למניעת OOM)
  - execution time limit
  - אין רשת
- הרשאות נתנות **explicitly** per-procedure

### הערך
ZETS יכול ללמוד procedures חדשים ולהריץ אותם **בבטחה**. self-extension אמיתי.

### דוגמה
**בלי sandbox:**
```
ZETS למד procedure "פנוי disk":
  while True: rm /home/dinio/zets/data/...
ZETS = dead.
```

**עם sandbox:**
```
ZETS למד procedure "פנוי disk":
  WASM module compiled
  Capabilities granted: read /tmp, write /tmp/cleanup.log
  No filesystem write to /home/dinio/zets/data
  Time limit: 60 sec
  
ZETS runs it safely. Worst case: timeout, sandbox crashes, ZETS continues.
```

### מעולה לעומת טוב
- **טוב:** WASM sandbox עם capability model
- **מעולה:** **automatic capability inference** — ZETS מנתח code חדש ומציע אילו capabilities הוא צריך, מבקש אישור user, ולומד מה standard לתחום (לדוגמה — procedures של email תמיד מקבלים send capability, אבל לא filesystem)

---

## #22 — Parse-to-Graph Boundary (הסיכון הגדול ביותר)

### מה הפער?
**Gemini קרא לזה "the SINGLE biggest risk".** ZETS מסתמך על parsing מדויק של שפה ל-atoms. אם parse שגוי = 6GB של גרף garbage. הכל קורס.

### איך לפתור?
Multi-layer defense:
1. **Confidence threshold** — אם parse confidence < 0.8 → שאל הבהרה
2. **Explicit confirmation** — atoms חדשים → אשר עם user לפני יצירה
3. **Rollback** — אם parse התברר שגוי, מחק את כל ה-edges שתלויים בו
4. **Audit trail** — log לכל parse decision
5. **Re-parse pass** — NightMode מבקר parses אחרונים עם models חדשים

### הערך
data integrity לאורך זמן. ZETS לא מתפורר אחרי שנה של שימוש.

### דוגמה
**בלי defense:**
```
משתמש: "שי קנה את הבית של ההורים שלי"
Parse error: "ההורים שלי" → atom_my_parents (הורי המשתמש)
                                vs
                              atom_shai_parents (הורי שי)
ZETS picks WRONG one.
1 שנה אחר כך, גרף מבולגן עם עשרות edges שגויות.
```

**עם defense:**
```
ZETS: רגע — "ההורים שלי" — של מי? אתה מתכוון להוריך 
       (לפי context הקודם, דיברנו על הוריך) 
       או הורי שי?
משתמש: הורי
ZETS: אישרתי. שי קנה את הבית של הוריך.
```

### מעולה לעומת טוב
- **טוב:** confidence threshold + confirmation
- **מעולה:** **causal graph of parses** — ZETS שומר את שרשרת ההסקה שהובילה לכל edge. אם parse התברר שגוי, אפשר לתקן את הסיבה ולא את הסימפטום, ו-ZETS לומד מהטעות לעתיד.

---

# 🟡 הפערים החשובים (9)

## #2 — סכמה רשמית לקשתות (Edge Schema)

### מה הפער?
כל edge מסוג IsA, HasPart, Causes... בלי סכמה רשמית, walks הולכים בכיוונים שגויים.

### איך לפתור?
לכל edge kind: direction, inverse, transitivity, domain/range constraints.

### הערך
walks נכונים, compile-time safety, אופטימיזציות.

### דוגמה
- IsA: directed, transitive (כלב → חיה → יצור חי)
- Synonym: bidirectional, NOT transitive
- Owns: directed, NOT transitive

### מעולה לעומת טוב
- **טוב:** 22 edge kinds מוגדרים
- **מעולה:** 22 edge kinds **מקבילים ל-22 אותיות עבריות** — כל אות-קשת = שער יצירה (ספר יצירה).

---

## #3 — דחיסה (Compression: Huffman + Delta)

### מה הפער?
Article paths תופסים 2GB. עם Zipf-distribution natural ב-language, אפשר לדחוס דרמטית.

### איך לפתור?
- Huffman: atoms שכיחים → 1 byte
- Delta: ההפרש מהאטום הקודם, לא absolute

### הערך
2GB → 600MB. **חיסכון 1.4GB.**

### דוגמה
Path של 500 atoms = 2000 bytes
Huffman: 850 bytes
+ Delta: 600 bytes

### מעולה לעומת טוב
- **טוב:** static Huffman table
- **מעולה:** **adaptive Huffman** שמתעדכן עם השימוש — אם מילה מסוימת הופכת חמה אצל user מסוים, היא עוברת לקוד 1-byte specifically בvault שלו.

---

## #6 — Global Workspace (זרקור תודעה)

### מה הפער?
walks מקבילים רצים בלי תיאום. אין "תמקדות". זה כלי, לא תודעה.

### איך לפתור?
לוח top-20 atoms פעילים. כל module משדר אליו ומקשיב ממנו.

### הערך
תיאום בין modules. בסיס לתודעה פונקציונלית (תיאוריית Baars/Dehaene).

### דוגמה
**בלי workspace:**
```
Walker A finds atom_X
Walker B finds atom_Y
Walker C finds related to both X and Y
But A, B, C don't know about each other.
Result: missed connection.
```

**עם workspace:**
```
A broadcasts X with salience 0.8
B broadcasts Y with salience 0.7
C reads workspace, sees both, connects them
Result: insight emerges.
```

### מעולה לעומת טוב
- **טוב:** top-20 buffer מהיר
- **מעולה:** **competitive attention** — atoms מתחרים על מקום ב-workspace, לא רק "highest score wins". הפסד ל-X גורם ל-X לנסות שוב מ-angle אחר.

---

## #9 — מצב רוח (Affective State)

### מה הפער?
ZETS לא מותאם ל-context רגשי. תמיד עובד אותו דבר.

### איך לפתור?
4 ערכי i8: frustration, curiosity, confidence, fatigue. משתנים עם success/failure.

### הערך
התאמה דינמית. נכשל הרבה → walks דיותר רחבים. מוצלח → ממוקדים.

### דוגמה
- 5 כשלונות ברצף → frustration עולה → walks deeper, יותר exploratory
- 10 הצלחות → confidence עולה → walks ממוקדים, מהירים

### מעולה לעומת טוב
- **טוב:** 4 ערכים סטטיים, מתעדכנים על success/failure
- **מעולה:** **mood as context** — ZETS מסביר למשתמש את ה-state ("אני קצת מבולבל היום, נסה לנסח אחרת"), ומפתח **שעון biological** — אנרגיה משתנה לאורך היום, fatigue אחרי 8 שעות.

---

## #12 — Frozen Regression Suite (מבחני יציבות)

### מה הפער?
ZETS אמור להיות דטרמיניסטי. בלי בדיקות אוטומטיות, "דטרמיניסטי" זה רק הצהרה.

### איך לפתור?
500+ tests שרצים בכל commit:
- Same query → same answer
- Same walk → same intermediate atoms
- Latency in budget
- Memory in budget

### הערך
דטרמיניזם **מוכח, לא מובטח**. Refactoring בטוח. גילוי regression.

### דוגמה
- שינוי ב-walks logic → 12 tests נופלים → רואים מה השפיע מיד
- בלי tests: שינוי שובר משהו → גילוי 6 חודשים אחרי, debug של ימים

### מעולה לעומת טוב
- **טוב:** snapshot tests + property-based
- **מעולה:** **mutation testing** — מערכת שמשנה bytes אקראיים בקוד ובודקת שלפחות test אחד נופל. אם לא — ה-tests לא מספיקים. נותן metric אמיתי לאיכות tests.

---

## #15 — Lightweight Learned Ranker (מודל דירוג קטן)

### מה הפער?
כש-walks מחזירים 20 candidates, איך מדרגים? graph score לא רגיש לnuance, Phi-3-mini יקר ל-20 candidates.

### איך לפתור?
Cross-encoder קטן (10-50M params, 10-50ms): (query, candidate) → relevance score 0-1. נלמד מ-clicks של user.

### הערך
30-50% improvement בdrijking precision על pure graph walk.

### דוגמה
שאלה: "מי המנכ"ל של Google?"
Candidates: 20 atoms שקשורים ל-Google ולמנכ"לים
Graph walk: כולם עם score דומה
Ranker: מבחין ש"Sundar Pichai" יותר רלוונטי מ-"Larry Page" (founder, לא current CEO)

### מעולה לעומת טוב
- **טוב:** cross-encoder generic
- **מעולה:** **personal ranker per user** — ZETS לומד את העדפות הניסוח של עידן (פורמלי? קצר? מדויק?), ה-ranker מתאים את עצמו אישית.

---

## #16 — איכות שפה ביציאה (NL Realization)

### מה הפער?
תשובות ZETS נשמעות רובוטיות — בעיקר בעברית, עם דקויות register וזרימה.

### איך לפתור?
Template-based generation + LM polish:
1. Build structured response from walk
2. Apply Hebrew templates (correct agreement, ניקוד אופציונלי)
3. LM pass: register + flow
4. Fallback: template-only אם LM unavailable

### הערך
ZETS נשמע כמו מומחה אנושי, לא כמו תוכנה.

### דוגמה
**רובוטי:**
"שי בעל עמדת מנכ"ל ב-CHOOZ. CHOOZ מבצע פעילות מסחר B2B."

**אנושי:**
"שי הוא מנכ"ל CHOOZ — חברת B2B עם 12,000 לקוחות עסקיים."

### מעולה לעומת טוב
- **טוב:** templates נכונים + LM polish
- **מעולה:** **register matching to user** — אם user מנהל מקצועי, register פורמלי. אם user מתבדח, register קליל. אם user ילד, פשוט.

---

## #17 — אנלוגיות בין-תחומיות (Zero-Shot Analogical Transfer)

### מה הפער?
ZETS מתעצר ב-domain אחד. LLMs יכולים "לקשר" quantum mechanics ל-baking via stylistic mimicry.

### איך לפתור?
Structure-Mapping Engine (Gentner 1983): מזהה תפקידים מבניים בdomain A, מחפש analogs ב-B.

### הערך
ZETS יכול לחשוב metaphorically, לקבל insights cross-domain.

### דוגמה
**בלי SME:**
```
משתמש: "איך leadership דומה לגננות?"
ZETS: זה שני תחומים שונים, אין להם קשר ישיר.
```

**עם SME:**
```
ZETS: שניהם עסקים בצמיחה. מנהיג = גנן (cultivator).
       צוות = צמחים (need different conditions).
       החלטות = השקיה (timing matters).
       תרבות ארגונית = אדמה (foundation for everything).
```

### מעולה לעומת טוב
- **טוב:** structure mapping בין שני domains נתונים
- **מעולה:** **proactive analogy** — ZETS מציע אנלוגיות אוטונומית כשהוא רואה שמשתמש מתקשה: "אולי תועיל לך אנלוגיה מ-X?"

---

## #19 — דקדוק עברי (Morphological Rule Explosion)

### מה הפער?
עברית = 7 בניינים × זמנים × מינים × גופים × ריבויים + יוצאים מן הכלל = אלפי כללים. בלי priorities → conflicts.

### איך לפתור?
Prioritized rule system: יוצאים מן הכלל קודם, כללים כלליים אחרון. First match wins.

### הערך
דטרמיניזם בDestrukt דקדוק עברי.

### דוגמה
מילה: "כתבתי"
- Rule 1 (specific): suffix "תי" → 1ms past
- Rule 2 (general): root extraction by 3 letters → "כתב"
- Rule 1 wins → tense=Past, pgn=1ms
- Rule 2 applies → root=כתב

### מעולה לעומת טוב
- **טוב:** rules סטטיים מסודרים
- **מעולה:** **rules נלמדים** — ZETS מזהה patterns חדשים בטקסט (loanwords, slang) ומציע rules חדשים, user מאשר.

---

## #21 — Code Quarantine (TrustLevel)

### מה הפער?
אם ZETS יוצר procedures חדשים, איך לדעת לאיזה לסמוך?

### איך לפתור?
Trust hierarchy:
- Experimental: רק ב-sandbox
- Tested: עבר test suite
- Human_Approved: user אישר
- Core: built-in

Promotion: tests + user approval + N successful runs.

### הערך
self-extension עם safety net. אין surprises.

### דוגמה
ZETS למד procedure "summarize email":
1. Experimental — רץ ב-sandbox על 100 דוגמאות
2. 95% accuracy → Tested
3. user מאשר → Human_Approved
4. עוד 30 ימים בלי failures → Core

### מעולה לעומת טוב
- **טוב:** 4 levels סטטיים
- **מעולה:** **probationary period** — Core חוזר ל-Tested אם יש כשלון אחד. דיגרדציה דינמית, לא רק קידום.

---

# 🟢 פער nice-to-have (1)

## #21 כבר כוסה למעלה — אין עוד nice-to-have

(האפקטיבי הוא רק affective state #9 שכבר nice-to-have)

---

# 🎯 סיכום

## כמות וקטגוריות

| קטגוריה | פערים | זמן |
|---|---|---|
| 🔴 קריטיים | 6 | ~6-8 שבועות |
| 🟡 חשובים | 9 | ~5-7 שבועות |
| 🟢 Nice-to-have | 2 | ~1 שבוע |
| **סה"כ פתוחים** | **17** | **~12-16 שבועות** |

## מה צריך **קודם**?

### Phase A — Foundation + Safety (שבוע 1-2)
- #2 Edge Schema
- #20 WASM Sandbox
- #21 TrustLevel
- #12 Regression Suite
- #18 Cache layout (תכנון)

### Phase B — Performance + Compression (שבוע 3-4)
- #18 Cache reordering
- #3 Huffman+Delta

### Phase C — Language Bridge (שבוע 5-7)
- #4 Phi-3-mini
- #15 Learned Ranker
- #5 Fuzzy Hopfield
- #16 NL Realization

### Phase D — Integrity + TMS (שבוע 8-10)
- #22 Parse-to-Graph defense
- #11b TMS deep
- #19 Morphology

### Phase E — Cognitive (שבוע 11-13)
- #6 Global Workspace
- #9 Affective State
- #17 Analogical Transfer
- #14 Planner

### Phase F — Knowledge (שבוע 14-16, ongoing)
- #13 Common-sense enrichment
