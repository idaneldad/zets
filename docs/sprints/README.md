# ZETS Sprint Tracker

**Last updated:** 21.04.2026 (Empathic Response layer added)

## Three parallel tracks

**Track 1 — Cognitive Engine** (the brain)
A → B → C → H → **J → K → D** → E → **L** → F → G → I

**Track 2 — Ingestion Pipeline** (the mouth)
ING1 → ING2 → ING3

**Track 3 — Nervous System** (the hands, deferred)
G → I (after all Track 1 done)

---

## Status board

### Track 1 (Cognitive Engine) — CRITICAL PATH
| ID | Name | Status | Brief | Notes |
|----|------|--------|-------|-------|
| A | CLI + iterator APIs | 🟢 Ready | [SPRINT_A.md](SPRINT_A.md) | First sprint |
| B | Query Planner + Multi-seed | 🟡 Blocked on A | [SPRINT_B.md](SPRINT_B.md) | |
| C | PreGraphBuilder + language data | 🟡 Needs brief | – | Multi-language |
| H | Sessions + Context Disambig | 🟡 Blocked on B | [SPRINT_H.md](SPRINT_H.md) | |
| **J** | **User Model** | 🔴 Needs design approval | [EMPATHIC_RESPONSE_SPEC.md](../EMPATHIC_RESPONSE_SPEC.md) | NEW |
| **K** | **Intent Decoder** | 🔴 Needs design approval | [EMPATHIC_RESPONSE_SPEC.md](../EMPATHIC_RESPONSE_SPEC.md) | NEW |
| **D** | **Cognitive Tree (beam 7×7)** | 🔴 Needs design approval | [COGNITIVE_TREE_SPEC.md](../COGNITIVE_TREE_SPEC.md) | The Big One |
| E | Composition Engine | 🟡 Blocked on D | – | |
| **L** | **Response Crafter** | 🔴 Needs design approval | [EMPATHIC_RESPONSE_SPEC.md](../EMPATHIC_RESPONSE_SPEC.md) | NEW |
| F | Feedback Learner | 🟡 Blocked on L | – | |

### Track 2 (Ingestion — parallel after A)
| ID | Name | Status | Brief |
|------|------|--------|-------|
| ING1 | Text + chunks + BLAKE3 | 🟢 Ready (after A) | [UNIVERSAL_INGESTION_ARCHITECTURE.md](../UNIVERSAL_INGESTION_ARCHITECTURE.md) |
| ING2 | Docx + PDF | 🟡 Blocked on ING1 | – |
| ING3 | Edge reader + sync | 🟡 Blocked on ING2 | – |

### Track 3 (Nervous System — deferred)
| ID | Name | Status | Brief |
|----|------|--------|-------|
| G | Tool Registry | 🟡 Blocked on F | [OPENCLAW_INTEGRATION.md](../OPENCLAW_INTEGRATION.md) |
| I | Cloud Relay | 🟡 Blocked on G | – |

---

## Critical path for AGI-like ZETS

`A → B → C → H → J → K → D → E → L → F`

Total estimated time: **16-20 weeks** disciplined work.
Every sprint in this path is essential for the product experience
Idan described.

---

## Why J, K, L were added

After Idan described his 14-step "sales consultant methodology" — understanding
the user, decoding intent, adapting style, 40/40/20 response partitioning —
it became clear that the original roadmap was missing an entire product layer.

Without J+K+L, ZETS answers questions. With them, ZETS becomes the "personal
consultant that understands who you are" that Idan described as the moat.

**J = User Model:** who's asking (profile, patterns, red lines, history)
**K = Intent Decoder:** what they really want (pain, hidden wants, signals)
**L = Response Crafter:** how to deliver (register, length, pacing, style)

D (Cognitive Tree) sits between K and L because:
- K feeds D the user-weighted seed selection
- D produces ranked candidates
- L renders them in style matching the user

---

## Workflow (unchanged)

1. Claude Code picks 🟢 Ready sprint
2. Reads full brief
3. Creates branch `sprint-<id>-<n>`
4. Implements per brief
5. Tests must pass (never drop test count)
6. Benchmarks on Oracle server `/home/dinio/zets`, not sandbox
7. Push, open PR, Idan merges

---

## Rules

- One sprint per person at a time
- Never break passing tests (currently 72)
- No new deps without brief approval
- If stuck → write `SPRINT_<ID>_NOTES.md` + stop + ping Idan
- Commits reference sprint ID

---

## Pending decisions (blocking sprints)

**For Sprint D** (COGNITIVE_TREE_SPEC section 10):
- A1/A2/A3 tree expansion
- B1/B2/B3 probability sources
- C1/C2/C3 disambiguation

**For Sprints J+K+L** (EMPATHIC_RESPONSE_SPEC section 10):
- X: User Model depth (X1/X2/X3)
- Y: Cognitive tag source (Y1/Y2/Y3)
- Z: Empty-profile default (Z1/Z2/Z3)
- AA: Pacing ethics (AA1/AA2/AA3)
- BB: 40/40/20 rigidity (BB1/BB2)

Until answered, these three sprints cannot be written as executable briefs.

---

## What's immediately actionable

Even without the pending decisions, **Sprints A, B, H, ING1 are fully
specified and ready to execute**. That's 4 weeks of solid work already
in the backlog.

Idan can:
- Approve Sprint A now → Claude Code starts
- OR answer pending decisions first → unlock D, J, K, L briefs
- OR both in parallel
