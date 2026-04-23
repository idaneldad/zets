# 📚 תוכן ארוך — Long-form Content

**Last updated:** 23.04.2026
**Current score:** 🟡 0.42 | **Target MVP:** 0.75 | **Gap:** −0.33

**באחריות:** `graph + motifs` ליצירה, `external (LLM)` לריאליזציה עשירה

## 🎯 משימה
סיפורים (1000 מילים עם arc), מאמרים אקדמיים, specs, summaries, persuasive essays, video scripts, keynote speeches.

## 🔬 Tests (7)

| # | Test | סוג | יעד | סטטוס | מודול |
|---|------|:---:|:---:|:-----:|--------|
| 11.1 | 1000-word story w/ arc | QA | 0.80 | 🟡 0.55 | composition/weaver + motifs |
| 11.2 | 3000-word academic article | QA | 0.90 | 🟡 0.45 | — (no citation mgmt) |
| 11.3 | Product spec | QA | 0.85 | 🟡 0.50 | procedure_template |
| 11.4 | 50-page → summary | QA+TEST | 0.85 | 🟡 0.50 | path_mining motifs |
| 11.5 | Persuasive essay | QA | 0.70 | 🟠 0.35 | MotifKind::ArgumentPattern |
| 11.6 | 10-min video script | QA | 0.76 | 🟠 0.30 | composition iter |
| 11.7 | Keynote speech | QA | 0.70 | 🟠 0.30 | composition |

## 🏗️ באחריות
- **Composition** (src/composition/) — motifs + Weaver. **graph**
- **path_mining** (src/path_mining.rs) — repeating patterns extraction. **graph**
- **LLM adapter** — rich prose surface. **external (optional)**

## היסטוריה
| תאריך | Score |
|:-----:|:-----:|
| 23.04 | 0.42 | Baseline. composition layer עושה הרבה |
