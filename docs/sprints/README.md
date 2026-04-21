# ZETS Sprint Tracker

**Last updated:** 21.04.2026

## Two parallel tracks

**Track 1 — Cognitive Engine** (most important)
A → B → C → H → **D** → E → F → G → I

**Track 2 — Ingestion Pipeline** (runs parallel from Sprint A+)
ING1 → ING2 → ING3

---

## Status board

### Track 1 (Cognitive Engine)
| ID | Name | Status | Brief | Blocks |
|----|------|--------|-------|--------|
| A | CLI cleanup + iterator APIs | 🟢 Ready | [SPRINT_A.md](SPRINT_A.md) | B, C, ING1 |
| B | Query Planner + Multi-seed | 🟡 Blocked on A | [SPRINT_B.md](SPRINT_B.md) | H |
| C | PreGraphBuilder + multi-lang data | 🟡 Needs brief | – | D |
| **H** | **Sessions + Context Disambig** | 🟡 Blocked on B | [SPRINT_H.md](SPRINT_H.md) | **D** |
| **D** | **Cognitive Tree (7×7 beam + bridges)** | 🔴 Needs design approval | [COGNITIVE_TREE_SPEC.md](../COGNITIVE_TREE_SPEC.md) | E |
| E | Composition Engine (LLM-quality output) | 🟡 Blocked on D | – | F |
| F | Feedback Learner | 🟡 Blocked on E | – | G |
| G | Tool Registry + Permissions | 🟡 Blocked on F | [OPENCLAW_INTEGRATION.md](../OPENCLAW_INTEGRATION.md) | I |
| I | Cloud Relay reference impl | 🟡 Blocked on G | – | – |

### Track 2 (Ingestion)
| ID | Name | Status | Brief | Blocks |
|------|------|--------|-------|--------|
| ING1 | Text + chunks + BLAKE3 | 🟢 Ready (after A) | [UNIVERSAL_INGESTION_ARCHITECTURE.md](../UNIVERSAL_INGESTION_ARCHITECTURE.md) | ING2 |
| ING2 | Docx + PDF unwrappers | 🟡 Blocked on ING1 | – | ING3 |
| ING3 | Edge reader + pack sync | 🟡 Blocked on ING2 | – | – |

---

## Critical path

For Idan to experience an AGI-like system, the path is:
**A → B → C → H → D → E**

That is ~11-14 weeks of disciplined work. Skipping any step makes Sprint D
(the big one) either impossible or wrong.

---

## What's changed (since last update)

1. **H moved up** — Sessions must exist before Cognitive Tree, because
   context-based disambiguation (Idan's "crown" example) is a D requirement.

2. **D expanded significantly** — What was "49 linear passes" is now
   "beam tree 7×7 with bridges, probabilities, conditional analysis."
   See `COGNITIVE_TREE_SPEC.md` for the full reasoning. 4-6 weeks to build.

3. **Sprint briefs added:**
   - `SPRINT_A.md` — 100% ready to execute
   - `SPRINT_B.md` — 100% ready (after A)
   - `SPRINT_H.md` — 100% ready (after B)

4. **Still needed (Idan's decisions):**
   - Approve `COGNITIVE_TREE_SPEC.md` decisions A, B, C
   - Write brief for Sprint C (PreGraphBuilder — need scope)
   - Write brief for Sprint E (Composition — need template strategy)

---

## Workflow (unchanged)

1. Claude Code picks a 🟢 Ready sprint.
2. Reads full brief in `docs/sprints/SPRINT_<ID>.md` or parent spec.
3. Creates branch `sprint-<id>-<name>`.
4. Implements per brief.
5. Tests must pass (never drop test count).
6. Benchmarks on Oracle server (`/home/dinio/zets`), not sandbox.
7. Push, open PR, Idan reviews + merges.

---

## Rules (unchanged)

- One sprint per person at a time
- Never break passing tests
- Benchmarks on Oracle, not Anthropic sandbox
- No new deps without brief approval
- If stuck → write `SPRINT_<ID>_NOTES.md` + stop + ping Idan
- Commits reference sprint ID in message
