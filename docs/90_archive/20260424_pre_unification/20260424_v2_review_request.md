# ZETS Knowledge Architecture — Full Review Request

**Date:** 2026-04-24  
**Author:** Idan Eldad (ZETS founder) + Claude (AI assistant)  
**Status:** Post-stress-test revision (v2 after v1 scored 3-4/10)

---

## Context

**ZETS** is an AGI knowledge system being built in Rust. It must store and retrieve:
- Multilingual knowledge (Hebrew, English, Chinese, Arabic, Greek, Russian, Spanish...)
- Concepts, words, entities, senses, synonyms, antonyms
- Sentences, documents, articles  
- Rules, procedures, workflows (Make-style with iteration counts)
- Media references (images, audio, video)
- Mathematical formulas
- Code (ASTs)
- Personal memories with context (who/where/when)
- Edges (relationships) with confidence, provenance, nuance

**Design principles:**
1. Lean storage — bit-packed, CSR layout, mmap-friendly
2. Fixed-point integers preferred over floats
3. Index-based access (no pointers)
4. Scale target: 10M atoms × 100M edges = <2GB total

---

## v1 Concept (that scored 3-4/10)

### Atom encoding (u64 fixed, 95% of atoms)
```
┌──────────┬──────────┬─────────────────────────────────┬────┐
│ lang (4b)│ type (4b)│ letters (55b = 11 × 5b)          │ 1b │
└──────────┴──────────┴─────────────────────────────────┴────┘
```

- lang: 4 bits = 16 languages max
- atom_type: 4 bits = 16 types
- letters: 55 bits = 11 letters × 5 bits each

### Edge (hot, 6 bytes each)
```
┌──────────────────┬──────────────────────┐
│ dst_atom_idx u32 │ packed_meta u16      │
└──────────────────┴──────────────────────┘

packed_meta (16 bits):
  type 5b (32 edge types, 21 used — 3 mothers × 7 directions)
  state_value i4 (-8..+7 bipolar, step 0.125)
  memory_strength u4 (0..15 exponential, Ebbinghaus curve)
  flags u3 (has_context, has_state_dep, deleted)
```

### 21 Edge Types in 3 Mothers (categories)
**Sensory:** visual_color, visual_shape, visual_size, taste_basic, smell_primary, texture, temperature  
**Functional:** use_culinary, use_general, cause_effect, ingredient_of, interacts_with, enables_action, prevents_state  
**Abstract:** category_is_a, analogy_similar, symbolic_cultural, metaphor_for, brand_association, emotional_valence, narrative_archetype

### 3-Axis Context
Spatial (world/place), Temporal (time/year), Identity (self/other/group)

### Storage
```rust
atoms: Vec<u64>
edges_hot: Vec<EdgeHot>       // 6B each
fwd_offsets: Vec<u32>          // CSR
rev_offsets: Vec<u32>
```

---

## v1 Stress-test: 10 Knowledge Scenarios

1. **Multilingual Dictionary** — "לימון = lemon = limón = 柠檬 = ليمون"
2. **Same-language synonyms** — שמחה / עליזות / אושר / ששון (happiness variants)
3. **Antonyms with neutral middle** — חם ↔ קר (with warm/tepid/cool between)
4. **Sentence as atom** — "הילד אכל את התפוח" — with grammar, roles
5. **Full document** — 2000-word essay with sections, refs, cross-references
6. **Procedure/Recipe** — לימונדה with quantities, conditionals, loops
7. **Make-like workflow** — DAG with branches and iteration counts
8. **Media** — images, audio, video with embeddings, timecodes
9. **Math expression** — f(x) = x² + 2x + 1 with symbolic manipulation
10. **Code** — 50-line Python function with AST, scope, control flow

---

## v1 Breaking Points (from GPT-5.4 + Gemini 2.5 Pro, both scored 3-4/10)

### Critical failures (consensus from both reviewers):

1. **u64 letter encoding fatally flawed**
   - 5 bits/letter × 11 letters max = 32 chars per slot
   - Can't represent Chinese (50K ideographs), full Arabic, accented Latin, punctuation, emoji
   - Most English words >11 chars don't fit
   
2. **Word = Concept conflation**
   - Same atom represents word form "lemon" and concept LEMON
   - Fails for polysemy: "lemon" (fruit) vs "lemon" (defective product)
   - Can't represent multilingual dictionary properly

3. **16 atom_types too few**
   - Cyc reached 10,000 relation types after 20 years
   - 4 bits for type = toy scale

4. **21 edge_types missing linguistic**
   - No subject_of / object_of / modifies / has_part
   - Cannot represent sentence grammar

5. **No edge reification**
   - Can't attach confidence, provenance, register, formality to edges
   - Bipolar i4 state_value too crude for nuance

6. **Sequences where trees needed**
   - Sentences need parse trees, documents need DOM trees
   - Formulas need expression trees, code needs ASTs
   - Workflows need DAGs with branches and iteration counts

7. **No slots/defaults** (Minsky Frames)
   - "Lemon" concept needs slots: color=yellow(default), taste=sour(default)
   - Current model cannot express this

8. **No blob store for media**
   - 32-byte atoms can't hold images
   - No plan for external references

9. **No control flow representation**
   - Workflows with conditions, loops, iteration counts have no form

---

## v2 Revised Architecture (addresses all 9 breaking points)

### Change 1: Three-layer Word / Sense / Concept separation (WordNet model)

```
LAYER Word:     "לימון"(he)  "lemon"(en)  "limón"(es)  "柠檬"(zh)
                    ↓            ↓            ↓            ↓
LAYER Sense:   [yellow citrus fruit, sour, small, oval]
                    ↓
LAYER Concept:  CONCEPT#lemon_fruit (language-agnostic)
```

- Words in different languages are separate atoms (WordForm type)
- All words meaning "the fruit" share a single Sense atom
- Sense atoms refer to abstract language-free Concept atoms
- Polysemy handled: one WordForm → multiple Senses → different Concepts

### Change 2: Drop u64-letter encoding — UTF-8 string table

```rust
// WordForm atom content (no longer bit-packed letters):
struct WordFormAtom {
    lang_id: u16,       // 65536 languages (up from 16)
    string_ptr: u32,    // index into global string_table
    length: u16,        // byte length
    flags: u16,         // case, diacritics, morphological features
}

// Global string table
string_table: Vec<u8>,  // UTF-8 all words concatenated  
string_offsets: Vec<u32>,
```

- Full Unicode support (any language, any script)
- No arbitrary length limit
- Handles Chinese (3-byte UTF-8), emoji (4-byte), accented Latin trivially

### Change 3: atom_type expanded 4 bits → 8 bits (16 → 256 types)

Organized in families:

```
Family 0x0x  Lexical:     WordForm, Lemma, Phoneme, Morpheme
Family 0x1x  Semantic:    Sense, Concept, Entity, Category, Property, Value
Family 0x2x  Structure:   Sequence, Set, Tree, DAG, Matrix, Frame
Family 0x3x  Process:     Event, Procedure, Rule, Function, Workflow
Family 0x4x  Language:    Sentence, Paragraph, Document, Formula, Code
Family 0x5x  Media:       MediaRef, Vector, Timeline
Family 0xFx  Meta:        Relation (reified edge), Annotation, Provenance, Context
```

Currently use ~64 types, 192 reserved for future.

### Change 4: edge_type expanded 5 bits → 7 bits (32 → 128 types)

Added linguistic edges that were missing:
- Grammatical: subject_of, object_of, modifies, determines, verb_of, tense_of
- Structural: has_part, part_of, contains, precedes, follows
- Lexical: word_of_language, lemma_of, has_sense, pronounced_as
- Sense-level: represents_concept, near_synonym_of, broader_than, narrower_than, scalar_antonym_of

### Change 5: Reification for edge nuance

Hot edges stay 6 bytes for the 95%. When edge needs nuance (5% of edges):

```rust
// A related_to B with confidence, provenance, register
// Becomes: A → relation_atom → B
// Where relation_atom (type 0xF0) has its own edges:
relation_atom has_confidence 0.85
relation_atom provenance (source_atom, date_atom)
relation_atom register (formal, literary)
```

### Change 6: Trees / DAGs / Frames as first-class atoms

Specialized content interpretations:

- **Tree** (0x22) — hierarchical, for documents/sentences/paragraphs
- **DAG** (0x23) — directed graph for workflows
- **Frame** (0x25) — Minsky-style with slots+defaults
- **Workflow** (0x34) — DAG + iteration counts + branch conditions (Make-style)
- **Sentence** (0x40) — parse tree with grammatical role edges
- **Document** (0x42) — tree of paragraphs → sentences
- **Formula** (0x43) — expression tree
- **Code** (0x44) — AST with symbol table

### Change 7: Media as external reference + Vector atoms

```rust
MediaRef atom (0x50) content:
    uri: string_ptr       // S3 URL, file path, IPFS hash
    mime_type: string_ptr
    duration_ms: u32
    dimensions: (u32, u32)
    embedding_ref: atom_id  // points to Vector atom
    annotations_ref: atom_id // points to Sequence of annotations

Vector atom (0x51) content:
    dim: u16
    model_id: u32
    data: [u8; dim * 2]  // fp16 by default
```

### v2 Storage Cost

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
| **HOT TOTAL** | **1.36 GB** |
| edge_reified (5% of edges) | 20 MB |
| atom_contexts (10% of atoms) | 30 MB |
| vector_store (1% have embeddings) | 100 MB |
| **COLD TOTAL** | **150 MB** |
| **GRAND TOTAL** | **~1.5 GB** |

vs. v1 at 890 MB → **+70% storage for proper multilingual dict + documents + workflows + media + code support.**

---

## The Questions For You

You are reviewing this design as a senior systems architect. Be brutally critical. We already went through one round of critique and revision — we don't need politeness, we need to find what's STILL broken.

### 1. Does v2 actually fix all 9 breaking points from v1?

For each of the 9 breaking points, say YES/PARTIAL/NO and explain specifically.

### 2. Are there NEW breaking points introduced by v2?

Adding complexity sometimes creates new problems. What did we break while fixing?

### 3. Is the Word/Sense/Concept three-layer actually sufficient?

WordNet works for English nouns/verbs/adjectives. Does it handle:
- Hebrew morphology (root+pattern system)
- Chinese compounds (single ideograph can be word OR morpheme OR radical)
- Arabic diglossia (written vs spoken forms)
- Idioms that can't be decomposed ("kick the bucket" = die, but "kick" + "bucket" + "the" doesn't compose)
- Metaphors frozen in language ("understand" = stand under)
- Cognates across related languages (Hebrew/Arabic share many roots)

### 4. Is the Storage (atom_headers + string_table + large content) actually efficient?

Critique the numbers. Where would this break at scale?

### 5. The 10 Scenarios revisited — does v2 handle each one end-to-end?

Walk through each:
1. "לימון = lemon = limón = 柠檬 = ليمון" — show the atoms and edges
2. "שמחה / עליזות / אושר" synonyms with nuance
3. "חם → warm → tepid → cool → cold" scalar
4. Sentence "הילד אכל את התפוח" with parse tree
5. 2000-word document with sections and cross-refs
6. Recipe with 4 lemons + 1 cup sugar + conditional "if too sour, add sugar"
7. Make.com workflow: trigger → filter → loop(N times) → send
8. Image with 3 bounding-box annotations and a 512-dim embedding
9. x² + 2x + 1 as expression tree, differentiate it
10. A 50-line Python function as AST with variable scope

### 6. What's the ONE biggest remaining flaw?

If you had to pick one thing that will cause the most trouble, what is it?

### FINAL VERDICT

Score v2 on a scale of 1-10.  
Compare to v1 (3-4/10).  
Is this good enough to start building in Rust, or does it need another round?
