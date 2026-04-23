# 🎨 תמונות — Image Composition

**Last updated:** 23.04.2026
**Current score:** 🟡 0.45 | **Target MVP:** 0.75 | **Gap:** −0.30

**באחריות:** `גרף + חיצוני` — ZETS בונה prompt מפורט (graph, motifs), Midjourney/SDXL/DALL-E מייצרים

## 🎯 משימה

זה **לא** "ZETS מצייר." זה "ZETS יודע **מה לבקש**."

דוגמה: "צור לי בולדוג חום עם פנים לבנות" → ZETS מרחיב ל-"photorealistic portrait of a brown bulldog with white facial markings, soft natural lighting, centered composition, detailed fur texture, warm color grading."

## ✅ הצלחה

- Specificity 0.80+ — prompt מפורט, לא שטוח
- Texture awareness — מציע textures (שיער ≠ צבע אחיד)
- Style consistency — 5 תמונות באותו style
- Negative prompt inference — מבין מה לא רוצה
- Iteration — "שוב, אבל עם background חוף" משמר constraints
- Cost-aware — model זול ל-draft, יקר לfinal

## 🔬 Tests (6)

| # | Test | סוג | יעד | סטטוס | מודול |
|---|------|:---:|:---:|:-----:|--------|
| 8.1 | Prompt specificity | QA | 0.80 | 🟡 0.55 | composition/motif_bank (ImagePrompt kind) |
| 8.2 | Texture/material awareness | QA | 0.80 | 🟠 0.25 | motif by_tag (חסרה texture lib) |
| 8.3 | Style consistency 5-set | QA | 0.85 | 🟡 0.50 | CompositionPlan.style_hints |
| 8.4 | Negative prompt | QA | 0.75 | 🟡 0.45 | CompositionPlan.must_exclude |
| 8.5 | Iteration w/ constraints | QA | 0.80 | 🟡 0.50 | plan constraints preserved |
| 8.6 | Cost-aware selection | TEST | 0.85 | 🟡 0.45 | plan.external_budget |

## 🏗️ באחריות

- **Composition** (`src/composition/`) — Motif + Plan + Weaver. **graph**
- **CapabilityOrchestrator** (עתיד) — שיקרא ל-Midjourney/SDXL/DALL-E API. **external**

## 📈 פער

- Texture library לא קיימת — צריך motif bank של materials
- Diffusion model runtime לא חובר

## היסטוריה
| תאריך | Score |
|:-----:|:-----:|
| 23.04 | 0.45 | Baseline. Composition layer חזק, runtime חסר |
