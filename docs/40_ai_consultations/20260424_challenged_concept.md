# ZETS Atom/Edge/Knowledge Model — The Current Concept (for Stress-Testing)

## Core Architecture (15 Principles Combined)

### Atom (the fundamental unit)

**Hybrid storage:**
- 95% atoms: u64 fixed (8 bytes each, O(1) random access)
- 5% atoms: dynamic bytes (rules, functions, long content)

**u64 encoding (small atoms):**
```
┌──────────┬──────────┬─────────────────────────────────┬────┐
│ lang (4b)│ type (4b)│ letters (55b = 11 × 5b)          │ 1b │
└──────────┴──────────┴─────────────────────────────────┴────┘
```

**Dynamic encoding (large atoms):**
```
HEADER (2 bytes):  [lang 4b][type 4b][size_class 4b][flags 4b]
CONTENT (2-32 bytes): interpretation depends on atom_type
```

### atom_type values (4 bits = 16 types)
- 0x0 concept  — word (letter encoding)
- 0x1 entity   — named entity
- 0x2 event    — event + time
- 0x3 rule     — [opcode][condition_atom_id][action_atom_id]
- 0x4 function — [fn_opcode][params]
- 0x5 template — pattern with placeholders
- 0x6 formula  — math expression
- 0x7 executable — wasm bytecode
- 0x8 sequence — ordered list [atom_id, atom_id, ...]
- 0x9 set      — unordered bag {atom_id, ...}

### Edge (hot path, 6 bytes each)
```
┌──────────────────┬──────────────────────┐
│ dst_atom_idx u32 │ packed_meta u16      │
└──────────────────┴──────────────────────┘

packed_meta (16 bits):
  type 5b (32 edge types, 21 used — 3 mothers × 7 directions)
  state_value i4 (-8..+7 bipolar, step 0.125)
  memory_strength u4 (0..15 exponential, Ebbinghaus)
  flags u3 (has_context, has_state_dep, deleted)
```

### Storage — CSR + Index-based (not pointers)
```rust
atoms_small: Vec<u64>           // 8B × 95% of atoms
atoms_large_data: Vec<u8>        // concatenated bytes for 5% large
atoms_large_offsets: Vec<u32>    // offset per large atom
edges_hot: Vec<EdgeHot>          // 6B each
fwd_offsets: Vec<u32>            // CSR forward
rev_offsets: Vec<u32>            // CSR reverse
```

### The 21 Edge Types (3 mothers × 7 directions)
**Sensory:** visual_color, visual_shape, visual_size, taste_basic, smell_primary, texture, temperature
**Functional:** use_culinary, use_general, cause_effect, ingredient_of, interacts_with, enables_action, prevents_state
**Abstract:** category_is_a, analogy_similar, symbolic_cultural, metaphor_for, brand_association, emotional_valence, narrative_archetype

### 3-Axis Context (orthogonal, not tree)
- Spatial (world, place)
- Temporal (time, year, era)
- Identity (self, other, group)

---

## The Challenge: Can This Handle EVERYTHING?

Idan asks: "Is this also a dictionary? Because there are languages, synonyms, opposites, and also sentences, articles, procedures, media types, documents..."

**The challenge scenarios:**

### SCENARIO 1 — Multilingual Dictionary
User writes: "לימון = lemon = limón = 柠檬 = ليمون"
- Same concept, 5 languages
- Must relate these atoms
- Cross-lingual synonyms

### SCENARIO 2 — Synonyms in one language
Hebrew: שמחה / עליזות / אושר / ששון — all "happiness"
- Subtle distinctions (formality, register, nuance)
- Not exact equality but close

### SCENARIO 3 — Antonyms/Opposites
חם ↔ קר, טוב ↔ רע, אהבה ↔ שנאה
- Bipolar relationships
- Some have a "neutral middle" (warm → tepid → cool)

### SCENARIO 4 — Sentence as an Atom?
"הילד אכל את התפוח"
- 4 content words + structure
- Is this one atom? Or 4 atoms + sequence?
- How to preserve grammar?

### SCENARIO 5 — Full Article/Document
A 2000-word essay on climate change.
- Title, abstract, sections, references
- Internal cross-references
- Can ZETS store this?

### SCENARIO 6 — Procedure/Recipe
"להכנת לימונדה: קח 4 לימונים, סחט, הוסף 1 כוס סוכר, ערבב עם 1 ליטר מים, קרר"
- Sequence of steps with quantities
- Conditional branches ("if too sour, add sugar")
- Iteration ("stir 10 times")

### SCENARIO 7 — Make-like Automation Flow
Make.com scenario: Trigger → Filter → Transform → Loop (N times) → Send
- Branches with conditions
- Loops with iteration counts
- Layers/modules

### SCENARIO 8 — Media Types
Images, audio, video — not text.
- Feature vectors (embeddings)?
- Timecodes?
- Spatial coordinates?

### SCENARIO 9 — Mathematical expression
"f(x) = x² + 2x + 1"
- Variables, operators, constants
- Evaluate with substitution
- Symbolic manipulation

### SCENARIO 10 — Code / Program
A Python function with 50 lines.
- Syntax tree
- Variable bindings
- Execution semantics

---

## The Question We Need Answered

For each of the 10 scenarios above:
1. Does the current atom/edge model handle it well?
2. If not — what breaks specifically?
3. What structural addition would be needed?
4. Is there an alternative (array, matrix, tree with layer-count like Make)?

---

## Idan's Intuition (Must Validate)

Idan suggests:
> "Maybe in some cases we need structures like arrays, or paths with descriptions, or matrices/trees with layer-count like Make.com automations."

This implies:
- Not everything fits as (atom, edges). Some need (ordered structure, stepwise, iterable).
- The atom system might need a complementary structure.

**Hypothesis to test:** Add atom_type 0xA "workflow" that stores a DAG with iteration counts, branch conditions, and layer semantics.
