# 🧠 זיכרון — Memory & Personal Knowledge

**Last updated:** 23.04.2026
**Current score:** 🟢 0.72 | **Target MVP:** 0.95 | **Gap:** −0.23

**באחריות:** `graph` (native)

## 🎯 משימה

ZETS זוכר אנשים, קשרים, העדפות, היסטוריות שיחות. לא מבוסס על LLM context window — על graph persistent.

## ✅ הצלחה

- 10 facts recalled @24h → 0.95 recall
- 50 facts @week → 0.85 recall
- User A vs User B — 1.00 separation (no contamination)
- Fact updates (job change) — consistent after
- Archived relationships — historical queries still work

## 🔬 Tests (9)

| # | Test | סוג | יעד | סטטוס | מודול |
|---|------|:---:|:---:|:-----:|--------|
| 3.1 | 10 facts @24h | QA | 0.95 | 🟢 0.85 | personal_graph + conversation |
| 3.2 | 50 facts @week | QA+TEST | 0.85 | 🟢 0.80 | כנ"ל |
| 3.3 | User A vs B separation | QA | 1.00 | ✅ 0.95 | scope_ref + per-source store |
| 3.4 | Contradiction detection | QA | 0.85 | 🟡 0.40 | metacognition (partial) |
| 3.5 | Fact update consistency | QA | 0.90 | 🟢 0.85 | relationship.end() + lifecycle |
| 3.6 | Archived relationship queries | QA | 1.00 | ✅ 0.95 | `was_active_at(ts)` |
| 3.7 | 5 clients separate | QA | 0.95 | ✅ 0.90 | Source enum + scope |
| 3.8 | Preference enforcement | QA | 0.85 | 🟠 0.25 | — no preference store |
| 3.9 | "אתמול דיברנו" summary | QA | 0.80 | 🟡 0.55 | sessions_for() + summarizer needed |

## 🏗️ באחריות

- **PersonalGraph** (`src/personal_graph/`) — identities + time-aware relationships. **graph**
- **Conversation** (`src/conversation/`) — sessions + persistent log. **graph**
- **Secrets Vault** (`src/secrets/vault.rs`) — encrypted file (not graph). **binary + file**

## 📈 פער

- Preference store חסר
- Contradiction detector חסר
- Conversation summarizer חסר

## היסטוריה
| תאריך | Score |
|:-----:|:-----:|
| 23.04 | 0.72 | Baseline. הכי חזק מ-Tier S. PersonalGraph+Conversation בוגרים |
