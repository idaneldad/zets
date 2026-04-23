# ZETS — The Master Index

**Last updated:** 23.04.2026 (late PM)
**Last tests run:** 23.04.2026 — **1,278 tests passing** (up from 1,095 this morning; +183 from 4 parallel agents)
**Current HumannessScore (self-estimated):** **0.48 / 1.00** (up from 0.39 — 4 new modules landed)

> זה המוח של ZETS — המפה של הכל. כל איזור מוצג כאן בקצרה, עם סטטוס וקישור לעמוד המפורט.

---

## המוח — איך זה מאורגן

ZETS מאורגן כמו מוח אנושי: **יש ליבה, יש חושים, יש אברים פעילים, יש זיכרון**. כל חלק הוא מודול בפני עצמו, עם מצב נוכחי, תוכנית, ובדיקות.

```
┌─────────────────────────────────────────────────────────┐
│                   ZETS — המוח                            │
│                                                          │
│   ┌────────────────────┐   ┌───────────────────────┐   │
│   │   ליבה (Core)      │   │   זיכרון (Memory)     │   │
│   │   Graph + Atoms    │   │   PersonalGraph       │   │
│   │   Walk + Reason    │   │   Conversation        │   │
│   │   Reader + Guard   │   │   Secrets Vault       │   │
│   │                    │   │   Preferences ✨ NEW   │   │
│   └────────────────────┘   └───────────────────────┘   │
│                                                          │
│   ┌────────────────────┐   ┌───────────────────────┐   │
│   │   חושים (Senses)   │   │   אברים (Organs)      │   │
│   │   Speech I/O       │   │   Reader (understand) │   │
│   │   Vision           │   │   Composer (create)   │   │
│   │   Audio            │   │   Guard (protect)     │   │
│   │   Text (Language)  │   │   Connectors (act)    │   │
│   └────────────────────┘   └───────────────────────┘   │
│                                                          │
│   ┌────────────────────────────────────────────────┐   │
│   │           יצירה (Generation)                    │   │
│   │   Composition (motif → story/prompt/music)      │   │
│   │   Procedure Templates (code + math patterns)    │   │
│   │   Orchestration (external tools via runtime) ✨ │   │
│   │   Canonization (variant detection) ✨ NEW       │   │
│   │   Calibration Harness (ECE, Brier) ✨ NEW       │   │
│   └────────────────────────────────────────────────┘   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

✨ = landed today (23.04.2026 via 4 parallel Claude Code agents)

---

## 14 יכולות (Capabilities) — מה ZETS יודע לעשות

| # | יכולת | תחום | סטטוס (23.04) | יעד MVP | באחריות | עמוד |
|---|--------|-------|:-------------:|:-------:|---------|-------|
| 1 | שיחה בשפה טבעית | Core | 🟡 **0.65** ↑ | 0.90 | גרף + motifs + Reader P2 | [conversational_language](capabilities/conversational_language.md) |
| 2 | תכנות | Core | 🟡 0.35 | 0.70 | גרף + חיצוני (LLM) | [programming](capabilities/programming.md) |
| 3 | זיכרון אישי | Core | 🟢 **0.82** ↑ | 0.95 | גרף + Preferences | [memory](capabilities/memory.md) |
| 4 | כיול ויושר | Core | 🟡 **0.68** ↑ | 0.80 | גרף + CalibrationHarness | [calibration](capabilities/calibration.md) |
| 5 | בטיחות | Core | 🟢 0.82 | 0.95 | binary (compile-time) | [safety](capabilities/safety.md) |
| 6 | קול — שמיעה+דיבור | Senses | 🔴 0.00 | 0.85 | חיצוני (Whisper/TTS) via runtime ✨ | [speech](capabilities/speech.md) |
| 7 | ראייה — הבנה | Senses | 🟠 0.06 | 0.75 | חיצוני (Gemini Vision) via runtime ✨ | [vision](capabilities/vision.md) |
| 8 | תמונות — יצירה | Senses | 🟡 0.45 | 0.75 | גרף + חיצוני (SD) | [image_composition](capabilities/image_composition.md) |
| 9 | אודיו ומוזיקה | Senses | 🟠 0.09 | 0.70 | גרף + חיצוני (Suno) | [audio_music](capabilities/audio_music.md) |
| 10 | וידאו | Senses | 🟠 0.04 | 0.65 | חיצוני (Sora) | [video](capabilities/video.md) |
| 11 | תוכן ארוך | Generation | 🟡 0.42 | 0.75 | גרף + motifs | [long_form_content](capabilities/long_form_content.md) |
| 12 | ניתוח ומחקר | Generation | 🟠 0.19 | 0.70 | גרף + חיצוני | [analysis_research](capabilities/analysis_research.md) |
| 13 | ביצוע משימות | Organs | 🟡 **0.52** ↑ | 0.80 | גרף + CapabilityRuntime ✨ | [task_orchestration](capabilities/task_orchestration.md) |
| 14 | מתמטיקה והיגיון | Generation | 🟡 0.47 | 0.80 | גרף (binary) | [math_reasoning](capabilities/math_reasoning.md) |
| 15 | זיהוי variants + epistemic | Meta | 🟢 **0.70** ✨ | 0.85 | גרף + Canonization ✨ | [canonization](capabilities/canonization.md) |

↑ = עלה היום | ✨ = מודול חדש | **Overall שוקלל: 0.48** (לעומת 0.39 הבוקר)

### Legend: באחריות
- **גרף** = ZETS עושה בעצמו, דרך walk + motifs
- **binary (compile-time)** = קוד מקומפל (Rust), לא בגרף
- **חיצוני** = מופעל API של צד שלישי
- **גרף + חיצוני** = ZETS מתזמר, tool חיצוני מבצע את החלק ה"כבד"

### Legend: סטטוס
- ✅ מלא + בדוק | 🟢 0.70-0.90 | 🟡 0.40-0.69 | 🟠 0.10-0.39 | 🔴 0.00

---

## 🧠 הארכיטקטורה

| מודול | תפקיד | סטטוס | קוד | מסמך |
|--------|--------|:-----:|-----|-------|
| Reader | הבנת input (emotion, intent, style) | 🟢 **0.75** ↑ | `src/reader/` | [system](architecture/system_overview.md) |
| Guard | הגנה + security | 🟢 0.85 | `src/guard/` | [architecture_dna](architecture/architecture_dna.md) |
| PersonalGraph | זהויות + קשרים (time-aware) | 🟢 0.90 | `src/personal_graph/` | [body_mind](architecture/body_mind.md) |
| Composition | יצירה graph-native | 🟡 0.55 | `src/composition/` | — |
| ProcedureTemplate | תבניות קוד/מתמטיקה | 🟢 0.80 | `src/procedure_template/` | — |
| ConversationStore | זיכרון שיחות | 🟢 0.85 | `src/conversation/` | — |
| Secrets Vault | סודות מוצפנים (לא בגרף) | 🟢 0.75 | `src/secrets/` | — |
| Benchmark | framework מדידה | 🟢 **0.75** ↑ | `src/benchmark/` | — |
| Wisdom Engines | קבלה + אסטרולוגיה + HD | 🟢 0.80 | `src/wisdom_engines/` | — |
| System Graph VM | ריצה + reasoning | 🟡 0.60 | `src/system_graph/` | [cognitive_kinds](architecture/cognitive_kinds.md) |
| Cognitive Modes | 4 שיטות traversal | 🟢 0.75 | `src/cognitive_modes.rs` | [cognitive_modes](architecture/cognitive_modes.md) |
| Sense Graph | WordNet synsets | 🟢 0.70 | `src/sense_graph.rs` | — |
| **CapabilityRuntime** ✨ | orchestrator ל-APIs חיצוניים | 🟢 **0.80** | `src/capability_runtime/` | [capability_runtime](capabilities/capability_runtime.md) |
| **Preferences** ✨ | העדפות user (explicit + inferred) | 🟢 **0.80** | `src/preferences/` | [preferences](capabilities/preferences.md) |
| **CalibrationHarness** ✨ | ECE + Brier + K/I/G | 🟢 **0.80** | `src/benchmark/calibration/` | [calibration](capabilities/calibration.md) |
| **Canonization** ✨ | variant + epistemic classification | 🟢 **0.70** | `src/canonization/` | [canonization](capabilities/canonization.md) |

---

## 📋 מלאי (Inventory) — מה יש בפועל

| נושא | קישור |
|-------|-------|
| 🌐 שפות טבעיות + תכנות + מדעיות | [languages](inventory/languages.md) |
| 🔌 חיבורים (connectors) | [connectors](inventory/connectors.md) |
| 🛠️ Capabilities חיצוניות | [capabilities_external](inventory/capabilities_external.md) |

---

## 🎯 החלטות (Decisions) — למה בנינו כך

| נושא | תאריך | קישור |
|-------|:-----:|-------|
| Rust only core | 22.04 | [rust_only_core](decisions/rust_only_core.md) |
| Capability registry | 22.04 | [capability_registry](decisions/capability_registry.md) |
| Engineering rules | 23.04 | [engineering_rules](decisions/engineering_rules.md) |
| What to build (and not) | 23.04 | [what_to_build](decisions/what_to_build.md) |
| Variantica transition | 23.04 | [variantica_transition](decisions/variantica_transition.md) |
| **Parallel agents workflow** ✨ | 23.04 | [parallel agents](PARALLEL_WORK.md) |

---

## 📊 HumannessScore Tracking

המדד המרכזי שמודד "כמה ZETS קרוב לאדם". שקלול של 100 tests ב-14 קטגוריות + חדש.

| תאריך | עדכון | Score | פירוט |
|:-----:|-------|:-----:|-------|
| 23.04.2026 AM | הערכה ראשונה לאחר audit | **0.39** | baseline |
| 23.04.2026 PM | Reader Phase 2 + 4 parallel agents (CapRun/Calib/Prefs/Canon) | **0.48** (+0.09) | 183 tests חדשים |

### פירוט הקפיצה 0.39 → 0.48

| Capability | קודם | עכשיו | סיבה |
|------------|:----:|:-----:|------|
| Cat 1 (שיחה) | 0.45 | 0.65 | Reader P2: emotion/intent/style |
| Cat 3 (זיכרון) | 0.72 | 0.82 | Preferences + inference |
| Cat 4 (כיול) | 0.51 | 0.68 | CalibrationHarness (ECE + Brier) |
| Cat 13 (משימות) | 0.34 | 0.52 | CapabilityRuntime |
| Meta (חדש) | — | 0.70 | Canonization |

**יעד MVP:** 0.60 (פער: −0.12 במקום −0.21)
**יעד V1:** 0.75 (פער: −0.27)
**יעד V2:** 0.90

### איך מגיעים ל-MVP 0.60?

פער של 0.12 מתורגם ל-4 משימות עיקריות (by priority):

1. **Speech wiring** (Cat 6: 0.00 → 0.75) — Whisper + TTS דרך CapabilityRuntime, +0.05 לכולל
2. **Vision wiring** (Cat 7: 0.06 → 0.70) — Gemini Vision, +0.04 לכולל
3. **Programming upgrade** (Cat 2: 0.35 → 0.65) — hybrid LLM orchestration, +0.02 לכולל
4. **Analysis+Research** (Cat 12: 0.19 → 0.50) — multi-source reasoning, +0.02 לכולל

**סה"כ:** 0.48 + 0.13 = **0.61 → MVP achieved**. צפוי בעוד 6-8 שבועות של עבודה ריאלית.

---

## 🔬 מחקר (Research)

| תחום | קישור |
|-------|-------|
| OpenClaw — כלים חיצוניים | [research_openclaw](research_openclaw.md) |

---

## ⚙️ איך לקרוא את המסמכים

### כל capability document מכיל:
1. **מה המשימה** — תיאור הדרישה
2. **מה תיחשב הצלחה** — הקריטריון המדיד
3. **איך בוחנים** — סוגי בדיקות (QA + TEST)
4. **טבלת tests** עם סטטוס, תאריכי בדיקה, ציונים לאורך זמן
5. **באחריות** — גרף / binary / חיצוני / שילוב
6. **קוד** — מה המודול שמטפל
7. **פער** — מה חסר להגיע ליעד

### שמות קבצים — הקונבנציה
- `<topic>.md` — לכל נושא עצמאי
- `global_<topic>.md` — כשהוא cross-module
- **ללא** V1/V2/date — זה בגיט
- גרסאות מנוהלות ב-**טבלת היסטוריה בתוך המסמך**

### סוגי בדיקות
- **QA** (Quality Assurance) — איכות, UX, נכונות, אמפתיה
- **TEST** — ביצועים, זיכרון, מהירות, עומס, stress
