# ZETS — Learning Content Guide

**Purpose:** Map every kind of content that takes ZETS from "dictionary lookup" to "human-level understanding." For each source, state what exists on the server, what needs to be downloaded, and why it matters.

**Date:** 21.04.2026

---

## Principle: Three layers of understanding

Based on Perplexity's analysis + our architecture, every data source we ingest serves one of three tiers:

| Tier | What it teaches the graph | Without it, graph fails at |
|------|---------------------------|----------------------------|
| **1. Lexical** | Words, senses, synonyms, morphology | "What does X mean?" |
| **2. Relational** | How concepts connect (is-a, part-of, causes) | "How does X relate to Y?" |
| **3. Contextual** | When to pick which meaning; cultural usage | "In this context, what does X really mean?" |

Only all three together approach human-like understanding.

---

## What we ALREADY have on the server

### Tier 1 (Lexical) — DONE ✅
- `enwiktionary.xml.bz2` (1.5 GB) → extracted 10 langs × 1.65M relations
  - 533K definitions, 18K synonyms, 1.9K antonyms, 1.1M POS tags
  - Location: `/home/dinio/zets/data/multilang/{en,de,fr,es,it,he,ar,ru,nl,pt}/`
- `hewiktionary.xml.bz2` (14 MB) → 27K Hebrew definitions, 20K synonyms
- **Homograph proof:** `Gift[DE]=poison` vs `gift[EN]=present` both loaded separately

### Tier 2 (Relational) — PARTIAL ⚠️
- `hewiki-latest.xml.bz2` (1.1 GB) — full Hebrew Wikipedia
- `simplewiki.xml.bz2` (329 MB) — Simple English Wikipedia
- Lev-knowledge extracts: 670K Hebrew Wiki entries (IS_A, PART_OF, CONTAINS)
- Tanakh 39 books (6.1 MB) — sequential text
- **Missing: cross-concept relations extracted, not just definitions**

### Tier 3 (Contextual) — MINIMAL ❌
- Nothing yet. No sentence-level usage, no disambiguation examples.
- This is where real "understanding" begins.

---

## What to add, in order of impact

### Priority 1 — Extract Wikipedia infoboxes (relational data)
**What:** Wikipedia infoboxes contain typed facts (birth date, nationality, occupation, etc.).  
**Why:** Converts raw articles into graph edges automatically. One article → 20+ edges.  
**Source already on server:** `simplewiki.xml.bz2` — 329 MB, manageable.  
**Effort:** 1-2 hours to write extractor.  
**Expected yield:** 5-10M structured edges from Simple English alone.

### Priority 2 — ConceptNet (multilingual relations)
**What:** 34M multilingual assertions like "dog IsA mammal", "car UsedFor transportation".  
**Why:** Ready-made Tier 2 data. Pre-structured, multilingual, free.  
**Source:** Download from https://conceptnet.io/downloads (~1 GB compressed).  
**Effort:** 30 min to download, 1 hour to extract.  
**Expected yield:** 10M+ high-quality edges across 10+ languages.

### Priority 3 — WordNet (English semantic network)
**What:** 117K English words organized into synsets with typed relations.  
**Why:** Gold standard for semantic relationships. Drives serious NLP since 1985.  
**Source:** Already have Wiktionary synonyms. WordNet is richer but English-only.  
**Effort:** 1 hour. WordNet ships as ~50 MB database.  
**Decide:** Add only if Wiktionary's 16K English synonyms prove insufficient.

### Priority 4 — Sentence-level corpora for context
**What:** Annotated corpora showing real usage: "She picked up the crown" (royal/dental/brand?).  
**Why:** This is where Tier 3 (contextual disambiguation) is learned.  
**Source options:**
- Wikipedia itself (articles = annotated sentences)
- OpenSubtitles (movie dialogue) — 50+ languages, free
- Tatoeba — community-translated sentence pairs
**Effort:** 2-3 hours per source.  
**Expected yield:** Millions of sentences with co-occurrence patterns.

### Priority 5 — Domain ontologies (specialized knowledge)
Only when ZETS is used in a specific domain:
- **Medical:** SNOMED CT, ICD-10, DrugBank (requires license for SNOMED)
- **Academic:** CrossRef metadata (free, 130M+ papers)
- **Geography:** GeoNames, OpenStreetMap names
- **Legal:** Eurlex, legislation databases

Skip for now — general knowledge is the priority.

### Priority 6 — Multi-modal grounding (the "human-like" bridge)
**What:** Images with captions, audio transcripts, video subtitles.  
**Why:** Ties abstract words to sensory concepts. "Red" ≠ just a word, it's a color.  
**Source options:**
- COCO captions (free, 120K images with 5 captions each)
- OpenSubtitles with audio timecodes
- Wikipedia images with alt-text
**Effort:** 3-5 hours per modality.  
**Status:** Deferred to V2. Text mastery first.

---

## The concrete 6-week content plan

Given what's already on the server, here's the realistic path:

### Week 1 — Lexicon expansion
- ✅ 10 langs extracted from enwiktionary (done today)
- Add: Wiktionary translations section (cross-language equivalents)
- Add: All 1.9M+ enwiktionary pages (we only processed 1.5M)
- **Target:** 3M+ definitions, 50K+ synonyms

### Week 2 — Wikipedia infobox extractor
- Parse simplewiki.xml.bz2 infoboxes → typed edges
- Parse hewiki for Hebrew infoboxes
- **Target:** 10M+ structured relations (IsA, BornIn, Occupation, etc.)

### Week 3 — ConceptNet ingestion
- Download + parse ConceptNet 5.9
- Filter to our 10 target languages
- **Target:** 10M+ multilingual relations

### Week 4 — Sentence-level usage
- Process simplewiki article bodies (not just infoboxes) into sentence corpus
- Build co-occurrence matrix: which words appear near which
- **Target:** 20M sentences, disambiguation data for top 10K ambiguous words

### Week 5 — Tanakh deep processing
- All 39 books as sequential Flows
- Gematria computed + stored as edges
- Cross-references between verses (already partially in lev-knowledge)

### Week 6 — Cross-language alignment
- Match synsets across languages via translation edges
- "dog" [en] ↔ "כלב" [he] ↔ "Hund" [de] → all point to shared concept node
- **This is what makes the graph language-agnostic, as Idan designed.**

---

## Files we have but haven't used yet

| File | Size | Contains | Status |
|------|------|----------|--------|
| `hewiki-latest.xml.bz2` | 1.1 GB | Full Hebrew Wikipedia | Unprocessed |
| `simplewiki.xml.bz2` | 329 MB | Simple English Wikipedia | Unprocessed |
| `notarikon_results.jsonl` | 1.3 MB | Gematria/Kabbalah results | Unprocessed |
| `rav_mekubal/cortex_import.jsonl` | 73 MB | Jewish texts, commentaries | Unprocessed |
| `rav_mekubal/cortex_import_round2.jsonl` | 24 MB | Additional commentaries | Unprocessed |

**Total unexploited content on server: ~2.5 GB.**

---

## The measurable target

After this plan is executed, the graph should contain:

- **30M+ edges** (vs 493K today)
- **500K+ unique concepts** across 10 languages  
- **Homograph resolution** for 50+ languages × 10K common words
- **Contextual disambiguation** for top 10K ambiguous terms
- **Provenance** on every edge (source traceable)

Running on Pi 5 with ~2 GB RAM footprint. The architecture supports it — the edges are 10 bytes each, so 30M edges = 300 MB.

---

## What this enables, concretely

After Week 6:
- Ask "what is a crown?" in English → see dental/royal/brand senses with confidence
- Ask "מה זה כתר?" in Hebrew → get Hebrew-native answer, same multi-sense breakdown
- Ask "Gift" with no language hint → ZETS shows BOTH `gift[EN]=present` AND `Gift[DE]=poison`, lets you pick
- Follow a chain: "dog → mammal → animal → living being" in any language
- Cross-reference: "what countries border France?" → structured edges from Wikipedia infobox
- Disambiguate in context: "I saw a crown at the dentist" → auto-picks dental sense

This is not an LLM. This is a deterministic knowledge engine that understands what users mean because the structure of the graph mirrors the structure of human concepts.

---

**End of learning content guide.**
