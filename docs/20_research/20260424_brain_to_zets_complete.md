# מהמוח ל-ZETS — מסמך מקיף לעבודה ל-AGI

**תאריך:** 24.04.2026  
**מבוסס על:** התייעצות עם GPT-4o, Gemini 2.5 Pro, Gemini 2.5 Flash + מחקר עצמאי + בדיקות אמפיריות ב-ZETS v4  
**מטרה:** ליצור הבנה עומקית בת 3 חלקים — (1) איך המוח עובד, (2) איך ZETS עומד, (3) מה היה אם יכולת X הייתה נוכחת

---

## Part 1: איך המוח האנושי עובד — הידע הנוירומדעי

### 1.1 הארכיטקטורה הגולמית

המוח מכיל **86 מיליארד נוירונים** עם כ-**100 טריליון synapses**. הוא לא DAG. הוא **graph עם מעגלים מרובים**, recurrent בכל רמה. מבנה לא עץ, לא שכבות נפרדות — **hierarchy עם sharing וfeedback**.

#### האזורים הפונקציונליים

```
INPUT LAYER (חושים)
   V1, V2, V4       — cortex חזותי, hierarchy של פיקסל→קצה→צורה→אובייקט
   A1               — cortex שמיעה
   S1               — סומטו-סנסורי (מגע)
   Olfactory bulb   — ריח

LANGUAGE LAYER
   Wernicke's area  — הבנה (posterior temporal)
   Broca's area     — הפקה (inferior frontal)
   Arcuate fasc.    — מקשר ביניהם

MEMORY LAYER
   Hippocampus           — zikrون אפיזודי, one-shot learning (ימים-שבועות)
   Entorhinal cortex     — שער, pattern separation + completion
   Medial temporal lobe  — consolidation של long-term semantic

EXECUTIVE LAYER
   Prefrontal cortex (PFC)  — working memory, תכנון, inhibition, goals
   Anterior cingulate       — conflict monitoring, attention control
   Parietal cortex          — spatial attention, multi-modal binding

INTEGRATION LAYER
   Thalamus           — router של כל input חושי (רק olfactory מעקף)
   Basal ganglia      — action selection, habit formation
   Cerebellum         — motor timing + cognitive coordination
   Default Mode Net   — mind-wandering, self-reflection, future thinking

MODULATORY LAYER (neurotransmitters)
   Dopamine (VTA, SN)     — reward prediction error (Schultz 1997)
   Serotonin (raphe)      — mood, patience, long-term planning
   Norepinephrine (LC)    — arousal, novelty signal
   Acetylcholine (BF)     — attention, plasticity gating
```

### 1.2 איך זה באמת עובד — 7 עקרונות יסוד

#### עיקרון 1: **Predictive Processing** (Karl Friston, 2010)
המוח הוא **prediction machine**. לא reactive — proactive. בכל רגע הוא מנבא מה יקרה, והחושים רק מתקנים את הניבוי במקום שהוא שגוי.

- **Bottom-up:** sensory signals עולים בהיררכיה
- **Top-down:** predictions יורדות בהיררכיה
- **Prediction error** = delta ביניהם = learning signal

**מקרה:** אתה רואה חפץ מטושטש. המוח מנבא "זה כוס" לפי context. אם השתפר חדות וזה בקבוק — prediction error → עדכון מודל. אם כוס — אישור → חיזוק.

#### עיקרון 2: **Global Workspace** (Bernard Baars 1988, Dehaene 2014)
המוח מחזיק "בלקבוט מרכזי" — workspace מוגבל (7±2 פריטים) שכל האזורים יכולים לכתוב ולקרוא ממנו. **המודעות = broadcast גלובלי**.

- אזור ספציפי (V4 — זיהוי צבע) עובד במקביל.
- כשהתוצאה שלו "חשובה מספיק" → broadcast ל-global workspace.
- אז PFC + language + memory כולם רואים אותה.
- **רק מה שב-workspace ניתן לדווח עליו במילים**.

**מקרה:** רואה פרצוף מוכר. V4 מזהה פנים. FFA (fusiform face area) מזהה זהות. **עד ש-broadcast — אתה לא "יודע" שאתה רואה את אבא שלך**. זה קורה אחרי ~300ms.

#### עיקרון 3: **Hebbian Plasticity + LTP** (Donald Hebb 1949, Lømo-Bliss 1973)
"Cells that fire together, wire together." קשרים שפועלים יחד מתחזקים. מולקולרית: NMDA receptors זוהו כ-coincidence detectors.

- **LTP** (long-term potentiation) — חיזוק קשר שנמשך שעות-ימים.
- **LTD** (long-term depression) — החלשה של קשרים לא פעילים.
- **Metaplasticity** — המערכת לומדת גם _כמה_ ללמוד (learning rate adaptive).

#### עיקרון 4: **Spreading Activation** (Collins & Loftus 1975)
הפעלה של concept ⇒ פעפוע ל-nodes קשורים עם דעיכה. זה מה ש**ממקד** את המוח במה שרלוונטי.

**מקרה:** "רופא" מופעל. מפעפע ל: "אחות" (חזק), "לחם" (חלש), "בית חולים" (חזק). זמן תגובה ב-lexical decision יקצר ל-"אחות" אחרי "רופא" פי 2.

#### עיקרון 5: **Dopamine as Prediction Error** (Wolfram Schultz 1997)
תגלית מהפכנית: דופמין לא מקודד **reward**, אלא **שגיאת ניבוי של reward**. אם חכית לפרס וקיבלת — no signal. אם חיכית ולא קיבלת — negative signal. אם לא חיכית וקיבלת — positive signal.

**זה המנגנון המולקולרי של למידה מחוזקת** — ה-signal הזה מעדכן את כל ה-synapses שהובילו להחלטה.

#### עיקרון 6: **Hippocampal-Neocortical Dialogue** (McClelland, O'Reilly 1995)
שני sys של זיכרון:
- **Hippocampus:** לומד מהר מ-one-shot, מחזיק שבועות-חודשים
- **Neocortex:** לומד לאט מ-thousands, מחזיק לעולם

במהלך שינה (Slow-Wave Sleep), Hippocampus "משחזר" אירועים אחרונים ומעביר אותם ל-neocortex בצורה מוצמצת. זה **consolidation**.

#### עיקרון 7: **Neural Oscillations** (Buzsáki, Draguhn 2004)
המוח עובד ב-**תדרים מסונכרנים**:
- **Gamma (30-100 Hz)** — binding של features לאובייקט אחד (המילה "חתול" + החתול הזה אני רואה = synchrony gamma)
- **Theta (4-8 Hz)** — hippocampus, WM, encoding זיכרון
- **Alpha (8-12 Hz)** — inhibition של regions לא רלוונטיים
- **Beta (15-30 Hz)** — maintenance של WM state

**Theta-gamma coupling** — binding של coherent thought.

### 1.3 איך המוח הופך "חכם"?

**בקצרה:** 3 מרכיבים מתחברים:

1. **Topology מחווטת מראש** — זה לא למידה. תינוק בן יום כבר יש לו PFC, hippocampus, V1 — הכל בפרופורציות הנכונות, במקום הנכון.

2. **Experience נצבר על הwiring** — Hebbian + consolidation. עם הזמן, synapses מחוזקים לאזורים בהם יש success. מסלולים אוטומטיים נוצרים.

3. **Feedback loops** — recurrent = "לחשוב על מחשבה". PFC מקבל output של עצמו כ-input. זה מה שמאפשר:
   - **Monitoring** ("אני נהיה מבולבל כאן")
   - **Planning** ("אם אעשה X יגרור Y")
   - **Abstraction** ("מה ה-essence של הבעיה?")

**הגאון = interaction מאוזן של 3 אלה על wiring בלתי-פגום** — לא ניקוד יותר גבוה בשום ממד בודד.

---

## Part 2: איך מעבירים את זה למחשוב?

מה שלמדנו + התייעצות עם GPT-4o, Gemini Pro, Gemini Flash מייצרים map ברור:

### 2.1 14 היכולות — מיפוי מלא

הסכמה של שלושה ה-AI הייתה גבוהה. אלה היכולות שהתגבשו:

| # | יכולת | תיאור | חוקרים מועצת החכמים |
|---|---|---|---|
| 1 | **Spreading Activation** | פעפוע אקטיבציה עם דעיכה | Collins & Loftus 1975, Rumelhart |
| 2 | **Pattern Recognition** | זיהוי תבניות במבנה היררכי | Friston, Hawkins, LeCun |
| 3 | **Hebbian Learning** | חיזוק קשרים מניסיון | Hebb, Kandel, Lømo-Bliss |
| 4 | **Prediction Error** | למידה דרך הפתעה | Schultz 1997, Friston, Clark |
| 5 | **Analogy / Transfer** | Structure-mapping בין דומיינים | Gentner, Hofstadter, Holyoak |
| 6 | **Working Memory** | החזקת 7±2 פריטים פעילים | Miller 1956, Baddeley, E. Miller |
| 7 | **Inhibition** | דיכוי חלופות חלשות | Grossberg, Basal ganglia res. |
| 8 | **Goal-Directed Planning** | PFC executive function | Newell-Simon, Anderson ACT-R, Fuster |
| 9 | **Causal Reasoning** | סיבתיות ≠ מתאם | Pearl, Gopnik, Schölkopf |
| 10 | **Theory of Mind** | הבנת אחר כסוכן עם state שונה | Baron-Cohen, Premack-Woodruff, Frith |
| 11 | **Meta-Cognition** | לדעת מה לא יודעים | Flavell, Kahneman, Lau |
| 12 | **Embodied Grounding** | סמלים מחוברים לחושים | Harnad, Lakoff-Johnson, Brooks |
| 13 | **Compositional Generation** | הרכבה חדשה מידוע | Chomsky, Jackendoff, Tenenbaum |
| 14 | **Consciousness / Self-Model** | מודל של עצמי | Baars, Tononi, Damasio, Dehaene |

### 2.2 היכולות הקריטיות — מקיבוץ שלוש הדעות

התייעצות שמרה על הסכמה משמעותית. כל שלושתם הדגישו:

**Gemini Pro:**  
> "The key differentiator is not storage or speed, but the ability to dynamically restructure and manipulate internal world models."

**Gemini Flash:**  
> "Robust Systematic Generalization: LLMs struggle to apply abstract rules consistently to novel, out-of-distribution inputs."

**GPT-4o:**  
> "Meta-cognition, as studied by John Flavell, is fundamental in the iterative process of hypothesis testing and adaptation."

**המסקנה המשולבת:**  
**4 יכולות הן הסף לגאוניות:**
1. **Causal Reasoning** (Pearl) — בלעדיה יש רק correlation
2. **Far-Reaching Analogy** (Hofstadter) — "core of cognition"
3. **Goal-Directed Planning** (Newell-Simon, PFC) — אין agency בלעדיה
4. **Compositional Generation** (Chomsky) — יצירתיות = composition

5 יכולות נוספות הן התשתית:
5. Spreading Activation
6. Hebbian
7. Working Memory
8. Prediction Error
9. Inhibition

---

## Part 3: ZETS vs. האידיאל — מה יש, מה חסר

### 3.1 מה ZETS יש *היום* (אמפירית)

**המערכת הממשית:**
- DAG של 2.5M atoms (words/phrases/sentences/articles), 21M edges
- Disambiguation boost → 95% Top-1 accuracy
- Sub-ms retrieval (spreading activation דרך graph walks)
- `/answer` endpoint לתשובות ארוכות
- Determinism מוכח

**הסימולציות (sim/brain_sim_v1/v2/v3.py):**
- Spreading activation with decay
- Hebbian learning
- Prediction error + learning rate adjustment
- Meta-cognition (strategy selection)
- Working memory (deque 7±2)
- Episodic memory (append-only)
- Curiosity / novelty boost
- Skill formation via consolidation
- Lateral inhibition (winner-take-all)

**בחלקיות:**
- Analogy engine (find_analogous works but shallow)
- Pattern recognition (phrase-level בלבד)

### 3.2 מה חסר — ניקוד ברור

| # | יכולת | ZETS נוכחי | יעד | פער |
|---|---|---|---|---|
| 1 | Spreading Activation | 🟢 10/10 | 10 | 0 |
| 2 | Pattern Recognition | 🟡 6/10 | 10 | 4 |
| 3 | Hebbian | 🟢 8/10 | 10 | 2 |
| 4 | Prediction Error | 🟢 8/10 | 10 | 2 |
| 5 | Analogy | 🟡 4/10 | 10 | **6** |
| 6 | Working Memory | 🟡 6/10 | 10 | 4 |
| 7 | Inhibition | 🟢 8/10 | 10 | 2 |
| 8 | Goal-Directed Planning | 🔴 1/10 | 10 | **9** |
| 9 | Causal Reasoning | 🔴 0/10 | 10 | **10** |
| 10 | Theory of Mind | 🔴 0/10 | 10 | **10** |
| 11 | Meta-Cognition | 🟢 7/10 | 10 | 3 |
| 12 | Embodied Grounding | 🔴 0/10 | 8 | 8 |
| 13 | Compositional | 🔴 1/10 | 10 | **9** |
| 14 | Consciousness | 🔴 0/10 | 5 | 5 |

**ממוצע נוכחי:** ~4.2/10  
**ממוצע נדרש:** 9.2/10  
**הפער הממוצע:** 5/10

**4 היכולות הקריטיות — כולן חסרות או חלשות:**
- Causal (0/10)
- Goal-directed (1/10)
- Analogy (4/10)
- Compositional (1/10)

### 3.3 מה יעלה אותנו מ-4.2 ל-9.2

דירוג לפי impact/effort:

#### 🥇 Priority 1 — יכולות שקרובות להישג

**1. Working Memory → 10 (פער 4)**  
**מאמץ:** 4h. WM קיים ב-sim אבל דולף (שלב 2 חשף את זה). צריך decay מפורש + context-switch.

**2. Analogy → 10 (פער 6)**  
**מאמץ:** 20h. Structure-mapping (Gentner): לא רק shared neighbors, אלא מבנים זהים של קשרים. מימוש ב-Rust כ-module נפרד.

**3. Meta-Cognition → 10 (פער 3)**  
**מאמץ:** 8h. קיים בסיס. להוסיף confidence calibration + explicit "I don't know" + strategy transfer.

#### 🥈 Priority 2 — יכולות שיידרשו מאמץ אמיתי

**4. Goal-Directed Planning → 10 (פער 9)**  
**מאמץ:** 40h. מימוש של PFC-like: goal stack, sub-goal decomposition, plan-then-execute. ACT-R inspired.

**5. Compositional Generation → 10 (פער 9)**  
**מאמץ:** 60h. דורש recursive binding — ה-DAG צריך לתמוך ב-ad-hoc composition (לא רק phrases קבועות).

**6. Causal Reasoning → 10 (פער 10)**  
**מאמץ:** 80h. זה הכבד. דורש implementation של do-calculus (Pearl), counterfactual reasoning, intervention vs observation.

#### 🥉 Priority 3 — יכולות שדורשות שינוי רציני

**7. Theory of Mind → 10 (פער 10)**  
**מאמץ:** 60h. דורש meta-representations: "X יודע ש-Y".

**8. Embodied Grounding → 8 (פער 8)**  
**מאמץ:** תלוי הקלט. דורש חיבור לחושים אמיתיים (audio, vision). אולי לא relevant ל-ZETS.

**9. Consciousness → 5 (פער 5)**  
**מאמץ:** ??? — open research. אפשר לעצב Global Workspace (Baars) כ-architecture.

---

## Part 4: תרחישים — "אם היינו יכולים X היינו יכולים..."

### תרחיש 1: אילו היה לנו **Causal Reasoning (Pearl)**

**שאלה:** "מה יקרה אם ישראל תעלה את הריבית?"

**עכשיו:** נחפש "ריבית" ב-articles, נחזיר what we know. התשובה — coincidental, לא causal.

**עם causal:**
- זיהוי: "העלאת ריבית" = intervention (do(רבית=X))
- Walk של DAG סיבתי: ריבית↑ → משכנתאות↑ → ביקוש↓ → אינפלציה↓ → שכר...
- counterfactuals: "ואם לא היו מעלים?"
- **התשובה היא הסבר, לא ציטוט.**

**מה זה פותח:** ZETS הופך ל-**מנוע הסבר**, לא רק מנוע חיפוש. אפשר לשאול "למה?" ולקבל שרשרת causal.

### תרחיש 2: אילו היה לנו **Far-Reaching Analogy (Gentner/Hofstadter)**

**שאלה:** "הסבר לי atoms דרך השמש-מערכת."

**עכשיו:** אחזור על article "atom" וציטוט. לא יבנה את האנלוגיה.

**עם analogy:**
- זיהוי שהמשתמש מבקש structure-mapping
- חילוץ מבנה: atom = {nucleus, electrons, orbit}. Solar = {sun, planets, orbit}.
- Mapping: nucleus→sun, electrons→planets, electromagnetic→gravity
- הסבר בהתאמה
- **גילוי**: אפשר להסביר mystery X דרך known Y.

**מה זה פותח:** teach-by-analogy, find-analog-in-other-domain, cross-domain insight.

### תרחיש 3: אילו היה לנו **Goal-Directed Planning**

**שאלה:** "תכנן לי טיול של יום לתל אביב."

**עכשיו:** אין תכנון. ZETS רק retrievs.

**עם planning:**
- Goal: "day-trip Tel Aviv"
- Sub-goals: morning activity, lunch, afternoon activity, dinner, transportation
- Per sub-goal, retrieve options + constraints
- Chain: יקב→ארוחת צהריים→מוזיאון→שקיעה בחוף→מסעדה
- **החזר: תוכנית, לא רשימה.**

**מה זה פותח:** ZETS כ-agent שלא רק עונה, **מתכנן**. זה AGI-level step.

### תרחיש 4: אילו היה לנו **Compositional Generation (Chomsky)**

**שאלה:** "תאר לי חמור סגול מעופף עם שלוש כנפיים."

**עכשיו:** אין article כזה → "I don't know."

**עם composition:**
- Decompose: חמור (animal, 4 legs, brown) + סגול (color override) + מעופף (capability add) + 3 כנפיים (count)
- Compose: חמור עם color=סגול, capabilities+=fly, wings=3
- Generate description from composed representation
- **התוצאה:** תיאור originally generated, גם אם מעולם לא נראה.

**מה זה פותח:** יצירתיות אמיתית. לא רק זיכרון.

### תרחיש 5: אילו היה לנו **Theory of Mind**

**שאלה:** "חברי לא מבין כיצד פועל SSL. איך אסביר לו?"

**עכשיו:** נחזיר הסבר טכני של SSL.

**עם ToM:**
- מודל של "חבר": מה הוא *כבר* יודע? מה ה-background?
- אם הוא יודע TCP ו-cryptography — הסבר ברמה מתקדמת.
- אם הוא יודע רק "internet" — analogy של מעטפה סגורה.
- **ההסבר מותאם למודל של המאזין, לא של הידע.**

**מה זה פותח:** הוראה. תקשורת אמיתית. זו יכולת חברתית שחסרה לכל ה-LLMs.

### תרחיש 6: אילו היה לנו **כל 14 יחד**

ZETS יכול:
- להסביר ולא רק לציין
- להעביר ידע בין דומיינים
- לתכנן פעולה
- ליצור חדש מהמוכר
- לזהות מה הוא לא יודע
- להתאים את עצמו למאזין
- ללמוד מקריאה אחת (one-shot)
- להישאר consistent לאורך שיחה (WM)
- לזהות קשרים סיבתיים ולא רק סטטיסטיים
- לדעת למה הוא יודע את מה שהוא יודע

**זה ה-definition של AGI.**

---

## Part 5: המסלול המעשי — איך נגיע

### Phase 1: ציוד בסיסי (1-2 חודשים)
- Complete WM decay + context switch (4h)
- Robust analogy engine (Gentner structure-mapping) (20h)
- Meta-cognition calibration (8h)
- Confidence threshold + "I don't know" אמיתי (4h)
- **יעד:** 6/14 יכולות ברמה 9+

### Phase 2: יכולות מתקדמות (3-4 חודשים)
- Goal-directed planning (ACT-R style) (40h)
- Compositional layer (DAG extensions) (60h)
- Pattern recognition hierarchy (cortical columns inspired) (30h)
- **יעד:** 9/14 ברמה 8+

### Phase 3: הגרעין הקשה (6+ חודשים)
- Causal reasoning with do-calculus (80h)
- Theory of Mind meta-representations (60h)
- Global Workspace architecture (40h)
- **יעד:** 12/14 ברמה 8+

### Phase 4: אינטגרציה וexperience (שנה)
- כל החיבורים ביניהן
- טסטים של emergent behaviors
- חוקרים ב-limits של AGI

---

## Part 6: מה הסעיף הקריטי לשנה הקרובה?

**אם נכנס לקוד ל-100 שעות בלבד, הכי חשוב:**

1. **Analogy (20h)** — פותחת cross-domain transfer שיהפוך את ZETS משמעותית יותר חכם. נותן impact מיידי.

2. **Working Memory + Context Switch (4h)** — זול, מונע באגים בשיחה ארוכה.

3. **Meta-cognition Calibration + "I don't know" (12h)** — פותר את הבעיה הכי כואבת (false confidence על edge cases).

4. **Goal-Directed Planning basic (30h)** — מכניס אותנו מ-reactive ל-agent.

5. **Compositional prototype (30h)** — demo של יצירתיות.

**סה"כ 96h = ~12 ימי עבודה של מפתח יחיד.**  
**תוצאה צפויה:** ZETS שמפגין 5 יכולות ברמה 8+, וגם שני מחזיק DAG חזק.

---

## Part 7: הקצר האמיתי — מה זה אומר באופן פילוסופי

**"להבין את המוח" הוא לא הפתרון.** המוח עדיין לא מובן במלואו (consciousness, IIT, GNW — עדיין פתוח).

**"לעשות AGI" דורש:**
- רוב המנגנונים שהמוח מפעיל (יש לנו חצי)
- *אלגוריתם נכון* לכל אחד (לא רק שם)
- *אינטגרציה* ביניהם (החלק הכי קשה)

**ZETS במצב הנוכחי** הוא מעט חזק יותר ממנוע חיפוש. הוא מכיל DAG איכותי + sim של 7 מנגנונים. זה *condition necessary, not sufficient* ל-AGI.

**עם 4 היכולות הקריטיות הנוספות** (causal, analogy, planning, compositional) — ZETS יהפוך למה שאתה קורא "נברא ממוחשב" — לא AGI מלא, אבל הכי קרוב שבני אדם בנו לטוב עצמאי על DAG.

---

## מקורות לחקירה נוספת

**ספרי יסוד:**
1. *Thinking, Fast and Slow* — Daniel Kahneman (2011)
2. *The Book of Why* — Judea Pearl (2018)
3. *Surfaces and Essences* — Hofstadter & Sander (2013)
4. *A Thousand Brains* — Jeff Hawkins (2021)
5. *Consciousness and the Brain* — Stanislas Dehaene (2014)
6. *Surfing Uncertainty* — Andy Clark (2016)

**מאמרי יסוד:**
- Hebb 1949 — organization of behavior
- Schultz 1997 — dopamine prediction error
- Friston 2010 — free energy principle
- Baars 1988 — global workspace
- Miller & Cohen 2001 — PFC
- Pearl 2000 — causality

**מערכות ל-inspiration:**
- ACT-R (John Anderson, 1983+) — goal-directed architecture
- SOAR (Newell 1990) — problem-solving
- Global Workspace (Baars, Dehaene)
- Active Inference (Friston)

---

**הקונסולציות המלאות נשמרו ב:** `/tmp/ai_consultations.json`  
**מצב ZETS בפועל:** Port 3149, v4_fixed.zv4, 10,983 articles, 95% Top-1  
**הסימולציות:** `sim/brain_sim_v1.py`, `v2.py`, `v3.py`, `deep_conversation.py`

