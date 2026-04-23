# ZETS AGI Readiness Roadmap

**Last updated:** 23-Apr-2026 (session #2 — audit + multi-modal missions)
**Prev baseline:** 22-Apr-2026 (b0bb742 + AGI prep)
**Commit at last update:** 5ed3d3a (+ local memory-infra commit pending)
**Code size:** ~45K lines Rust, 223 modules  *(claim — to be verified by mission P-M)*
**Tests:** *(claim: 1,278 passing — to be verified by mission P-M)*

See also: `CLAUDE.md`, `STATUS.md`, `docs/VISION_VS_REALITY.md`,
`docs/CLAUDE_ACTIONS_AUDIT.md`.

---

## Honest assessment — where we stand (23.04.2026)

### What ZETS has

| Capability | Status | Evidence |
|---|---|---|
| Dictionary lookup | ✓ Working | 144,670 concepts, 1.6µs query (claim) |
| Morphology | ✓ Working | 16 languages, data-driven (`src/morphology/`) |
| Binary pack format | ✓ Working | `src/pack.rs`, `src/mmap_core.rs` |
| Persistence (WAL) | ✓ Working | Crash-safe, append-only (`src/wal.rs`) |
| Encryption | 🟡 Partial | AES-256-GCM works; Argon2 key derivation pending |
| System graph + VM | ✓ Working | 33 opcodes, bounded recursion |
| 6-graph scopes | ✓ Working | System / User / Data / Language / Log / Testing |
| Cascade routing | ✓ Working | 25-48ns per scope check (claim) |
| Trust weighting | ✓ Working | Auto-recalibration |
| Testing sandbox | ✓ Working | Stage → Test → Verify → Promote |
| Multi-hop reasoning | ✓ Working | 1.2µs per 3-hop query (is_ancestor) |
| Metacognition | ✓ Working | Gap detection + learning proposals |
| Capability runtime | ✓ Working | `src/capability_runtime/` |
| Calibration harness | ✓ Working | ECE, Brier, KIG (Know/Infer/Guess) |
| Preference store | ✓ Working | Inferred from conversation |
| Canonization engine | ✓ Working | Variant → canonical with epistemic classification |
| Procedure-atom infra | ✓ Working | `src/procedure_atom.rs`, `src/procedure_template/` |
| MCP server | ✓ Working | `mcp/zets_mcp_server.py` |

### What ZETS is missing (honest gap list)

| Capability | Status | Needed for AGI |
|---|---|---|
| Semantic edges in data | ✗ Mostly 0 | Fundamental — dictionary vs graph |
| Causal reasoning routes | ✗ None | Why? questions |
| Temporal reasoning | ✗ None | Before/after, duration, sequence |
| Theory of mind | ✗ None | Understanding user context |
| Natural language parser | ✗ None | Query → graph coordinates — mission **P-I** |
| Natural language generator | ✗ None | Graph → sentence with morphology — mission **P-I** |
| Common ancestor (LCA) | ✗ None | Similarity, analogy — **~100 bytes of bytecode** |
| Contradiction detection | ✗ None | Consistency |
| Active learning loop | 🟡 Partial | MetaCognition detects; no actuation (mission **P-D**) |
| Knowledge breadth | ✗ Small | 144K vs Wikidata's 100M |
| **HTTP fetch primitive** | **✗ None** | **mission P-A** |
| **HTML parser primitive** | **✗ None** | **mission P-B** |
| **Procedure loader** | **✗ None** | **mission P-C** |
| **Image understanding (no LLM)** | **✗ None** | **mission P-N (new)** |
| **Speech (no LLM)** | **✗ None** | **mission P-O (new)** |
| **Video understanding** | **✗ None** | **mission P-Q (new)** |
| **Provenance DB (client recovery)** | **🟡 Partial** | **mission P-P (new)** |
| **Code quality (duplication → graph gaps)** | **✗ Not audited** | **mission P-M (new — runs first)** |

---

## Quantified gap to top-tier AI

| Metric | ZETS today (claim) | GPT-4 / Claude | Gap |
|---|---|---|---|
| Concepts | 144K | ~billions | 10,000× |
| Reasoning routes | 1 (`is_ancestor`) | ∞ chains | ∞ |
| Languages (morphology) | 16 | 100+ | 6× |
| Hallucination rate | **0%** | 5-15% | **ZETS wins** |
| Determinism | **100%** | stochastic | **ZETS wins** |
| Explainability | **full trail** | opaque | **ZETS wins** |
| Privacy | **local+encrypted** | cloud | **ZETS wins** |
| Image understanding | **✗ none** | multimodal | **gap — P-N** |
| Speech | **✗ none** | multimodal | **gap — P-O** |
| Video | **✗ none** | limited | **gap — P-Q** |
| Math reasoning | **✗ none** | good-but-errs | **gap — P-H** |

---

## Phased roadmap — realistic milestones (revised 23.04.2026)

### Phase 0 — Code-quality baseline (ONE WEEK — *do this first*)

Mission **P-M** establishes ground truth before we add more code.

- [ ] Run `cargo check --all-targets --all-features` — record warnings.
- [ ] Run `cargo clippy --all-targets -- -D warnings` — record violations.
- [ ] Run `cargo test --all` — verify the "1,278 tests" claim.
- [ ] Run `tokei` and `cloc` — verify the "45K LoC / 223 modules" claim.
- [ ] Static duplication scan (`cargo similar`, `dupdetector`, or manual per-module):
      every duplicated block → "this is a missing graph atom" item.
- [ ] Dead-code scan (`cargo udeps`, `#[allow(dead_code)]` audit).
- [ ] Output: `docs/CODE_QUALITY_REPORT.md` with actionable items grouped by
      *"fix in code" / "lift to graph atom" / "delete dead code" / "keep — justified"*.

**Outcome:** we know exactly what we have, and duplications surface as graph gaps.

### Phase 1 — Reasoning primitives (2-3 weeks)

Add these as bytecode routes (no Rust changes):

- [ ] `is_descendant(a, b, depth)` — inverse of is_ancestor
- [ ] `common_ancestor(a, b, max_depth)` — LCA in IS_A tree
- [ ] `causal_chain(effect, max_depth)` — traverse CAUSES edges backward
- [ ] `part_of_path(whole, max_depth)` — traverse PART_OF
- [ ] `has_property(concept, property)` — check attribute edges
- [ ] `contradiction_detect(concept_a, concept_b)` — find incompatibilities

**Outcome:** ZETS can answer "Why?", "What's similar to?", "What's X made of?"

### Phase 2 — Autonomous learning spine (4-6 weeks, in parallel with Phase 1 where possible)

Missions P-A through P-G land. Zetson infant goes live.

- [ ] **P-A** HTTP fetch primitive (Rust) — robots.txt, rate limits, ETag cache.
- [ ] **P-B** HTML parser primitive (Rust) — Wikipedia-grade.
- [ ] **P-C** Procedure loader + 20 initial procedures (TOML).
- [ ] **P-D** Learning loop executor (walks procedure DAGs).
- [ ] **P-E** Seed loader (YAML → atoms injection).
- [ ] **P-F** Observability dashboard.
- [ ] **P-L** Source registry & trust recalibration.
- [ ] **P-G** Zetson first-day demo — integration.

**Outcome:** ZETS has the skeleton to learn autonomously from web sources.

### Phase 3 — Edge population (1-2 months)

The biggest knowledge gap. Populate the graph with semantic edges.

- [ ] Scanner for Wiktionary dumps → IS_A edges from "X is a Y" definitions
- [ ] ConceptNet import → CAUSES, PART_OF, USED_FOR
- [ ] Wikidata import → instance_of, subclass_of, part_of
- [ ] Target: every concept has ≥3 edges on average

**Outcome:** Data graph becomes actually a graph (not just dictionary).

### Phase 4 — Natural language I/O (2-3 months)

Mission **P-I**. Query understanding + answer generation.

- [ ] Simple intent parser: "what is X?" → lookup(X) + return_gloss
- [ ] "is X a Y?" → is_ancestor(X, Y, 5)
- [ ] "why does X happen?" → causal_chain(X, 5)
- [ ] Answer templates using morphology (Hebrew + English)
- [ ] Confidence-aware hedging

**Outcome:** ZETS answers in natural sentences.

### Phase 5 — Math (2-3 weeks, after Phase 2)

Mission **P-H**.

- [ ] Procedure atoms for add / subtract / multiply / divide on naturals.
- [ ] Word-problem parsing.
- [ ] Wire to MATH-lite benchmark.

**Outcome:** ZETS answers basic arithmetic deterministically, with trace.
**Decision point:** if throughput inadequate, consider `bignum_ops` sub-primitive.

### Phase 6 — Multi-modal (3-4 months, can start parallel with Phase 4)

- [ ] **P-N** Image understanding — OpenCV/scikit-image features → concept atoms.
- [ ] **P-O** Speech — Vosk/Whisper-local + phoneme atoms + Piper TTS.
- [ ] **P-Q** Video — composition of P-N + P-O + temporal edges.

**Outcome:** ZETS sees, listens, watches — all offline, all explainable.

### Phase 7 — Active learning closed loop (3-4 months)

Close the metacognition loop.

- [ ] Sandbox receives learning proposals from metacognition.
- [ ] Each proposal → sandbox stages concept+edge candidates.
- [ ] Tests run automatically (no conflict, source trusted, confidence threshold).
- [ ] Verified proposals promoted to Data scope.
- [ ] Trust profiles updated based on corroboration.

**Outcome:** ZETS learns from its own failures, without human intervention.

### Phase 8 — Dense DB productization (1-2 months)

Mission **P-P**.

- [ ] CLI: `zets recover <client-id> --to <dir>`.
- [ ] API: any atom → full write history reconstructable.
- [ ] Benchmark: atoms/MB, recovery latency.
- [ ] Update `PRODUCT.md` with the storage-engine section.

**Outcome:** ZETS can be sold as a dense provenance-preserving graph DB in addition
to its cognitive role. This is a separate business line.

### Phase 9 — Benchmarks & honest leaderboard (ongoing)

Mission **P-K**.

- [ ] Wire MMLU-lite, HumanEval-lite, MATH, GPQA.
- [ ] Publish leaderboard: ZETS vs GPT vs Claude vs Gemini — with honest
      "N/A for creative prose" in appropriate cells.
- [ ] Update investor materials to reflect measured numbers.

### Phase 10 — Specialization tracks (ongoing)

Domain experts, one at a time — each a composition of existing primitives +
new procedures + domain knowledge atoms:

- [ ] Programming (Rust, Python) — symbol graphs + docs
- [ ] Medicine — SNOMED CT import + causal edges
- [ ] Law — case-based reasoning routes
- [ ] Jewish wisdom / Kabbalah — already partially present (`src/wisdom_engines/`)

---

## Shattered assumptions (preserved from prior session, still relevant)

1. **"AGI = bigger model"** — False. GPT-4 still hallucinates 10%.
2. **"AGI = more data"** — Necessary but insufficient.
3. **"Neural is the way"** — Neurosymbolic is consensus (Bengio/Marcus/Pearl).
4. **"ZETS is close to AGI"** — False. Missing multi-hop, causal, temporal, theory of mind, multi-modal, NL I/O, math, and most of the graph itself.
5. **"ZETS is far from AGI"** — False. Has unique deterministic advantages and a clean architecture for getting there incrementally.
6. **"Must mimic LLMs"** — No, complement them (symbolic side).
7. **"Reasoning = more opcodes"** — No, routes + edges making chains.
8. **"More languages = smarter"** — Wrong, connections matter more than quantity.
9. **"System Graph solved it"** — Only infrastructure; 3 routes not 3,000.
10. **"Turing Test = AGI"** — Already broken 1996; AGI = learn any new task.
11. **(New 23.04.2026) "Multi-modal needs LLMs"** — False. Classical CV/DSP + learned feature→concept atoms suffices for a long way and stays explainable.
12. **(New 23.04.2026) "Duplicate code is fine if cleaner"** — False. Every duplication is a missing graph atom.

---

## What changed in this session (23.04.2026 session #2)

### Additions to the roadmap
- Phase 0 inserted (code quality baseline) — **runs first**.
- Phase 6 (multi-modal) added as explicit phase with three missions.
- Phase 8 (dense DB productization) added as business line.
- Five new missions (P-M, P-N, P-O, P-P, P-Q) specced and queued for Claude Code.

### Infrastructure additions
- `CLAUDE.md` — root entry point for any Claude/agent.
- `STATUS.md` — living state across sessions.
- `docs/VISION_VS_REALITY.md` — honest map.
- `docs/CLAUDE_ACTIONS_AUDIT.md` — every Claude action → graph migration.
- `docs/DECISIONS_LOG.md` — append-only decision trail.
- `.gitignore` hardened (17GB of wikipedia dumps no longer tracked).

### Clarifications
- "No code duplication — duplication is a graph gap" rule made explicit.
- Multi-modal stack pinned: OpenCV / scikit-image / librosa / Vosk / Whisper-local / Piper / Tesseract. No Google Cloud.
- Claude-access model documented in CLAUDE.md §5.

---

## The next concrete session

Priority order:
1. **Run mission P-M (code quality audit) via Claude Code on ddev.** Non-interactive,
   writes `docs/CODE_QUALITY_REPORT.md`. **This is the single highest-leverage
   move right now** because everything else assumes we know the current state.
2. Review P-M output with Idan. Decide which duplications become graph atoms
   (mission P-M-fix), which are justified to keep, which are dead code to delete.
3. Start P-A + P-B in parallel.

**After those 3 steps: ZETS has verified ground truth, and the Zetson bootstrap
starts for real.**
