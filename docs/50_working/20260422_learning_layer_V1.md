# ZETS — Learning Layer V1
## Cognitive Training vs Asserted Knowledge

**תאריך:** 22.04.2026 (late)  
**מצב:** 366/366 tests, `63213d6` on main  
**גרסה:** V1 — אסטרטגיה לפני יישום  
**מקור:** עידן זיהה שה-ZETS הנוכחי לא מבחין בין "עובדה" לבין "דפוס שפה שאני רוצה ללמוד ממנו".

---

## 🔍 הבעיה שעידן זיהה

ZETS היום שומר כל atom כ-"asserted fact":
- "Paris is capital of France" — נכון
- "I feel sad" (מ-Reddit post) — נשמר באותה צורה, אבל זו **לא** אמת אוניברסלית, זה דפוס דיבור של אדם אחד

הבעיה הרחבה יותר:
1. **Copyright:** אסור לנו לאחסן תמונה של לברדור מ-Getty Images, אבל כן נוצצה לנו היכולת לזהות כלבים.
2. **Storage:** מיליארדי פוסטים = גרף אינסופי. רוב הטקסט הוא **דוגמה** של דפוס, לא עובדה חדשה.
3. **Semantic integrity:** אם ZETS במצב Precision מחזיר "users feel sad" כfact, זה הזיה. זה observation מ-sample.

**המקבילה האנושית:**
> "אני לא מחזיק תמונות של לברדור וספרים אבל כן את הידע  
> ואת היכולת לחזור מילה במילה או לצייר את הכלב בדומה"

אנחנו חיים עם הפרדה טבעית בין:
- **Episodic memory** — "הלברדור הזה שראיתי בפארק אתמול" (פרט)
- **Semantic memory** — "איך לברדור נראה בדרך כלל" (prototype)
- **Procedural memory** — "איך לצייר כלב" (skill)

זה בדיוק מה שZETS חסר.

---

## 🧠 Triangulation — 3 AI Opinions (Gemini Pro, Groq 70b, Gemini Flash)

שאלתי את שלושתם על האדם ארכיטקטוני: 4 graphs נפרדים או 1 graph + provenance tags?  
**כולם הסכימו: 1 graph + provenance.** אבל נתנו 5 תובנות שמשנות הכל:

### תובנה #1 (Gemini Pro) — provenance = feature, not bug
> "The 'con' you list — that the semantics of 'truth' depends on provenance tag — is not a con.  
> **It is the entire point.** This is how human knowledge works."

`Asserted` = textbook. `Observed` = anecdote. `Learned` = generalization. אלה לא bugs, אלה **היעד**.

### תובנה #2 (Gemini Pro) — Tulving's episodic-semantic split
> "`Observed` edges on raw atoms are **episodic memory** (that specific Reddit post).  
> `Learned` edges on `Prototype` atoms are **semantic memory** (the general pattern of sadness)."

ZETS מקבל grounding פסיכולוגי ישיר — זה לא rebuild של idea חדשה, זה יישום של Tulving 1972.

### תובנה #3 (Gemini Flash) — **אל תמחק raws, תארכב**
> "Deleting raw atoms after clustering is a **massive mistake**.  
> It makes debugging, re-evaluation, and fine-tuning impossible."

התיקון: שמור 2-3 **canonical exemplars** מקושרים ל-prototype. רק "outliers" נמחקים לארכיון.

### תובנה #4 (Groq + Gemini Flash) — Bounded drift
> "If a new input's feature vector is too dissimilar to any existing prototype  
> (beyond a threshold), create a **new** prototype, rather than forcing it into an existing one."

זה מונע "prototype contamination" — כשdata של "כעס" זולג לprototype של "עצב".

### תובנה #5 (Gemini Pro) — הכשל הגדול: Concept Drift
> "Your biggest challenge will be managing the boundary and stability  
> between your crisp symbolic world and your fuzzy, learned one."

אם `Asserted: sadness → leads_to → crying` קיים, אבל `Prototype:sadness` drifted ל-teen-angst, הכלל נשבר בשקט. צריך versioning של prototypes.

---

## 🏗️ הארכיטקטורה הסופית

**אחד graph. 5 atom kinds חדשים. provenance tag. זה הכל.**

### הרחבות ל-`AtomKind`

```rust
pub enum AtomKind {
    // Existing 9 (don't touch):
    Concept, Text, ImageFrame, AudioChunk, PoseVector,
    Template, Delta, Composition, Relation,
    
    // NEW — Learning Layer:
    Pattern,        // template with slots: "I just want to [VERB]"
    Prototype,      // centroid atom — the Platonic ideal of a cluster
    FeatureVector,  // raw 512-dim embedding (CLIP/MiniLM/etc)
    Exemplar,       // canonical example kept per prototype (2-3)
    ArchivedRaw,    // reference to gzipped archive of deleted raw
}
```

### Provenance על כל edge

```rust
pub enum Provenance {
    Asserted,     // "X is Y" — stated as truth (Wikipedia, textbook)
    Observed,     // "user said X" — happened once in training data
    Learned,      // "pattern X correlates with emotion Y" — from N observations
    Hypothesis,   // proposed by dreaming, confidence not yet verified
}

pub struct AtomEdge {
    // existing fields...
    pub provenance: Provenance,  // NEW
    pub confidence: u8,          // NEW: 0-255
}
```

### מה זה פותח

**Query 1:** `smart_walk(mode=Precision)` מסנן רק `Asserted` + `Learned (confidence ≥ 200)`.  
**Query 2:** `smart_walk(mode=Divergent)` כולל `Hypothesis` + `Observed`.  
**Query 3:** `dreaming` יוצר רק `Hypothesis` edges.  
**Query 4:** `explain(atom)` מציג: "הגעתי למסקנה כי X is_a Y (Asserted, Wikipedia) + X expresses Z (Learned from 47 dialogues, confidence 215)".

---

## 🔬 Distillation Process — איך prototypes נוצרים

**לא continuous. threshold-based per domain + nightly sweep.**

### אלגוריתם

```
For each DOMAIN (emotion, dialogue_pattern, visual_concept, ...):
  1. Collect Observed atoms in this domain from last N days
  2. If count < THRESHOLD (e.g. 50), skip
  3. Run clustering (k-means or DBSCAN on feature vectors)
  4. For each cluster:
     a. Compute centroid -> new Prototype atom
     b. Pick 2-3 most-central samples -> link as Exemplars
     c. Archive remaining Observed atoms -> ArchivedRaw atom
     d. Create Learned edges from Prototype to related concepts
  5. Mark domain as "distilled_at_timestamp"
```

### Bounded drift — מתי prototype חדש vs קיים

```
For a new Observed atom O with feature vector f:
  closest_proto = argmin(cosine_distance(f, proto.centroid))
  if cosine_distance(f, closest_proto.centroid) < DRIFT_THRESHOLD:
    add O to closest_proto's pending cluster
  else:
    mark O as "pending_new_prototype"
```

קבוע `DRIFT_THRESHOLD`: התחלה ב-0.3 (cosine). ניתן לtune לפי domain.

### Deletion criteria (מה כן ארכוב, מה לא)

**Archive (gzipped):** Observed atoms שהייצגו ≥3 פעמים בcluster של prototype stable (נפח ≥ 100, drift < 0.05 ב-30 ימים האחרונים).  
**Keep as Exemplar:** 2-3 atoms בכל prototype — הקרובים ביותר לcentroid + extremes של cluster לtoward debugging.  
**Never delete:** Asserted facts. Hypothesis edges (עד שverified).

---

## 📁 Data Layer Extensions

נצטרך subdirs חדשים ב-`data/`:

```
data/
├── baseline/               existing
├── benchmarks/             existing
├── corpora/                existing
├── installer/              existing
├── learning/               NEW
│   ├── prototypes/         serialized Prototype atoms + metadata
│   │   ├── emotion_sad_v1.proto
│   │   ├── emotion_joy_v1.proto
│   │   └── dialogue_comfort_v1.proto
│   └── archives/           gzipped raw atoms (episodic memory archive)
│       └── dialogue_2026_04.atoms.gz
└── prototypes_manifest.json   registry of all prototypes + versions
```

---

## 🛠️ Implementation Plan

**Phase A (additive, non-breaking):** — 1 session
- `src/learning_layer.rs` — new module with Pattern/Prototype/Exemplar types (**not yet** `AtomKind` variants)
- `ProvenanceLog` — side-car `HashMap<EdgeId, Provenance>` (not inside AtomEdge)
- `src/distillation.rs` — clustering + prototype creation (stub, no ML yet)
- Tests: 15+ covering pattern storage, provenance tagging, exemplar selection

**Phase B (migration, breaking):** — 1 session (with format_version bump)
- Extend `AtomKind` enum → `atom_persist` format_version: 2
- Migration tool: `migrate_v1_to_v2.rs` reads v1 atoms, adds default Provenance::Asserted
- Update all existing atoms to have provenance

**Phase C (feature integration):** — 2-3 sessions
- `smart_walk` filters by provenance per cognitive mode
- `dreaming` creates only `Hypothesis` edges
- `explain()` API shows provenance breakdown

**Phase D (distillation + clustering):** — 2-3 sessions + dependency
- Real clustering: add `ndarray` + simple k-means (or implement from scratch)
- Feature vectors: add `ort` crate for ONNX MiniLM embeddings
- Distillation scheduler

**Phase E (dialogue ingestion):** — 2 sessions
- `src/dialogue.rs` — DialogTurn, Conversation types
- `ingest_dialogue_jsonl()` on empathetic-dialogues subset
- Pattern search: "find me 5 examples where user frustrated → agent deescalated"

---

## ⚠️ סיכונים + mitigations

| סיכון | Mitigation |
|-------|------------|
| Concept drift (prototype זז שקט) | Versioning: `Prototype::sadness_v3` keeps history; explicit deprecation when drift > threshold |
| Prototype contamination (mixing emotions) | Bounded drift threshold + human-in-the-loop review for new prototypes |
| Over-distillation (לאבד ניואנס) | Keep 2-3 canonical exemplars per prototype; never delete Asserted |
| Query performance degradation | Provenance stored as enum (1 byte per edge); minimal overhead |
| Semantic gap (fuzzy prototype ↔ crisp symbol) | Confidence score on every edge; Precision mode filters by confidence ≥ 200 |
| Format migration errors | v1 snapshots remain in git; v2 tool is additive (adds Provenance::Asserted to all) |
| Copyright leakage | Never store original images/text post-archive; exemplars are text-only references to hash |

---

## 🎯 מה משתנה מבחינת יכולות

### ZETS היום
- יכול לענות "what is X?" על atoms שeingested
- Dreaming יוצר edges חדשים
- לא מבחין בין "observed in one dialogue" ל-"known fact"

### ZETS עם Learning Layer
- יכול לזהות "this user sounds sad" מ-prototype matching
- יכול לחפש: "show me 5 dialogues where frustration → resolution" (pattern query)
- `Precision` mode מסנן את ה-Observed — מחזיר רק ידע אמיתי
- `Narrative` mode משתמש בpatterns לתגובה טבעית
- Audit trace מציג אם תשובה מגיעה מfact או מpattern distilled

### מה זה **לא** עושה
- לא הופך את ZETS ל-LLM. generation עדיין דורש או template או LLM adapter.
- לא יוצר הבנה "עמוקה" כמו human empathy. זה pattern matching statistical.
- לא מייצר emotions בעצמו. רק מזהה ו-routes.

---

## ✅ המלצה לאישור

**אני ממליץ להתחיל Phase A (additive) בלבד בסשן הבא.** 

למה:
1. **נטו additive** — לא שובר tests, לא שובר format.
2. בסוף Phase A יש **demo מדיד**: "ingest 20 dialogue examples → pattern emerges → queryable."
3. רק אחרי שעידן רואה שזה עובד ב-Phase A → מאשר Phase B (הmigration).

Phase A deliverable:
- `src/learning_layer.rs` (~400 lines)
- `src/distillation.rs` stub (~200 lines, no ML yet, hand-written)
- 15+ tests
- `src/bin/pattern_demo.rs` — demo שmingest 20 short dialogues, distill 3 patterns
- Updated roadmap in docs

**השאלה לך, עידן:** ✅ אישור Phase A או החזר לreview?
