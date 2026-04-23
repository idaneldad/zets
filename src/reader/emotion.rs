//! # Emotion detection — 8 textual signals
//!
//! Phase 2 of Reader. Detects 8 canonical signals from plain text
//! (no audio, no face). Each is a fast, deterministic check; signals
//! are added to `EmotionRead.signals` as sense_key → strength pairs.
//!
//! The 8 signals:
//!
//! 1. **punctuation_intensity** — "!!!", "???", ALL CAPS
//! 2. **hedging** — "maybe", "I guess", "אולי", "I'm not sure"
//! 3. **fragment** — single words, incomplete sentences
//! 4. **repetition** — words/punctuation repeated immediately
//! 5. **self_blame** — "my fault", "I messed up", "אשמתי"
//! 6. **temporal_urgency** — "NOW", "immediately", "דחוף", "ASAP"
//! 7. **metaphor_distress** — "drowning", "can't breathe", "טובע"
//! 8. **positive_affect** — "love", "amazing", "אהבה", "מדהים"
//!
//! The 8 signals are TEXTUAL only — not semantic deep inference.
//! They're cheap, local, and deterministic. Semantic depth comes
//! from later stages (pragmatics, graph walk).

use crate::reader::input::ReadInput;
use crate::reader::reading::{EmotionRead, EmotionalState};

pub struct EmotionDetector;

impl EmotionDetector {
    /// Detect emotion signals in the input message.
    pub fn detect(input: &ReadInput) -> EmotionRead {
        let msg = input.message;
        let msg_lower = msg.to_lowercase();
        let mut signals = std::collections::HashMap::new();

        // ─── 1. Punctuation intensity ─────────────────────
        let punct_intensity = detect_punctuation_intensity(msg);
        if punct_intensity > 0.0 {
            signals.insert("emotion_signal.punctuation_intensity".to_string(), punct_intensity);
        }

        // ─── 2. Hedging ────────────────────────────────────
        let hedging = detect_hedging(&msg_lower);
        if hedging > 0.0 {
            signals.insert("emotion_signal.hedging".to_string(), hedging);
        }

        // ─── 3. Fragment ───────────────────────────────────
        let fragment = detect_fragment(msg);
        if fragment > 0.0 {
            signals.insert("emotion_signal.fragment".to_string(), fragment);
        }

        // ─── 4. Repetition ─────────────────────────────────
        let repetition = detect_repetition(&msg_lower);
        if repetition > 0.0 {
            signals.insert("emotion_signal.repetition".to_string(), repetition);
        }

        // ─── 5. Self-blame ─────────────────────────────────
        let self_blame = detect_self_blame(&msg_lower);
        if self_blame > 0.0 {
            signals.insert("emotion_signal.self_blame".to_string(), self_blame);
        }

        // ─── 6. Temporal urgency ───────────────────────────
        let urgency = detect_urgency(&msg_lower);
        if urgency > 0.0 {
            signals.insert("emotion_signal.temporal_urgency".to_string(), urgency);
        }

        // ─── 7. Metaphor distress ──────────────────────────
        let distress = detect_metaphor_distress(&msg_lower);
        if distress > 0.0 {
            signals.insert("emotion_signal.metaphor_distress".to_string(), distress);
        }

        // ─── 8. Positive affect ────────────────────────────
        let positive = detect_positive_affect(&msg_lower);
        if positive > 0.0 {
            signals.insert("emotion_signal.positive_affect".to_string(), positive);
        }

        // Overall intensity = max of all signals
        let intensity = signals.values().copied().fold(0.0_f32, f32::max);

        // Dominant state inference
        let dominant = infer_state(&signals);

        // Confidence = 0.5 baseline + 0.1 per active signal, capped at 1.0
        let confidence = (0.5 + 0.1 * signals.len() as f32).min(1.0);

        // Arousal (energy) = max of urgency + anger + distress
        let arousal = [
            signals.get("emotion_signal.punctuation_intensity").copied().unwrap_or(0.0),
            signals.get("emotion_signal.temporal_urgency").copied().unwrap_or(0.0),
            signals.get("emotion_signal.metaphor_distress").copied().unwrap_or(0.0),
            signals.get("emotion_signal.repetition").copied().unwrap_or(0.0),
        ].iter().copied().fold(0.0_f32, f32::max);

        // Valence: positive - negative
        let positive = signals.get("emotion_signal.positive_affect").copied().unwrap_or(0.0);
        let negative = signals.get("emotion_signal.metaphor_distress").copied().unwrap_or(0.0)
            + signals.get("emotion_signal.self_blame").copied().unwrap_or(0.0);
        let valence = (positive - negative).max(-1.0_f32).min(1.0_f32);

        let _ = intensity;
        let _ = confidence;

        EmotionRead {
            signals,
            primary: dominant,
            arousal,
            valence,
        }
    }
}

// ─── Signal 1: Punctuation intensity ──────────────────────────────

fn detect_punctuation_intensity(msg: &str) -> f32 {
    let mut score = 0.0;

    // Count exclamations
    let excl = msg.matches('!').count();
    let quest = msg.matches('?').count();

    // 1 is normal; 2+ is emphatic; 3+ is intense
    if excl >= 3 {
        score += 0.7;
    } else if excl == 2 {
        score += 0.4;
    } else if excl == 1 {
        score += 0.1;
    }

    if quest >= 3 {
        score += 0.5;
    } else if quest == 2 {
        score += 0.2;
    }

    // ALL-CAPS words (min 3 chars, not already-caps abbrev like "API")
    let caps_words = msg
        .split_whitespace()
        .filter(|w| {
            w.len() >= 4 && w.chars().all(|c| c.is_uppercase() || !c.is_alphabetic())
        })
        .count();
    if caps_words > 0 {
        score += (caps_words as f32 * 0.15).min(0.5);
    }

    score.min(1.0)
}

// ─── Signal 2: Hedging ────────────────────────────────────────────

fn detect_hedging(msg_lower: &str) -> f32 {
    let hedges = [
        // English
        "maybe", "perhaps", "i guess", "i think", "not sure", "kind of",
        "sort of", "i suppose", "probably", "might be", "could be",
        "not really", "don't know",
        // Hebrew
        "אולי", "נראה לי", "לא בטוח", "כנראה", "ייתכן", "אני חושב",
        "נדמה לי", "לא יודע",
    ];
    let mut count = 0;
    for h in hedges {
        if msg_lower.contains(h) {
            count += 1;
        }
    }
    (count as f32 * 0.3).min(1.0)
}

// ─── Signal 3: Fragment ───────────────────────────────────────────

fn detect_fragment(msg: &str) -> f32 {
    let trimmed = msg.trim();
    let word_count = trimmed.split_whitespace().count();

    // 0 words = noise
    if word_count == 0 {
        return 0.0;
    }

    // 1-2 words without terminal punctuation = fragment
    let has_terminal = trimmed.ends_with('.')
        || trimmed.ends_with('!')
        || trimmed.ends_with('?')
        || trimmed.ends_with('…');

    if word_count == 1 {
        return 0.8;
    }
    if word_count == 2 && !has_terminal {
        return 0.6;
    }
    if word_count == 3 && !has_terminal {
        return 0.3;
    }

    // Sentence-capital + period = not fragment
    0.0
}

// ─── Signal 4: Repetition ─────────────────────────────────────────

fn detect_repetition(msg_lower: &str) -> f32 {
    let words: Vec<&str> = msg_lower.split_whitespace().collect();
    if words.len() < 2 {
        return 0.0;
    }

    // Adjacent word repetition
    let mut adjacent = 0;
    for i in 1..words.len() {
        if words[i] == words[i - 1] && words[i].len() > 1 {
            adjacent += 1;
        }
    }

    // Character repetition like "whaaaat", "noooo"
    let char_rep = count_char_repetitions(msg_lower);

    ((adjacent as f32 * 0.35) + char_rep).min(1.0)
}

fn count_char_repetitions(msg: &str) -> f32 {
    let mut found = 0;
    let chars: Vec<char> = msg.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if !c.is_alphabetic() {
            i += 1;
            continue;
        }
        let mut run = 1;
        while i + run < chars.len() && chars[i + run] == c {
            run += 1;
        }
        if run >= 4 {
            found += 1;
            i += run;
        } else {
            i += 1;
        }
    }
    (found as f32 * 0.25).min(0.6)
}

// ─── Signal 5: Self-blame ─────────────────────────────────────────

fn detect_self_blame(msg_lower: &str) -> f32 {
    let phrases = [
        "my fault", "i messed up", "i'm stupid", "i'm an idiot",
        "i can't do", "i failed", "i ruined", "i'm terrible at",
        "i'm bad at", "sorry i",
        // Hebrew
        "אשמתי", "זאת אשמתי", "פישלתי", "אני טיפש", "לא הצלחתי",
        "הרסתי", "סליחה שאני",
    ];
    let mut count = 0;
    for p in phrases {
        if msg_lower.contains(p) {
            count += 1;
        }
    }
    (count as f32 * 0.5).min(1.0)
}

// ─── Signal 6: Temporal urgency ───────────────────────────────────

fn detect_urgency(msg_lower: &str) -> f32 {
    let urgent = [
        "now", "immediately", "asap", "urgent", "right now",
        "right away", "emergency", "quickly",
        // Hebrew
        "עכשיו", "דחוף", "מיד", "דחיפות", "מהר", "חירום",
    ];
    let mut score: f32 = 0.0;
    for u in urgent {
        if msg_lower.contains(u) {
            score += 0.35;
        }
    }

    // Check for ALL-CAPS "NOW" or multiple exclamations with urgent words
    if msg_lower.contains("asap") && msg_lower.contains('!') {
        score += 0.2;
    }

    score.min(1.0)
}

// ─── Signal 7: Metaphor distress ──────────────────────────────────

fn detect_metaphor_distress(msg_lower: &str) -> f32 {
    let distress = [
        "drowning", "can't breathe", "falling apart", "losing it",
        "can't take", "breaking down", "overwhelmed", "buried",
        "going crazy", "lost", "stuck",
        // Hebrew
        "טובע", "לא נושם", "מתפרק", "מאבד את זה", "לא יכול יותר",
        "נשבר", "מוצף", "קבור", "משתגע", "תקוע", "אבוד",
    ];
    let mut count = 0;
    for d in distress {
        if msg_lower.contains(d) {
            count += 1;
        }
    }
    (count as f32 * 0.45).min(1.0)
}

// ─── Signal 8: Positive affect ────────────────────────────────────

fn detect_positive_affect(msg_lower: &str) -> f32 {
    let positive = [
        "love", "amazing", "wonderful", "fantastic", "excellent",
        "thrilled", "excited", "happy", "great", "brilliant",
        "awesome", "beautiful", "perfect",
        // Hebrew
        "אוהב", "מדהים", "נפלא", "מעולה", "מצוין", "מרגש", "שמח",
        "יפה", "מושלם", "כיף",
    ];
    let mut count = 0;
    for p in positive {
        if msg_lower.contains(p) {
            count += 1;
        }
    }
    (count as f32 * 0.3).min(1.0)
}

// ─── Infer dominant state ─────────────────────────────────────────

fn infer_state(signals: &std::collections::HashMap<String, f32>) -> EmotionalState {
    let get = |k: &str| signals.get(k).copied().unwrap_or(0.0);

    let distress = get("emotion_signal.metaphor_distress")
        + get("emotion_signal.self_blame")
        + get("emotion_signal.fragment") * 0.5;
    let anger = get("emotion_signal.punctuation_intensity")
        + get("emotion_signal.repetition");
    let urgency = get("emotion_signal.temporal_urgency");
    let positive = get("emotion_signal.positive_affect");
    let uncertainty = get("emotion_signal.hedging");

    // Highest wins, with threshold
    let scores = [
        (EmotionalState::Overwhelmed, distress),
        (EmotionalState::Angry, anger),
        (EmotionalState::Anxious, urgency),
        (EmotionalState::Excited, positive),
        (EmotionalState::Frustrated, uncertainty * 0.7),
    ];

    let (best_state, best_score) = scores
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .copied()
        .unwrap_or((EmotionalState::Neutral, 0.0));

    if best_score > 0.3 {
        best_state
    } else {
        EmotionalState::Neutral
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::input::SessionContext;
    use crate::reader::source::{Source, UserRole};

    fn mk_input<'a>(msg: &'a str, src: &'a Source, sess: &'a SessionContext) -> ReadInput<'a> {
        ReadInput::new(msg, src, &[], sess)
    }

    fn mk_ctx() -> (Source, SessionContext) {
        let src = Source::User {
            id: "test".into(),
            role: UserRole::Owner,
        };
        let sess = SessionContext::new("s", 1000);
        (src, sess)
    }

    #[test]
    fn test_clean_message_no_signals() {
        let (src, sess) = mk_ctx();
        let inp = mk_input("what's the weather today", &src, &sess);
        let e = EmotionDetector::detect(&inp);
        assert!(e.signals.is_empty() || e.signals.len() <= 1);
        assert_eq!(e.primary, EmotionalState::Neutral);
    }

    #[test]
    fn test_exclamations_detected() {
        let (src, sess) = mk_ctx();
        let inp = mk_input("STOP!!!", &src, &sess);
        let e = EmotionDetector::detect(&inp);
        assert!(e.signals.contains_key("emotion_signal.punctuation_intensity"));
        assert!(e.arousal > 0.3);
    }

    #[test]
    fn test_hedging_detected_english() {
        let (src, sess) = mk_ctx();
        let inp = mk_input("maybe I think it could be fine", &src, &sess);
        let e = EmotionDetector::detect(&inp);
        assert!(e.signals.contains_key("emotion_signal.hedging"));
    }

    #[test]
    fn test_hedging_detected_hebrew() {
        let (src, sess) = mk_ctx();
        let inp = mk_input("אולי לא בטוח שזה יעבוד", &src, &sess);
        let e = EmotionDetector::detect(&inp);
        assert!(e.signals.contains_key("emotion_signal.hedging"));
    }

    #[test]
    fn test_fragment_detected() {
        let (src, sess) = mk_ctx();
        let inp = mk_input("help", &src, &sess);
        let e = EmotionDetector::detect(&inp);
        assert!(e.signals.contains_key("emotion_signal.fragment"));
    }

    #[test]
    fn test_self_blame_hebrew() {
        let (src, sess) = mk_ctx();
        let inp = mk_input("אשמתי, פישלתי שוב", &src, &sess);
        let e = EmotionDetector::detect(&inp);
        assert!(e.signals.contains_key("emotion_signal.self_blame"));
        assert_eq!(e.primary, EmotionalState::Overwhelmed);
    }

    #[test]
    fn test_urgency_english() {
        let (src, sess) = mk_ctx();
        let inp = mk_input("I need this ASAP, right now!", &src, &sess);
        let e = EmotionDetector::detect(&inp);
        assert!(e.signals.contains_key("emotion_signal.temporal_urgency"));
    }

    #[test]
    fn test_metaphor_distress() {
        let (src, sess) = mk_ctx();
        let inp = mk_input("I'm drowning in emails, can't take it", &src, &sess);
        let e = EmotionDetector::detect(&inp);
        assert!(e.signals.contains_key("emotion_signal.metaphor_distress"));
        assert_eq!(e.primary, EmotionalState::Overwhelmed);
    }

    #[test]
    fn test_positive_affect() {
        let (src, sess) = mk_ctx();
        let inp = mk_input("This is amazing! I love it!", &src, &sess);
        let e = EmotionDetector::detect(&inp);
        assert!(e.signals.contains_key("emotion_signal.positive_affect"));
        assert_eq!(e.primary, EmotionalState::Excited);
    }

    #[test]
    fn test_character_repetition() {
        let (src, sess) = mk_ctx();
        let inp = mk_input("whaaaaat nooooo", &src, &sess);
        let e = EmotionDetector::detect(&inp);
        assert!(e.signals.contains_key("emotion_signal.repetition"));
    }

    #[test]
    fn test_multiple_signals() {
        let (src, sess) = mk_ctx();
        let inp = mk_input("אשמתי!!! אני לא יכול יותר!!!", &src, &sess);
        let e = EmotionDetector::detect(&inp);
        // Self-blame + punct + distress
        assert!(e.signals.len() >= 2);
        assert_eq!(e.primary, EmotionalState::Overwhelmed);
    }

    #[test]
    fn test_arousal_grows_with_signals() {
        let (src, sess) = mk_ctx();
        let inp1 = mk_input("hello", &src, &sess);
        let inp2 = mk_input("MAYBE!!! I don't know... I'm stuck!!!", &src, &sess);
        let e1 = EmotionDetector::detect(&inp1);
        let e2 = EmotionDetector::detect(&inp2);
        assert!(e2.arousal > e1.arousal);
    }
}
