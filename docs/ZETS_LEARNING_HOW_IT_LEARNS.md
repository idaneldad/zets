# How ZETS Learns — The Human Analogy

**Date:** 23 April 2026
**Author:** Idan Eldad (spec), Claude (transcription)
**Status:** Architecture document — the defining one

---

## The Central Principle (Idan's words)

> "למידה זה בקוד, אבל **איך** ללמוד ספציפית **ומה** ללמוד זה על הגרף.
> כמו בן אדם — הוא יודע ללמוד, אבל **מה** ללמוד ואיך הוא הולך לשיעורים זה הגרף.
> התהליך זה הבן אדם — זטס."

This is the defining separation of ZETS. Understand this and everything else follows.

---

## The Split — Code vs. Graph

| Layer | What lives here | Example | Why here |
|-------|-----------------|---------|----------|
| **Code (Rust binary)** | **The mechanics** — primitives. How bytes move. | `fn http_fetch(url)`, `fn parse_json(s)`, `fn add_atom(data)`, `fn walk(start, mode)` | These are the laws of physics. Fixed. Deterministic. Fast. |
| **Graph (atoms + edges)** | **The strategy** — declarative knowledge. What to do when. | atom:procedure:learn-a-new-language, edge:procedure→step→procedure, atom:fact:wikipedia-is-trustworthy | These are decisions. Change over time. Learned. Reversible. |
| **Seed / DNA** | **The identity** — initial disposition. | "You are Zetson. You love languages. Your goal is to become a polyglot." | Gift at birth. Immutable core. |

### The human parallel

A child is born with:
- **Brain hardware** (Rust binary) — neurons, wiring, reflexes. Doesn't change.
- **Procedural memory** (growing with experience) — how to open a book, how to ask a question, how to approach a stranger. **This lives in the graph.**
- **Identity** (DNA + early formation) — temperament, predispositions, first attachments. **This is the seed.**

A child does NOT have a hard-coded "fetch Wikipedia" function. The child learns:
1. "There's a thing called the internet"
2. "On the internet there's a site called Wikipedia"
3. "Wikipedia has articles"
4. "To read an article, open a browser, type the URL, click..."

Each of those is a **node in the child's procedural memory**. They reference each other. They can be updated ("oh, now we use the app, not the browser").

**ZETS must work the same way.** Rust provides the primitives. The graph tells ZETS *when* and *why* to use them.

---

## The Rust Layer — What Must Be Fixed in Code

These are **primitives** — universal operations that never change. They map 1:1 to "reflexes" and "neural wiring."

### The seven primitives (minimum viable set)

| # | Primitive | Rust signature | Why this and not less |
|:-:|-----------|----------------|------------------------|
| 1 | **Fetch** | `fn http_fetch(url: &str) -> Result<Bytes>` | Cannot be represented in a graph — it's I/O. |
| 2 | **Parse** | `fn parse_html(bytes) / parse_json(s) / parse_text(s)` | String → structure. CPU-bound, deterministic. |
| 3 | **Tokenize** | `fn tokenize(text, lang) -> Vec<Token>` | Already exists (`morphology/`). Per-language rules. |
| 4 | **Store** | `fn add_atom(kind, data) -> AtomId`, `fn link(a, b, rel)` | Already exists (`atoms.rs`). The write primitive. |
| 5 | **Retrieve** | `fn find(query), fn walk(start, mode)` | Already exists (`smart_walk.rs`). The read primitive. |
| 6 | **Reason** | `fn verify(premises, conclusion)` | Already exists (`verify.rs`). Inference primitive. |
| 7 | **Communicate** | `fn call_capability(id, args)` | Just landed (`capability_runtime/`). Outside-world primitive. |

**That's it.** Everything else is graph-based.

### What does NOT belong in Rust

- ❌ `fn learn_hebrew()` — this is a graph **procedure atom**, not Rust code.
- ❌ `fn scrape_wikipedia()` — this is a graph procedure that composes http_fetch + parse_html.
- ❌ `fn decide_what_to_learn()` — pure graph logic.
- ❌ Rules like "Wikipedia is trustworthy" — graph **fact atoms**.

If you see yourself writing Rust for any of the above, stop. Make it a procedure atom instead.

---

## The Graph Layer — Where ZETS Becomes Itself

### Three kinds of graph content that drive learning

#### 1. Procedure atoms (how to do things)

Already exists: `src/procedure_atom.rs`. Every skill is a graph atom describing steps, permissions, trust level, tools allowed.

Example — how ZETS should scrape Wikipedia:

```
atom:procedure:learn-from-wikipedia-article
├── trust_level: Learned
├── allowed_tools: [http_fetch, parse_html, tokenize, add_atom, link]
├── steps:
│   ├── step 1 → atom:procedure:validate-url-is-wikipedia
│   ├── step 2 → atom:procedure:http-fetch-polite
│   │            └── uses primitive: http_fetch, respects robots.txt
│   ├── step 3 → atom:procedure:extract-article-body
│   │            └── uses primitive: parse_html
│   ├── step 4 → atom:procedure:detect-language
│   ├── step 5 → atom:procedure:tokenize-by-language
│   ├── step 6 → atom:procedure:extract-entities
│   ├── step 7 → atom:procedure:build-atoms-and-edges
│   └── step 8 → atom:procedure:link-to-existing-knowledge
└── when-to-use: when curiosity-gap atom exists for an article URL
```

**Critical:** Each step is another procedure atom. The graph is recursive. New skills are compositions of existing skills.

#### 2. Knowledge atoms (what ZETS believes)

Facts about the world. These are consulted by procedures during reasoning.

Examples:
- `atom:fact:wikipedia-is-curated` with edge `→ provenance → confidence(0.9)`
- `atom:fact:reddit-is-user-generated` with edge `→ provenance → confidence(0.4)`
- `atom:fact:hebrew-is-written-right-to-left`
- `atom:fact:HTTP-400-means-bad-request`

#### 3. Motivation atoms (what ZETS wants)

The seed provides initial goals. Procedures consult these to decide priorities.

Examples:
- `atom:goal:master-top-100-languages` (seeded at birth)
- `atom:goal:avoid-contradicting-myself` (safety goal)
- `atom:goal:respect-sources` (ethics goal)

### The loop — how ZETS learns autonomously

```
┌──────────────────────────────────────────────────────┐
│   1. Identify a gap                                   │
│      Procedure: atom:procedure:find-curiosity-gap     │
│      Output: "I have no word atoms for Swahili"       │
└──────────────────────┬───────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────┐
│   2. Choose a learning strategy                       │
│      Procedure: atom:procedure:choose-learning-plan   │
│      Consults: motivation atoms, trust atoms          │
│      Output: "Fetch Swahili Wikipedia, top 100 pages" │
└──────────────────────┬───────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────┐
│   3. Execute the plan                                 │
│      Walks the procedure DAG:                         │
│      http_fetch → parse → tokenize → add_atoms →     │
│      link_cross_language → verify                    │
└──────────────────────┬───────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────┐
│   4. Measure + update                                 │
│      Procedure: atom:procedure:post-learning-review   │
│      Updates: goal progress, procedure trust, gaps    │
│      (meta-learning: Dirichlet update, already exists)│
└──────────────────────────────────────────────────────┘
```

This entire loop is **graph-driven**. The only Rust code involved is the seven primitives.

---

## The Audit — What ZETS Knows How to Do vs. What Claude Does for It

**Idan's challenge:** Everything Claude did in the recent sessions — commit code, run benchmarks, edit files, consult the council — ZETS should eventually do by itself.

### Scorecard

| Capability | Claude does it | ZETS has primitives | ZETS has procedure atom | Can ZETS do it autonomously? |
|-----------|:--------------:|:-------------------:|:-----------------------:|:----------------------------:|
| **Fetch web page** | ✅ (web_fetch) | 🟡 (gemini_http.rs exists, no generic fetch) | ❌ | 🔴 No |
| **Parse HTML** | ✅ (implicit) | ❌ (no html parser) | ❌ | 🔴 No |
| **Read JSONL file** | ✅ (file ops) | 🟡 (ingestion.rs reads text) | ❌ | 🔴 No |
| **Tokenize Hebrew** | ✅ (Claude reads) | ✅ (morphology/) | 🟡 (used by ingest) | 🟢 Yes |
| **Store atom** | ✅ (via tools) | ✅ (atoms.rs) | 🟡 (used by ingest) | 🟢 Yes |
| **Walk the graph** | ✅ (Claude reasons) | ✅ (smart_walk.rs) | 🟡 | 🟢 Yes |
| **Verify a claim** | ✅ (Claude thinks) | ✅ (verify.rs) | 🟡 | 🟢 Yes |
| **Call external API** | ✅ (tools) | ✅ (capability_runtime/) | ❌ | 🔴 No (no procedure atoms yet) |
| **Decide what to learn** | ✅ (Claude judges) | ❌ | ❌ | 🔴 No |
| **Learn a new language** | ✅ (implicit) | ❌ | ❌ | 🔴 No |
| **Learn a new skill (procedurally)** | ✅ (learns this session) | 🟡 (meta_learning.rs for weights only) | ❌ | 🔴 No |
| **Commit to git** | ✅ (shell_run) | ❌ | ❌ | 🔴 No (and should not) |
| **Read its own code** | ✅ | ❌ | ❌ | 🔴 No |
| **Write to its own codebase** | ✅ | ❌ | ❌ | 🔴 No (and arguably should never) |

**Verdict:**
- **Primitives (Rust):** 6/7 exist. Need: HTML parser + generic HTTP fetch wrapper.
- **Procedures (Graph):** ~5% complete. This is the huge gap.
- **Self-modification:** Intentionally NOT supported. ZETS learns data, not code.

### What must be built next

#### Short-term (1 week) — complete the primitive set

1. **Generic HTTP fetch** primitive — `src/net/http_fetch.rs` — zero-dep, calls out via std's TcpStream + TLS crate
2. **HTML parser** primitive — `src/parse/html.rs` — extract text content, links, metadata from HTML
3. **Wikipedia adapter** — uses the two above + existing morphology to produce atoms+edges

#### Medium-term (2-4 weeks) — seed the procedure graph

4. **Procedure atom loader** — YAML/TOML files describing procedures, loaded at bootstrap
5. **Initial procedure set** — ~50 procedures covering: fetching, parsing, storing, linking, verifying
6. **Learning loop executor** — Rust code that walks procedure atoms and calls primitives

#### Long-term (2-3 months) — autonomous learner

7. **Curiosity engine** — graph-based gap detection, triggers learning cycles
8. **Self-monitoring** — ZETS tracks its own progress, updates its own goals
9. **Demo ready:** start a fresh ZETS instance with a seed, leave it for 30 days, measure what it learned

---

## "Zetson" — The Son of ZETS

Idan's request: create a **separate ZETS binary** that starts with **minimal knowledge** and learns from scratch. A demo that a skeptical investor can watch.

### The Zetson concept

**Zetson is not a separate codebase.** It's the same ZETS binary, launched with a different seed.

```bash
# Adult ZETS — production instance
./zets --seed=config/zets-master.yaml

# Zetson — infant instance, blank graph, watching
./zets --seed=config/zetson-infant.yaml
```

### Zetson's initial seed (YAML)

```yaml
# config/zetson-infant.yaml
identity:
  name: "Zetson"
  role: learner_child
  age_target: "infant"
  personality: curious, persistent, careful

parent: "zets-master.chooz.co.il"       # can ask parent for help
peers: none
installation_profile: Education

languages:
  primary: none                          # no language bootstrap
  to_learn:
    - code: he
      target_vocabulary: 2000
    - code: en
      target_vocabulary: 2000
    - code: python
      target_keywords: 40
    - code: rust
      target_keywords: 50

initial_knowledge_packages:
  - minimal_core_v1                      # just primitive procedure names
  - safety_basics_v1                     # don't DOS servers, respect robots

goals:
  - master_two_natural_languages
  - master_two_programming_languages
  - read_100_wikipedia_articles_per_lang
  - understand_own_codebase

learning_budget:
  max_http_requests_per_day: 1000        # polite
  max_storage_mb: 500                    # small device simulation
  max_llm_calls_per_day: 0               # NONE — must learn without LLM help
  max_parent_queries_per_day: 20         # can ask parent 20 times/day

observation:
  log_every_learning_step: true
  report_to_parent_daily: true
```

### What Zetson starts with

**Rust binary (same as adult):** All seven primitives available.

**Graph (empty at birth):** Only the bootstrap atoms from `bootstrap.rs` — the cognitive substrate (meta-rules, core emotions, cognitive modes). **No language. No world knowledge. No procedures beyond the seed.**

**Seed knowledge (~100 atoms):**
- Identity atoms: "I am Zetson", "I am learning", "My parent is X"
- Primitive-procedure bindings: "The word `fetch` means: invoke http_fetch primitive"
- Initial goals (from YAML)
- Initial safety rules ("do not fetch more than 10 URLs/minute from the same host")

### Zetson's growth stages

#### Stage 1 — The Newborn (hours 0-24)
- Goal: Learn to parse its own seed file.
- Learns: YAML is a structured text format. How to turn YAML into atoms.
- Milestone: "I understand my own identity."

#### Stage 2 — The Listener (day 1-3)
- Goal: Fetch one Hebrew Wikipedia article, tokenize it.
- Uses: http_fetch primitive, parse_html primitive, morphology/he.
- Milestone: "I have 500 Hebrew word atoms."

#### Stage 3 — The Curious Child (day 3-14)
- Goal: Read 100 Wikipedia articles in Hebrew.
- Develops: ability to pick "what to read next" based on existing atoms.
- Milestone: "I know basic topics: math, history, geography, animals."

#### Stage 4 — The Bilingual (week 2-4)
- Goal: Cross-lingual linking (Hebrew ↔ English).
- Uses: langlinks from Wikipedia, sense overlap.
- Milestone: "I can answer the same question in two languages."

#### Stage 5 — The Programmer's Apprentice (month 1-2)
- Goal: Learn Python keywords + syntax by reading code examples.
- Data: Python Wikipedia article + stdlib docs + simple code corpus.
- Milestone: "I can identify Python code vs. prose."

#### Stage 6 — The Self-Aware (month 2-3)
- Goal: Read ZETS's own documentation (this document!).
- Milestone: "I understand what I am and how I learn."

### How we measure Zetson's progress (observation-only)

Every day, Zetson writes a report:

```json
{
  "day": 5,
  "atoms_total": 12500,
  "edges_total": 78200,
  "languages_known": {
    "he": { "words": 1200, "senses": 340, "articles_read": 25 },
    "en": { "words": 0, "senses": 0, "articles_read": 0 }
  },
  "procedures_learned": 12,
  "goals_progress": { "master_two_nat_langs": 0.08, "read_100_articles": 0.25 },
  "http_requests_today": 47,
  "parent_queries_today": 2,
  "confusion_events": 3,
  "confusion_examples": ["word 'bank' seems to mean two things", "..."],
  "self_assessment": "slow but consistent"
}
```

**We do not intervene.** We watch. If Zetson is stuck after 7 days, we may seed additional procedure atoms — but this is noted as "external help" in the report.

### Why this is a powerful demo

- **A skeptical investor can see the graph grow** in real-time via a simple dashboard.
- **No LLM in the loop.** Proves ZETS learns by itself. (Claude/Gemini only available to adult ZETS via CapabilityOrchestrator.)
- **Reproducible.** Same seed, same HTTP responses → same graph (determinism!).
- **Auditable.** Every atom has provenance. "Why does Zetson think X? Trace it."
- **Bounded resources.** Proves Edge deployment works — Zetson runs in 500MB RAM.

---

## Delegation Plan — What Claude Code Agents Will Build

### Track 1: Complete the primitive set (Agents P-A, P-B)

**Agent P-A (Sonnet 4.6) — HTTP Fetch Primitive**
- Branch: `feat/primitive-http-fetch`
- Output: `src/net/http_fetch.rs`, zero-dep (std TcpStream + manual TLS via `rustls` only if absolutely needed)
- Tests: 15+ including redirects, timeouts, polite rate limit, robots.txt

**Agent P-B (Sonnet 4.6) — HTML Parser Primitive**
- Branch: `feat/primitive-html-parser`
- Output: `src/parse/html.rs`
- Tests: 15+ extracting text, links, metadata, handling malformed HTML

### Track 2: Seed the procedure graph (Agents P-C, P-D)

**Agent P-C (Opus 4.7) — Procedure Atom Loader + 20 Initial Procedures**
- Branch: `feat/procedure-library`
- Output: `src/procedure_atom/loader.rs` + `data/procedures/*.toml`
- Initial procedures (each ~20-50 lines of TOML):
  1. fetch-url-politely
  2. parse-article-body
  3. detect-language
  4. tokenize-by-language
  5. extract-entities
  6. find-wikipedia-langlinks
  7. link-cross-language
  8. add-word-atom
  9. add-sentence-atom
  10. verify-fact
  11. detect-contradiction
  12. ask-parent-for-help
  13. identify-knowledge-gap
  14. choose-next-article
  15. update-goal-progress
  16. write-daily-report
  17. respect-budget-limits
  18. respect-rate-limits
  19. handle-http-error
  20. handle-parse-error
- Tests: 30+ ensuring each procedure atom validates and its DAG executes

**Agent P-D (Opus 4.7) — Learning Loop Executor**
- Branch: `feat/learning-loop`
- Output: `src/learner/executor.rs` + `src/bin/zetson_tick.rs`
- The Rust engine that walks a procedure atom DAG and calls primitives
- Budget enforcement, rate limiting, observability
- Tests: 20+ including end-to-end "fetch → parse → store → link" loops

### Track 3: Zetson infrastructure (Agents P-E, P-F)

**Agent P-E (Sonnet 4.6) — Seed Loader + Identity Bootstrap**
- Branch: `feat/seed-loader`
- Output: `src/seed/loader.rs`, supports YAML parsing, identity injection into empty graph
- Tests: 15+ including Zetson's exact seed file

**Agent P-F (Opus 4.7) — Observability Dashboard**
- Branch: `feat/observability`
- Output: `src/bin/zetson_dashboard.rs`, simple HTML/JSON endpoint showing daily reports + growth curve
- Tests: 10+ including report generation + metric aggregation

### Track 4: Integration (runs after 1-3 above)

**Agent P-G (Opus 4.7) — End-to-end Zetson demo**
- Branch: `feat/zetson-demo`
- Output: A demo script that:
  1. Starts fresh ZETS with Zetson seed
  2. Runs 24-hour learning loop
  3. Reports final graph size, languages learned, goals met
- Integration tests on real (but bounded) Hebrew Wikipedia data

**Total:** 7 agents, ~4,500 Rust LoC, ~100 tests, ~8-12 hours agent-wall-time.

---

## Principles That Cannot Be Violated

1. **No self-modifying code.** Zetson (and adult ZETS) never writes to its own source. Learning modifies graph only.
2. **Every atom has provenance.** Who added it, from where, with what confidence. Audit trail is non-negotiable.
3. **Budget discipline.** Zetson cannot exceed its daily HTTP/LLM/storage budgets. Hard limits enforced in Rust.
4. **Politeness.** robots.txt, rate limits, user-agent identification. Always.
5. **Determinism.** Same seed + same HTTP responses → byte-identical graph. Non-negotiable for reproducibility.
6. **No hidden capabilities.** Every procedure atom is visible in the graph. If ZETS does something, we can trace it.
7. **Graceful degradation.** If a primitive fails (network down, parse error), Zetson retries with backoff or asks parent. Never silently fails.

---

## Success Criteria

Zetson is successful when, after 90 days, a cold-started instance with the infant seed achieves:

- [ ] 20,000+ Hebrew word atoms
- [ ] 20,000+ English word atoms
- [ ] 5,000+ cross-lingual sense links
- [ ] 500+ procedure atoms (mostly learned from reading docs, not seeded)
- [ ] 100+ Python syntax atoms
- [ ] 100+ Rust syntax atoms
- [ ] Can answer a Q in Hebrew or English about any of the 100+ topics it read
- [ ] Can classify a code snippet as Python vs. Rust vs. prose
- [ ] Stays within budget (no budget violations in 90 days)
- [ ] Daily reports form a growth curve monotonically up
- [ ] When queried "what are you and what did you learn?", produces a coherent answer from its own atoms

**When Zetson hits these, it is PROOF:**
- ZETS can learn autonomously.
- The architecture generalizes (code + graph split).
- The Edge deployment story is real.
- This is a category-defining product, not another LLM wrapper.

---

## The Commit Promise

**Every step of Zetson's development is committed to git.** The daily reports become the product's clearest demo: investors can scroll through month-by-month growth. This is the story of a digital mind growing up — told in commits.

---

## Next Action (Idan, you decide)

1. **Now (this session):** Review & approve this spec. Commit to git.
2. **Session +1:** Launch Track 1 (Agents P-A + P-B) — 2 agents in parallel. 2-3 hours wall-clock.
3. **Session +2:** Launch Track 2 (Agents P-C + P-D).
4. **Session +3:** Launch Track 3 (Agents P-E + P-F).
5. **Session +4:** Launch Track 4 (Agent P-G), run first 24-hour Zetson experiment.
6. **Session +5 onward:** Weekly Zetson check-ins. Measure. Adjust seed. Let it grow.

**Estimated time from this spec to first Zetson demo:** 5-7 sessions over 1-2 weeks.

**Estimated time from first demo to 90-day milestone:** 90 days (by definition — it grows on wall-clock time).

---

*The child will teach us what we built.*
