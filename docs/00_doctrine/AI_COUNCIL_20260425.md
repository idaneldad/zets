# 🏛️ מועצת ה-AI — ZETS Architecture & Code Consultation

**תאריך:** 25.04.2026  
**מטרה:** מסמך מקור-אמת יחיד למי במועצה, איך פונים אליו, מתי משתמשים.  
**Claude קורא את המסמך הזה בתחילת כל סבב.**

---

# 🎯 העיקרון המרכזי

**לכל סוגיה מורכבת (ארכיטקטורה, פתרון פער, החלטה אסטרטגית) — 7 מודלים שונים במקביל.**

הרכב המועצה:
- **3 חובה** (Tier 1): Google + OpenAI + Anthropic — תמיד נכנסים
- **4 בחירה** (Tier 2): Claude בוחר לפי סוג הסוגיה מתוך Together.ai

---

# 🔑 כלל הגישה

**מודל שיש לנו אליו מפתח ישיר → תמיד דרך הישיר.**  
**רק מי שאין → דרך Together.ai.**

| Provider | גישה | סטטוס | תפקיד במועצה |
|---|---|---|---|
| **Anthropic** | ישיר (`api.anthropic.com`) | ✅ | חובה |
| **OpenAI** | ישיר (`api.openai.com`) | ✅ | חובה |
| **Google** | ישיר (`generativelanguage.googleapis.com`) | ✅ | חובה |
| **Together.ai** | ישיר (`api.together.xyz`) | ✅ | מאגר ל-Tier 2 |

---

# 🥇 Tier 1 — חברים קבועים (גישה ישירה)

## 1. **Claude Opus 4.7** (Anthropic)
- **API:** `https://api.anthropic.com/v1/messages`
- **Model ID:** `claude-opus-4-5`
- **Headers:** `x-api-key: $ANTHROPIC_API_KEY`, `anthropic-version: 2023-06-01`
- **מחיר:** $5 / $25 per M tokens
- **Timeout:** 120s
- **תפקיד:** Chief architect, סינתזה, החלטות עמוקות
- **דירוג ארכיטקט/קוד:** 9.8 / 9.7
- **הערה:** טוקנייזר חדש צורך עד 35% יותר tokens

## 2. **GPT-5.5** (OpenAI)
- **API:** `https://api.openai.com/v1/chat/completions`
- **Model ID:** `gpt-5.5`
- **Header:** `Authorization: Bearer $OPENAI_API_KEY`
- **Param:** `max_completion_tokens` (לא `max_tokens`)
- **מחיר:** $5 / $30
- **Timeout:** 90s (120s ל-reasoning prompts)
- **תפקיד:** ידע רחב, system thinking, second opinion
- **דירוג:** 9.7 / 9.6
- **חלופה זולה:** `gpt-5.4` ($2.50/$15), `gpt-5.4-mini` ($0.75/$4.50)

## 3. **Gemini 3.1 Pro Preview** (Google)
- **API:** `https://generativelanguage.googleapis.com/v1beta/models/gemini-3.1-pro-preview:generateContent?key=$GEMINI_API_KEY`
- **Model ID:** `gemini-3.1-pro-preview`
- **מחיר:** $2 / $12 (עד 200K context)
- **Timeout:** 90s
- **תפקיד:** multimodal, long-context, perspective ייחודית
- **דירוג:** 9.3 / 9.1
- **חלופה זולה יותר:** `gemini-2.5-pro` ($1.25/$10), `gemini-2.5-flash` ($0.30/$2.50)

---

# 🥈 Tier 2 — מאגר השלמה (דרך Together.ai)

**Endpoint:** `https://api.together.xyz/v1/chat/completions`  
**Header:** `Authorization: Bearer $TOGETHER_API_KEY`  
**חובה:** `User-Agent: Mozilla/5.0` (Cloudflare חוסם בלי זה)

Claude בוחר 4 מתוך הרשימה לפי סוג הסוגיה:

| מודל | Model ID | מחיר | Timeout | חוזק |
|---|---|---|---|---|
| **DeepSeek R1-0528** | `deepseek-ai/DeepSeek-R1-0528` | $3/$7 | **180s** | Reasoning עמוק עם CoT גלוי |
| **DeepSeek V4 Pro** | `deepseek-ai/DeepSeek-V4-Pro` | $2.10/$4.40 | 120s | Distributed systems, refactoring |
| **Kimi K2.6** | `moonshotai/Kimi-K2.6` | $1.20/$4.50 | **180s** | Long-context, codebase שלם |
| **Qwen 3.5 397B** | `Qwen/Qwen3.5-397B-A17B` | $0.60/$3.60 | 90s | **עברית native (201 שפות)** |
| **Qwen3 Coder 480B** | `Qwen/Qwen3-Coder-480B-A35B-Instruct-FP8` | $2.00 | 90s | קוד ייעודי בלבד |
| **Qwen3 Coder Next** | `Qwen/Qwen3-Coder-Next-FP8` | $0.50/$1.20 | 60s | קוד יומיומי |
| **GLM 5.1** | `zai-org/GLM-5.1` | $1.40/$4.40 | 90s | עומק תיאורטי, CS fundamentals |
| **MiniMax M2.7** | `MiniMaxAI/MiniMax-M2.7` | $0.30/$1.20 | 90s | **Value Hero** — איזון מצוין |
| **Cogito v2.1 671B** | `deepcogito/cogito-v2-1-671b` | $1.25 | **180s** | Trade-off analysis, שופט שלישי |
| **DeepSeek V3.1** | `deepseek-ai/DeepSeek-V3.1` | $0.60/$1.70 | 90s | Debugging, edge cases |
| **GPT-OSS 120B** | `openai/gpt-oss-120b` | $0.15/$0.60 | 60s | Specs, ניתוח ראשוני זול |
| **Llama 3.3 70B Turbo** | `meta-llama/Llama-3.3-70B-Instruct-Turbo` | $0.88 | 30s | Fast & cheap fallback |

---

# 🎯 איך Claude בוחר את 4 השלמת המועצה

## כלל אצבע: 7 מודלים ל-7 perspectives שונות

### לסוגיה ארכיטקטונית מורכבת
3 חובה + 4 בחירה:
1. Claude Opus 4.7 (חובה)
2. GPT-5.5 (חובה)
3. Gemini 3.1 Pro (חובה)
4. **DeepSeek R1-0528** — reasoning עמוק
5. **GLM 5.1** — עומק תיאורטי
6. **Cogito 671B** — trade-off analysis
7. **MiniMax M2.7** — second opinion זול

### לסוגיה הקשורה לעברית/morphology
3 חובה + 4 בחירה:
4. **Qwen 3.5 397B** — Hebrew native
5. **DeepSeek R1** — reasoning
6. **GLM 5.1** — תיאוריה לשונית
7. **Kimi K2.6** — context ארוך

### לסוגיה של אלגוריתמים/data structures
3 חובה + 4 בחירה:
4. **DeepSeek R1** — reasoning
5. **DeepSeek V4 Pro** — distributed systems
6. **GLM 5.1** — fundamentals
7. **Cogito 671B** — trade-offs

### לכתיבת/ביקורת קוד מורכב
3 חובה + 4 בחירה:
4. **Qwen3 Coder 480B** — code specialist
5. **DeepSeek V4 Pro** — refactoring
6. **GPT-5.3-Codex** (ישיר OpenAI) — agentic coding
7. **MiniMax M2.7** — value second opinion

### לסוגיה הדורשת context גדול (קובץ שלם, מספר מסמכים)
3 חובה + 4 בחירה:
4. **Kimi K2.6** — long context flagship
5. **DeepSeek R1** — reasoning
6. **Qwen 3.5 397B** — long context גם
7. **MiniMax M2.7** — value

### לסוגיה אקדמית/תיאורטית עמוקה
3 חובה + 4 בחירה:
4. **GLM 5.1** — academic depth
5. **Cogito 671B** — systemic reasoning
6. **DeepSeek R1** — visible CoT
7. **Qwen 3.5 397B** — multilingual perspective

---

# ⏱️ Timeout Cheat Sheet

| סוג מודל | Timeout מומלץ | למה |
|---|---|---|
| Standard chat (Llama, GPT-4o) | 60s | Inference מהיר |
| Premium chat (Opus, GPT-5.5) | 90-120s | מודלים גדולים |
| **Reasoning (R1, Cogito 671B, GPT-5.5 thinking)** | **180s** | CoT צורך הרבה tokens+זמן |
| **Long-context (Kimi K2.6)** | **180s** | Input גדול = איטי |
| Code specialists (Qwen Coder, Codex) | 90s | Generation יכולה להיות ארוכה |

**כלל ברירת מחדל:** במצב ספק — תן 180s. עדיף לחכות מאשר לעשות retry.

---

# 📊 דירוג שימושי — מסקנות מהטבלה של עידן

## הכי חזק נטו
**Claude Opus 4.7** (9.8/9.7)

## הכי משתלם לפיתוח שוטף
**Claude Sonnet 4.6** ($3/$15) — 9.5/9.5 בדירוג, חלופה לעבודה יום-יום ב-CHOOZ

## הכי טוב לקוד אג'נטי
**GPT-5.3-Codex** (`gpt-5.3-codex`, $1.75/$14)

## ה-Value Hero ב-Together
**MiniMax M2.7** ($0.30/$1.20) — 9.0/9.0 דירוג, **9.7 Value 4 Money**

## הכי זול לעיבוד בכמות
**Gemini 3.1 Flash-Lite** ($0.25/$1.50) או **LFM2-24B-A2B** ($0.03/$0.12)

---

# 🔧 פטרן ההפעלה הסטנדרטי

```python
import json, subprocess
from concurrent.futures import ThreadPoolExecutor, as_completed

def run_council(prompt, gap_type='architecture'):
    """Run 7-model parallel consultation."""
    
    # Tier 1 — always
    council = [
        ('claude_opus47', call_anthropic, 120),
        ('gpt55',         call_openai,    120),
        ('gemini31pro',   call_gemini,    90),
    ]
    
    # Tier 2 — based on type
    tier2 = pick_tier2_models(gap_type)  # 4 models
    council.extend(tier2)
    
    # Parallel execution
    with ThreadPoolExecutor(max_workers=7) as pool:
        futures = {
            pool.submit(fn, prompt, timeout=t): name 
            for name, fn, t in council
        }
        results = {}
        for f in as_completed(futures):
            name = futures[f]
            results[name] = f.result()
    
    # Save raw to git
    save_json(f'docs/40_ai_consultations/<gap>/{date}.json', results)
    
    # Synthesis (shvirat kelim)
    return synthesize(results)
```

---

# 📁 איפה למצוא דברים

| מה | איפה |
|---|---|
| המסמך הזה (Council registry) | `docs/00_doctrine/AI_COUNCIL_20260425.md` |
| מפתחות API | `/home/dinio/.env` (chmod 600) |
| מסמך פערים מלא | `docs/40_ai_consultations/20260424_OPEN_GAPS_STATUS.md` |
| סינתזה V1 (6 קריטיים) | `docs/40_ai_consultations/20260425_PHENOMENAL_SYNTHESIS.md` |
| סינתזה V2 (4 קריטיים) | `docs/40_ai_consultations/20260425_PHENOMENAL_SYNTHESIS_V2.md` |
| Raw consultation responses | `docs/40_ai_consultations/phenomenal*/` |
| Master spec | `docs/AGI.md` |

---

# 🚦 סטטוס מפתחות (25.04.2026)

```
ANTHROPIC_API_KEY  ✅ עובד   (Claude Opus 4.7 דרך claude-opus-4-5)
OPENAI_API_KEY     ✅ עובד   (GPT-5.5, GPT-5.4, GPT-4o)
GEMINI_API_KEY     ✅ עובד   (Gemini 3.1 Pro Preview, 2.5-pro, 2.5-flash)
TOGETHER_API_KEY   ✅ עובד   (50 chars, tgp_v1_*)
GROQ_API_KEY       🚫 חסום   (CF403 מהשרת)
```

