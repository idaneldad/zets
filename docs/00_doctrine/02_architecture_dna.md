# Architecture DNA — what makes ZETS ZETS

**Date:** 2026-04-23  
**Status:** Living document — update when adding/changing core principles  
**Read this before:** designing any new module, evaluating an external project, deciding what to build

---

## The five non-negotiable principles

These are not goals or aspirations. They are constraints. Code that violates them does not belong in ZETS.

### 1. Everything is a graph atom or edge

If you find yourself writing a `HashMap<String, T>` outside an indexing layer, you're probably doing it wrong. Knowledge, procedures, sense, permissions, audit logs — all are atoms with edges to other atoms.

**Why:** A graph is the only data structure where:
- Sub-things naturally compose into bigger things
- Cross-references work in both directions
- Storage scales with information content, not container count
- Self-modification is just adding nodes

**The price:** Lookup is sometimes slower than direct array access. mtreemap mitigates this (98% cache hit on cluster-aligned access).

### 2. Bounded execution always

The system_graph VM has hard limits: max_call_depth=32, max_ops_per_run=10K, 16 registers. **These are immutable.** Procedures that need more work either decompose into sub-procedures or are not allowed.

**Why:** Without bounded execution, self-modification leads to runaway computation, and trust in learned procedures becomes impossible.

### 3. Knowledge enters as atoms+edges only

No `STATIC_FACTS` lookup tables. No `if word == "שלום" return "hello"`. No "configuration" that contains domain knowledge.

When ZETS needs to know "שלום means hello", that's:
- atom: `word:שלום`
- atom: `sense:greeting.open`
- atom: `word:hello`
- edge: `word:שלום --expresses_sense--> sense:greeting.open`
- edge: `word:hello --expresses_sense--> sense:greeting.open`

**Why:** Knowledge in code = stale, untestable, untraceable. Knowledge in graph = updateable, queryable, auditable.

### 4. Procedures are also atoms

A procedure is not a function — it's a node in the graph that has steps (each itself an atom or a CallProcedure to another procedure atom). Bytecode is only the leaves.

**Why:** This is what enables capability registry, deduplication, learning new procedures from external sources, audit trails. See ADR 0004.

### 5. Trust levels gate all execution

Every procedure has a TrustLevel: System, OwnerVerified, Learned, Experimental. The VM enforces execution mode based on trust:
- System + OwnerVerified → direct execution
- Learned → sandboxed (transactional, rollback on fail)
- Experimental → simulation only (returns trace, no side effects)

**Why:** ZETS will eventually learn procedures from external sources. Without trust gates, a mis-extracted procedure could corrupt the brain.

---

## The kabbalistic backbone (this matters)

ZETS's architecture is intentionally aligned to traditional Jewish mystical structures because they happen to map cleanly onto computational primitives:

- **10 sefirot ↔ 10 pipeline stages** (Keter through Malkhut)
- **22 Hebrew letters ↔ 22 base edge types** (each letter is a connection type)
- **5 partzufim ↔ 5 walk modes**
- **7 angels ↔ 7 background daemons**
- **231 gates ↔ 231 reasoning patterns** (combinatorial pairs)
- **32 paths of wisdom ↔ 32 opcodes**

This is not theology. It's a forced-coherence design constraint that prevents random expansion. If you want to add a new opcode, the question is: which path of wisdom does it correspond to? If you can't answer, the opcode probably isn't fundamental enough to add.

**Practical implication:** when you propose a new module, check the kabbalistic mapping. If it doesn't fit, either there's a missing core piece or the proposal is too narrow.

---

## What ZETS deliberately is NOT

### Not a chatbot
ZETS is a knowledge engine. Chat interfaces are downstream consumers (Lev, Cortex). Don't build chat-specific features into ZETS core.

### Not an LLM wrapper
LLMs are external helpers, not the source of intelligence. ZETS does symbolic graph reasoning. LLMs may be invoked for:
- Natural language → spec translation
- Spec → fresh code generation (anti-copying)
- Explanation generation (graph trace → prose)

But never: as the primary reasoner, as the primary store, as the source of truth.

### Not a microservices platform
Single-binary, single-process Rust core. mcp/ Python is a temporary bridge to be replaced.

### Not a vector database
Embeddings are an INDEX over the graph, not the graph itself. Don't store an embedding without a corresponding atom.

### Not a workflow engine
Workflow engines (Airflow, Temporal) optimize for reliability of long-running multi-step jobs. ZETS optimizes for fast, small, composable graph-walks. Different trade-offs.

---

## Decision shortcuts

When in doubt, apply these in order:

1. **"Can this be an atom + edges?"** → if yes, do that
2. **"Does an existing capability already do this?"** → if yes, compose it (don't duplicate)
3. **"Is this knowledge or code?"** → knowledge → graph; code → src/*.rs
4. **"Does this need bounded execution?"** → if it touches procedures, yes
5. **"Will this still make sense in 5 years?"** → if no, don't build it now

---

## What we learned the hard way

A non-exhaustive list of mistakes we've made and how to avoid repeating them:

### "שלום ≠ hello" mistake (23.04.2026)
Initial design used SAME_AS edges between language-equivalent words. This is wrong — שלום covers 3 senses, hello only 1. The correct model is words → senses → words (WordNet synsets).

**Lesson:** When in doubt about cross-language equivalence, model through abstract sense atoms, not direct word-to-word edges.

### "phi_check rewards Fibonacci length" mistake (Lev, before)
A scoring function that boosted answers near phi-ratio length empirically chose worse answers because length-bias is not semantic relevance.

**Lesson:** Don't use mathematical aesthetics as a proxy for semantic quality. Test on real data.

### "FLAT mmap" missed opportunity (until 23.04.2026)
The initial mmap layout was insertion-order. Cluster-tree layout (mtreemap) gave 39× fewer pages for typical walks. This was a 2-week task that should have been done from day one.

**Lesson:** Storage layout matters for graph data because traversals are the dominant access pattern. Spend the time to align physical layout with semantic locality.

### "diagnostic labels in code" mistake (corrected via Gemini)
Search strategies were initially named ADHD, Asperger, Savant, etc. Per Gemini's critique these are inappropriate for engineering abstractions.

**Lesson:** Use behavioral descriptors (Precise, Exploratory, PolymathWeaving) — never diagnostic labels. Compile-time test enforces this.

---

## Cross-references

- `00_doctrine/01_engineering_rules.md` — Idan's binding process rules
- `30_decisions/` — specific architectural decisions (ADRs)
- `10_architecture/` — current state of the system
