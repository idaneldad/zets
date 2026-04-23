# 💻 תכנות — Programming

**Last updated:** 23.04.2026
**Current score:** 🟡 0.35 | **Target MVP:** 0.70 | **Gap:** −0.35

**באחריות:** `graph` (procedure_template) + `external (LLM)` ל-code generation

## 🎯 משימה

ZETS צריך להבין, לייצר, לתקן, ולתרגם קוד ב-Python, Rust, JavaScript, SQL ועוד. חיוני לPhase C (ingest של כלים חיצוניים).

## ✅ הצלחה

- Pass@1 0.70+ על Python HumanEval-level tasks
- Pass@1 0.55+ על Rust equivalents
- Cross-language translation (Python → Rust) 0.75+ correctness
- Code review — 0.70 recall על 5 bugs ב-200 שורות

## 🔬 Tests (8)

| # | Test | סוג | יעד | סטטוס | מודול |
|---|------|:---:|:---:|:-----:|--------|
| 2.1 | Python func from spec (50 tasks) | QA | 0.70 | 🟡 0.45 | llm_adapter + procedure_template |
| 2.2 | Rust func from spec | QA | 0.55 | 🟡 0.40 | procedure_template |
| 2.3 | JavaScript func from spec | QA | 0.70 | 🟠 0.30 | procedure_template |
| 2.4 | SQL generation | QA | 0.80 | 🟡 0.50 | system_graph |
| 2.5 | Debug 50-line code | QA | 0.65 | 🟠 0.20 | — |
| 2.6 | Refactor to spec | QA | 0.70 | 🟠 0.25 | procedure_template.shape_hash |
| 2.7 | Cross-language translation | QA | 0.75 | 🟡 0.55 | **procedure_template.binding** |
| 2.8 | Code review (5 issues/200 lines) | QA | 0.70 | 🟠 0.20 | guard (partial) |

## 🏗️ באחריות

- **ProcedureTemplate** (`src/procedure_template/`) — קטלוג תבניות + binding בין שפות. **graph**
- **llm_adapter** (`src/llm_adapter.rs`) — parsing NL → structured. **external (Gemini Flash)**
- **system_graph/vm** (`src/system_graph/vm.rs`) — execution engine. **binary**

## 📈 פער

**Top gap:** אין codegen pipeline. template → actual code דורש LLM call layer שעדיין לא נבנה.

## היסטוריה
| תאריך | Score |
|:-----:|:-----:|
| 23.04 | 0.35 | Baseline. ProcedureTemplate + llm_adapter exist, no codegen flow |
