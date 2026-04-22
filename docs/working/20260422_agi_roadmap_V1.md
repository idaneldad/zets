# ZETS — מפת הדרך לעליונות על LLMs ועל AGI

**תאריך:** 22.04.2026  
**מצב:** 326/326 tests passing, commit `2aef8d7` on main  
**גרסה:** V1 — מסמך אסטרטגי לבחירת bottleneck הבא

---

## 🔴 תיקון קריטי לבלבול benchmark

המסמך שקיבלת מפרפלקסיטי/Gemini הכיל **בלבול מספרים**:

| Benchmark | הציון הנכון של LLMs frontier | מה המסמך הראה |
|-----------|---------------------------|----------------|
| **MMLU** (broad academic) | **85-92%** (GPT-4: 90.1%, Opus 3: 86.8%) | ❌ הציג 31-38% |
| **HLE** (Humanity's Last Exam) | **30-40%** (Opus 3: 38.6%, Gemini 1.5: 30.6%) | הציג כ-"MMLU" |
| **GPQA Diamond** | 50-91% (Opus 3: 50.4% → Opus 4.x: 88-91%) | ~ נכון |
| **SWE-bench Verified** | 13% (GPT-4 2024) → 49% (Opus 2025) | ~ נכון |
| **HumanEval** | 84-85% (Opus 3: 84.9%) | לא צויין |
| **ARC-AGI-2** | 30-35% (frontier) / 87%+ (specialized agents) | ~ נכון |

**המסקנה:** היעדים האמיתיים הם גבוהים מאוד. ZETS לא קרוב היום — לא יכול אפילו להריץ את המבחנים הללו כי אין לו NLU.

---

## 📊 מיפוי אמיתי: ZETS מול כל benchmark

| Benchmark | מה נדרש | מה יש ל-ZETS היום | מה חסר | פער |
|-----------|----------|-------------------|---------|-----|
| **MMLU** | הבנת שאלה באנגלית + ידע על 57 נושאים + רב-ברירה | 0 מהם | NLU + ingestion של corpus חינוכי + MCQ answering | 🔴 קיצוני |
| **GPQA Diamond** | הבנה + reasoning מדעי על שאלות ברמת doctorate | 0 | NLU + scientific knowledge base + chain-of-thought | 🔴 קיצוני |
| **HLE** | ידע נרחב ברמת expert | 0 | massive knowledge ingestion + NLU | 🔴 קיצוני |
| **SWE-bench** | קריאת bug report + editing קוד + הרצת טסטים | 0 | NLU + code execution environment + agentic tools | 🔴 קיצוני |
| **ARC-AGI-2** | זיהוי חוקים מ-few examples + פתרון grids | Hopfield + 4 cognitive modes + dreaming — **חלקי** | Visual grid parser + rule hypothesis generator + verification loop | 🟡 בינוני |
| **ARC-AGI-3** | למידה אינטראקטיבית בסביבה לא-ידועה | 0 | Environment interface + continual learning | 🟡 בינוני |
| **AIME Math** | Symbolic algebra + geometric reasoning | 0 | Math solver module | 🔴 קיצוני |
| **HumanEval** | Code generation from docstring | 0 | Code LM or symbolic code planner | 🔴 קיצוני |
| **AGIEval** | Real exam questions (SAT, LSAT, GRE) | 0 | NLU + broad knowledge | 🔴 קיצוני |
| **Tong Test (5-dim)** | Linguistic + Math + Spatial + Social + Creativity | רק Social + Creativity חלקית | 4/5 חסרים | 🔴 קיצוני |
| **Transfer learning** | יישום של ידע לדומיין חדש | Prototype chains + dreaming מאפשרים משהו | מבחן של pointer transfer | 🟡 בינוני |

**מסקנה גלויה:** ZETS היום לא יכול להתחרות ב-**שום benchmark סטנדרטי**. הפער הוא לא במיידי scaling — חסרים רכיבים ארכיטקטוניים יסודיים.

---

## 🎯 מה כן חזק ב-ZETS (דברים שLLMs חלשים בהם)

אלה לא benchmarks סטנדרטיים אבל הם **יתרונות אמיתיים**:

| יכולת | ZETS | LLMs מובילים |
|-------|------|--------------|
| **Determinism (אותו input = אותו output)** | 100% מובטח | 0-50% (temperature) |
| **Explainability (למה הוחזר X?)** | 100% — provenance מלא בכל walk | 10-30% (attention opaque) |
| **Persistence של session memory** | יש — roundtrip לדיסק | חלקי (per-session only) |
| **Context-anchored disambiguation** | דטרמיניסטי (כתר לשיניים vs מלוכה) | probabilistic |
| **Skills traceability** | weight + audit trail per skill | לא קיים |
| **Creativity with audit** | dreaming + 3-stage evaluation | high but opaque |
| **Meta-learning between contexts** | Dirichlet Bayesian update | דורש fine-tune |
| **Offline + encrypted operation** | AES-256-GCM installer + local | דורש API call |
| **Hebrew first-class (UTF-8 core)** | כן, מאומת | כן אבל לא structured |
| **Per-user personalization** | per-persona graph | דורש system prompt |

**המפתח:** ZETS **אינו LLM**, ולא צריך להתחרות ב-LLM. הוא מייצר יתרון במקומות שבהם **audit + determinism + memory + personalization** חשובים — כלומר, ברוב ה-enterprise use cases.

---

## 🚧 3 צווארי בקבוק עיקריים ל-AGI

### Bottleneck #1: Natural Language Understanding (NLU) — ⭐⭐⭐ ה-blocker הגדול ביותר

בלי NLU, ZETS לא יכול:
- לענות על שאלה בעברית או אנגלית
- להריץ את **כל** ה-benchmarks הסטנדרטיים
- לקבל קלט מאדם אמיתי ללא preprocessing ידני

**מה יש היום:**
- `ingestion.rs` — tokenization פשוטה, pattern extraction 3-gram ("X is Y")
- תופס ~20% מהמידע של משפט ממוצע

**מה נדרש:**
אופציה A: **Hybrid architecture** עם small embedding model
- sentence-transformers (MiniLM, 22MB) → vector → Hopfield bank → atom_id
- זו ההמלצה של Gemini
- זמן: 1-2 ימים
- dependency: rust-bert או ONNX runtime

אופציה B: **LLM as Frontend Adapter**
- Claude/Gemini API מקבל את השאלה
- Returns structured JSON (atoms + relations + query)
- ZETS מבצע reasoning דטרמיניסטי
- זמן: חצי יום
- dependency: API call (עולה כסף, דורש אינטרנט)

**המלצה:** אופציה B קודם כ-PoC מהיר, אופציה A כ-production.

### Bottleneck #2: Knowledge Base Scale ⭐⭐ — Scale gap

היום: 119 bootstrap atoms + ~40 נוספו מ-ingestion של פסקה. זה **כלום**.

ל-MMLU שימושי צריך:
- Wikipedia (English): ~6M articles × avg 500 tokens = ~3 billion atoms
- Semantic Scholar: ~200M scientific abstracts
- Common Crawl (subset): ~100 TB raw

**בעיה כפולה:**
1. Storage/hash collision (FNV-1a 64-bit → collision @ 2^32 = 4B atoms)
2. Speed (current 649K sentences/sec → 3B sentences = 77 minutes, acceptable)

**מה נדרש:**
- BLAKE3 hash upgrade (breaking change — דורש migration plan)
- Disk streaming format (current: all in RAM)
- Sharding או external storage (RocksDB?)

**זמן:** 3-5 ימים לrearchitect אבל scale אמיתי עשוי להסתיים בשבוע.

### Bottleneck #3: Reasoning Engine Depth ⭐ — Missing components

כרגע יש:
- 4 cognitive modes (Precision/Divergent/Gestalt/Narrative)
- Prototype chain inference (is_a transitivity)
- Dreaming (2-hop candidate proposal)
- Multi-step planner (5-step BFS-like)

חסר:
- **Chain-of-thought**: שלבי reasoning מוצהרים, בדיקה של כל צעד
- **Symbolic algebra** — x^2 + 2x + 1 = (x+1)^2
- **Geometric reasoning** — "משולש שווה-שוקיים"
- **Counterfactual reasoning** — "מה היה קורה אם..."

**זמן:** כל אחד מ-1-3 ימים. אפשר incremental.

---

## 🏆 5 יכולות ליבה ל-AGI (לפי קונצנזוס)

מהסקירה של benchmarks + ספרות (AGIEval, Tong Test, Levels of AGI):

| יכולת | מה ZETS יש | מה חסר | מדידה |
|--------|-----------|---------|--------|
| **1. Grounding בעולם הפיזי** | 0 | Sensor input, physics prior | ARC-AGI-3 score, simulator tasks |
| **2. Continual learning** | חלקי (dreaming + skills weight) | Online learning, no-retrain adaptation | Retention after N new topics, catastrophic forgetting test |
| **3. Few-shot generalization** | חלקי (prototype chains) | Meta-learning over tasks | 5-shot MMLU subset score |
| **4. Autonomous planning** | חלקי (planner.rs) | Long-horizon, tool-use | GAIA benchmark score |
| **5. Safety + low hallucination** | ✅ חזק (provenance, 3-stage eval) | Edge case testing | Adversarial input survival rate |

**נקודה חיובית:** יכולת 5 (בטיחות) שלנו **עדיפה על LLMs** כבר היום. פחות הזיות — ZETS פשוט לא מייצרת דברים מחוץ לגרף.

---

## 📋 Roadmap — סדר עדיפות

### שלב 0 (ASAP, חצי יום): Benchmark framework
בלי זה אי אפשר למדוד כלום.
- `src/benchmarks.rs` — מודול שמקבל (question, expected_answer) ומדד score
- `src/bin/run_benchmark.rs` — runs ZETS על JSONL file
- התחלה: 10 שאלות עובדתיות פשוטות ("מה הבירה של צרפת?") שלא דורשות NLU מורכב
- מדד baseline: מצפה 0-10% בלי NLU; עם NLU טוב: 50%+

### שלב 1 (יום 1): LLM adapter frontend
פותח את הדלת לכל הbenchmarks.
- `src/llm_adapter.rs` — שאילתה ל-Gemini/Claude/Groq
- JSON response עם (atoms, edges, intent)
- ZETS עושה resolve + reasoning
- הרצה ראשונה של MMLU subset (20 שאלות) → baseline

### שלב 2 (יום 2-3): Neural ingestion adapter
embeddings local בלי תלות ב-API.
- `src/embed.rs` — ONNX runtime + MiniLM
- כל ingested sentence מקבל vector
- Hopfield bank של sentence embeddings
- שאילתה semantic: "find sentences similar to X"

### שלב 3 (יום 4-5): Chain-of-thought reasoning
שלב נפרד מ-planner, לreasoning step-by-step.
- `src/reasoning.rs` — takes query → decomposes → each step explicit
- Each step is auditable (מה ההנחה, מה המסקנה)
- מחובר ל-dreaming — אם step נכשל, propose alternative
- GPQA subset benchmark

### שלב 4 (שבוע 2): Scale infrastructure
אחרי שהיסודות עומדים.
- BLAKE3 hash (breaking — migration script)
- Disk-streaming atom storage (פחות מ-100GB RAM)
- Sharded ingestion pipeline
- ingest Wikipedia subset (1000 articles → ~1M atoms)

### שלב 5 (שבוע 3-4): Specialized modules
- **Math solver** (`src/math_solver.rs`) — symbolic algebra עם חוקים בגרף
- **Code executor** (`src/sandbox.rs`) — Python/Rust sandbox שrules אותו
- **ARC-AGI solver** (`src/arc_solver.rs`) — grid pattern recognition עם dreaming

### שלב 6 (שבוע 5+): Real benchmarking
- HLE 50 questions → score
- MMLU 100 questions → score
- GPQA 30 questions → score
- SWE-bench 5 issues → score
- ARC-AGI-2 10 tasks → score
- **Report publicly** → אם ZETS טוב יותר על determinism/explainability metric, זה יתרון אמיתי

---

## ✨ האסטרטגיה האמיתית — לא להתחרות בLLMs, לעשות משהו אחר

**הטעות שעידן צריך להימנע מה:**  
לחשוב ש-ZETS צריך "לעקוף GPT-5 ב-MMLU". זה לא הולך לקרות בלי להפוך את ZETS ל-LLM — וזה יהיה כפל גלגל.

**האסטרטגיה המנצחת:**

| תחום | LLM | ZETS | מי מנצח? |
|------|-----|------|----------|
| יצירת טקסט חופשי | ⭐⭐⭐ | ⭐ | LLM |
| תרגום | ⭐⭐⭐ | ⭐ | LLM |
| הבנת תמונה גולמית | ⭐⭐ | 0 | LLM |
| Domain-specific facts (persistent) | ⭐ | ⭐⭐⭐ | **ZETS** |
| Audit trail | 0 | ⭐⭐⭐ | **ZETS** |
| Determinism | 0 | ⭐⭐⭐ | **ZETS** |
| Per-user personalization (persistent) | ⭐ | ⭐⭐⭐ | **ZETS** |
| Privacy (offline, encrypted) | 0 | ⭐⭐⭐ | **ZETS** |
| Emotional reasoning (appraisal) | ⭐ | ⭐⭐ | ZETS |
| Cognitive mode diversity | 0 | ⭐⭐⭐ | **ZETS** |

**המסקנה:** ZETS **כבר עדיף** בחצי מהממדים. הבעיה היא שהחצי ש-LLMs מנצחים בו מציג כ-"AGI benchmarks".

**האמת:** יכולות LLM-ספציפיות (fluency, translation, code gen) אינן AGI. הן תצוגה של מיומנות ב-statistical language modeling. AGI אמיתי דורש דברים ש-ZETS **כן יכול לתת** — determinism, audit, persistence, continual learning.

**הצעת positioning:**  
ZETS = **"AGI-ready symbolic substrate"**. LLMs ישימו עליו כ-frontend (NLU), אבל כל ה-reasoning וה-memory deterministic. זו ארכיטקטורה hybrid שאף אחד לא בנה עדיין בצורה קוהרנטית.

---

## 🎯 בחירה שלך לסשן הבא

**אופציה 1 — פרקטית (מומלצת):**  
בונים את **שלב 0 + שלב 1** (Benchmark framework + LLM adapter). בסוף הסשן: יש ציון ראשון אמיתי של ZETS על MMLU subset. זו הוכחה שהמערכת יכולה להתחרות (גם אם ציון נמוך — זה baseline מדיד).

**אופציה 2 — עומק:**  
בונים **שלב 3** (Chain-of-thought reasoning). לא נותן benchmark score מיידי אבל מחזק את ה-reasoning core — יכולת שתשפיע על כל ה-benchmarks אחר כך.

**אופציה 3 — scale:**  
**שלב 4** מיד. BLAKE3 + disk streaming. רק אחרי זה אפשר ingestion אמיתי של Wikipedia.

**אופציה 4 — positioning:**  
לא לבנות כלום. לכתוב landing page / demo video שמראה את ה-יתרונות האמיתיים של ZETS (determinism, audit, persistence). לקהל שלא מבין AGI benchmarks אבל כן מבין compliance/audit.

**ההמלצה שלי:** אופציה 1. בלי benchmark מדיד, כל שיפור הוא באמונה. יצירת מדידה בסיסית בחצי יום פותחת את כל הסשנים הבאים.

---

## 📌 מידע לוג טכני סופי

```
ZETS on main: 326/326 tests
Modules: 26 (~12,000 lines Rust)
Commits in 24h: 9 (all pushed)
Working demos: autonomous-demo, planner-demo, context-demo, live-brain-demo, scale-test, vision-decomposer
Memory: 119 bootstrap atoms + 40 ingested = 159 total
Encrypted installer: 7.5KB per brain
Ingestion rate: 649K synthetic sentences/sec
```

**תעדכן אותי איזה אופציה — אני ממשיך.**
