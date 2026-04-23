# 🧮 מתמטיקה והיגיון — Math & Reasoning

**Last updated:** 23.04.2026
**Current score:** 🟡 0.47 | **Target MVP:** 0.80 | **Gap:** −0.33

**באחריות:** `graph + binary (Rust native)` + `external (LLM for symbolic)`

## 🎯 משימה
אריתמטיקה, algebra, calculus, logic puzzles, proofs, statistical reasoning, unit conversion.

## 🔬 Tests (7)

| # | Test | סוג | יעד | סטטוס | מודול |
|---|------|:---:|:---:|:-----:|--------|
| 14.1 | Arithmetic 10-digit | QA | 1.00 | ✅ 1.00 | system_graph/vm (Rust native) |
| 14.2 | Algebra (high-school) | QA | 0.90 | 🟡 0.50 | — no symbolic |
| 14.3 | Calculus (intro) | QA | 0.75 | 🟠 0.20 | — no symbolic |
| 14.4 | Logic puzzles (20) | QA | 0.80 | 🟡 0.55 | cognitive_modes::PrecisionMode |
| 14.5 | Simple theorem proofs | QA | 0.65 | 🟠 0.25 | system_graph rules |
| 14.6 | Statistical reasoning | QA | 0.75 | 🟠 0.30 | — |
| 14.7 | Unit conversion | QA | 0.95 | 🟡 0.50 | procedure_template MathSymbol |

## 🏗️ באחריות
- **system_graph/vm** — arithmetic execution (**binary**)
- **cognitive_modes::PrecisionMode** — bounded DFS (**graph**)
- **procedure_template** — math patterns (graph)
- Symbolic math — **חיצוני** (SymPy, Mathematica API, future)

## היסטוריה
| תאריך | Score |
|:-----:|:-----:|
| 23.04 | 0.47 | Baseline. Arithmetic solid, symbolic missing |
