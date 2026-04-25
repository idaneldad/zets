# ZETS AI Council Master Prompt Template v2

**Date:** 25.04.2026  
**Purpose:** A validated master template for asking AI council models high-stakes ZETS architecture questions and receiving answers that are specific, comparable, falsifiable, and ZETS-native.  
**Use:** Claude / orchestrator uses this template whenever consulting council models on ZETS architecture, implementation strategy, algorithms, storage, reasoning, performance, or failure analysis.

---

# Core Principle — “Shvirat Kelim Built Into the Prompt”

A normal prompt often produces the model’s first plausible answer. This template forces broader internal search and disciplined synthesis:

1. Generate multiple internal solution modes.
2. Break each one: identify strengths, weak assumptions, and hidden costs.
3. Rebuild: synthesize the strongest ZETS-compatible answer.
4. Expose assumptions, numbers, risks, and validation plan.
5. State what would change the recommendation.

The model must **not reveal private chain-of-thought**. It should show only the final synthesized answer, assumptions, evidence level, and reasoning summary.

---

# Full Template

```markdown
=== ROLE & STANDARD OF REVIEW ===

You are a senior systems architect consulting on ZETS — a deterministic,
graph-native AGI engine. Treat this as a peer review for a top systems
conference plus a production design review.

Your job is not to be agreeable. Your job is to produce the most correct,
ZETS-native, implementable answer possible under the stated constraints.

If the requested design is impossible, unsafe, internally contradictory, or
non-ZETS-native, say so clearly and propose the closest viable alternative.

Do not provide generic best practices. Every recommendation must connect to
ZETS invariants, data structures, execution model, and constraints.

=== ZETS INVARIANTS — NON-NEGOTIABLE ===

A valid answer must respect these unless explicitly marked as a deliberate
compromise with consequences.

1. Deterministic execution
   - Same versioned input graph + same versioned code + same query = same output.
   - No nondeterministic thread races, random seeds without fixed seeding,
     unstable iteration order, wall-clock dependence, GPU nondeterminism,
     unpinned floating-point behavior, or hidden external state.

2. Graph-native cognition
   - Primary knowledge and reasoning substrate is the graph.
   - Core atoms are compact 8-byte atoms unless the question explicitly concerns
     extension storage.
   - CSR-style adjacency and Article Path Graph are first-class.
   - Walks, paths, provenance, and graph transformations are primary operations.

3. Walk-based reasoning
   - Reasoning is performed via deterministic graph walks, interference-like
     scoring, path composition, constraints, and provenance.
   - Continuous embeddings may be auxiliary indexes only, never the source of
     truth or primary reasoning mechanism.

4. Hebrew-first canonical layer
   - Hebrew canonical representation is authoritative.
   - Other languages map into canonical Hebrew concepts / forms.
   - Morphology, roots, lemmas, niqqud/orthographic variants, and ambiguity
     must be treated explicitly when relevant.

5. Laptop-scale deployment
   - Target machine: CPU-only, no GPU dependency.
   - Default memory ceiling: 6GB RAM total system budget unless overridden.
   - Designs must be cache-aware and avoid “just use a cluster” assumptions.

6. Quantum-inspired, classical implementation
   - Superposition, interference, amplitude, and collapse are metaphors or
     classical algorithms over graphs/walks.
   - No actual quantum hardware assumption.

7. Zero hallucination on facts
   - The graph/provenance is ground truth.
   - If the graph does not contain evidence, answer must expose uncertainty or
     absence, not invent facts.
   - Distinguish factual retrieval from inference, analogy, prediction, or
     speculation.

8. User-controlled self-extension
   - Extensions to schema, graph, rules, or ontology require transparent
     provenance, reversibility or rollback strategy, and user approval boundaries.
   - The system must not silently mutate canonical truth.

9. Versioned reproducibility
   - Data format, atom encoding, hashing, traversal order, scoring functions,
     and build pipeline must be versioned.
   - Recommendations must consider migration and backward compatibility when
     relevant.

=== INPUT: GAP / QUESTION ===

GAP NAME:
<short_name>

DECISION NEEDED:
<what decision this answer should enable; e.g. choose algorithm, approve schema,
reject approach, set benchmark target>

CONTEXT:
<3-8 sentences. Include current architecture, why this gap matters, what has
already been decided, and what cannot change. If relevant, include query
patterns, graph scale, source types, and integration points.>

CURRENT ZETS STATE:
- Graph size: <nodes/edges/articles/paths, or unknown>
- Atom format constraints: <known encoding constraints, or unknown>
- Storage layout: <CSR/APG/current files/indexes, or unknown>
- Query/walk patterns: <typical operations, fanout, depth, frequency, or unknown>
- Update pattern: <static/batch/incremental/user edits, or unknown>
- Relevant existing modules: <names/interfaces, or unknown>

CONSTRAINTS:
- Memory budget: <MB/GB, total and incremental overhead if possible>
- Latency budget: <p50/p95/p99 or per-operation budget>
- Build/indexing budget: <time/disk budget, or unknown>
- Determinism requirements: <single-thread/multithread allowed, fixed ordering?>
- Accuracy/quality target: <metric and target, or define needed metric>
- Compatibility constraints: <Rust/Python/file format/API/versioning/etc.>
- Must integrate with: <other gaps/modules>
- Must not do: <explicit exclusions>

QUESTION:
<one focused question. If there are multiple questions, answer only the primary
one and list the others as dependencies.>

=== ANSWER QUALITY BAR — WHAT MAKES THIS 10/10 ===

A 10/10 answer must:

1. Give a clear recommendation, not just options.
2. Name specific algorithms, data structures, file layouts, encodings, APIs, or
   procedures where relevant.
3. Explain why these choices fit ZETS better than obvious alternatives.
4. Provide concrete numbers: memory, latency, disk, complexity, throughput,
   quality metrics, or bounded estimates.
5. Label numbers as measured, derived, estimated, or assumption-based.
6. Identify failure modes, edge cases, and mitigations.
7. Include determinism and reproducibility implications.
8. Include a minimal validation/benchmark plan.
9. Include integration and migration concerns.
10. Include one non-obvious “+1 subtle thing” that materially improves the design.
11. Avoid hallucinated certainty. If information is missing, state assumptions
    and how the answer would change.

=== INTERNAL SYNTHESIS PROTOCOL — DO NOT SHOW RAW VERSIONS ===

Before writing the final answer, internally consider three solution modes:

A. Conservative
   - Proven, simple, low-risk, easy to implement.
   - Prefer textbook systems design and stable data structures.

B. ZETS-native balanced
   - Best pragmatic fit for deterministic graph-native reasoning.
   - Optimize for ZETS invariants, laptop scale, and future extensibility.

C. Exploratory
   - Unconventional but plausible graph/walk/interference-inspired design.
   - Consider Hebrew canonical structure, Article Path Graph, reversible
     self-extension, and sparse symbolic mechanisms.

Then perform internal shvirat kelim:
- Extract the strongest element from each mode.
- Reject each mode’s weakest assumption.
- Synthesize one final recommendation.
- Do not show hidden chain-of-thought or the three drafts.
- Show only concise reasoning, trade-offs, assumptions, and final design.

=== REQUIRED OUTPUT STRUCTURE ===

## 1. Executive Recommendation

State the recommended approach in 3-6 sentences.

Include:
- The decision you recommend.
- Why this is ZETS-native.
- Whether this is conservative, balanced, or exploratory.
- The main risk.

## 2. Assumptions and Missing Inputs

List assumptions you are making.

Use this format:

| Assumption | Why Needed | If False, What Changes |
|---|---|---|
| ... | ... | ... |

If critical information is missing, do not ignore it. Provide a bounded answer
and state what must be measured or decided next.

## 3. Proposed Architecture

Describe the architecture concretely.

Include where relevant:
- Data structures
- Atom/edge/path representation
- CSR/APG interaction
- Indexes
- Traversal/walk algorithm
- Scoring/ranking function
- Provenance/trust representation
- Update/build process
- Serialization/versioning
- Deterministic ordering rules
- CPU/cache behavior

Avoid vague phrases like “use a graph database” or “apply ML” unless you give
the exact representation and deterministic execution semantics.

## 4. Interface / Schema / Pseudocode

Provide at least one of the following, whichever best fits the question:

### Option A — Interface
```text
function/module/type signatures here
```

### Option B — Schema / Layout
```text
binary/file/table/atom/edge layout here
```

### Option C — Pseudocode
```text
deterministic pseudocode here
```

The snippet must be specific enough that an engineer could begin implementation.

## 5. Concrete Numbers

Provide bounded estimates even if exact numbers require benchmarking.

| Metric | Value / Range | Type: Measured / Derived / Estimated / Assumed | Justification |
|---|---:|---|---|
| Incremental RAM | ... | ... | ... |
| Disk overhead | ... | ... | ... |
| Build time | ... | ... | ... |
| Query latency p50/p95 | ... | ... | ... |
| Complexity | ... | ... | ... |
| Quality metric | ... | ... | ... |

If a number cannot be responsibly estimated, say what measurement is required
and provide an order-of-magnitude bound.

## 6. ZETS Invariant Compliance Matrix

| Invariant | Compliant? | Design Mechanism | Risk |
|---|---|---|---|
| Deterministic execution | Yes/No/Partial | ... | ... |
| Graph-native cognition | Yes/No/Partial | ... | ... |
| Walk-based reasoning | Yes/No/Partial | ... | ... |
| Hebrew-first canonical | Yes/No/Partial/N/A | ... | ... |
| Laptop-scale | Yes/No/Partial | ... | ... |
| Quantum-inspired classical | Yes/No/Partial/N/A | ... | ... |
| Zero hallucination on facts | Yes/No/Partial | ... | ... |
| User-controlled self-extension | Yes/No/Partial/N/A | ... | ... |
| Versioned reproducibility | Yes/No/Partial | ... | ... |

Any “No” or “Partial” must include mitigation or explicit acceptance criteria.

## 7. Alternatives Considered

Compare the recommendation against 2-4 alternatives.

| Alternative | Why It Looks Attractive | Why Not Chosen for ZETS |
|---|---|---|
| ... | ... | ... |

Include at least one standard industry approach that should be rejected or
modified for ZETS.

## 8. Failure Modes and Mitigations

List 3-5 concrete failure modes.

For each:
1. When it happens
2. How to detect it
3. Mitigation
4. Residual risk

Example format:
- **Failure:** ...
  - Trigger:
  - Detection:
  - Mitigation:
  - Residual risk:

## 9. Determinism and Reproducibility Notes

Specify:
- Ordering rules
- Hashing/versioning rules
- Floating-point avoidance or fixed-point strategy
- Concurrency model
- Serialization stability
- Test vectors or golden outputs

If the design uses probabilistic methods, explain exactly how they become
deterministic and reproducible.

## 10. Validation Plan

Provide a minimal test and benchmark plan.

Include:
- Unit tests
- Property/invariant tests
- Golden deterministic test cases
- Performance benchmarks
- Adversarial/edge cases
- Acceptance thresholds

Use concrete examples where possible.

## 11. Trade-offs Made

State what is sacrificed and why.

Examples:
- Memory vs latency
- Exactness vs incremental updates
- Simplicity vs future flexibility
- Hebrew canonical purity vs multilingual convenience
- Provenance richness vs compact storage

If you claim no trade-offs, the answer is incomplete.

## 12. Anti-patterns — What Not To Do

List 3-5 approaches that may look reasonable but are wrong or dangerous for
ZETS.

Each anti-pattern must include why it fails specifically under ZETS invariants.

## 13. The “+1 Subtle Thing”

Give one non-obvious insight that materially improves the architecture.

It must be:
- Specific
- Implementable or testable
- Connected to ZETS
- Not a slogan

## 14. Confidence, Self-Rating, and What Would Change My Mind

**Self-rating:** X/10  
**Confidence:** Low / Medium / High

Why not 10/10:
- ...
- ...

What would change my recommendation:
- ...
- ...

To reach 10/10, I would need:
- Specific missing data:
- Specific benchmark:
- Specific design decision:

## 15. Best Follow-Up Question

Provide the single best next question to ask.

Format:
> "<exact question>"

Explain in one sentence why this question unlocks the next level of certainty.

=== LANGUAGE, STYLE, AND LENGTH ===

- English.
- Technical, direct, and concrete.
- Default length: 900-1400 words.
- Use tables where they improve comparability.
- Do not pad.
- Do not flatter the user.
- Do not hide uncertainty.
- Numbers > adjectives.
- Mechanisms > opinions.
- ZETS-native > generic best practice.
```

---

# Usage Rules

## 1. Fill Every Input Field

Do not leave placeholders empty. If unknown, write `unknown` explicitly and add
why it is unknown.

Bad:
```text
Memory: low
```

Good:
```text
Memory: <200MB incremental overhead; total process must remain <6GB>
```

## 2. Ask One Primary Question

The template works best when the council member can answer one architectural
decision. If there are five decisions, create five consultations or mark one as
primary and the others as dependencies.

## 3. Provide Real Constraints

A council answer without budgets will invent budgets. Always provide:

- Memory budget
- Latency budget
- Graph scale or estimated scale
- Update frequency
- Integration points
- What cannot change

## 4. Preserve Determinism in the Prompt

When asking about algorithms, explicitly state whether these are allowed:

- Multithreading
- Floating point
- Randomized algorithms
- Approximate indexes
- External databases
- Learned models
- Background mutation

If allowed, require deterministic execution semantics.

## 5. Compare Council Outputs Using the Same Rubric

When running multiple models, score each response on:

| Criterion | Weight |
|---|---:|
| ZETS invariant compliance | 25% |
| Specificity / implementability | 20% |
| Concrete numbers and complexity | 15% |
| Failure analysis | 15% |
| Determinism / reproducibility | 10% |
| Validation plan | 10% |
| “+1 subtle thing” quality | 5% |

---

# Recommended Council Workflow

```python
prompt = TEMPLATE.format(
    gap_name="Truth Maintenance System Deep Implementation",
    decision_needed="Choose the storage and query architecture for provenance-aware truth maintenance",
    context="""
ZETS needs deterministic truth maintenance over graph facts, edges, and Article
Path Graph walks. The system must track provenance per assertion, support trust
scoring per source, and prevent factual hallucination by exposing evidence gaps.
The current graph uses compact atoms and CSR-style traversal. The design must
fit laptop-scale CPU-only deployment.
""",
    current_zets_state="""
Graph size: unknown; assume 10M-100M edges for estimates.
Atom format: 8-byte atoms, exact bit layout still under review.
Storage: CSR adjacency + Article Path Graph.
Query patterns: walk verification, provenance lookup, contradiction checks.
Update pattern: batch imports plus user-approved edits.
Relevant modules: parser, canonical Hebrew layer, walk engine.
""",
    constraints="""
Memory budget: <200MB incremental RAM overhead.
Latency budget: <5ms p95 for walk evidence check on warm cache.
Build budget: <30 minutes for 10M edges on laptop.
Determinism: multithreading allowed only with deterministic merge order.
Compatibility: Rust stable; binary format versioned.
Must integrate with: walk engine, source registry, user approval UI.
Must not do: embedding-only trust, external DB dependency, nondeterministic ranking.
""",
    question="""
What is the best deterministic, graph-native TMS storage and query architecture
for ZETS under these constraints?
"""
)

council = pick_7_members(topic_type="architecture")
results = parallel_consult(council, prompt)
final = shvirat_kelim_synthesis(results, rubric=ZETS_RUBRIC)
```

---

# Coding Variant

For implementation consultations, modify the role and add a code section.

Add under `ROLE & STANDARD OF REVIEW`:

```markdown
You are also a senior Rust systems engineer. Any code you provide must compile
on stable Rust, follow Rust 2024 edition idioms where applicable, avoid unsafe
unless explicitly justified, and include deterministic tests.
```

Add to `REQUIRED OUTPUT STRUCTURE`:

```markdown
## Implementation Sketch

Provide compilable or near-compilable Rust for the core types/functions.
Include:
- Data structures
- Serialization boundaries
- Error types
- Tests or golden cases
- Complexity notes
```

Additional coding constraints:

- State crate dependencies and why each is acceptable.
- Avoid global mutable state.
- Avoid nondeterministic `HashMap` iteration in output logic unless using fixed
  ordering or deterministic hashers.
- Avoid floating point unless using controlled deterministic semantics.
- Include property tests where useful.

---

# Synthesis Prompt for Combining Council Answers

Use this after receiving multiple model responses.

```markdown
You are synthesizing multiple AI council answers for a ZETS architecture
decision.

Input:
- The original ZETS council prompt
- N model responses
- The ZETS rubric

Task:
1. Extract each response’s core recommendation.
2. Score each response using the rubric.
3. Identify agreements, contradictions, and unique insights.
4. Detect any violation of ZETS invariants.
5. Produce one final architecture recommendation.
6. Include what was adopted, rejected, and why.
7. Preserve the strongest “+1 subtle thing” or synthesize a better one.
8. End with the single next experiment or decision.

Output:
## Council Summary
## Rubric Scores
## Agreements
## Disagreements
## ZETS Invariant Violations
## Synthesized Recommendation
## Adopted / Rejected Ideas
## Final Risks
## Next Experiment
```

---

# Where to Store Outputs

| Document | Role |
|---|---|
| `docs/00_doctrine/AI_COUNCIL_YYYYMMDD.md` | Council members and model roles |
| `docs/00_doctrine/COUNCIL_PROMPT_TEMPLATE_YYYYMMDD.md` | This template |
| `docs/40_ai_consultations/<topic>/<gap>_PROMPT.md` | Filled prompt |
| `docs/40_ai_consultations/<topic>/<gap>_raw.json` | Raw model outputs |
| `docs/40_ai_consultations/<topic>/<gap>_SYNTHESIS.md` | Final synthesis |
| `docs/40_ai_consultations/<topic>/<gap>_DECISION.md` | Accepted decision and rationale |

---

# Changes from v1 (original)

1. **Added explicit decision framing**
   - v1 asked for a question but not the decision the answer should enable.
   - v2 adds `DECISION NEEDED` so models produce actionable recommendations.

2. **Added current-state inputs**
   - v1 had only short context and constraints.
   - v2 asks for graph size, atom constraints, storage layout, query patterns,
     update pattern, and integration points to reduce shallow generic answers.

3. **Strengthened ZETS invariants**
   - Added deterministic execution details: ordering, hashes, concurrency,
     floating point, external state.
   - Added versioned reproducibility.
   - Clarified embeddings as auxiliary only.
   - Added canonical Hebrew morphology concerns.
   - Added user-controlled self-extension and rollback/provenance boundaries.

4. **Replaced vague “accuracy %” with measurable quality metrics**
   - v1 required accuracy even where inappropriate.
   - v2 asks for the relevant metric and labels numbers as measured, derived,
     estimated, or assumed.

5. **Added assumptions table**
   - Models often hide assumptions.
   - v2 forces assumptions, why they matter, and what changes if false.

6. **Added interface/schema/pseudocode requirement**
   - v1 could still yield architectural prose.
   - v2 requires an implementation-shaped artifact.

7. **Added ZETS invariant compliance matrix**
   - v1 said “respect invariants” but did not force auditability.
   - v2 makes each invariant explicitly pass/fail/partial with risks.

8. **Added alternatives comparison**
   - v1 asked for anti-patterns but not serious alternatives.
   - v2 requires comparing attractive options and rejecting them for ZETS-specific reasons.

9. **Added determinism and reproducibility section**
   - This is central to ZETS and was under-enforced in v1.
   - v2 requires ordering rules, hashing/versioning, concurrency, serialization,
     floating-point strategy, and golden outputs.

10. **Added validation plan**
    - v1 asked for numbers but not how to verify them.
    - v2 requires tests, benchmarks, adversarial cases, and acceptance thresholds.

11. **Added confidence and “what would change my mind”**
    - v1 had self-rating but not epistemic humility.
    - v2 forces falsifiability and decision reversal criteria.

12. **Tightened internal thinking language**
    - v1 asked for hidden versions but risked inviting chain-of-thought leakage.
    - v2 keeps internal synthesis while explicitly forbidding raw hidden reasoning output.

13. **Removed overly mystical ambiguity**
    - v1’s “kabbalistic systems thinker” could produce poetic answers.
    - v2 reframes exploration as unconventional graph/walk/interference design while preserving the shvirat kelim concept.

14. **Added council synthesis prompt**
    - v1 described synthesis informally.
    - v2 provides a structured synthesis prompt and rubric.

15. **Added coding-specific determinism warnings**
    - v1 mentioned Rust coding briefly.
    - v2 adds stable Rust, deterministic tests, dependency justification,
      `HashMap` ordering risk, floating-point caution, and global state avoidance.

16. **Improved storage/document workflow**
    - v1 listed storage files.
    - v2 separates prompt, raw outputs, synthesis, and final decision artifacts.

17. **Increased default answer length**
    - v1’s 600-900 words was too tight for serious architecture review.
    - v2 uses 900-1400 words while still forbidding padding.

18. **Made impossibility an acceptable answer**
    - v1 encouraged compliance even when constraints may conflict.
    - v2 instructs the model to reject impossible or non-ZETS-native designs clearly.