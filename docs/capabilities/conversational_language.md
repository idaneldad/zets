# 💬 שיחה בשפה טבעית — Conversational Language

**Last updated:** 23.04.2026
**Last tests run:** 23.04.2026 — 30 reader tests passing
**Current score:** 🟡 0.45 | **Target MVP:** 0.90 | **Gap:** −0.45

**באחריות:** `graph + motifs` (graph-native reading) + `external (LLM)` ל-generation עשיר

---

## 🎯 מה המשימה

ZETS צריך לנהל שיחה טבעית עם בני אדם בעברית, אנגלית, ועוד שפות — מבין emotion, intent, style, וזוכר היסטוריה.

## ✅ מה תיחשב הצלחה

- Coherence 0.90+ על 50-turn conversation בעברית ובאנגלית
- Code-switching עברית↔אנגלית בלי שבירה
- Long-history recall — פריט ממסר #3 נזכר במסר #100
- זיהוי tone (formal/casual/angry/joyful) — F1 0.80+
- Humor generation עם human rating >3.5/5

---

## 🔬 Tests (11 מתוכננים)

| # | Test | סוג | יעד | סטטוס 23.04 | מודול |
|---|------|:---:|:---:|:-----------:|--------|
| 1.1 | עברית 50-turn | QA | 0.90 | 🟡 0.45 | reader (Phase 2 partial) |
| 1.2 | אנגלית 50-turn | QA | 0.90 | 🟡 0.45 | reader |
| 1.3 | ערבית/צרפתית/רוסית | QA | 0.75 | 🟠 0.20 | morphology (ערבית בלבד) |
| 1.4 | HE↔EN code-switching | QA | 0.95 | 🟡 0.60 | sense_graph |
| 1.5 | Intent classification (5 classes) | QA | 0.85 | 🟡 0.50 | reader/intent.rs (new) |
| 1.6 | Coreference (100 cases) | QA | 0.80 | 🔴 0.00 | — |
| 1.7 | Tone recognition | QA | 0.80 | 🟠 0.25 | reader/emotion.rs (new) |
| 1.8 | Long-history recall@100 | TEST | 0.85 | 🟢 0.75 | conversation |
| 1.9 | 3-participant no-misattrib | QA | 0.95 | 🟢 0.80 | conversation (per-source) |
| 1.10 | Sarcasm detection | QA | 0.70 | 🔴 0.00 | — |
| 1.11 | Humor generation | QA | 0.70 | 🟠 0.15 | composition/motif |

## 🏗️ באחריות

- **Reader** (`src/reader/`) — emotion, intent, style inference — **graph**
- **Conversation** (`src/conversation/`) — session + history — **graph**
- **Composition** (`src/composition/`) — response generation — **graph + motifs**
- **LLM adapter** (`src/llm_adapter.rs`) — rich prose generation — **external (Gemini Flash)**

## 📈 פער ליעד

**Top gap:** Reader Phase 2 עדיין לא wire (emotion/intent/style stubs just implemented in this session, not yet committed). כשיעלה — Cat 1 קופץ ל-0.65+.

## היסטוריית שינויים

| תאריך | Score | שינוי |
|:-----:|:-----:|-------|
| 23.04 | 0.45 | Baseline. Reader skeleton + ConversationStore. Phase 2 emotion/intent/style בtransit |
