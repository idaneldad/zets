# 🧠 ZETS AI Council — Model Registry

**Last updated:** 25.04.2026  
**Purpose:** Single source of truth for which AI models we consult, when, with what timeouts.  
**Location:** `docs/00_doctrine/MODEL_REGISTRY_20260425.md` — Claude reads this at start of every session.

---

# 🔑 Active Credentials (in /home/dinio/.env)

| Provider | Key var | Status | Notes |
|---|---|---|---|
| OpenAI | `OPENAI_API_KEY` | ✅ Working | GPT-5.5 + GPT-4o tested |
| Anthropic | `ANTHROPIC_API_KEY` | ✅ Working | Claude Opus 4.5 tested |
| Google | `GEMINI_API_KEY` | ❌ Invalid | Need new key — Gemini 3.1+ wanted |
| Together.ai | `TOGETHER_API_KEY` | ⚠️ Pending | Idan provided 25-char key, rejected as invalid. Need 64+ char `tgp_v1_*` key |
| Groq | `GROQ_API_KEY` | 🚫 Blocked | CF403 from server |

---

# 🥇 TIER 1 — Default Council (use for all major decisions)

## **Claude Opus 4.7** (this conversation)
- **Cost:** Premium
- **Strength:** Architectural reasoning, careful synthesis, anti-flattery
- **Use for:** All architectural decisions, shvirat kelim, Idan-facing reasoning
- **Timeout:** 120s for complex prompts
- **API:** Anthropic /v1/messages, model `claude-opus-4-5` for direct API calls

## **GPT-5.5 / 5.4** (OpenAI)
- **Cost:** Premium
- **Strength:** Broad knowledge, strong reasoning, fast
- **Use for:** Second opinion on architecture, code review
- **Timeout:** 60s for standard, 120s for reasoning prompts
- **API:** model `gpt-5.5` (works) or `gpt-4o` (works), NOT `gpt-5.5-pro` (returns 404)
- **Param note:** Use `max_completion_tokens` for gpt-5.x, `max_tokens` for gpt-4o

## **Gemini 3.1 Pro** ⏳ pending key
- **Use for:** Third perspective, especially good at long-context analysis
- **Timeout:** 90s
- **Endpoint:** `https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-pro:generateContent`
- **Status:** Need new API key from Idan

---

# 🥈 TIER 2 — Together.ai (when key valid)

⏳ **All Together models pending valid `TOGETHER_API_KEY`.**

Endpoint: `https://api.together.xyz/v1/chat/completions`  
**Header required:** `User-Agent: Mozilla/5.0` (Cloudflare blocks default Python urllib)

## **DeepSeek R1-0528** — `deepseek-ai/DeepSeek-R1-0528`
- **Cost:** $3.00 / $7.00 per M tokens
- **Strength:** Reasoning with visible chain-of-thought in `<think>` tags
- **Use for:** Architectural deep-dives — when we need to SEE the reasoning
- **Timeout:** **180s** (reasoning models slow; CoT eats tokens)
- **Best for ZETS gaps:** #14 Planner, #11b TMS deep, #17 Analogical Transfer

## **DeepSeek V4 Pro** — `deepseek-ai/DeepSeek-V4-Pro`
- **Cost:** $2.10 / $4.40
- **Strength:** Distributed systems, refactoring, MoE architecture trained on diverse data
- **Use for:** Code architecture critique, ZETS mmap/CSR design review
- **Timeout:** 90s
- **Best for ZETS gaps:** #2 Edge Schema, #3 Compression, #18 Cache layout

## **Kimi K2.6 FP4** — `moonshotai/Kimi-K2.6-FP4`
- **Cost:** $1.20 / $4.50
- **Strength:** **Long-context** (huge window) + multi-modal + tool calling
- **Use for:** Reading entire AGI.md (3,988 lines) + status docs in single prompt
- **Timeout:** **180s** (long context = slow)
- **Best for ZETS gaps:** Anything requiring full-corpus analysis, audit of consistency across docs

## **Qwen3.5 397B A17B** — `Qwen/Qwen3.5-397B-A17B`
- **Cost:** $0.60 / $3.60
- **Strength:** **201 languages including native Hebrew**, hybrid thinking mode
- **Use for:** Hebrew NLP tasks, multilingual common-sense
- **Timeout:** 90s
- **Best for ZETS gaps:** **#4 Hebrew Bridge** (alt to AlephBert), **#13 Common Sense**, **#16 NL Realization**, **#19 Morphology**

## **Qwen3 Coder 480B** — `Qwen/Qwen3-Coder-480B-A35B-Instruct-FP8`
- **Cost:** $2.00 (input only)
- **Strength:** Pure code specialist, agentic coding, deep generation
- **Use for:** When we move to actual Rust implementation
- **Timeout:** 60s
- **Best for ZETS gaps:** Phase A coding work (#22 ParseAtom, #2 Edge Schema impl)

## **Cogito v2.1 671B** — `deepcogito/cogito-v2-671b`
- **Cost:** $1.25
- **Strength:** Systemic reasoning, trade-off analysis, NFR analysis
- **Use for:** Third judge when GPT vs Claude disagree
- **Timeout:** **180s** (671B reasoning = slow)
- **Best for ZETS gaps:** Tie-breaking decisions, #11b TMS architecture

## **GLM 5.1 FP4** — `Zai-org/GLM-5.1-FP4`
- **Cost:** $1.40 / $4.40
- **Strength:** Academic/theoretical depth, CS fundamentals
- **Use for:** Theoretical justification (Gentner SME, Friston PP, Doyle TMS)
- **Timeout:** 90s
- **Best for ZETS gaps:** #6 Global Workspace (Baars/Dehaene), #17 Analogical Transfer (Gentner)

## **Llama 3.3 70B Turbo** — `meta-llama/Llama-3.3-70B-Instruct-Turbo`
- **Cost:** $0.88
- **Strength:** Fast, cheap, open-source
- **Use for:** Quick consultations, debugging, batch processing
- **Timeout:** 30s
- **Best for ZETS gaps:** #13 Common-Sense bulk enrichment (cheapest path)

---

# 🎯 Gap → Model Mapping

For consultation on each unbroken gap, use these specific models:

| Gap | Primary | Secondary | Tertiary | Why |
|---|---|---|---|---|
| **#2** Edge Schema | DeepSeek V4 Pro | Cogito 671B | Claude Opus | Schema design = systemic |
| **#3** Compression | DeepSeek V4 Pro | GPT-5.5 | Claude Opus | Algorithm + perf eval |
| **#5** Fuzzy Hopfield | DeepSeek R1 | GLM 5.1 | Claude Opus | Reasoning + theory |
| **#6** Global Workspace | GLM 5.1 | Cogito 671B | Claude Opus | Theoretical depth |
| **#9** Affective State | Claude Opus | Qwen3.5 | — | Trivial impl |
| **#11b** TMS Deep | DeepSeek R1 | Cogito 671B | GPT-5.5 | Hardest reasoning |
| **#12** Regression Suite | DeepSeek V4 Pro | GPT-5.5 | — | Standard practice |
| **#15** Learned Ranker | DeepSeek V4 Pro | Qwen3.5 | — | ML system design |
| **#16** NL Realization | Qwen3.5 | Claude Opus | GPT-5.5 | Hebrew quality matters |
| **#17** Analogical Transfer | GLM 5.1 | DeepSeek R1 | Claude Opus | Theoretical (Gentner) |
| **#19** Morphology Rules | Qwen3.5 | Claude Opus | — | Hebrew specialist |
| **#21** Code Quarantine | DeepSeek V4 Pro | Claude Opus | — | Already designed |

---

# ⏱️ Timeout Cheat Sheet

| Model class | Suggested timeout | Why |
|---|---|---|
| Standard chat (Llama, GPT-4o, Claude) | 60s | Fast inference |
| Premium chat (GPT-5.5, Claude Opus) | 90-120s | Larger models |
| Reasoning (R1, Cogito 671B, GPT-5.5 thinking) | **180s** | CoT generation eats tokens + time |
| Long-context (Kimi K2.6, Llama 4 Scout) | **180s** | Big input → slower processing |
| Code (Qwen3 Coder, GPT-5.5-Codex) | 90s | Generation can be long |

**Rule:** when in doubt, set 180s. Better to wait than retry.

---

# 🔁 Standard Consultation Pattern

For each gap requiring AI council:

```python
# 1. Send to 3 models in parallel (NOT sequential)
results = await asyncio.gather(
    call_claude_opus(prompt),       # 90s timeout
    call_deepseek_r1(prompt),       # 180s timeout (reasoning)
    call_qwen35(prompt),            # 90s timeout
)

# 2. Save raw responses to JSON
save_to_git(f'docs/40_ai_consultations/phenomenal/{gap_id}.json', results)

# 3. Shvirat kelim — Claude Opus synthesizes
synthesis = synthesize_breaking_pots(results)

# 4. Idan reviews synthesis, makes final call
```

---

# 📁 Where to Find Things

| What | Where |
|---|---|
| Model registry (this doc) | `docs/00_doctrine/MODEL_REGISTRY_20260425.md` |
| API keys | `/home/dinio/.env` (chmod 600) |
| Gap status | `docs/40_ai_consultations/20260424_OPEN_GAPS_STATUS.md` |
| Phenomenal synthesis | `docs/40_ai_consultations/20260425_PHENOMENAL_SYNTHESIS.md` |
| Hebrew gap explanation | `docs/40_ai_consultations/20260425_GAPS_HEBREW_DETAILED.md` |
| Raw consultation JSONs | `docs/40_ai_consultations/phenomenal/*.json` |
| Master spec | `docs/AGI.md` |
| Idan refinements log | `docs/40_ai_consultations/20260424_idan_refinements_session3.md` |

