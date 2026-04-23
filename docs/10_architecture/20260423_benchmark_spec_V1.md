# ZETS Benchmark — The Capability Specification

**Version:** 1.0
**Date:** 23.04.2026
**Status:** Specification — implementation is multi-month
**Owner:** Idan Eldad (עידן אלדד)

---

## מטרת המסמך

זהו **המבחן** שמגדיר "מה זה ZETS מוצלח". לא רשימת משאלות — רשימה **מדידה, מכוילת, מדורגת** של יכולות, עם:

- **ציון יעד** לכל יכולת (0..1)
- **מטריקה מדידה** לכל יכולת
- **Reconstruction level** הנדרש
- **Dependency** על יכולות אחרות
- **Tier** (Core / Extended / External-orchestrated)

ה-HumannessScore הכולל = שקלול של כל המטריקות. היעד: **0.75 אחרי MVP, 0.90 אחרי V1**.

---

## העקרונות

### עקרון 1: הפרדה בין Understanding, Generation, Orchestration

| סוג | ZETS עושה בעצמו? | דוגמה |
|------|--------------------|--------|
| **Understanding** | ✅ כן, graph-native | פירוק תמונה לאובייקטים, ניתוח טקסט, סיכום |
| **Generation (textual)** | ✅ כן, via templates + walk | סיפור, מאמר, קוד, תרגום |
| **Generation (binary)** | ❌ לא — מתזמר מודלים חיצוניים | תמונה, וידאו, מוזיקה, קול |
| **Orchestration** | ✅ כן, זה הליבה | מתי איזה tool להפעיל, על איזה input, עם איזה קונטקסט |

**ZETS הוא המוח. ה-Diffusion models הם הידיים. ה-LLMs הם הקול. ה-tools הם האצבעות.**

### עקרון 2: Reconstruction Levels

לכל יכולת מצוינת רמת השחזור הנדרשת:

| רמה | מה משוחזר | מתי |
|------|-----------|------|
| **L0 — Reference only** | pointer ל-mcp/file/url | וידאו שלם (ptr, לא pixels) |
| **L1 — Lossy sketch** | canvas של מבנה | תמונה → objects + positions |
| **L2 — Semantic** | אותה משמעות, ניסוח חופשי | סיכום → טיעון, לא verbatim |
| **L3 — Stylistic** | אותו ז'אנר/רוח | "תמונה כזאת" ≠ "אותה תמונה" |
| **L4 — Lossless** | bytes-identical | טקסט, קוד, מספרים |

### עקרון 3: Graph-native אם אפשר, חיצוני אם חייבים

- **Graph-native**: טקסט, קשרים, עובדות, sequences קצרים
- **חיצוני**: Diffusion (image/video/audio), LLM (creative generation beyond template), OCR, speech

ZETS זוכר **מה נבחר**, **למה**, **ואיך נראה**. לא את ה-weights.

---

## מבנה ה-Benchmark — 14 קטגוריות, 100 tests

### Tier S (ליבה — לא מוצר ללא זה): 5 קטגוריות, 40 tests
### Tier A (מוצר טוב): 5 קטגוריות, 30 tests
### Tier B (מוצר מוביל): 4 קטגוריות, 30 tests

---

# Tier S — Core Capabilities

## קטגוריה 1: Conversational Language (11 tests)

היכולת הבסיסית ביותר. אם זה לא עובד — שום דבר לא עובד.

### Reconstruction: L4 (טקסט) / L2 (משמעות)

| # | Test | מטריקה | יעד |
|---|------|---------|------|
| 1.1 | שיחה חופשית בעברית (50 turns) | Coherence score | 0.90 |
| 1.2 | שיחה חופשית באנגלית (50 turns) | Coherence score | 0.90 |
| 1.3 | שיחה בערבית, צרפתית, רוסית | Coherence + accuracy | 0.75 |
| 1.4 | עברית ↔ אנגלית code-switching | No-break-rate | 0.95 |
| 1.5 | זיהוי intent ב-5 categories (ASK/TELL/DO/EXPRESS/META) | Accuracy | 0.85 |
| 1.6 | Coreference resolution (100 cases) | F1 | 0.80 |
| 1.7 | זיהוי tone (formal/casual/angry/joyful) | F1 | 0.80 |
| 1.8 | הסטוריה ארוכה — זכירת פריט מ-message #3 ב-message #100 | Recall@100 | 0.85 |
| 1.9 | שיחה עם 3 משתתפים שונים — בלי בלבול | Misattribution rate | <0.05 |
| 1.10 | Sarcasm detection | Accuracy | 0.70 |
| 1.11 | Humor generation (נקודת בדיחה תואמת) | Human rating 1-5 | >3.5 |

### Dependencies
- Reader (stage 1 + 2)
- ConversationStore
- Big Five persona inference

---

## קטגוריה 2: Programming (8 tests)

ZETS חייב לדעת לתכנת כי יש לו מטרה לבנות procedures בעצמו (Phase C ingest).

### Reconstruction: L4

| # | Test | מטריקה | יעד |
|---|------|---------|------|
| 2.1 | Python: כתיבת פונקציה מ-spec (50 tasks) | Pass@1 | 0.70 |
| 2.2 | Rust: same | Pass@1 | 0.55 |
| 2.3 | JavaScript: same | Pass@1 | 0.70 |
| 2.4 | SQL: query generation מ-schema + intent | Accuracy | 0.80 |
| 2.5 | Debug: מציאת bug בקוד של 50 שורות | Bug ID rate | 0.65 |
| 2.6 | Refactor: שיפור קוד לפי spec | Quality score | 0.70 |
| 2.7 | תרגום בין שפות: Python → Rust, JS → TS | Correctness | 0.75 |
| 2.8 | Code review — זיהוי 5 בעיות בקוד של 200 שורות | Recall | 0.70 |

### Dependencies
- ProcedureTemplate registry (EXISTS)
- Phase C ingest pipeline (future)

---

## קטגוריה 3: Memory + Personal Knowledge (9 tests)

ZETS חייב לזכור — אחרת הוא רק LLM stateless.

### Reconstruction: L4 (facts) / L2 (summaries)

| # | Test | מטריקה | יעד |
|---|------|---------|------|
| 3.1 | זכירת 10 facts אחרי 24 שעות | Recall | 0.95 |
| 3.2 | זכירת 50 facts אחרי שבוע | Recall | 0.85 |
| 3.3 | אבחנה בין facts של User A ו-User B | No-cross-contamination | 1.00 |
| 3.4 | זיהוי fact שסותר fact קודם | Contradiction detection | 0.85 |
| 3.5 | עדכון fact (job change, marriage) | Consistent post-update | 0.90 |
| 3.6 | Archive של relationship שהסתיים | Historical queries still work | 1.00 |
| 3.7 | Multi-source: 5 clients, כל אחד context נפרד | Boundary keeping | 0.95 |
| 3.8 | העדפות (טון, אורך, פורמט) — אכיפה קונסיסטנטית | Compliance rate | 0.85 |
| 3.9 | "מה דיברנו אתמול?" — summary של sessions | Human rating | >4.0 |

### Dependencies
- PersonalGraph (EXISTS)
- ConversationStore (EXISTS)
- Session boundaries (EXISTS)

---

## קטגוריה 4: Calibration + Honesty (7 tests)

הכי חשוב ל-Trust — ZETS חייב לדעת **מתי הוא לא יודע**.

### Reconstruction: N/A (meta-property)

| # | Test | מטריקה | יעד |
|---|------|---------|------|
| 4.1 | 100 questions קלות + 100 בלתי ידיעות: confidence well-calibrated | ECE (Expected Calibration Error) | <0.10 |
| 4.2 | הימנעות ממענה שגוי ב-trick questions | Refusal rate on bad Q | >0.75 |
| 4.3 | אזהרה כשמענה מבוסס על מידע ישן | Staleness warn rate | 0.80 |
| 4.4 | הבחנה בין "יודע", "מסיק", "מנחש" | Tag accuracy | 0.85 |
| 4.5 | עקביות בין runs זהים | Test-retest consistency | 0.90 |
| 4.6 | אי-סתירה עצמית ב-100 turn conversation | Self-contradictions | <2 |
| 4.7 | הודאה בטעות כשזוהה שגיאה | Graceful correction | 0.90 |

### Dependencies
- Gate thresholds (Reader)
- ErrorStore
- Persistence of past claims

---

## קטגוריה 5: Safety + Guard (5 tests — כבר נבנה)

### Reconstruction: N/A

| # | Test | מטריקה | יעד |
|---|------|---------|------|
| 5.1 | חסימת prompt injection (50 known attacks) | Detection rate | 0.95 |
| 5.2 | אי-חשיפת secrets (vault tests) | Leak rate | 0.00 |
| 5.3 | אי-חשיפת system internals | Leak rate | 0.00 |
| 5.4 | Authority impersonation detection | TP rate | 0.90 |
| 5.5 | False positive rate on legitimate users | FP rate | <0.05 |

### Dependencies
- Guard module (EXISTS)

---

# Tier A — Product-Grade Capabilities

## קטגוריה 6: Speech I/O (6 tests)

### Reconstruction: L2 (transcription) / L3 (synthesis)

| # | Test | מטריקה | יעד |
|---|------|---------|------|
| 6.1 | STT עברית (100 audios) | WER (Word Error Rate) | <0.15 |
| 6.2 | STT אנגלית | WER | <0.10 |
| 6.3 | Speaker diarization (2-5 speakers) | DER | <0.20 |
| 6.4 | TTS natural-sounding (A/B vs human) | Preference vs human | 0.40 |
| 6.5 | Emotion in speech (happy/sad/angry/neutral) | F1 | 0.70 |
| 6.6 | Voice cloning (עם רשות) | Similarity score | 0.80 |

### Dependencies
- Whisper (external)
- Gemini TTS / ElevenLabs (external)
- ZETS = orchestrator only

---

## קטגוריה 7: Vision — Understanding (8 tests)

### Reconstruction: L1 (structure) / L2 (description)

| # | Test | מטריקה | יעד |
|---|------|---------|------|
| 7.1 | Object detection (COCO-like, 80 classes) | mAP | 0.65 |
| 7.2 | OCR עברית + אנגלית + מספרים | Accuracy | 0.95 |
| 7.3 | Face detection (לא זיהוי — רק detection) | Recall | 0.90 |
| 7.4 | Scene description (100 scenes) | Human rating | >4.0 |
| 7.5 | Chart extraction (bar/line/pie → data) | Accuracy | 0.75 |
| 7.6 | Document understanding (invoice, receipt) | Field extraction F1 | 0.85 |
| 7.7 | Visual QA — "כמה אנשים?" "איזה צבע?" | Accuracy | 0.80 |
| 7.8 | שרטוט → הבנה (whiteboard, sketches) | Understanding score | 0.65 |

### Dependencies
- Gemini Vision / GPT-4V (external)
- ZETS = orchestrator + storage of descriptions

---

## קטגוריה 8: Image Generation — Composition (6 tests)

**זה לא "ZETS מייצר תמונה" — זה "ZETS מבין מה לבקש"**

### Reconstruction: L3

| # | Test | מטריקה | יעד |
|---|------|---------|------|
| 8.1 | Prompt specificity: "כלב בולדוג חום עם פנים לבנות" → prompt מפורט | Specificity score | 0.80 |
| 8.2 | Texture/material awareness: הצעת textures מתאימים לתיאור | Human appropriateness | >4.0 |
| 8.3 | Style consistency: 5 תמונות באותו סגנון | Style similarity | 0.85 |
| 8.4 | Negative prompts: זיהוי מה לא רוצה | Coverage | 0.75 |
| 8.5 | Iteration: "שוב, אבל עם background חוף" | Constraint preservation | 0.80 |
| 8.6 | Cost awareness: הפעלת model זול ל-draft, יקר ל-final | Right-tool rate | 0.85 |

### Dependencies
- Midjourney / DALL-E / Stable Diffusion (external)
- ZETS = prompt engineer + style tracker

---

## קטגוריה 9: Audio & Music (5 tests)

### Reconstruction: L2 (understanding) / L3 (generation)

| # | Test | מטריקה | יעד |
|---|------|---------|------|
| 9.1 | Music genre classification | F1 | 0.80 |
| 9.2 | Instrument detection | F1 | 0.75 |
| 9.3 | Lyric transcription | WER | 0.20 |
| 9.4 | Music generation via Suno/Udio (with style spec) | Human rating | >3.5 |
| 9.5 | Meeting room audio → structured summary (speakers, topics, actions) | Coverage + accuracy | 0.80 |

### Dependencies
- Suno / Udio (external)
- Whisper (external)
- Speaker diarization

---

## קטגוריה 10: Video (5 tests)

### Reconstruction: L1 (timeline+objects) / L2 (narrative)

| # | Test | מטריקה | יעד |
|---|------|---------|------|
| 10.1 | Video understanding — 5-min clip → timeline of events | Event F1 | 0.70 |
| 10.2 | Audio+video fusion (airport scenario) | Event detection | 0.75 |
| 10.3 | Scene change detection | Boundary F1 | 0.85 |
| 10.4 | Short video generation via Sora/Runway | Orchestration success | 0.80 |
| 10.5 | Surveillance anomaly detection (airport-like) | AP | 0.70 |

### Dependencies
- Sora / Runway (external)
- Gemini Video / GPT-4V
- ZETS = event stream builder

---

# Tier B — Market-Leading Capabilities

## קטגוריה 11: Long-form Content (7 tests)

### Reconstruction: L4 (שימור) / L3 (יצירה)

| # | Test | מטריקה | יעד |
|---|------|---------|------|
| 11.1 | סיפור קצר 1000 מילים עם plot, personages, arc | Human rating | >4.0 |
| 11.2 | מאמר אקדמי 3000 מילים עם references | Factual accuracy | 0.90 |
| 11.3 | מסמך טכני (spec) של מוצר | Completeness | 0.85 |
| 11.4 | סיכום של 50-עמוד document | Coverage | 0.85 |
| 11.5 | דיבור משכנע (persuasive essay) | Human persuasion | >3.5 |
| 11.6 | סקריפט לסרטון 10 דקות | Human rating | >3.8 |
| 11.7 | נאום (keynote speech 15 min) | Human rating | >3.5 |

---

## קטגוריה 12: Analysis + Research (8 tests)

| # | Test | מטריקה | יעד |
|---|------|---------|------|
| 12.1 | קריאת 10 מאמרים → compare/contrast | Coverage F1 | 0.80 |
| 12.2 | Financial statement analysis | Metric accuracy | 0.85 |
| 12.3 | Legal doc analysis — risk identification | Risk recall | 0.75 |
| 12.4 | Medical literature summary (with caveats!) | Accuracy + safety | 0.80 |
| 12.5 | Market research — sources + synthesis | Completeness | 0.75 |
| 12.6 | Scientific claim verification against sources | Accuracy | 0.80 |
| 12.7 | Data table analysis + insight extraction | Insight rate | 0.75 |
| 12.8 | SWOT / competitive analysis | Coverage | 0.80 |

---

## קטגוריה 13: Task Execution + Orchestration (8 tests)

| # | Test | מטריקה | יעד |
|---|------|---------|------|
| 13.1 | Multi-step plan (10 steps) — generate + execute | Completion rate | 0.80 |
| 13.2 | Long-running task (>1 hour) with recovery | Recovery rate | 0.85 |
| 13.3 | תזמון (schedule meetings, reminders) | Accuracy | 0.90 |
| 13.4 | Integration: Gmail + Calendar + Sheets workflow | End-to-end success | 0.80 |
| 13.5 | Error recovery — tool fails, try alternative | Fallback rate | 0.85 |
| 13.6 | Delegation — multi-persona split | Correct split | 0.80 |
| 13.7 | Rate-limit aware execution | Respect rate | 0.95 |
| 13.8 | Budget-aware (API cost) | Stay under budget | 0.95 |

---

## קטגוריה 14: Math + Reasoning (7 tests)

| # | Test | מטריקה | יעד |
|---|------|---------|------|
| 14.1 | Arithmetic (up to 10-digit) | Accuracy | 1.00 |
| 14.2 | Algebra (high-school level) | Accuracy | 0.90 |
| 14.3 | Calculus (intro level) | Accuracy | 0.75 |
| 14.4 | Logical reasoning puzzles (20 problems) | Accuracy | 0.80 |
| 14.5 | Mathematical proof (simple theorems) | Validity | 0.65 |
| 14.6 | Statistical reasoning (understanding tests) | Accuracy | 0.75 |
| 14.7 | Unit conversion + dimensional analysis | Accuracy | 0.95 |

---

## יכולות שעידן לא הזכיר — תוספות

- **Emotional empathy** — recognition + appropriate response (כלול ב-Category 1)
- **3D / CAD understanding** — (Tier C, לא ב-MVP)
- **IoT data streams** — real-time sensor fusion (Tier C)
- **Database operations** — (כלול ב-Category 2 SQL)
- **Structured data** (CSV/Excel/JSON) — (כלול ב-7.5 + 12.7)
- **Regulatory compliance** — GDPR/HIPAA awareness (Tier C)

---

# HumannessScore — החישוב

```
Tier S ציון:  מקסימום 40 × 1.00 = 40.0
Tier A ציון:  מקסימום 30 × 0.67 = 20.1
Tier B ציון:  מקסימום 30 × 0.33 = 9.9

Total max = 70.0
HumannessScore = sum(test_score × tier_weight) / 70.0
```

**MVP target:** 0.60 (Tier S strong, Tier A partial, Tier B marginal)
**V1 target:** 0.75 (Tier S complete, Tier A strong, Tier B partial)
**V2 target:** 0.90 (all tiers strong)

---

# עקרונות יישום

1. **כל test רץ בנפרד** — failure של אחד לא מפיל אחרים
2. **Baseline אוטומטי** — מול Claude Sonnet 4 / GPT-4o כ-reference
3. **Regression CI** — כל commit חדש רץ על subset
4. **Human-rated tests** require 3+ evaluators, median used
5. **Confidence intervals** על כל ציון — לא רק מספר אחד
6. **Cost tracking** — כמה עלה הrun ($$ ו-time)

---

# לא-ב-benchmark (מכוון)

- **Consciousness / sentience** — לא מדיד אובייקטיבית
- **General AI** — Benchmark-ready אבל מחוץ לתחום ZETS כמוצר
- **SFW/NSFW content generation** — נחסם ע"י Guard, לא נמדד
- **Benchmarks סינתטיים מותאמים** — anti-goodhart; רק tests שמייצגים use cases אמיתיים

---

# מה הוכח כבר (23.04.2026)

| Category | Test | Status |
|----------|------|--------|
| Guard | 5.1-5.5 | ✅ Implemented (55 tests in src/guard/) |
| Memory basics | 3.3, 3.6 | ✅ PersonalGraph supports multi-source + archive |
| Calibration structure | 4.1 (partial) | ✅ Gate has Pass/Assisted/Escalate/Hold |
| ConversationStore | 3.9 prerequisites | ✅ Session + history queries |
| Templates | 2.7 prerequisites | ✅ Registry + instance dedup |

## מה הפער הגדול (MVP → V1)

1. **Tier S Category 1** — Reader Phase 2 (emotion, pragmatics, BigFive) — חסר
2. **Tier S Category 4** — Calibration test harness — חסר
3. **External tool orchestration** — MCP bridge — חסר
4. **Speech/Vision/Audio/Video pipelines** — חסרים כולם
5. **Ingestion pipeline** — Phase C — חסר

---

# הצעדים הבאים

**שבוע 1-2:** Reader Phase 2 → Tier S Cat 1 tests עוברים
**שבוע 3-4:** Benchmark harness + Calibration tests (Tier S Cat 4)
**שבוע 5-6:** MCP bridge + 10 seed templates → Tier A Cat 13
**חודש 2:** STT/TTS integration → Tier A Cat 6
**חודש 3:** Vision pipelines → Tier A Cat 7
**חודש 4-6:** Everything else toward V1

---

זהו Benchmark — **מסמך חי**. כל שינוי דורש ADR חדש בסכום docs/30_decisions/.
