//! # Style detection — Big Five personality inference from text
//!
//! Phase 2 of Reader. Infers communication style and Big Five (OCEAN)
//! traits from textual features. Used for mirroring — matching the
//! user's register and tone in responses.
//!
//! The 5 OCEAN dimensions inferred:
//!   - **Openness** — curiosity, abstract words, unusual phrasing
//!   - **Conscientiousness** — precision, structure, spelling correctness
//!   - **Extraversion** — exclamations, questions, social words
//!   - **Agreeableness** — politeness, gratitude, warmth
//!   - **Neuroticism** — self-reference, negative affect
//!
//! These are WEAK signals from a single message. Over 20+ messages
//! the averages stabilize — this is why StyleRead is designed to be
//! *updated cumulatively*.

use crate::reader::input::ReadInput;
use crate::reader::reading::{BigFive, StyleRead};

pub struct StyleDetector;

impl StyleDetector {
    pub fn detect(input: &ReadInput) -> StyleRead {
        let msg = input.message;
        let msg_lower = msg.to_lowercase();

        let big_five = infer_big_five(msg, &msg_lower);
        let formality = infer_formality(msg, &msg_lower);
        let tech_density = infer_tech_density(&msg_lower);
        let avg_sentence_len = compute_avg_sentence_len(msg);
        let question_ratio = compute_question_ratio(msg);
        let emo_intensity = infer_emo_intensity(&msg_lower);
        let hedging = infer_hedging(&msg_lower);

        StyleRead {
            big_five,
            formality,
            tech_density,
            avg_sentence_len,
            question_ratio,
            emo_intensity,
            hedging,
        }
    }
}

// ─── Big Five ─────────────────────────────────────────────────────

fn infer_big_five(msg: &str, msg_lower: &str) -> BigFive {
    let mut bf = BigFive::default();

    // Start at 0.5 (unknown) — signals nudge up or down
    bf.openness = 0.5;
    bf.conscientiousness = 0.5;
    bf.extraversion = 0.5;
    bf.agreeableness = 0.5;
    bf.neuroticism = 0.5;

    // ─── Openness: curiosity + abstract vocabulary ──────
    let curious_markers = [
        "wonder", "curious", "fascinating", "interesting", "imagine",
        "מעניין", "מסקרן", "מרתק", "תאר לך",
    ];
    let abstract_markers = [
        "essentially", "fundamentally", "philosophical", "theoretical",
        "concept", "abstraction", "meta",
        "בעצם", "רעיון", "תאורטי", "פילוסופי",
    ];
    for m in curious_markers {
        if msg_lower.contains(m) {
            bf.openness = (bf.openness + 0.1).min(1.0);
        }
    }
    for m in abstract_markers {
        if msg_lower.contains(m) {
            bf.openness = (bf.openness + 0.08).min(1.0);
        }
    }

    // ─── Conscientiousness: structure + precision ───────
    // Numbered lists, bullet points
    if msg.contains("1.") || msg.contains("2.") || msg.contains("- ") {
        bf.conscientiousness = (bf.conscientiousness + 0.15).min(1.0);
    }
    // Specific numbers or dates
    if msg.chars().filter(|c| c.is_ascii_digit()).count() >= 3 {
        bf.conscientiousness = (bf.conscientiousness + 0.1).min(1.0);
    }
    // Excessive typos reduce conscientiousness
    let typo_markers = [" teh ", " dont ", " cant ", " wont ", " youre "];
    for m in typo_markers {
        if msg_lower.contains(m) {
            bf.conscientiousness = (bf.conscientiousness - 0.1).max(0.0);
        }
    }

    // ─── Extraversion: !? + social words ────────────────
    let excl = msg.matches('!').count();
    if excl >= 2 {
        bf.extraversion = (bf.extraversion + 0.15).min(1.0);
    } else if excl == 1 {
        bf.extraversion = (bf.extraversion + 0.07).min(1.0);
    }
    let social_markers = [
        "we", "us", "together", "party", "everyone", "team",
        "אנחנו", "ביחד", "כולם", "צוות",
    ];
    for m in social_markers {
        if msg_lower.split_whitespace().any(|w| w == m) {
            bf.extraversion = (bf.extraversion + 0.05).min(1.0);
        }
    }

    // ─── Agreeableness: politeness + gratitude ──────────
    let polite_markers = [
        "please", "thank you", "thanks", "appreciate", "kind",
        "בבקשה", "תודה", "מעריך", "טוב",
    ];
    for m in polite_markers {
        if msg_lower.contains(m) {
            bf.agreeableness = (bf.agreeableness + 0.1).min(1.0);
        }
    }
    // Hostile words reduce
    let hostile = ["hate", "stupid", "idiot", "garbage", "שונא", "טיפש"];
    for h in hostile {
        if msg_lower.contains(h) {
            bf.agreeableness = (bf.agreeableness - 0.15).max(0.0);
        }
    }

    // ─── Neuroticism: self-reference + negative affect ──
    let self_refs = msg_lower
        .split_whitespace()
        .filter(|w| *w == "i" || *w == "me" || *w == "my" || *w == "אני" || *w == "שלי")
        .count();
    if self_refs >= 5 {
        bf.neuroticism = (bf.neuroticism + 0.15).min(1.0);
    } else if self_refs >= 3 {
        bf.neuroticism = (bf.neuroticism + 0.08).min(1.0);
    }
    let negative_affect = [
        "anxious", "worried", "afraid", "scared", "tired",
        "לחוץ", "דואג", "מפחד", "עייף",
    ];
    for n in negative_affect {
        if msg_lower.contains(n) {
            bf.neuroticism = (bf.neuroticism + 0.1).min(1.0);
        }
    }

    bf
}

// ─── Formality ───────────────────────────────────────────────────

fn infer_formality(msg: &str, msg_lower: &str) -> f32 {
    let mut score: f32 = 0.5;

    // Formal markers
    let formal = [
        "therefore", "furthermore", "however", "consequently",
        "i would like", "i am writing",
        "בברכה", "בכבוד", "ברצוני", "לכבוד",
    ];
    for f in formal {
        if msg_lower.contains(f) {
            score += 0.1;
        }
    }

    // Casual markers
    let casual = [
        "hey", "yo", "gonna", "wanna", "lol", "btw", "tbh",
        "סבבה", "אחי",
    ];
    for c in casual {
        if msg_lower.contains(c) {
            score -= 0.15;
        }
    }

    // Emoji or excessive punctuation → casual
    if msg.contains('😂') || msg.contains('🔥') || msg.contains(":)") || msg.contains(":D") {
        score -= 0.2;
    }

    score.clamp(0.0_f32, 1.0)
}

// ─── Tech density ─────────────────────────────────────────────────

fn infer_tech_density(msg_lower: &str) -> f32 {
    let tech_terms = [
        "api", "json", "http", "https", "function", "algorithm", "query",
        "database", "latency", "throughput", "compile", "deploy", "git",
        "docker", "kubernetes", "async", "mutex", "hash", "cache",
        "רגרסיה", "אלגוריתם", "פונקציה", "שאילתה",
    ];
    let mut count = 0;
    for t in tech_terms {
        if msg_lower.contains(t) {
            count += 1;
        }
    }
    (count as f32 * 0.1).min(1.0)
}

// ─── Sentence length ──────────────────────────────────────────────

fn compute_avg_sentence_len(msg: &str) -> f32 {
    let sentences: Vec<&str> = msg
        .split(|c| c == '.' || c == '!' || c == '?')
        .filter(|s| !s.trim().is_empty())
        .collect();

    if sentences.is_empty() {
        return msg.split_whitespace().count() as f32;
    }

    let total_words: usize = sentences.iter().map(|s| s.split_whitespace().count()).sum();
    total_words as f32 / sentences.len() as f32
}

// ─── Question ratio ───────────────────────────────────────────────

fn compute_question_ratio(msg: &str) -> f32 {
    let questions = msg.matches('?').count();
    let total_terminators = msg.matches(|c: char| c == '.' || c == '!' || c == '?').count();
    if total_terminators == 0 {
        return 0.0;
    }
    questions as f32 / total_terminators as f32
}

// ─── Emo intensity ────────────────────────────────────────────────

fn infer_emo_intensity(msg_lower: &str) -> f32 {
    let intense_emotion = [
        "love", "hate", "amazing", "terrible", "devastated", "ecstatic",
        "thrilled", "furious", "heartbroken",
        "אוהב", "שונא", "מדהים", "נורא",
    ];
    let mut count = 0;
    for e in intense_emotion {
        if msg_lower.contains(e) {
            count += 1;
        }
    }
    (count as f32 * 0.25).min(1.0)
}

// ─── Hedging ──────────────────────────────────────────────────────

fn infer_hedging(msg_lower: &str) -> f32 {
    let hedges = [
        "maybe", "perhaps", "i guess", "i think", "possibly",
        "might", "could be", "not sure", "probably",
        "אולי", "נראה לי", "לא בטוח", "אפשרי",
    ];
    let mut count = 0;
    for h in hedges {
        if msg_lower.contains(h) {
            count += 1;
        }
    }
    (count as f32 * 0.25).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::input::SessionContext;
    use crate::reader::source::{Source, UserRole};

    fn mk<'a>(msg: &'a str, src: &'a Source, sess: &'a SessionContext) -> ReadInput<'a> {
        ReadInput::new(msg, src, &[], sess)
    }

    fn ctx() -> (Source, SessionContext) {
        (
            Source::User {
                id: "t".into(),
                role: UserRole::Owner,
            },
            SessionContext::new("s", 1000),
        )
    }

    #[test]
    fn test_default_unknown_big_five() {
        let (s, ss) = ctx();
        let st = StyleDetector::detect(&mk("hello", &s, &ss));
        // With minimal text, all big five stay near 0.5
        assert!((st.big_five.openness - 0.5).abs() < 0.1);
        assert!((st.big_five.conscientiousness - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_openness_high_from_curiosity() {
        let (s, ss) = ctx();
        let st = StyleDetector::detect(&mk(
            "I wonder about the fundamental nature of concepts - how curious these interesting ideas are.",
            &s, &ss
        ));
        assert!(st.big_five.openness > 0.7);
    }

    #[test]
    fn test_conscientiousness_high_from_structure() {
        let (s, ss) = ctx();
        let st = StyleDetector::detect(&mk(
            "Let me break it down:\n1. First point\n2. Second point (at 14:30)\n3. Third with reference 2024-03-15",
            &s, &ss
        ));
        assert!(st.big_five.conscientiousness > 0.6);
    }

    #[test]
    fn test_extraversion_high_from_excl() {
        let (s, ss) = ctx();
        let st = StyleDetector::detect(&mk(
            "Wow we did it together as a team!!! Everyone contributed!",
            &s, &ss
        ));
        assert!(st.big_five.extraversion > 0.6);
    }

    #[test]
    fn test_agreeableness_high_from_politeness() {
        let (s, ss) = ctx();
        let st = StyleDetector::detect(&mk(
            "Please, thank you so much! I really appreciate your kind help.",
            &s, &ss
        ));
        assert!(st.big_five.agreeableness > 0.7);
    }

    #[test]
    fn test_agreeableness_low_from_hostility() {
        let (s, ss) = ctx();
        let st = StyleDetector::detect(&mk("I hate this stupid garbage", &s, &ss));
        assert!(st.big_five.agreeableness < 0.3);
    }

    #[test]
    fn test_neuroticism_high_from_self_reference() {
        let (s, ss) = ctx();
        let st = StyleDetector::detect(&mk(
            "I am so worried, I feel anxious, I can't sleep, I'm scared of my boss.",
            &s, &ss
        ));
        assert!(st.big_five.neuroticism > 0.7);
    }

    #[test]
    fn test_formality_formal() {
        let (s, ss) = ctx();
        let st = StyleDetector::detect(&mk(
            "I would like to inquire, furthermore, therefore regarding the matter.",
            &s, &ss
        ));
        assert!(st.formality > 0.6);
    }

    #[test]
    fn test_formality_casual() {
        let (s, ss) = ctx();
        let st = StyleDetector::detect(&mk("hey yo gonna wanna grab coffee? lol", &s, &ss));
        assert!(st.formality < 0.4);
    }

    #[test]
    fn test_tech_density() {
        let (s, ss) = ctx();
        let st = StyleDetector::detect(&mk(
            "The API uses JSON over HTTP; function calls cached in Redis, deployed via Docker.",
            &s, &ss
        ));
        assert!(st.tech_density > 0.4);
    }

    #[test]
    fn test_tech_density_low_everyday() {
        let (s, ss) = ctx();
        let st = StyleDetector::detect(&mk("went to the beach yesterday", &s, &ss));
        assert!(st.tech_density < 0.2);
    }

    #[test]
    fn test_avg_sentence_len() {
        let (s, ss) = ctx();
        let st = StyleDetector::detect(&mk("short. another short.", &s, &ss));
        assert!(st.avg_sentence_len < 3.0);

        let st2 = StyleDetector::detect(&mk(
            "This is a considerably longer sentence with many words that should increase the average substantially.",
            &s, &ss
        ));
        assert!(st2.avg_sentence_len > 10.0);
    }

    #[test]
    fn test_question_ratio() {
        let (s, ss) = ctx();
        let st = StyleDetector::detect(&mk(
            "What? Why? When?",
            &s, &ss
        ));
        assert_eq!(st.question_ratio, 1.0);

        let st2 = StyleDetector::detect(&mk("Hello. How are you? Fine.", &s, &ss));
        assert!(st2.question_ratio > 0.2 && st2.question_ratio < 0.5);
    }

    #[test]
    fn test_hedging_detected() {
        let (s, ss) = ctx();
        let st = StyleDetector::detect(&mk("maybe I think it could probably be okay", &s, &ss));
        assert!(st.hedging > 0.3);
    }

    #[test]
    fn test_hedging_hebrew() {
        let (s, ss) = ctx();
        let st = StyleDetector::detect(&mk("אולי לא בטוח, נראה לי שאפשרי", &s, &ss));
        assert!(st.hedging > 0.3);
    }
}
