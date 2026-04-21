# ZETS — Learning Content Guide V2

**Date:** 21.04.2026
**Supersedes:** LEARNING_CONTENT_GUIDE.md (V1)
**Purpose:** Map every content type that pushes ZETS toward human-level understanding. For each — what exists on server, what to fetch, realistic effort estimate, expected graph yield.

---

## The 8 content tiers (from Perplexity's analysis, ranked by impact for ZETS)

Perplexity identified 8 categories of content. Each adds a different dimension to the graph. Ranked here by **return-on-effort** for our specific architecture:

| # | Category | What it teaches | ZETS status | Priority |
|---|----------|-----------------|-------------|----------|
| 1 | Lexical dictionaries | Word meanings, morphology, synonyms, antonyms | ✅ DONE (1.65M rows, 10 langs) | — |
| 2 | Encyclopedic facts | World model, entities, events | ⚠️ Raw dumps on server, not processed | **HIGH** |
| 3 | Formal ontologies | Typed relations (is-a, causes, part-of) | ❌ Not on server | **HIGH** |
| 4 | Annotated corpora | POS, NER, dependency, semantic roles | ⚠️ POS only from Wiktionary | MEDIUM |
| 5 | Textbooks & papers | Reasoning, domain knowledge | ❌ Not on server | MEDIUM |
| 6 | Thesauri | Style, register, idiom | ⚠️ Partial via Wiktionary syns | LOW (redundant) |
| 7 | Multimodal grounding | Word ↔ image, sound, video | ❌ Not on server | DEFER to V2 |
| 8 | Real-time streams | Current events, trends | ❌ Not integrated | DEFER to V2 |

---

## Tier 1: Lexical dictionaries — STATUS: COMPLETE

**What we built today:**
- Extracted 10 languages from `enwiktionary.xml.bz2` (1.5 GB dump).
- 1.65 million relations: 533K definitions, 18K synonyms, 1.9K antonyms, 1.1M POS tags.
- Homograph proof: `Gift[DE]="poison"` separate from `gift[EN]="present"`.

**Data location on server:** `/home/dinio/zets/data/multilang/{en,de,fr,es,it,he,ar,ru,nl,pt}/*.tsv`

**What still to harvest from this tier (next session, 1-2 hours):**
- Morphological inflection tables (conjugations, declensions)
- Etymology chains (word → origin → cognates)
- Pronunciation/IPA (for multimodal bridge later)
- Usage examples (sentences showing real-world use)
- Translations section (cross-language equivalence, not just homographs)

---

## Tier 2: Encyclopedic facts — STATUS: RAW, UNPROCESSED

**What we have on the server, untouched:**

| File | Size | Content | Value |
|------|------|---------|-------|
| `hewiki-latest.xml.bz2` | 1.1 GB | Full Hebrew Wikipedia (~400K articles) | HIGH |
| `simplewiki.xml.bz2` | 329 MB | Simple English Wikipedia (~200K articles) | HIGH |
| Existing `wiki_full.tsv` | 96 MB | 660K Hebrew entries — first sentences only | MEDIUM |

**Problem with what we already extracted:** Previous passes captured *first sentence only*. That's good for definitions but throws away **infoboxes** — the structured data that's actually graph-shaped.

**What to extract next (2-3 hours):**

### 2a. Infoboxes (structured facts)
Every Wikipedia article with `{{Infobox ...}}` contains pre-structured facts:
```
Albert Einstein
  born: 1879-03-14
  born_in: Ulm, Germany
  occupation: [physicist, philosopher]
  field: [general relativity, photoelectric effect]
  nobel_prize: 1921
```

Each infobox → 20-50 graph edges. 200K articles × 30 edges avg = **6M typed edges**.

### 2b. First paragraph categorization
The lead paragraph defines what the article is *about*. Patterns like:
- "X is a Y..." → `X IsA Y` edge
- "X was born in Y..." → `X BornIn Y` edge
- "X, founded by Y..." → `X FoundedBy Y` edge

Yield: ~200K more typed edges from Simple English alone.

### 2c. Category links (hierarchy)
Every article has `[[Category:X]]` tags at bottom. These form a tree:
```
Albert Einstein
  → Category:Physicists → Category:Scientists → Category:People
```

Yield: deep is-a chains for every entity. ~500K hierarchy edges.

**Why this matters for "human-level understanding":**
Wiktionary tells us `crown` has 5 senses. Wikipedia tells us **which crown** — Queen Elizabeth II wore a specific one (Imperial State Crown), which is stored in the Tower of London, which was built in 1066. That chain of facts = reasoning substrate.

---

## Tier 3: Formal ontologies — STATUS: NEED TO DOWNLOAD

Perplexity specifically mentioned: Wikidata, Gene Ontology, SNOMED CT, FrameNet, PropBank.

**For ZETS, most valuable to download:**

### 3a. ConceptNet 5.9 (TOP PRIORITY)
- **URL:** `https://conceptnet.s3.amazonaws.com/downloads/2019/edges/conceptnet-assertions-5.7.0.csv.gz`
- **Size:** ~1 GB compressed, ~4 GB uncompressed
- **Contains:** 34M typed assertions across 80+ languages
- **Example edges:** `(dog, IsA, mammal)`, `(coffee, UsedFor, waking_up)`, `(rain, Causes, wet_ground)`
- **Effort:** 30 min download + 2 hours to filter/ingest
- **Yield:** 10M+ edges after filtering to our 10 target languages
- **Impact:** This is the single biggest jump in reasoning capability we can get

### 3b. Wikidata subset (SECOND PRIORITY)
- **URL:** `https://dumps.wikimedia.org/wikidatawiki/entities/latest-all.json.bz2`
- **Size:** ~90 GB compressed, ~1.5 TB uncompressed — **too big for direct use**
- **Alternative:** Use Wikidata Query Service to extract specific subsets
- **Example query:** "all humans with birth date, death date, occupation"
- **Effort:** 3-5 hours for targeted extraction
- **Yield:** 10-50M entity facts
- **Decision:** Skip full dump. Pull on-demand via SPARQL endpoint when specific entities needed.

### 3c. WordNet (English semantic network)
- **URL:** `https://wordnetcode.princeton.edu/3.0/WordNet-3.0.tar.gz`
- **Size:** 12 MB compressed
- **Contains:** 117K English words organized into 82K synsets with 5 typed relations
- **Effort:** 1 hour download + integrate
- **Yield:** 150K high-quality English edges
- **Decision:** Low marginal value — we already have 265K English definitions + 16K synonyms from Wiktionary. Add only if quality > quantity need arises.

---

## Tier 4: Annotated corpora — STATUS: HAVE RAW, NO ANNOTATIONS

Perplexity mentioned Penn Treebank, Hebrew Treebank, FrameNet, PropBank.

**Reality check:** True annotated treebanks (with parse trees, NER tags, semantic roles) are **LICENSED products**. Penn Treebank costs $$, Hebrew Treebank requires academic agreement. These are not download-and-go.

**ZETS pragmatic alternative:**

### 4a. Derive annotations from Wiktionary + Wikipedia
- **POS tags:** Already have 1.1M from Wiktionary.
- **NER (Named Entity Recognition):** Every Wikipedia title IS a named entity. Cross-reference mentions in article bodies → automatic NER corpus.
- **Relations:** Infobox-derived (tier 2a).

**Estimated yield without license:**
- 200K entity names from Wikipedia
- 50K+ relation triples from infoboxes
- 1.1M POS-tagged word instances from Wiktionary

**Decision:** Skip paid treebanks. Build our own from already-available sources. Good enough for ZETS's purpose.

---

## Tier 5: Textbooks & academic papers — STATUS: DEFERRED

Perplexity mentioned: Semantic Scholar, OpenAlex, arXiv, GitHub, Stack Overflow.

**For ZETS in 2026:** Not the right fit yet. Textbooks add reasoning patterns the current graph can't consume — they assume the reader can do multi-step inference from procedural descriptions.

**When this becomes valuable:** After Tiers 2 + 3 are complete and ZETS has a solid world model. Then textbook content teaches *procedures* (how to solve a math problem, how to diagnose a disease).

**Defer to V3 of the content plan.**

---

## Tier 6: Thesauri — STATUS: ADEQUATE FROM WIKTIONARY

Perplexity mentioned Wordnik (1.7M English words with examples).

**Decision:** Wiktionary's synonym/antonym tables give us 80% of thesaurus value. Dedicated thesaurus downloads are **redundant** until we exceed current coverage.

Skip unless specific quality gap emerges.

---

## Tier 7: Multimodal grounding — DEFERRED TO V2

Perplexity mentioned COCO (images), LibriSpeech (audio), video corpora.

**Reasoning for deferral:**
1. ZETS's first target users (CHOOZ, DINIO) don't need image-grounded knowledge
2. Multimodal encoding requires 100s of MB per 1K images
3. Text mastery is prerequisite — walk before run

**When to add:** Only once ZETS serves real text queries reliably. Multimodal is a V2 expansion, not a V1 requirement.

---

## Tier 8: Real-time streams — DEFERRED TO V2

Perplexity mentioned blogs, news feeds, Stack Overflow.

**Reasoning for deferral:**
1. Real-time updates require an ingestion daemon (operational complexity)
2. Quality is inconsistent — needs filtering
3. Graph determinism breaks if facts change under us

**When to add:** After V1 is stable and a "temporal edge" type is designed (edges with timestamp + expiry).

---

## Hebrew-specific: Kabbalah/Tanakh content

We have 100MB of Hebrew religious texts on the server that Perplexity didn't mention:
- `rav_mekubal/cortex_import.jsonl` (73 MB) — commentaries, Kabbalistic texts
- `data/tanakh/` (6.1 MB) — 39 books
- `notarikon_results.jsonl` (1.3 MB) — gematria/notarikon computed results

**Value:** This is UNIQUE content. No other AI system has it structured as a graph. For the Lev project specifically — this IS the core.

**Effort:** 2-3 hours to parse JSONL → graph edges.

**Yield:**
- 10K+ concept nodes (sefirot, angels, gates, letters)
- 50K+ cross-references between Tanakh verses
- Gematria values as numeric edges

---

## Execution order (recommended for next 3 sessions)

### Session 1 — finish lexical + start encyclopedic (4 hours)
1. Process remaining ~400K enwiktionary pages (already running)
2. Extract translations section from Wiktionary (cross-language equivalents)
3. Write simplewiki infobox extractor → first 10K articles as POC
4. **Target delivery:** 2M+ lexical rows + 500K infobox facts

### Session 2 — ConceptNet + deep Wikipedia (5 hours)
1. Download ConceptNet 5.9 (1 GB)
2. Filter to 10 target languages, ingest
3. Finish simplewiki infobox extraction (all 200K articles)
4. Extract Wikipedia category hierarchies
5. **Target delivery:** 15M+ relations in graph

### Session 3 — Hebrew deep content (4 hours)
1. Parse rav_mekubal JSONLs into graph edges
2. Process Tanakh as sequential Flows (one book = one Flow)
3. Cross-reference verses by gematria equivalence
4. **Target delivery:** Full Hebrew knowledge layer — unique competitive advantage

After 3 sessions: graph contains ~20M edges across 10 languages + deep Hebrew specialty. This exceeds most commercial knowledge graphs for our target domain.

---

## What "human-level understanding" actually requires (and what it doesn't)

Perplexity used the phrase "human-level understanding" freely. Reality check:

**What the full 8-tier content will enable ZETS to do:**
- Answer factual questions about 500K+ entities with provenance
- Disambiguate homographs across 10 languages
- Follow is-a and part-of chains for reasoning ("is a dolphin a mammal?" → yes, via is-a chain)
- Detect contradictions between sources
- Generate multi-sentence explanations using deterministic templates
- Cite sources for every claim

**What it WILL NOT enable:**
- Creative writing, humor, sarcasm (requires LLM)
- Novel reasoning beyond what's in the graph (no inference beyond walks)
- Emotional intelligence, theory of mind (requires much more)
- Real-time conversation that feels fully human (requires LLM for syntax)

**Honest conclusion:** The 8-tier content gets ZETS to "dictionary/encyclopedia/atlas rolled into one, deterministic, with citations." That's already a commercial-grade product. It is NOT AGI. It doesn't need to be — it fills a niche no LLM fills (determinism + provenance + tiny compute footprint).

---

## Summary decision tree

**If you want maximum ZETS capability with minimum effort:**
1. Finish Wiktionary extraction (already queued) → 2 more hours
2. Add ConceptNet → 3 hours, biggest capability jump
3. Infoboxes from simplewiki → 2 hours, structured facts
4. **STOP HERE** for V1 release. Rest is incremental.

**If you want Hebrew/Jewish-specialty edge:**
5. Process rav_mekubal + Tanakh deeply → 4 hours
6. This is what makes Lev unique and Zets differentiated

**If you want multimodal later (V2):**
7. Add WordNet (for English depth)
8. Add COCO captions (for image grounding)
9. Add LibriSpeech (for audio grounding)

**Defer indefinitely:**
- Paid corpora (Penn Treebank, SNOMED CT)
- Full Wikidata dump (90 GB — use SPARQL on demand instead)
- Real-time news feeds (operational burden > value)

---

**End of learning content guide V2.**
