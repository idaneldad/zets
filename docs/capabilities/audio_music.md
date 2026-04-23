# 🎵 אודיו ומוזיקה — Audio & Music

**Last updated:** 23.04.2026
**Current score:** 🟠 0.09 | **Target MVP:** 0.70 | **Gap:** −0.61

**באחריות:** `graph + חיצוני` — ZETS motif-bank למבנה מוזיקלי, Suno/Udio לwaveform

## 🎯 משימה

הבנת אודיו (genre, instruments, lyrics) + יצירת מוזיקה, סיכום meeting-room.

## ✅ הצלחה
- Genre classification F1 0.80+
- Instrument detection F1 0.75+
- Lyric WER <0.20
- Music gen human rating >3.5/5
- Meeting summary coverage 0.80+

## 🔬 Tests (5)

| # | Test | סוג | יעד | סטטוס | מודול |
|---|------|:---:|:---:|:-----:|--------|
| 9.1 | Genre classification | QA | 0.80 | 🔴 0.00 | — |
| 9.2 | Instrument detection | QA | 0.75 | 🔴 0.00 | — |
| 9.3 | Lyric transcription | QA | 0.80 | 🔴 0.00 | (חלק מ-STT) |
| 9.4 | Music generation (Suno) | QA | 0.70 | 🟠 0.25 | composition/motif MusicalPhrase kind |
| 9.5 | Meeting audio summary | QA+TEST | 0.80 | 🟠 0.20 | conversation + reader (needs STT) |

## 🏗️ באחריות
- **Composition** — chord progressions, motif structure (graph)
- **Suno/Udio/Stability Audio** — actual waveform (external)
- **Whisper + diarization** — meeting transcription (external)

## היסטוריה
| תאריך | Score |
|:-----:|:-----:|
| 23.04 | 0.09 | MotifKind::MusicalPhrase exists, no audio integration |
