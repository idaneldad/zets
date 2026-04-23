# 🎤 קול — Speech I/O

**Last updated:** 23.04.2026
**Current score:** 🔴 0.00 | **Target MVP:** 0.85 | **Gap:** −0.85

**באחריות:** `חיצוני` — Whisper (STT) + Gemini/ElevenLabs (TTS). ZETS מתזמר

## 🎯 משימה

קליטת דיבור → טקסט, והפקת דיבור מטקסט. כולל speaker diarization, emotion, voice cloning (עם רשות).

## ✅ הצלחה

- Hebrew STT: WER <0.15
- English STT: WER <0.10
- Speaker diarization (2-5): DER <0.20
- TTS A/B vs human: 40%+ prefer AI
- Emotion in speech: F1 0.70+
- Voice cloning similarity: 0.80+

## 🔬 Tests (6)

| # | Test | סוג | יעד | סטטוס | מודול |
|---|------|:---:|:---:|:-----:|--------|
| 6.1 | Hebrew STT | QA | 0.85 | 🔴 0.00 | — |
| 6.2 | English STT | QA | 0.90 | 🔴 0.00 | — |
| 6.3 | Speaker diarization | QA | 0.80 | 🔴 0.00 | — |
| 6.4 | TTS natural | QA | 0.40 | 🔴 0.00 | — |
| 6.5 | Emotion in speech | QA | 0.70 | 🔴 0.00 | — |
| 6.6 | Voice cloning | QA | 0.80 | 🔴 0.00 | — |

## 🏗️ באחריות

**כולו חיצוני.** ZETS צריך להיות ה-orchestrator:
- Whisper API (OpenAI) — Hebrew + English
- Gemini TTS / ElevenLabs — natural speech synthesis
- pyannote.audio — speaker diarization
- המופעל דרך capability_runtime (עתיד)

## 📈 פער

**כל המודול חסר.** דרישה: `src/capabilities/speech/` עם orchestrator ל-Whisper + Gemini TTS.

## היסטוריה
| תאריך | Score |
|:-----:|:-----:|
| 23.04 | 0.00 | Baseline. אין integration. תלוי ב-CapabilityOrchestrator |
