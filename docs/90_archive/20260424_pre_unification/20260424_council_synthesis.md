# מועצת החכמים — דיון על המבנה

## נוכחים (רק רלוונטיים לתחום)

**Ted Codd (1923-2003)** — אבי ה-Relational Model, נורמליזציה של נתונים
**George Miller (1920-2012)** — WordNet, מילוני מחשב, הבחנה Word/Sense
**Doug Lenat (1950-2023)** — Cyc, אונטולוגיות ענקיות, הפרדת לוגיקה מידע
**Donald Knuth** — מבני נתונים, עצים, אלגוריתמים
**Marvin Minsky (1927-2016)** — Society of Mind, Frames (מסגרות)
**Tim Berners-Lee** — RDF, Semantic Web, הקונספט של triples

---

## השאלה על השולחן

"האם מבנה אחד של atom+edge יכול להחזיק: מילונים, סינונימים, שפות, משפטים,
מסמכים, פרוצדורות, Make-flows, מדיה, נוסחאות, קוד?"

---

## Ted Codd:
> "אני דוחה את ההצעה כפי שהיא. אתם מערבבים שתי רמות שונות: את ה-**surface form**
> (המילה, האותיות) עם ה-**meaning** (המושג). זה מה שכל מסד נתונים חכם עושה בנפרד.
> 
> First Normal Form אומר: each attribute has one value of one type.
> אצלכם atom_type 'concept' גם שומר אותיות וגם מייצג מושג. זו violation.
> 
> הפתרון הפשוט: הפרידו. Word table וConcept table. רלציה ביניהם."

## George Miller:
> "WordNet פתר את הבעיה הזאת ב-1985. המבנה היה:
> 
>   **Word**     →  אותיות, שפה, הגייה  
>   **Synset**   →  קבוצת מילים עם אותה משמעות (synonym set)  
>   **Concept**  →  המושג המופשט  
> 
> יחסים:
>   word `has_sense` synset  
>   synset `represents` concept  
>   synset `hypernym_of` synset (IS-A)  
>   synset `meronym_of` synset (PART-OF)  
>   synset `antonym_of` synset  
> 
> הסינונים שלכם בעברית (שמחה/עליזות/אושר) — כל אחד הוא word אחר,
> אבל כולם חולקים sense אחד של 'happiness state'. 
> 
> ההצעה שלכם חסרה את שכבת ה-Sense. זה הפער הגדול ביותר."

## Doug Lenat:
> "Cyc בנה 20 שנה את האונטולוגיה. הלקח הכי חשוב: **23 atom_types זה לא מספיק**.
> 
> Cyc הגיע ל-500,000 concepts ו-10,000 yp types של קשרים. 
> מה שאתם מתכננים זה micro-Cyc. אבל 4 bits ל-atom_type = 16 types?
> זה פחות ממה שמסעדה צריכה לתפריט. קחו 8 bits = 256 types לפחות.
> 
> גם: יש הבדל מהותי בין 'concept' (מושג לא-גשמי) ל-'entity' (דבר קונקרטי).
> 'אהבה' היא concept. 'עידן אלדד' היא entity. לא לערבב."

## Donald Knuth:
> "אני רוצה לדבר על העצים ועל הסדרות.
> 
> משפט זה לא sequence. זה **parse tree**. 
> מסמך זה לא sequence. זה **DOM tree**.  
> פרוצדורה זה לא sequence. זה **control flow graph**.
> פונקציה מתמטית זה לא sequence. זה **expression tree**.
> 
> ניסוב sequence לייצג כל אחד מאלה = איבוד מידע.
> 
> הפתרון: atom_type=tree → content מכיל עץ מסודר (parent-children)
> atom_type=dag → content מכיל צמתים עם תנאים/לולאות.
> 
> זה לא הורס את המודל — זה מעשיר אותו. אותו atom, content נקרא אחרת."

## Marvin Minsky:
> "ה-Frame מתוך 'Society of Mind': כל concept הוא frame עם **slots** (תאים).
> 
> Frame 'לימון':
>   slot: צבע   → צהוב (default)
>   slot: טעם   → חמוץ (default)
>   slot: צורה  → אליפטי (default)
>   slot: ripeness → 0.8 (current)
> 
> מה שחסר במודל שלכם: **slots עם defaults**.
> 
> Slots פותרים כמה בעיות ביחד:
>   - משמעות חלקית של המילה (default values)
>   - רמה של יחסים מורכבים (slot 'מורכב מ' → רשימת atoms)
>   - אופציונליות (slots יכולים להיות null)
>   - גרסה (slot עם timestamp מתי עודכן)
> 
> Frames + Inheritance = הבסיס ל-OOP, ל-JSON, ל-KG.
> אתם צריכים גם את זה."

## Tim Berners-Lee:
> "RDF triples: **Subject - Predicate - Object**. 
> זה פשוט, אוניברסלי, עובד 25 שנה.
> 
> אבל יש חולשה של RDF: אי אפשר לתאר קשר בין קשר. 
> 'עידן אמר שלימון הוא צהוב' — איך זה נכנס?
> 
> הפתרון: **Named Graphs** (quads) או **reification**.
> שתי הביקורות שלכם (GPT-5.4 ו-Gemini) הצביעו על reification כפתרון.
> זה אותו פתרון. תלמדו מ-RDF, עובד.
> 
> אבל אל תיצמדו ל-triples בלבד. RDF סובל מזה שהוא **flat**.
> המודל ההיברידי שהצעתם (atom יכול להכיל ANY) יותר עשיר. תשמרו על זה."

---

## סיכום הקונצנזוס במועצה

**שבירות מרכזיות (הסכמה מלאה):**

1. ❌ **u64 encoding כקידוד-לכל-מילה** — נכשל במולטי-לשונית, לא מספיק bits
2. ❌ **Word=Concept confusion** — חייבים להפריד word / sense / concept  
3. ❌ **16 atom_types** — חייב 256 (8 bits)
4. ❌ **21 edge_types** — חסרים linguistic (subject/object/modifier/has_part), חסר sense_relation
5. ❌ **Edges בלי reification** — אי אפשר לתאר יחס של יחס, nuance, provenance
6. ❌ **Sequence במקום tree** — משפטים, מסמכים, פרוצדורות, נוסחאות צריכים עצים
7. ❌ **אין slots/defaults** (Minsky) — מושגים צריכים שדות אופציונליים
8. ❌ **Media וגדולים-אחרים** — חייב blob store חיצוני + vector store
9. ❌ **Control flow** (תנאים, לולאות, iteration counts) — צריך DAG עם metadata

**שריד מהותי (חיובי):**

✓ **Hybrid atom storage** (u64 fast-path + dynamic large-path) — הרעיון הנכון
✓ **atom_type כ-dispatch לתוכן** — הגישה הנכונה
✓ **Edge-as-6-bytes-hot** — אם reified-cases עוברים ל-cold path
✓ **CSR + mmap** — הנדסית נכון
✓ **3 context axes** (world/time/self) — בסיס טוב

---

## הארכיטקטורה המתוקנת (מה שצריך לבנות בפועל)

### שכבות (Layers) — 5 שכבות, לא מבנה אחד

```
LAYER 5: Sense Graph (sense-to-concept mapping)
         ──────────────────────────────────────────
LAYER 4: Concept Graph (abstract meanings, language-agnostic)
         ──────────────────────────────────────────
LAYER 3: Structure Graph (trees, DAGs, sequences, frames)
         ──────────────────────────────────────────
LAYER 2: Atom Graph (basic edges between atoms)
         ──────────────────────────────────────────
LAYER 1: Word Layer (lexical forms, per language)
```

### Atom hierarchy (8-bit atom_type = 256 types)

**Family 0x0x — Lexical:**
  0x00 WordForm   — מילה ספציפית בשפה ספציפית  
  0x01 Lemma      — צורה-בסיסית (canonical)  
  0x02 Phoneme    — הגייה  
  0x03 Morpheme   — יחידת משמעות קטנה  

**Family 0x1x — Sense/Concept:**
  0x10 Sense        — משמעות ספציפית של מילה  
  0x11 Concept      — מושג מופשט (language-agnostic)  
  0x12 Entity       — ישות קונקרטית (אדם, מקום, דבר יחידני)  
  0x13 Category     — קטגוריה (is-a root)  
  0x14 Property     — תכונה (צבע, גובה)  
  0x15 Value        — ערך ספציפי של תכונה  

**Family 0x2x — Structure (עצים/גרפים):**
  0x20 Sequence     — רצף מסודר  
  0x21 Set          — קבוצה לא-מסודרת  
  0x22 Tree         — עץ היררכי (למסמך/משפט)  
  0x23 DAG          — גרף מכוון לא-מעגלי (workflow)  
  0x24 Matrix       — מטריצה 2D  
  0x25 Frame        — Minsky-style frame עם slots  

**Family 0x3x — Action/Process:**
  0x30 Event        — אירוע (מה, מי, מתי)  
  0x31 Procedure    — פרוצדורה (עץ steps+conditions+loops)  
  0x32 Rule         — IF-THEN  
  0x33 Function     — פונקציה  
  0x34 Workflow     — Make-style (DAG + iteration counts)  

**Family 0x4x — Language/Math:**
  0x40 Sentence     — משפט (parse tree)  
  0x41 Paragraph    — פסקה (tree of sentences)  
  0x42 Document     — מסמך (tree of paragraphs)  
  0x43 Formula      — נוסחה (expression tree)  
  0x44 Code         — קוד (AST)  

**Family 0x5x — Media/Data:**
  0x50 MediaRef     — URI + metadata לblob חיצוני  
  0x51 Vector       — embedding/feature vector  
  0x52 Timeline     — spans עם timecodes  

**Family 0xFx — Meta:**
  0xF0 Relation     — reified edge (when edge needs properties)  
  0xF1 Annotation   — annotation על אובייקט אחר  
  0xF2 Provenance   — מקור/מחבר/תאריך  
  0xF3 Context      — הקשר שיחה/זמן/זהות  

יוצאים 64 types עם מקום ל-192 מילואים לשנים הבאות.

### Edge types — 128 types (7 bits), קובצו:

**Linguistic (אלה שלא היו):**
  subject_of, object_of, modifies, determines, quantifies, negates,
  tense_of, aspect_of, mood_of  

**Structural (אלה שלא היו):**
  has_part, part_of, contains, positioned_at, precedes, follows  

**Lexical (חדשים):**
  word_of, lemma_of, form_of, morpheme_of, pronounced_as,
  has_sense, near_synonym_of, broader_than, narrower_than  

**Sense-level:**
  sense_of, denotes, connotes, metaphor_of, register_of  

**Existing 21 (מומרים מ-5bit ל-7bit):**
  visual_color, taste, smell...
  use_culinary, cause_effect, ingredient_of...
  analogy_similar, symbolic_cultural, emotional_valence...

Plus antonym scale positioning.

### Reification — עד איך זה עובד בפועל

Edge רגילה (95% מהזמן) = 6 bytes (כמו קודם).

Edge עם הערות/provenance/nuance = reified:
  1. Create Relation atom (type 0xF0)
  2. Original "edge" הופכת לשני edges: 
     (A → relation_atom) + (relation_atom → B)
  3. ל-relation_atom עצמו יש edges עם provenance, nuance, confidence

זה מה שהגרף שם עלות של +3 edges לקשרים עשירים (1% מהזמן).

---

## תשובה לעידן

**"האם זה בעצם גם המילון?"**

**כן, אבל רק אחרי התיקונים הנ"ל.** בלי השכבות Word/Sense/Concept, זה לא מילון — זה רק אסוציאציות שטוחות.

**"יכול להיות שצריך מבנה כמו מערך, מסלולים עם הסבר, מטריצה, עץ עם מספר חזרות?"**

**כן, בהחלט. זה בדיוק מה שחסר במודל הנוכחי.**

- **מערך** = atom_type 0x20 Sequence
- **מסלול עם הסבר** = atom_type 0xF0 Relation (reified edge)  
- **מטריצה** = atom_type 0x24 Matrix
- **עץ עם מספר חזרות** = atom_type 0x34 Workflow (DAG עם iteration_count במטא של כל node)

---

## ציון סופי של הקונספט הנוכחי: 4/10

- רעיון יסוד טוב (atom_type-dispatched content)
- ביצוע חסר בשכבות מהותיות (Word/Sense/Concept distinction)
- Scale של 16 types הוא toy — לא מערכת
- u64 encoding הוא pessimization שצריך לזרוק
- החסרונות ניתנים לתיקון — לא צריך לזרוק את הכל
