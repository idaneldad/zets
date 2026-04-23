# 🏗️ תכנית עבודה מקבילה — Parallel Development Plan

**Last updated:** 23.04.2026
**HumannessScore נוכחי:** 0.39
**יעד MVP:** 0.60 (פער: −0.21)

---

## 🎯 הרעיון

במקום Claude יחיד שעובד טורית, **3-4 Claude Code instances במקביל** עובדים על branches נפרדים, כל אחד על משימה independent.

**מה זה לא יהיה:**
- ❌ 3 agents על אותם קבצים — merge hell
- ❌ 3 agents אוטונומיים לגמרי — drift + contradictions
- ❌ עבודה בלי coordinator — אתה (עידן) coordinator-in-chief

**מה זה כן יהיה:**
- ✅ Branch לכל agent
- ✅ משימה clearly-defined עם done criteria
- ✅ Interface contract ברור בין modules
- ✅ Daily merge לmain
- ✅ אתה בודק ו-approve כל merge

---

## 📋 איזה משימות מתאימות לparallelization?

### ✅ מתאים — independent modules

| משימה | מודול חדש? | תלויות | מתאים? |
|--------|:---------:|--------|:------:|
| CapabilityOrchestrator skeleton | ✅ `src/capability_runtime/` | Vault (קיים) | ✅ כן |
| Whisper integration | ✅ `src/capabilities/speech/` | CapabilityOrchestrator | ⏸ אחרי הראשון |
| Gemini Vision integration | ✅ `src/capabilities/vision/` | CapabilityOrchestrator | ⏸ אחרי הראשון |
| Calibration harness | ✅ `src/benchmark/calibration/` | metacognition (קיים) | ✅ כן |
| Preference store | ✅ `src/preferences/` | personal_graph (קיים) | ✅ כן |
| Contradiction detector | ✅ `src/contradiction/` | personal_graph + conversation | ✅ כן |
| MCP client | ✅ `src/mcp_client/` | — | ✅ כן |
| Texture library ל-image composition | 🟡 הרחבת motif_bank | composition (קיים) | 🟡 זהירות |

### ❌ לא מתאים — conflicts בלתי נמנעים

| משימה | למה לא |
|--------|---------|
| שינויים ב-lib.rs | כל agent ייגע בו |
| Refactoring של Reader | משפיע על כל ה-tests |
| Cargo.toml updates | conflicts על dependencies |
| Breaking API changes | tests יישברו everywhere |

---

## 🌳 מבנה Branches

```
main (את)
├── feat/capability-runtime      ← Agent A
├── feat/calibration-harness     ← Agent B
├── feat/preference-store        ← Agent C
├── feat/mcp-client              ← Agent D (אם יש 4)
└── feat/contradiction-detector  ← (serial, אחרי C)
```

**Rules:**
1. כל agent עובד רק על **הbranch שלו**
2. כל agent יוצר **רק מודול חדש** + tests — לא נוגע בקוד קיים
3. שינוי ב-lib.rs נעשה ב-merge commit ע"י אתה
4. Merge חוזר למה main **יום ב-יום**
5. **Pre-merge check:** `cargo test --lib` חייב לעבור

---

## 🎯 3 המשימות לסשן הזה

ממליץ להתחיל עם 3 — לא 4. יותר = יותר בלבול.

### 🅰️ Agent A — CapabilityOrchestrator

**Branch:** `feat/capability-runtime`
**קובץ ראשי:** `src/capability_runtime/`
**משימה:** בונה את ה-executor לכל הcapabilities החיצוניות

**Mission file (עידן מעתיק למסך של Agent A):**
```
You are working on branch feat/capability-runtime.

Goal: Create src/capability_runtime/ module that orchestrates external
      capability invocations (Whisper, Gemini Vision, Midjourney, etc).

Interface contract (MUST conform):
  pub struct CapabilityOrchestrator { ... }
  
  pub async fn invoke(
      &self,
      capability_id: &str,
      args: serde_json::Value,
      caller: &IdentityId,
  ) -> Result<CapabilityResult, CapabilityError>;

Files to create:
  src/capability_runtime/mod.rs
  src/capability_runtime/orchestrator.rs
  src/capability_runtime/registry.rs
  src/capability_runtime/executor.rs
  src/capability_runtime/budget.rs

Features:
  - Async execution via tokio
  - Secrets via vault (src/secrets/)
  - Budget tracking per caller
  - Retry on transient failures (exponential backoff)
  - Rate limit respect (429 → wait)
  - Timeout enforcement
  - Audit log entries via error_store

Do NOT:
  - Modify src/lib.rs (I'll do the registration)
  - Change any existing file
  - Depend on modules not yet built (no actual Whisper calls yet,
    just the framework)

Tests required:
  - 15+ unit tests
  - Mock HTTP server for integration tests
  - Budget exhaustion test
  - Timeout test
  - Retry test

When done:
  git push origin feat/capability-runtime
  Create PR to main
  Tag Idan for merge approval
```

---

### 🅱️ Agent B — Calibration Harness

**Branch:** `feat/calibration-harness`
**קובץ ראשי:** `src/benchmark/calibration/`
**משימה:** בנצ'מרק אמיתי של ECE + consistency + Know/Infer/Guess tagging

**Mission file:**
```
You are working on branch feat/calibration-harness.

Goal: Create calibration measurement harness for benchmark Cat 4.
      Currently score is 0.51. Target 0.80.

Interface contract:
  pub struct CalibrationHarness {
      questions: Vec<CalibrationQuestion>,
      results: Vec<CalibrationResult>,
  }
  
  pub fn compute_ece(&self) -> f32;
  pub fn compute_brier_score(&self) -> f32;
  pub fn accuracy_per_bucket(&self) -> Vec<BucketStats>;

Files to create:
  src/benchmark/calibration/mod.rs
  src/benchmark/calibration/harness.rs
  src/benchmark/calibration/question.rs
  src/benchmark/calibration/metrics.rs
  data/benchmark/calibration_set_easy.jsonl  (50 easy Q)
  data/benchmark/calibration_set_hard.jsonl  (50 hard Q)
  data/benchmark/calibration_set_trick.jsonl (30 trick Q)

Features:
  - ECE (Expected Calibration Error) computation, 10 buckets
  - Brier score
  - Per-bucket accuracy
  - Test-retest consistency (same Q x2, verify same answer)
  - Trick question detection (ZETS should refuse)
  - Staleness detection (Q with outdated info)
  - Know/Infer/Guess tagging accuracy measurement

Do NOT:
  - Modify src/lib.rs
  - Change metacognition.rs (just USE its Confidence enum)
  - Create actual ZETS-query logic (harness runs against stub initially)

Tests required:
  - 20+ unit tests
  - ECE computation test on synthetic data
  - Brier score test
  - Consistency check test

Data files: Q/A + ground truth + difficulty class.
Hebrew + English mix.

When done:
  git push origin feat/calibration-harness
  PR to main
```

---

### 🅲️ Agent C — Preference Store

**Branch:** `feat/preference-store`
**קובץ ראשי:** `src/preferences/`
**משימה:** זיכרון עיקבי של העדפות משתמש (tone, length, format, topics)

**Mission file:**
```
You are working on branch feat/preference-store.

Goal: Track user preferences consistently. Currently Cat 3.8 score 0.25.
      Target 0.85.

Interface contract:
  pub struct PreferenceStore { ... }
  
  pub fn get_preference(&self, owner: &IdentityId, key: &str) -> Option<PreferenceValue>;
  pub fn set_preference(&mut self, owner: &IdentityId, key: &str, value: PreferenceValue);
  pub fn infer_from_conversation(&mut self, owner: &IdentityId, messages: &[HistoryEntry]);

Files to create:
  src/preferences/mod.rs
  src/preferences/store.rs
  src/preferences/key.rs
  src/preferences/inference.rs

Features:
  - Per-owner storage (uses IdentityId from personal_graph)
  - Standard keys: tone, length, format, language, detail_level
  - Custom keys (user-defined)
  - Inference: extract preferences from conversation automatically
  - Override support: explicit > inferred
  - History: who set when
  - Visibility integration with personal_graph::Visibility

Do NOT:
  - Modify personal_graph.rs (USE it, don't change)
  - Modify conversation/ (read from it, don't change)
  - Change src/lib.rs

Tests required:
  - 15+ unit tests
  - Inference from conversation test
  - Conflict resolution (explicit overrides inferred)
  - Per-owner isolation test
  - Visibility/ACL test

When done:
  git push origin feat/preference-store
  PR to main
```

---

## 🕐 Timeline

### יום 1 (היום)
- אתה פותח 3 Claude Code sessions
- מכניס לכל אחד את ה-mission file
- כל אחד עובד 2-4 שעות על branch שלו
- בסוף היום: 3 PRs ל-main

### יום 2 (מחר)
- אתה בודק כל PR (10 דקות לכל אחד)
- מריץ `cargo test --lib` על כל branch
- אם עובר — merge
- אם נכשל — מציין ב-PR מה צריך

### יום 3
- Integration — חיבור של 3 המודולים ב-lib.rs (אתה עושה)
- 1 Claude יחיד מחבר: Reader → Guard → CapabilityOrchestrator → Response
- Tests end-to-end

**אחרי 3 ימים:** HumannessScore **צפוי: 0.48-0.52**

---

## 🛠️ איך אתה פותח Claude Code יחד

### Option 1: 3 חלונות terminal

```bash
# Terminal 1 — Agent A
cd /home/dinio/zets
git checkout -b feat/capability-runtime
# paste Agent A mission into Claude Code

# Terminal 2 — Agent B
cd /tmp && git clone git@github.com:idaneldad/zets.git zets-B
cd zets-B
git checkout -b feat/calibration-harness
# paste Agent B mission

# Terminal 3 — Agent C
cd /tmp && git clone git@github.com:idaneldad/zets.git zets-C
cd zets-C
git checkout -b feat/preference-store
# paste Agent C mission
```

### Option 2: 3 tmux panes באותה machine

קל יותר אם אתה יושב ליד מחשב אחד.

### Option 3: Claude Code WebUI + 2 desktop Claude Code

זה הkombination הכי מעשי — WebUI אוטונומי + 2 desktop שאתה צופה.

---

## ⚠️ מלכודות שכדאי להימנע מהן

1. **"כולם יעבדו על lib.rs"** — כל אחד יגדיר רק module, אתה עושה registration
2. **"אתה תבדוק רק בסוף"** — לא. כל 30 דקות תציץ, לתפוס hallucinations מוקדם
3. **"אל תתערב"** — תתערב. אם agent מחליט על interface שונה ממה שהmission אמר, עצור אותו
4. **"איחוד הוא קל"** — תכנן 1-2 שעות לintegration. זה לא 10 דקות
5. **"3 agents = 3x מהירות"** — מציאותית זה 1.8-2.2x, לא 3x. עדיין הרבה

---

## 📊 ה-gain הצפוי

| משימה | Cat מושפעת | Gain |
|--------|:-----------:|:----:|
| CapabilityOrchestrator | 6, 7, 9, 10 (partial) | +0.05 מיד, +0.15 כשמחברים capabilities |
| Calibration Harness | 4 | +0.04 (score ל-0.75) |
| Preference Store | 3, 1 | +0.03 |

**סה"כ אחרי 3 ימים:** HumannessScore **~0.48-0.52** (מ-0.39)

---

## 🎯 Session הבא — מה יקרה

בסשן אחרי 3 הימים:
- Tier A scores יעלו כי יש orchestrator
- Integration של Whisper + Gemini Vision (2 agents parallel אחרי שorchestrator עומד)
- Tier A צפוי לקפוץ ל-0.30+
- HumannessScore צפוי **~0.60** (MVP)

---

## 📝 הוראות לעידן — צעד-אחר-צעד

**היום:**
1. פתח terminal 1: מעתיק את Agent A mission, פותח Claude Code
2. פתח terminal 2 (או tmux pane 2): Agent B mission
3. פתח terminal 3: Agent C mission
4. עובר בין הterminals, מציץ שכל אחד מתקדם נכון

**עוקב אחרי:**
- אם agent מבקש "can I modify lib.rs?" — תגיד **לא**, הוא צריך להישאר ב-module שלו
- אם agent אומר "this is done" — trigger `cargo test --lib` בbranch שלו
- אם tests עוברים — git push; אם לא — הודיע לagent
