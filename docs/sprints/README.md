# ZETS Sprint Tracker

Each sprint is a self-contained unit of work. Claude Code (or Idan)
picks one, implements it fully, ships, then picks the next.

## Status board

| ID  | Name | Status | Owner | Branch | PR |
|-----|------|--------|-------|--------|-----|
| A   | CLI cleanup + iterator APIs       | 🟢 Ready | unassigned | sprint-a-cli | – |
| B   | Query Planner + Multi-seed walks  | 🟡 Blocked on A | – | – | – |
| C   | PreGraphBuilder refactor          | 🟡 Blocked on A | – | – | – |
| D   | Deliberation Engine (49 passes)   | 🟡 Blocked on B | – | – | – |
| E   | Composition Engine v1             | 🟡 Blocked on D | – | – | – |
| F   | Feedback Learner                  | 🟡 Blocked on E | – | – | – |
| G   | Tool Registry + Permissions       | 🟡 Blocked on F | – | – | – |
| H   | Sessions + Background Scheduler   | 🟡 Blocked on G | – | – | – |
| I   | Cloud Relay reference impl        | 🟡 Blocked on H | – | – | – |
| ING1| Ingestion Sprint 1: text + chunks | 🟢 Ready (parallel)| unassigned | sprint-ing1 | – |
| ING2| Ingestion Sprint 2: docx/pdf      | 🟡 Blocked on ING1| – | – | – |
| ING3| Ingestion Sprint 3: edge reader   | 🟡 Blocked on ING2| – | – | – |

## Workflow

1. Claude Code (or human) picks a "🟢 Ready" sprint.
2. Reads `docs/sprints/SPRINT_<ID>.md` for the full task brief.
3. Creates branch `sprint-<id>-<short-name>` from main.
4. Implements per the brief.
5. Ensures all existing tests still pass + new tests added.
6. Commits with reference to sprint ID.
7. Pushes branch.
8. Idan reviews, merges, marks status complete.

## Rules

- **One sprint at a time per assignee.** No parallel sprints by same person.
- **Never break existing tests.** 72 currently pass; never let count drop.
- **Benchmarks on Oracle server, not Anthropic sandbox.** Required for any perf claim.
- **Commit messages reference sprint ID.** e.g. "Sprint A: CLI iterator refactor"
- **No new dependencies without sprint brief approval.**
- **If stuck, write findings to `docs/sprints/SPRINT_<ID>_NOTES.md` and stop.**
  Do not freelance.

## Two parallel tracks

ZETS development runs on TWO parallel tracks:

**Track 1 — Cognitive Engine (A → F → G,H,I)**
The brain + nervous system: query, deliberate, compose, learn, act.

**Track 2 — Ingestion Pipeline (ING1 → ING2 → ING3)**
The mouth: how content enters the graph.

Track 2 can run in parallel with Track 1 once Sprint A ships (Sprint A
fixes core APIs that both tracks use).
