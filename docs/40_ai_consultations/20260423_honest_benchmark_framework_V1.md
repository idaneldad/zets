# ZETS Benchmark Framework — Honest Ranking vs LLMs and Humans

**Date:** 23.04.2026
**Origin:** Idan's request: "After we implement, I want honest ranking against
industry LLM leaders and other AI engines, and against humans — so we know
what remains to research and improve."

---

## Core principle: Benchmarks must measure what we CLAIM to be better at

Most current benchmarks measure **what LLMs are good at** (text fluency,
pattern matching on MMLU). ZETS claims to be different — a graph-native
knowledge engine with explicit reasoning. We must test that claim.

**Rule:** No self-congratulation. If we lose, we report the loss. If we win,
we document by how much and why.

---

## Three dimensions of comparison

### Dimension 1: Against industry LLMs
- gpt-4o (OpenAI, reasoning + retrieval)
- claude-4.7 (Anthropic, what I am)
- gemini-2.5 (Google, long-context, tool use)
- llama-3.3-70b / deepseek-v3 (open-weight state-of-the-art)

### Dimension 2: Against specialized engines
- Wolfram Alpha (symbolic reasoning, math)
- Google Knowledge Graph (entity lookup)
- Neo4j / TigerGraph (graph query performance)
- Elasticsearch (text search at scale)

### Dimension 3: Against humans
- Domain expert (biologist for biology, historian for history)
- Generalist knowledge worker
- Student at various levels (for pedagogical tasks)

---

## Benchmark tasks — 8 categories, honestly scored

### Cat 1 — Factual recall (LLM strong)
- TriviaQA (5K questions)
- Natural Questions (3K questions)
- **Metric:** accuracy%, latency ms
- **Expected:** LLMs win; ZETS competitive at ~85% of LLM performance

### Cat 2 — Multi-hop reasoning (graph should win)
- HotpotQA (multi-hop, with supporting passages)
- MuSiQue (2-4 hop questions requiring composition)
- **Metric:** accuracy AND showing-the-path (EM + F1 on supporting facts)
- **Expected:** ZETS should equal or beat LLMs because we have the path

### Cat 3 — Arithmetic and numerical
- GSM8K (grade-school math word problems)
- MATH (competition math)
- **Metric:** accuracy + show-work
- **Expected:** LLMs currently win on GSM8K. ZETS needs to delegate to
  Python/symbolic tool and compose results

### Cat 4 — Common sense (current blind spot)
- CommonsenseQA
- Social IQa
- HellaSwag
- **Metric:** accuracy
- **Expected:** LLMs win. This is ZETS's weakest area. Need to measure gap.

### Cat 5 — Graph-native queries (ZETS strong)
- "What's the shortest path from Einstein to Pythagoras via 'influenced-by'?"
- "List all concepts within 2 hops of 'photosynthesis' that are also
  referenced in cellular biology articles"
- **Metric:** path length correctness, latency, coverage
- **Expected:** ZETS DOMINATES — LLMs have no real graph

### Cat 6 — Multi-language consistency
- Parallel TriviaQA in 10 languages (Hebrew, Arabic, Japanese, Russian, etc.)
- Is the answer the same in all languages?
- **Metric:** cross-lingual agreement rate
- **Expected:** ZETS's SAME_AS edges should give 90%+ consistency

### Cat 7 — Accessibility & Personalization (unique to us)
- Same question, 4 personas, check response appropriateness
- Blind user: is response audio-friendly?
- Deaf user: does response use visual-spatial language?
- Aphasia: concrete images/diagrams not abstract prose?
- Medical patient: cautious, multi-source, with confidence markers?
- **Metric:** human rater scores 1-5 for appropriateness per persona
- **Expected:** ZETS unique — LLMs can imitate but not structurally tailor

### Cat 8 — Explainability (graph-native advantage)
- For each answer: can the system show the exact chain of atoms used?
- **Metric:** % of answers with complete traceable path
- **Expected:** ZETS = 100%, LLMs < 30% (they have no explicit path)

---

## How we run this honestly

### Phase 1 — Setup (week 1)
- Standardize 100-question subset per category
- Gold-standard answers vetted by human experts
- No cherry-picking — fixed set, drawn from published benchmarks

### Phase 2 — Measurement (week 2)
- Run all 800 questions through each system
- Record: answer, path (if any), latency, confidence
- Independent human rater for appropriateness (cat 7)

### Phase 3 — Analysis (week 3)
- Per-category rank table
- Error analysis: why did we lose where we lost?
- Gap identification: top 3 weaknesses with proposed fixes

### Phase 4 — Publish (week 4)
- Full report, including losses
- Open-source the benchmark harness
- Invite community to replicate

---

## What we must NOT do

- ❌ **Cherry-pick questions that favor us.** Use published benchmarks unchanged.
- ❌ **Compare against deliberately-weakened baselines.** Use the real gpt-4o,
  the real Claude, at published settings.
- ❌ **Mix "graph native" with "LLM inside" without clearly labeling.** If we
  use an LLM to reformulate a query, that's valid but must be disclosed.
- ❌ **Claim "better" without confidence intervals.** Report standard error,
  not point estimates.
- ❌ **Hide failures.** A lost category is useful information.

---

## Success criteria (what would count as winning)

Not "beat gpt-4o on everything." That's unrealistic.

**True success:**
1. **Cat 2 (multi-hop):** Match or exceed gpt-4o accuracy AND provide path
2. **Cat 5 (graph-native):** ≥10× faster than any system + correct
3. **Cat 6 (multi-lingual consistency):** ≥90%, beat all LLMs
4. **Cat 8 (explainability):** 100% traceable, beat all LLMs
5. **Cat 7 (personalization):** Uniquely structural, LLMs can only simulate

**What we're willing to lose:**
- Cat 1 (factual) — LLMs have more training data; we aim for competitive
- Cat 3 (math) — Delegate, don't claim primary competency
- Cat 4 (common sense) — Known weakness, but measurable gap tells us how much

---

## Implementation plan

### Rust module: `src/bench/`
```
src/bench/
├── mod.rs           — registry of benchmark suites
├── runner.rs        — execute question through system, collect metrics
├── categories/      — one file per category (facts, multihop, graph, ...)
├── harness.rs       — parallel execution + checkpointing
└── report.rs        — aggregate + format results
```

### Scoring components
- **Accuracy:** exact match, F1, or semantic similarity depending on task
- **Latency:** median, p50, p90, p99
- **Path-quality:** % answers with complete path (only for graph-native)
- **Consistency:** cross-lingual or cross-persona agreement
- **Explainability:** human-rater score 1-5

### Comparison harness
- Each system via its API (OpenAI, Anthropic, Google)
- ZETS native via MCP
- Same questions, same prompts, recorded per-run
- Reproducibility: all prompts version-tagged

---

## Next steps

1. Build `src/bench/runner.rs` with one category (multi-hop) as proof
2. Collect 100 HotpotQA questions, vet human answers
3. Run through ZETS + Claude + Gemini + gpt-4o
4. Produce first honest report
5. Iterate: where ZETS loses, dig into root cause and fix
