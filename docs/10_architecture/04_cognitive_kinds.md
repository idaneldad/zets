# ZETS Cognitive Tree — Beam Deliberation with Bridges

**Date:** 21.04.2026
**Status:** Design SPEC — awaiting Idan decisions on 3 critical questions
**Replaces/upgrades:** Sprint D ("Deliberation Engine") from AGI_LITE_SPEC.md
**Why:** Original Sprint D described linear 49-pass deliberation. Idan's
actual vision is richer — parallel beam tree with bridges, probabilities,
multiple answer variants. This spec captures the richer vision.

---

## 0. Idan's vision (quoted, unfiltered)

> "המוח מריץ כבר הרבה חשיבת הסקה אחת אחרי השניה בדומה לרקורסיה של WHAT IF
> אני אעשה ככה או אחרת כמו הסתפגות מהירה בעץ החלטות שזורם לנוירונים שונים
> ואולי גם קופץ קדימה לאסוציאציות במקומות אחרים בעץ"

> "7 רעיונות אפשריים אסוציאטיביים... בכל אחד מהם נעשה איתור של 7 אסוציאציות
> המשך... נוכל לחבר לו גשר... לתת ליוזר תשובות מגוונות ברמות טמפרטורה שונות"

> "הסיכוי לעשות ככה וככה בהצלחה זה 30% אבל אם תשנה ככה וככה זה יגדיל ל-70%"

> "הבנת הקשר ביחס לסיטואציה... כתר בחנות תחפושות ≠ כתר ברפואת שיניים"

Three things are happening here, not one. Separating them is critical.

---

## 1. Three distinct cognitive mechanisms

### Mechanism 1 — Beam Tree (What-If Recursion)
The parallel exploration of 7 alternatives × 7 sub-alternatives.
This is **beam search with depth-2 expansion**, not linear pass.

```
              Question Q
              /     |     \
           A         B         C   ...G    (7 branches)
          /|\       /|\       /|\
        A1..A7   B1..B7   C1..C7    ...   (49 leaves)
```

Each branch has a **probability**. Each leaf has a **refined probability**.
Top-K leaves become the final answer variants.

### Mechanism 2 — Bridges
During exploration, if branch A6 reaches synset X, and branch C2 ALSO
reaches synset X — we've found a structural commonality. Create a
temporary bridge edge: `A6 --Bridge-to--> C2`. Influences scoring.

### Mechanism 3 — Context Disambiguation
"Crown" → which synset? Determined by:
- Active session context (previous turns)
- Speaker profile (dentist vs shopkeeper)
- Conversation topic graph overlap

---

## 2. The critical decisions I'm flagging

Before implementation, Idan must choose:

### Decision A: Is "49" a tree or a passes count?

- **(A1) Full 7×7 tree:** 49 distinct leaf states. Accurate but slow (~5s on Pi).
- **(A2) Linear 49 passes on 7 candidates:** 7 answers, each refined 7 times.
  Fast (~500ms) but no branching.
- **(A3) Adaptive tree:** starts at 7, prunes weak branches, adds bridges.
  Target: <1s on Pi, leaves final count dynamic.

**My recommendation: A3.**

### Decision B: Where do probability numbers come from?

Three honest sources:
- **(B1) Edge weights** — collected from user feedback over time.
- **(B2) Path confidence** — function of hops × edge quality × source tier.
- **(B3) Source reliability** — Tier 1 (Tanakh/primary) = 95%, Tier 3 (wiki) = 70%.

**My recommendation: all three.** Combined score = `min(B1, B2, B3) × walk_depth_decay`.
Never invent a probability. If no source supports a number, say "uncertain."

### Decision C: Context Disambiguation approach

- **(C1) Session-bag overlap:** Record synsets mentioned in session. "Crown"
  has multiple synsets; pick the one with most overlap to session bag.
- **(C2) Speaker-profile priors:** User has a profession synset. "Crown"
  meaning weighted by profession-adjacency.
- **(C3) Conversation-topic graph:** Recent N turns = topic subgraph.
  Weight disambiguation by topic overlap.

**My recommendation: (C1) + (C3).** C2 requires user profiles (privacy + scope).

---

## 3. The algorithm (assuming A3 + all-B + C1+C3)

```
fn think(query: &Query, ctx: &SessionContext) -> Vec<AnswerVariant> {
    // STAGE 1: Disambiguate query terms using context (Mechanism 3)
    let resolved_seeds = disambiguate(&query.terms, ctx);

    // STAGE 2: Beam-expand level 1 (7 branches)
    let level1 = expand_candidates(&resolved_seeds, k=7);
    //   Each candidate is (synset, relation, partial_path, p)

    // STAGE 3: Beam-expand level 2 (7 per branch)
    let mut level2 = vec![];
    for candidate in &level1 {
        let children = expand_candidates(candidate.synset, k=7);
        level2.extend(children.map(|c| (candidate, c)));
    }
    //   level2 has up to 49 entries

    // STAGE 4: Detect bridges (Mechanism 2)
    let bridges = find_shared_synsets(&level2);
    //   Bridges increase confidence of both involved branches

    // STAGE 5: Score + rank using B1+B2+B3
    let scored: Vec<_> = level2.iter().map(|(parent, leaf)| {
        let weight_score = edge_weight(leaf);      // B1
        let path_score = path_confidence(parent, leaf);  // B2
        let source_score = source_tier(leaf);      // B3
        let bridge_bonus = bridge_contribution(leaf, &bridges);
        let total = weight_score
            .min(path_score)
            .min(source_score)
            * walk_decay(depth=2)
            + bridge_bonus;
        (parent, leaf, total)
    }).collect();

    // STAGE 6: Select top-K (typically 3-5)
    let top_k = select_top(&scored, K=3);

    // STAGE 7: For each variant, compute
    //   "If condition X changes, probability shifts from X% to Y%"
    let answer_variants = top_k.iter().map(|v| {
        let conditional = conditional_analysis(v, &scored);
        AnswerVariant {
            main_path: v.clone(),
            base_probability: v.total,
            conditional_alternatives: conditional,
        }
    }).collect();

    answer_variants
}
```

---

## 4. The data model additions required

### New synset types
- `Session` (already proposed in Nervous System spec)
- `Turn` — single query-response pair within session
- `ConditionalFact` — "if X changes, then Y instead of Z"

### New relations
- `AlternativeMeaning` — links ambiguous surface to multiple synsets
- `DependsOn` — conditional dependency (X holds because Y)
- `BridgedWith` — dynamic bridge found during walk
- `TopicOfSession` — what this session is about
- `ProfessionOfSpeaker` — user role for context

### New edge properties
Current edge: `{source, target, relation, weight}`.
Add: `confidence_source_tier` (1/2/3), `created_in_session` (for overlay).

---

## 5. Output format example

Query: *"Should I invest in gold now?"*

Beam tree output (simplified):
```json
{
  "primary_answer": {
    "text": "Based on historical returns (2020-2026), gold averaged 8% annually.
             Current macro conditions suggest moderate confidence.",
    "probability_success": 0.62,
    "sources": ["WSJ_2025", "HistoricalReturns_DB", "FedPolicy_2025"]
  },
  "alternatives": [
    {
      "condition": "If Fed raises rates in Q2 2026",
      "probability_shift": "from 62% to 35%",
      "reasoning_path": "Rate hike → USD stronger → gold lower"
    },
    {
      "condition": "If geopolitical tension increases",
      "probability_shift": "from 62% to 78%",
      "reasoning_path": "Risk-off sentiment → gold demand up"
    }
  ],
  "bridges_found": [
    "Oil prices connect to both gold and Fed policy"
  ],
  "confidence_overall": 0.55,
  "deliberation_stats": {
    "branches_explored": 7,
    "leaves_evaluated": 34,  // adaptive pruning
    "bridges_detected": 2,
    "time_ms": 847
  }
}
```

This gives user: probability, conditions, sources, time spent thinking.
**This is the "LLM-quality output" Idan requested, but with deterministic
provenance that LLMs cannot provide.**

---

## 6. Why this is hard, and why it's worth it

**Hard parts:**
1. **Beam tree state management** — keeping 49 parallel walks in bounded RAM.
   Solution: arena allocation, predecessor pointers only.
2. **Bridge detection** — naive O(49²) comparison. Solution: hash table by synset.
3. **Probability composition** — avoiding spurious precision. Solution: round to
   5% buckets, never show 62.347%.
4. **Context disambiguation** — requires session state to work well.
   Solution: ship Sprint H (sessions) before this.
5. **Deterministic "What-if"** — If/then reasoning requires graph to carry
   conditional facts. Solution: extend meta-graph with `DependsOn` relation.

**Why worth it:**
The moment a user sees *"62% base, shifts to 78% if X"* with sources —
it's categorically different from any chatbot response. That's the product
differentiation. That's the moat.

---

## 7. Implementation sequencing

This SPEC **extends** Sprint D from AGI_LITE_SPEC, doesn't replace other sprints.

Sequencing that works:

| Sprint | What | Why this order |
|--------|------|----------------|
| A | CLI + iterator | Foundation hygiene |
| B | Query Planner + seeds | Need disambiguation foundation |
| C | PreGraphBuilder | Multi-language support |
| H | Sessions first | Context-disambig needs session state |
| **D** | **Beam tree (this spec)** | **The Big One** |
| E | Composition | Format the beam output |
| F | Feedback Learner | Improves edge weights used by B1 |
| G | Tool Registry | Nervous system |
| I | Cloud Relay | External channels |

**Key change from previous plan:** Session management (H) moves UP before
D because context-disambig needs it.

---

## 8. Estimated work

Beam tree implementation: **~800 lines Rust**, 2-3 weeks focused work.
Sub-components:
- Beam state arena: 150 lines
- 49-leaf expansion: 200 lines
- Bridge detection: 100 lines
- Probability scoring: 200 lines
- Conditional analysis: 150 lines
- Tests: 200 lines

This is the biggest sprint in the plan. Justified by being the heart of
the cognitive engine.

---

## 9. What this SPEC does NOT yet contain

Honest gaps for future resolution:

1. **Where do "conditional facts" come from?**
   "If Fed raises rates, gold drops" — does this come from ingested text?
   From user teaching? From market data feeds? Open question.

2. **How does the system distinguish causal from correlative?**
   Edges alone don't tell us. May need a new meta-relation `CausesChangeIn`.

3. **Temperature concept — concrete meaning?**
   Idan mentioned "temperature variants." In LLMs, temperature = randomness.
   In deterministic ZETS, "temperature" probably means "tolerance for weaker
   edges." t=0 → only ≥0.8 edges; t=0.7 → also ≥0.3 edges.
   Needs formal definition.

4. **How to avoid the "hallucination through composition" trap?**
   If the system composes 5 weak-but-related edges into an answer,
   it might sound confident. Solution: per-edge confidence propagates;
   final confidence = minimum along path, not product.

---

## 10. Decision checkpoint for Idan

Before I write Sprint D code:

1. ☐ **Approve algorithm structure** (section 3)?
2. ☐ **Approve decision A3** (adaptive tree) — or prefer A1/A2?
3. ☐ **Approve decisions B1+B2+B3 combined** — or one only?
4. ☐ **Approve C1+C3** for disambiguation — or add C2?
5. ☐ **Approve sequencing** with H before D?
6. ☐ **Answer section 9 gaps** — especially gaps 1 and 3.

Without answers: I cannot estimate accurately, and the sprint will take
longer or produce something you didn't want.

---

## 11. Honest risk statement

This is the hardest engineering work in all of ZETS. It is the difference
between "knowledge retrieval" and "cognitive reasoning."

I can build knowledge retrieval in 2 weeks. I need 4-6 weeks for this
cognitive tree, done right. If we rush it, we end up with a system that
"looks smart" but breaks on edge cases.

**My ask:** Approve the SPEC, let me ship Sprints A+B+C+H first (5 weeks),
then dedicate 4-6 weeks to Sprint D (this one). After that, E+F+G+I are
small finish-work.

Total path to AGI-like-lite: ~13-15 weeks of disciplined work.

This is an honest number. Don't let anyone (me included) tell you it's faster.

---

**End of Cognitive Tree spec. Next: Idan signs off or requests changes.**
