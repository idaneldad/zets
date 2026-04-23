# What to build, what to skip — operational decisions

**Date:** 2026-04-23  
**Status:** Living document  
**Read this when:** evaluating a new feature request, prioritizing sprints, deciding scope

---

## Build NOW (active development)

- **Capability Registry** — see ADR 0004
- **Procedure ingestion pipeline** — read external repos → graph procedures
- **API learning** — read OpenAPI/Swagger → working procedure
- **Audit trail opcode** — every invocation logged to WAL
- **Fallback edges** — graceful degradation between procedures

## Build NEXT (planned, sequenced)

- Hot-reload procedure scanner (background task)
- Lazy bytecode load via mtreemap
- Pre-commit hook: anti-copying enforcement
- Common-sense bootstrap (ConceptNet + ATOMIC ingestion)
- Goal manager (priority queue + meta-reasoning)

## Build LATER (good ideas, wrong time)

- Mobile app (iOS/Android) — wait until core API is stable
- Voice wake word — wait until on-device LLM integration
- Live canvas UI — wait until product-market fit
- Federation between ZETS instances — wait until single instance proven
- Marketplace for procedures (ClawHub-equivalent) — wait until 50+ verified procedures exist

## Build NEVER

These are deliberate non-goals. Saying NO is part of the architecture.

### Don't: Become a chat application
Lev/Cortex/other consumers can be chat. ZETS itself is a knowledge engine API. Resist requests to "add a chat interface" — direct them to the consumer products.

### Don't: Wrap LLMs as the primary brain
LLMs are external helpers. The graph is the brain. If a feature requires "ask LLM and remember answer", it's wrong — the answer should be derivable from the graph.

### Don't: Add a configuration system for domain knowledge
Configuration is for paths, ports, secrets. Domain knowledge ("hospital opening hours", "currency exchange rates") goes in the graph as atoms+edges.

### Don't: Implement microservices
Single binary. mcp/ Python is being decommissioned, not extended.

### Don't: Add Python dependencies to core
src/*.rs is Rust-only. py_testers/ is for prototyping. mcp/ is for external tools only and is decommissioning.

### Don't: Use diagnostic labels in code
ADHD/Autistic/Savant/Asperger are forbidden as identifiers. Use behavioral descriptors. Compile-time test in `src/search/strategy.rs` enforces this.

### Don't: Store source code from external repos
Anti-copying requires that ingestion stages 1-2 (extract behavior, canonicalize) work on AST/spec only. Source code is processed in temporary buffers and discarded. See ADR 0004 + lessons doc.

### Don't: Add LLM API keys to data graph
Secrets stay in `~/.env` with chmod 600. The graph references SecretId atoms; the actual secret value is resolved at runtime from env, never stored in graph.

### Don't: Sacrifice bounded VM for "convenience"
max_call_depth=32, max_ops=10K. These are not negotiable. Procedures that need more must decompose.

### Don't: Implement features just because OpenClaw has them
OpenClaw has 117 extensions and 53 skills. Most are noise. We import patterns, not feature counts.

---

## When asked "should we add X?"

Filter through these questions in order. If any answer is "no", default is don't build.

1. **Does X solve a real, current pain?** Or is it speculative?
2. **Can X be expressed as atoms+edges in the existing graph?** Or does it need a new subsystem?
3. **Does X compose with existing capabilities?** Or is it a parallel implementation?
4. **Will X make ZETS more or less coherent with the kabbalistic backbone?**
5. **Will X break any of the 5 doctrine principles?**
6. **Is the maintenance cost forever?** Or one-time?
7. **What's the test that proves X works?** If we can't write it, we can't build it.

---

## Sprint sizing rules

- Sprint = 1 week
- Max 5 deliverables per sprint
- Each deliverable: code + tests + ADR (if architectural) + docs update
- If a deliverable exceeds 1 week → split it

---

## Cross-references

- `02_architecture_dna.md` — the 5 non-negotiable principles
- `04_anti_patterns.md` — TBD
- `30_decisions/` — historical decisions with consequences
