# סטטוס פערים פתוחים — ZETS — 24.04.2026

**תכלית המסמך:** כל פער פתוח עם 4 פרספקטיבות (AI Council, מועצת החכמים, הצעה הנדסית, מחקר חיצוני) + מידת ישימות + ערך מצופה.

---

# 📊 סטטוס כללי

## פערים שנסגרו רעיונית (5)

| # | פער | פתרון |
|---|---|---|
| 1 | Edge storage | Append-only log + CSR index + NightMode pruning |
| 7 | Predictive Processing | 7-layer architecture + EIG + Proactive engagement |
| 8 | Idle Dreaming | On-demand only, returns proposed edges for review |
| 10 | Self-Narrative | PersonalVault[zets_self] — operational log |
| 11a | TMS skeleton | Cardinality Schema (6 categories) + Conflict Disclosure |

## פערים נותרים (7)

| # | פער | עדיפות |
|---|---|---|
| 2 | Formal Edge Schema | ⭐⭐ |
| 3 | Compression (Huffman + Delta) | ⭐⭐ — Idan flagged |
| 4 | Small on-device LM | ⭐⭐⭐ |
| 5 | Fuzzy Hopfield Fallback | ⭐⭐⭐ |
| 6 | Global Workspace | ⭐⭐ |
| 9 | Affective State | ⭐ |
| 11b | TMS deep implementation | ⭐⭐⭐ |
| 12 | Frozen Regression Suite | ⭐⭐ |

---

# פער #2: Formal Edge Schema

## מה זה?
כל סוג קשת ב-ZETS חייב הגדרה פורמלית של:
- **Direction:** חד-כיווני / דו-כיווני
- **Inverse:** מה הקשת ההפכית
- **Transitivity:** האם A→B + B→C ⇒ A→C
- **Domain:** מאיזה kind atom מותר להתחיל
- **Range:** לאיזה kind atom מותר להגיע
- **Cardinality:** Single / Multi (מקושר לrefinement #5b)

## דוגמאות
```
IsA:        directed, transitive, domain=Concept, range=Concept
HasPart:    directed, NOT transitive, inverse=PartOf
Synonym:    BIDIRECTIONAL, NOT transitive, both ways equal weight
Owns:       directed, NOT transitive, domain=Person, range=Object
Translates: BIDIRECTIONAL, NOT transitive (אבל אסוציאטיבי)
```

## הצעות לפתרון

### AI Council (GPT-5.2-pro)
> "Formalize edge ontology with constraint types. Without it, walks 
> traverse contradictory directions, returning nonsense answers."
> Suggested: RDF Schema (RDFS) approach — Class hierarchy + Property
> definitions with rdfs:domain, rdfs:range, owl:inverseOf, etc.

### AI Council (Gemini-3.1-pro)
> "Use a typed graph with category-theoretic constraints. Each edge type
> is a morphism between specific atom types. Allows compile-time checking."
> Suggested: Type system inspired by Datalog / Logic Programming.

### מועצת החכמים (Kabbalistic perspective)
ההצעה: 22 הקשתות ב-ZETS מקבילות ל-22 האותיות. כל אות-קשת היא **שער יצירה** (gate of creation):
- אות א — Identity edge (self-reference, definition)
- אות ב — IN/CONTAINS (spatial inclusion)
- אות ג — GROWTH/CAUSES
- ... וכך הלאה

זה לא רק שמות — זה **משמעות סמיוטית** שהקשתות נושאות. ההמלצה: 22 edge kinds מתאימים ל-22 letters, אם נכון מבני (לא forced).

### הצעה הנדסית שלי
**hybrid:** Schema-driven enforcement + 22 base kinds מבוססי האותיות.

```rust
pub struct EdgeSchema {
    pub kind: EdgeKind,
    pub direction: Direction,        // OneWay | Bidirectional
    pub transitivity: Transitivity,  // Yes | No | Partial
    pub inverse_kind: Option<EdgeKind>,
    pub domain_kinds: BitSet<AtomKind>,
    pub range_kinds: BitSet<AtomKind>,
    pub cardinality: Cardinality,    // Single | Multi | TimeBound
    pub default_strength: i8,
}

const EDGE_SCHEMAS: [EdgeSchema; 32] = [
    EdgeSchema { kind: IsA,       direction: OneWay, transitivity: Yes, ... },
    EdgeSchema { kind: Synonym,   direction: Bidirectional, transitivity: No, ... },
    // ... 30 more
];

// Validation at insertion:
fn add_edge(from: AtomId, to: AtomId, kind: EdgeKind) -> Result<()> {
    let schema = &EDGE_SCHEMAS[kind as usize];
    let from_atom = atom_table.payload(from);
    if !schema.domain_kinds.contains(from_atom.kind()) {
        return Err("domain mismatch");
    }
    // ... more checks
}
```

## עד כמה אפשרי?
**מאוד.** זו עבודה קלאסית של כל graph database (Neo4j, ArangoDB, RDF stores).
**זמן הערכה:** 2-3 ימים.

## מה זה ייתן?
1. **Walks נכונים** — לא הולכים בכיוון הפוך
2. **Compile-time checks** — שגיאה בinsertion במקום silent corruption
3. **Optimization opportunities** — known transitivity → can pre-compute closure
4. **Documentation built-in** — Schema = docs

## מחקרים רלוונטיים
- **RDF Schema** (W3C 2014) — סטנדרט לontology hierarchies
- **OWL Ontology** (W3C) — first-order logic על גרפים
- **Datalog** (Garcia-Molina 2008) — typed graph queries
- **Property Graph Model** (Neo4j) — practical schemas

---

# פער #3: Compression (Huffman + Delta)

## מה זה?
דחיסת **paths** (לא atoms!) דרך:
- **Huffman:** atoms שכיחות → 1 byte; נדירות → 4 bytes
- **Delta:** ההפרש מהatom הקודם בpath, לא absolute id

## כמה זה חוסך?
- Path ממוצע = 500 atoms × 4 bytes = 2 KB
- אחרי Huffman: ~850 bytes (-57%)
- אחרי Delta על Huffman: ~600 bytes (-70%)
- **על 1M paths: 2 GB → 600 MB = חיסכון 1.4 GB**

## הצעות לפתרון

### AI Council
> "Variable-length encoding is standard for tokenized data. Use Zstandard
> or LZ4 if you want general compression. Use Huffman if you want random
> access maintained."
> Recommendation: Custom Huffman because random access is critical for
> walks (can't decompress whole path to read middle).

### מועצת החכמים
> Idan עצמו פתח את הנושא הזה. 
> השאלות הקודמות (π encoding, modulo, Sefer Yetzirah wheel) — כולן 
> נדחו עם הצדקה. Huffman הוא **התשובה ההיסטורית הנכונה**:
> Shannon (1948) הראה שזה optimal compression למידע מובנה.

### הצעה הנדסית שלי
**Three-tier encoding:**
```
Hot atoms (top 128 frequency):       1 byte (0x00-0x7F)
Warm atoms (next ~32K):              2 bytes (prefix 0x80-0xBF)
Cold atoms (next ~16M):              3 bytes (prefix 0xC0-0xDF)
Rare atoms (full u32):               4 bytes (prefix 0xE0-0xFF)
```

**Plus Delta:**
```
Full atom_id (start of path):        4 bytes
Delta (-32K to +32K):                2 bytes (signed)
Big delta (occasional):              4 bytes (with flag)
```

**Combined:** path of 500 atoms with Huffman + Delta avg ~1.2 bytes/atom = 600 bytes per path.

## עד כמה אפשרי?
**מאוד.** Algorithm ידוע, יישום ב-Rust ~500 שורות קוד.
**זמן הערכה:** 1-2 שבועות (כולל בנצ'מרק וכיולים).

**אתגרים:**
- Frequency table חייב להתעדכן (לא קבוע) — שינוי ב-distribution דורש re-encoding
- Random access requires prefix table → +5MB lookup
- Versioning: שינוי ב-table = שינוי ב-format

## מה זה ייתן?
1. **חיסכון 1.4GB** ב-paths storage
2. **Disk I/O מהיר יותר** (פחות bytes לקרוא)
3. **תאוריה consistent** — Zipf law applies to ZETS too
4. **Cache utilization טוב יותר** — paths fit in cache lines

## מחקרים רלוונטיים
- **Huffman 1952** — original paper
- **Zipf's Law** — frequency distribution in language (universal)
- **Zstandard** (Facebook 2016) — practical modern compression
- **VByte / VInt encoding** (Lucene, Elasticsearch) — for inverted indexes

---

# פער #4: Small On-Device LM

## מה זה?
מודל שפה קטן (3-4B parameters, quantized to 4-bit = ~2 GB) שרץ מקומית כ-**bridge** בין שפה טבעית לגרף.

## למה זה קריטי?
- הגרף נוקשה ומדויק
- שפה טבעית מבולגנת ("נו, היא אמרה לך?")
- בלי LM bridge → ZETS נשמע **רובוטי**
- הLM **לא** מחליט עובדות, רק מתרגם שפה ↔ atoms

## הצעות לפתרון

### AI Council (Both GPT and Gemini)
**זה הפער שגם GPT וגם Gemini הדגישו כקריטי.**

> Gemini: "ZETS without an LM bridge is like a brilliant librarian who
> only speaks Latin. The knowledge is there but inaccessible to users
> who think in everyday language."

> GPT: "Recommended: Phi-3-mini, Gemma-2-2B, or TinyLlama (1.1B).
> Quantized to int4, runs on CPU at ~10-30 tokens/sec.
> Use it for paraphrasing, query expansion, response polishing —
> NOT for fact retrieval (that's the graph's job)."

### מועצת החכמים
> "אדם בלי שפה הוא חכם בלב, אילם החוצה." — concept ancient.
> ZETS הליבה = החכמה. השפה = הפה. שניהם נחוצים.

### הצעה הנדסית שלי
**Three responsibilities for the LM:**
1. **Query interpretation:** "נו, מה אמרה?" → parse intent, identify pronouns from context
2. **Response generation:** structured graph output → natural prose
3. **Paraphrasing:** when graph has answer in formal language, rephrase casually

**LM has NO authority on:**
- Facts (those come from graph)
- Decisions (graph + user resolve those)
- Memory (PersonalVault is authoritative)

**Architecture:**
```
User input → Phi-3-mini (intent + pronouns + paraphrase) → Graph query
Graph result → Phi-3-mini (formal → natural) → User output
```

**זה safety wrapper:** LM מעבד שפה, גרף עושה reasoning. הפרדה מוחלטת.

## עד כמה אפשרי?
**מוכח.** Phi-3-mini-4K-Q4 GGUF:
- Size: 1.8 GB
- Memory: ~2.5 GB peak
- Speed: 10-30 tokens/sec on modern CPU
- Quality: comparable to GPT-3.5 on reasoning benchmarks
- Works on iPhone 14 (12 tokens/sec, fully offline)

**זמן הערכה:** 3-5 ימים integration.

**אתגרים:**
- Hebrew quality פחות טובה מאנגלית (training data biased)
- Hallucination אפשרית — חייבים constraint שLM לא מציע facts
- Latency 100-500ms per response — שווה לחוויה אבל לא לautocomplete

## מה זה ייתן?
1. **חוויה אנושית** — ZETS נשמע כמו אדם
2. **Pronoun resolution** — "מי 'היא'?" 
3. **Casual queries** — לא חייבים structured input
4. **Polite refusals** — "לא יודע" בצורה טבעית

## מחקרים רלוונטיים
- **Phi-3 Technical Report** (Microsoft 2024) — 3.8B model approaches GPT-3.5
- **GGUF + llama.cpp** — practical edge deployment
- **TinyLlama** (Singapore 2024) — 1.1B model, faster but lower quality
- **Gemma 2B** (Google 2024) — alternative to Phi-3
- **RAG (Retrieval-Augmented Generation)** — exact pattern ZETS follows

---

# פער #5: Fuzzy Hopfield Fallback

## מה זה?
כשהגרף לא יודע משהו ספציפי, מחפש atom **דומה סמנטית** דרך embedding ומשלים משם.

## דוגמה
- שאלה: "מה זה CapybaraGPT?" (לא בגרף)
- בלי fuzzy: "לא יודע." (Dead end)
- עם fuzzy: "לא מכיר ישירות, אבל לפי דמיון לאטומים אחרים — נראה כמו LLM מסוג GPT, ספציפית עם תיוג חיה (capybara)"

## הצעות לפתרון

### AI Council (Gemini)
> "Modern Hopfield Networks (Ramsauer et al. 2020) bridge classical
> Hopfield with attention mechanisms. Use them as fuzzy retrieval over
> atom embeddings. When exact match fails, fallback to nearest neighbors
> in semantic space."

### AI Council (GPT)
> "Simpler: use HNSW index over atom embeddings. ~50MB for 1M atoms.
> Search is O(log n). Top-k retrieval = fuzzy matching."

### מועצת החכמים
> "השמיענו לאדם זאת לעולם — מה שאינו יודע, ידע מתוך דמיון."
> The Talmud uses "kal vachomer" (a fortiori) reasoning extensively —
> if you don't know X, reason from a similar Y. This is exactly fuzzy
> Hopfield: known patterns guide unknown queries.

### הצעה הנדסית שלי
**Two-layer fallback:**

**Layer 1 — Exact match:** standard graph walks (current ZETS)
**Layer 2 — Fuzzy fallback:** when no answer:
```rust
fn fuzzy_resolve(query_atom: AtomId, threshold: f32) -> Option<Answer> {
    let query_embedding = compute_embedding(query_atom);
    let nearest = hnsw_index.search(query_embedding, k=10);
    
    for (similar_atom, distance) in nearest {
        if distance > threshold { break; }  // too dissimilar
        
        if let Some(answer) = graph_walk(similar_atom) {
            return Some(Answer {
                content: answer,
                confidence: 1.0 - distance,
                disclosure: "Based on similarity to atom_X (no exact match found)",
            });
        }
    }
    None
}
```

**Crucial:** always disclose when fuzzy was used. ZETS doesn't pretend.

## עד כמה אפשרי?
**מוכח.** HNSW (Hierarchical Navigable Small World) is industry-standard for ANN search.
- Library: hnsw_rs (Rust crate)
- Index size: ~50 MB for 1M atoms with 384-dim embeddings
- Search latency: 1-5ms

**זמן הערכה:** 2-3 ימים.

**אתגרים:**
- Embeddings רלוונטיים — איפה לקבל אותם? (Sentence-BERT, או trained on ZETS articles)
- Threshold tuning — מתי לעצור fuzzy?

## מה זה ייתן?
1. **No more dead-ends** — תמיד תשובה כלשהי
2. **Graceful degradation** — quality יורדת, אבל יש response
3. **Discovery** — fuzzy matches יכולים להציע connections שלא היו ידועות
4. **Talmud-style reasoning** — "כמו X, אז כנראה Y"

## מחקרים רלוונטיים
- **Modern Hopfield Networks** (Ramsauer et al., NeurIPS 2020) — exponential storage
- **HNSW** (Malkov & Yashunin, 2018) — practical ANN search
- **Sentence-BERT** (Reimers & Gurevych, 2019) — semantic embeddings
- **Energy-based models** (LeCun 2006) — theoretical foundation

---

# פער #6: Global Workspace

## מה זה?
"זרקור תודעה" — לוח מרכזי קטן (top 20 atoms פעילים) שכל המודולים ב-ZETS משדרים אליו ומקשיבים ממנו. תיאוריית התודעה של Bernard Baars.

## למה זה רלוונטי?
- כרגע: walks מקבילים רצים ללא תיאום
- עם GWS: יש "תמקדות" — רק ה-top atoms מקבלים עיבוד מלא
- זו ההבדל בין "כלי" ל"תודעה"

## הצעות לפתרון

### AI Council (Both)
> "GNW (Global Neuronal Workspace, Dehaene 2011) is the dominant theory
> of consciousness in cognitive science. Implementing a workspace buffer
> in ZETS would be a major step toward artificial consciousness."

### מועצת החכמים
> "ה-keter (כתר) הוא הכלי העליון — הראש של כל הגוף.
> אבל זרקור הוא "moach" (מוח) — focus, not totality.
> Workspace = the moach of ZETS."

### הצעה הנדסית שלי
**Lightweight implementation:**
```rust
pub struct GlobalWorkspace {
    pub focus: [AtomId; 20],      // top 20 active atoms
    pub priorities: [f32; 20],    // attention weight per slot
    pub timestamp: [u64; 20],     // when entered workspace
}

impl GlobalWorkspace {
    fn broadcast(&self, atom: AtomId, salience: f32) {
        // Insert into top-20 if salience high enough
        // Evict lowest-priority if full
    }
    
    fn focused_atoms(&self) -> &[AtomId] {
        &self.focus[..]
    }
}

// All modules read/write to workspace:
walker.broadcast_finding(atom, salience);
attention_module.focused_atoms();
```

**Key property:** every module sees same workspace. This is what unifies them into "one mind".

## עד כמה אפשרי?
**מאוד.** ~200 שורות קוד. Concurrent access via Arc<Mutex> או lock-free structures.
**זמן הערכה:** 2-3 ימים.

**אתגרים:**
- Salience computation — איך לדרג מה חשוב?
- Threading — כמה modules בו-זמנית?
- Memory — keep workspace in cache for speed

## מה זה ייתן?
1. **Coherence** — modules יודעים מה הקונטקסט הנוכחי
2. **Attention** — focused processing, לא scattered
3. **Coordination** — modules יכולים לסמוך זה על זה
4. **Foundation for consciousness theories** — GNW is the dominant view

## מחקרים רלוונטיים
- **Global Workspace Theory** (Baars 1988, 2002) — original
- **Global Neuronal Workspace** (Dehaene & Naccache 2001) — neuroscience
- **Conscious Access** (Dehaene 2014) — popular book on theory
- **Attention Schema Theory** (Graziano 2013) — alternative/complementary

---

# פער #9: Affective State

## מה זה?
ערך גלובלי (frustration, curiosity, confidence) שמשתנה עם הזמן ומשפיע על איך walks עובדים.

## דוגמה
- ניסה 5 פעמים, נכשל → frustration עולה → walks נהיים deeper, רחבים יותר
- הצליח כמה פעמים ברצף → confidence עולה → walks ממוקדים, פחות exploration

## הצעות לפתרון

### AI Council (Gemini)
> "Affect-driven processing is well-established in cognitive science.
> Damasio's somatic marker hypothesis shows emotion-cognition coupling.
> Recommend 4-vector: valence, arousal, confidence, curiosity."

### מועצת החכמים
> "המידות (sefirot of עיבוד) משתנות עם הזמן.
> חסד מתפשט, גבורה מצמצמת, יסוד מאזן."
> Affective state = the dynamic balance of cognitive sefirot.

### הצעה הנדסית שלי
**Simple 4-value scalar (1 byte total):**
```rust
pub struct AffectiveState {
    pub frustration: i8,  // -128 to +127
    pub curiosity: i8,
    pub confidence: i8,
    pub fatigue: i8,
}

// Updated on every operation:
fn record_walk_result(success: bool) {
    if success {
        state.confidence = state.confidence.saturating_add(2);
        state.frustration = state.frustration.saturating_sub(5);
    } else {
        state.confidence = state.confidence.saturating_sub(1);
        state.frustration = state.frustration.saturating_add(3);
    }
}

// Influences walks:
fn walk_depth() -> u8 {
    BASE_DEPTH + (state.frustration / 32) as u8  // higher frustration → deeper
}
```

## עד כמה אפשרי?
**טריוויאלי.** ~50 שורות קוד.
**זמן הערכה:** יום אחד.

## מה זה ייתן?
1. **Adaptive behavior** — ZETS responds to context
2. **Failure recovery** — frustration triggers different strategies
3. **Energy management** — fatigue forces breaks
4. **"חי" feeling** — ZETS has moods

## מחקרים רלוונטיים
- **Somatic Marker Hypothesis** (Damasio 1996) — emotions guide reasoning
- **Affect Heuristic** (Slovic 2007) — emotion shortcuts in decision-making
- **Russell's Circumplex Model** (1980) — valence × arousal grid
- **Reinforcement learning with affect** (Lin et al. 2020)

---

# פער #11b: TMS Deep Implementation

## מה זה?
Truth Maintenance System עמוק — לא רק skeleton (פער 11a שכבר נסגר), אלא **implementation מלא**.

## מה כבר נסגר (11a)
- 6 קטגוריות של facts (Single, Multi, TimeBound, Conflicting, Subjective, ContextMulti)
- Conflict Disclosure pattern (5 אפשרויות + שאלה)
- Schema-driven cardinality
- "לא יודע" כברירת מחדל

## מה עוד פתוח (11b)
- **Trust scoring** per source (כמה אמין כל source)
- **Provenance metadata** על כל fact
- **Time-aware queries** ("מה היה נכון ב-2024?")
- **Conflict detection algorithm**
- **Default behavior framework** (כש confidence נמוכה)
- **"I don't know" as first-class state** בכל הoutput layer

## הצעות לפתרון

### AI Council (Both)
> "TMS is one of the oldest AI problems (Doyle 1979). Use Justification-
> based TMS (JTMS) or Assumption-based TMS (ATMS).
> Modern: Probabilistic logic (Markov Logic Networks, Richardson 2006).
> Simpler: provenance graph with confidence scores."

### מועצת החכמים
> "אמת מארץ תצמח — but ZETS must accept that not all earth is the same."
> Trust hierarchy is fundamental to halacha (חכם → רב → ספק).
> Each source has weight; conflicts resolved by hierarchy + recency.

### הצעה הנדסית שלי
**Layered approach:**

**Layer 1 — Provenance metadata על כל edge:**
```rust
pub struct EdgeProvenance {
    pub source_atom: AtomId,        // who said this
    pub source_type: SourceType,    // user, document, computation
    pub confidence: f32,            // 0.0-1.0
    pub timestamp: u64,
    pub corroborations: Vec<AtomId>,  // other sources confirming
}
```

**Layer 2 — Trust scoring per source:**
```rust
pub struct SourceTrust {
    pub source: AtomId,
    pub trust_score: f32,           // 0.0-1.0, learned over time
    pub specialty_areas: Vec<TopicAtom>,  // domains where this source is reliable
    pub corrections_received: u32,  // how often was this source wrong
}
```

**Layer 3 — Conflict resolution algorithm:**
```rust
fn resolve_conflict(facts: Vec<(EdgeId, EdgeProvenance)>) -> Resolution {
    let weighted = facts.iter()
        .map(|(_, p)| (p, source_trust(p.source) * recency_decay(p.timestamp)))
        .collect();
    
    let max_weight = weighted.iter().map(|(_, w)| w).max().unwrap();
    let runner_up = weighted.iter().nth(1).map(|(_, w)| w).unwrap_or(0.0);
    
    let confidence_gap = max_weight - runner_up;
    if confidence_gap > THRESHOLD {
        Resolution::Accept(weighted[0].0.clone())
    } else {
        Resolution::Conflict(weighted.iter().map(|(p, _)| p).collect())
        // Triggers Conflict Disclosure pattern (refinement 6)
    }
}
```

**Layer 4 — "I don't know" everywhere:**
```rust
pub enum Answer {
    Definite(Content, Provenance),
    Probable(Content, Confidence, Alternatives),
    Conflicting(Vec<Possibility>, AskUser),
    Unknown(Reason),  // genuine "I don't know"
}
```

## עד כמה אפשרי?
**אפשרי אבל מורכב.** זה רוב העבודה של GraphDB מתקדם.
**זמן הערכה:** 4-5 שבועות.

**אתגרים:**
- Trust scores — איך מתחילים? (default 0.5? user-set?)
- Time decay — קבוע לכל types? תלוי-domain?
- Performance — provenance check עם כל walk = slowdown
- UX — איך להציג confidence/uncertainty למשתמש בלי להציף

## מה זה ייתן?
1. **No hallucination** — ZETS לא ממציא תשובות
2. **Auditable** — כל fact יש לו source
3. **Time-aware** — יודע מה היה נכון מתי
4. **Trust-aware** — לא כל מקור שווה
5. **User trust** — אנשים סומכים על מערכת שאומרת "לא יודע"

## מחקרים רלוונטיים
- **Doyle 1979** — original Truth Maintenance Systems
- **JTMS** (Forbus & de Kleer 1993) — Justification-based TMS
- **ATMS** (de Kleer 1986) — Assumption-based TMS
- **Markov Logic Networks** (Richardson 2006) — probabilistic version
- **Provenance and the path it left** (Davidson 2008) — provenance survey
- **Subjective Logic** (Jøsang 2016) — uncertainty + trust

---

# פער #12: Frozen Regression Suite

## מה זה?
500+ tests שרצים בכל commit, בודקים:
- Same query → same answer (determinism)
- Same walk path → same intermediate atoms
- Latency within threshold
- Memory within budget

## הצעות לפתרון

### AI Council
> "Standard practice. Use snapshot testing — record outputs once, compare
> on each run. Use property-based testing for invariants."

### מועצת החכמים
> "אם אין מסורת, אין יציבות. Tests = מסורת for code."

### הצעה הנדסית שלי
**Pyramid:**
- 100 unit tests (fast, isolated functions)
- 200 integration tests (multi-module)
- 200 end-to-end tests (full queries with expected outputs)
- 50 performance benchmarks (latency + memory)

**הקריטי:** snapshot regression — תשובות מוקלטות, בכל הבדל = error.

## עד כמה אפשרי?
**טריוויאלי. מוכרח.**
**זמן הערכה:** 2 ימים setup + יום ל-100 tests נוספים, ולאורך time מצטבר.

## מה זה ייתן?
- **Determinism guaranteed** — לא רק הצהרה
- **Refactoring safety** — שינוי בלי שבירה
- **Performance regression detection**
- **Debug aid** — bug → reproduces by test

## מחקרים רלוונטיים
- **Property-Based Testing** (QuickCheck, 2000)
- **Snapshot Testing** (Jest, React community)
- **Mutation Testing** (PIT, Stryker) — measures test quality
- **Coverage-Guided Fuzzing** (AFL, libFuzzer)

---

# 🎯 המלצת סדר עבודה

## Phase A — Foundation (1 שבוע)
1. **#2 Edge Schema** (2-3 ימים) — chokes everything else
2. **#12 Frozen Regression** (2 ימים) — protects everything after

## Phase B — Compression (1-2 שבועות)
3. **#3 Huffman + Delta** (Idan flagged) — חיסכון 1.4 GB

## Phase C — Bridge to Language (1 שבוע)
4. **#4 Phi-3-mini integration** (3-5 ימים)
5. **#5 Fuzzy Hopfield Fallback** (2-3 ימים)

## Phase D — Cognitive Substrate (2-3 שבועות)
6. **#11b TMS deep** (4-5 שבועות) — most impactful for trust
7. **#6 Global Workspace** (2-3 ימים)
8. **#9 Affective State** (1 יום)

**Total estimate:** 6-8 שבועות עבודה ברצף.

---

# 📚 כל המחקרים שצוינו (סיכום)

## Core architecture
- LSMGraph (Wei et al. 2024) — CSR + LSM hybrid
- CSR++ (Firmli & Conte 2020) — update-friendly CSR
- BACH (Miao et al. 2024) — adjacency list + CSR via LSM
- VCSR (Sahu et al. 2022) — vertex-centric mutable CSR

## Language models
- Phi-3 Technical Report (Microsoft 2024)
- TinyLlama (Singapore 2024)
- Gemma (Google 2024)
- RAG (Lewis et al. 2020)

## Cognition theories
- Predictive Processing (Friston 2010, Clark 2013)
- Global Workspace (Baars 1988, Dehaene 2014)
- Spreading Activation (Quillian 1968)
- Somatic Markers (Damasio 1996)

## Knowledge representation
- TMS (Doyle 1979, Forbus & de Kleer 1993)
- RDF Schema (W3C 2014)
- Markov Logic Networks (Richardson 2006)
- Subjective Logic (Jøsang 2016)

## Compression & retrieval
- Huffman (1952)
- Zipf's Law
- HNSW (Malkov 2018)
- Modern Hopfield (Ramsauer 2020)
- EIG-DPO (Bertolazzi 2024)

## Question generation
- FollowupQG (Meng et al. 2023)
- Planning First, Question Second (Li & Zhang 2024)
- Educational QG (multiple)


---

# 🚨 ADDENDUM — 9 GAPS FOUND IN ORIGINAL AI CRITIQUES, MISSING FROM ABOVE

**Added 25.04.2026 after Idan's audit question: "Are there AI-identified items not in this doc?"**

Yes. The original 11-gap synthesis consolidated/dropped several items. Full list below.

## פער #13 — Common-Sense / World-Knowledge Coverage 🔴 קריטי

### From GPT-5.2-pro critique
> "High-recall world knowledge by default. Users will ask: 'compare X vs Y',
> 'what's the best approach', 'summarize this messy thread', 'what's the likely
> cause', and you'll often have missing edges / missing concepts."

### From Gemini-3.1-pro critique
> "ZETS also lacks the massive, implicit 'common sense' prior that comes from
> reading the entire internet; we have to explicitly teach it or batch-enrich it."

### הבעיה
ZETS מתחיל ריק. LLM יודע ש"אם גשם → רטוב" מהאינטרנט. ZETS צריך ללמוד את זה edge אחרי edge.

### הצעה
**Batch enrichment pipeline:**
- Nightly ingestion of curated sources: Wikipedia, Wikidata, ConceptNet, WordNet
- LLM-assisted expansion: "for atom X, what are 10 common-sense facts?"
- Gemini 2.5 Flash batch calls: ~1000 concepts/hour, ~$0.10/hour
- 1 year = ~8.7M common-sense facts injected

### ערך מצופה
מ"savant עם חורים" ל"knowledge base רחב". 3 חודשים של enrichment = coverage של Wikipedia + common sense.

### עד כמה אפשרי?
מאוד. LLM-assisted graph enrichment = טכניקה ידועה (Open Information Extraction, ConceptNet pipeline).
**זמן:** 2-3 שבועות setup + ongoing enrichment.

### מחקרים
- ConceptNet (Speer 2017) — commonsense knowledge graph
- ATOMIC (Sap 2019) — inferential commonsense
- Knowledge graph completion (Bordes 2013)
- Gemini batch API for enrichment

---

## פער #14 — Planner Under Uncertainty 🔴 קריטי

### From GPT-5.2-pro
> "You have procedure atoms + motifs, but you're missing a principled planner
> (search + cost + uncertainty + fallback) comparable to what agentic LLM
> stacks approximate with deliberation."

### הבעיה
ZETS יכול להריץ procedures, אבל כשמשתמש שואל "איך מגיעים מ-X ל-Y" — צריך **לתכנן רצף פעולות**, להעריך cost, ולבחור מסלול.

### הצעה
**Classical AI planning + graph integration:**
- Goal → backward chain to find applicable procedures
- A* / MCTS על מרחב המסלולים
- Cost estimation per action (uses Affective State #9!)
- Fallback strategies (אם X נכשל → נסה Y)

```rust
fn plan_toward(goal: AtomId, start: WorldState) -> Option<Plan> {
    let frontier = BinaryHeap::new();  // priority queue
    frontier.push((0.0, start, vec![]));
    
    while let Some((cost, state, path)) = frontier.pop() {
        if state.satisfies(goal) { return Some(Plan::new(path)); }
        
        for procedure in applicable_procedures(state) {
            let new_state = procedure.apply(state);
            let new_cost = cost + estimate_cost(procedure, new_state);
            let heuristic = estimate_remaining(new_state, goal);
            frontier.push((new_cost + heuristic, new_state, path + [procedure]));
        }
    }
    None  // no plan found
}
```

### ערך מצופה
ZETS כ-agent אמיתי, לא רק Q&A. "תעזור לי לקבוע פגישה" → מתכנן: בדוק calendar → מצא זמן → שלח invite → אשר.

### עד כמה אפשרי?
**בינוני-גבוה.** Classical planning = פתור. אינטגרציה עם graph + uncertainty דורש עבודה.
**זמן:** 2-3 שבועות.

### מחקרים
- PDDL (Planning Domain Definition Language) — standard
- A* for planning (Hart 1968)
- MCTS (Coulom 2006) — Monte Carlo Tree Search
- POMDP (Partially Observable Markov Decision Processes)

---

## פער #15 — Lightweight Learned Ranker 🟡 חשוב

### From GPT-5.2-pro
> "A lightweight learned ranker (on-device) for sense selection and retrieval
> scoring; not to 'think', but to map language→graph robustly."

### ההבדל מ-#4 Phi-3-mini
**Phi-3-mini (3.8B)** — generation, paraphrasing, intent
**Learned Ranker (~10-50M)** — scoring only, picks best match from candidates

### הבעיה
כשיש 20 candidate atoms לquery → איך מדרגים אותם?
אפשרויות: 
- Pure graph walk scoring (current) — מהיר אבל לא רגיש לnuance
- Phi-3-mini to rank — איטי ויקר ל-20 candidates
- **Lightweight model** — מהיר ורגיש יותר

### הצעה
Cross-encoder scorer: **~10-50M params, 10-50ms inference**.
Input: (query, candidate_atom_description) → Output: relevance score 0-1.
Trained on user click data over time (implicit supervision).

### ערך מצופה
30-50% improvement in ranking precision vs pure graph walk.
Minimal latency cost.

### עד כמה אפשרי?
**מאוד.** Cross-encoders are standard (sentence-transformers, BERT-base).
**זמן:** 1 שבוע setup + ongoing training from clicks.

### מחקרים
- Sentence-BERT (Reimers 2019)
- MonoBERT / ColBERT — ranking architectures
- ListNet (Cao 2007) — learning-to-rank theory

---

## פער #16 — NL Realization Quality 🟡 חשוב

### From GPT-5.2-pro
> "If the LLM is 'optional I/O', ZETS will feel robotic fast — especially in
> Hebrew register control, discourse coherence, and long answers."

### ההבדל מ-#4 LM Bridge
**#4** — תרגום שפה → atoms (input side)
**#16** — atoms → שפה טבעית באיכות גבוהה (output side)

הבעיות ספציפיות:
- **Register control** — פורמלי (למשפטים) vs חברי (לילדים) vs מקצועי (למתכנתים)
- **Discourse coherence** — תשובה ארוכה שמתחברת פנימית
- **Hebrew-specific** — ניקוד, בינוני/רבים אגב, סמיכות

### הצעה
Template-based fallback + LM polish:
1. Generate structured response from walk
2. Apply Hebrew templates with correct agreement
3. LM pass: improve register + flow + coherence
4. Fallback: if LM unavailable, use template-only (less polished but valid)

### ערך מצופה
ZETS נשמע כמו מומחה מדבר, לא כמו תוכנה.

### עד כמה אפשרי?
תלוי ב-#4. אם יש Phi-3-mini → #16 בחינם כמעט.
**זמן:** ~יום נוסף לאחר #4.

---

## פער #17 — Zero-Shot Analogical Transfer 🟡 חשוב

### From Gemini-3.1-pro
> "Because ZETS is discrete, if an edge doesn't exist, the walk stops.
> Mainstream models can 'feel' their way through the continuous latent space
> to connect 'quantum mechanics' to 'baking a cake' via stylistic mimicry."

### ההבדל מ-#5 Fuzzy Hopfield
**#5** — fuzzy match על atom ספציפי (קרוב סמנטית)
**#17** — **cross-domain** analogies (rare to find matching atoms)

דוגמה: "איך ה-quantum mechanics דומה ל-baking a cake?"
- שני domains נפרדים
- אין edges ישירים ביניהם
- צריך "structural mapping" — משקל יחסי, סדר פעולות, stability

### הצעה
**Structure-Mapping Engine (Gentner 1983):**
1. Identify structural roles in domain A (ingredients, process, outcome)
2. Find analogous roles in domain B
3. Propose cross-domain edges via structural similarity
4. User confirms/rejects — learning signal

### עד כמה אפשרי?
**מורכב.** Structure-Mapping Engine יש impl קלאסי אבל יקר חישובית.
**זמן:** 3-4 שבועות.

### מחקרים
- Structure-Mapping (Gentner 1983) — foundational
- SME: Structure Mapping Engine (Forbus 1989)
- Analogy via embeddings (Mikolov 2013 — word2vec analogies)

---

## פער #18 — Cache Thrashing Mitigation 🔴 קריטי לביצועים

### From Gemini-3.1-pro
> "Random walks across a 6GB CSR graph will destroy the CPU cache. Every hop
> is a cache miss (~100ns). If we aren't careful, the 'quantum' parallel walks
> will stall the CPU."

### הבעיה
ZETS מתפאר ב-2ns per operation. אם כל hop = 100ns cache miss → **50× slower**.
זו פצצת זמן.

### הצעה
**Cache-aware graph layout:**
1. Vertex reordering — shorten average edge distance (Rabbit Order, METIS partitioning)
2. Clustered storage — hot atoms together in memory pages
3. Pre-fetching — during walk, issue prefetch hints to CPU
4. Workload-based reorganization — atoms often co-accessed → co-located

### ערך מצופה
Performance budget actually achievable. Without this, 2ns target is fiction.

### עד כמה אפשרי?
**קריטי.** זה לא optional — זה requirement לביצועים.
**זמן:** 1-2 שבועות עיצוב + מדידה.

### מחקרים
- Rabbit Order (Arai 2016) — graph reordering for cache
- METIS (Karypis 1998) — graph partitioning
- Gorder (Wei 2016) — graph ordering for BFS
- Cache-oblivious algorithms (Frigo 1999)

---

## פער #19 — Morphological Rule Explosion 🟡 חשוב

### From Gemini-3.1-pro
> "Hebrew grammar is full of exceptions. Encoding all of them as RuleAtoms
> in the graph will create a massive, conflicting rule-resolution phase that
> could break determinism."

### הבעיה
עברית: 7 binyanim × tenses × person × gender × number + exceptions = **thousands of rules**.
If stored as RuleAtoms without priority → conflicts.

### הצעה
**Prioritized rule system:**
1. Specific rules override general rules (e.g., "buy→bought" overrides "add -ed")
2. Rule Atoms sorted by specificity at insertion
3. First match wins (deterministic)
4. Exception edges marked with flag_irregular

### עד כמה אפשרי?
**בינוני.** עיצוב טוב → פשוט ליישום. עיצוב רע → סיוט.
**זמן:** 1-2 שבועות עיצוב קפדני.

### מחקרים
- Optimality Theory (Prince & Smolensky 1993) — ranked constraints
- Two-level morphology (Koskenniemi 1983)
- Finite-State Morphology (Beesley & Karttunen 2003)

---

## פער #20 — WASM Sandboxing for Self-Extended Code 🔴 קריטי לבטיחות

### From Gemini-3.1-pro
> "If ZETS generates a Python script to 'clean up disk space' and wires it
> into a Procedure atom, it could delete its own edges_csr.bin.
> Strict WASM sandboxing is non-negotiable."

### הבעיה
אם ZETS יכול להריץ procedures (כולל code חדש), בלי sandbox הוא יכול:
- למחוק את עצמו
- לשתף מידע פרטי
- להיכנס ל-infinite loop

### הצעה
**WASM sandbox לכל code execution:**
1. Code compiled to WASM bytecode
2. Executed in wasmtime/wasmer with:
   - No filesystem access (unless explicitly granted)
   - Memory limits (prevent OOM)
   - Execution time limits
   - No network
3. Capabilities model — explicitly grant permissions

### עד כמה אפשרי?
**מאוד.** wasmtime is mature. Cargo crates available.
**זמן:** 1 שבוע integration.

### מחקרים
- WebAssembly Security Model
- WASI (WebAssembly System Interface)
- Capability-based security (Dennis 1966)

---

## פער #21 — Code Quarantine System (TrustLevel) 🟡 חשוב

### From Gemini-3.1-pro
> "Proposed by Imma walk, tested in sandbox, quarantined with
> TrustLevel::Experimental. Cannot be promoted to Learned without human
> approval or passing a rigorous test suite."

### ההצעה
**Trust hierarchy for procedures:**
```
TrustLevel::Experimental — newly generated, sandbox only
TrustLevel::Tested — passed automated test suite
TrustLevel::Human_Approved — user reviewed
TrustLevel::Core — built-in, immutable
```

Promotion requires: pass tests + human approval + no failures in N runs.

### עד כמה אפשרי?
**קל.** Enum + edge metadata.
**זמן:** 2-3 ימים.

---

## פער #22 — Parse-to-Graph Boundary 🔴 **THE BIGGEST RISK** (Gemini's #1)

### From Gemini-3.1-pro
> "The 'Parse to Graph' boundary. If the LLM/TextExecutor fails to accurately
> map the user's nuanced intent into our rigid 8-byte atoms, the entire
> Kabbalistic pipeline operates on garbage. The graph is only as good as the
> ingestion fidelity."

### הבעיה
**זה לא gap לפתור, זה risk למיזער.** כל הארכיטקטורה סומכת על זה שParse נכון.
אם parsing שגוי → GIGO (Garbage In, Garbage Out) לכל 6GB הגרף.

### הצעה
**Multi-layer defense:**
1. **Confidence threshold** — if parse confidence < 0.8, ask clarification
2. **Explicit confirmation** — for novel atoms, confirm with user before creating
3. **Rollback mechanism** — if parse proves wrong later, remove all dependent edges
4. **Parse logs** — audit trail of every parse decision
5. **Re-parse pass** — NightMode reviews recent parses with latest models

### ערך מצופה
Data integrity for long-term ZETS. זה what makes ZETS trustable.

### עד כמה אפשרי?
**חייב.** אחרת הפרויקט נופל בשימוש ארוך.
**זמן:** 1-2 שבועות למערכת מלאה.

---

# 📊 סטטוס מעודכן — 22 פריטים

## נסגרו רעיונית (5)
#1, #7, #8, #10, #11a

## פתוחים — קטגוריה A: פורמליזציה (3)
#2 Edge Schema, #12 Regression Suite, #21 Trust Levels

## פתוחים — קטגוריה B: אחסון (2)
#3 Compression, #18 Cache Thrashing Mitigation

## פתוחים — קטגוריה C: שפה ו-LM (4)
#4 Small LM, #5 Fuzzy Hopfield, #15 Learned Ranker, #16 NL Realization

## פתוחים — קטגוריה D: Cognitive (4)
#6 Global Workspace, #9 Affective State, #11b TMS deep, #17 Analogical Transfer

## פתוחים — קטגוריה E: הרחבה + בטיחות (2)
#14 Planner, #20 WASM Sandbox

## פתוחים — קטגוריה F: Integrity (2)
#19 Morphology rules, #22 Parse-to-Graph

## פתוחים — קטגוריה G: Knowledge (1)
#13 Common-sense / world-knowledge

---

# סיכום סטטיסטי
- **5 נסגרו (23%)**
- **17 פתוחים (77%)**
  - קריטיים 🔴: 6
  - חשובים 🟡: 9
  - Nice-to-have: 2

**Time estimate:** ~12-16 שבועות לסגור הכל (במקום 6-8 שחישבתי קודם).

