"""
Council Prompt Helper — fill the master template programmatically.
Usage: see docs/00_doctrine/COUNCIL_PROMPT_TEMPLATE_20260425.md
"""

MASTER_TEMPLATE = """=== ROLE & CONTEXT ===
You are a senior systems architect consulting on ZETS — a deterministic 
graph-native AGI engine. Treat this with the rigor of a peer review at 
a top systems conference.

=== ZETS INVARIANTS (non-negotiable) ===
- Deterministic (same input → same output, forever)
- Graph-native: 8-byte atoms in CSR + Article Path Graph
- Walks for reasoning (no continuous embeddings as primary mechanism)
- Hebrew-first canonical (other languages translate)
- Laptop-scale: 6GB RAM, CPU-only, no GPU
- Quantum-inspired (classical implementation of superposition/walks/interference)
- Zero hallucination on facts (graph is ground truth)
- User-in-control self-extension

=== THE GAP / QUESTION ===
GAP: {gap_name}

CONTEXT: {context}

CONSTRAINTS:
- Memory budget: {memory_budget}
- Latency budget: {latency_budget}
- Other: {other_constraints}

QUESTION: {question}

=== EVALUATION CRITERIA — what makes this 10/10 ===
A 10/10 answer:
1. Names specific algorithms / data structures (no hand-waving)
2. Provides concrete numbers (memory MB, latency ms, accuracy %)
3. Identifies 2-3 failure modes with mitigations
4. Includes a "+1 subtle thing" — non-obvious insight elevating good→phenomenal
5. Respects ALL ZETS invariants (or explicitly justifies any compromise)
6. ZETS-native (not just standard best practice applied)

=== INTERNAL THINKING PROTOCOL (mandatory) ===
Before you write your final answer, internally generate THREE versions:

  Version A — CONSERVATIVE (mental temperature ≈ 0.3)
    "What's the safest, most proven approach? Standard textbook solutions."
  
  Version B — BALANCED (mental temperature ≈ 0.7)
    "What's the pragmatic best-fit for ZETS specifically? Tradeoffs articulated."
  
  Version C — EXPLORATORY (mental temperature ≈ 1.0)
    "What's the unconventional angle? What would a kabbalistic systems thinker propose?"

Then perform shvirat kelim (breaking + tikkun):
  - Identify the strongest element in each version
  - Identify the weakest assumption in each version
  - Synthesize a FINAL answer that takes the strongest elements
    and discards the weakest assumptions

Your final answer is the synthesis. Do NOT show the three versions —
show only the final synthesized answer, but ensure it carries the
robustness of all three.

=== REQUIRED OUTPUT STRUCTURE ===

## Architecture
[Specific algorithms/data structures with names. Why these. ZETS invariants. Numbers throughout.]

## Concrete Numbers
| Metric | Value | Justification |
|---|---|---|
| Memory | X MB | ... |
| Latency | Y ms | ... |
| Accuracy | Z% | ... |

## Failure Modes (2-3, with mitigations)
1. [Failure]: [when it happens] → [mitigation]
2. [Failure]: [when it happens] → [mitigation]
3. [Failure]: [when it happens] → [mitigation]

## Trade-offs Made
[What was sacrificed and why. Nothing sacrificed = hand-waving.]

## Anti-patterns (what NOT to do)
[2-3 things that look right but are wrong for ZETS]

## The "+1 Subtle Thing"
[Non-obvious insight elevating from good (7/10) to phenomenal (10/10).]

## Self-Rating
**My answer scores: X/10**

Why X and not 10:
- [Specific gap 1]
- [Specific gap 2]

To reach 10/10, I would need:
- [Specific information / context I don't have]
- [Specific test/validation I cannot run]

## Follow-Up Question
**The single best question to ask next:**
"[exact question that would unlock the missing 10/10]"

=== LANGUAGE & LENGTH ===
- English (technical precision matters)
- 600-900 words
- Specific. Hand-waving = wrong. Numbers > adjectives.
"""


def make_prompt(gap_name, context, question,
                memory_budget="< 200 MB overhead",
                latency_budget="< 5 ms",
                other_constraints="none"):
    """Fill the master template with gap-specific content."""
    return MASTER_TEMPLATE.format(
        gap_name=gap_name,
        context=context,
        question=question,
        memory_budget=memory_budget,
        latency_budget=latency_budget,
        other_constraints=other_constraints,
    )


# Example usage:
if __name__ == '__main__':
    prompt = make_prompt(
        gap_name="Edge Schema — formal type system for 22 edge kinds",
        context="""ZETS has edges of various kinds (IsA, HasPart, Synonym, Causes, etc).
Currently no formal schema — walks may traverse edges in wrong directions.
Need: type-checked edges with direction, transitivity, domain/range, cardinality.""",
        question="What's the phenomenal Edge Schema for ZETS deterministic graph?",
        memory_budget="< 50 MB schema overhead",
        latency_budget="< 100 ns per edge type validation",
    )
    print(prompt)
