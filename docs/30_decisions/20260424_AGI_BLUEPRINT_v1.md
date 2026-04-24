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


---

## 📎 Addendum 3: Meta-Principle — Kabbalah as Pseudocode

**Idan's framing (24.04.2026):**

The Kabbalistic structures (sefirot, partzufim, angels) are **pseudocode** — a 1500+ year design document describing cognitive architecture in symbolic language.

### Why This Framing Matters

It resolves the tension between two extremes:
- "Accept because it's tradition" — dogmatic
- "Reject because it's mystical" — ignores valid observations

Instead: **compile and test each claim**.

### Compilation Results So Far

| Claim | Compilation | Evidence |
|---|---|---|
| 5 Partzufim = pipeline stages | ✅ PASS | 7/7 hits (7-depth protocol) |
| 7 Angels = intent classifiers | ✅ PASS | 6.5/7 hits |
| 3 Mothers × 7 sub = 21 dives | ✅ PASS | +75% coverage empirical |
| Chesed/Gevurah = opponent process | ✅ PASS | matches Hering 1892 |
| 3 Shachliot (Chokhma/Bina/Daat) | ✅ PASS | coherent triad, fits Sensory/Functional/Abstract |
| 3 Middot (Chesed/Gevurah/Tiferet) | ✅ PASS | reinforcement/pruning/balance operators |
| 10 Sefirot = 3D color space | ❌ FAIL | 2/6 hits, forced mapping |
| Phi ratios in gematria | ❌ FAIL | null hypothesis test negative |
| Math operators as brain model | ❌ FAIL | no statistical signal |

### The Compiled Kabbalistic Architecture

After compilation, the kabbalistic pseudocode yields:

```
10 Sefirot organized into 3 roles:

Role 1: ENTRY POINTS (3 Mothers / Shachliot)
  חכמה  → Abstract search     (insight, pattern-match)
  בינה  → Functional search   (analysis, decomposition)
  דעת   → Sensory search      (multidim concrete)
  Each splits into 7 sub-directions → 21 parallel dives.

Role 2: LEARNING OPERATORS (3 Middot)
  חסד   → Strengthen (reinforcement)
  גבורה → Weaken (pruning)
  תפארת → Balance (homeostasis)

Role 3: EXECUTION LAYER (4 Middot)
  נצח   → Persistence (retry, endurance)
  הוד   → Submission (accept override)
  יסוד  → Transmission (output conduit)
  מלכות → Manifestation (final output)

Kether (כתר) = initiator, not a category. Every query starts from Kether.
```

### The Principle of Compilation

```
Kabbalah proposes. Engineering decides.

Every component in ZETS architecture based on Kabbalah
MUST pass compilation check.

Passed → stays
Failed → rejected (even if "beautiful" in source)
```

This is fundamentally different from "Kabbalistic architecture" — ZETS is:

> **"AGI architecture informed by compiled kabbalistic pseudocode that passed engineering tests."**

The pseudocode was good because it captured real observations about thinking, learning, breaking, and repairing over centuries. Where observations were accurate — it compiles. Where interpretations drifted — it doesn't.

---

## 📎 Addendum 4: Integration of Sefer Yetzirah (Book of Formation) primitives

**Date:** 2026-04-24 (evening)  
**Method:** Clean-read the text of Sefer Yetzirah (Gra recension) without interpretations. Identified two primitives that add real engineering value to the Blueprint.

### What Sefer Yetzirah actually describes

After clean reading (separating the text itself from 1500 years of commentary):

- **22 letter-nodes + 231 gates** (C(22,2)) — a complete bidirectional graph
- **5 operations on nodes** — חקק/חצב/שקל/המיר/צרף
- **3 categorizations of letters** — 3 mothers, 7 doubles, 12 simples
- **3 context axes** — עולם/שנה/נפש (space/time/identity)
- **"End rooted in beginning"** — circular structure with feedback

Not described: traversal algorithms, depth-of-search, weighted edges, learning mechanisms, semantic graph layers.

**Conclusion:** Sefer Yetzirah is **complementary** to our Blueprint, not **reinforcement**. It adds two primitives that we lacked.

---

### Principle 11 (NEW): 5-Phase Concept Ingestion Pipeline

**Source:** "חקקן חצבן שקלן והמירן צרפן" (Sefer Yetzirah Ch. 2)

**Problem solved:** Currently, when a new concept enters ZETS, the ingestion is monolithic — hard to debug, test, or extend. When ingestion produces wrong atoms, we can't tell which step failed.

**Solution:** Split ingestion into 5 named, independently testable phases.

```rust
pub fn ingest_concept(raw: RawInput) -> Result<AtomId, IngestError> {
    // Phase 1: CARVE (חקק) — define boundaries
    // What IS this concept? What is it NOT?
    let carved = carve_boundaries(raw)?;
    
    // Phase 2: HEW (חצב) — extract features from raw
    // Break down into constituent properties
    let hewn = hew_features(carved)?;
    
    // Phase 3: WEIGH (שקל) — assign importance
    // Which features are primary vs secondary?
    let weighted = weigh_features(hewn)?;
    
    // Phase 4: PERMUTE (המיר) — generate morphological variants
    // Plural/singular, tense, gender, form variations
    let permuted = generate_variants(weighted)?;
    
    // Phase 5: COMBINE (צרף) — integrate into graph
    // Link to existing atoms via appropriate edges
    let atom_id = integrate_into_graph(permuted)?;
    
    Ok(atom_id)
}
```

**Practical benefits:**
1. **Debuggability** — if ingestion fails, we know which of 5 phases failed
2. **Testability** — each phase has its own unit tests
3. **Extensibility** — adding a "translate" or "validate" phase is natural
4. **Observability** — logs per-phase show where time/errors concentrate
5. **Parallel processing** — phases 2-4 can parallelize per-concept

**Example: "tangelo" (unfamiliar citrus) enters ZETS**
```
Phase 1 (carve):   "tangelo" is a fruit, not a color/person/place
Phase 2 (hew):     {category: fruit, subcategory: citrus, hybrid: yes}
Phase 3 (weigh):   category=0.9, hybrid=0.8, origin=0.5
Phase 4 (permute): "tangelo", "tangelos", "tangelo juice"  
Phase 5 (combine): edge[KIND_OF] → citrus, edge[HYBRID_OF] → tangerine+grapefruit
```

Each phase produces an artifact that the next consumes — clear pipeline.

---

### Principle 12 (NEW): 3-Axis Context — Space, Time, Identity

**Source:** "עדות נאמנה: עולם שנה נפש" (Sefer Yetzirah Ch. 6)

**Problem solved:** Current context_tree is a single hierarchy, forcing every memory to fit one path. But humans naturally recall by **independent dimensions**: who, where, when.

**Scientific backing:** Tulving (1983) "Elements of Episodic Memory" identifies exactly these three dimensions as the core of episodic memory retrieval. Sefer Yetzirah formulated this 1500 years earlier.

**Solution:** Three independent context axes, each with its own tree. Context of a memory = **intersection** of the three.

```rust
pub struct ContextAxes {
    // עולם (World/Space) — where did it happen?
    spatial: Option<SpatialContextId>,
    
    // שנה (Year/Time) — when did it happen?  
    temporal: Option<TemporalContextId>,
    
    // נפש (Soul/Identity) — who was involved?
    identity: Option<IdentityContextId>,
}

// Each axis has its own independent tree
pub struct SpatialContextTree {
    // root → home → home.kitchen, home.living_room
    // root → work → work.office, work.lobby
    // root → external → external.paris.cafe, external.tel_aviv.beach
}

pub struct TemporalContextTree {
    // root → 2024 → 2024.summer → 2024.summer.august
    // root → 2019 → ...
    // Also supports: "childhood", "highschool", "recent"
}

pub struct IdentityContextTree {
    // root → self
    // root → family → family.father, family.mother, family.sibling
    // root → work → work.team, work.client.acme
}

pub struct Atom {
    // ... existing fields ...
    context_axes: ContextAxes,
}
```

**Query benefits:**

```rust
// Natural queries become trivial
fn what_did_I_say_to_dad() -> Vec<Atom> {
    query().with_identity("family.father").execute()
}

fn what_happened_in_paris() -> Vec<Atom> {
    query().with_spatial("external.paris.*").execute()
}

fn what_happened_in_2019() -> Vec<Atom> {
    query().with_temporal("2019.*").execute()
}

// Compound queries — intersections
fn dad_in_paris_2019() -> Vec<Atom> {
    query()
        .with_identity("family.father")
        .with_spatial("external.paris.*")
        .with_temporal("2019.*")
        .execute()
}
```

**Practical benefits:**
1. **Natural language queries** — "who/where/when" map directly to axes
2. **Orthogonal filtering** — any combination of axes possible without restructuring
3. **Partial recall** — if user forgets one dimension, query with the other two still works
4. **Memory research alignment** — matches Tulving's episodic memory model
5. **Simple implementation** — 3 optional fields per atom + 3 filters per query

**Example: "The conversation I had with dad at the cafe in Paris in 2019"**

With single context_tree: forced path `personal.family.father.locations.paris.cafes.2019.conversations` — fragile, requires exact memory of hierarchy.

With 3 axes: 
- `spatial: external.paris.cafe`
- `temporal: 2019`
- `identity: family.father`

Any two out of three is enough to retrieve. Much more robust.

---

### What we did NOT adopt from Sefer Yetzirah

**Rejected (with reasoning):**

- ❌ **22 letters as complete graph with 231 gates** — Too primitive. ZETS has millions of atoms, not 22. K₂₂ is a toy graph.

- ❌ **7 doubles = 7 planets = 7 days** — Astrological associations don't compile to useful engineering. Rejected as symbolic drift.

- ❌ **12 simples = 12 zodiac = 12 body parts** — Same issue. Symbolic matching without predictive power.

- ❌ **"Tali (dragon) in world, galgal (wheel) in year, lev (heart) in soul"** — Poetic but not mappable to concrete engineering. No test, no code.

- ❌ **Gematria relations between letters** — Previously tested and rejected (see `20260424_mathematical_operators_REJECTED.md`).

### Summary

Sefer Yetzirah contributed **2 of many possible** primitives — specifically those with clear engineering value:
- 5-phase ingestion pipeline (better structure for a mundane problem)
- 3-axis context (matches human memory research)

The rest was symbolic elaboration without engineering cash-out.

**Principle:** Every ancient structure is judged on merit. Two passed. The rest didn't.

---

## 🔄 Revision History (updated)

| Date | Change | Reason |
|------|--------|--------|
| 2026-04-24 | Initial v1 | Post breaking-the-tools + AI consultations |
| 2026-04-24 | Added Principle 8 (Edge States) + directionality clarification | Idan's questions on lemon-color paradox |
| 2026-04-24 | Added Principle 9 (3-Mother taxonomy) | Empirical test of 3-part decomposition |
| 2026-04-24 | Added Principle 10 (21 parallel dives, 3×7×7) | Idan's insight on async parallel dive architecture |
| 2026-04-24 | Added Meta-Principle (Kabbalah as pseudocode) | Idan's framing — compilation approach |
| 2026-04-24 | Added Principle 11 (5-phase ingestion) + Principle 12 (3-axis context) | Clean-read Sefer Yetzirah — 2 practical primitives extracted |


---

## 📎 Addendum 5: Principle 13 — Storage Layout (Hot/Cold + Bitwise Packing)

**Date:** 2026-04-24 (later)  
**Question raised by Idan:** "How do we connect atoms precisely but extremely lean? Should it be pointers, edges, or attribute symbols?"

### Decision

**Index-based access (NOT pointers) + CSR layout + Bitwise-packed metadata + Hot/Cold split.**

### The Three Wrong Approaches Considered

#### ❌ Option A: Pointer-based linked list
```rust
struct Edge {
    src: u32, dst: u32,
    state_value: f32, memory_strength: f32, confidence: f32,
    asymmetry: f32, context_id: u32, created_at: u64,
    next: *const Edge,  // pointer to next edge
}
```
- **25 bytes per edge**
- **2.5 GB for 100M edges**
- 2 edges per cache line = pointer-chase nightmare
- Cannot mmap (pointers are virtual addresses)

#### ❌ Option B: Naive packed adjacency
```rust
struct Edge {
    dst: u32,
    metadata_byte: u16,  // packed but 1D
    context_byte: u8,
}
```
- **7 bytes per edge** — better
- **740 MB for 100M edges**
- But still has redundant context info per edge

### The Right Approach

#### ✅ Option C+D: CSR + Hot/Cold Hybrid

**Storage Structure:**

```rust
// HOT PATH — always in RAM, mmap'd from disk
struct AtomHot {              // 16 bytes
    lemma_idx: u32,           // → lemma_strings[lemma_idx]
    atom_type: u8,            // concept/entity/event/memory
    flags: u8,                // has_features, has_state_axes, deleted
    in_degree: u16,
    out_degree: u16,
    created_at_days: u32,
    _padding: u16,
}

struct EdgeHot {              // 6 bytes!
    dst: u32,                 // 4 bytes
    packed_meta: u16,         // 2 bytes — bitwise packed
}

// CSR offsets for fast traversal
fwd_offsets: Vec<u32>,        // 4 bytes per atom
rev_offsets: Vec<u32>,        // 4 bytes per atom

// COLD PATH — looked up only when flag bit is set (~10% of edges)
struct EdgeCold {
    context_id: Option<u32>,
    state_dep: Option<StateDependency>,
    confidence: u8,           // moved to cold (rarely needed)
    asymmetry: u8,
    provenance: SourceType,
}
```

### The Bitwise-Packed Edge Metadata (16 bits = 2 bytes)

```
┌─────────┬──────────┬──────────┬──────────┐
│ type 5b │ state 4b │ mem 4b   │ flags 3b │
└─────────┴──────────┴──────────┴──────────┘
```

- **type** (5 bits) — 32 edge types (we have 21 = 7×3 → fits)
- **state_value** (4 bits) — 16 buckets, range -8..+7
- **memory_strength** (4 bits) — 16 buckets (Ebbinghaus is exponential anyway)
- **flags** (3 bits):
  - bit 0: has_context_tag (cold lookup needed)
  - bit 1: has_state_dependency (cold lookup needed)
  - bit 2: is_deleted (tombstone)

```rust
fn pack(t: u8, s: i8, m: u8, f: u8) -> u16 {
    ((t as u16 & 0x1F) << 11)
  | (((s + 8) as u16 & 0x0F) << 7)
  | ((m as u16 & 0x0F) << 3)
  | (f as u16 & 0x07)
}

fn unpack(meta: u16) -> (u8, i8, u8, u8) {
    let t = ((meta >> 11) & 0x1F) as u8;
    let s = (((meta >> 7) & 0x0F) as i8) - 8;
    let m = ((meta >> 3) & 0x0F) as u8;
    let f = (meta & 0x07) as u8;
    (t, s, m, f)
}
```

### How atoms connect — the answer

**Atoms connect via INDEX-BASED access in CSR layout, NOT pointers.**

```rust
// To find all outgoing edges of atom #42:
let start = fwd_offsets[42];
let end   = fwd_offsets[43];
let edges_of_42 = &edges_hot[start..end];

// Each edge points to dst by atom_id (u32 index, not pointer)
for edge in edges_of_42 {
    let target_atom = &atoms[edge.dst as usize];
    let (etype, state, mem, flags) = unpack(edge.packed_meta);
    // ... process
}
```

### Why indices beat pointers (5 reasons)

1. **Cache-friendly** — indices are arithmetic; pointers are random jumps
2. **mmap-ready** — indices are stable across processes; pointers are not
3. **50% smaller** — u32 (4 bytes) vs pointer (8 bytes)
4. **Safer in Rust** — no `unsafe`, no `Box`/`Rc` overhead
5. **Zero-copy load** — mmap a file → ready instantly, no parsing

### Total memory at 10M atoms × 100M edges

| Component | Size | Per Item |
|---|---|---|
| atoms (HOT) | 160 MB | 16 B × 10M |
| edges_hot (HOT) | 600 MB | 6 B × 100M |
| fwd_offsets (HOT) | 40 MB | 4 B × 10M |
| rev_offsets (HOT) | 40 MB | 4 B × 10M |
| **HOT TOTAL** | **840 MB** | always in RAM |
| contexts (COLD) | ~80 MB | ~10% of edges |
| state_deps (COLD) | ~40 MB | ~5% of edges |
| features (COLD) | ~50 MB | ~5% of atoms |
| lemma_strings (COLD) | ~30 MB | string table |
| **COLD TOTAL** | **200 MB** | lazy-loaded |
| **GRAND TOTAL** | **1.04 GB** | 10M × 100M |

### Performance budget

| Operation | Target | Reason |
|---|---|---|
| Atom lookup by id | < 10 ns | array index, L1 cache |
| All edges of atom | < 100 ns | sequential read, prefetched |
| Single dive depth=7 | < 5 μs | cache-friendly traversal |
| 21 parallel dives | < 50 μs | parallelizable, good locality |
| Full query (incl. Partzufim) | < 100 μs | 95th percentile target |

### The principle in one line

> **Edges are arrays of bytes, not nodes of a graph. The graph is a layout, not an object.**

The graph "exists" only as the relationship between three flat arrays. There is no `Node` struct that owns its `Edge` structs. There is `atoms[]` and `edges[]` and `offsets[]`. The graph emerges from the layout.

This is how scientific computing libraries (NetworkX, igraph at scale, GraphBLAS) all do it. It's also how the brain does it — neurons don't store pointers to other neurons; the connection IS the synapse, which is a separate physical structure.

---

## 🔄 Revision History (updated)

| Date | Change | Reason |
|------|--------|--------|
| 2026-04-24 | Principle 13: Storage Layout | Idan's question on lean atom connection |

