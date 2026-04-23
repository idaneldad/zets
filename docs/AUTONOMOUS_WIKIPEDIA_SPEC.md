# Autonomous Wikipedia Ingestion + Cross-Language Knowledge Consolidation

**Last updated:** 23 April 2026
**Status:** Design spec — ready for Claude Code delegation

---

## The Question

> "האם ZETS כבר יודע לטעון את כל ויקיפדיה בכל השפות ברקע, בלעדיי?"

## The Answer (honest audit)

### ✅ What already works (Python side)

**Infrastructure (`mcp/autonomous/multi_lang_wiki.py`):**
- Downloads Wikipedia dumps politely (max 2 concurrent, respects robots.txt)
- Parses XML → JSONL streaming
- Deletes raw .bz2 after parse (saves disk)
- Per-language progress tracking with resume capability
- 48 languages already downloaded and parsed
- Top 15 languages: **500,000 articles each** written to JSONL

**Current state (23.04.2026):**
```
Total: 48 languages parsed
Total parsed articles: ~15M+
Total written to JSONL: ~7.5M (capped at 500K per language)
Disk used: 17 GB
```

### ❌ What does NOT work

**The parsed JSONL files have NEVER been ingested into the ZETS AtomStore.**

Only my `benchmark_wikipedia.rs` (written today) knows how to read them — and only for a benchmark, not for persistent learning.

There is no:
1. Autonomous Rust-side ingestion loop that consumes JSONL → AtomStore → persist
2. Cross-language linking pipeline (Hebrew "שלום" ↔ English "peace" via shared senses)
3. Coverage verification ("is this concept truly unique, or are we missing translations?")
4. Translation discovery ("this concept has no translation — fetch more sources")
5. Cross-language compression (shared atom reuse across languages)

**This document specifies all five.**

---

## The Four Phases — What Needs to Be Built

### Phase A — Autonomous Ingestion Loop

**Goal:** ZETS continuously consumes the parsed JSONL corpus into its AtomStore, persisting state between runs. No human in the loop.

**Components:**

```rust
// src/autonomous/wiki_ingester.rs — NEW

pub struct WikiAutonomousIngester {
    store: AtomStore,
    progress: IngestionProgress,     // per-language cursor
    persist_path: PathBuf,
    config: IngestConfig,
    cross_lang: CrossLanguageLinker,  // Phase B
    coverage: CoverageValidator,      // Phase C
}

impl WikiAutonomousIngester {
    /// Main loop — consumes N articles per tick, persists state.
    pub fn tick(&mut self, articles_per_tick: usize) -> TickReport;
    
    /// Takes a language, reads N articles from its JSONL cursor, 
    /// ingests each, links cross-language, persists.
    pub fn process_language(&mut self, lang: LanguageId, n: usize) -> LangReport;
}
```

**Progress file format (`data/autonomous/wiki_progress.json`):**
```json
{
  "last_tick_ts": 1719923842,
  "languages": {
    "he": { "jsonl_offset_bytes": 12845632, "articles_ingested": 23045, "atoms_created": 178439 },
    "en": { "jsonl_offset_bytes": 8943221,  "articles_ingested": 15320, "atoms_created": 245992 },
    ...
  },
  "total_articles": 872341,
  "total_atoms": 5432109,
  "total_edges": 98765432
}
```

**Deploy model:**
- Cron job: every 5 min, run `cargo run --release --bin wiki_ingester_tick -- --n=200`
- 200 articles per tick × 12 ticks/hour × 24 hours = 57,600 articles/day
- 7.5M parsed articles → 130 days continuous (acceptable)
- But: we don't need all. **MVP target: top 10 languages × 10K articles each = 100K articles = ~2 days**

**Deliverable:**
- `src/autonomous/wiki_ingester.rs`
- `src/bin/wiki_ingester_tick.rs` (CLI entry)
- Cron script + systemd service
- Test: 100 articles ingested + re-persisted + re-loaded = identical graph

---

### Phase B — Cross-Language Linking

**Goal:** When ZETS ingests an article in Hebrew about "Einstein" and later sees an article in English about "Einstein," the two become **linked** in the graph — sharing knowledge without duplication.

**The key insight (from `sense_graph.rs`):**

> ❌ WRONG: `word:שלום --SAME_AS--> word:hello`
> ✅ RIGHT: Both words link to shared **sense atoms** (greeting, peace-state, etc.)

**Why wrong?** שלום has 3 senses (greeting.open, greeting.close, peace.state). hello has only one (greeting.open). Direct SAME_AS loses information.

**Why right?** Shared senses form a cross-lingual backbone. Each word → its own senses → shared sense atoms.

**Components:**

```rust
// src/cross_lang/linker.rs — NEW (building on sense_graph.rs)

pub struct CrossLanguageLinker {
    sense_graph: SenseGraph,                   // existing
    title_index: HashMap<String, Vec<AtomId>>, // title → atoms across langs
    interlingual_links: HashMap<AtomId, Vec<AtomId>>,  // from Wikipedia's langlinks
}

impl CrossLanguageLinker {
    /// Called after each article is ingested.
    /// Links this article to its translations in other languages (if they exist).
    pub fn link_article(&mut self, 
        article_atom: AtomId, 
        title: &str, 
        lang: LanguageId,
        wiki_langlinks: Option<&[WikiLangLink]>
    ) -> Vec<LinkCreated>;
    
    /// Fingerprint-based: same entity across langs via sense graph overlap.
    pub fn find_sister_articles(&self, atom: AtomId) -> Vec<(AtomId, f32)>;
}

pub struct LinkCreated {
    pub from: AtomId,
    pub to: AtomId,
    pub evidence: LinkEvidence,  // WikiLangLinks | SenseOverlap | TitleMatch | Manual
    pub confidence: f32,
}
```

**Data source for langlinks:** Wikipedia dumps include a `langlinks` section per article. We can extract during parse.

**Algorithm:**

1. **Fast path — Wikipedia langlinks:** If the parsed article has langlinks, use them directly. Each langlink = explicit SAME_AS with confidence 1.0.

2. **Medium path — title normalization:** For entities with clear titles (people, places), normalize ("Albert Einstein" == "אלברט איינשטיין" == "アルベルト・アインシュタイン") via shared Wikidata QID if available.

3. **Slow path — sense overlap:** After article ingested, walk the sense graph. If Hebrew article about "X" has >80% sense overlap with English article about "X" candidate, propose SISTER_OF edge.

**Deliverable:**
- `src/cross_lang/linker.rs` 
- `src/cross_lang/wiki_langlinks_parser.rs` (read langlinks from Wikipedia dumps)
- Test: Hebrew "אינשטיין" + English "Einstein" → SISTER_OF edge created with evidence=WikiLangLinks

---

### Phase C — Coverage Verification ("uniqueness audit")

**Idan's insight:**

> "אם חומר מופיע רק פעם אחת ואין לו תרגום — יש לברר אם זה נכון. כמעט אין מידע שאמור להיות ייחודי לגמרי בלי אזכור."

**Goal:** Periodically audit the graph for atoms that are suspiciously isolated — concepts mentioned only once, in only one language, with no cross-references. Flag them for investigation.

**Components:**

```rust
// src/validation/coverage.rs — NEW

pub struct CoverageValidator {
    store: &AtomStore,
}

#[derive(Debug)]
pub struct SuspiciousAtom {
    pub atom_id: AtomId,
    pub label: String,
    pub languages_seen: HashSet<LanguageId>,
    pub reference_count: usize,
    pub suspicion_score: f32,   // 0..1 — higher = more suspicious
    pub reason: SuspicionReason,
}

pub enum SuspicionReason {
    SingleLanguageOnly { lang: LanguageId },       // Only in one language
    SingleMention,                                  // Only one article mentions it
    NoTranslationFound,                             // In >1 lang but no SISTER_OF
    OrphanConcept { incoming_edges: usize },       // <3 incoming edges
}

impl CoverageValidator {
    /// Walk the graph, find suspicious atoms.
    pub fn audit(&self) -> CoverageReport;
    
    /// For a suspicious atom, propose external sources to check.
    pub fn propose_external_lookup(&self, atom: &SuspiciousAtom) -> Vec<ExternalLookup>;
}

pub struct ExternalLookup {
    pub atom_id: AtomId,
    pub query: String,
    pub sources: Vec<LookupSource>,  // Wikidata, Wikipedia (other langs), DBpedia, web search
    pub reason: String,
}
```

**Heuristics for "suspicious":**

| Signal | Suspicion raise |
|--------|:---------------:|
| Only seen in 1 language | +0.3 |
| < 3 incoming edges | +0.2 |
| Referenced only by source-atom (article title) | +0.3 |
| Named entity (capitalized / proper noun) — strong signal translation should exist | +0.2 |
| Known topic class (person, place, company, event) | +0.2 per class |
| No langlinks from any lang | +0.3 |

Threshold: score > 0.6 → flag for external lookup.

**Deliverable:**
- `src/validation/coverage.rs`
- `src/bin/coverage_audit.rs` (scheduled: run once a day)
- Output: `data/validation/suspicious_atoms_YYYYMMDD.jsonl`

---

### Phase D — Translation Discovery & Web Augmentation

**Goal:** For atoms flagged as suspicious in Phase C, autonomously seek external sources to confirm/enrich/translate.

**Components:**

```rust
// src/autonomous/translation_discovery.rs — NEW

pub struct TranslationDiscoverer {
    lookup_queue: VecDeque<ExternalLookup>,
    cache: HashMap<String, LookupResult>,  // dedupe
    rate_limiter: RateLimiter,
    budget_tracker: BudgetTracker,
}

pub enum LookupSource {
    WikidataQid,                       // look up entity Q-number
    WikipediaOtherLanguage { lang },   // fetch same article in other lang
    DBpedia,                           // linked-data source
    WebSearch { query },               // last resort — search the web
    LLMTranslation { to_lang },        // ask LLM if all else fails
}

impl TranslationDiscoverer {
    /// Pick next item from queue, execute lookup, enrich the graph.
    pub fn process_next(&mut self, store: &mut AtomStore) -> DiscoveryResult;
}
```

**Flow per suspicious atom:**

1. **Wikidata first** — if atom has a Wikidata QID hint (from title match or existing link), fetch `sitelinks` for that QID. Returns "this entity exists in EN, HE, AR, ZH, ..." — now we know what translations should exist.

2. **Wikipedia cross-language** — if Wikidata says it exists in language X but we don't have the article, fetch from Wikipedia API. Ingest it. Link to original.

3. **DBpedia** — for entities not in Wikidata but in DBpedia (academic/scientific knowledge).

4. **Web search (polite)** — broader web lookup with source evaluation via `trust_scorer.py` and `source_tiers.py` (already exist!).

5. **LLM translation (cost-gated)** — if atom has concept in one lang and no sources found elsewhere, use CapabilityOrchestrator (exists!) to query Gemini Flash for translation. Tag the result as `EpistemicStatus::Tradition` or similar low-confidence.

**Integration with CapabilityRuntime** (landed today):
- All external calls go through CapabilityOrchestrator
- Budget tracking: max $X/day for LLM translation
- Audit trail: every enriched edge has source = capability call ID
- Rate limiting: respect each source's API limits

**Deliverable:**
- `src/autonomous/translation_discovery.rs`
- `src/bin/discover_tick.rs` (consumes suspicious_atoms queue, runs lookups)
- Wikidata API adapter, Wikipedia API adapter
- Test: "given suspicious atom X, lookup enriches graph with Y new cross-lingual edges"

---

## Phase E — Cross-Language Atom Compression (Pruning)

This is the **most interesting** technical challenge.

### Idan's question:

> "יש דרך גם ליעל חיבור בין שפות כדי לשים גם בזה פחות צמתים?"

### The problem

If we ingest Hebrew Wikipedia + English Wikipedia + Arabic Wikipedia, we'll have:
- Hebrew articles: ~500K × ~100 atoms/article = 50M atoms
- English articles: ~500K × ~100 atoms/article = 50M atoms
- Arabic articles: ~500K × ~100 atoms/article = 50M atoms
- Total: **~150M atoms** — but a HUGE fraction are duplicates ("Albert Einstein" is one person, referenced 500 times across 500 languages).

### The concept — 3 levels of sharing

```
┌─────────────────────────────────────────────────────────┐
│   Level 3: Concept Atom (language-free)                  │
│   atom:concept:person:einstein                           │
│   Birth: 1879, Nationality: German-American              │
│   (shared across ALL languages)                          │
└───────────────────┬─────────────────────────────────────┘
                    │ REALIZED_AS
        ┌───────────┼───────────┬───────────┐
        ▼           ▼           ▼           ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ Level 2:     │ │ Level 2:     │ │ Level 2:     │
│ Word Atom HE │ │ Word Atom EN │ │ Word Atom AR │
│ word:אינשטיין│ │ word:Einstein│ │ word:آينشتاين│
└──────┬───────┘ └──────┬───────┘ └──────┬───────┘
       │ HAS_SENSE      │ HAS_SENSE      │ HAS_SENSE
       ▼                ▼                ▼
┌──────────────────────────────────────────────┐
│  Level 1: Sense Atom (shared)                │
│  sense:einstein.person.physicist             │
└──────────────────────────────────────────────┘
```

### The rules

1. **Proper nouns (people, places, organizations, events):**
   - Single **concept atom** at the top, language-free
   - N word atoms (one per language) — these are just strings
   - One or more sense atoms per concept
   - Storage cost: 1 concept + N words + M senses (M << N usually)

2. **Common nouns (everyday objects, abstract concepts):**
   - NO single concept atom (because "apple" in Hebrew ≠ "apple" in Russian for certain uses)
   - Word atoms per language
   - Shared sense atoms where they truly overlap
   - More storage than (1), but fewer false synonyms

3. **Numbers, dates, quantities:**
   - Language-free from the start (1879 is 1879 in every language)
   - Single atom, no word duplication

### The pruning algorithm (incremental)

```rust
// src/optimization/cross_lang_prune.rs — NEW

pub struct CrossLangPruner {
    store: &mut AtomStore,
    dry_run: bool,
}

impl CrossLangPruner {
    /// Main entry — runs periodically, consolidates duplicates.
    pub fn prune(&mut self) -> PruneReport;
    
    /// Step 1: Find candidate groups (words in different languages pointing to the same sense cluster).
    fn find_consolidation_candidates(&self) -> Vec<ConsolidationGroup>;
    
    /// Step 2: For each group, decide if they truly represent the same concept.
    fn verify_same_concept(&self, group: &ConsolidationGroup) -> ConsolidationDecision;
    
    /// Step 3: Create canonical concept atom. Rewire edges. Mark old atoms as Merged.
    fn consolidate(&mut self, group: &ConsolidationGroup, canonical_data: ConceptData);
}

pub struct ConsolidationGroup {
    pub word_atoms: Vec<(LanguageId, AtomId)>,  // same concept in N languages
    pub shared_senses: Vec<SenseId>,             // senses they share
    pub evidence: Vec<ConsolidationEvidence>,   // why we think they're the same
}

pub enum ConsolidationEvidence {
    WikipediaLangLinks { wikidata_qid: String },  // strongest: Wikipedia confirms
    HighSenseOverlap { overlap_pct: f32 },        // senses are >90% shared
    SharedEntityType { entity_type: String },     // both tagged as Person, Place, etc.
    ContextualSimilarity { score: f32 },          // co-occurrence patterns match
}
```

### Pruning decision tree

```
For each candidate group:
├── Has Wikipedia langlinks or Wikidata QID?
│   └── YES → Consolidate with confidence=1.0, evidence=WikipediaLangLinks
│
├── >90% sense overlap?
│   └── YES → Consolidate with confidence=0.9, evidence=HighSenseOverlap
│
├── Shared entity type AND shared senses >70%?
│   └── YES → Consolidate with confidence=0.7, evidence=SharedEntityType
│
└── Else → Keep separate, flag for review
```

### Storage savings (estimated)

Based on the graph size target (ingest top 10 languages × 10K articles each = 100K articles):

| Approach | Atoms | Savings |
|----------|------:|--------:|
| **Naive** (no consolidation) | ~10,000,000 | baseline |
| **Word-level dedup only** (current ZETS behavior) | ~6,000,000 | 40% |
| **+ Sense-level sharing** (sense_graph.rs exists) | ~3,500,000 | 65% |
| **+ Concept-level consolidation** (this proposal) | **~1,800,000** | **82%** |

**5-10× storage reduction is realistic.** Combined with ZETS's 521KB binary + mmap, this makes Edge-scale deployment viable.

### Integration with Canonization (landed today)

The [canonization.md](capabilities/canonization.md) module already handles:
- Variant detection (translations, citations, derivatives)
- Epistemic classification (Religious, Scientific, Traditional)
- QuotePolicy (what can/can't be reproduced)

**Cross-language consolidation extends canonization:**
- Hebrew Genesis + Quran Creation + Norse Ginnungagap → `ParallelTradition` family (already supported!)
- English "World War II" + Russian "Великая Отечественная война" → `Variant` with perspective differences
- Scientific paper in EN + HE + ZH → `Translation` with fidelity scores

The mechanism is general. Wikipedia ingestion just stress-tests it at scale.

---

## The Memory Math — Why Idan's Idea Works

### Idan's insight:

> "אם אנחנו ניקח, נלמד, נמחק ושוב ככה, הדיסק שלנו לא יתמלא אבל יהיה לנו המון ידע טעון."
> "רוב הדלתא בסוף היא בעיקר בסדר הפעולות כי רוב המילים כבר יהיו ורוב הקשרים."

### Why he's right

Zipfian distribution of vocabulary:
- First 10,000 most common words = ~80% of any text
- After 100K articles: most **word atoms** are already there. New articles only add ~50-200 new words each.
- New articles primarily add **edges** (which-word-with-which-word in new contexts) and **sentence atoms** (which are small).

### Projected growth curve

```
Articles ingested   | Total atoms | Delta per 10K articles
          0         |    ~100     |   -
     10,000 (HE)    | 1,000,000   |   1M    (cold start)
    100,000 (HE)    | 5,500,000   |   0.5M  (vocabulary saturates)
  1,000,000 (HE)    | 15,000,000  |   0.1M  (only long-tail vocabulary)
    + 100,000 (EN)  | 20,000,000  |   5M    (new language! cold start again)
    + 1M (EN)       | 30,000,000  |   0.1M  (EN vocabulary saturates)
    + 50 langs      | 80,000,000  |   50M total for all languages
   With consolidation (Phase E) | **15,000,000** | 80% saving |
```

### Disk usage per stage (with mmap)

| Stage | AtomStore size on disk |
|-------|----------------------:|
| Empty | 0 MB |
| HE Wiki (100K articles) | ~500 MB |
| HE + EN (200K articles) | ~1 GB |
| Top 10 langs (1M articles) | ~4 GB |
| Top 10 langs + pruning | **~1-2 GB** |
| All 48 langs + pruning | **~5-8 GB** |

This is **realistic for Master tier** (cloud server with 1TB disk).

For Edge tier, knowledge packages handle the scale-down (as specified in `PRODUCT.md`).

---

## Deployment — How This Runs

### Master tier (our cloud, ddev.chooz.co.il)

```
┌────────────────────────────────────────────────────────┐
│  systemd: zets-ingester.service                         │
│  ├── timer: every 5 min                                 │
│  ├── runs: wiki_ingester_tick --n=200                   │
│  └── if fails N times, emails alert                    │
│                                                          │
│  systemd: zets-crosslink.service (every 30 min)         │
│  ├── runs: crosslink_tick --max=1000                    │
│                                                          │
│  systemd: zets-coverage-audit.service (daily 03:00)    │
│  ├── runs: coverage_audit --full                        │
│  └── produces: data/validation/suspicious_YYYYMMDD.json │
│                                                          │
│  systemd: zets-discover.service (every 15 min)         │
│  ├── runs: discover_tick --budget=$5/day                │
│  └── consumes suspicious queue, enriches graph         │
│                                                          │
│  systemd: zets-prune.service (weekly Sun 04:00)        │
│  ├── runs: prune --dry-run=false                        │
│  └── consolidates duplicate cross-lingual atoms        │
└────────────────────────────────────────────────────────┘
```

### Observability (essential for autonomous systems)

Every tick writes:
```
data/autonomous/ticks/YYYY-MM-DD.jsonl
```

Each line:
```json
{
  "ts": 1719923842,
  "service": "wiki_ingester",
  "articles_processed": 200,
  "atoms_before": 5432109,
  "atoms_after": 5451234,
  "edges_added": 48291,
  "duration_ms": 17842,
  "errors": [],
  "memory_mb": 245
}
```

Dashboard (simple HTML from `/stats` endpoint) reads these, shows growth curves.

---

## Delegation Plan — What Claude Code Will Build

Because this is ~3,000 lines of new Rust + infrastructure, it must be delegated to parallel Claude Code agents.

### Agent W-A (Opus 4.7) — Autonomous Ingester
**Mission:** Build `src/autonomous/wiki_ingester.rs` + `src/bin/wiki_ingester_tick.rs` + systemd service + tests.
**Output:** PR on branch `feat/wiki-autonomous-ingester`
**Tests required:** 10+ tests including persist/resume, crash recovery, progress tracking.

### Agent W-B (Sonnet 4.6) — Cross-Language Linker
**Mission:** Build `src/cross_lang/linker.rs` + Wikipedia langlinks parser.
**Depends on:** Existing `sense_graph.rs`
**Output:** PR on branch `feat/cross-lang-linker`
**Tests required:** 15+ tests, including Hebrew/English/Arabic linking via langlinks and sense overlap.

### Agent W-C (Sonnet 4.6) — Coverage Validator
**Mission:** Build `src/validation/coverage.rs` + `src/bin/coverage_audit.rs`.
**Output:** PR on branch `feat/coverage-audit`
**Tests required:** 10+ tests with fixtures of suspicious vs legitimate atoms.

### Agent W-D (Opus 4.7) — Translation Discovery
**Mission:** Build `src/autonomous/translation_discovery.rs` + Wikidata/DBpedia adapters.
**Depends on:** CapabilityOrchestrator (landed today), Agent W-C output.
**Output:** PR on branch `feat/translation-discovery`
**Tests required:** 12+ tests with mocked external sources.

### Agent W-E (Opus 4.7) — Cross-Language Pruner
**Mission:** Build `src/optimization/cross_lang_prune.rs` + `src/bin/prune_tick.rs`.
**Depends on:** Agents W-A, W-B complete.
**Output:** PR on branch `feat/cross-lang-prune`
**Tests required:** 20+ tests including correctness (no information loss), savings measurement.

### Agent W-F (Sonnet 4.6) — Integration + Systemd
**Mission:** Systemd unit files, observability dashboard, integration tests.
**Output:** PR on branch `feat/wiki-autonomous-integration`
**Depends on:** All above agents merged.

**Total estimated LoC:** ~3,500 Rust + 500 config + docs
**Total estimated tests:** 90+
**Total estimated time:** 6-8 hours agent-wall-time (running 4 agents in parallel = ~2-3 hours)

---

## Risks & Mitigations

| Risk | Likelihood | Mitigation |
|------|:----------:|------------|
| Wikipedia dumps change format | Low | Parser is in Python (`multi_lang_wiki.py`), stable format for decades |
| Ingestion fills disk | Medium | Progress tracking + hard limits per language + disk monitoring |
| Cross-lang false positives (linking wrong things) | Medium | Confidence thresholds, manual review queue, audit trail |
| External APIs rate-limit or change | Medium | CapabilityOrchestrator has rate limits + fallbacks |
| LLM translation cost escalates | High | Hard daily budget cap in `discover_tick` |
| Pruning loses information | Medium | Dry-run mode, full backup before prune, audit log |
| Graph becomes too large for Master server | Low | Multi-instance architecture — shard by language/domain |

---

## Success Metrics — How We Know It Works

### Short-term (2 weeks)
- [ ] 100K articles ingested autonomously across 10 languages
- [ ] >90% of famous entities linked cross-language (spot check 100 manually)
- [ ] Coverage audit identifies >100 suspicious atoms
- [ ] Translation discovery enriches graph with >500 new cross-lingual edges
- [ ] Pruning reduces atom count by >60% without quality loss

### Medium-term (2 months)
- [ ] 1M articles ingested (top 10 languages, ~100K each)
- [ ] Queryable: "give me what ZETS knows about Einstein" returns unified view from all languages
- [ ] Graph size on disk <10 GB after pruning
- [ ] Autonomous loop runs for 30 days without human intervention

### Long-term (6 months)
- [ ] All 48 languages ingested (selective — top 100K articles each)
- [ ] Translation discovery has enriched >50K gaps
- [ ] First external benchmark (LDBC SNB SF-1) runs on this knowledge base
- [ ] Demo: "load this Hebrew book, find what we already know, teach us what's new"

---

## The Value Proposition (for investor deck)

> "ZETS is not a static knowledge graph. It is an **autonomous learner** — once deployed, it continuously ingests the world's public knowledge, links concepts across languages, identifies its own gaps, and seeks external sources to fill them. **Without human intervention.**
>
> In 30 days, ZETS builds a 1M-article knowledge graph across 10 languages using a commodity server. The same graph — via our knowledge-package system — can deploy to an Edge device in under 100 MB.
>
> This is not an LLM fine-tune. It's a different species of AI: one that never forgets, always learns, and can explain every fact it knows."

---

## Next Steps

1. **This session (today):** Commit this spec document to git.
2. **Next session:** Prepare 6 Claude Code agent missions (W-A through W-F), launch 4 in parallel.
3. **2 days later:** Review PRs, merge, deploy Phase A to production.
4. **1 week later:** Phase B + C deployed, coverage audits running.
5. **2 weeks later:** Phase D + E, full autonomous loop live.
6. **1 month later:** First 1M-article milestone, investor demo ready.

---

*This document is the spec. The agents are the builders. The graph is the product.*
