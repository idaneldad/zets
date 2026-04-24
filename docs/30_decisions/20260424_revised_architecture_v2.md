# ZETS Architecture v2 — Post-Challenge Revision

**Date:** 2026-04-24  
**Trigger:** Brutal stress-test by GPT-5.4, Gemini 2.5 Pro, and internal Council of Sages (Codd, Miller, Lenat, Knuth, Minsky, Berners-Lee).

**Verdict on v1:** 3-4/10. Good core idea, fundamental gaps.  
**Revised goal:** Address all 9 breaking points without losing the lean-compact spirit.

---

## What Survived (Lean Spirit)

✓ Hybrid atom storage (u64 fast-path + dynamic large-path)  
✓ atom_type dispatches content interpretation  
✓ 6-byte hot edges  
✓ CSR layout + mmap  
✓ 3 orthogonal context axes (space/time/identity)  
✓ Index-based access (not pointers)

## What Changed

### Change 1: WordForm ≠ Sense ≠ Concept — Three Layers

Inspired by WordNet (Miller 1985). The single biggest fix.

```
LAYER Word:     "לימון" (he)  "lemon" (en)  "limón" (es)  "柠檬" (zh)
                    ↓            ↓              ↓              ↓
LAYER Sense:   [yellow citrus fruit, sour taste, small size...]
                    ↓
LAYER Concept:  CONCEPT#lemon_fruit (language-agnostic)
```

- Words in different languages are different atoms.
- All words that mean "the fruit lemon" share the same Sense.
- Sense points to Concept (abstract, language-free).
- "lemon" in English also has a Sense "defective product" → different concept.

This solves: multilingual dict, synonyms, nuance, polysemy.

### Change 2: atom_type from 4 bits to 8 bits — 256 types organized in families

```
Family 0x0x  Lexical:     WordForm, Lemma, Phoneme, Morpheme
Family 0x1x  Semantic:    Sense, Concept, Entity, Category, Property, Value
Family 0x2x  Structure:   Sequence, Set, Tree, DAG, Matrix, Frame
Family 0x3x  Process:     Event, Procedure, Rule, Function, Workflow
Family 0x4x  Language:    Sentence, Paragraph, Document, Formula, Code
Family 0x5x  Media:       MediaRef, Vector, Timeline
Family 0xFx  Meta:        Relation (reified), Annotation, Provenance, Context
```

Cost: +1 byte per atom (now 9 bytes instead of 8 for header).  
Gain: room to actually express all knowledge types.

### Change 3: Drop u64-letter encoding. Words are pointers to a string table.

The 11-letter × 5-bit encoding was clever but fatal:
- Can't represent Chinese, Arabic, mixed-case, diacritics, punctuation
- 32 characters per slot = English alphabet barely fits
- 11 letters = most English words don't fit

**New approach:**
```
WordForm atom content:
  lang_id: u16    (65536 languages — plenty)
  string_ptr: u32 (index into string_table)
  length: u16     (byte length)
  flags: u16      (case, diacritics, morphological features)
```

String table is a flat `Vec<u8>` concatenating all word bytes in UTF-8.  
For 10M words avg 8 bytes each = 80MB — much less than the bit-packing savings claimed.

**Why this is better:**
- Full UTF-8 support (any language, any script)
- No arbitrary length limit
- Unicode normalization possible
- Same infrastructure handles Chinese ideographs (3 bytes UTF-8), emoji (4 bytes), accented Latin

### Change 4: Edge metadata — reification for nuance

Hot edges stay 6 bytes (95% of edges).  
When an edge needs nuance (confidence, provenance, formality, register):

```rust
// Before: one edge A → B
// After: two edges + reified atom
A → relation_atom → B

relation_atom is type 0xF0 (Relation), with its own edges:
  relation_atom has_confidence 0.85
  relation_atom provenance (user_statement, 2026-04-24)
  relation_atom nuance (formal, literary, archaic)
```

Cost: 3 edges instead of 1, but only for the 5% of edges that need nuance.

### Change 5: Trees, DAGs, Frames as first-class atoms

**Previously** we said "sequence covers it." It doesn't.

**Now:**
- **Tree** atom (0x22) — hierarchical structure: `[root_atom_id, (child_id, child_id, ...)]` nested
- **DAG** atom (0x23) — directed graph: `[nodes: [id,id,...], edges: [(from,to,meta)]]`
- **Frame** atom (0x25) — Minsky's frame: `{slots: [(name_atom, value_atom, default_atom, required_bool)]}`
- **Workflow** atom (0x34) — DAG + iteration counts + branch conditions (Make-style)
- **Formula** atom (0x43) — expression tree with operator nodes
- **Code** atom (0x44) — AST with binding table

Each large-atom `content` is a serialized structure (BinaryFormat with a small typed header).

### Change 6: Linguistic edges added

New edge types (using the expanded 7-bit edge_type space, 128 types):

**Grammatical:**
  subject_of, object_of, indirect_object_of,  
  modifies (adjective/adverb), determines (article),  
  verb_of, tense_of, aspect_of, mood_of, voice_of  

**Structural:**  
  has_part, part_of, contains, positioned_at,  
  precedes, follows, spans  

**Lexical:**  
  word_of_language, lemma_of, form_of,  
  pronounced_as, transliterated_as, has_sense  

**Sense-level:**  
  represents_concept, denotes, connotes,  
  near_synonym_of, broader_than, narrower_than,  
  register_of, intensity_of,  
  scalar_antonym_of, complementary_antonym_of  

Plus the original 21 (renumbered into the 128 space).  
Total: ~80 used, 48 reserved.

### Change 7: Media and large binary — external with reference

Never store media bytes in the atom graph.

```rust
MediaRef atom (0x50) content:
  uri: string_ptr     (S3 URL, file path, IPFS hash)
  mime_type: string_ptr
  duration_ms: u32
  dimensions: (u32, u32)  // for images/video
  embedding_ref: atom_id  // points to Vector atom
  annotations_ref: atom_id // points to Sequence of Annotation atoms
```

### Change 8: Vector/embedding as first-class atom

Machine-learned embeddings (for semantic similarity, vector search):

```rust
Vector atom (0x51) content:
  dim: u16
  model_id: u32
  data: [u8; dim * 2]  // fp16 by default (or quantized int8)
```

Enables: "find atoms semantically similar to X" via cosine distance.  
Works alongside the graph (graph gives structure, vectors give semantic neighborhoods).

---

## The Revised Storage Layout

```rust
struct ZetsCore {
    // ─── ATOMS ───
    atom_headers: Vec<AtomHeader>,    // 16 bytes each (incl. 8-bit type, lang, flags)
    atom_small_content: Vec<u64>,     // u64 content for simple atoms
    atom_large_content: Vec<u8>,      // concatenated bytes for structured atoms
    atom_large_offsets: Vec<u32>,     // offset per large atom
    
    // ─── STRINGS ───
    string_table: Vec<u8>,            // UTF-8 all strings concatenated
    string_offsets: Vec<u32>,         // offset per string
    
    // ─── EDGES (CSR) ───
    edges_hot: Vec<u64>,              // 8 bytes: dst_u32 + edge_type_u8 + packed_meta_u24
    fwd_offsets: Vec<u32>,            // CSR forward per atom
    rev_offsets: Vec<u32>,            // CSR reverse per atom
    
    // ─── COLD LOOKUPS ───
    edge_reified: HashMap<u32, u32>,  // edge_idx → atom_id of Relation
    atom_contexts: HashMap<u32, ContextAxes>,  
    edge_confidences: HashMap<u32, u8>,
    
    // ─── INDICES ───
    wordform_lookup: HashMap<(u16, String), u32>,  // (lang, word) → atom_id
    concept_by_name: HashMap<String, u32>,          // concept_name → atom_id
    
    // ─── EMBEDDINGS ───
    vector_store: VectorIndex,  // HNSW or FAISS for semantic search
}
```

### Revised Storage Cost (10M atoms × 100M edges)

| Component | Size |
|---|---|
| atom_headers (16B × 10M) | 160 MB |
| string_table (avg 8B × 5M words) | 40 MB |
| string_offsets | 20 MB |
| atom_small_content (u64 × ~6M) | 48 MB |
| atom_large_content (~50B × 4M) | 200 MB |
| atom_large_offsets | 16 MB |
| edges_hot (8B × 100M) | 800 MB |
| fwd_offsets + rev_offsets | 80 MB |
| **HOT TOTAL** | **~1.36 GB** |
| edge_reified (5% of edges) | 20 MB |
| atom_contexts (10% of atoms) | 30 MB |
| vector_store (if 1% have embeddings) | 100 MB |
| **COLD TOTAL** | **~150 MB** |
| **GRAND TOTAL** | **~1.5 GB** |

vs. v1 at 890 MB. Cost: **+70% storage** for massive capability gain (full multilingual dict, true sentence parsing, documents, workflows, media, code).

Still fits easily on any laptop.

---

## How This Answers Idan's Key Question

**"Is this also the dictionary?"**

**Now yes.** With WordForm/Sense/Concept separation:
- Multilingual dictionary: words in N languages → same Concept ✓
- Synonyms: words → same Sense (with register/intensity nuance) ✓
- Antonyms: Sense atoms on a Scale atom with ordered positions ✓
- Polysemy: one WordForm → multiple Senses → different Concepts ✓

**"Do we need structures like arrays, paths, matrices, trees with iteration count like Make?"**

**Yes, all of them, as atom_types:**
- Array/Sequence → atom_type 0x20
- Path with annotation → Tree (0x22) or DAG (0x23) with Annotation (0xF1) edges
- Matrix → atom_type 0x24
- Tree with iteration count → Workflow (0x34) with per-node iteration metadata

**Everything lives in one atom system**, but **specialized structures live inside atom_types that understand them**.

---

## Migration Plan

Not a rewrite. Progressive expansion:
1. Week 1: Add WordForm/Sense/Concept distinction. Migrate existing atoms.
2. Week 2: Expand atom_type to 8 bits. Add Tree/DAG/Workflow types.
3. Week 3: Add linguistic edge types. Start parsing sentences.
4. Week 4: Add Media/Vector atoms. Hook up embedding model.
5. Week 5+: Reification for nuance. Fill in specialty types.

---

## Files
- `docs/40_ai_consultations/20260424_knowledge_model_challenge_v2.json` — raw AI responses
- `docs/40_ai_consultations/20260424_council_synthesis.md` — council of sages synthesis  
- `docs/40_ai_consultations/20260424_challenged_concept.md` — what was tested
- `docs/30_decisions/20260424_revised_architecture_v2.md` — this document

