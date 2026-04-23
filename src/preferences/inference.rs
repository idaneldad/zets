//! # Inference — detect preferences from conversation history
//!
//! Analyzes a slice of `HistoryEntry` messages and returns inferred
//! preference values. Only messages from the owner (`Author::FromSource`)
//! are analyzed.
//!
//! ## Minimum requirements before inferring
//!   - At least 3 owner messages
//!   - At least 80% consistency among signals
//!
//! ## Confidence formula
//!   confidence = min(1.0, signal_count / 5.0)

use crate::preferences::key::PreferenceKey;
use crate::preferences::value::PreferenceValue;
use crate::reader::input::{Author, HistoryEntry};

/// Minimum owner messages before any inference is attempted.
const MIN_MESSAGES: usize = 3;
/// Minimum fraction of messages that must agree on a signal.
const CONSISTENCY_THRESHOLD: f32 = 0.8;
/// Denominator for confidence calculation.
const CONFIDENCE_DENOM: f32 = 5.0;

/// A single inference result: a key-value pair to set.
#[derive(Debug, Clone)]
pub struct InferredPreference {
    pub key: PreferenceKey,
    pub value: PreferenceValue,
}

/// Run all inference signals over a set of history entries.
///
/// Returns a list of (key, value) pairs that should be upserted into
/// the preference store for the owner.
pub fn infer_preferences(messages: &[HistoryEntry], now_ms: i64) -> Vec<InferredPreference> {
    // Collect only owner messages
    let owner_msgs: Vec<&HistoryEntry> = messages
        .iter()
        .filter(|e| e.who == Author::FromSource)
        .collect();

    if owner_msgs.len() < MIN_MESSAGES {
        return vec![];
    }

    let mut results = Vec::new();

    // Signal 1: Length preference from avg message character count
    if let Some(pref) = infer_length(&owner_msgs, now_ms) {
        results.push(pref);
    }

    // Signal 2: Language from dominant script / explicit markers
    if let Some(pref) = infer_language(&owner_msgs, now_ms) {
        results.push(pref);
    }

    // Signal 3: Formality from casual/formal markers
    if let Some(pref) = infer_tone_from_formality(&owner_msgs, now_ms) {
        results.push(pref);
    }

    // Signal 4: Format preference from explicit markers
    if let Some(pref) = infer_format(&owner_msgs, now_ms) {
        results.push(pref);
    }

    // Signal 5: Explicit corrective patterns (shorter/longer)
    if let Some(pref) = infer_corrective_length(&owner_msgs, now_ms) {
        // Corrective overrides signal 1 if found
        results.retain(|r| r.key.as_str() != "length");
        results.push(pref);
    }

    results
}

// ─── Signal: Length ──────────────────────────────────────────────────

fn infer_length(msgs: &[&HistoryEntry], now_ms: i64) -> Option<InferredPreference> {
    let avg_chars = msgs.iter().map(|e| e.content.chars().count()).sum::<usize>() as f32
        / msgs.len() as f32;

    let value = if avg_chars < 50.0 {
        "short"
    } else if avg_chars > 200.0 {
        "long"
    } else {
        return None; // medium is the default; don't emit noise
    };

    // Count how many messages agree with this bucket
    let agreeing = msgs
        .iter()
        .filter(|e| {
            let len = e.content.chars().count();
            if value == "short" {
                len < 50
            } else {
                len > 200
            }
        })
        .count();

    let consistency = agreeing as f32 / msgs.len() as f32;
    if consistency < CONSISTENCY_THRESHOLD {
        return None;
    }

    let signal_strength = (agreeing as f32).min(CONFIDENCE_DENOM);
    let confidence = signal_strength / CONFIDENCE_DENOM;
    let msg_ids: Vec<String> = msgs.iter().map(|e| e.ts_ms.to_string()).collect();

    Some(InferredPreference {
        key: PreferenceKey::new("length"),
        value: PreferenceValue::inferred(value, msg_ids, confidence, now_ms),
    })
}

// ─── Signal: Language ────────────────────────────────────────────────

fn infer_language(msgs: &[&HistoryEntry], now_ms: i64) -> Option<InferredPreference> {
    // Check for explicit language preference patterns first
    let mut explicit_he = 0usize;
    let mut explicit_en = 0usize;
    for msg in msgs {
        let lower = msg.content.to_lowercase();
        if lower.contains("בעברית") || lower.contains("in hebrew") {
            explicit_he += 1;
        }
        if lower.contains("in english") || lower.contains("באנגלית") {
            explicit_en += 1;
        }
    }

    if explicit_he > 0 || explicit_en > 0 {
        let value = if explicit_he >= explicit_en { "he" } else { "en" };
        let count = explicit_he.max(explicit_en);
        let confidence = (count as f32 / CONFIDENCE_DENOM).min(1.0);
        let msg_ids: Vec<String> = msgs.iter().map(|e| e.ts_ms.to_string()).collect();
        return Some(InferredPreference {
            key: PreferenceKey::new("language"),
            value: PreferenceValue::inferred(value, msg_ids, confidence.max(0.6), now_ms),
        });
    }

    // Fall back to dominant script detection
    let (he_count, en_count) = msgs.iter().fold((0usize, 0usize), |(he, en), msg| {
        let hebrew_chars = msg.content.chars().filter(|c| is_hebrew(*c)).count();
        let latin_chars = msg
            .content
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .count();
        (he + hebrew_chars, en + latin_chars)
    });

    let total = he_count + en_count;
    if total == 0 {
        return None;
    }

    let he_ratio = he_count as f32 / total as f32;
    let en_ratio = en_count as f32 / total as f32;

    // Need at least 70% dominance to infer
    let (value, ratio) = if he_ratio > 0.7 {
        ("he", he_ratio)
    } else if en_ratio > 0.7 {
        ("en", en_ratio)
    } else {
        return None; // Mixed — keep "auto"
    };

    // Check per-message consistency
    let agreeing = msgs
        .iter()
        .filter(|e| {
            let hc = e.content.chars().filter(|c| is_hebrew(*c)).count();
            let ec = e
                .content
                .chars()
                .filter(|c| c.is_ascii_alphabetic())
                .count();
            let t = hc + ec;
            if t == 0 {
                return false;
            }
            if value == "he" {
                hc as f32 / t as f32 > 0.5
            } else {
                ec as f32 / t as f32 > 0.5
            }
        })
        .count();

    let consistency = agreeing as f32 / msgs.len() as f32;
    if consistency < CONSISTENCY_THRESHOLD {
        return None;
    }

    let confidence = (ratio * consistency).min(1.0);
    let msg_ids: Vec<String> = msgs.iter().map(|e| e.ts_ms.to_string()).collect();

    Some(InferredPreference {
        key: PreferenceKey::new("language"),
        value: PreferenceValue::inferred(value, msg_ids, confidence, now_ms),
    })
}

fn is_hebrew(c: char) -> bool {
    ('\u{0590}'..='\u{05FF}').contains(&c) || ('\u{FB1D}'..='\u{FB4F}').contains(&c)
}

// ─── Signal: Tone from formality ─────────────────────────────────────

fn infer_tone_from_formality(msgs: &[&HistoryEntry], now_ms: i64) -> Option<InferredPreference> {
    let formal_markers: &[&str] = &[
        "therefore",
        "furthermore",
        "however",
        "consequently",
        "i would like",
        "i am writing",
        "בברכה",
        "בכבוד",
        "ברצוני",
    ];
    let casual_markers: &[&str] = &[
        "hey", "yo", "gonna", "wanna", "lol", "btw", "tbh", "סבבה", "אחי",
    ];

    let mut formal_count = 0usize;
    let mut casual_count = 0usize;

    for msg in msgs {
        let lower = msg.content.to_lowercase();
        let has_formal = formal_markers.iter().any(|m| lower.contains(m));
        let has_casual = casual_markers.iter().any(|m| lower.contains(m));
        if has_formal && !has_casual {
            formal_count += 1;
        } else if has_casual && !has_formal {
            casual_count += 1;
        }
    }

    let total = msgs.len();
    let (value, count) = if formal_count > casual_count && formal_count > 0 {
        ("formal", formal_count)
    } else if casual_count > formal_count && casual_count > 0 {
        ("casual", casual_count)
    } else {
        return None;
    };

    let consistency = count as f32 / total as f32;
    if consistency < CONSISTENCY_THRESHOLD {
        return None;
    }

    let confidence = (count as f32 / CONFIDENCE_DENOM).min(1.0);
    let msg_ids: Vec<String> = msgs.iter().map(|e| e.ts_ms.to_string()).collect();

    Some(InferredPreference {
        key: PreferenceKey::new("tone"),
        value: PreferenceValue::inferred(value, msg_ids, confidence, now_ms),
    })
}

// ─── Signal: Format markers ──────────────────────────────────────────

fn infer_format(msgs: &[&HistoryEntry], now_ms: i64) -> Option<InferredPreference> {
    let bullet_markers: &[&str] = &[
        "bullet list",
        "bullet points",
        "numbered",
        "in a list",
        "רשימה",
        "נקודות",
    ];
    let prose_markers: &[&str] = &["in prose", "as text", "paragraph", "just tell me", "briefly"];

    let mut bullet_count = 0usize;
    let mut prose_count = 0usize;

    for msg in msgs {
        let lower = msg.content.to_lowercase();
        if bullet_markers.iter().any(|m| lower.contains(m)) {
            bullet_count += 1;
        }
        if prose_markers.iter().any(|m| lower.contains(m)) {
            prose_count += 1;
        }
    }

    let (value, count) = if bullet_count > prose_count && bullet_count > 0 {
        ("bullet_list", bullet_count)
    } else if prose_count > bullet_count && prose_count > 0 {
        ("prose", prose_count)
    } else {
        return None;
    };

    let confidence = (count as f32 / CONFIDENCE_DENOM).min(1.0);
    let msg_ids: Vec<String> = msgs.iter().map(|e| e.ts_ms.to_string()).collect();

    Some(InferredPreference {
        key: PreferenceKey::new("format"),
        value: PreferenceValue::inferred(value, msg_ids, confidence, now_ms),
    })
}

// ─── Signal: Corrective length patterns ──────────────────────────────
//
// These are explicit user corrections, not incidental occurrences of
// length-related words. We require at least 2 matching messages to
// distinguish real corrections from accidental word use.

fn infer_corrective_length(msgs: &[&HistoryEntry], now_ms: i64) -> Option<InferredPreference> {
    let short_markers: &[&str] = &[
        "shorter",
        "more briefly",
        "too long",
        "be brief",
        "i said shorter",
        "בקצרה",
        "יותר קצר",
        "קצר יותר",
    ];
    // Use unambiguous corrective phrases — avoid standalone "longer" which
    // appears in ordinary prose ("a much longer message...")
    let long_markers: &[&str] = &[
        "make it longer",
        "more detail",
        "expand on",
        "elaborate",
        "too short",
        "need more",
        "יותר ארוך",
        "הרחב",
    ];

    let mut short_count = 0usize;
    let mut long_count = 0usize;

    for msg in msgs {
        let lower = msg.content.to_lowercase();
        if short_markers.iter().any(|m| lower.contains(m)) {
            short_count += 1;
        }
        if long_markers.iter().any(|m| lower.contains(m)) {
            long_count += 1;
        }
    }

    // Require at least 2 matching messages to avoid false positives from
    // incidental word use (e.g. "a longer explanation..." in normal prose).
    let (value, count) = if short_count >= 2 && short_count > long_count {
        ("short", short_count)
    } else if long_count >= 2 && long_count > short_count {
        ("long", long_count)
    } else {
        return None;
    };

    let confidence = (count as f32 / CONFIDENCE_DENOM).min(1.0).max(0.6);
    let msg_ids: Vec<String> = msgs.iter().map(|e| e.ts_ms.to_string()).collect();

    Some(InferredPreference {
        key: PreferenceKey::new("length"),
        value: PreferenceValue::inferred(value, msg_ids, confidence, now_ms),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::input::{Author, HistoryEntry};

    fn entry(ts: i64, content: &str) -> HistoryEntry {
        HistoryEntry {
            ts_ms: ts,
            who: Author::FromSource,
            content: content.into(),
        }
    }

    fn zets_entry(ts: i64, content: &str) -> HistoryEntry {
        HistoryEntry {
            ts_ms: ts,
            who: Author::FromZets,
            content: content.into(),
        }
    }

    #[test]
    fn test_no_inference_below_3_messages() {
        let msgs = vec![entry(1, "hi"), entry(2, "ok")];
        let result = infer_preferences(&msgs, 3000);
        assert!(result.is_empty());
    }

    #[test]
    fn test_zets_messages_are_excluded() {
        // 2 owner + 3 zets = only 2 owner msgs → no inference
        let msgs = vec![
            entry(1, "short"),
            entry(2, "ok"),
            zets_entry(3, "Here is a much longer response that could skew averages badly."),
            zets_entry(4, "Another long message."),
            zets_entry(5, "Yet another verbose response from ZETS here."),
        ];
        let result = infer_preferences(&msgs, 6000);
        assert!(result.is_empty());
    }

    #[test]
    fn test_inferred_length_short_from_short_messages() {
        let msgs = vec![
            entry(1, "כן"),
            entry(2, "תמשיך"),
            entry(3, "בסדר"),
            entry(4, "טוב"),
            entry(5, "אוקי"),
        ];
        let result = infer_preferences(&msgs, 6000);
        let length_pref = result.iter().find(|r| r.key.as_str() == "length");
        assert!(length_pref.is_some());
        assert_eq!(length_pref.unwrap().value.value, "short");
    }

    #[test]
    fn test_inferred_length_long_from_long_messages() {
        // All 5 messages are clearly > 200 chars each
        let msgs = vec![
            entry(1, "This is a really long message about various topics that goes on and on with many words to make it exceed two hundred chars, so we can test the length detection works correctly with long messages from the user here."),
            entry(2, "Another very long message that talks about many things in great detail, providing extensive context and background information about the subject at hand, which makes this message exceed the two hundred character threshold comfortably."),
            entry(3, "Yet another lengthy response from the user with a lot of content and information packed into it, definitely more than two hundred characters of text to make the test reliable and the inference fires correctly every single time."),
            entry(4, "A fourth lengthy message to ensure consistency across multiple examples, since we need eighty percent consistency to infer the preference correctly here, and this message is well over two hundred characters long."),
            entry(5, "And a fifth long message to help push the signal past the threshold and achieve minimum confidence needed for the length preference to be inferred correctly from the conversation history entries in the test suite."),
        ];
        let result = infer_preferences(&msgs, 6000);
        let length_pref = result.iter().find(|r| r.key.as_str() == "length");
        assert!(length_pref.is_some());
        assert_eq!(length_pref.unwrap().value.value, "long");
    }

    #[test]
    fn test_inferred_language_from_dominant_hebrew() {
        let msgs = vec![
            entry(1, "תמשיך"),
            entry(2, "כן, זה"),
            entry(3, "בקצרה"),
            entry(4, "נקודות בלבד"),
            entry(5, "עכשיו הבא"),
        ];
        let result = infer_preferences(&msgs, 6000);
        let lang_pref = result.iter().find(|r| r.key.as_str() == "language");
        assert!(lang_pref.is_some());
        assert_eq!(lang_pref.unwrap().value.value, "he");
    }

    #[test]
    fn test_inferred_language_from_dominant_english() {
        let msgs = vec![
            entry(1, "please continue"),
            entry(2, "yes that works"),
            entry(3, "got it thanks"),
            entry(4, "okay sounds good"),
            entry(5, "perfect"),
        ];
        let result = infer_preferences(&msgs, 6000);
        let lang_pref = result.iter().find(|r| r.key.as_str() == "language");
        assert!(lang_pref.is_some());
        assert_eq!(lang_pref.unwrap().value.value, "en");
    }

    #[test]
    fn test_hebrew_explicit_preference() {
        let msgs = vec![
            entry(1, "please answer בעברית"),
            entry(2, "תמשיך"),
            entry(3, "כן"),
        ];
        let result = infer_preferences(&msgs, 6000);
        let lang_pref = result.iter().find(|r| r.key.as_str() == "language");
        assert!(lang_pref.is_some());
        assert_eq!(lang_pref.unwrap().value.value, "he");
    }

    #[test]
    fn test_english_explicit_preference() {
        let msgs = vec![
            entry(1, "please respond in English"),
            entry(2, "yes"),
            entry(3, "ok"),
        ];
        let result = infer_preferences(&msgs, 6000);
        let lang_pref = result.iter().find(|r| r.key.as_str() == "language");
        assert!(lang_pref.is_some());
        assert_eq!(lang_pref.unwrap().value.value, "en");
    }

    #[test]
    fn test_no_inference_on_low_consistency() {
        // Mix of short (< 50 chars) and long (> 200 chars): 3 short, 2 long
        // Consistency = 3/5 = 0.6 < 0.8 threshold → no length inference
        let msgs = vec![
            entry(1, "hi"),
            entry(2, "A message with lots of words that provides extensive detail and context about many things going on in the situation, making it well over two hundred characters for sure here."),
            entry(3, "ok"),
            entry(4, "Another wordy message with extensive content and background information to ensure this one also exceeds the two hundred character threshold needed for long detection purposes."),
            entry(5, "short"),
        ];
        let result = infer_preferences(&msgs, 6000);
        // Length should not be inferred due to inconsistency
        let length_pref = result.iter().find(|r| r.key.as_str() == "length");
        assert!(length_pref.is_none());
    }

    #[test]
    fn test_inferred_formality_casual() {
        let msgs = vec![
            entry(1, "hey yo what's up lol"),
            entry(2, "gonna check btw"),
            entry(3, "wanna grab coffee yo"),
            entry(4, "hey lol"),
            entry(5, "yo btw wanna"),
        ];
        let result = infer_preferences(&msgs, 6000);
        let tone_pref = result.iter().find(|r| r.key.as_str() == "tone");
        assert!(tone_pref.is_some());
        assert_eq!(tone_pref.unwrap().value.value, "casual");
    }

    #[test]
    fn test_corrective_length_short() {
        let msgs = vec![
            entry(1, "that was too long"),
            entry(2, "shorter please"),
            entry(3, "be brief"),
            entry(4, "ok"),
            entry(5, "more briefly"),
        ];
        let result = infer_preferences(&msgs, 6000);
        let length_pref = result.iter().find(|r| r.key.as_str() == "length");
        assert!(length_pref.is_some());
        assert_eq!(length_pref.unwrap().value.value, "short");
    }
}
