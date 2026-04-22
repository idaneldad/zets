//! Corpus acquisition — download + clean text/dialogue corpora for training.
//!
//! The document Idan shared (22.04.2026) identifies 5 corpus sources:
//!   1. Reddit (r/Israel, r/CasualUK, etc.)  — scraping
//!   2. OpenSubtitles                          — download + parse
//!   3. HuggingFace datasets                  — HTTP API
//!   4. Podcast transcripts                   — Whisper + parse
//!   5. Telegram/Discord                      — NOT recommended (legal)
//!
//! This module implements the MINIMUM viable pipeline:
//!   - fetch_huggingface_dataset: download a dataset split (JSONL format)
//!   - scrub_pii: remove phones, emails, handles, URLs
//!   - detect_intent_emotion: heuristic auto-tagger for raw text
//!
//! We deliberately don't integrate Reddit/Discord scraping here —
//! those need per-site rate-limiting, TOS compliance, and user auth
//! that belongs in a separate agent process. HuggingFace is the path
//! of least legal risk: public licensed datasets only.
//!
//! Zero-dep: uses curl for HTTP. Aligned with ZETS policy.

use std::path::Path;
use std::process::Command;

use crate::dialogue::{ConvOutcome, Conversation, DialogTurn, Emotion, Intent};

// ────────────────────────────────────────────────────────────────
// HuggingFace dataset fetcher
// ────────────────────────────────────────────────────────────────

/// Known safe datasets — map a short name to the HuggingFace URL pattern.
/// Using datasets.server API which returns JSONL rows.
pub fn known_dataset_url(name: &str) -> Option<String> {
    match name {
        // Empathetic Dialogues: 25K conversations with emotion labels
        "empathetic_dialogues" => Some(
            "https://datasets-server.huggingface.co/rows?\
             dataset=empathetic_dialogues&config=default&split=train&offset=0&length=100".into()
        ),
        // DailyDialog: everyday conversations
        "daily_dialog" => Some(
            "https://datasets-server.huggingface.co/rows?\
             dataset=daily_dialog&config=default&split=train&offset=0&length=100".into()
        ),
        // Persona-Chat
        "persona_chat" => Some(
            "https://datasets-server.huggingface.co/rows?\
             dataset=bavard/personachat_truecased&config=full&split=train&offset=0&length=100".into()
        ),
        _ => None,
    }
}

/// Fetch a dataset preview from HuggingFace (up to 100 rows).
/// Returns the raw JSON text for caller to parse.
pub fn fetch_huggingface_preview(name: &str) -> Result<String, String> {
    let url = known_dataset_url(name)
        .ok_or_else(|| format!("unknown dataset: {}. Known: empathetic_dialogues, daily_dialog, persona_chat", name))?;

    let output = Command::new("curl")
        .args([
            "-sS",
            "--max-time", "30",
            "--fail",
            "-H", "User-Agent: zets-corpus-fetcher/1.0",
            &url,
        ])
        .output()
        .map_err(|e| format!("curl failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("curl returned error: {}", stderr));
    }

    let body = String::from_utf8(output.stdout)
        .map_err(|e| format!("non-utf8 response: {}", e))?;

    if body.is_empty() {
        return Err("empty response from HuggingFace".into());
    }

    Ok(body)
}

/// Save raw response to a file for later processing.
pub fn save_raw_response<P: AsRef<Path>>(body: &str, path: P) -> Result<(), String> {
    std::fs::write(path.as_ref(), body)
        .map_err(|e| format!("write failed: {}", e))
}

// ────────────────────────────────────────────────────────────────
// PII scrubbing — the document specifically warns about this
// ────────────────────────────────────────────────────────────────

/// Remove common PII from text before ingestion.
/// This is explicitly required by GDPR + the corpus document.
pub fn scrub_pii(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut iter = text.chars().peekable();

    while let Some(ch) = iter.next() {
        // Emails: anything with @ followed by a word
        if ch == '@' {
            // Check if previous chars look like an email and skip to next whitespace
            // Simplified: just flag @-prefixed tokens
            out.push_str("[EMAIL]");
            while let Some(&next) = iter.peek() {
                if next.is_whitespace() || is_punct_boundary(next) { break; }
                iter.next();
            }
            continue;
        }
        out.push(ch);
    }

    // Scrub phones: sequences of 7-15 digits (possibly with +, spaces, dashes, parens)
    out = scrub_phone_numbers(&out);

    // Scrub URLs
    out = scrub_urls(&out);

    // Scrub @handles (Twitter-style)
    out = scrub_handles(&out);

    out
}

fn is_punct_boundary(c: char) -> bool {
    matches!(c, '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}' | '"' | '\'')
}

fn scrub_phone_numbers(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check if we're at the start of a phone-like sequence
        let start = i;
        let mut digit_count = 0;
        let mut j = i;

        while j < chars.len() {
            let c = chars[j];
            if c.is_ascii_digit() {
                digit_count += 1;
                j += 1;
            } else if matches!(c, ' ' | '-' | '+' | '(' | ')' | '.') && digit_count > 0 {
                j += 1;
            } else {
                break;
            }
        }

        // Phone: 7-15 digits total, at least one connector or digit span >= 4
        if digit_count >= 7 && digit_count <= 15 && (j - start) >= 7 {
            out.push_str("[PHONE]");
            i = j;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }

    out
}

fn scrub_urls(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let lower = text.to_lowercase();
    let chars: Vec<char> = text.chars().collect();
    let lchars: Vec<char> = lower.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check for http://, https://, www.
        let is_http = i + 7 < lchars.len() && lchars[i..i+7].iter().collect::<String>() == "http://";
        let is_https = i + 8 < lchars.len() && lchars[i..i+8].iter().collect::<String>() == "https://";
        let is_www = i + 4 < lchars.len() && lchars[i..i+4].iter().collect::<String>() == "www.";

        if is_http || is_https || is_www {
            out.push_str("[URL]");
            // Skip until whitespace
            while i < chars.len() && !chars[i].is_whitespace() {
                i += 1;
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }

    out
}

fn scrub_handles(text: &str) -> String {
    // @username — but we already handled @ as email. Check for standalone @word at word boundary
    // Our email rule already catches @word; this is a no-op for safety.
    text.to_string()
}

// ────────────────────────────────────────────────────────────────
// Heuristic intent + emotion auto-tagger
// ────────────────────────────────────────────────────────────────

/// Guess the intent of an utterance based on simple lexical cues.
/// This is deliberately low-precision — for production, use an LLM adapter.
/// But for bootstrap training on raw dialogues, this gets us started.
pub fn guess_intent(text: &str) -> Intent {
    let lower = text.to_lowercase();
    let trimmed = lower.trim();

    // Questions end with ?
    if trimmed.ends_with('?') {
        if trimmed.starts_with("can you") || trimmed.starts_with("could you")
            || trimmed.starts_with("would you") || trimmed.starts_with("will you") {
            return Intent::Request;
        }
        if trimmed.starts_with("what do you mean") || trimmed.contains("clarify") {
            return Intent::Clarify;
        }
        return Intent::Question;
    }

    // Greetings
    if trimmed.starts_with("hi") || trimmed.starts_with("hello")
        || trimmed.starts_with("hey") || trimmed == "hi" {
        return Intent::Greet;
    }
    if trimmed.starts_with("bye") || trimmed.starts_with("goodbye")
        || trimmed.starts_with("see you") {
        return Intent::Farewell;
    }

    // Complaints
    if lower.contains("broken") || lower.contains("doesn't work")
        || lower.contains("frustrated") || lower.contains("terrible")
        || lower.contains("awful") || lower.contains("again") {
        return Intent::Complain;
    }

    // Empathy
    if lower.contains("sorry") || lower.contains("that sounds")
        || lower.contains("i understand") || lower.contains("must be hard") {
        return Intent::Empathize;
    }

    // Decline
    if trimmed.starts_with("no") || lower.contains("i don't want")
        || lower.contains("not interested") {
        return Intent::Decline;
    }

    // Agreement
    if trimmed.starts_with("yes") || trimmed.starts_with("thanks")
        || trimmed.starts_with("thank you") || lower.contains("sounds good")
        || lower.contains("agree") || lower.contains("okay") {
        return Intent::Agree;
    }

    // Requests
    if lower.contains("please") || lower.contains("could you")
        || trimmed.starts_with("i need") || trimmed.starts_with("i want") {
        return Intent::Request;
    }

    // Default: informational statement
    Intent::Inform
}

/// Guess the emotion of an utterance based on lexical cues.
/// Again, low precision. Good enough for bootstrap.
pub fn guess_emotion(text: &str) -> Emotion {
    let lower = text.to_lowercase();

    // Sadness
    let sad_words = ["sad", "lost", "died", "passed away", "depressed", "lonely",
                     "hurt", "crying", "miserable", "devastated", "grief"];
    if sad_words.iter().any(|w| lower.contains(w)) {
        return Emotion::Sadness;
    }

    // Anger
    let anger_words = ["angry", "mad", "furious", "pissed", "broken", "frustrated",
                       "awful", "terrible", "hate", "ridiculous"];
    if anger_words.iter().any(|w| lower.contains(w)) {
        return Emotion::Anger;
    }

    // Joy
    let joy_words = ["happy", "great", "awesome", "wonderful", "love it",
                     "amazing", "worked", "thank", "excellent"];
    if joy_words.iter().any(|w| lower.contains(w)) {
        return Emotion::Joy;
    }

    // Fear
    let fear_words = ["scared", "afraid", "worried", "anxious", "nervous", "terrified"];
    if fear_words.iter().any(|w| lower.contains(w)) {
        return Emotion::Fear;
    }

    // Trust
    let trust_words = ["appreciate", "means a lot", "thanks for", "grateful"];
    if trust_words.iter().any(|w| lower.contains(w)) {
        return Emotion::Trust;
    }

    // Surprise
    let surprise_words = ["wow", "unbelievable", "really?", "no way"];
    if surprise_words.iter().any(|w| lower.contains(w)) {
        return Emotion::Surprise;
    }

    Emotion::Neutral
}

/// Guess outcome of a full conversation based on last turn + emotional trajectory.
pub fn guess_outcome(turns: &[DialogTurn]) -> ConvOutcome {
    if turns.is_empty() { return ConvOutcome::Ongoing; }

    let last = &turns[turns.len() - 1];
    let last_lower = last.text.to_lowercase();

    // Explicit converted signals
    if last_lower.contains("i'll take") || last_lower.contains("i'll buy")
        || last_lower.contains("let's do it") || last_lower.contains("sign me up") {
        return ConvOutcome::Converted;
    }

    // Explicit resolved — last message is agreement or thanks
    if matches!(last.intent, Intent::Agree | Intent::Farewell) {
        return ConvOutcome::Resolved;
    }

    // Escalated — angry streaks
    let anger_count = turns.iter().filter(|t| t.emotion == Emotion::Anger).count();
    if anger_count >= 2 || last_lower.contains("escalate")
        || last_lower.contains("supervisor") || last_lower.contains("manager") {
        return ConvOutcome::Escalated;
    }

    // Short conversations with unanswered questions = abandoned
    if turns.len() <= 2 && matches!(last.intent, Intent::Question | Intent::Request) {
        return ConvOutcome::Abandoned;
    }

    ConvOutcome::Resolved
}

// ────────────────────────────────────────────────────────────────
// Build a Conversation from raw alternating speaker lines
// ────────────────────────────────────────────────────────────────

/// Build a Conversation from a list of (speaker_name, text) pairs.
/// Intent + emotion are auto-guessed. Outcome derived from final turn.
pub fn build_conversation_from_turns(
    id: &str,
    source: &str,
    speakers_and_texts: &[(String, String)],
    speaker_resolver: &mut dyn FnMut(&str) -> crate::atoms::AtomId,
) -> Conversation {
    let turns: Vec<DialogTurn> = speakers_and_texts.iter().enumerate()
        .map(|(i, (speaker, raw_text))| {
            let cleaned = scrub_pii(raw_text);
            let intent = guess_intent(&cleaned);
            let emotion = guess_emotion(&cleaned);
            DialogTurn {
                speaker: speaker_resolver(speaker),
                text: cleaned,
                intent,
                emotion,
                turn_index: i as u32,
            }
        })
        .collect();

    let outcome = guess_outcome(&turns);

    Conversation {
        id: id.to_string(),
        source: source.to_string(),
        outcome,
        turns,
    }
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrub_email_replaces_with_token() {
        let text = "Contact me at john@example.com for details.";
        let cleaned = scrub_pii(text);
        assert!(!cleaned.contains("example.com"));
        assert!(cleaned.contains("[EMAIL]"));
    }

    #[test]
    fn scrub_phone_hebrew_format() {
        let text = "Call me at 054-123-4567 or 052 987 6543 please.";
        let cleaned = scrub_pii(text);
        assert!(!cleaned.contains("054-123-4567"));
        assert!(cleaned.contains("[PHONE]"));
    }

    #[test]
    fn scrub_phone_international() {
        let text = "My number is +1 (555) 123-4567 if you need.";
        let cleaned = scrub_pii(text);
        assert!(!cleaned.contains("555"));
        assert!(cleaned.contains("[PHONE]"));
    }

    #[test]
    fn scrub_url_https() {
        let text = "Check out https://example.com/page for info.";
        let cleaned = scrub_pii(text);
        assert!(!cleaned.contains("example.com"));
        assert!(cleaned.contains("[URL]"));
    }

    #[test]
    fn scrub_url_www() {
        let text = "Visit www.wikipedia.org today.";
        let cleaned = scrub_pii(text);
        assert!(!cleaned.contains("wikipedia"));
        assert!(cleaned.contains("[URL]"));
    }

    #[test]
    fn scrub_preserves_normal_text() {
        let text = "The capital of France is Paris.";
        let cleaned = scrub_pii(text);
        assert_eq!(cleaned, text);
    }

    #[test]
    fn guess_intent_question() {
        assert_eq!(guess_intent("What is the capital?"), Intent::Question);
        assert_eq!(guess_intent("Can you help me?"), Intent::Request);
        assert_eq!(guess_intent("What do you mean by that?"), Intent::Clarify);
    }

    #[test]
    fn guess_intent_greeting() {
        assert_eq!(guess_intent("Hi there"), Intent::Greet);
        assert_eq!(guess_intent("Hello everyone"), Intent::Greet);
        assert_eq!(guess_intent("Goodbye for now"), Intent::Farewell);
    }

    #[test]
    fn guess_intent_complaint() {
        assert_eq!(guess_intent("This is broken again"), Intent::Complain);
        assert_eq!(guess_intent("The service is terrible"), Intent::Complain);
    }

    #[test]
    fn guess_intent_empathy() {
        assert_eq!(guess_intent("I'm sorry to hear that"), Intent::Empathize);
        assert_eq!(guess_intent("That sounds really hard"), Intent::Empathize);
    }

    #[test]
    fn guess_intent_agree() {
        assert_eq!(guess_intent("Yes of course"), Intent::Agree);
        assert_eq!(guess_intent("Thanks so much"), Intent::Agree);
        assert_eq!(guess_intent("Sounds good to me"), Intent::Agree);
    }

    #[test]
    fn guess_emotion_sadness() {
        assert_eq!(guess_emotion("I lost my job"), Emotion::Sadness);
        assert_eq!(guess_emotion("My dog passed away"), Emotion::Sadness);
    }

    #[test]
    fn guess_emotion_anger() {
        assert_eq!(guess_emotion("This is broken again"), Emotion::Anger);
        assert_eq!(guess_emotion("I am furious"), Emotion::Anger);
    }

    #[test]
    fn guess_emotion_joy() {
        assert_eq!(guess_emotion("That's amazing, thank you!"), Emotion::Joy);
        assert_eq!(guess_emotion("It worked perfectly"), Emotion::Joy);
    }

    #[test]
    fn guess_emotion_neutral_default() {
        assert_eq!(guess_emotion("The sky is blue"), Emotion::Neutral);
    }

    #[test]
    fn guess_outcome_agree_last_turn_resolved() {
        let turns = vec![
            DialogTurn { speaker: 1, text: "help?".into(), intent: Intent::Request, emotion: Emotion::Neutral, turn_index: 0 },
            DialogTurn { speaker: 2, text: "here".into(), intent: Intent::Inform, emotion: Emotion::Neutral, turn_index: 1 },
            DialogTurn { speaker: 1, text: "thanks".into(), intent: Intent::Agree, emotion: Emotion::Trust, turn_index: 2 },
        ];
        assert_eq!(guess_outcome(&turns), ConvOutcome::Resolved);
    }

    #[test]
    fn guess_outcome_anger_streak_escalated() {
        let turns = vec![
            DialogTurn { speaker: 1, text: "broken".into(), intent: Intent::Complain, emotion: Emotion::Anger, turn_index: 0 },
            DialogTurn { speaker: 1, text: "again".into(), intent: Intent::Complain, emotion: Emotion::Anger, turn_index: 1 },
            DialogTurn { speaker: 2, text: "let me escalate".into(), intent: Intent::Inform, emotion: Emotion::Neutral, turn_index: 2 },
        ];
        assert_eq!(guess_outcome(&turns), ConvOutcome::Escalated);
    }

    #[test]
    fn guess_outcome_empty_turns() {
        assert_eq!(guess_outcome(&[]), ConvOutcome::Ongoing);
    }

    #[test]
    fn known_dataset_url_recognized_names() {
        assert!(known_dataset_url("empathetic_dialogues").is_some());
        assert!(known_dataset_url("daily_dialog").is_some());
        assert!(known_dataset_url("persona_chat").is_some());
        assert!(known_dataset_url("fake_unknown_dataset").is_none());
    }

    #[test]
    fn build_conversation_from_turns_basic() {
        use std::collections::HashMap;
        let mut cache: HashMap<String, crate::atoms::AtomId> = HashMap::new();
        let mut next_id: crate::atoms::AtomId = 0;
        let mut resolver = |name: &str| -> crate::atoms::AtomId {
            if let Some(&id) = cache.get(name) { return id; }
            next_id += 1;
            cache.insert(name.to_string(), next_id);
            next_id
        };

        let turns = vec![
            ("user".to_string(), "I lost my job yesterday".to_string()),
            ("ai".to_string(), "I'm sorry to hear that".to_string()),
            ("user".to_string(), "Thanks for listening".to_string()),
        ];

        let conv = build_conversation_from_turns(
            "c1", "test", &turns, &mut resolver,
        );

        assert_eq!(conv.turns.len(), 3);
        assert_eq!(conv.turns[0].emotion, Emotion::Sadness);
        assert_eq!(conv.turns[1].intent, Intent::Empathize);
        assert_eq!(conv.turns[2].intent, Intent::Agree);
        // Outcome: last turn is Agree -> Resolved
        assert_eq!(conv.outcome, ConvOutcome::Resolved);
    }
}
