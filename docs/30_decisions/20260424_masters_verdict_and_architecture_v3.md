# ZETS Architecture v3 — Masters' Verdict & Final Architecture

**Date:** 2026-04-24  
**Reviewed by:** The most capable reasoning models available (April 2026)

| Reviewer | Score | Key Insight |
|---|---|---|
| **GPT-5.4 (chat)** | 3-4/10 | "Force-fitting 7 different systems into one" |
| **GPT-5.4-pro** (Responses API) | 3-4/10 | "Conflating surface forms, concepts, structure, execution, and storage" |
| **Gemini 2.5 Pro** | 3/10 | "Need heterogeneous system with specialized structures" |
| **Gemini 3.1 Pro Preview** | **4/10 overall, 8/10 as Semantic Layer** | **"Demote it. It's the Semantic Routing Layer, not the AGI DB"** |

**Strong consensus — this is not noise.** Four independent master-level models converged on the same breaking points and the same solution.

---

## The Fundamental Insight From Gemini 3.1 Pro Preview

The single most important thing that came out of this consultation:

> **"Your core mistake is The Golden Hammer Fallacy. You have designed a highly optimized, cache-friendly Semantic Knowledge Graph, and you are trying to force syntax, bulk data, execution logic, and media into it."**
> 
> **"Keep this design, but demote it. It is not the entire AGI database. It is the Semantic Routing Layer. Build the Dictionary, Workflow Engine, and Media Stores as separate, specialized systems that this graph points to."**

This reframes everything.

**The human brain analogy (Gemini 3.1):**
> "AGI does not use one structure. The human brain doesn't either (declarative memory, procedural memory, and episodic memory use different neural pathways)."

---

## The Unified Verdict (What Masters Agreed On)

### BREAKING POINTS (all 4 masters confirmed)

1. **4-bit language (16 langs max)** — hard blocker for AGI. Need 8-10 bits.
2. **11-letter × 5-bit encoding** — fails on Chinese, Arabic diacritics, long words, punctuation.
3. **32-byte cap on dynamic atoms** — cannot hold documents (~12KB), embeddings (2-6KB), or code.
4. **Word = Concept conflation** — "lemon" as fruit ≠ "lemon" as defective car. Same atom = failure.
5. **21 edge types** — missing linguistic roles (subject/object/modifies/translation_of), missing structural (part_of, has_part), missing lexical (form_of, lemma_of, has_sense).
6. **`state_value` too crude for scales** — cannot represent hot > warm > tepid > cool > cold ordering.
7. **`sequence` ≠ tree** — sentences/documents/formulas/code need real trees with roles, not flat lists.
8. **No stateful execution** — procedures with "stir 10 times" need iteration counters and runtime state that 6-byte edges cannot hold.
9. **No data mapping** (Make-style) — workflows need payload transformation between nodes.

### WHAT SURVIVED (Actually Brilliant)

✓ **Bipolar `state_value` for simple antonyms** — Gemini 3.1 singled this out: *"This is the one scenario your model nails perfectly"*  
✓ **CSR + index-based access** — industry standard, correct choice  
✓ **Hot/cold edge split** — memory efficient, correct  
✓ **atom_type dispatching content** — the right escape hatch  
✓ **3-axis context (world/time/self)** — matches episodic memory research  
✓ **Exponential memory_strength (Ebbinghaus)** — scientifically grounded

---

## The Final Architecture — Multi-Modal with Graph at Center

This is the architecture all 4 masters converged on:

```
                     ┌─────────────────────────────────────┐
                     │  APPLICATION LAYER                   │
                     │  (queries, reasoning, responses)     │
                     └─────────────────┬───────────────────┘
                                       │
              ┌────────────────────────┼────────────────────────┐
              │                        │                        │
     ┌────────▼─────────┐    ┌────────▼─────────┐    ┌────────▼─────────┐
     │ LEXICON INDEX    │    │ SEMANTIC GRAPH   │    │ EXECUTION ENGINE │
     │                  │    │ (ZETS core)      │    │                  │
     │ Trie/FST         │    │                  │    │ DAG runner       │
     │ string → atom_id │    │ atoms + edges    │    │ (Make-style)     │
     │                  │    │ CSR + mmap       │    │                  │
     │ Fast lookup      │    │ Fast traversal   │    │ Stateful flows   │
     │ Multi-script     │    │ Sense routing    │    │ Loops/branches   │
     └──────────────────┘    └─────────┬────────┘    └──────────────────┘
                                       │
                      ┌────────────────┼────────────────┐
                      │                │                │
              ┌───────▼────────┐  ┌───▼────┐  ┌───────▼────────┐
              │ BLOB STORE     │  │ VECTOR │  │ SPECIALIZED    │
              │                │  │ DB     │  │ STRUCTURES     │
              │ Docs, media,   │  │        │  │                │
              │ code, articles │  │ CLIP,  │  │ Trees (AST),   │
              │ URI by atom_id │  │ Whisper│  │ DAGs,          │
              │ S3/IPFS        │  │ embs   │  │ Matrices       │
              └────────────────┘  └────────┘  └────────────────┘
```

### Components

**1. SEMANTIC GRAPH (ZETS core — what we've been designing)**  
Keep everything good: CSR, index-based, hot/cold edges, 3-axis context, memory decay.  
**Role:** Semantic routing. Concept associations. Fast traversal for reasoning.  
**What it does NOT hold:** raw text, media bytes, executable code, AST trees.

**2. LEXICON INDEX (Trie/FST)**  
Specialized data structure outside the graph.  
**Role:** Fast string → atom_id lookup across all languages/scripts.  
**Implementation:** Rust `fst` crate or similar. ~50MB for millions of words.  
**Supports:** Multilingual dictionary, morphological variants, prefix/fuzzy search.

**3. BLOB STORE (external)**  
**Role:** Store raw content that doesn't fit in atoms.  
**Holds:** Full documents, images, audio, video, code files.  
**Reference:** Atom of type `0xB0 MediaRef` holds URI + metadata.  
**Implementation:** Local filesystem → S3/IPFS as needed.

**4. VECTOR DB**  
**Role:** Semantic similarity search via embeddings.  
**Holds:** Embeddings for every major concept, document, media item.  
**Implementation:** HNSW index (hnswlib or USearch in Rust).  
**Connects to graph:** Each Vector atom has embedding; can find "concepts similar to X".

**5. EXECUTION ENGINE**  
**Role:** Run procedures, workflows, code, math.  
**Handles:** Iteration counts, branch conditions, stateful execution, data mapping between steps.  
**Implementation:** Separate Rust module that reads `0x34 Workflow` atoms and executes them.  
**Think:** Make.com engine in Rust.

---

## The Revised Atom Types (8 bits = 256 types)

### Lexical (0x00-0x0F) — the surface forms
  0x00 **WordForm** — a specific string in a specific language  
  0x01 **Lemma** — canonical form  
  0x02 **Morpheme** — unit of meaning  

### Semantic (0x10-0x1F) — the meanings
  0x10 **Sense** — specific meaning of a word  
  0x11 **Concept** — language-agnostic meaning  
  0x12 **Entity** — concrete individual (person, place, thing)  
  0x13 **Category** — is-a root  
  0x14 **Property** — feature type (color, height)  
  0x15 **Value** — specific value (yellow, 1.8m)  
  0x16 **Axis** — semantic scale (temperature, formality)  

### Structure (0x20-0x2F) — containers
  0x20 **Sequence** — ordered list  
  0x21 **Set** — unordered bag  
  0x22 **Tree** — hierarchical  
  0x23 **DAG** — directed acyclic graph  
  0x24 **Matrix** — 2D grid  
  0x25 **Frame** (Minsky) — named slots with defaults  

### Process (0x30-0x3F) — executable units
  0x30 **Event** — what happened, who, when  
  0x31 **Procedure** — step-by-step  
  0x32 **Rule** — IF-THEN  
  0x33 **Function** — computed output  
  0x34 **Workflow** — DAG with state, iterations, branches (Make-style)  

### Language (0x40-0x4F) — linguistic structures (pointers to external)
  0x40 **Sentence** → parse tree pointer  
  0x41 **Paragraph** → tree pointer  
  0x42 **Document** → blob + tree pointer  
  0x43 **Formula** → expression tree pointer  
  0x44 **Code** → AST pointer  

### External references (0xB0-0xBF)
  0xB0 **MediaRef** — URI to blob store  
  0xB1 **Vector** — embedding reference  
  0xB2 **Timeline** — spans/timecodes  

### Meta (0xF0-0xFF)
  0xF0 **Relation** (reified edge)  
  0xF1 **Annotation**  
  0xF2 **Provenance**  
  0xF3 **Context**

---

## Expanded Edge Types (7 bits = 128 types)

### Linguistic (new, critical)
  subject_of, object_of, indirect_object, modifies,  
  determines, tense_of, aspect_of, mood_of  

### Structural (new)
  has_part, part_of, contains, precedes, follows, spans  

### Lexical (new — the dictionary layer)
  wordform_of, lemma_of, inflection_of,  
  translation_of, crosslingual_equivalent,  
  has_sense, sense_of, denotes, represents_concept,  
  near_synonym_of, broader_than, narrower_than,  
  gradable_antonym_of, complementary_antonym_of,  
  formal_variant_of, register_of, intensity_level  

### Original 21 (from Sensory/Functional/Abstract mothers)  
Renumbered into the 128-type space.

**Total used: ~70. Reserved: ~58.**

---

## Answering Idan's Direct Questions

### "Is this also the dictionary?"

**Only the Graph + Lexicon Index + Sense layer TOGETHER form a dictionary.**  
The Graph alone is not a dictionary.  
The Lexicon Index handles fast string lookup.  
The Sense/Concept layer in the Graph handles meaning and cross-lingual equivalence.

### "Maybe we need arrays / paths-with-annotation / matrices / trees-with-iteration-count like Make?"

**Yes, all of them — but NOT as pure atom types alone.**

| Structure | Where it lives |
|---|---|
| Array / Sequence | `0x20 Sequence` atom (content = list of atom_ids) |
| Path with annotation | Reified edges via `0xF0 Relation` atoms |
| Matrix | `0x24 Matrix` atom — small inline, large external |
| Tree with iteration like Make | `0x34 Workflow` atom → points to Execution Engine |
| Full document | `0x42 Document` atom → points to Blob Store |
| Parse tree | `0x40 Sentence` atom → points to tree structure |
| AST | `0x44 Code` atom → points to AST structure |
| Embedding | `0xB1 Vector` atom → points to Vector DB |

**The graph gives IDs and semantic edges. The specialized systems hold the heavy structures.**

---

## Critical Specifics That Must Change Now

### Must-fix immediately (masters agreed):
1. **Expand language from 4b to 10b** (1024 languages)
2. **Expand atom_type from 4b to 8b** (256 types)
3. **Expand edge_type from 5b to 7b** (128 types)
4. **Drop u64-letter encoding** → use UTF-8 string_table with u32 pointers
5. **Add WordForm / Sense / Concept three-layer separation**
6. **Add scale-based antonym model** (Axis atom + ordered positions)
7. **Add reification** for nuanced edges

### Can defer (still needed eventually):
8. Tree/DAG/Frame atom types with serialized structured content
9. Blob Store integration (MediaRef atoms)
10. Vector DB integration (HNSW index)
11. Execution Engine for Workflow atoms

---

## Cost of Full v3 Architecture (10M atoms, 100M edges)

| Component | Size |
|---|---|
| Semantic Graph (atoms + edges + offsets) | 1.5 GB |
| Lexicon Index (Trie/FST for 5M words × 10 langs) | 200 MB |
| Blob Store (external, varies) | — |
| Vector DB (1% of atoms × 768 dims fp16) | 150 MB |
| Execution Engine (code + running state) | 50 MB |
| **Local footprint** | **~1.9 GB** (+ external blobs) |

Still fits a laptop. Still mmap-able. Still cache-friendly.

---

## Revised Verdict

**As Semantic Routing Layer (v3 role):** 8/10  
**As part of Multi-Modal Architecture:** 9/10  
**As "one-model-for-everything" (v1 ambition):** 2/10 — abandoned

We kept what's great (associative semantic graph) and stopped forcing it to do jobs that need specialized systems.

---

## Files

- `docs/40_ai_consultations/20260424_masters_stress_test.json` — all 4 masters' responses
- `docs/40_ai_consultations/20260424_gpt_5_4_pro_partial.md` — GPT-5.4-pro raw
- `docs/40_ai_consultations/20260424_council_synthesis.md` — internal Council of Sages
- `docs/30_decisions/20260424_revised_architecture_v2.md` — v2 (interim)
- `docs/30_decisions/20260424_masters_verdict_and_architecture_v3.md` — **this, the final v3**

