# AGI Blueprint — Living Document

**Status:** 🟡 Draft v1 — in active design, updated every iteration  
**Created:** 2026-04-24  
**Last updated:** 2026-04-24  
**Purpose:** The single source of truth for ZETS AGI architecture. Every design decision, every revision, every AI consultation lands here.

---

## 🔥 Why This Document Exists

Previous design docs were scattered across `20_research/`, `architecture/`, and memory. This is the **one document** that:
- Holds current architecture state
- Records every "breaking the tools" cycle
- Stores AI consultation results
- Shows what's accepted, what's rejected, what's pending

When we finish, this becomes the ADR (Architecture Decision Record) for v2.

---

## 🧠 The 7 Core Design Principles (Post-Breaking)

These are principles that **survived** rigorous critique. Before each was accepted, we tried to break it — only what held up stayed.

### Principle 1: Angels = 7 Traversal Directions (NOT edge types)

**Broken assumption:** "7 angels = 7 edge types" was too narrow.

**Why it broke:**
- Real knowledge has 15-25+ relation types (causal, temporal, semantic-cultural, etc.)
- Forcing all edges into 7 categories loses information

**Accepted structure:**
- **7 Angels** = 7 **investigation directions** for graph traversal
  - אוריאל (Uriel): visual-oriented search
  - רפאל (Raphael): taste/healing oriented
  - גבריאל (Gabriel): ancient/smell/instinct
  - מיכאל (Michael): use/action oriented
  - חניאל (Haniel): emotion/aesthetic
  - רזיאל (Raziel): hidden connections/analogy
  - סנדלפון (Sandalphon): origin/source
- **Edge Types** = separate system, 15-25+ types, open
- Each angel uses **multiple edge types** via profile weights
  - Uriel profile: `{visual_color: 0.9, visual_shape: 0.8, visual_texture: 0.6}`
  - Raphael profile: `{taste: 0.9, healing_action: 0.7, bitter_sweet: 0.8}`

---

### Principle 2: Sefirot = 10-Dimensional Weight Vector (NOT switch)

**Broken assumption:** "10 sefirot = 10 discrete entry points" was false dichotomy.

**Why it broke:**
- Real queries mix types: "how to make lemonade?" = definition + decomposition + support
- Forcing ONE sefira excludes legitimate parallel processing

**Accepted structure:**
- Each query classified as **10D vector** of weights [0, 1]
- Query enters through **top-3 sefirot in parallel**, weighted
- Vector example: `{כתר: 0.0, חכמה: 0.7, בינה: 0.6, דעת: 0.5, חסד: 0.3, גבורה: 0.0, תפארת: 0.4, נצח: 0.2, הוד: 0.1, יסוד: 0.5, מלכות: 0.8}`
- Top-3 here: מלכות(0.8) + חכמה(0.7) + בינה(0.6) → parallel entries
- Results from 3 entries **merged** via weighted sum

---

### Principle 3: Partzufim = Pipeline with Parallel Stage + Feedback

**Broken assumption:** "5 Partzufim sequential pipeline" was LLM-style, not brain-style.

**Why it broke:**
- Brains parallel-process. Abba+Ima mating (per Ari'zal) is PARALLEL.
- Sequential pipeline = no feedback for failures

**Accepted structure:**
```
  Arich Anpin     (serial, first)       → goal extraction
       ↓
  Abba  +  Ima    (parallel, together)  → insight + decomposition
       ↓
  Zeir Anpin      (serial, integration) → processing with WM + emotion
       ↓
  Nukva           (serial, output)      → materialization

  Feedback: if ZA fails, return to Abba+Ima with error signal
```

**Note from AI consultation (Gemini Pro):** "Dogmatic top-down rigidity" warning. Response: we keep Partzufim as structure but **allow adaptation** — the pipeline is a *default*, not mandatory. If learning shows a different flow works better, adapt.

---

### Principle 4: Edge Fields = 5 Continuous Values (NOT 3)

**Broken assumption:** 3 fields (state, context, memory) missed crucial info.

**Why it broke:**
- Missing: confidence (how sure?)
- Missing: asymmetry (causal edges are directional)
- Missing: bidirectionality factor

**Accepted structure:**
```
Edge {
    // Identity
    src:               AtomId
    dst:               AtomId
    edge_type:         EdgeTypeId     // 15-25 open types
    
    // 5 Continuous values
    state_value:       f32  [-1, +1]  // connection strength + direction
    context_scope:     ContextId      // tree node, not float
    memory_strength:   f32  [0, 1]    // decays over time
    confidence:        f32  [0, 1]    // how certain
    asymmetry_factor:  f32  [0, 1]    // 0=bidirectional, 1=one-way
    
    // Provenance
    source:            SourceType     // user/inference/path_build/external
    created_at:        Timestamp
    last_reinforced:   Timestamp
    based_on:          Option<PathRef>  // if consolidation
    
    // Stats
    use_count:         u32
    success_count:     u32
    last_used:         Timestamp
}
```

---

### Principle 5: Context = Tree, NOT Scalar

**Broken assumption:** `context_scope` as single float (0=public, 1=personal) was too flat.

**Why it broke:**
- Real contexts are hierarchical: `personal.car.justy_1984` is inside `personal.car` is inside `personal`
- A single float can't express "specific to this car AND also applies to my cars generally"

**Accepted structure:**
```
ContextTree:
  root
  ├── personal
  │   ├── personal.family
  │   │   └── personal.family.sibling
  │   ├── personal.car
  │   │   └── personal.car.justy_1984
  │   └── personal.school.highschool
  ├── work
  │   └── work.company_car
  └── public
      └── public.common_knowledge
```

- Queries can filter by subtree: "only personal.*" or "personal.car.* union public.*"
- Decay rate is function of context depth: deeper personal contexts decay slower
- Scope hierarchy enables precise memory retrieval

---

### Principle 6: Decay & Reinforcement = Exponential (NOT Linear)

**Broken assumption:** "1% decay/day, 2% reinforcement, 10% bonus" — arbitrary linear.

**Why it broke:**
- 1% linear decay = half-life 69 days (too short for meaningful memories)
- 2% linear reinforcement = overflow above 1.0 after 35 uses
- Doesn't match biological Ebbinghaus curve

**Accepted structure (Ebbinghaus + SM-2 based):**

```rust
// Exponential decay
fn current_strength(edge: &Edge, now: Timestamp) -> f32 {
    let days_since = (now - edge.last_reinforced).days();
    let tau = 10.0                              // base: 10 days
            + 20.0 * edge.context_depth         // personal deeper = slower decay
            + 30.0 * (edge.use_count as f32).sqrt();  // used more = slower decay
    edge.memory_strength * (-days_since / tau).exp()
}

// Reinforcement on successful use
fn reinforce(edge: &mut Edge, success: bool) {
    let factor = if success { 0.15 } else { 0.05 };
    edge.memory_strength = (edge.memory_strength * (1.0 + factor)).min(1.0);
    edge.last_reinforced = now();
    edge.use_count += 1;
    if success { edge.success_count += 1; }
}
```

**Half-life examples (computed):**
- New public edge (context_depth=0, use=0): half-life ≈ **7 days**
- Personal edge (depth=2, use=10): half-life ≈ **130 days**
- Deep personal, heavily used (depth=3, use=50): half-life ≈ **1.5 years**
- Critical memory (depth=3, use=200): half-life ≈ **4 years**

This matches human memory curves.

---

### Principle 7: Storage Tiering = Never Delete, Always Tier

**Broken assumption:** "Keep everything in RAM" — doesn't scale.

**Why it broke:**
- 100M edges × 256 bytes = 25GB (fits RAM today)
- 10B edges × 256 bytes = 2.5TB (SSD only)
- 100B edges = 25TB (needs tiering)

**Accepted structure:**
```
HOT    (RAM, top 1% most active)
  - Loaded on startup, always accessible < 1ms
  - Based on memory_strength > 0.8 OR accessed in last 24h

WARM   (SSD fast, top 20% used last month)
  - mmap-backed, < 10ms access
  - Based on memory_strength > 0.3 OR accessed in last 30 days

COLD   (compressed SSD)
  - zstd-compressed, < 100ms access (decompress time)
  - Everything not in Hot/Warm

ARCHIVED  (tape/Glacier, only if memory_strength < 0.001 for 1+ year)
  - Exists but rarely loaded, > 1s access
  - Nothing deleted — just cold-stored
```

**Current state (Idan's instruction):** All on disk for now. SSD fast enough. Tiering when actual scale demands it. User count is zero, so no pressure.

---

## 🔬 AI Consultation Results (2026-04-24)

Raw responses stored in `docs/40_ai_consultations/20260424_agi_blueprint_consultation.json`.

### Summary of critiques:

**GPT-4o:**
- ✓ Generally accepts architecture
- ⚠ Sefirot 10D may be overcomplicating
- ⚠ Missing NLU layer for query understanding
- Storage: Recommends Neo4j/TigerGraph (we disagree — see below)

**Gemini 2.5 Pro (most critical):**
- ❌ "Dogmatic top-down rigidity" — warns against Kabbalah as hardcoded structure
- ❌ 7 angels as fixed set = "giving grandmaster only 7 moves"
- ❌ Missing: **agency and grounding** — no autonomous goals, no sensory input
- ✓ Storage: **RocksDB** (we agree)

**Gemini 2.5 Flash:**
- ❌ Partzufim as fixed pipeline = weakest point
- ❌ Missing explicit **learning mechanisms** beyond memory
- ❌ No abstraction/generalization/planning algorithms
- Storage: Neo4j or embedded RocksDB

### Our Response to Critiques

**On "Kabbalah rigidity":**  
Valid concern. But we treat Kabbalah as **validated pattern** (tested via 7-depth protocol), not as dogma. If data shows a better flow, we adapt. The 5 Partzufim is a **starting architecture**, not eternal law.

**On "missing learning/abstraction":**  
Accepted. **Adding** to blueprint: schema extraction, abstraction layer, generalization mechanism. These need Phase 2 development.

**On "missing agency":**  
Accepted as future Phase 3. Current Phase 1 is knowledge + retrieval + learning loop. Agency comes after foundation works.

**On "grounding":**  
Partial. ZETS has multidim sensory spaces (taste, color). But no live sensors. Phase 3 feature.

**Storage decision: RocksDB**
- Embedded (no separate server)
- Battle-tested
- Rust bindings excellent (`rust-rocksdb`)
- Scales to petabytes
- We don't need Cypher (we have our own query language)

---

## 📦 Data Model Summary

### Atom (node in graph)
```rust
struct Atom {
    id:             AtomId,
    lemma:          String,              // canonical form
    features:       HashMap<String, Value>,  // number, gender, tense, etc.
    
    // For concept nodes that are bipolar axes
    is_axis:        Option<BipolarAxis>, // if axis: {neg: "cold", pos: "hot"}
    
    // For concept nodes that are multidim spaces  
    is_multidim:    Option<MultidimSpace>, // if space: {dims: [...], interactions: ...}
    
    // Statistics
    in_degree:      u32,
    out_degree:     u32,
    created_at:     Timestamp,
}
```

### Edge (connection)
See Principle 4.

### Context (tree node)
```rust
struct Context {
    id:              ContextId,
    name:            String,     // "personal.car.justy_1984"
    parent:          Option<ContextId>,
    depth:           u32,        // used in decay formula
}
```

---

## 🌊 Query Flow

```
1. QUERY arrives ("what does lemon remind me of?")
   ↓
2. INTENT CLASSIFICATION (angels profile)
   → Determines which angels activate
   → Output: angels_weights = {Uriel: 0.8, Hanael: 0.7, Raziel: 0.9, ...}
   ↓
3. SEFIROT VECTOR COMPUTATION
   → Determines entry points
   → Output: {Chokhmah: 0.6, Binah: 0.5, Malkhut: 0.8}
   ↓
4. PARALLEL ENTRY
   → Enter graph via top-3 sefirot simultaneously
   ↓
5. ANGEL TRAVERSALS (7 in parallel, depth 7)
   → Each angel does graph walk weighted by its edge-type profile
   → Depth 7 default, measured empirically
   → Uses memory_strength × state_value × confidence for weighting
   ↓
6. SYNTHESIS (Abba + Ima partzufim)
   → Abba: find insight patterns across results
   → Ima: decompose into structured answer
   → Parallel output merged
   ↓
7. INTEGRATION (Zeir Anpin partzuf)
   → Working memory + emotional context
   → Check consistency
   → If fails → feedback loop to Abba+Ima
   ↓
8. OUTPUT (Nukva partzuf)
   → Generate response
   → Apply energy modulation from angels (×φ or ×1/φ for emphasis)
   ↓
9. LEARNING (subconscious)
   → Store: query + path_taken + answer + user_reaction
   → Reinforce edges on successful path
   → If path repeats ≥5 times, create shortcut (path_building)
   ↓
10. NIGHTMODE (background, not during query)
    → Consolidation: move hot → warm → cold based on stats
    → Schema extraction: common patterns become templates
    → Decay: apply time decay to all edges
```

---

## 🎯 Phase Plan

### Phase 1 (NOW): Foundation
- [x] Revise principles (this document)
- [ ] Implement Atom + Edge + Context structures (Rust)
- [ ] Build RocksDB storage layer
- [ ] Implement 7 angels traversal
- [ ] Implement 10D sefirot vector classifier
- [ ] Implement decay/reinforcement formulas
- [ ] Test on "fruits & vegetables" corpus (50-100 concepts)

### Phase 2: Learning
- Schema extraction from repeated patterns
- Abstraction layer (generalize from instances)
- Path building algorithm
- NightMode consolidation

### Phase 3: Agency (future)
- Autonomous goal setting
- Sensory grounding (if applicable)
- Self-modification guardrails

---

## 🔄 Revision History

| Date | Change | Reason |
|------|--------|--------|
| 2026-04-24 | Initial v1 | Post breaking-the-tools + AI consultations |

---

## ✅ Acceptance Status

- [x] Principle 1 (Angels as directions) — **validated** via 7-depth protocol
- [x] Principle 2 (Sefirot as vector) — **agreed** with Idan
- [x] Principle 3 (Partzufim parallel) — **validated** via Ari'zal references  
- [x] Principle 4 (5 edge fields) — **agreed** with Idan
- [x] Principle 5 (Context tree) — **agreed** with Idan
- [x] Principle 6 (Exponential decay) — **computed** with biological match
- [x] Principle 7 (Storage tiering) — **deferred** per Idan's instruction (disk-only for now)

---

## 🖼️ Diagram Prompt for ChatGPT-5 Image Generator

The following prompt should produce a technical diagram accurately representing this architecture. Paste it into ChatGPT with image generation:

```
Create a technical architecture diagram showing a knowledge graph AI system
("ZETS"). It should contain these elements clearly labeled in English:

TOP SECTION — Intent Layer:
- "7 Angels (Traversal Directions)" — row of 7 circles labeled Uriel, Raphael, 
  Gabriel, Michael, Haniel, Raziel, Sandalphon with arrows pointing DOWN
- Each angel shows a small "profile" badge listing 2-3 edge types

MIDDLE-TOP — Query Router:
- "10D Sefirot Vector" — a horizontal bar chart with 10 bars of varying heights,
  labeled Keter, Chokhmah, Binah, Da'at, Chesed, Gevurah, Tiferet, Netzach, 
  Hod, Yesod, Malkhut
- An arrow from the bar chart down, labeled "Top-3 parallel entry points"

CENTER — Graph Core:
- A rich knowledge graph of interconnected nodes (atoms)
- Central atom labeled "LEMON" with 15+ edges radiating to concepts:
  YELLOW, SOUR, CITRUS_FRUIT, LEMONADE, TEA_WITH_LEMON, SPHERE_SHAPE, 
  MEDITERRANEAN, SUMMER, LIME, GRAPEFRUIT
- Each edge shows 5 tiny icons representing: state_value, memory_strength,
  confidence, asymmetry, context_scope
- Some edges drawn THICKER (higher memory_strength), some DASHED (lower strength)
- A secondary hub for "1984 SUBARU JUSTY" with edges to: YELLOW, 1984, 
  HIGHSCHOOL_MEMORY, FAMILY_CONTEXT, BROTHER
- LEMON and JUSTY connected via ONE dashed shortcut labeled "path-built"

RIGHT SIDE — Partzufim Pipeline:
- Vertical boxes stacked:
  1. ARICH ANPIN (Goal extraction)
  2. ABBA + IMA (in parallel — shown as two boxes side by side) 
     ABBA=Flash Insight, IMA=Structured Decomposition
  3. ZEIR ANPIN (Integration with WM)
  4. NUKVA (Output)
- Arrows between stages, plus a curved FEEDBACK arrow from ZA back to ABBA+IMA

LEFT SIDE — Context Tree:
- A tree diagram:
  root → personal → personal.car → personal.car.justy_1984
  root → personal → personal.family → personal.family.sibling
  root → work
  root → public

BOTTOM — Storage Tiers:
- Four horizontal bars:
  HOT (RAM) — red/orange, labeled "top 1%"
  WARM (SSD mmap) — yellow, labeled "top 20%"  
  COLD (compressed) — blue, labeled "rest"
  ARCHIVED — gray, labeled "never deleted"

BOTTOM-RIGHT — Learning Loop:
- Small cycle diagram:
  QUERY → PATH → ANSWER → SUCCESS? → REINFORCE → (back to start)
- Separate cycle for NIGHTMODE:
  DECAY → CONSOLIDATE → EXTRACT_SCHEMAS

STYLE:
- Dark navy background with subtle grid
- Cyan/blue accents for active elements
- Yellow highlights for memory-strength indicators  
- Connections should look like synapses (glowing lines)
- Clean, technical, NOT cartoonish
- English text only (no Hebrew — avoid confusion)
- NO buzzwords like "AGI", "AI", "CORE CPU" — just the real components
- NO brain icons — this is abstract graph architecture
```

---

## 📎 References

- `docs/00_doctrine/20260424_brain_architecture_facts.md` — validated facts from previous work
- `docs/20_research/20260424_brain_to_zets_complete.md` — 14 AGI capabilities mapping
- `docs/40_ai_consultations/20260424_agi_blueprint_consultation.json` — today's AI responses
- `sim/brain_v4/seven_angels_dive.py` — empirical validation of 7 angels approach


---

## 📎 Addendum: Revisions from Idan's Questions (later 2026-04-24)

### Question 1: Edge directionality — bidirectional or unidirectional?

**Answer: Unidirectional storage, bidirectional indexing.**

- Edges stored **once** (src → dst) to avoid 2× memory + sync issues
- Two indexes maintained:
  - **Forward index:** `src → [edges]` (who does this atom connect to?)
  - **Reverse index:** `dst → [edges]` (who points to this atom?)
- The existing `asymmetry_factor` field (0-1) captures whether the relationship is truly symmetric or directional
  - 0.0 = equivalent in both directions (e.g., אבא ↔ אמא as family members)
  - 1.0 = fully one-way (e.g., אש → חום: fire causes heat, not vice versa)
  - Intermediate = partial asymmetry (e.g., לימון → צהוב: lemon implies yellow, yellow doesn't imply lemon)

**Implementation note:** In RocksDB, this is two column families or two key prefixes:
- `fwd:{src_id}:{edge_id}` → edge data
- `rev:{dst_id}:{edge_id}` → reference to forward key

---

### Question 2: State-dependent relationships (lemon green → yellow as it ripens)

This exposed a gap in the original blueprint: not all edges are binary-true. Some depend on the **state** of the concept.

### Principle 8 (NEW): Edge States — 4 Relationship Types

Edges live in 4 distinct states, each requiring different modeling:

| Type | Example | Active When | Representation |
|------|---------|-------------|----------------|
| **1. Static Permanent** | לימון → חמוץ | Always | Regular edge, no state dependency |
| **2. Static Default** | לימון → צהוב (when ripe) | Default state | Edge with `state_dependency: peak_range` |
| **3. Dynamic on State** | לימון לא-בשל → ירוק | State axis in range | Edge with `state_dependency: active_range` |
| **4. Temporal Transition** | ירוק → צהוב over time | Evolving state | Edge with `temporal_transition` metadata |

### State Axes per Concept

Each concept can have **multiple state axes** that modulate its edges:

```rust
struct StateAxis {
    id:         StateAxisId,
    name:       String,        // "ripeness", "freshness", "age", "season"
    range:      (f32, f32),    // typically (0.0, 1.0)
    default:    f32,           // assumed value if not specified
    description: String,
}
```

Example for לימון (lemon):

```
Concept: לימון
  State axes:
    - ripeness  [0, 1]  default=0.9  ("בשל" by default when we say "lemon")
    - freshness [0, 1]  default=0.8  
    - season    cyclic  (no default)
  
  Edges:
    [taste]         → חמוץ          (no state dep)           
    [category]      → הדר           (no state dep)
    [color]         → ירוק          (ripeness < 0.4)           TYPE 3
    [color]         → צהוב          (ripeness > 0.6)           TYPE 2 (default)
    [texture]       → קשה           (ripeness < 0.3)           TYPE 3
    [texture]       → רך            (ripeness > 0.7)           TYPE 3
    [surface]       → מבריק         (freshness > 0.7)          TYPE 3
    [moisture]      → עסיסי         (freshness > 0.5)          TYPE 3
    [transition]    ירוק→צהוב       (ripeness over ~30 days)  TYPE 4
```

### StateDependency Structure

```rust
struct StateDependency {
    axis:          StateAxisRef,
    active_range:  (f32, f32),        // in what range edge is valid
    peak_value:    f32,                // max strength in active range
    curve:         CurveType,          // how strength varies
}

enum CurveType {
    Constant,                          // uniform in range
    Linear { start: f32, end: f32 },   // linear increase/decrease
    Bell { center: f32, width: f32 },  // peak at center
    Sigmoid { midpoint: f32, slope: f32 }, // smooth transition
}
```

### Query Behavior with State Dependencies

When asked "what color is a lemon?":
1. Find all `[color]` edges from `לימון`
2. Filter by active state (use default if unspecified):
   - `ripeness = 0.9` (default)
3. With default: `ירוק` inactive (0.9 > 0.4), `צהוב` active (0.9 > 0.6) → **answer: צהוב**
4. If context says "unripe lemon": `ripeness = 0.2` → `ירוק` active → answer: **ירוק**

### Temporal Transitions (Type 4)

```rust
struct TemporalTransition {
    from_state:       StateValue,      // {axis: ripeness, value: 0.3}
    to_state:         StateValue,      // {axis: ripeness, value: 0.9}
    typical_duration: Duration,         // 30 days, ±10 days
    trigger:          TransitionTrigger,
}

enum TransitionTrigger {
    TimeElapsed,
    Environmental(String),   // "sunlight", "cold", "water"
    Event(String),           // "picked", "cooked", "damaged"
    Threshold(StateAxisRef, f32),
}
```

This enables queries like:
- "How will the lemon change in 10 days?"
- "What causes a lemon to ripen?"
- "How long until it ripens?"

### Rationale

Previous model assumed "an edge is either true or false". Reality:
- **Lemons ARE yellow** (when ripe — the default when unspecified)
- **Lemons ARE green** (before ripening)
- **Both statements are true simultaneously** — just at different state values

Without state axes, we'd have to choose one and be wrong half the time, OR store contradicting edges without resolution.

---

## 🔄 Revision History (updated)

| Date | Change | Reason |
|------|--------|--------|
| 2026-04-24 | Initial v1 | Post breaking-the-tools + AI consultations |
| 2026-04-24 | Added Principle 8 (Edge States) + directionality clarification | Idan's questions exposed gap: static-vs-dynamic edges, unidirectional storage |

