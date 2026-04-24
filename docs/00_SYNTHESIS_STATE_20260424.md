# ZETS Synthesis — End of 2026-04-24

**Purpose:** Single-page map of where ZETS stands after today's shvirat kelim + reconstruction.
**Read this first** if you're coming fresh to the repo tomorrow or later.

---

## What Happened Today (one paragraph)

Idan asked for a serious shvirat kelim — distrust past work, learn only from what verifiably succeeded vs failed, don't recycle mistakes. I (Claude) ran a forensic audit on the 67K lines of Rust, discovered 4 parallel atom systems, abandoned architecture plans, and 637 unwrap() calls. I admitted my own morning Blueprint (1,559 lines of theory) was greenfield fantasy. Then, through deep questioning from Idan about linguistic representation and atom structure, three binding architectural decisions crystallized. An empirical POC validated the approach on real Hebrew + Arabic Wikipedia data.

**We now have a sized architecture, not a dreamed one.**

---

## The Three Binding ADRs (in reading order)

### ADR-1 — Atom as Sigil, Executor as Doer
**File:** `docs/30_decisions/20260424_ARCH_DECISION_atom_sigil_executor_doer.md`

Three-layer architecture:
- **Layer 1 (Graph):** Thin atoms (8 bytes), edges, indexes. Microsecond operations.
- **Layer 2 (Executor Registry):** Named pluggable doers for heavy work (Image/Audio/Video/Code/Doc/Web/etc).
- **Layer 3 (Learning):** Async graph updates from execution results.

**The atom is a sigil, not a container.** Heavy data (documents, media, code, computation) lives in Executors, pointed to by name.

### ADR-2 — Linguistic Representation (Word/Sense/Concept)
**File:** `docs/30_decisions/20260424_ARCH_DECISION_linguistic_representation.md`

Four layers of language:
- **Concept** (language-agnostic, universal reasoning happens here)
- **Sense** (abstract meaning, captures polysemy — שלום has 3 senses)
- **Lemma** (dictionary form, per language, with intrinsic features)
- **WordForm** (surface form with morphological features)

Reasoning happens at Concept layer. Language is I/O. Grammar (gender, number, tense, definiteness) lives on edges/features. Agreement is computed at realization time, not stored.

### ADR-3 — Compressed Semitic-Based Atom Layout
**File:** `docs/30_decisions/20260424_ARCH_DECISION_3_compressed_semitic_atom.md`

Three 8-byte variants covering all languages:
- **HebrewWord**: root (12b) + binyan (3b) + tense (3b) + pgn (4b) + def (1b) + semantic_id (24b) — Hebrew, Arabic, Aramaic share unified Semitic root pool
- **ForeignWord**: language_id (8b) + string_ref (24b) + semantic_id (24b) — English, German, loanwords, names
- **Logographic**: codepoint (24b) + semantic_id (32b) — Chinese, Japanese kanji, Korean hanja

---

## Empirical Grounding

**File:** `docs/20_research/20260424_hebrew_root_compression_poc.md`

Real measurements on Hebrew Wikipedia (3,000 articles, 3.6M tokens):
- **80.8% of unique Hebrew words** fit the 3-letter root model
- **78.1% of tokens** covered
- **656 roots shared between Hebrew and Arabic** (~50% of tokens in each)
- **All 8-byte atom fields validated** against real data

Scripts: `docs/20_research/poc_scripts/` (reproducible).

---

## What These Decisions Together Enable

### For users (the "why it matters" view):

1. **AGI can create code, songs, articles, procedures** — same Associate→Compose→Execute flow, different executors per domain.
2. **Media (image/audio/video)** — external encoders produce vectors, Hopfield banks do recall, graph reasons.
3. **The brain-like parallel stream** (Idan's concert example) — spreading activation with multi-channel seeds; intersections amplify.
4. **Multi-lingual reasoning** — think at Concept layer, realize to any target language.
5. **Hebrew-first without being Hebrew-only** — Semitic compression where it works, foreign-anchor where it doesn't.

### For storage (the "fits on laptop" view):

- 10M atoms × 8 bytes = 80 MB atom core
- Shared root pool: 2,931 entries, ~128 KB
- 1B edges × 6 bytes = 6 GB
- **Total ~6.1 GB — fits 8 GB laptop budget**

### For correctness (the "not an LLM wrapper" view):

- All reasoning in deterministic graph operations
- LLM only at I/O boundary (parse input, realize output)
- Walks are traceable; explanations are the paths taken
- Continuous learning via edge updates, no retraining

---

## What's ALREADY BUILT (keep)

Verified by running code or passing tests (see forensic audit):
- 1,301 unit tests passing
- `autonomous-demo` runs: ingestion + walks + dreaming + persistence work
- `vision-decomposer` runs: Hopfield hierarchical recall proven
- 144K concepts loaded, 22GB data
- Morphology for 14 languages (HE/AR/EN/ES/FR/DE/IT/PT/RU/ZH/JA/VI/TR/NL)
- 35.4% compression measured via BPE fold on Hebrew Wiki
- sense_graph.rs, metacognition.rs, spreading_activation.rs, dreaming.rs, canonization/, distillation.rs

---

## What's BROKEN or DEBT (must fix)

- **4 parallel atom systems** (atoms.rs, graph_v4/, piece_graph.rs, mmap_core.rs) — pick one per ADR-3, archive others
- **637 unwrap() calls** in non-test code — brittle
- **1 failing doctest** (hash_registry.rs Unicode arrow)
- **Performance claims unverified** (2.6MB RAM, 80.8µs latency)
- **Test count drift** (claimed 1,354, actual 1,301)
- **6 abandoned architecture plans** in docs/_archive/

---

## What's SPEC-ONLY (to build)

- P-A: HTTP fetch primitive
- P-B: HTML parser primitive
- P-C: Procedure loader + 20 initial procedure atoms
- P-D: Learning loop executor
- P-E: Seed loader (YAML → atoms)
- P-F: Observability dashboard
- P-G: Zetson infant binary
- P-N/O/Q: Image/Speech/Video pipelines (need CLIP/Whisper integration)

---

## The Roadmap Forward (concrete, prioritized)

### Phase 0 — Archive the old (1 session)
- Move all pre-ADR architecture docs to `docs/90_archive/20260424_pre_adrs/`
- Pick ONE atom system per ADR-3; archive the others to `src/_legacy/`
- Fix the 1 failing doctest
- Freeze a clean `v0.2-adr-based` tag

### Phase 1 — Core atom implementation (1-2 weeks)
- Implement 8-byte `AtomCore` with 3 variants per ADR-3
- Implement Semitic root pool with persistence
- Migration path for existing concepts (144K) to new layout
- Benchmark: encode/decode, RAM usage, root sharing
- Verification checklist from ADR-3

### Phase 2 — Executor Registry (1-2 weeks)
- Formalize `capability_runtime/` as Layer 2 per ADR-1
- Register first 3 executors: TextExecutor, DocExecutor, CodeExecutor (sandbox)
- Define wire protocol (likely Rust trait for built-ins)

### Phase 3 — Linguistic layers (2-3 weeks)
- Concept atom variant (kind=0x5)
- Phrase-lemma atom variant (kind=0x6)
- Agreement rule atoms per ADR-2
- Wire morphology/ as TextExecutor

### Phase 4 — Zetson (the infant) — (3-4 weeks)
- P-A: HTTP fetch (in TextExecutor? or WebExecutor?)
- P-B: HTML parser
- P-E: Seed loader
- P-C: 20 initial procedures (TOML)
- P-D: Learning loop executor (walks procedure DAG)
- P-G: First real Zetson run

### Phase 5 — Media (1-2 months)
- P-N: ImageExecutor with CLIP integration
- P-O: AudioExecutor with Whisper integration
- P-Q: VideoExecutor (keyframe → Image + Audio chain)

---

## Non-Goals (what ZETS will NOT try to be)

- **Not a transformer-style LLM.** Fluency at long-range text is not the goal.
- **Not photorealistic image gen.** External tools (Stable Diffusion, etc.) do that.
- **Not studio music.** External tools (Suno) do that.
- **Not a replacement for Claude.** A different kind of intelligence.

**Instead:**
- Transparent, traceable, continuously learning, edge-device cognition.
- Hebrew-first, multilingual.
- Structural reasoning, not statistical mimicry.
- A system that can explain its own answers.

---

## Open Architecture Questions (carried forward)

From ADR-1:
- Executor wire protocol exact spec
- Executor versioning across atom upgrades
- Can ZETS "learn" new executors?

From ADR-2:
- When to materialize WordForms vs generate on-demand
- Disambiguation of ambiguous parses
- Phrase-lemma vs compound threshold

From ADR-3:
- Root pool federation across instances
- Dialect/drift encoding
- Romanization/Latinization dual-anchor
- Gematria as computed or cached
- Numeral variant layout
- Code/Procedure variant layout (Phase 1 will resolve this)

---

## The Core Principle (re-stated)

Idan's own words, distilled from today's session:

> "Learning is in code. What to learn and how — is in the graph.
> Like a human: knows how to learn, but what to learn is in the graph."

All three ADRs honor this. The 7 primitives (fetch/parse/tokenize/store/retrieve/reason/communicate) are Rust. Everything else — procedures, knowledge, motivations, language rules, creative motifs — lives in graph atoms.

---

## Git State

- Branch: `main`
- Latest commit: ADR-3 compressed Semitic atom (111b36a)
- All decisions and POC results committed + pushed
- Ready for archiving + Phase 0

**When tomorrow starts, read this file first, then the 3 ADRs, then the POC.**
That's the minimal context to continue.

