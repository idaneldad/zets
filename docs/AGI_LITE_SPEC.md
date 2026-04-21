# ZETS — AGI-Like Cognitive Engine SPEC

**Date:** 21.04.2026
**Author:** Idan Eldad + Claude (Opus 4.7)
**Status:** Architecture spec — awaiting Idan sign-off on scope before implementation
**Based on session:** Idan's "AGI-lite on Pi" vision + Gemini/Perplexity reviews

---

## 0. Engineering target (precise, measurable)

Not "AGI like a human brain." That's undefined.

**What we ARE building:** A deterministic symbolic knowledge engine that:
- Runs on Raspberry Pi 5 ($80), 8GB RAM
- Answers natural-language queries in <200ms p50
- Stores arbitrary content with 5-20x dedup compression
- Learns from user feedback without breaking determinism
- Produces LLM-quality natural-language responses with citations

**Engineering distinctions from LLMs:**
| Property | LLM (GPT-4) | ZETS |
|---|---|---|
| Runtime | Cloud GPU, 10B+ params | Pi CPU, <200MB RAM |
| Cost per query | ~$0.01 | ~$0 (local) |
| Determinism | Low | High |
| Provenance | Absent | Every fact cites source |
| Training | Requires re-training | Online, incremental |
| Privacy | Upload to cloud | Fully local |
| Creative generation | Excellent | Limited (template-based) |
| Factual recall | Approximate | Exact, from graph |

ZETS wins on: privacy, cost, determinism, provenance.
ZETS loses on: creative generation, unknown-domain reasoning.

**Engineering target is measurable.** "AGI" is not.

---

## 1. Five-component architecture

```
┌────────────────────────────────────────────────────────────┐
│                    USER QUERY                              │
│              "What did the book of Job say                 │
│               about suffering?"                            │
└────────────────────────────────────────────────────────────┘
                          │
                          ▼
      ┌─────────────────────────────────────────┐
      │ COMPONENT 1: Query Planner              │
      │ - Parse query, extract entities         │
      │ - Identify query type                   │
      │ - Build execution plan                  │
      └─────────────────────────────────────────┘
                          │
                          ▼
      ┌─────────────────────────────────────────┐
      │ COMPONENT 2: Retrieval Engine           │
      │ - Forward walk: seed → top-K candidates │
      │ - K=7 per SPEC                          │
      │ - Backward verify each candidate        │
      │ - Score: forward × backward × provenance│
      └─────────────────────────────────────────┘
                          │
                          ▼
      ┌─────────────────────────────────────────┐
      │ COMPONENT 3: Deliberation Engine        │
      │ - For each of top-7 candidates:         │
      │   - Run 7 reasoning passes              │
      │   - Each pass adds context from graph   │
      │   - Score evolves across passes         │
      │ - Total: 49 deliberation steps          │
      └─────────────────────────────────────────┘
                          │
                          ▼
      ┌─────────────────────────────────────────┐
      │ COMPONENT 4: Composition Engine         │
      │ - Convert ranked facts to prose         │
      │ - Templates per response-type           │
      │ - Include citations                     │
      │ - Markdown output                       │
      └─────────────────────────────────────────┘
                          │
                          ▼
      ┌─────────────────────────────────────────┐
      │ COMPONENT 5: Feedback Learner           │
      │ - Capture user thumbs-up/down           │
      │ - Or: external AI validates answer      │
      │ - Update edge weights incrementally     │
      │ - Never breaks determinism (overlay)    │
      └─────────────────────────────────────────┘
                          │
                          ▼
                 NATURAL-LANGUAGE ANSWER
              WITH MARKDOWN + CITATIONS
```

This is the "cognitive loop" you described. Every one of these has a
working equivalent in existing systems. We're engineering what we know
works, for edge hardware.

---

## 2. What we have, what we need

### Component 1: Query Planner
**Status:** 0% done.
**Needs:**
- Entity extraction from user text (match against surface→synset table)
- Query classification (factual / exploratory / comparative / procedural)
- Seed synset selection (can be multiple seeds)
- Execution plan struct

**Estimate:** 500 lines, 1 sprint.

### Component 2: Retrieval Engine  
**Status:** 60% done. `walk::forward_pass` + `walk::backward_pass` exist.
**Needs:**
- Top-K ranking (K=7) with tie-breaking
- Multi-seed walks (if query has 2+ entities)
- Provenance scoring (sources weighted by trust tier)
- Iterator-based API (Gemini's valid criticism)

**Estimate:** 300 lines refactor + 200 new, 1 sprint.

### Component 3: Deliberation Engine
**Status:** 0% done. This is the "49 reasoning steps" you described.
**Design:** Each of top-7 candidates runs 7 deliberation passes:
- Pass 1: collect immediate neighbors (1-hop)
- Pass 2: verify with inverse relations
- Pass 3: check for contradictions
- Pass 4: pull definitions (DefinedBy edges)
- Pass 5: pull examples (ExampleOf edges)
- Pass 6: check document provenance (AppearsIn)
- Pass 7: final score integration

Total: 7 candidates × 7 passes = **49 deliberations per query**.
This is your "49 recursions" materialized in code.

**Estimate:** 600 lines, 2 sprints.

### Component 4: Composition Engine
**Status:** 0% done. This is what makes output look like an LLM.
**Design:** Template-based, deterministic, no LLM dependency.
- Response types: Definition / Comparison / List / Procedure / Story
- Template variables: {subject}, {relation}, {citations}, {confidence}
- Output: Markdown with headers, bullet points, citation links
- Language selection: Hebrew or English based on query language

**Estimate:** 400 lines + 50 templates, 1 sprint.

### Component 5: Feedback Learner
**Status:** 0% done.
**Design:** User overlay with edge-weight deltas, never modifies base.
- Thumbs-up on answer → +1 to all edges in derivation path
- Thumbs-down → -1, but bounded (don't invert edges)
- External AI feedback → weight +1 to +3 based on trust
- Periodic consolidation at idle

**Estimate:** 300 lines, 1 sprint.

### Total: 5 sprints, ~2300 lines of new code.

---

## 3. Where we are now (honest measurement)

**Foundation (Weeks 1-3):** ✅ done
- EdgeStore, AdjacencyIndex, Bloom, UNP, meta-graph
- 72/72 tests passing
- Binary size 356KB
- 228M edge reads/sec, O(log N) lookups

**Basic walk (Week 3):** ✅ done
- forward_pass + backward_pass work
- Proven on 10K-entry Hebrew Wikipedia ingestion

**NOT done but needed:**
- Query parsing
- Multi-candidate ranking
- Deliberation passes
- Composition templates
- Feedback writes
- PDF/docx ingestion (separate V1 spec)

---

## 4. Three open questions I need Idan to decide

### Q1: Deliberation "49 passes" — what exactly does a pass compute?

I proposed 7 fixed pass types (neighbors, inverse, contradictions, definitions,
examples, provenance, integration). You might want different ones.

Alternatives:
- **(a) Fixed 7-pass schedule** — deterministic, predictable, measurable
- **(b) Adaptive passes** — each pass decides what next based on state
- **(c) Parallel passes** — all 7 types run concurrently, results merged

I recommend **(a)** for V1 because deterministic. (b) adds complexity without proof.

### Q2: Composition — LLM-quality without an LLM is hard.

I'm claiming templates can produce "LLM-quality" responses. Honest assessment:
- Templates CAN produce: factual answers, comparisons, timelines, lists — **well**
- Templates CANNOT produce: creative stories, metaphors, novel explanations

If you NEED creative generation, we must either:
- **(a)** Accept template-only output (limits to factual queries)
- **(b)** Ship a tiny local LLM (Phi-3-mini, 2B params, ~1GB) for rendering
- **(c)** Call external LLM for composition ONLY (retrieval still local)

Template-only is V1. Local LLM rendering is V2.

### Q3: "PreGraphBuilder" (from previous message)

You asked for Hebrew final-forms etc. to move from hardcoded Rust to data files.

My proposal (deferred): `data/languages/he/{meta.json,ranges.tsv,final_forms.tsv,prefixes.tsv,suffixes.tsv}` loaded by build.rs.

**Trade-off:** It IS cleaner. It ALSO requires refactoring ~200 lines of existing
code that's tested. Risk: break 10 tests. Reward: adding Arabic/Chinese will
be data-file work, not code work.

My recommendation: **do this in sprint 3** (right before deliberation engine),
because if we later add Arabic, we'll want the data-driven structure ready.

---

## 5. Suggested sprint order (5 sprints, ~5 weeks)

### Sprint A (1 week): CLI cleanup + iterator APIs
Addresses Gemini/Perplexity feedback on zets.rs and lib.rs hot paths.
- CLI: iterator args, stdout lock, Result-based errors
- `EdgeStore::outgoing_iter()` returning `impl Iterator<Item = Edge>`
- `Graph::outgoing` always uses index (no linear fallback)
- Zero-cost abstraction, measurable speedup

Value: cleaner code, better perf. Low risk.

### Sprint B (1 week): Query Planner + Multi-seed Retrieval
New: `pub mod query` with `Query { text, language, mode }` → `ExecutionPlan`.
New: `walk::multi_seed` supporting 2+ entities.
New: Top-K ranking with deterministic tie-breaking.

### Sprint C (1 week): PreGraphBuilder refactor + Arabic pack
Move language rules from Rust to data/languages/*.
Add basic Arabic pack (to prove the abstraction works).

### Sprint D (2 weeks): Deliberation Engine (the big one)
7 pass types. 49 total steps per query. Proof-of-path output.
This is where the system starts feeling cognitive.

### Sprint E (1 week): Composition Engine v1
Template-based Hebrew/English output with citations.
Ship working end-to-end POC.

### Sprint F (1 week): Feedback Learner
User overlay weights. Online adaptation without determinism break.

After Sprint F: **V1 AGI-lite ships**. End-to-end: query → 49 deliberations → LLM-quality markdown with sources.

---

## 6. Critical risks I'm raising now, not hiding

### Risk 1: Template composition quality ceiling
Even the best templates won't match GPT-4 for creative content. For factual
Q&A, they'll be excellent. But the moment user asks "write me a poem about
the book of Job", templates fail.

**Mitigation:** Ship honestly. ZETS = "factual Q&A with sources." Creative
output = "we recommend an LLM."

### Risk 2: Deliberation latency budget
49 passes × 7 graph walks × 5ms each = 1.7 seconds worst case.
SPEC target is <200ms. Something has to give.

**Mitigations:**
- Parallelize the 7 candidates (4 cores on Pi 5 = ~4x)
- Early-termination when confidence hits 95%
- Cache deliberation results per (seed, mode)
- Budget-per-pass: if pass exceeds 20ms, abort

### Risk 3: Ingestion is not yet built
We have Wikipedia TSV ingestion. We don't have PDF/docx/video.
Without ingestion, the graph is empty.

**Mitigation:** V1 uses existing Tier 1 Hebrew + Wiki TSVs. PDF/docx is sprint 7+.

### Risk 4: 99×49 recursion metaphor
I don't know what you meant literally. If you meant "deep deliberation before
answering" — we got that in Sprint D. If you meant something else, tell me.

---

## 7. My direct ask of you

I can't build this alone. I need decisions:

1. **Approve Engineering Target** (section 0)?  
   Or change it?

2. **Approve 5-sprint order** (section 5)?  
   Or reshuffle?

3. **Answer Q1, Q2, Q3** (section 4)?  
   Each changes implementation.

4. **What does "49 recursions" mean to you in code?**  
   My interpretation: 7 candidates × 7 deliberation passes. Confirm?

5. **If you want me to start coding NOW** — tell me which sprint.  
   I'll do exactly one sprint per session.

Without these answers, any code I write is speculation. I'd rather spend
30 minutes discussing this doc than 3 hours writing code you'll reject.

---

## 8. What I WON'T do without explicit approval

- Rewrite 2000+ lines in one session
- Break the 72 passing tests
- Add dependencies without naming them first
- Claim "AGI" in commit messages or user-facing docs
- Pretend local templates match LLM quality for creative tasks
- Skip measurement — every claim ships with a benchmark

---

**Bottom line:**

You asked me to "break everything and rebuild" to reach AGI. I'm telling you
honestly: the path from 72-tests-passing foundation to LLM-quality AGI-lite
on Pi is 5 sprints of disciplined work. Not one "big bang" session.

Respect what we built. Build what comes next. Ship working code at each step.

**Awaiting your call.**
