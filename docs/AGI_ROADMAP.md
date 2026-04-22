# ZETS AGI-Readiness Roadmap (22.04.2026)

## Honest reality check
- Code: 7,715 lines (clean, 103/103 tests pass)
- Concepts: 144,670 (dictionary-like)
- **Semantic edges per concept: 0** ← critical gap
- Reasoning routes: 0
- System graph: infrastructure only (3 bootstrap routes)

## Strategic premise
Not competing with LLMs on breadth. **Winning on:**
1. Zero hallucination (deterministic by design)
2. Explainability (Log scope = full audit trail)
3. Privacy (Paranoid encryption per user)
4. Self-improvement (Testing sandbox safe modification)

## 12-step plan to AGI-readiness

### Phase A — Reasoning primitives (week 1-2)
**1. Extend opcode set** [IN PROGRESS THIS SESSION]
  - ListIndex, ListLen (for navigating edge-neighbor lists)
  - Dup, Swap, Pop (stack management for complex routes)
  - Add, Sub, Lt, Gt (numeric ops for confidence/trust math)
  - Not (boolean inversion)

**2. Multi-hop reasoning routes** [IN PROGRESS THIS SESSION]
  - `is_a_ancestor(concept, depth_max)` — walk IS_A chain
  - `find_common_ancestor(a, b)` — reasoning 101
  - `path_between(a, b, max_depth)` — 3+ hops

**3. Causal chain routes** [PHASE B start]
  - `because(effect)` → walks CAUSES backward
  - `consequences_of(cause, depth)` → forward walk

### Phase B — Edge population (week 3-6) ← THE CRITICAL GAP
**4. Wiktionary edge extraction**
  - Parse definitions → extract "is a", "part of", "has", "causes"
  - Write to Testing sandbox first, validate, promote to Data
  - Target: 100K IS_A edges, 50K PART_OF edges from English

**5. Wikipedia edge extraction**
  - First sentence of every article = Hearst paradise
  - Target: 1M semantic edges across 16 languages

**6. Bootstrap ConceptNet import**
  - ConceptNet has 8M edges already cleaned
  - Map to ZETS concept IDs via surface matching
  - Instant +8M edges

**After Phase B:** From 0 to 10M+ semantic edges. ZETS becomes a real knowledge graph.

### Phase C — Learning primitives (week 7-10)
**7. Trust learning loop**
  - Every query → LogEntry → trust update for contributing sources
  - Automated recalibration from Log scope

**8. Active learning (curiosity)**
  - Gap detection: scan Log for failed queries, identify topic clusters
  - Self-directed reading: fetch Wikipedia articles in gap areas
  - Targeted edge extraction from new articles

**9. Contradiction detection**
  - When two sources conflict, open Testing sandbox
  - Meta-route: `resolve_contradiction(edge_a, edge_b)` → mark winner

### Phase D — Interaction (week 11-14)
**10. Minimal NLU parser**
  - Intent classification (question/command/statement)
  - Entity extraction (nouns in query)
  - NO deep parsing — just enough to route

**11. Template NLG composer**
  - Graph facts → natural sentence with morphology agreement
  - Hebrew RTL, English LTR
  - Start with 50 templates, grow to 500

**12. Session continuity + Theory of Mind**
  - Track what user knows already (from Log)
  - Avoid re-explaining
  - Personal context: "you mentioned X earlier"

## What MUST be true at end of Phase C

**Benchmarks ZETS should win at (by design):**
- TruthfulQA: 95%+ (LLMs ~60-70%)
- GSM8K (math, if we add symbolic math routes): 98%+
- Hallucination rate: 0%
- Explainability: full trail per answer

**Benchmarks ZETS won't win:**
- Creative writing (LLMs strong here)
- Ambiguous commonsense (LLMs better with distributions)
- HumanEval code generation (not our goal)

## What we DON'T build
- No LLM integration in core (optional sidecar only)
- No cloud dependency
- No statistical "probably means X" logic
- No "emergent behavior" — everything traceable

## Session priority (22.04.2026)

**Today:** Phase A steps 1+2 — opcode extension + multi-hop IS_A route

**Next session:** Phase B step 4 — Wiktionary edge extraction prototype

**By end of May 2026:** Phase B complete, ZETS has 10M+ edges

**By end of July 2026:** Phase C complete, ZETS learns continuously

**By end of September 2026:** Phase D complete, ZETS usable as personal assistant

## Anti-patterns to avoid
1. Don't add opcodes without a route that needs them
2. Don't bulk-import edges without provenance (source + trust + timestamp)
3. Don't merge from Testing to Data without regression test
4. Don't expand to a new language before Hebrew+English are solid
5. Don't chase LLM features we can't do better — stay symbolic

## Success metric
Not "does it feel like AGI" but:
**Can ZETS answer "why?" for every answer it gives, correctly, within 10ms, with <1% hallucination, on a laptop?**

If yes → AGI-ready for factual domains. Ship.
