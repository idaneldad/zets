# 🛠️ יכולות חיצוניות — External Capabilities

**Last updated:** 23.04.2026
**Last verified:** 23.04.2026

---

## מה המשימה

כשZETS צריך יכולת שלא קיימת ב-graph (diffusion, speech, deep vision), הוא מתזמר API חיצוני. המסמך הזה מרכז את כל ה-integration — מה חיבור עובד, מה זמין כ-API, מה חסר.

---

## 🎯 סטטוס integration

| יכולת | שירות | זמין כ-API | wired ל-ZETS | נבדק | מיועד לCategory |
|-------|-------|:----------:|:------------:|:-----:|:---------------:|
| Speech → Text | OpenAI Whisper | ✅ | ❌ | — | Cat 6 |
| Speech → Text | Google Speech | ✅ | ❌ | — | Cat 6 |
| Text → Speech | Gemini TTS | ✅ | ❌ | — | Cat 6 |
| Text → Speech | ElevenLabs | ✅ | ❌ | — | Cat 6 |
| Speaker diarization | pyannote.audio | ✅ (local) | ❌ | — | Cat 6 |
| Vision → Understanding | Gemini Vision | ✅ | ❌ | — | Cat 7 |
| Vision → Understanding | GPT-4V | ✅ | ❌ | — | Cat 7 |
| OCR | Tesseract | ✅ (local) | ❌ | — | Cat 7 |
| Image generation | Midjourney | ✅ (via API) | ❌ | — | Cat 8 |
| Image generation | Stable Diffusion | ✅ (local/API) | ❌ | — | Cat 8 |
| Image generation | DALL-E 3 | ✅ | ❌ | — | Cat 8 |
| Music generation | Suno | ✅ | ❌ | — | Cat 9 |
| Music generation | Udio | ✅ | ❌ | — | Cat 9 |
| Video generation | Sora | ✅ (limited) | ❌ | — | Cat 10 |
| Video generation | Runway Gen-3 | ✅ | ❌ | — | Cat 10 |
| LLM completion | Gemini Flash | ✅ | 🟡 partial | 23.04 | Cat 1, 2, 11, 12 |
| LLM completion | Claude | ✅ | 🟡 partial | 23.04 | Cat 1, 2, 11, 12 |
| LLM completion | GPT-4o | ✅ | 🟡 partial | 23.04 | Cat 1, 2, 11, 12 |

---

## 📋 מה קיים ב-ZETS

### LLM Adapter (`src/llm_adapter.rs`)

יש `QuestionParse` struct שמגדיר את החוזה:
- Input: natural language question
- Output: JSON `{intent, key_terms, expected_answer_type}`

**מטרה:** LLM עושה רק parsing. ZETS שומר על reasoning דטרמיניסטי.

**מצב:** 15 tests passing, חלקית wired. אין end-to-end HTTP לGemini עדיין בעבודה רגילה.

### Gemini HTTP (`src/gemini_http.rs`)

קובץ קיים, מימוש חלקי. זה ה-building block ל-runtime execution.

---

## 🔌 מה חסר — ה-CapabilityOrchestrator

רוב היכולות החיצוניות מחכות למודול **אחד** שמחבר הכל:

```rust
// Missing: src/capability_runtime/
pub struct CapabilityOrchestrator {
    secrets: VaultRef,
    error_store: ErrorStoreRef,
    budget_tracker: BudgetTracker,
    rate_limiter: RateLimiter,
}

impl CapabilityOrchestrator {
    pub async fn invoke(&self, capability: &str, args: Value) -> Result<Value>;
}
```

פעם שקיים, כל ה-integration החסרה (Whisper, Gemini Vision, Midjourney) נכנסת אליו.

**זמן משוער:** 3-4 ימי עבודה.
**גיין צפוי ל-HumannessScore:** +0.12 (פותח את Cat 6-10)

---

## 📁 Secrets Management

המפתחות לAPIs הללו מנוהלים דרך `src/secrets/` (Vault). ראה מצב ב-[connectors.md](connectors.md#secrets-management).

מפתחות שעידן אוחז פיזית:
- Gemini API key
- OpenAI API key (ב-`/home/dinio/.env`, chmod 600)
- Anthropic (Claude Max + API)
- GitHub token
- Brevo SMTP + Gmail App Password
- GreenAPI (WhatsApp)

---

## בדיקות QA + TEST

| Test | סוג | סטטוס |
|------|:---:|:-----:|
| llm_adapter parse Hebrew | QA | ✅ 23.04 |
| gemini_http basic call | QA | 🟠 partial |
| (missing) whisper integration e2e | QA+TEST | 🔴 |
| (missing) latency per capability call | TEST | 🔴 |
| (missing) rate limit per tier | TEST | 🔴 |
| (missing) cost tracking | TEST | 🔴 |

---

## היסטוריית שינויים

| תאריך | שינוי |
|:-----:|-------|
| 23.04.2026 | Audit ראשון. 0/10+ capabilities wired. Orchestrator חסר |
