# 🎯 ביצוע משימות — Task Execution & Orchestration

**Last updated:** 23.04.2026
**Current score:** 🟡 0.34 | **Target MVP:** 0.80 | **Gap:** −0.46

**באחריות:** `graph + חיצוני`

## 🎯 משימה
Multi-step plans, long-running tasks with recovery, scheduling, Gmail+Calendar+Sheets workflows, tool-fail fallback, multi-persona delegation, rate-limit + budget awareness.

## 🔬 Tests (8)

| # | Test | סוג | יעד | סטטוס | מודול |
|---|------|:---:|:---:|:-----:|--------|
| 13.1 | 10-step plan execute | TEST | 0.80 | 🟡 0.50 | planner + composition/plan |
| 13.2 | Long task w/ recovery | TEST | 0.85 | 🟠 0.30 | error_store (no retry logic) |
| 13.3 | Schedule meetings | QA | 0.90 | 🔴 0.00 | — no calendar runtime |
| 13.4 | Gmail+Cal+Sheets workflow | QA+TEST | 0.80 | 🟡 0.45 | connectors/seed (9 bundles, no runtime) |
| 13.5 | Tool-fail fallback | QA | 0.85 | 🟠 0.35 | guard_pipeline + error_store |
| 13.6 | Multi-persona delegation | QA | 0.80 | 🟡 0.50 | cognitive_modes + search personas |
| 13.7 | Rate-limit aware | TEST | 0.95 | 🟠 0.20 | — |
| 13.8 | Budget-aware | TEST | 0.95 | 🟡 0.40 | CompositionPlan.external_budget |

## 🏗️ באחריות
- **planner** (src/planner.rs) — planning
- **composition/plan** — executable plans
- **connectors** — 9 bundles defined (no runtime) — see [connectors.md](../inventory/connectors.md)
- **CapabilityOrchestrator** (עתיד) — the missing piece

## היסטוריה
| תאריך | Score |
|:-----:|:-----:|
| 23.04 | 0.34 | Baseline. Planning + bundles exist, execution runtime חסר |
