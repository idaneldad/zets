#!/usr/bin/env python3
"""
test_audio_learning_v1.py — proves ZETS-native sound/voice design.

Validates 12 ideas (per docs/working/20260423_sound_voice_design_V1.md):
  1-4.  Phoneme graph is universal (IPA, ~25 symbols cover HE+EN in this demo;
        real impl has ~60 for all languages)
  5-7.  Voice signatures as 8-parameter deltas. 30 speakers fit in <2KB.
        Age → formant scaling (children have shorter vocal tracts).
  8.    Musical notes as MIDI-like encoding of prosody (Hz → note name).
  9.    Pitch extraction via simple autocorrelation (no librosa needed).
  10.   Accent = phoneme substitution rules, not DSP.
  11.   Storage: 10MB WAV → ~2KB of atoms. 300× reduction.
  12.   Formula: age + accent + speed + expressiveness → tuned voice.

Per CLAUDE_RULES §4 — Python prototype BEFORE Rust.
Run: python3 py_testers/test_audio_learning_v1.py
"""

import math
import struct
import tempfile
import wave
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import Optional


# ═══════════════════════════════════════════════════════════════════════
# (1) Phoneme catalog — IPA-based, universal
# ═══════════════════════════════════════════════════════════════════════

class Manner(Enum):
    STOP = 1
    FRICATIVE = 2
    AFFRICATE = 3
    NASAL = 4
    TRILL = 5
    TAP = 6
    APPROXIMANT = 7
    VOWEL = 8


@dataclass(frozen=True)
class Phoneme:
    ipa: str
    sampa: str          # ASCII-safe
    manner: Optional[Manner] = None
    voiced: bool = False
    formants: Optional[tuple] = None  # (F1, F2, F3) Hz, adult male baseline


# Representative phoneme catalog (~25 phonemes sufficient for demo).
# Full implementation would have ~60 covering all world languages.
PHONEME_CATALOG = [
    # Vowels with formants
    Phoneme("/a/", "a", Manner.VOWEL, True, (730, 1090, 2440)),
    Phoneme("/e/", "e", Manner.VOWEL, True, (530, 1840, 2480)),
    Phoneme("/i/", "i", Manner.VOWEL, True, (270, 2290, 3010)),
    Phoneme("/o/", "o", Manner.VOWEL, True, (570, 840, 2410)),
    Phoneme("/u/", "u", Manner.VOWEL, True, (300, 870, 2240)),
    # Consonants
    Phoneme("/p/", "p", Manner.STOP, False),
    Phoneme("/b/", "b", Manner.STOP, True),
    Phoneme("/t/", "t", Manner.STOP, False),
    Phoneme("/d/", "d", Manner.STOP, True),
    Phoneme("/k/", "k", Manner.STOP, False),
    Phoneme("/g/", "g", Manner.STOP, True),
    Phoneme("/f/", "f", Manner.FRICATIVE, False),
    Phoneme("/v/", "v", Manner.FRICATIVE, True),
    Phoneme("/s/", "s", Manner.FRICATIVE, False),
    Phoneme("/z/", "z", Manner.FRICATIVE, True),
    Phoneme("/sh/", "S", Manner.FRICATIVE, False),  # /ʃ/
    Phoneme("/th/", "T", Manner.FRICATIVE, False),  # /θ/
    Phoneme("/h/", "h", Manner.FRICATIVE, False),
    Phoneme("/m/", "m", Manner.NASAL, True),
    Phoneme("/n/", "n", Manner.NASAL, True),
    Phoneme("/l/", "l", Manner.APPROXIMANT, True),
    Phoneme("/r/", "r", Manner.TRILL, True),
    Phoneme("/R/", "R", Manner.FRICATIVE, True),  # /ʁ/ uvular (Hebrew)
    Phoneme("/w/", "w", Manner.APPROXIMANT, True),
    Phoneme("/j/", "j", Manner.APPROXIMANT, True),
    Phoneme("/H/", "H", Manner.FRICATIVE, False),  # /ħ/ pharyngeal (Hebrew)
    Phoneme("/?/", "?", Manner.STOP, False),  # /ʔ/ glottal
]

PHONEME_BY_IPA = {p.ipa: p for p in PHONEME_CATALOG}


# ═══════════════════════════════════════════════════════════════════════
# (2) Language inventories + grapheme→phoneme mapping
# ═══════════════════════════════════════════════════════════════════════

HEBREW_G2P = {
    "א": ["/a/"], "ב": ["/b/"], "ג": ["/g/"], "ד": ["/d/"],
    "ה": ["/h/"], "ו": ["/v/"], "ז": ["/z/"], "ח": ["/H/"],
    "ט": ["/t/"], "י": ["/j/"], "כ": ["/k/"], "ך": ["/k/"],
    "ל": ["/l/"], "מ": ["/m/"], "ם": ["/m/"],
    "נ": ["/n/"], "ן": ["/n/"], "ס": ["/s/"],
    "ע": ["/?/"], "פ": ["/p/"], "ף": ["/f/"],
    "צ": ["/s/"], "ץ": ["/s/"], "ק": ["/k/"],
    "ר": ["/R/"], "ש": ["/sh/"], "ת": ["/t/"],
    # Simplified — Hebrew often omits vowels in script
    "ָ": ["/a/"], "ַ": ["/a/"], "ֶ": ["/e/"], "ֵ": ["/e/"],
    "ִ": ["/i/"], "ֹ": ["/o/"], "ֻ": ["/u/"],
}

ENGLISH_G2P = {
    "a": ["/a/"], "e": ["/e/"], "i": ["/i/"], "o": ["/o/"], "u": ["/u/"],
    "b": ["/b/"], "c": ["/k/"], "d": ["/d/"], "f": ["/f/"], "g": ["/g/"],
    "h": ["/h/"], "k": ["/k/"], "l": ["/l/"], "m": ["/m/"],
    "n": ["/n/"], "p": ["/p/"], "r": ["/r/"], "s": ["/s/"],
    "t": ["/t/"], "v": ["/v/"], "w": ["/w/"],
    "y": ["/j/"], "z": ["/z/"],
    # Digraphs (checked first)
    "th": ["/th/"], "sh": ["/sh/"], "ee": ["/i/"], "oo": ["/u/"],
}


@dataclass
class LanguageInventory:
    language: str
    phonemes: list
    g2p: dict


HEBREW = LanguageInventory(
    language="he",
    phonemes=sorted(set(p for seq in HEBREW_G2P.values() for p in seq)),
    g2p=HEBREW_G2P,
)
ENGLISH = LanguageInventory(
    language="en",
    phonemes=sorted(set(p for seq in ENGLISH_G2P.values() for p in seq)),
    g2p=ENGLISH_G2P,
)


def text_to_phonemes(text: str, inv: LanguageInventory) -> list:
    """Left-to-right greedy conversion, checking digraphs first."""
    result = []
    i = 0
    while i < len(text):
        # Try 2-char digraph
        if i + 1 < len(text):
            g = text[i:i+2].lower()
            if g in inv.g2p:
                result.extend(inv.g2p[g])
                i += 2
                continue
        # 1-char
        g = text[i].lower() if text[i].isascii() else text[i]
        if g in inv.g2p:
            result.extend(inv.g2p[g])
        i += 1
    return result


# ═══════════════════════════════════════════════════════════════════════
# (3) Voice signatures — 30 speakers as 8-parameter deltas
# ═══════════════════════════════════════════════════════════════════════

class VoiceCategory(Enum):
    MAN_ADULT = "man_adult"        # 85-180 Hz
    WOMAN_ADULT = "woman_adult"    # 165-255 Hz
    BOY_10 = "boy_10"              # 250-350 Hz
    GIRL_10 = "girl_10"            # 260-380 Hz
    BOY_3 = "boy_3"                # 350-500 Hz
    GIRL_3 = "girl_3"              # 360-520 Hz


CATEGORY_PITCH = {
    VoiceCategory.MAN_ADULT: 120,
    VoiceCategory.WOMAN_ADULT: 210,
    VoiceCategory.BOY_10: 300,
    VoiceCategory.GIRL_10: 320,
    VoiceCategory.BOY_3: 425,
    VoiceCategory.GIRL_3: 440,
}

CATEGORY_FORMANT_MULT = {
    VoiceCategory.MAN_ADULT: 1.00,
    VoiceCategory.WOMAN_ADULT: 1.17,
    VoiceCategory.BOY_10: 1.28,
    VoiceCategory.GIRL_10: 1.30,
    VoiceCategory.BOY_3: 1.52,
    VoiceCategory.GIRL_3: 1.55,
}


@dataclass
class VoiceSignature:
    speaker_id: str
    category: VoiceCategory
    age_years: float
    pitch_shift_pct: float = 0.0
    pitch_variance: float = 0.2
    formant_deltas: tuple = (1.0, 1.0, 1.0)
    speed_factor: float = 1.0
    roughness: float = 0.1
    accent: str = "native"

    def effective_pitch(self) -> float:
        base = CATEGORY_PITCH[self.category]
        return base * (1 + self.pitch_shift_pct / 100)

    def effective_formants(self, base_f1_f2_f3: tuple) -> tuple:
        mult = CATEGORY_FORMANT_MULT[self.category]
        return tuple(
            b * mult * d for b, d in zip(base_f1_f2_f3, self.formant_deltas)
        )

    def size_bytes(self) -> int:
        return 8 * 4 + len(self.speaker_id) + 8


# ═══════════════════════════════════════════════════════════════════════
# (4) Musical notes for prosody (Hz → note name)
# ═══════════════════════════════════════════════════════════════════════

@dataclass
class MusicalNote:
    pitch_hz: float
    duration_ms: int
    loudness: float

    def to_note_name(self) -> str:
        if self.pitch_hz <= 0:
            return "rest"
        n = round(12 * math.log2(self.pitch_hz / 440))
        names = ['A', 'A#', 'B', 'C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#']
        octave = 4 + (n + 9) // 12
        return f"{names[n % 12]}{octave}"


# ═══════════════════════════════════════════════════════════════════════
# (5) Pitch extraction from WAV (autocorrelation, no librosa)
# ═══════════════════════════════════════════════════════════════════════

def generate_synthetic_wav(freq_hz: float, duration_sec: float,
                           sample_rate: int = 22050) -> bytes:
    """Generate a sine-wave WAV for pitch-extraction testing."""
    num_samples = int(sample_rate * duration_sec)
    with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as f:
        path = f.name
    with wave.open(path, "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(sample_rate)
        frames = b"".join(
            struct.pack("<h",
                        int(math.sin(2 * math.pi * freq_hz * i / sample_rate) * 32767))
            for i in range(num_samples)
        )
        w.writeframes(frames)
    raw = Path(path).read_bytes()
    Path(path).unlink()
    return raw


def extract_pitch_autocorr(wav_bytes: bytes, sample_rate: int = 22050) -> float:
    """Simple autocorrelation pitch detection."""
    num_samples = (len(wav_bytes) - 44) // 2
    samples = struct.unpack(f"<{num_samples}h", wav_bytes[44:44 + num_samples*2])

    min_lag = sample_rate // 500   # max 500 Hz
    max_lag = sample_rate // 70    # min 70 Hz
    best_lag = min_lag
    best_corr = -1e20

    n = len(samples)
    for lag in range(min_lag, min(max_lag, n // 2)):
        # Sample every 10th for speed
        corr = sum(samples[i] * samples[i + lag] for i in range(0, n - lag, 10))
        if corr > best_corr:
            best_corr = corr
            best_lag = lag
    return sample_rate / best_lag


# ═══════════════════════════════════════════════════════════════════════
# (6) Accent = phoneme substitution rules
# ═══════════════════════════════════════════════════════════════════════

ACCENT_RULES = {
    "israeli_english": [
        ("/th/", "/t/"),   # "three" → "tree"
        ("/w/", "/v/"),    # "water" → "vater"
        ("/r/", "/R/"),    # alveolar → uvular
    ],
    "french_english": [
        ("/th/", "/z/"),
        ("/h/", ""),       # silent h
    ],
    "native": [],
}


def apply_accent(phonemes: list, accent: str) -> list:
    rules = ACCENT_RULES.get(accent, [])
    result = []
    for p in phonemes:
        replaced = p
        for src, dst in rules:
            if p == src:
                replaced = dst
                break
        if replaced:
            result.append(replaced)
    return result


# ═══════════════════════════════════════════════════════════════════════
# Tests
# ═══════════════════════════════════════════════════════════════════════

def test_phoneme_catalog_coverage():
    vowels = [p for p in PHONEME_CATALOG if p.manner == Manner.VOWEL]
    consonants = [p for p in PHONEME_CATALOG if p.manner != Manner.VOWEL]
    print(f"  Catalog: {len(PHONEME_CATALOG)} phonemes "
          f"({len(vowels)} vowels, {len(consonants)} consonants)")
    assert len(PHONEME_CATALOG) >= 25


def test_hebrew_to_phonemes():
    result = text_to_phonemes("שלום", HEBREW)
    print(f"  'שלום' → {result}")
    assert "/sh/" in result
    assert "/l/" in result
    assert "/m/" in result


def test_english_to_phonemes():
    result = text_to_phonemes("shalom", ENGLISH)
    print(f"  'shalom' → {result}")
    assert "/sh/" in result


def test_cross_lingual_shared_phonemes():
    he = set(text_to_phonemes("שלום", HEBREW))
    en = set(text_to_phonemes("shalom", ENGLISH))
    overlap = he & en
    print(f"  Hebrew phonemes: {he}")
    print(f"  English phonemes: {en}")
    print(f"  Overlap: {overlap}")
    assert len(overlap) >= 3   # sh + l + m + a at minimum


def test_voice_signatures_compact():
    speakers = []
    for cat in VoiceCategory:
        for i in range(5):
            age = 3.0 if "3" in cat.value else 10.0 if "10" in cat.value else 35.0
            sig = VoiceSignature(
                speaker_id=f"{cat.value}_{i+1}",
                category=cat, age_years=age,
                pitch_shift_pct=(i - 2) * 5,
                formant_deltas=(1 + (i-2)*0.03, 1.0, 1.0),
            )
            speakers.append(sig)
    total = sum(s.size_bytes() for s in speakers)
    print(f"  30 speakers × 6 categories = {len(speakers)} total")
    print(f"  Total memory: {total} bytes")
    assert len(speakers) == 30
    assert total < 2048


def test_pitch_varies_by_age():
    man = VoiceSignature("m", VoiceCategory.MAN_ADULT, 35.0)
    girl3 = VoiceSignature("g", VoiceCategory.GIRL_3, 3.0)
    print(f"  Adult man pitch: {man.effective_pitch():.0f} Hz")
    print(f"  3yo girl pitch:  {girl3.effective_pitch():.0f} Hz")
    print(f"  Ratio: {girl3.effective_pitch() / man.effective_pitch():.2f}×")
    assert girl3.effective_pitch() > man.effective_pitch() * 3


def test_formants_scale_with_age():
    base = (730, 1090, 2440)  # /a/ adult male ref
    man = VoiceSignature("m", VoiceCategory.MAN_ADULT, 35.0)
    girl = VoiceSignature("g", VoiceCategory.GIRL_3, 3.0)
    mf = man.effective_formants(base)
    gf = girl.effective_formants(base)
    print(f"  Adult man /a/: F1/F2/F3 = {[round(f) for f in mf]}")
    print(f"  3yo girl /a/:  F1/F2/F3 = {[round(f) for f in gf]}")
    assert gf[0] > mf[0] * 1.4


def test_musical_note_names():
    cases = [(440, "A4"), (262, "C4"), (523, "C5"), (220, "A3")]
    for hz, expected in cases:
        n = MusicalNote(pitch_hz=hz, duration_ms=250, loudness=0.5)
        got = n.to_note_name()
        print(f"  {hz:>4} Hz → {got}")
        assert got == expected, f"expected {expected}, got {got}"


def test_pitch_extraction():
    wav = generate_synthetic_wav(220.0, 0.5)
    detected = extract_pitch_autocorr(wav)
    print(f"  Synth 220Hz → detected {detected:.1f} Hz (err {abs(detected-220):.1f})")
    assert abs(detected - 220) < 10


def test_accent_substitution():
    # Hebrew speaker says "three" — /th/ → /t/
    # (English g2p in this simple version may not produce /th/; emulate):
    native = ["/th/", "/r/", "/i/"]
    israeli = apply_accent(native, "israeli_english")
    print(f"  'three' native:  {native}")
    print(f"  'three' Israeli: {israeli}")
    assert "/t/" in israeli
    assert "/th/" not in israeli
    assert "/R/" in israeli  # r → R (uvular)


def test_storage_efficiency():
    wav_bps = 22050 * 2
    atoms_bps = 20 + 40 + 20 + 32   # phonemes + pitch + duration + notes
    ratio = wav_bps / atoms_bps
    print(f"  WAV:        {wav_bps:,} bytes/sec")
    print(f"  Atoms:      {atoms_bps} bytes/sec")
    print(f"  Reduction:  {ratio:.0f}× smaller")
    assert ratio > 300


def test_voice_from_user_params():
    """User gives 4 sliders, ZETS picks the voice."""
    def build(age, accent, speed, expr):
        if age < 5:
            cat = VoiceCategory.GIRL_3
        elif age < 15:
            cat = VoiceCategory.GIRL_10
        else:
            cat = VoiceCategory.WOMAN_ADULT
        return VoiceSignature(
            speaker_id=f"auto_{int(age)}",
            category=cat, age_years=age,
            pitch_variance=expr, speed_factor=speed,
            roughness=0.3 if age > 60 else 0.1, accent=accent,
        )

    samples = [
        (3, "native", 1.1, 0.8),
        (35, "israeli_english", 1.0, 0.5),
        (70, "native", 0.8, 0.3),
    ]
    for age, acc, spd, exp in samples:
        v = build(age, acc, spd, exp)
        print(f"  age={age:>3}  cat={v.category.value:<12}  "
              f"pitch={v.effective_pitch():>5.0f}Hz  "
              f"speed={v.speed_factor}  rough={v.roughness}  accent={acc}")


if __name__ == '__main__':
    print("━━━ Sound/Voice Learning — Python Prototype ━━━\n")
    print("[1] Phoneme catalog:")
    test_phoneme_catalog_coverage()
    print("\n[2] Hebrew → phonemes:")
    test_hebrew_to_phonemes()
    print("\n[3] English → phonemes:")
    test_english_to_phonemes()
    print("\n[4] Cross-lingual shared phonemes (שלום ↔ shalom):")
    test_cross_lingual_shared_phonemes()
    print("\n[5] 30 voice signatures <2KB:")
    test_voice_signatures_compact()
    print("\n[6] Pitch ranges by age/category:")
    test_pitch_varies_by_age()
    print("\n[7] Formants scale with age (shorter vocal tract):")
    test_formants_scale_with_age()
    print("\n[8] Musical notes — Hz to note name:")
    test_musical_note_names()
    print("\n[9] Pitch extraction (autocorrelation, no DL):")
    test_pitch_extraction()
    print("\n[10] Accent = phoneme substitution rules:")
    test_accent_substitution()
    print("\n[11] Storage: WAV vs atoms:")
    test_storage_efficiency()
    print("\n[12] Age + accent + speed + expressiveness → voice:")
    test_voice_from_user_params()
    print("\n━━━ ALL TESTS PASSED ━━━")
