# ZETS System Overview — Full Inventory + Self-Improvement Architecture

**תאריך:** 23.04.2026
**מצב:** 36,245 שורות Rust, 677 tests passing, 17.5M Wikipedia articles 48 langs
**Commit אחרון:** 77edc73

---

# PART A: סקירה מלאה — מה יש ביד

## 1. Rust core (100+ מודולים, 36,245 שורות)

### שכבת נתונים (Data layer)
| מודול | תפקיד | שורות |
|-------|-------|-------|
| `atoms.rs` | AtomStore + AtomEdge (from,to,relation:u8,weight:u8,slot:u16) | 547 |
| `piece_graph.rs` | Pieces (strings) content-addressed | 497 |
| `sense_graph.rs` | WordNet synsets — שלום≠hello fix | 380 |
| `bitflag_edge.rs` | BitflagRelation (6 axes in 14 bits) | 330 |
| `path_mining.rs` | Motif mining — articles as paths | 480 |
| `hash_registry.rs` | FNV-1a content-hash index | — |

### שכבת אחסון (Storage layer)
| מודול | תפקיד |
|-------|-------|
| `mmap_core.rs` | Current flat mmap (3 sections: lang/pieces/concepts) |
| `mmap_lang.rs` | Per-language mmap packs |
| `mtreemap.rs` | **NEW: Cluster-tree mmap layout (98% cache hit vs 17%)** |
| `pack.rs` / `atom_persist.rs` | Persistent atom/edge storage |
| `wal.rs` | Write-ahead log (append-only, LSM-style) |
| `crypto.rs` / `encrypted_installer.rs` | AES-GCM encryption |

### שכבת שפה (Language layer)
| מודול | תפקיד |
|-------|-------|
| `morphology/core.rs` | Core morphology engine |
| `morphology/families.rs` | Language families (Semitic, IE, CJK, etc) |
| `morphology/languages.rs` | Per-language rules |
| `morphology/registry.rs` + `rules.rs` | Rule registration + dispatch |

### שכבת חשיבה (Cognitive layer)
| מודול | תפקיד |
|-------|-------|
| `cognitive.rs` | 16 CognitiveKinds (semantic/episodic/procedural/...) |
| `cognitive_modes.rs` | Switching between cognitive modes |
| `brain_profile.rs` | Accessibility profiles (age/impairment) |
| `dreaming.rs` | Offline consolidation |
| `spreading_activation.rs` | Hebbian-style activation spread |
| `hopfield.rs` | Hopfield network for pattern recall |
| `smart_walk.rs` | Intelligent graph walk |
| `metacognition.rs` | Meta-reasoning (thinking about thinking) |

### שכבת למידה (Learning layer)
| מודול | תפקיד |
|-------|-------|
| `learning_layer.rs` | Core learning mechanisms |
| `meta_learning.rs` | Learning how to learn |
| `distillation.rs` | Compress knowledge into smaller forms |
| `ingestion.rs` | Parse raw content into atoms |
| `corpus_acquisition.rs` | Fetch external corpora |
| `edge_extraction.rs` | Extract edges from text |
| `bootstrap.rs` | Initial seed |

### שכבת חיפוש (Search layer)
| מודול | תפקיד |
|-------|-------|
| `search/mod.rs` | Public API (MAX_BEAM=64, MAX_DEPTH=16) |
| `search/strategy.rs` | **7 behavioral strategies** (Precise/Exploratory/Exhaustive/RapidIteration/DeepDive/Standard7x7/**PolymathWeaving**) |
| `search/persona.rs` | PersonaGraph → strategy mapping |
| `search/beam.rs` | Parallelized beam search (30-550× vs sequential) |
| `search/cancel.rs` | Cooperative cancellation token |

### שכבת דחיסה (Fold layer)
| מודול | תפקיד |
|-------|-------|
| `fold/mod.rs` | Public API |
| `fold/bpe.rs` | BPE incremental (7.4× faster than naive) |
| `fold/vocab.rs` + `vocab_index.rs` | Vocabulary with hash consing |
| `fold/merkle.rs` | SHA-256 Merkle IDs |
| `fold/normalize.rs` | Pre-hash normalization |
| `fold/walk.rs` | Lossless roundtrip unfold |
| `fold/tier.rs` | Hot/cold tiering |
| `fold/background.rs` | WAL-triggered compaction |
| `fold/edges/` | 4-layer edge compression (adjacency/huffman/pattern/weight) |

### **⭐ שכבת הsystem_graph — התשובה העיקרית לשאלה שלך**
| מודול | תפקיד | שורות |
|-------|-------|-------|
| `system_graph/mod.rs` | Entry — "homoiconic core" | 37 |
| `system_graph/opcodes.rs` | **32 opcodes**: Flow/Stack/Graph/Morph/List/Math/Logic/Persist | 150 |
| `system_graph/value.rs` | VM values (int/bool/atom_id/string) | 76 |
| `system_graph/routes.rs` | Route struct + Tier (Hot/Warm/Cold/Archive) | 124 |
| `system_graph/vm.rs` | Bounded VM (max_call_depth=32, max_ops=10K, 16 registers) | 674 |
| `system_graph/bootstrap.rs` | 3 initial routes: R_EXTRACT_HEARST_X_IS_A_Y, R_LEARN_FROM_DEFINITION, R_MORPH_LOOKUP_FALLBACK | 213 |
| `system_graph/reasoning.rs` | R_IS_ANCESTOR + other reasoning routes | 230 |
| `system_graph/graph.rs` | SystemGraph container | 146 |

**זה בדיוק הרעיון שתיארת** — procedures as graph data, VM בגרף, bounded execution. **35 tests עוברים**.

### שכבות נוספות
- **Inference**: `inference.rs`, `explain.rs`, `verify.rs`, `scenario.rs`, `planner.rs`
- **Dialogue**: `dialogue.rs`, `persona.rs`, `session.rs`
- **LLM adapter**: `llm_adapter.rs`, `gemini_http.rs`
- **Network**: `http_server.rs`, `scopes/` (router + registry)
- **Safety**: `ethics_core.rs`, `testing_sandbox.rs`
- **Ops**: `skills.rs`, `prototype.rs`, `benchmarks.rs`, `distillation.rs`

## 2. Binaries (43 demo + tools)

- **Autonomous/AGI**: `agi_demo`, `autonomous_demo`, `live_brain_demo`
- **Brain demos**: `brain_demo`, `cognitive_demo`, `dialogue_demo`, `planner_demo`, `pattern_demo`
- **Ingest**: `ingest_corpus`, `stream_ingest`, `wiki_bridge`, `populate_edges`
- **Benchmark**: `benchmark_runner`, `run_benchmark`, `measure_moats`, `scale_test`, `fold_benchmark`
- **Content seeding**: `seed_programming_taxonomy`, `build_persona_atoms`
- **Tooling**: `pack_read/write/inventory`, `mmap_read`, `check_edges`
- **Server**: `zets_client_server`
- **Sandbox/verify**: `verify_demo`, `explain_demo`, `inference_demo`

## 3. Services running ("Clients" בלשונך)

| Port | Service | Role |
|------|---------|------|
| 3141 | Cortex gateway (old) | legacy |
| 3143 | Deploy agent | CI/CD hook |
| 3144-3145 | ZETS MCP / HTTP API | main APIs |
| 3147 | ZETS HTTP (`zets_http_api.py`) | public API |
| 3149 | Lev (legacy) | older knowledge |
| **3251-3266** | **16 ZETS persona clients** (`zets_client.py serve`) | each persona has own port |
| 11434 | Ollama | local LLM |

**16 persona clients** = 16 זהויות נפרדות, כל אחת עם port נפרד. 

## 4. Python — מה נשאר ולמה

### Python **production code** (צריך להחליף ל-Rust):
```
mcp/zets_http_api.py           ← HTTP endpoint - יש src/http_server.rs
mcp/zets_mcp_server.py         ← MCP wrapper
mcp/zets_client.py             ← 16 persona clients
mcp/personas.py + build_persona_snapshots.py
mcp/multi_client.py / v2 / v3  ← persona coordinator
mcp/autonomous/*.py            ← 7 scripts (crawler/learner/scorer/ingestor)
```

### Python **scripts** (tools, can stay or Rust-ify):
```
scripts/extract_hewiktionary_v2.py   ← Hebrew dictionary extractor
scripts/build_wiki_full.py           ← Wiki dump builder
scripts/extract_multilingual.py      ← Multi-lang extractor
```

### Python **py_testers/** (OK per CLAUDE_RULES §4):
12 prototypes — להישאר. תכלית: validate לפני Rust.

**בלאגן Python:** יש 16 תהליכי python רצים + scripts עתיקים. צריך:
1. **Rust-ify:** zets_http_api + zets_client (Rust server + Rust persona walker)
2. **Keep:** py_testers/ (prototypes), scripts/ (one-shot ingestion)
3. **Archive:** mcp/autonomous/ — החליף במודול Rust `corpus_acquisition`

## 5. Data

| Type | Size |
|------|------|
| Wikipedia dumps | 17 GB (48/48 languages) |
| Packs (built atoms) | 98 MB |
| Autonomous state | 3.2 MB |
| Articles ingested | 17.5M |

---

# PART B: Self-Improvement Architecture

## הCONCLUSION מ-2 AIs

**gpt-4o (10.3s, 3KB):** Implementable, highlights common-sense reasoning as biggest miss.

**Gemini 2.5 Flash (31.4s, 20KB):** Comprehensive. Key points:

### 1. **Procedure shape** — DAG OF STEP-ATOMS (primary) + bytecode arrays (for primitive leaves)

```rust
pub struct Procedure {
    id: AtomId,
    name: String,
    input_params: Vec<AtomId>,
    output_params: Vec<AtomId>,
    preconditions: GraphQuery,
    postconditions: GraphQuery,
    steps: Vec<ProcedureStep>,  // ordered DAG
    trust_level: TrustLevel,    // System/LearnedVerified/LearnedUnverified/UserDefined
    cost_estimate: u32,         // opcodes budget
    max_cost: u32,
}

pub enum ProcedureStep {
    CallProcedure { proc_id: AtomId, args: HashMap<AtomId,AtomId>,
                    on_success: Option<AtomId>, on_failure: Option<AtomId> },
    ExecuteBytecode { bytecode: Vec<Opcode>, ... },
    IfElse { condition: GraphQuery, ... },
    Loop { condition, body, max_iterations },
    Parallel { steps: Vec<AtomId>, join: JoinType },
}
```

### 2. **Self-improvement loop** — 4 phases

```
1. OBSERVATION
   - User query unmatched → gap detected
   - Failed procedure → improvement goal
   - Meta-reasoning: "lots of Arabic articles, no Arabic procedure"
   
2. INFORMATION GATHERING (pattern extraction — primary)
   - Query 17.5M articles for relevant text
   - BPE+Merkle fold identifies recurring patterns
   - BitflagRelation + CognitiveKinds identify semantic roles
   - LLM distillation as HELPER (not source)
   
3. PROCEDURE FORMULATION
   - Hypothesize DAG of step-atoms
   - Map to existing opcodes + sub-procedures
   - Assign preconditions/postconditions from context
   - Store as Procedure atom
   
4. EVALUATION
   - Simulate in sandboxed subgraph
   - Compare to alternatives (cost, success rate)
   - If fail → meta-reason WHY → refine
   - If pass → promote trust_level
```

### 3. **Minimum viable kernel — what's in Rust vs loadable**

**HARDCODED (must stay in Rust):**
- 32 opcodes ✓ (already have)
- VM with bounded execution ✓
- Atom/edge primitives ✓
- BPE/Merkle fold ✓
- Graph traversal primitives ✓
- Self-reflection opcodes (get_procedure_list, get_atom_properties)
- Goal management (priority queue)
- Error/logging

**LOADABLE AS ROUTES:**
- `learn_new_language(lang)` ← top-level procedure
- `find_language_corpus(lang)` ← sub
- `extract_common_words(corpus)` ← sub
- `identify_grammatical_patterns(corpus)` ← sub
- `create_lexicon_atoms(words)` ← sub
- `translate_text(src, tgt, text)` ← basic rule-based
- `extract_procedure_from_doc(doc, domain)` ← critical for building/medicine/etc
- `meta_reason_on_failure(proc_id)` ← improvement
- `propose_hypothesis(pattern)` ← improvement

**MINIMUM ROUTE SET (to bootstrap):** ~15-20 routes + 50-100 atoms.

### 4. **Safety — Budget + Trust + Sandbox**

- **Bounded VM** ✓ (already have 32 call depth, 10K ops)
- **Budget** — per-procedure max_cost, global rate limit, learning budget separate
- **Trust levels**: System (hardcoded) / LearnedVerified / LearnedUnverified / UserDefined
- **Permissions**: low-trust procedures limited to SIMULATION, not execution
- **Sandbox**: new procedures run in isolated subgraph first
- **Rollback**: transactional updates, snapshots
- **Meta-monitoring**: failure rates, budget overruns, inconsistencies → auto-improvement goals

### 5. **Execution vs Explanation** — BOTH via mode flag

- Default for "how do I X?" → **Explanation** (traverse DAG → human text)
- "Simulate X with Y" → **Simulation** (run in bounded VM, return trace)
- "Execute X" → **Execution** (requires confirmation + high trust)

### 6. **Walk-through: "teach me Arabic"**

```
1. read_input("teach me Arabic")
2. goal_parser matches "teach me X" → learn_new_language(lang="Arabic")
3. Preconditions: has_wikipedia_data("Arabic") → TRUE
4. learn_new_language("Arabic") executes:
   Step 1: find_language_corpus("Arabic")
     - query_wikipedia_index("Arabic")
     - load_article_content(ids...)
     - tokenize_content(raw) via BPE
   Step 2: extract_common_words(corpus)
     - count_token_frequency
     - filter_stopwords
     - create_word_atoms → atoms with CognitiveKind: Word, Language: Arabic
   Step 3: identify_grammatical_patterns(corpus)
     - segment_sentences
     - find_recurring_sequences (n-grams)
     - propose_syntactic_rules
     - propose_morphological_rules
   Step 4: create_grammar_rules
     - Create new Procedure atoms for each rule
   Step 5: seed_translation_pairs(Arabic, English)
     - Find cognates in sense_graph
     - Build initial word→sense links
   Step 6: update_language_capability("Arabic")
5. Result: atoms added, new routes stored, capability reported
```

### 7. **Biggest miss — common-sense reasoning**

Per gpt-4o: Cyc / SOAR focus on common-sense & contextual adaptation.

Gemini didn't flag this specifically but implies it via "semantic grounding" and "cross-language cognates."

**תיקון:** CognitiveKind::Commonsense אטום — contains world-state assumptions (objects fall, water is wet, etc). Loaded with seed routes. ZETS can challenge/update but not erase.

---

# PART C: What I'm building next (decide-and-build)

Per CLAUDE_RULES §4 (Python tester first, then Rust):

## Step 1: Python prototype — `test_procedure_graph_v1.py`
Validate the DAG-of-step-atoms model with a toy "build_house" procedure:
- Steps: `acquire_land` → `pour_foundation` → `build_walls` → `add_roof` → `inspect`
- Preconditions: money, tools, permit
- Sub-procedure: `pour_foundation` expands to `mix_concrete` + `set_forms` + `pour`
- Test execution: simulated success & simulated failure (missing precondition)

## Step 2: Rust — `src/procedure_graph.rs` (new module)
- `Procedure` struct matching Gemini's design
- `ProcedureStep` enum (CallProcedure / ExecuteBytecode / IfElse / Loop / Parallel)
- `TrustLevel` enum (System / LearnedVerified / LearnedUnverified / UserDefined)
- `ProcedureStore` with atomic CRUD
- Integration with `system_graph/vm.rs` — VM can call `ExecuteProcedure` via new opcode

## Step 3: New opcodes in `system_graph/opcodes.rs`
- `ExecuteProcedure = 60`
- `CheckPrecondition = 61`
- `RegisterProcedure = 62` (self-modification!)
- `GetProcedureList = 63`
- `CreateAtomTyped = 64`

## Step 4: Seed routes in `system_graph/bootstrap.rs`
Add minimum viable kernel:
- `R_LEARN_NEW_LANGUAGE`
- `R_EXTRACT_PROCEDURE_FROM_DOC`
- `R_META_REASON_ON_FAILURE`

## Step 5: Self-improvement goal manager
- `src/goal_manager.rs` — priority queue of goals
- Triggers: unmatched query, failed procedure, dormant domain
- Emits: improvement goals into system_graph

## Step 6: Seed Knowledge Pack
- Bundle: Rust binary + routes.json + atoms.seed + tanakh_1206047_letters.txt
- On fresh machine: runs bootstrap → learns first language → recursively learns others

## Step 7: "Paint a house" procedure extraction
Demo: feed ZETS a text description → extract DAG → execute in simulation → verify output matches input
(This proves the architecture works on a real procedure domain)

---

# SUCCESS METRICS

1. **Procedure graph** compiles + 20+ unit tests pass
2. **Bootstrap** on fresh machine: in 10 min, ZETS learns 1000 Hebrew words from Wiki
3. **Procedure extraction**: from "how to paint a house" doc → DAG of 5+ steps
4. **Meta-improvement**: ZETS detects failed translation → proposes improved grammar rule
5. **Cross-language**: "teach me Arabic" → creates atoms for top 1000 Arabic words + 10 grammar patterns
6. **Explanation**: "how do I build a house?" → returns readable narrative with sub-procedures
7. **Simulation**: "simulate building a house" → returns step-by-step trace

## למידה מ-2 AIs

**הסכמות (high confidence):**
- DAG-of-step-atoms as primary representation
- Bytecode arrays only for primitive leaves
- 4-phase self-improvement loop
- Trust levels + sandbox + budget for safety
- Simulation as default, execution behind gates
- Pattern extraction > LLM distillation (as primary source)

**ייחודי ל-Gemini:**
- Specific opcode extensions list
- Detailed walk-through of "teach me Arabic"
- Parallel execution in DAG
- Rollback via transactional updates
- LLM as HELPER for semantic parsing, not primary

**ייחודי ל-gpt-4o:**
- Common-sense reasoning (Cyc/SOAR) as biggest miss
- Extensibility of opcode set as concern
