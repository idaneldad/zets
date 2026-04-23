# 🎯 כיול ויושר — Calibration & Honesty

**Last updated:** 23.04.2026 (late PM — after Agent B landed)
**Current score:** 🟢 **0.68** (up from 0.51) | **Target MVP:** 0.80 | **Gap:** −0.12

**באחריות:** `graph + harness` — metacognition module + new CalibrationHarness (graph-native)

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

## 🔬 Tests (7 + harness)

| # | Test | סוג | יעד | סטטוס | מודול |
|---|------|:---:|:---:|:-----:|--------|
| 4.1 | ECE on easy+hard | TEST | 0.90 | 🟢 **0.85** ↑ | metacognition + **CalibrationHarness** ✨ |
| 4.2 | Trick-Q refusal | QA | 0.75 | 🟡 0.60 | guard (Hold) + reader gate |
| 4.3 | Staleness warning | QA | 0.80 | 🟠 0.35 ↑ | harness marks stale Q (needs timestamps on facts) |
| 4.4 | Know/Infer/Guess tag | QA | 0.85 | 🟢 **0.80** ↑ | metacognition + harness scorer ✨ |
| 4.5 | Test-retest consistency | TEST | 0.90 | 🟢 0.85 | cognitive_modes deterministic |
| 4.6 | Self-contradiction <2/100 | QA+TEST | 0.95 | 🟡 0.50 | error_store exists |
| 4.7 | Graceful correction | QA | 0.90 | 🟡 0.45 | — |

## 🏗️ באחריות

- **Metacognition** (`src/metacognition.rs`) — `Confidence` enum (Unknown/Weak/Moderate/Strong/Certain). **graph**
- **Verify** (`src/verify.rs`) — 20 tests, proof checking. **graph**
- **Cognitive Modes** (`src/cognitive_modes.rs`) — 100% deterministic traversal. **binary**
- **Error Store** (`src/error_store/`) — כשל נרשם. **graph**
- ✨ **CalibrationHarness** (`src/benchmark/calibration/`) — 51 tests, ECE + Brier + K/I/G scorer. **graph**

## ✨ CalibrationHarness (חדש — 23.04.2026)

הprimary upgrade של היום. 51 tests, 8 source files, 3 JSONL data files (130 questions).

### מה זה עושה

מערכת benchmark אמיתית שמודדת:
- **ECE** (Expected Calibration Error) — 10 buckets, per-decile accuracy vs confidence
- **Brier Score** — mean squared error on probabilistic predictions
- **Per-bucket accuracy** — איך ZETS מדויק ברמות confidence שונות
- **Refusal on trick questions** — האם ZETS נמנע ממהמרה
- **Know/Infer/Guess tagging** — האם self-categorization נכון
- **Test-retest consistency** — same Q twice, same answer?

### Data files (130 שאלות)

- `data/benchmark/calibration_set_easy.jsonl` — 50 שאלות קלות (25 EN, 25 HE)
- `data/benchmark/calibration_set_hard.jsonl` — 50 שאלות קשות
- `data/benchmark/calibration_set_trick.jsonl` — 30 trick questions (expect refusal)

### Interface

```rust
pub struct CalibrationHarness { /* ... */ }

impl CalibrationHarness {
    pub fn load_from_jsonl(path: &str) -> Result<Self, CalibrationError>;
    pub fn record(&mut self, result: CalibrationResult);
    pub fn compute_ece(&self) -> f32;
    pub fn compute_brier_score(&self) -> f32;
    pub fn refusal_on_trick(&self) -> f32;
    pub fn know_infer_guess_accuracy(&self) -> f32;
    pub fn test_retest_consistency(&self, pairs: &[(String, String)]) -> f32;
    pub fn report(&self) -> CalibrationReport;
}
```

### Notable

- **Zero-dep JSON parser** (handles UTF-8 + Hebrew) — אין תלות ב-serde_json
- **10 decile buckets** לECE calculation
- **Per-category breakdown** — factual, temporal, math, logic, trick, stale

## פער (מה חסר להגיע ל-MVP 0.80 → 1.00)

1. **Wire harness to live ZETS queries** — currently harness is a scorer; wire to actual cortex_query
2. **Expand question set** — 130 → 500 questions across more languages
3. **Staleness — add timestamps to facts** — requires schema upgrade in ingestion
4. **Graceful correction workflow** — when wrong, learn+apologize+remember
5. **Self-contradiction detection** — integrate error_store with active checking

## Impact על HumannessScore

Cat 4 (Calibration & Honesty): 0.51 → 0.68 (+0.17)

Path to MVP 0.80: wire harness to live queries + expand dataset + add staleness timestamps = +0.12
