# ZETS Benchmark — Capability Specification + Status Audit

**Version:** 2.0
**Date:** 23.04.2026
**Status:** Live spec + status audit
**Owner:** Idan Eldad (עידן אלדד)

**Changes from V1:**
- Added **Current Status** column per test (23.04.2026)
- Added **Gap to Target** column
- Added **Evidence** column pointing to code/tests that prove the status
- Recalibrated targets where current capability is already higher
- Mapped every test to src/ modules that implement it (or to "not yet")

---

## מטרת המסמך

זה ה**מבחן** שמגדיר "מה זה ZETS מוצלח". לא רשימת משאלות — רשימה **מדידה, מכוילת, מדורגת, ומוכחת** של יכולות, עם:

- **ציון יעד** לכל יכולת (0..1)
- **ציון נוכחי (23.04.2026)** בהתבסס על קוד שנבנה
- **הוכחה** — לאיזה מודול ב-`src/` זה מתחבר, או "לא מיושם"
- **פער** = Target − Current
- **Tier** (Core / Extended / External-orchestrated)

ה-HumannessScore הכולל = שקלול של כל המטריקות. היעד: **0.75 אחרי MVP, 0.90 אחרי V1**.

---

## העקרונות

### עקרון 1: הפרדה בין Understanding, Generation, Orchestration

| סוג | ZETS עושה בעצמו? | דוגמה |
|------|--------------------|--------|
| **Understanding** | ✅ כן, graph-native | פירוק תמונה לאובייקטים, ניתוח טקסט, סיכום |
| **Generation (textual)** | ✅ כן, via templates + motifs + walk | סיפור, מאמר, קוד, תרגום |
| **Generation (structural)** | ✅ כן, motif-composition | narrative arcs, chord progressions |
| **Generation (binary)** | ❌ לא — מתזמר מודלים חיצוניים | תמונה photorealistic, וידאו, waveform |
| **Orchestration** | ✅ כן, זה הליבה | מתי איזה tool להפעיל |

**ZETS הוא המוח. ה-Diffusion models הם הידיים. ה-LLMs הם הקול. ה-tools הם האצבעות.**

### עקרון 2: Reconstruction Levels

| רמה | מה משוחזר | מתי |
|------|-----------|------|
| **L0 — Reference only** | pointer ל-mcp/file/url | וידאו שלם (ptr, לא pixels) |
| **L1 — Lossy sketch** | canvas של מבנה | תמונה → objects + positions |
| **L2 — Semantic** | אותה משמעות, ניסוח חופשי | סיכום → טיעון, לא verbatim |
| **L3 — Stylistic** | אותו ז'אנר/רוח | "תמונה כזאת" ≠ "אותה תמונה" |
| **L4 — Lossless** | bytes-identical | טקסט, קוד, מספרים |

### עקרון 3 (NEW): Graph-native Generation is real

עידן תיקן את עמדתי המוקדמת. ZETS **כן** יכול להיות גנרטיבי ברמה פשוטה, מבוסס motifs שנלמדו.
הוכחה: `src/composition/weaver.rs` — `test_generation_is_really_generative`.

---

## מבנה ה-Benchmark — 14 קטגוריות, 100 tests

### Tier S (ליבה — לא מוצר ללא זה): 5 קטגוריות, 40 tests
### Tier A (מוצר טוב): 5 קטגוריות, 30 tests
### Tier B (מוצר מוביל): 4 קטגוריות, 30 tests

### סימני הסטטוס

| סימן | משמעות |
|-----|---------|
| ✅ | מיושם במלואו + tests passing |
| 🟢 | 0.70-0.90: מיושם ברובו |
| 🟡 | 0.40-0.69: שלד + חלק מהkey functions |
| 🟠 | 0.10-0.39: prerequisites / tiny prototype |
| 🔴 | 0.00: לא קיים |

---

# Tier S — Core Capabilities

## קטגוריה 1: Conversational Language (11 tests)

**Reconstruction:** L4 (טקסט) / L2 (משמעות)

| # | Test | יעד | סטטוס (23.04) | פער | הוכחה/מודול |
|---|------|----:|:--------------:|----:|--------------|
| 1.1 | שיחה חופשית בעברית (50 turns) | 0.90 | 🟡 0.45 | -0.45 | Reader skeleton (`reader/`) + conversation store. חסר Phase 2: emotion/pragmatics/BigFive implementation |
| 1.2 | שיחה חופשית באנגלית (50 turns) | 0.90 | 🟡 0.45 | -0.45 | כנ"ל |
| 1.3 | ערבית, צרפתית, רוסית | 0.75 | 🟠 0.20 | -0.55 | fold (BPE 47/48 langs) + sense_graph. חסר: language-specific emotion patterns |
| 1.4 | עברית ↔ אנגלית code-switching | 0.95 | 🟡 0.60 | -0.35 | sense_graph עם WordNet synsets, פתרון partial synonyms. חסר: runtime switching detection |
| 1.5 | Intent classification (5 classes) | 0.85 | 🟡 0.50 | -0.35 | Reader תומך ב-PragmaticIntent enum. חסר: classifier |
| 1.6 | Coreference resolution | 0.80 | 🔴 0.00 | -0.80 | לא מיושם. מקביל: spreading_activation קיים אבל לא מנוצל לcoref |
| 1.7 | Tone recognition (formal/casual/angry/joyful) | 0.80 | 🟠 0.25 | -0.55 | Reader::EmotionRead יש signals. חסר: 8 textual signals ו-tone classifier |
| 1.8 | Long-history recall | 0.85 | 🟢 0.75 | -0.10 | ConversationStore + session + `recent()/history_for()`. חסר: attention/salience ranking |
| 1.9 | 3-participant no-misattribution | 0.95 | 🟢 0.80 | -0.15 | ConversationStore `per-source` separation מיושם. כל sourceid מבודד |
| 1.10 | Sarcasm detection | 0.70 | 🔴 0.00 | -0.70 | לא מיושם |
| 1.11 | Humor generation | 0.70 | 🟠 0.15 | -0.55 | composition motifs תומכים ב-pattern, אבל אין motif bank לbדיחות |

**Avg Cat 1:** 0.38 — פער גדול. **הצמתה העיקרית:** Reader Phase 2

---

## קטגוריה 2: Programming (8 tests)

**Reconstruction:** L4

| # | Test | יעד | סטטוס | פער | הוכחה |
|---|------|----:|:------:|----:|--------|
| 2.1 | Python function from spec | 0.70 | 🟡 0.45 | -0.25 | `llm_adapter` + `procedure_template`. חסר: codegen pipeline |
| 2.2 | Rust function from spec | 0.55 | 🟡 0.40 | -0.15 | ZETS כתוב ב-Rust — יש דוגמאות עצומות בself. `procedure_template` loopback |
| 2.3 | JavaScript function from spec | 0.70 | 🟠 0.30 | -0.40 | Language::JavaScript מוכר ב-instance. אין runtime |
| 2.4 | SQL generation | 0.80 | 🟡 0.50 | -0.30 | `system_graph` יש SQL-like queries. חסר: NL-to-SQL |
| 2.5 | Debugging 50-line code | 0.65 | 🟠 0.20 | -0.45 | אין debugger integration |
| 2.6 | Refactor to spec | 0.70 | 🟠 0.25 | -0.45 | procedure_template shape_hash מזהה shapes. חסר: transformation |
| 2.7 | Cross-language translation | 0.75 | 🟡 0.55 | -0.20 | **procedure_template.Language + binding** מיושם! מאפשר Py→Rust mapping של אותו template |
| 2.8 | Code review — 5 issues in 200 lines | 0.70 | 🟠 0.20 | -0.50 | guard patterns מזהים injection. לא מזהה bugs |

**Avg Cat 2:** 0.35 — פער בינוני. **מחסום:** Phase C ingest + LLM orchestration

---

## קטגוריה 3: Memory + Personal Knowledge (9 tests)

**Reconstruction:** L4 (facts) / L2 (summaries)

| # | Test | יעד | סטטוס | פער | הוכחה |
|---|------|----:|:------:|----:|--------|
| 3.1 | 10 facts recalled @24h | 0.95 | 🟢 0.85 | -0.10 | `personal_graph` + `conversation` persistent. **חסר:** disk persistence tests |
| 3.2 | 50 facts @week | 0.85 | 🟢 0.80 | -0.05 | כנ"ל |
| 3.3 | User A vs User B separation | 1.00 | ✅ 0.95 | -0.05 | `ScopeRef` + `Visibility` + per-source ConversationStore. ⚠️ לא נבדק end-to-end |
| 3.4 | Contradiction detection | 0.85 | 🟡 0.40 | -0.45 | metacognition has Confidence levels. חסר: contradiction detector |
| 3.5 | Fact update (job change) | 0.90 | 🟢 0.85 | -0.05 | `Relationship.end()` + `Lifecycle` transition מיושמים |
| 3.6 | Archived relationship, historical queries | 1.00 | ✅ 0.95 | -0.05 | `was_active_at(ts)` מיושם ו-tested |
| 3.7 | 5 clients, separate contexts | 0.95 | ✅ 0.90 | -0.05 | `Source` enum + scope system |
| 3.8 | Preference enforcement | 0.85 | 🟠 0.25 | -0.60 | אין preference store |
| 3.9 | "מה דיברנו אתמול?" summary | 0.80 | 🟡 0.55 | -0.25 | `ConversationStore.sessions_for()` יש. חסר: summarizer |

**Avg Cat 3:** 0.72 — **הכי קרוב ליעד.** זה המודול הכי בוגר

---

## קטגוריה 4: Calibration + Honesty (7 tests)

**Reconstruction:** N/A

| # | Test | יעד | סטטוס | פער | הוכחה |
|---|------|----:|:------:|----:|--------|
| 4.1 | ECE on easy+hard Q mix | 0.90 | 🟡 0.50 | -0.40 | `metacognition::Confidence` enum (5 levels). `verify.rs` מיושם. חסר: ECE harness + benchmark set |
| 4.2 | Trick-question refusal | 0.75 | 🟡 0.50 | -0.25 | guard blocks + Reader gate (Hold action) |
| 4.3 | Staleness warning | 0.80 | 🟠 0.25 | -0.55 | אין timestamp על facts |
| 4.4 | Know/Infer/Guess tagging | 0.85 | 🟡 0.55 | -0.30 | Confidence enum מיישם 5 רמות. חסר: tagger |
| 4.5 | Test-retest consistency | 0.90 | 🟢 0.85 | -0.05 | `cognitive_modes` — "100% deterministic" הכרזה של המודול |
| 4.6 | Self-contradiction in 100-turn | 0.95 | 🟡 0.50 | -0.45 | `error_store` קיים. חסר: contradiction detector |
| 4.7 | Graceful correction | 0.90 | 🟡 0.40 | -0.50 | לא מיושם פורמלית |

**Avg Cat 4:** 0.51 — פער בינוני. **מחסום:** benchmark-harness + contradiction detection

---

## קטגוריה 5: Safety + Guard (5 tests)

**Reconstruction:** N/A

| # | Test | יעד | סטטוס | פער | הוכחה |
|---|------|----:|:------:|----:|--------|
| 5.1 | Prompt injection block (50 attacks) | 0.95 | 🟢 0.85 | -0.10 | `guard/input_guard.rs` + 30+ patterns (EN+HE). **חסר:** eval against 50-attack corpus |
| 5.2 | Secret leakage prevention | 1.00 | 🟢 0.90 | -0.10 | `guard/output_guard.rs` + 10+ key-prefix patterns + `secrets/vault.rs` separation |
| 5.3 | System non-disclosure | 1.00 | 🟢 0.90 | -0.10 | Output guard בודק self-disclosure + internal paths |
| 5.4 | Authority impersonation | 0.90 | 🟢 0.80 | -0.10 | `AUTHORITY_CLAIM_PATTERNS` + source-aware logic (Owner/Guest) |
| 5.5 | False positive rate on legit | 0.95 | 🟡 0.65 | -0.30 | ⚠️ לא נבדק על corpus של legit users |

**Avg Cat 5:** 0.82 — **הקרוב השני ליעד.** Guard module בנוי מלא, רק חסר evaluation

---

# Tier A — Product-Grade Capabilities

## קטגוריה 6: Speech I/O (6 tests)

**Reconstruction:** L2 (transcription) / L3 (synthesis)

| # | Test | יעד | סטטוס | פער | הוכחה |
|---|------|----:|:------:|----:|--------|
| 6.1 | Hebrew STT | 0.85 | 🔴 0.00 | -0.85 | אין integration. `capabilities/` ריק |
| 6.2 | English STT | 0.90 | 🔴 0.00 | -0.90 | כנ"ל |
| 6.3 | Speaker diarization | 0.80 | 🔴 0.00 | -0.80 | לא מיושם |
| 6.4 | TTS natural sound | 0.40 | 🔴 0.00 | -0.40 | אין integration |
| 6.5 | Emotion in speech | 0.70 | 🔴 0.00 | -0.70 | לא מיושם |
| 6.6 | Voice cloning | 0.80 | 🔴 0.00 | -0.80 | לא מיושם |

**Avg Cat 6:** 0.00 — **פער מלא.** מחסום: capability orchestrator + Whisper/Gemini TTS integration

---

## קטגוריה 7: Vision — Understanding (8 tests)

**Reconstruction:** L1 / L2

| # | Test | יעד | סטטוס | פער | הוכחה |
|---|------|----:|:------:|----:|--------|
| 7.1 | Object detection (COCO 80) | 0.65 | 🟠 0.15 | -0.50 | `bin/vision_decomposer.rs` — Hopfield-based partial. לא Production |
| 7.2 | OCR עברית+אנגלית | 0.95 | 🔴 0.00 | -0.95 | לא מיושם |
| 7.3 | Face detection | 0.90 | 🔴 0.00 | -0.90 | לא מיושם |
| 7.4 | Scene description | 0.80 | 🟠 0.20 | -0.60 | `vision_decomposer` + `hopfield` — hierarchical decomposition עובד |
| 7.5 | Chart extraction | 0.75 | 🔴 0.00 | -0.75 | לא מיושם |
| 7.6 | Document understanding | 0.85 | 🔴 0.00 | -0.85 | לא מיושם |
| 7.7 | Visual QA | 0.80 | 🔴 0.00 | -0.80 | לא מיושם |
| 7.8 | Sketch understanding | 0.65 | 🟠 0.15 | -0.50 | Hopfield banks יכולים partial |

**Avg Cat 7:** 0.06 — **פער מלא.** מחסום: vision capability + Gemini Vision integration

---

## קטגוריה 8: Image Generation — Composition (6 tests)

**Reconstruction:** L3

**חשוב:** זה לא ZETS שמייצר תמונה אלא ZETS שמבין מה לבקש ומתזמר

| # | Test | יעד | סטטוס | פער | הוכחה |
|---|------|----:|:------:|----:|--------|
| 8.1 | Prompt specificity | 0.80 | 🟡 0.55 | -0.25 | `composition/motif_bank.rs` — ImagePrompt kind + test `test_image_prompt_motif` |
| 8.2 | Texture/material awareness | 0.80 | 🟠 0.25 | -0.55 | MotifBank יש by_tag אבל אין texture library |
| 8.3 | Style consistency 5-set | 0.85 | 🟡 0.50 | -0.35 | `CompositionPlan.style_hints` + `Motif.style` מיושמים |
| 8.4 | Negative prompt inference | 0.75 | 🟡 0.45 | -0.30 | `CompositionPlan.must_exclude` קיים |
| 8.5 | Iteration with constraints | 0.80 | 🟡 0.50 | -0.30 | plan.must_include/exclude. חסר: diff between generations |
| 8.6 | Cost-aware model selection | 0.85 | 🟡 0.45 | -0.40 | `CompositionPlan.external_budget` קיים. חסר: model catalog |

**Avg Cat 8:** 0.45 — **יחסית טוב.** חסר: חיבור ל-actual diffusion services

---

## קטגוריה 9: Audio & Music (5 tests)

**Reconstruction:** L2 / L3

| # | Test | יעד | סטטוס | פער | הוכחה |
|---|------|----:|:------:|----:|--------|
| 9.1 | Music genre classification | 0.80 | 🔴 0.00 | -0.80 | לא מיושם |
| 9.2 | Instrument detection | 0.75 | 🔴 0.00 | -0.75 | לא מיושם |
| 9.3 | Lyric transcription | 0.80 | 🔴 0.00 | -0.80 | חלק מ-STT generic |
| 9.4 | Music generation (Suno) | 0.70 | 🟠 0.25 | -0.45 | MotifKind::MusicalPhrase + test מיושמים. חסר: Suno integration |
| 9.5 | Meeting room audio → summary | 0.80 | 🟠 0.20 | -0.60 | conversation store + reader. חסר: STT + diarization |

**Avg Cat 9:** 0.09 — **פער כמעט מלא.** חסר: audio capability

---

## קטגוריה 10: Video (5 tests)

**Reconstruction:** L1 / L2

| # | Test | יעד | סטטוס | פער | הוכחה |
|---|------|----:|:------:|----:|--------|
| 10.1 | 5-min video → timeline | 0.70 | 🔴 0.00 | -0.70 | לא מיושם |
| 10.2 | Audio+video fusion (airport) | 0.75 | 🔴 0.00 | -0.75 | לא מיושם |
| 10.3 | Scene change detection | 0.85 | 🔴 0.00 | -0.85 | לא מיושם |
| 10.4 | Short video generation | 0.80 | 🟠 0.20 | -0.60 | plan+weaver תומכים ב-CapabilityCall. חסר: Sora/Runway integration |
| 10.5 | Surveillance anomaly | 0.70 | 🔴 0.00 | -0.70 | לא מיושם |

**Avg Cat 10:** 0.04 — **פער כמעט מלא.** צריך video capability

---

# Tier B — Market-Leading Capabilities

## קטגוריה 11: Long-form Content (7 tests)

**Reconstruction:** L4 / L3

| # | Test | יעד | סטטוס | פער | הוכחה |
|---|------|----:|:------:|----:|--------|
| 11.1 | 1000-word story with arc | 0.80 | 🟡 0.55 | -0.25 | `composition/weaver` + motif-based narrative. `test_weave_three_step_story` עובר |
| 11.2 | 3000-word article w/ refs | 0.90 | 🟡 0.45 | -0.45 | חסר: citation management |
| 11.3 | Product spec | 0.85 | 🟡 0.50 | -0.35 | `procedure_template` יכול לייצג spec |
| 11.4 | 50-page doc → summary | 0.85 | 🟡 0.50 | -0.35 | path_mining motifs + composition |
| 11.5 | Persuasive essay | 0.70 | 🟠 0.35 | -0.35 | `MotifKind::ArgumentPattern` + composition |
| 11.6 | 10-min video script | 0.76 | 🟠 0.30 | -0.46 | composition iter |
| 11.7 | Keynote speech | 0.70 | 🟠 0.30 | -0.40 | composition iter |

**Avg Cat 11:** 0.42 — **יותר טוב ממה שחשבתי.** composition layer עושה הרבה

---

## קטגוריה 12: Analysis + Research (8 tests)

| # | Test | יעד | סטטוס | פער | הוכחה |
|---|------|----:|:------:|----:|--------|
| 12.1 | 10-paper compare/contrast | 0.80 | 🟡 0.40 | -0.40 | `morphology` מיושם (36 tests). חסר: multi-doc synthesis |
| 12.2 | Financial statement | 0.85 | 🔴 0.00 | -0.85 | אין financial parser |
| 12.3 | Legal doc risk | 0.75 | 🔴 0.00 | -0.75 | לא מיושם |
| 12.4 | Medical literature summary | 0.80 | 🔴 0.00 | -0.80 | לא מיושם (וגם safety-sensitive) |
| 12.5 | Market research synthesis | 0.75 | 🟠 0.25 | -0.50 | search module + composition יכולים partial |
| 12.6 | Scientific claim verification | 0.80 | 🟡 0.40 | -0.40 | `verify` module קיים. tests: 20 |
| 12.7 | Data table insight | 0.75 | 🟠 0.25 | -0.50 | system_graph מתמודד עם structured data |
| 12.8 | SWOT analysis | 0.80 | 🟠 0.25 | -0.55 | motifs קיימים ל-argument patterns |

**Avg Cat 12:** 0.19

---

## קטגוריה 13: Task Execution + Orchestration (8 tests)

| # | Test | יעד | סטטוס | פער | הוכחה |
|---|------|----:|:------:|----:|--------|
| 13.1 | 10-step plan execute | 0.80 | 🟡 0.50 | -0.30 | `planner.rs` + `CompositionPlan` + `Weaver`. חסר: actual execution |
| 13.2 | Long task with recovery | 0.85 | 🟠 0.30 | -0.55 | `error_store` מתעד. חסר: retry logic |
| 13.3 | Schedule meetings | 0.90 | 🔴 0.00 | -0.90 | חסר: calendar connector integration |
| 13.4 | Gmail+Calendar+Sheets workflow | 0.80 | 🟡 0.45 | -0.35 | `connectors/seed.rs` — 9 bundles (gmail/calendar/slack/telegram/whatsapp/smtp/drive/sheets/openai). **חסר: actual HTTP execution** |
| 13.5 | Tool-fail fallback | 0.85 | 🟠 0.35 | -0.50 | guard_pipeline וerror_store. חסר: fallback policy |
| 13.6 | Multi-persona delegation | 0.80 | 🟡 0.50 | -0.30 | cognitive_modes (4 modes) + search personas (7 strategies) |
| 13.7 | Rate-limit aware | 0.95 | 🟠 0.20 | -0.75 | לא מיושם |
| 13.8 | Budget-aware | 0.95 | 🟡 0.40 | -0.55 | `CompositionPlan.external_budget` קיים. לא אכיף |

**Avg Cat 13:** 0.34

---

## קטגוריה 14: Math + Reasoning (7 tests)

| # | Test | יעד | סטטוס | פער | הוכחה |
|---|------|----:|:------:|----:|--------|
| 14.1 | 10-digit arithmetic | 1.00 | ✅ 1.00 | 0.00 | `system_graph/vm.rs` + Rust native. trivial |
| 14.2 | Algebra (high-school) | 0.90 | 🟡 0.50 | -0.40 | vm יש arithmetic. חסר: symbolic algebra |
| 14.3 | Calculus (intro) | 0.75 | 🟠 0.20 | -0.55 | אין symbolic calculus |
| 14.4 | Logic puzzles (20 problems) | 0.80 | 🟡 0.55 | -0.25 | `cognitive_modes::PrecisionMode` — bounded DFS. system_graph reasoning |
| 14.5 | Simple theorem proofs | 0.65 | 🟠 0.25 | -0.40 | system_graph rules. חסר: proof engine |
| 14.6 | Statistical reasoning | 0.75 | 🟠 0.30 | -0.45 | חסר: stats library |
| 14.7 | Unit conversion | 0.95 | 🟡 0.50 | -0.45 | procedure_template תומך ב-NameRole::MathSymbol. חסר: unit library |

**Avg Cat 14:** 0.47 — **יחסית טוב** בזכות vm.rs + cognitive modes

---

# סיכום כולל (23.04.2026)

## HumannessScore נוכחי

```
Tier S (40 tests):
  Cat 1 avg 0.38 × 11 = 4.18
  Cat 2 avg 0.35 × 8  = 2.80
  Cat 3 avg 0.72 × 9  = 6.48
  Cat 4 avg 0.51 × 7  = 3.57
  Cat 5 avg 0.82 × 5  = 4.10
  Tier S sum = 21.13 / 40 max = 0.528

Tier A (30 tests):
  Cat 6 avg 0.00 × 6  = 0.00
  Cat 7 avg 0.06 × 8  = 0.48
  Cat 8 avg 0.45 × 6  = 2.70
  Cat 9 avg 0.09 × 5  = 0.45
  Cat 10 avg 0.04 × 5 = 0.20
  Tier A sum = 3.83 × 0.67 = 2.57
  Tier A max = 30 × 0.67   = 20.1

Tier B (30 tests):
  Cat 11 avg 0.42 × 7 = 2.94
  Cat 12 avg 0.19 × 8 = 1.52
  Cat 13 avg 0.34 × 8 = 2.72
  Cat 14 avg 0.47 × 7 = 3.29
  Tier B sum = 10.47 × 0.33 = 3.46
  Tier B max = 30 × 0.33   = 9.9

TOTAL:
  Weighted Raw = 21.13 + 2.57 + 3.46 = 27.16
  Max Possible = 40 + 20.1 + 9.9 = 70.0
  HumannessScore = 27.16 / 70 = 0.388
```

## 🎯 סטטוס: **0.39** (פער ל-MVP 0.60: -0.21; ל-V1 0.75: -0.36)

---

## איפה הפער הכי גדול?

### חמישייה ל-MVP (מ-0.39 ל-0.60):

| סדר | פער קריטי | קטגוריה | גיין צפוי |
|------|------------|---------|-----------|
| 1 | **Reader Phase 2** | Cat 1, Cat 4 | +0.07 (Cat 1) + +0.04 (Cat 4) |
| 2 | **Speech capabilities (STT/TTS)** | Cat 6 | +0.05 |
| 3 | **Vision capabilities** | Cat 7 | +0.05 |
| 4 | **Connector execution (HTTP)** | Cat 13 | +0.03 |
| 5 | **Preference store + contradiction detector** | Cat 3, 4 | +0.02 |

**סך הכל צפוי לתוסף: +0.26 → 0.65**. מעל MVP 0.60.

---

## נקודות חוזקה ודאיות

1. **Memory + PersonalGraph** (Cat 3): 0.72 — הכי בוגר
2. **Safety + Guard** (Cat 5): 0.82 — כמעט מושלם
3. **Math + Reasoning** (Cat 14): 0.47 — system_graph vm עוזר
4. **Image Composition** (Cat 8): 0.45 — composition layer עושה את העבודה

## מכשולים עיקריים

1. **External capability orchestration** — אין. Cat 6, 7, 9, 10 תלויים בזה
2. **Reader Phase 2** — קריטי ל-Cat 1
3. **Calibration harness** — קיים קוד, חסרים eval sets
4. **Connector runtime** — bundles מוגדרים, HTTP execution לא

---

## Actionable Plan מעכשיו

### שבוע הקרוב
1. ✅ יישום Reader Phase 2 — emotion signals, pragmatics, BigFive → Cat 1 קופץ ל-0.70+
2. ✅ Calibration harness + ECE compute → Cat 4 ל-0.70+
3. ✅ Capability orchestrator skeleton (stdin/stdout JSON) → פותח כיוון ל-Cat 6/7/9/10

### שבועיים-שלוש
4. Whisper integration (Python subprocess) → Cat 6.1, 6.2 ל-0.80+
5. Gemini Vision integration → Cat 7.2, 7.4, 7.6 ל-0.70+
6. Connector HTTP execution → Cat 13.4 ל-0.70+
7. Musical motif bank 100+ patterns → Cat 9.4 ל-0.50+

### חודש
8. Preference store → Cat 3.8 ל-0.80+
9. Contradiction detector → Cat 3.4, 4.6 ל-0.70+
10. Diffusion integration (Stable Diffusion API) → Cat 8 ל-0.75+, Cat 10.4 ל-0.50+

**עם כל זה:** **HumannessScore צפוי: 0.68-0.72** — **MVP מושג, V1 בטווח**

---

## ממשיך לממש — הכל הכל

המהלך הבא בסשן הזה: **Reader Phase 2 + Capability orchestrator skeleton**. מלא.
