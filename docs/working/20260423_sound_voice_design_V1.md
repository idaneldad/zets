# Sound/Voice Learning Design — Phonetic-Graph, Cadence, Voice Signatures

**תאריך:** 23.04.2026
**בקשת עידן:** לקלוט הקלטות מ-Gemini, ללמוד איך מילה בכל שפה נשמעת פונטית,
לזהות קצב+מבטא, לייצר דיבור חזרה עם 30 חתימות קול (5×6 קבוצות: גברים/נשים/ילדים/ילדות/בנים-3/בנות-3),
לקונפיג gil+מבטא+קצב ו-ZETS יחשב נוסחה לבד. תווים מוסיקליים כאמצעי לקצב+טון.

---

## שבירת כלים — 7 הנחות שגויות

### 🔨 #1 "נשמור WAV ונעביר דרך DL models"
**השבר:** 44.1kHz WAV = 10MB/דקה. 1000 הקלטות = 10GB. וגם, DL models לא מתאימים לפילוסופיית ZETS.

**התובנה:** אנחנו לא צריכים WAV מלא. אנחנו צריכים:
- **phoneme sequence** (IPA או SAMPA) — ~20 bytes/sec
- **prosody trajectory** (pitch contour, loudness) — ~200 bytes/sec
- **voice signature** (static per-speaker, not per-word) — 50 bytes
- **duration map** (phoneme → ms) — ~20 bytes/sec

סה"כ: **~300 bytes/sec** במקום **176,400 bytes/sec** של WAV. **factor 600× reduction.**

### 🔨 #2 "נלמד פונטיקה ע"י אימון מודל"
**השבר:** יש כבר IPA — האלפבית הפונטי הבינלאומי. **~60 סמלים מכסים את כל השפות.**

**התובנה:** פונטיקה זה **graph mapping**, לא ML:
- כל שפה יש לה **phoneme inventory** (מה הצלילים שלה — למשל עברית יש 22, אנגלית 44, מנדרינית ~30)
- כל מילה = **sequence of phonemes**
- ללמוד שפה חדשה = **לבנות את ה-mapping** grapheme→phoneme
- זה עובד ב-graph native!

```
atom "קול" ──phonetic──> [/k/, /o/, /l/]
atom "call" ──phonetic──> [/k/, /ɔː/, /l/]
```

רק 1 atom נוסף לכל מילה + 60 edges possible. **Trivial scale.**

### 🔨 #3 "קצב נלמד מ-audio waveform"
**השבר:** קצב זה **מטריקה של durations**, לא spectrum.

**התובנה:** לכל phoneme יש **distribution of durations** בכל speaker+language:
- ילד בן 3 אומר /k/ בממוצע ב-80ms
- גבר בוגר אומר /k/ ב-45ms
- רוסית דוברת רוסית /k/ ב-30ms (שפה מהירה)

**זו נוסחה, לא waveform.** 30 speakers × 60 phonemes × 1 float = 1800 numbers = **7KB סה"כ** לכל ה-matrix.

### 🔨 #4 "מבטא = acoustic features"
**השבר:** מבטא זה **phoneme substitution rules** + prosody shifts.

**התובנה:** ישראלי מדבר אנגלית → מחליף phonemes שאין בעברית:
- `/θ/` → `/t/` (three → tree)
- `/w/` → `/v/`
- ה-`/r/` מקבל gum roll במקום retroflex

זה **ruleset, לא DSP**. ZETS כבר יודע לעבוד עם ruleset ב-graph!

```
accent:israeli-english ──substitutes──> [/θ/→/t/, /w/→/v/, /r/→/ʁ/]
accent:israeli-english ──stress-pattern──> Hebrew-like (last syllable)
```

### 🔨 #5 "תווים לא עוזרים לדיבור"
**השבר — עידן נגע בזה במפורש.** דיבור **הוא** prosody, וprosody **זה** musical.

**התובנה:** נציג prosody ע"י **musical staff notation**:

```
מילה: "שלום"
       ↓
Note sequence: [D4 quarter, F4 eighth, A3 half]
Where: D4 = pitch of /ʃ/ syllable, F4 = /a/, A3 = /lom/
Duration: quarter=normal, eighth=quick, half=extended
Loudness: piano/mezzo/forte
```

**MIDI-like encoding של דיבור.** זה:
- Compact (3-5 notes per word)
- Deterministic (same word → same sequence in same voice)
- Configurable (pitch up → higher voice, tempo up → faster speech)
- Composable (combine notes + phonemes → full speech)

### 🔨 #6 "צריך ML להפיק TTS"
**השבר:** **Formant synthesis** עובד מ-1950. אין צריך ML.

**התובנה:** לכל phoneme יש **formant signature**:
- /a/ = F1=730Hz, F2=1090Hz, F3=2440Hz (male adult)
- /i/ = F1=270Hz, F2=2290Hz, F3=3010Hz
- /u/ = F1=300Hz, F2=870Hz, F3=2240Hz

**2-3 sinusoid generators מסונכרנים → דיבור מובן.** לא מדהים, אבל **זה עובד בלי GPU**.

ל-"תחושה אמיתית" אפשר להוסיף שכבה של Gemini TTS (יש API key) בתור cloud option, והקוד המקומי הוא formant synth default.

### 🔨 #7 "30 חתימות קול = 30 models"
**השבר:** חתימה = **delta מ-canonical**, לא model מלא.

**התובנה:** יש **base voice** אחת (formant defaults). כל speaker = **set of deltas**:
- `voice:gavri_adult_1` ──pitch-shift──> -15% (גבוה/נמוך מהממוצע)
- `voice:gavri_adult_1` ──formant-F2──> +5%
- `voice:gavri_adult_1` ──speed──> 1.1×
- `voice:gavri_adult_1` ──roughness──> 0.3

30 speakers × ~8 parameters = 240 floats = **960 bytes**. כל ה-voice-bank.

---

## הארכיטקטורה — ZETS-native

### Layer 1: Phoneme Graph (universal)

```rust
struct Phoneme {
    ipa: String,         // "/k/", "/a/", "/ʃ/"
    sampa: String,       // "k", "a", "S" (ASCII-safe)
    place: Place,        // bilabial, alveolar, velar...
    manner: Manner,      // stop, fricative, nasal...
    voiced: bool,
    vowel_features: Option<VowelFeatures>,  // if vowel
}

struct VowelFeatures {
    height: f32,         // 0=open, 1=close
    backness: f32,       // 0=front, 1=back
    rounded: bool,
}
```

**~60 phonemes cover everything.** Store as atoms in graph.

### Layer 2: Language Phoneme Inventory

```rust
struct LanguageInventory {
    language: LanguageCode,      // "he", "en", "ar"
    phonemes: Vec<PhonemeId>,   // which phonemes this language uses
    grapheme_to_phoneme: HashMap<String, Vec<PhonemeId>>,
    // "ש" → [/ʃ/]  in Hebrew
    // "sh" → [/ʃ/]  in English
    common_substitutions: Vec<(PhonemeId, PhonemeId)>,
    // for pronunciations by non-native speakers
}
```

### Layer 3: Word-to-Phoneme (grapheme graph)

```rust
// For each word atom, an edge to its phoneme sequence
atom "shalom" ──pronounced-as──> [/ʃ/, /a/, /l/, /o/, /m/]
atom "שלום" ──pronounced-as──> [/ʃ/, /a/, /l/, /o/, /m/]  # same!
// SAME_AS edge connects both spellings
```

**Cross-lingual bonus:** "shalom" and "שלום" share phonemes → automatic transliteration.

### Layer 4: Prosody as Musical Notes

```rust
struct MusicalNote {
    pitch: f32,          // Hz (C4=262, A4=440...)
    duration_ms: u32,
    loudness: f32,       // 0-1, maps to dB
    attack: f32,         // 0=smooth, 1=percussive
}

struct WordProsody {
    word_id: AtomId,
    notes: Vec<MusicalNote>,  // one per syllable typically
    // maps to the phoneme sequence of the word
}
```

**דוגמה: "שלום" ב-declarative vs question:**
```
Declarative: [D4 short, F4 med, A3 long]      # falling cadence
Question:    [D4 short, F4 short, A5 long]    # rising cadence  
Call:        [F4 short, A4 short, D5 long]    # upward inflection
```

**אותה מילה, 3 prosody patterns** — זה ההבדל בין declarative/question/call.

### Layer 5: Voice Signature (per-speaker delta)

```rust
struct VoiceSignature {
    speaker_id: String,        // "gavri_adult_1"
    category: VoiceCategory,    // Man/Woman/Boy/Girl/Boy3/Girl3
    age_years: f32,
    
    // Deltas from canonical:
    pitch_shift_pct: f32,       // -20% to +30%
    pitch_variance: f32,        // 0=monotone, 1=expressive
    formant_shift: [f32; 3],    // F1/F2/F3 deviations
    speed_factor: f32,          // 0.7 to 1.5
    roughness: f32,             // creak, breathiness
    accent: AccentId,           // native, non-native, regional
}

enum VoiceCategory {
    ManAdult,      // 85-180 Hz fundamental
    WomanAdult,    // 165-255 Hz
    Boy10,         // 250-350 Hz
    Girl10,        // 260-380 Hz
    Boy3,          // 350-500 Hz
    Girl3,         // 360-520 Hz
}
```

**30 speakers = 30 × 8 floats = 960 bytes.** Entire voice bank.

### Layer 6: Learn from Gemini Audio

```rust
// Incoming: Gemini provides WAV + text
struct IncomingAudio {
    wav_bytes: Vec<u8>,          // small sample, delete after extraction
    transcript: String,
    language: LanguageCode,
    speaker_id: Option<String>,  // if known
}

fn learn_from_audio(audio: IncomingAudio, graph: &mut Graph) {
    // 1. Extract pitch contour (0.5 sec granularity) — ffmpeg + simple FFT
    let pitch = extract_pitch_track(&audio.wav_bytes);
    
    // 2. Extract timing (phoneme boundaries via known IPA)
    let phonemes = text_to_phonemes(&audio.transcript, audio.language);
    let durations = align_phonemes_to_audio(&phonemes, &audio.wav_bytes);
    
    // 3. Build prosody notes
    let notes = pitch_and_duration_to_notes(&pitch, &durations);
    
    // 4. Store AS ATOMS (not WAV):
    let word_atoms = graph.words_from(&audio.transcript);
    for (word, notes) in word_atoms.zip(notes_per_word) {
        graph.add_edge(word, graph.prosody_atom(notes), "prosody");
    }
    
    // 5. Update speaker's voice signature (rolling average)
    if let Some(sp) = audio.speaker_id {
        update_voice_signature(sp, &pitch, &durations);
    }
    
    // 6. DELETE the WAV — we have what we need
}
```

**Incoming 10MB WAV → ~2KB of atoms/edges.** WAV deleted after extraction.

### Layer 7: Generate Speech (TTS)

```rust
fn synthesize(text: &str, voice: &VoiceSignature,
              emotion: Emotion) -> Vec<u8> {
    // 1. Text → phonemes via language inventory
    let phonemes = text_to_phonemes(text, voice.language());
    
    // 2. Apply accent substitutions if non-native
    let adjusted = apply_accent(phonemes, voice.accent);
    
    // 3. Lookup prosody atoms for each word (or synthesize default)
    let notes = gather_prosody(&text, emotion);
    
    // 4. Apply voice signature deltas
    let tuned_notes = apply_voice_delta(&notes, voice);
    
    // 5. Formant synthesis (3-formant)
    generate_audio(&adjusted, &tuned_notes)
}
```

**End result:** Text → WAV bytes, deterministic, no ML, no GPU.
**Optional cloud enhancement:** If quality insufficient, route to Gemini TTS.

---

## Configurable Axes (עידן ביקש)

המשתמש מגדיר:
- **גיל** (3, 10, adult, elderly) → maps to VoiceCategory + pitch band
- **מבטא** (native_he, israeli_en, american_en, ...) → AccentId
- **קצב** (slow 0.7×, normal 1.0×, fast 1.3×) → speed_factor
- **expressiveness** (monotone 0 → expressive 1) → pitch_variance

**ZETS מחשב נוסחה:**
```rust
fn build_voice(age: f32, accent: AccentId, speed: f32,
               expressiveness: f32) -> VoiceSignature {
    let category = VoiceCategory::from_age(age);
    let base = category.base_signature();
    VoiceSignature {
        category,
        age_years: age,
        pitch_shift_pct: base.pitch_shift_pct,
        pitch_variance: expressiveness,
        formant_shift: base.formant_shift,
        speed_factor: speed,
        roughness: if age > 60.0 { 0.3 } else { 0.1 },
        accent,
        ...
    }
}
```

**5 parameters → full voice.** User משחק עם slider, ZETS מרכיב.

---

## סיכום כיצד זה יעבוד

### כש-ZETS מקבל הקלטה מ-Gemini:

```
1. ffmpeg extract: pitch track @ 100Hz, loudness envelope
2. text (from Gemini transcript) → phoneme sequence via language atlas
3. align phonemes to pitch track → duration per phoneme
4. extract note sequence from pitch contour → MusicalNote atoms
5. link: word atom ──prosody──→ note sequence atom
6. if new speaker: build VoiceSignature from deltas to base
7. DELETE WAV (we have 2KB of structured knowledge instead of 10MB)
```

### כש-ZETS מייצר דיבור:

```
1. text → phonemes (language-specific)
2. apply accent substitutions (if non-native)
3. gather prosody from graph (or default)
4. apply voice signature (pitch shift, formant shift, speed)
5. formant synthesis → 3-sinusoid audio at 22kHz
6. optional: route to Gemini TTS if quality needs boost
```

### אחסון:

| Asset | Size |
|-------|------|
| 60 phonemes catalog | ~3 KB |
| 50 languages × phoneme inventory | ~150 KB |
| 1M word prosody atoms | ~5 MB |
| 30 voice signatures | ~1 KB |
| Accent rules (20 accents × 10 subs) | ~5 KB |
| **Total sound knowledge** | **~5 MB** |

**כל הידע הקולי של ZETS לכל השפות = 5MB.** מאוד חסכוני.

---

## ה-Phased Implementation

**Phase 1 — Phoneme graph (Rust, ~200 lines):**
- `src/sound/phoneme.rs` — Phoneme struct + 60 atom seeds
- `src/sound/inventory.rs` — per-language inventory

**Phase 2 — Audio extraction (Python prototype first):**
- `py_testers/test_audio_learning_v1.py` — ffmpeg pitch extraction + phoneme alignment
- If proves out: `src/sound/analyzer.rs`

**Phase 3 — Prosody atoms:**
- `src/sound/prosody.rs` — MusicalNote + WordProsody
- Edges: word ──prosody──> notes

**Phase 4 — Voice signatures:**
- `src/sound/voice.rs` — VoiceSignature with deltas
- 30 default signatures (5×6 categories)

**Phase 5 — Formant synthesis:**
- `src/sound/synth.rs` — 3-formant generator
- No external deps (just std math)

**Phase 6 — Gemini integration:**
- `zets-tools/audio_bridge/` (Python, separate repo)
- Gemini API → extract → feed atoms to ZETS

---

## Python Prototype שאני אבנה

`py_testers/test_audio_learning_v1.py`:

1. **Phoneme inventory** — 22 Hebrew + 44 English + mapping
2. **Text to phonemes** — "שלום" → [/ʃ/, /a/, /l/, /o/, /m/]
3. **Pitch extraction** — from tiny WAV (synthetic), 100Hz sampling
4. **Notes from pitch** — 440Hz tone for 200ms → A4 quarter note
5. **Voice signature deltas** — adult male vs 3yo girl → 8 parameters differ
6. **Accent substitution** — "three" by Israeli speaker → [/t/, /r/, /i/]
7. **Storage efficiency** — show 10MB WAV → 2KB atoms

If passes → propose Rust implementation per phase.

---

## מה **לא** עשיתי (מחכה לאישור)

Per CLAUDE_RULES §4:
- ❌ `src/sound/*.rs`
- ❌ `Cargo.toml` dependencies (אולי `hound` לWAV או רק std)
- ❌ Gemini integration
- ❌ WebAudio output (browser-side)

**ממתין לאישור לבנות Python prototype קודם.**
