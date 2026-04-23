# 🎯 כיול ויושר — Calibration & Honesty

**Last updated:** 23.04.2026
**Current score:** 🟡 0.51 | **Target MVP:** 0.80 | **Gap:** −0.29

**באחריות:** `graph` — מבוסס metacognition module (graph-native)

## 🎯 משימה

ZETS חייב לדעת **מה הוא לא יודע**. זה ההבדל בין מערכת אמינה למערכת שמהמרצה confidence על שטויות.

## ✅ הצלחה

- ECE < 0.10 (Expected Calibration Error)
- Trick-question refusal 75%+
- Staleness warning על info ישן
- Know/Infer/Guess tagging 85%+ accuracy
- Test-retest consistency 90%+ (deterministic)
- Self-contradiction <2 per 100-turn conversation
- Graceful correction when wrong

## 🔬 Tests (7)

| # | Test | סוג | יעד | סטטוס | מודול |
|---|------|:---:|:---:|:-----:|--------|
| 4.1 | ECE on easy+hard | TEST | 0.90 | 🟡 0.50 | metacognition + verify |
| 4.2 | Trick-Q refusal | QA | 0.75 | 🟡 0.50 | guard (Hold) + reader gate |
| 4.3 | Staleness warning | QA | 0.80 | 🟠 0.25 | — no timestamps on facts |
| 4.4 | Know/Infer/Guess tag | QA | 0.85 | 🟡 0.55 | metacognition (5 levels) |
| 4.5 | Test-retest consistency | TEST | 0.90 | 🟢 0.85 | cognitive_modes deterministic |
| 4.6 | Self-contradiction <2/100 | QA+TEST | 0.95 | 🟡 0.50 | error_store exists |
| 4.7 | Graceful correction | QA | 0.90 | 🟡 0.40 | — |

## 🏗️ באחריות

- **Metacognition** (`src/metacognition.rs`) — `Confidence` enum (Unknown/Weak/Moderate/Strong/Certain). **graph**
- **Verify** (`src/verify.rs`) — 20 tests, proof checking. **graph**
- **Cognitive Modes** (`src/cognitive_modes.rs`) — 100% deterministic traversal. **binary**
- **Error Store** (`src/error_store/`) — כשל נרשם. **graph**

## 📈 פער

- Benchmark harness לECE חסר
- Timestamp על כל fact חסר (לsaleness)
- Contradiction detector לא מיושם

## היסטוריה
| תאריך | Score |
|:-----:|:-----:|
| 23.04 | 0.51 | Baseline. Infrastructure exists (Confidence, verify, cognitive_modes), harness missing |
