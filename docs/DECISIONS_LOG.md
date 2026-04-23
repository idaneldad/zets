# DECISIONS_LOG.md — rolling log of decisions, renames, and deletions

> **Purpose:** future Claudes, future Idan, and future anyone-else should be able
> to answer two questions from this file alone:
> 1. *Why did we do X this way?*
> 2. *Was there something here that isn't here anymore, and why did we remove it?*
>
> **Rule:** append-only. Never rewrite history. If a past decision was wrong,
> add a new entry saying so — don't edit the old one.

---

## How to add an entry

```
### YYYY-MM-DD — short title
**Context:** one sentence on what problem prompted the decision.
**Decision:** what we chose.
**Alternatives rejected:** quick list with one-liner reasons.
**Council?** list which 5 experts were consulted (if a weighty call).
**Revisit when:** condition under which this should be re-opened.
```

---

## 2026-04-23 — Memory-infrastructure layer (5 files, one reading order)

**Context:** Across sessions (and potentially parallel sessions), Claude keeps
restarting from zero: no canonical entry point, no living state file, no shared
understanding of vision-vs-reality or what Claude-actions still need graph coverage.

**Decision:** Introduce 5 files with strict roles, and a strict reading order
documented in the root `CLAUDE.md`:
1. `CLAUDE.md` (root) — principles, reading order, dos/don'ts.
2. `STATUS.md` (root) — live state, session log, next-actions queue.
3. `docs/VISION_VS_REALITY.md` — honest map of dreamed vs built.
4. `docs/CLAUDE_ACTIONS_AUDIT.md` — every Claude action → graph migration plan.
5. `docs/DECISIONS_LOG.md` (this file) — why-we-did-it log + deletions.

**Alternatives rejected:**
- *Single mega-file* — bad: multiple concerns, too long, no append discipline.
- *GitHub Issues only* — not repo-local; future Claudes don't always have GH access.
- *One per session* — ephemeral, loses compounding value.

**Council:**
- SRE: "one canonical entry point, don't split the truth."
- Technical Writer: "hierarchy INDEX → STATUS → details."
- Git Engineer: "session log with branch names + status fields."
- Product Manager: "vision vs reality on one page, gap list owned."
- AI/Agent Architect: "machine-readable (YAML) sections so agents can parse."

**Revisit when:** we have ≥3 parallel Claude Code sessions running and the STATUS.md
session-log pattern either scales or doesn't.

---

## 2026-04-23 — Core architectural principle (code vs graph split)

**Context:** Idan articulated the rule; session produced
`docs/ZETS_LEARNING_HOW_IT_LEARNS.md` formalising it.

**Decision:** Lock the 7-primitive rule. Rust contains only:
`fetch`, `parse`, `tokenize`, `store`, `retrieve`, `reason`, `communicate`.
Everything else — including math, language learning strategies, source trust,
curriculum, dialogue style, image analysis meaning-layer, speech meaning-layer —
goes in the graph as procedure / knowledge / motivation atoms.

**Alternatives rejected:** math as Rust module (rejected — contradicts principle),
each language as its own Rust module (rejected — morphology data is TSV),
curriculum as Rust config (rejected — curriculum is the graph's business).

**Revisit when:** a procedure is demonstrably impossible to express efficiently
in bytecode after real-world measurement (not conjecture).

---

## 2026-04-23 — No code duplication; duplication is a graph gap

**Context:** Idan said "שום קוד לא צריך לחזור על עצמו כי זה למעשה קשתות למושגים
שצריך שיהיו בגרף". This extends the core principle in an important direction.

**Decision:** When Rust code duplicates (same logic in two places), the
duplication is a *signal*. The response is:
1. Identify the shared concept.
2. Lift it to a graph atom (procedure or knowledge).
3. Replace both sites with a graph walk, not a shared Rust helper.

Shared helper functions in Rust are a smell, unless they are one of the 7
primitives or trivial plumbing (< 5 lines).

**Mission P-M (code quality audit)** was created specifically to hunt down
duplications and produce a "duplication → graph-gap" report as output.

**Revisit when:** P-M report is in hand — may reveal exceptions where the
rule is impractical.

---

## 2026-04-23 — Seven (now twelve, now seventeen) agent missions

**Context:** Moving from spec to execution requires parallelization across Claude Code.

**Decision:** Missions P-A through P-Q, four+ tracks:
- Track 1 (primitives, parallel): P-A, P-B
- Track 2 (procedure infra, parallel): P-C, P-D, P-E
- Track 3 (observability): P-F
- Track 4 (domain packs, some parallel): P-H, P-I, P-J, P-L
- Track 5 (benchmarks): P-K
- Track 6 (multi-modal, added 23.04.2026 session #2): P-N, P-O, P-Q
- Track 7 (persistence role, added 23.04.2026 session #2): P-P
- Cross-cutting (code quality): **P-M — runs FIRST**

**P-M ordering decision:** we run code quality audit **before** new feature missions,
because if the repo has duplications / dead code / test failures hiding,
adding more code on top multiplies the mess.

---

## 2026-04-23 — Multi-modal strategy: no LLM in runtime

**Context:** Idan asked about image, speech, and video understanding.

**Decision:** All modalities processed by classical signal-processing and
computer-vision primitives (OpenCV, scikit-image, librosa, Vosk, Piper, Tesseract).
No LLM call at runtime. LLM may be allowed *during bootstrap* for labeling
training examples, but only if offline (e.g. Whisper-local), and the learned
atoms must be explainable without the LLM.

**Council for this decision:**
- Vision researcher: "Classical features (LBP, HOG, SIFT) + learned feature→concept edges is sufficient for a very long way, and it's explainable."
- Signal-processing engineer: "MFCC + VAD + Vosk covers 95% of speech needs offline."
- Systems architect: "No LLM runtime dep keeps the edge story real."
- Product manager: "Users who care about ZETS buy it specifically because there's no LLM — don't break that promise."
- Safety/Privacy expert: "Speech + video + LLM = data that leaves device = privacy problem. Classical pipeline stays local."

**Rejected:** using Google Speech-to-Text for bootstrap. Council:
privacy violation, wrong thesis, creates cloud dependency we'd have to rip out later.

**Revisit when:** we hit a specific task where classical methods provably fail
(e.g., translating spoken Hebrew dialects with no available Vosk model).

---

## 2026-04-23 — Dense DB / provenance as product feature

**Context:** Idan: "בגרף שלנו הוא DATABASE אמין שפשוט שמור בשיטת גרף — זה נכון
להכל ולכן חשוב לוודא את זה". That is, the graph must double as a reliable
customer-data database, with dense storage and fast recovery.

**Decision:** Treat "dense storage + provenance + fast recovery" as a first-class
product feature. Add section to `PRODUCT.md`. Create mission **P-P** to:
1. Measure current density (atoms/MB, edges/MB).
2. Document the recovery API (any atom → its full write history).
3. Add a CLI: `zets recover <client-id> --to <dir>` that reconstructs
   a client's full data set from their atoms + provenance.
4. Benchmark recovery latency.

**Revisit when:** P-P lands. Then ZETS can claim "dense DB with provenance" in
investor materials with a reproducible benchmark.

---

## 2026-04-23 — Naming: "Zetson" for the infant ZETS

**Context:** We need a clear handle for the "ZETS that learns everything from scratch."

**Decision:** **Zetson** (= "ZETS son"). Launches with `config/zetson-infant.yaml`
as its seed, empty graph, capped budget, zero LLM access.

**Revisit when:** Zetson graduates past "Self-Aware" stage (90+ days).

---

## 2026-04-23 — PRODUCT.md rendering — not a real bug

**Context:** Idan reported a GitHub loading error on `docs/PRODUCT.md`.

**Decision:** Verified via GitHub's markdown rendering API (`POST /markdown`):
file renders as HTTP 200 with 62KB of valid HTML, all headings in order.
Conclusion: transient (likely CDN lag shortly after push, or browser cache).
No code change made.

**Revisit when:** Idan reports it again after a fresh reload.

---

## (Earlier decisions, pre-23.04.2026)

Entries before today are scattered across `docs/_archive/`. Search there when
investigating a past decision.

---
---

# 🗑️ DELETION / RENAME LOG

> **Rule:** anything removed or renamed gets logged here with a one-liner.

**Format:**
```
- YYYY-MM-DD | removed/renamed/created | <path> | <why + where it went>
```

- 2026-04-23 | created | `CLAUDE.md`, `STATUS.md`, `docs/VISION_VS_REALITY.md`, `docs/CLAUDE_ACTIONS_AUDIT.md`, `docs/DECISIONS_LOG.md` | first memory-infrastructure commit (session #2)
- 2026-04-23 | gitignored | `data/autonomous/`, `data/autonomous_cache/`, `data/wikipedia_dumps/` | runtime data, regenerable. Wikipedia dumps were 17GB of bloat. Remove from tracked history was deemed too aggressive — left in git history, just stopped tracking new changes.
- 2026-04-23 | gitignored | `mcp/autonomous/status.json`, `mcp/autonomous/multilang_status.json` | runtime state, not source
- 2026-04-23 | lint-cleaned | `src/benchmark/calibration/harness.rs`, `src/error_store/store.rs`, `src/guard/input_guard.rs`, `src/mtreemap.rs`, `src/personal_graph/graph.rs` | removed unused imports / `_atoms_start` underscore prefix; no behaviour change

---

## For future sessions: how to use the deletion log

When a future Claude is searching for a file that seems like it should exist,
**grep this log before hunting**:

```bash
grep -i "<filename-fragment>" docs/DECISIONS_LOG.md
```

If the grep hits, the file was intentionally moved/deleted. Follow the breadcrumb.
If the grep misses, the file really never existed (or was deleted silently — in
which case, the deleter violated this log, please open an issue).

---

*Maintain this file. Future you will be grateful.*
