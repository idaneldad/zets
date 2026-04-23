# Media as Graph — Speech/Music/Sound/Image/Video — Decomposition + Recomposition

**תאריך:** 23.04.2026
**בקשת עידן:** פרק ל-atoms, הרכב חזרה (playback or synthesis); רפרודוקציה של
מה שנקלט, או יצירה חדשה. חדרי ישיבות, מצלמות אבטחה, שירים, תמונות, סרטונים.
"לבסוף צמיד זה שחזור, או תשובה מתוך זה או המצאה מודעת."

**החלטות שהתקבלו לבד (per Idan's directive):**
- OPEN Q#1 Authority keys → **Multi-authority** (B). ZETS-Anthropic CA + user-defined CAs. Community can add.
- OPEN Q#2 Federation default → **Hash-only ON**, content OFF (B). Light participation by default.
- OPEN Q#3 Sound TTS cloud fallback → **Both** (B). Formant default + Gemini if user wants quality.
- OPEN Q#4 Gemini audio input → **Both** (C). 10 seed voices from Gemini, grow from user over time.
- OPEN Q#5 Music notation UX → **B** — GUI for musicians. Debug-viewable always. Speech renderable as sheet music when asked.

כל ההחלטות **הפיכות** — עידן יכול לתקן דרך atom override.

---

# שבירת כלים אחודה — 9 הנחות שצריך לשבור

### 🔨 #1 "מדיה = blob אטום"
**השבר:** לא. מדיה = **graph of atoms** ברמת הפירוק הטבעי של ה-modality.
- Speech → phonemes + prosody notes
- Music → notes + chords + rhythm + lyrics
- Sound → events + timestamps + sources
- Image → regions + objects + relations + metadata
- Video → scenes + shots + frames + audio track

**אותו graph.** merged, searchable, reconstructible, generatable.

### 🔨 #2 "שמור כל frame"
**השבר:** video 30fps = 30 frames/sec. ברוב frames כלום לא השתנה. **Keyframes + deltas**.
- Keyframe כל 1-5 שניות (או בשינוי scene)
- Between: רק ה-deltas (מה זז)
- Audio track: phonemes + prosody כמו שפה

### 🔨 #3 "מצלמת אבטחה = hours of empty"
**השבר:** 99% של מצלמת אבטחה אין שינוי. **event-triggered atoms**:
- Motion detected → create atom
- Person enters → atom {time, region, bbox}
- Object moves → atom {from, to, time}

**ZETS store = events only.** כל ה-boring footage לא מאוחסן.

### 🔨 #4 "שיר = audio waveform"
**השבר:** שיר = **atoms משני types מסוכרנים**:
- Melody atoms (notes over time)
- Lyric atoms (syllables over time)
- Timing edges מחברים (syllable_at_beat_3 → note_D4_at_beat_3)

Output מ-ZETS: יכול להפיק MIDI + TTS → recombine. גם בלי audio מקורי.

### 🔨 #5 "reconstruction = pixel-perfect"
**השבר:** לא צריך. "צמיד" של עידן = **faithful enough to purpose**. לרפלקציה אישית או שליחה לחבר — אם הרוח, התוכן, ה-speaker נכונים, זה מצליח. ל-forensic evidence — שמור raw separate track.

**2 modes:**
- `Gist mode` — symbolic atoms only. Reconstruct approximates.
- `Evidence mode` — raw bytes stored אחורה בגרף של hash, recoverable exact.

### 🔨 #6 "generation = ML model"
**השבר:** generation = **graph walk + rules**. ZETS הולך:
- Take prompt: "שליחת שאלה קולית של adult man saying 'מה שלומך?'"
- Lookup: voice signature → pitch + formants
- Lookup: phonemes of מה שלומך → [/m/,/a/,/sh/,/l/,/o/,/m/,/kh/,/a/]
- Lookup: default prosody for question intonation → rising contour
- Synthesize: formant synth + prosody → WAV bytes

אפשר גם cloud: אותו graph decision route ל-Gemini TTS עם parameters מפורשים.

### 🔨 #7 "image = flat array"
**השבר:** image = **hierarchical decomposition**:
- Frame-level: resolution, colorspace, dominant colors
- Region-level: foreground/background bbox
- Object-level: detected objects + class + confidence
- Relation-level: object A → next to → object B
- Metadata: who/when/where/camera

כל רמה = atom. Image retrieval דרך walk ב-graph.

### 🔨 #8 "הקלטת חדר ישיבות = 1 stream"
**השבר:** חדר ישיבות = **multi-source separable**:
- Speaker A atoms (with speaker_id)
- Speaker B atoms
- Background noise atoms (chair creaks, typing)
- Each timestamped
- Recombine: just merge by timestamp

משנה אם רוצים רק "מה אמר דובר B" — פשוט filter.

### 🔨 #9 "reconstruction = identical bits"
**השבר:** 3 רמות של "אותו דבר":
- **Bit-identical** (hash match) — evidence mode
- **Perceptually-identical** (same words, pitch curve, style) — default
- **Semantically-equivalent** (same meaning, different wrapping) — paraphrase mode

User picks. Default = perceptually-identical.

---

# The Unified Media Graph

```
                  ┌─────────────────────────┐
                  │     MediaAtom (kind)     │
                  └──────────┬──────────────┘
         ┌────────┬──────────┼──────────┬─────────────┬──────────┐
         │        │          │          │             │          │
      Speech   Music      Sound      Image          Video     Document
         │        │          │          │             │          │
   ┌─────┴──┐  ┌──┴────┐  ┌──┴───┐   ┌──┴───┐    ┌───┴───┐   ┌─┴──┐
   │Phoneme│  │ Note  │  │Event │   │Region│    │Keyframe│  │Page│
   │Prosody│  │ Chord │  │Source│   │Object│    │ Delta  │  │Para│
   │ Notes │  │ Lyric │  │ Time │   │Label │    │Audio-T │  │Word│
   │Speaker│  │ BPM   │  │      │   │BBox  │    │SubsAtom│  │    │
   └───────┘  └───────┘  └──────┘   └──────┘    └────────┘   └────┘
```

**Common edges** connect across modality:
- `occurs_at` (timestamp)
- `co-occurs_with` (same time)
- `depicts` (image shows X)
- `uttered_by` (speech → speaker atom)
- `mentions` (speech/text → concept atom)
- `sample_of` (derived → source)

---

# Decomposition pipelines (per modality)

## 🎙 Speech (already designed — test_audio_learning_v1.py, 12/12)
```
WAV → ffmpeg pitch-track → autocorr pitch → 
  + phoneme alignment → 
  + speaker detection (delta from canonical) →
  atoms: WordAtom ──prosody→ [MusicalNote]
         WordAtom ──phonemes→ [Phoneme]
         SpeakerAtom signature
```
**Delete WAV** after extraction.

## 🎼 Music
```
MP3/WAV → ffmpeg → 
  + pitch detection (FFT peaks) →
  + onset detection (energy changes) →
  + tempo estimation (beat tracking) →
  + chord inference (multi-pitch voting) →
  + lyric extraction (if vocal) — uses speech pipeline →
  atoms: NoteAtom, ChordAtom, BeatGrid, LyricSyllable
```

## 🔔 Sound events (non-speech, non-music)
```
WAV → 
  + onset detection →
  + spectral class match (against sound dictionary: door, glass, footsteps) →
  + confidence per class →
  atoms: SoundEventAtom { class, confidence, time, duration, loudness }
```

## 🖼 Image
```
JPG/PNG → 
  + downsize to 512×512 for analysis →
  + color histogram →
  + edge/region detection →
  + if model available: object detection (Gemini Vision for accuracy) →
  + metadata extraction (EXIF) →
  atoms: ImageAtom (w/ thumbnail), RegionAtom, ObjectAtom, ColorAtom
```

## 🎥 Video
```
MP4 → ffmpeg extract →
  + video: keyframe every N sec → Image pipeline per keyframe →
  + audio: Speech pipeline on track →
  + scene detection: large-diff keyframes boundary →
  atoms: ScenaAtom → contains VideoFrame(s) + SpeechAtoms in time range
```

## 📄 Document (bonus — תואם הכל)
```
TXT/PDF/DOCX → 
  + page segmentation →
  + paragraph → sentence → word atoms (already in ZETS core) →
  + cross-reference mentions to image/video atoms →
  atoms: DocumentAtom, PageAtom, ParagraphAtom (existing)
```

---

# Reconstruction modes (the "צמיד" concept)

## Mode A: Faithful Playback
User asks: "play back what A said at 3pm in the meeting"
- Walk graph → SpeakerAtom(A) + time 15:00 → list of WordAtoms
- Each WordAtom → phonemes + prosody
- Speaker's voice signature → synth pipeline
- Formant synth or Gemini TTS → WAV
- Output

**Cost:** ~200ms for a minute of speech.

## Mode B: Semantic Reconstruction
User asks: "summarize A's comments in my voice"
- Walk graph → A's wordatoms → collate meaning
- Transform: apply user's voice signature + preferred prosody
- Re-synth

Results: same content, different voice/style.

## Mode C: Creative Generation
User asks: "write a song about my cat in the style of the Beatles"
- Walk graph for "cat" concepts, "Beatles" style atoms
- Generate lyric atoms (creative combination)
- Generate note atoms (Beatles chord progressions + melodic contours)
- Synthesize

**This is art-level.** ZETS as creative tool, not just recall.

## Mode D: Evidence Mode (forensic)
User asks: "give me the original recording, exact bits"
- If stored: return WAV from hash store
- If deleted: "only symbolic atoms available — here's reconstruction"
- Clear labeling

---

# Real scenarios (what Idan described)

### Meeting room recording
```
Input: 2-hour WAV, 4 speakers
Process:
  1. Speaker diarization → 4 SpeakerAtoms
  2. Speech pipeline per segment → WordAtoms with speaker_id + time
  3. Topics extraction (cluster words) → TopicAtoms
  4. Decisions/action-items (rule-based on phrases) → ActionAtoms
Output atoms: ~50KB
  (2-hour WAV was 1.4GB → 28,000× compression)
Recall: "what did Tal say about pricing?" → filter + playback or text
```

### Airport security camera
```
Input: 24×7 continuous video
Process:
  1. Motion detection → only-when-active atoms
  2. Face detection → FaceAtom + bounding box (hash, not image)
  3. Behavior inference: walking/running/standing → BehaviorAtom
  4. Alert rules: running + restricted-zone → AlertAtom
Output atoms: 99% empty hours → 0 atoms. Only events.
  (24h * 30fps * 1MB frame = 2.5TB → ~5MB of events)
Recall: "alerts last week" → walk AlertAtoms; reconstruct frames on-demand
```

### Song creation
```
Input: prompt "like Idan singing about coding"
Process:
  1. Lookup voice:idan signature
  2. Generate lyric atoms from "coding" topics + rhyme graph
  3. Generate melody: progression C-Am-F-G + vocal contour
  4. Align: syllables to beats
  5. Speech+music synth: overlay formant + melody
Output: WAV 3 min
```

### Q&A audio message (what Idan specifically asked)
```
User types: "שאלה — מה שלומך?"
  OR speaks it in an existing voice
Process:
  1. Text → phonemes (Hebrew)
  2. Default prosody: question intonation (rising)
  3. Voice: chosen signature (user's, or recipient-appropriate)
  4. Synth → WAV
Recipient gets: WAV that reads like a natural voice message
```

---

# Storage strategy (aggressive compression)

| Modality | Raw | As atoms | Ratio |
|----------|-----|----------|-------|
| 1 min speech (44kHz WAV) | 10.6 MB | ~10 KB | 1000× |
| 1 min music (MP3) | 1 MB | ~20 KB (notes+lyrics) | 50× |
| 1 hour meeting audio | 635 MB | ~25 KB | 25000× |
| 1 image 1920×1080 | 3 MB | ~2 KB (+ thumb 30 KB) | 100× (or 1500× without thumb) |
| 1 minute video 720p | 40 MB | ~80 KB (+ keyframes 2 MB) | 20× (500× without frames) |
| 24h security footage | 2.5 TB | ~5 MB (events only) | 500000× |
| Song 3 min | 7 MB | ~60 KB (notes+lyrics+style) | 120× |

**אם user רוצה fidelity של 100% — raw ב-evidence store נפרד, hash ב-atom.**

---

# Integration — ZETS-tools/media_bridge (Python external tooling)

זה לא חלק מ-Rust core. זה **separate repo** שמשתמש ב-ZETS דרך HTTP:
```
zets-tools/media_bridge/
├── extract_speech.py      # WAV → phoneme/prosody atoms
├── extract_music.py       # MP3 → note/chord atoms
├── extract_image.py       # JPG → region/object atoms
├── extract_video.py       # MP4 → scene/frame atoms
├── gemini_client.py       # Cloud enhancement calls
└── synthesize.py          # atoms → WAV/MP4 output
```

ZETS core (Rust) ≠ this. Rust stays clean.

---

# Rust implementation (phased, after Idan approves)

**Phase 1 — `src/media/atom_types.rs`**
Common types: MediaAtom enum, timestamps, durations, confidences

**Phase 2 — `src/media/speech.rs`**
Bridge to existing sound_voice design. Decode/encode phonemes + prosody.

**Phase 3 — `src/media/music.rs`**
Note/Chord/Beat/Lyric atoms. MIDI import/export.

**Phase 4 — `src/media/image.rs`**
ImageAtom w/ thumbnail + region/object atom links.

**Phase 5 — `src/media/video.rs`**
Scene/keyframe decomposition. Uses media::speech + media::image.

**Phase 6 — `src/media/sound_events.rs`**
Door/glass/footsteps/etc atom dictionary.

**Phase 7 — `src/media/reconstruction.rs`**
Playback/Semantic/Creative/Evidence mode routers.

**Total:** ~1500 lines Rust for core. External tools (ffmpeg, Gemini) in zets-tools.

---

# Privacy + Ethics (tied to trust_spaces design)

Every media atom inherits from TrustRelation:
- Meeting room → trust:work_colleagues; can't be shared outside that trust space
- Security camera → trust:authorized_personnel_only
- Private voice message → trust:sender+recipient only
- Public song → trust:public

**Query-time filter**: asker doesn't match trust → atom not returned.

---

# What the Python prototype will prove (next step)

`py_testers/test_media_graph_v1.py`:
1. Decompose sample audio → phoneme atoms (uses ffmpeg)
2. Decompose sample image → region atoms (simple pixel-average, no ML)
3. Decompose simulated meeting → multi-speaker atoms
4. Reconstruct from atoms: `play A's words` → WAV output
5. Generate new: `say this text in voice X` → WAV output
6. Security camera simulation: motion events → atoms (99% compression)
7. Cross-modal: "match speech atoms to image keyframes by time"
8. Evidence mode: return raw bytes if stored, else atom-based approximation

**Expected:** 8/8 passing, proving the graph-native approach.

---

# Summary: בצמיד של עידן

**"צמיד זה שחזור, או תשובה מתוך זה או המצאה מודעת"** ← exactly 3 modes:
- **שחזור** = Mode A (faithful playback from atoms)
- **תשובה מתוך זה** = Mode B (semantic reconstruction / Q&A over media)
- **המצאה מודעת** = Mode C (creative generation from graph style+content)

All three pull from the same media-as-graph atom store. No special cases.
