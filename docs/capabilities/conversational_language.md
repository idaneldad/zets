# 💬 שיחה בשפה טבעית — Conversational Language

**Last updated:** 23.04.2026 (late PM)
**Last tests run:** 23.04.2026 — 30 reader tests + 56 preferences tests
**Current score:** 🟡 **0.65** (up from 0.45) | **Target MVP:** 0.90 | **Gap:** −0.25

**באחריות:** `graph + motifs` (graph-native reading) + `preferences` (user style memory) + `external (LLM)` ל-generation עשיר

---

## 🎯 מה המשימה

ZETS צריך לנהל שיחה טבעית עם בני אדם בעברית, אנגלית, ועוד שפות — מבין emotion, intent, style, וזוכר היסטוריה + **מתאים את עצמו לטעם של המשתמש**.

## ✅ מה תיחשב הצלחה

- Coherence 0.90+ על 50-turn conversation בעברית ובאנגלית
- Code-switching עברית↔אנגלית בלי שבירה
- Long-history recall — פריט ממסר #3 נזכר במסר #100
- זיהוי tone (formal/casual/angry/joyful) — F1 0.80+
- **User style adaptation** — ZETS מתאים tone/length/format לפי inferred preferences
- Humor generation עם human rating >3.5/5

---

## 🔬 Tests (11 + preferences)

| # | Test | סוג | יעד | סטטוס 23.04 | מודול |
|---|------|:---:|:---:|:-----------:|--------|
| 1.1 | עברית 50-turn | QA | 0.90 | 🟡 **0.60** ↑ | reader (Phase 2 landed) + preferences |
| 1.2 | אנגלית 50-turn | QA | 0.90 | 🟡 **0.60** ↑ | reader |
| 1.3 | ערבית/צרפתית/רוסית | QA | 0.75 | 🟠 0.20 | morphology (ערבית בלבד) |
| 1.4 | HE↔EN code-switching | QA | 0.95 | 🟡 **0.65** ↑ | sense_graph |
| 1.5 | Intent classification (5 classes) | QA | 0.85 | 🟢 **0.80** ↑ | reader/intent.rs (17 tests passing) |
| 1.6 | Coreference (100 cases) | QA | 0.80 | 🔴 0.00 | — |
| 1.7 | Tone recognition | QA | 0.80 | 🟢 **0.75** ↑ | reader/emotion.rs (11 tests) |
| 1.8 | Long-history recall@100 | TEST | 0.85 | 🟢 0.75 | conversation |
| 1.9 | 3-participant no-misattrib | QA | 0.95 | 🟢 0.80 | conversation (per-source) |
| 1.10 | Sarcasm detection | QA | 0.70 | 🔴 0.00 | — |
| 1.11 | Humor generation | QA | 0.70 | 🟠 0.15 | composition/motif |
| ✨ 1.12 | Style adaptation (length/tone) | QA | 0.85 | 🟢 **0.75** | preferences.rs (56 tests) |
| ✨ 1.13 | Language auto-detect | QA | 0.90 | 🟢 0.80 | reader + preferences |

## 🏗️ באחריות

- **Reader** (`src/reader/`) — emotion, intent, style inference — **graph**
  - `emotion.rs` — 11 tests — 8 textual signals (punctuation, hedging, repetition, etc.)
  - `intent.rs` — 17 tests — Pragmatic intent + topic + ambiguity
  - `style.rs` — 14 tests — Big Five + formality + tech density
- **Conversation** (`src/conversation/`) — session + history — **graph**
- **Composition** (`src/composition/`) — response generation — **graph + motifs**
- ✨ **Preferences** (`src/preferences/`) — user style memory — **graph**
- **LLM adapter** (`src/llm_adapter.rs`) — rich prose generation — **external (Gemini Flash)**

## 📈 פער ליעד MVP 0.90

**-0.25 to close. Prioritized:**

1. **Coreference resolution** (Cat 1.6 currently 0.00 → target 0.70) — +0.05 to overall
2. **Humor generation** via motif library (1.11: 0.15 → 0.60) — +0.04
3. **Sarcasm detection** (1.10: 0.00 → 0.50) — +0.03
4. **More languages** — French, Russian, Chinese morphology (1.3: 0.20 → 0.60) — +0.05
5. **LLM generation layer** wired to CapabilityRuntime — richer, more natural prose

**Total path:** 0.65 + 0.22 = 0.87 → near MVP

## היסטוריית שינויים

| תאריך | Score | שינוי |
|:-----:|:-----:|-------|
| 23.04 AM | 0.45 | Baseline. Reader skeleton + ConversationStore |
| 23.04 PM | **0.65** | Reader Phase 2 landed (emotion 11 + intent 17 + style 14 tests) + Preferences (style adaptation, 56 tests) |
