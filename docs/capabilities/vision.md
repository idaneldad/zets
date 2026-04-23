# 👁️ ראייה — Vision Understanding

**Last updated:** 23.04.2026
**Current score:** 🟠 0.06 | **Target MVP:** 0.75 | **Gap:** −0.69

**באחריות:** `גרף + חיצוני` — Hopfield פנימי לפירוק hierarchical; Gemini Vision / GPT-4V לdeep understanding

## 🎯 משימה

הבנת תמונות: detect objects, OCR, describe scenes, extract charts, analyze documents, visual QA.

## ✅ הצלחה

- Object detection (COCO 80): mAP 0.65+
- OCR עברית+אנגלית+מספרים: 0.95+
- Scene description: Human rating >4.0/5
- Document understanding: F1 0.85+
- Visual QA: 0.80+

## 🔬 Tests (8)

| # | Test | סוג | יעד | סטטוס | מודול |
|---|------|:---:|:---:|:-----:|--------|
| 7.1 | Object detection COCO | QA | 0.65 | 🟠 0.15 | `bin/vision_decomposer` — Hopfield partial |
| 7.2 | OCR HE+EN | QA | 0.95 | 🔴 0.00 | — |
| 7.3 | Face detection (not recog) | QA | 0.90 | 🔴 0.00 | — |
| 7.4 | Scene description | QA | 0.80 | 🟠 0.20 | `hopfield` bank hierarchy |
| 7.5 | Chart extraction | QA | 0.75 | 🔴 0.00 | — |
| 7.6 | Document understanding (invoice) | QA | 0.85 | 🔴 0.00 | — |
| 7.7 | Visual QA | QA | 0.80 | 🔴 0.00 | — |
| 7.8 | Sketch understanding | QA | 0.65 | 🟠 0.15 | hopfield |

## 🏗️ באחריות

- **Hopfield banks** (`src/hopfield.rs` + `src/bin/vision_decomposer.rs`) — hierarchical decomposition. **graph** (partial)
- **Gemini Vision / GPT-4V** — deep scene understanding. **external**

## 📈 פער

- אין OCR (Tesseract integration)
- אין Gemini Vision integration
- אין chart/document parsers

## היסטוריה
| תאריך | Score |
|:-----:|:-----:|
| 23.04 | 0.06 | Baseline. Hopfield vision_decomposer prototype only |
