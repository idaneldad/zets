# 🔬 ניתוח ומחקר — Analysis & Research

**Last updated:** 23.04.2026
**Current score:** 🟠 0.19 | **Target MVP:** 0.70 | **Gap:** −0.51

**באחריות:** `graph + חיצוני`

## 🎯 משימה
Multi-paper synthesis, financial analysis, legal risk ID, medical lit summary (עם safety), market research, scientific verification, data insights, SWOT.

## 🔬 Tests (8)

| # | Test | סוג | יעד | סטטוס | מודול |
|---|------|:---:|:---:|:-----:|--------|
| 12.1 | 10-paper compare/contrast | QA | 0.80 | 🟡 0.40 | morphology + search |
| 12.2 | Financial statement analysis | QA | 0.85 | 🔴 0.00 | — |
| 12.3 | Legal doc risk | QA | 0.75 | 🔴 0.00 | — |
| 12.4 | Medical literature + safety | QA | 0.80 | 🔴 0.00 | — |
| 12.5 | Market research synthesis | QA | 0.75 | 🟠 0.25 | search |
| 12.6 | Scientific claim verification | QA | 0.80 | 🟡 0.40 | verify (20 tests) |
| 12.7 | Data table insight | QA | 0.75 | 🟠 0.25 | system_graph |
| 12.8 | SWOT analysis | QA | 0.80 | 🟠 0.25 | motifs ArgumentPattern |

## 🏗️ באחריות
- **verify** (src/verify.rs) — proof checking
- **morphology** — text analysis (36 tests)
- **search** (src/search/) — 27 tests
- **system_graph** — structured reasoning
- Domain-specific parsers — external

## היסטוריה
| תאריך | Score |
|:-----:|:-----:|
| 23.04 | 0.19 | Baseline. verify+morphology+search exist, no domain parsers |
