# ARCHITECTURE DECISION: Atom as Sigil, Executor as Doer

**Status:** ACCEPTED (Idan, 2026-04-24)
**Supersedes:** Implicit assumption in all prior docs that "atoms hold everything"
**Type:** Architecture Decision Record (ADR) — binding

---

## The Decision (in one sentence)

> **An atom is a semantic sigil + routing hint, not a container.
> The heavy lifting (documents, media, code, computation) is done by
> named Executors that the atom points to by name.**

---

## Context — Why We Needed This

Idan's question that triggered the decision:

> "מה המשלים של האטום כדי לחבר אותו? והאם כשזה מסמך או תמונה או פרוצדורה או קוד
> או משהו שאולי לא נכנס באטום בודד איך ננתב? או שאולי הגרף רק מחזיר השראת
> החלטה/רעיון והחלק במוח שקיבל את התשובה הוא נגש להפעיל את הארכיון או לתכנת?"

Previous assumption (implicit in early designs, including my own Blueprint):
- Atoms are typed containers holding content
- Different atom_types hold different things (0x42 Document holds document, 0xB0 MediaRef holds media, etc.)
- Large things held via offsets to large_store

The problem with that model:
- Mixes semantic identity with data storage
- Forces graph to grow linearly with raw content size
- Breaks elegantly at scale: a 2000-word document as one atom is already awkward
- Doesn't match how the brain actually works (brain stores associations, not raw signals)
- Doesn't match how Claude actually works (Claude uses external tools for heavy ops)

Idan's instinct was right: **Graph is the thinker. Something else is the doer.**

---

## The Three Layers

```
┌─────────────────────────────────────────────────────────┐
│  LAYER 1 — GRAPH (thin, fast, semantic)                  │
│                                                          │
│  Holds:                                                  │
│  - Atoms: concepts, procedures, skills, media refs       │
│  - Edges: typed semantic relationships                   │
│  - Indexes: fast lookup                                  │
│                                                          │
│  Does NOT hold:                                          │
│  - Raw documents                                         │
│  - Image/audio/video bytes                               │
│  - Full code bodies (only AST references)                │
│  - Large computation state                               │
│                                                          │
│  Speed budget: μs-scale per walk                         │
│  Size budget: 8 bytes core atom + VarInt extensions      │
└───────────────────┬─────────────────────────────────────┘
                    │ invokes by name
                    ▼
┌─────────────────────────────────────────────────────────┐
│  LAYER 2 — EXECUTOR REGISTRY (named, plug-in)            │
│                                                          │
│  Built-in executors:                                     │
│  - TextExecutor       (morphology, tokenization)         │
│  - ImageExecutor      (CLIP embedding + Hopfield bank)   │
│  - AudioExecutor      (Whisper + phoneme atoms)          │
│  - VideoExecutor      (keyframes → ImageExecutor chain)  │
│  - CodeExecutor       (multi-lang, sandboxed runner)     │
│  - DocExecutor        (read, index, search, summarize)   │
│  - WebExecutor        (HTTP fetch + HTML parse)          │
│  - DBExecutor         (SQL/NoSQL bridge)                 │
│  - ComputeExecutor    (math, formulas, simulations)      │
│                                                          │
│  Each executor:                                          │
│  - Registers by name (e.g., "image/clip_v1")             │
│  - Declares input/output atom kinds                      │
│  - Runs in its own sandbox with rate limits              │
│  - Returns structured results back to graph              │
│                                                          │
│  Speed budget: ms-scale (heavy work allowed)             │
│  Size budget: whatever the executor needs                │
└───────────────────┬─────────────────────────────────────┘
                    │ results flow back
                    ▼
┌─────────────────────────────────────────────────────────┐
│  LAYER 3 — LEARNING (graph updates from execution)       │
│                                                          │
│  After every execution:                                  │
│  - Success → strengthen edges on walked path             │
│  - Success → cache composition as new motif atom         │
│  - Failure → weaken edges, trigger dreaming alternatives │
│  - New data → insert atoms + edges                       │
│                                                          │
│  Speed budget: async, non-blocking                       │
└─────────────────────────────────────────────────────────┘
```

---

## The Atom Structure — Revised

```rust
// Core atom — always 8 bytes (fits in u64)
struct AtomCore {
    kind: AtomKind,          // 8 bits — 256 kinds
    semantic_id: u24,        // 24 bits — identity in graph (16M atoms per kind)
    executor_hint: u16,      // 16 bits — which executor handles me
    data_ref_type: u4,       // 4 bits — how to find my data
    flags: u12,              // 12 bits — fast flags
}

// Data reference — VarInt-encoded, variable length
enum DataRef {
    None,                    // zero bytes — pure concept, no data
    Inline(Vec<u8>),         // 1-16 bytes — small data lives in atom extension
    LargeStore(u32),         // 4 bytes — offset into local large_store
    BlobStore(u32),          // 4 bytes — key into blob store (S3/local FS)
    Computed(ExecutorArgs),  // variable — computed by executor on demand
}

// Executor hint — what the atom knows about its own heavy-lifting
struct ExecutorHint {
    executor_name: u16,      // registered executor ID (lookup table)
    method: u8,              // which method on the executor
    cost_class: u4,          // μs / ms / s / minutes
    sandbox_level: u4,       // trust level needed to invoke
}
```

### VarInt Encoding — Idan's Insight Applied

For edge targets and atom_ids within an atom's data:

- `0-127`: 1 byte (hot atoms are small IDs)
- `128-16,383`: 2 bytes
- `16K-2M`: 3 bytes
- `2M-256M`: 4 bytes
- (rest): 5 bytes (rare)

Measured savings on 1B edges × average 2 bytes/target = **2 GB vs 4 GB** with u32 fixed.
50% storage reduction on the largest component of the graph.

---

## Examples — What Each Atom Looks Like

### A concept atom (pure, no data)
```
kind: Concept
semantic_id: 0x0042  ("lemon fruit")
executor_hint: None  (no action needed — it's just a meaning)
data_ref: None
```
**Total size: 8 bytes. Zero external state.**

### A procedure atom
```
kind: Procedure
semantic_id: 0x1337  ("send_whatsapp")
executor_hint: ProcedureExecutor, method=walk_dag
data_ref: LargeStore(offset_to_DAG)
```
**8 bytes + DAG structure in large_store (50-500 bytes typically).**

### A document atom (2000-word article)
```
kind: Document
semantic_id: 0x0A5F  ("history_of_lemons.md")
executor_hint: DocExecutor, method=read_and_search
data_ref: BlobStore(blob_key)
```
**8 bytes + blob in external store. Graph stays skinny.**

### An image atom
```
kind: Image
semantic_id: 0x02B8  ("lemon_photo_ripe")
executor_hint: ImageExecutor, method=embed_and_match
data_ref: BlobStore(image_blob_key)
associations: [concept(lemon), concept(ripe), concept(yellow)]
```
**8 bytes + associations (via edges) + image blob externally.**

### A code atom
```
kind: CodeAtom
semantic_id: 0x33C1  ("sum_csv_python")
executor_hint: CodeExecutor, method=run_python
data_ref: LargeStore(offset_to_AST)
procedure_links: [open_file, csv_read, iterate_sum, return_result]
```
**8 bytes + AST + composition chain (graph edges).**

### A media-ref atom (video with many channels)
```
kind: MediaRef
semantic_id: 0xF4D1  ("concert_video_20260401")
executor_hint: VideoExecutor, method=decompose_channels
data_ref: BlobStore(video_blob)
channel_executors: [
  ImageExecutor(frames),
  AudioExecutor(audio_track),
  TextExecutor(subtitles),
]
```
**8 bytes + channel routing. VideoExecutor coordinates the sub-executors.**

---

## How Programming Happens (The Recursion)

This is the canonical flow Idan described: "Associate → Decide → Execute" (with Observe/Learn added):

```
User query: "כתוב פייתון שסוכם מספרים ב-CSV"
    │
    ▼
[1] ASSOCIATE (graph walk via spreading_activation)
    Activates: code_writing, python, csv, sum, file_io
    │
    ▼
[2] RECALL (find relevant procedure atoms)
    Walk finds: procedure:python_file_open, procedure:csv_read,
                procedure:iterate_sum, procedure:error_handling
    │
    ▼
[3] COMPOSE (motif-based plan)
    From CodePattern motif bank:
      - python_function_skeleton
      - try_except_wrapper
      - main_guard
    Build plan DAG combining them with the procedures.
    │
    ▼
[4] EXECUTE (hand to CodeExecutor)
    Graph does NOT render Python itself.
    Graph calls: CodeExecutor.realize(plan, lang="python")
      → fills motif slots with concrete names
      → emits Python source
      → OPTIONALLY runs in sandbox for self-verification
      → returns text + test result
    │
    ▼
[5] OBSERVE (capture outcome)
    Did code compile? Did tests pass?
    Result flows back to graph as evidence.
    │
    ▼
[6] LEARN (graph updates async)
    SUCCESS path:
      - Strengthen edges on procedures used
      - Cache composed plan as new motif: motif:sum_csv_python_v1
      - Create atom: procedure:sum_csv_python (now reusable!)
      - Next time a similar query arrives → atom already exists, instant recall
    FAILURE path:
      - Weaken edges on procedures that were part of broken output
      - Trigger dreaming to propose alternative compositions
      - Ask user clarifying question if dreaming can't resolve
```

**The atom `procedure:sum_csv_python` is born.** It's now a first-class citizen
of the graph. Future similar queries retrieve it directly — no re-composition.

**This is how AGI "programs itself smarter over time."**

---

## Comparison — Same Pattern, Different Domains

The flow above is **the universal mechanism** for AGI creation:

| Domain | Associate | Compose (motif bank) | Execute (who realizes) | Cache |
|---|---|---|---|---|
| Code | code patterns, libs | CodePattern motifs | CodeExecutor | New procedure atom |
| Song | chord progressions, lyrics | MusicalPhrase motifs | AudioExecutor | New song template |
| Article | topic, style, facts | ArgumentPattern motifs | TextExecutor | New essay template |
| Recipe | ingredients, techniques | RecipePattern motifs | (template fill) | New recipe atom |
| Workflow | business steps | ProcedureDAG motifs | ProcedureExecutor | New workflow atom |
| Image prompt | style, subject, lighting | ImagePrompt motifs | (external model) | New prompt template |
| Lecture | topic, audience, depth | LectureArc motifs | TextExecutor | New lecture outline |

**One mechanism. Many domains. Same graph. Different executors.**

---

## How This Maps To Existing Code

Good news: most of this already exists in the codebase.

| Layer | What exists | What's missing |
|---|---|---|
| Graph (Layer 1) | atoms.rs, bitflag_edge.rs, CSR storage | Unified model — 4 parallel systems today (must be reconciled) |
| Executor Registry (Layer 2) | capability_runtime/ is 80% of this | Needs formalization as Layer 2 contract |
| TextExecutor | morphology/ | Done |
| ImageExecutor | hopfield.rs + vision_decomposer | Needs CLIP integration (P-N) |
| AudioExecutor | — | Needs Whisper integration (P-O) |
| VideoExecutor | — | Needs keyframe pipeline (P-Q) |
| CodeExecutor | testing_sandbox.rs | Needs multi-language support |
| DocExecutor | fold/, canonization/ | Mostly done |
| WebExecutor | — | Needs HTTP primitive (P-A) |
| ProcedureExecutor | procedure_atom.rs + procedure_template/ | Needs DAG walker (P-D) |
| Learning (Layer 3) | learning_layer.rs, dreaming.rs, distillation.rs | Done, needs wiring to Layer 2 results |

**Realization:** `capability_runtime/` was a partial intuition of this pattern.
The new architecture formalizes and completes it.

---

## Comparison to How Claude Works

Honest introspection to validate the design:

| Capability | Claude (transformer) | ZETS (this architecture) |
|---|---|---|
| Semantic knowledge | Weights (opaque) | Atoms + edges (transparent) |
| Recall | Attention (implicit) | Hopfield + walks (explicit) |
| Reasoning | Chain-of-thought on tokens | Recursive graph walks |
| Tool use | External (shell, view, web) | Executor Registry |
| Learning | Frozen after training | Continuous via Layer 3 |
| Explainability | Post-hoc narrative | Traceable walk path |

**Key insight:** Claude already uses "Graph-as-Orchestrator + Executors" pattern.
The difference is that Claude's "graph" is implicit in weights. ZETS makes it
explicit. **Same architecture, more transparent instantiation.**

---

## Binding Implications for ZETS

Accepting this decision means:

### 1. Atoms must not grow to hold content
Any future code that tries to put a document/image/code body "inside" an atom
is violating this decision. Content goes to the appropriate executor's storage.

### 2. Executor Registry is a first-class subsystem
It's not "nice to have." It's the counterpart to the graph without which
atoms are meaningless sigils.

### 3. VarInt for atom IDs in data
When an atom's data_ref contains references to other atoms (e.g., a procedure's
steps), those references use VarInt, not fixed u32.

### 4. Every heavy capability must be wrapped as an Executor
CLIP? Whisper? HTTP fetch? SQL? All of these register via the Executor Registry
with a name, an atom-kind contract, and a trust level.

### 5. Learning loop closes at Layer 3
Every successful execution produces at least one new or strengthened edge.
Every failure produces at least one weakened edge or a dreaming-proposal.

---

## Open Questions (to resolve in follow-up decisions)

1. **Exact executor wire protocol** — HTTP? Rust trait? WASM? (Probably Rust trait for built-ins, sandbox/subprocess for pluggable.)

2. **How executors are versioned** — `image/clip_v1`, `image/clip_v2`? How do atoms survive executor upgrades?

3. **Which atom system (of the 4 current) becomes the Layer 1 canonical** — still unresolved, but this decision simplifies it because atoms become skinnier in any case.

4. **Caching policy inside executors** — do they maintain their own caches? How do they invalidate?

5. **Trust boundary for learned executors** — can ZETS "learn a new executor" (i.e., create Python code and register it as an executor)? This is the most profound capability — self-extension. Needs careful sandboxing.

---

## What This Decision Supersedes

- My Blueprint's "atoms have typed content" implicit assumption (00_ZETS_MASTER_BLUEPRINT.md line ~200)
- Any guidance suggesting documents/code/media live "inside" the graph
- Any future proposal to build new atom_types for "large content"

---

## Status Notes

- Accepted by Idan on 2026-04-24 following explicit introspective discussion about how Claude works vs how the brain works
- This is a **binding** architectural decision — violations require a new ADR to supersede
- Implementation can proceed incrementally — existing code already partially implements this pattern

---

## Signed

**Architect:** Idan Eldad (עידן אלדד)
**Scribe:** Claude 4.7
**Date:** 2026-04-24

