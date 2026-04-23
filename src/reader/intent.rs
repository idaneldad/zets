//! # Intent detection — pragmatic reading of the message
//!
//! Phase 2 of Reader. Distinguishes surface meaning from pragmatic meaning.
//!
//! Example: "can you pass the salt?" → literal: question about ability,
//! pragmatic: request for the salt.
//!
//! The detector uses a mix of:
//!   - Surface pattern matching (question marks, imperatives, interjections)
//!   - Speech-act classification (5 canonical acts)
//!   - Ambiguity scoring (how many equally-plausible readings?)
//!   - Hint detection (Boredom, Leaving, Urgent)

use crate::reader::input::ReadInput;
use crate::reader::reading::{Hint, IntentRead, PragmaticIntent};

pub struct IntentDetector;

impl IntentDetector {
    pub fn detect(input: &ReadInput) -> IntentRead {
        let msg = input.message.trim();
        let msg_lower = msg.to_lowercase();

        let literal = msg.to_string();
        let pragmatic = infer_pragmatic(&msg_lower, msg);
        let ambiguity = score_ambiguity(&msg_lower, msg);
        let topic = infer_topic(&msg_lower);
        let hints = detect_hints(&msg_lower, msg);
        let confidence = (1.0 - ambiguity * 0.5).max(0.3);

        // The PragmaticIntent enum wraps Hint values.
        // If we detected hints, use the first one as PragmaticIntent::Hint
        let pragmatic = if !hints.is_empty() && matches!(pragmatic, PragmaticIntent::Literal) {
            PragmaticIntent::Hint(hints[0])
        } else {
            pragmatic
        };
        let _ = confidence;

        IntentRead {
            literal,
            pragmatic,
            ambiguity,
            topic,
        }
    }
}

/// Map surface form to pragmatic speech act.
fn infer_pragmatic(msg_lower: &str, original: &str) -> PragmaticIntent {
    // Indirect request patterns ("can you...", "could you...", "would you mind")
    if msg_lower.starts_with("can you")
        || msg_lower.starts_with("could you")
        || msg_lower.starts_with("would you")
        || msg_lower.starts_with("will you")
        || msg_lower.starts_with("תוכל")
        || msg_lower.starts_with("אפשר")
        || msg_lower.starts_with("אתה יכול")
    {
        return PragmaticIntent::ImplicitHelp;
    }

    // Direct imperative ("give me", "show me", "tell me")
    let imperative_starts = [
        "give me", "show me", "tell me", "send me", "find",
        "תן לי", "הראה לי", "ספר לי", "שלח לי", "מצא",
    ];
    for s in imperative_starts {
        if msg_lower.starts_with(s) {
            return PragmaticIntent::ImplicitHelp;
        }
    }

    // Question (?, interrogative)
    if original.ends_with('?')
        || msg_lower.starts_with("what ")
        || msg_lower.starts_with("why ")
        || msg_lower.starts_with("how ")
        || msg_lower.starts_with("when ")
        || msg_lower.starts_with("where ")
        || msg_lower.starts_with("who ")
        || msg_lower.starts_with("is it")
        || msg_lower.starts_with("are you")
        || msg_lower.starts_with("מה ")
        || msg_lower.starts_with("איך ")
        || msg_lower.starts_with("מתי ")
        || msg_lower.starts_with("איפה ")
        || msg_lower.starts_with("מי ")
        || msg_lower.starts_with("למה ")
    {
        return PragmaticIntent::Literal;
    }

    // Expressive (emotion-heavy, no request)
    let expressive_markers = [
        "wow", "damn", "great", "oh no", "omg",
        "וואו", "בחיי", "אוי",
    ];
    if expressive_markers.iter().any(|m| msg_lower.contains(m)) {
        return PragmaticIntent::Literal;
    }

    // Complaint / grievance
    let complaint_markers = [
        "this doesn't work", "this is broken", "not working", "failed again",
        "לא עובד", "זה לא פועל", "שוב נכשל", "זה רע",
    ];
    if complaint_markers.iter().any(|m| msg_lower.contains(m)) {
        return PragmaticIntent::ImplicitHelp;
    }

    // Greeting
    let greetings = [
        "hello", "hi ", "hey", "good morning", "good afternoon", "good evening",
        "שלום", "היי", "בוקר טוב", "ערב טוב", "צהריים טובים",
    ];
    if greetings.iter().any(|g| msg_lower.starts_with(g) || msg_lower == g.trim()) {
        return PragmaticIntent::Literal;
    }

    // Default: literal statement
    PragmaticIntent::Literal
}

/// How many equally-plausible readings exist?
fn score_ambiguity(msg_lower: &str, original: &str) -> f32 {
    let mut ambiguity = 0.0;

    // Very short messages are ambiguous
    let word_count = msg_lower.split_whitespace().count();
    if word_count <= 2 {
        ambiguity += 0.4;
    } else if word_count <= 4 {
        ambiguity += 0.2;
    }

    // No punctuation + no clear verb = ambiguous
    let has_terminal = original.ends_with('.')
        || original.ends_with('!')
        || original.ends_with('?');
    if !has_terminal && word_count > 5 {
        ambiguity += 0.15;
    }

    // Pronouns without clear referent
    let floating_pronouns = ["it", "this", "that", "they", "זה", "זאת", "הם"];
    let floating_count = floating_pronouns
        .iter()
        .filter(|p| msg_lower.split_whitespace().any(|w| w == **p))
        .count();
    ambiguity += (floating_count as f32 * 0.1).min(0.3);

    ambiguity.min(1.0)
}

/// Infer domain/topic if recognizable.
fn infer_topic(msg_lower: &str) -> Option<String> {
    let topics: &[(&str, &[&str])] = &[
        ("domain.finance", &[
            "money", "payment", "invoice", "budget", "stock", "price",
            "כסף", "תשלום", "חשבונית", "תקציב", "מחיר",
        ]),
        ("domain.tech", &[
            "code", "bug", "error", "function", "api", "deploy",
            "קוד", "באג", "שגיאה", "פונקציה",
        ]),
        ("domain.health", &[
            "pain", "doctor", "symptom", "medicine", "tired", "sick",
            "כאב", "רופא", "תסמין", "תרופה", "עייף", "חולה",
        ]),
        ("domain.relationship", &[
            "friend", "family", "partner", "wife", "husband", "daughter", "son",
            "חבר", "משפחה", "בן זוג", "אישה", "בעל", "בת", "בן",
        ]),
        ("domain.work", &[
            "job", "boss", "meeting", "deadline", "client", "project",
            "עבודה", "מנהל", "פגישה", "לקוח", "פרויקט",
        ]),
    ];

    for (topic_id, keywords) in topics {
        for kw in *keywords {
            if msg_lower.contains(kw) {
                return Some(topic_id.to_string());
            }
        }
    }
    None
}

/// Detect behavioral hints (Boredom, Leaving, Urgent).
fn detect_hints(msg_lower: &str, _original: &str) -> Vec<Hint> {
    let mut hints = Vec::new();

    // Boredom: short, no energy
    let boredom_markers = [
        "ok", "sure", "whatever", "k", "yeah",
        "בסדר", "סבבה", "לא משנה",
    ];
    if boredom_markers.iter().any(|b| msg_lower.trim() == *b) {
        hints.push(Hint::Boredom);
    }

    // Leaving
    let leaving_markers = [
        "bye", "goodbye", "gotta go", "gtg", "later", "talk later",
        "ביי", "שלום", "נדבר אחכ", "אני יוצא",
    ];
    if leaving_markers.iter().any(|l| msg_lower.contains(l)) {
        hints.push(Hint::Leaving);
    }

    // Urgent (also handled in emotion, but surfaced as hint too)
    let urgent_markers = [
        "urgent", "asap", "emergency", "now",
        "דחוף", "חירום", "עכשיו",
    ];
    if urgent_markers.iter().any(|u| msg_lower.contains(u)) {
        hints.push(Hint::Urgent);
    }

    hints
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
    fn test_indirect_request() {
        let (s, ss) = ctx();
        let i = IntentDetector::detect(&mk("Can you help me write this?", &s, &ss));
        assert_eq!(i.pragmatic, PragmaticIntent::ImplicitHelp);
    }

    #[test]
    fn test_indirect_request_hebrew() {
        let (s, ss) = ctx();
        let i = IntentDetector::detect(&mk("תוכל לעזור לי עם זה?", &s, &ss));
        assert_eq!(i.pragmatic, PragmaticIntent::ImplicitHelp);
    }

    #[test]
    fn test_direct_request() {
        let (s, ss) = ctx();
        let i = IntentDetector::detect(&mk("show me the latest commits", &s, &ss));
        assert_eq!(i.pragmatic, PragmaticIntent::ImplicitHelp);
    }

    #[test]
    fn test_question() {
        let (s, ss) = ctx();
        let i = IntentDetector::detect(&mk("What is the capital of France?", &s, &ss));
        assert_eq!(i.pragmatic, PragmaticIntent::Literal);
    }

    #[test]
    fn test_hebrew_question() {
        let (s, ss) = ctx();
        let i = IntentDetector::detect(&mk("מה השעה?", &s, &ss));
        assert_eq!(i.pragmatic, PragmaticIntent::Literal);
    }

    #[test]
    fn test_greeting() {
        let (s, ss) = ctx();
        let i = IntentDetector::detect(&mk("Hello there", &s, &ss));
        assert_eq!(i.pragmatic, PragmaticIntent::Literal);
    }

    #[test]
    fn test_complaint() {
        let (s, ss) = ctx();
        let i = IntentDetector::detect(&mk("This doesn't work at all", &s, &ss));
        assert_eq!(i.pragmatic, PragmaticIntent::ImplicitHelp);
    }

    #[test]
    fn test_complaint_hebrew() {
        let (s, ss) = ctx();
        let i = IntentDetector::detect(&mk("זה לא עובד שוב", &s, &ss));
        assert_eq!(i.pragmatic, PragmaticIntent::ImplicitHelp);
    }

    #[test]
    fn test_expressive() {
        let (s, ss) = ctx();
        let i = IntentDetector::detect(&mk("Wow, amazing!", &s, &ss));
        assert_eq!(i.pragmatic, PragmaticIntent::Literal);
    }

    #[test]
    fn test_literal_statement() {
        let (s, ss) = ctx();
        let i = IntentDetector::detect(&mk("I just deployed the service.", &s, &ss));
        assert_eq!(i.pragmatic, PragmaticIntent::Literal);
    }

    #[test]
    fn test_ambiguity_short_message() {
        let (s, ss) = ctx();
        let i = IntentDetector::detect(&mk("ok", &s, &ss));
        assert!(i.ambiguity >= 0.4);
    }

    #[test]
    fn test_low_ambiguity_clear() {
        let (s, ss) = ctx();
        let i = IntentDetector::detect(&mk("What is the current Fed funds rate?", &s, &ss));
        assert!(i.ambiguity < 0.3);
    }

    #[test]
    fn test_topic_finance() {
        let (s, ss) = ctx();
        let i = IntentDetector::detect(&mk("Check my budget", &s, &ss));
        assert_eq!(i.topic.as_deref(), Some("domain.finance"));
    }

    #[test]
    fn test_topic_tech() {
        let (s, ss) = ctx();
        let i = IntentDetector::detect(&mk("There's a bug in the code", &s, &ss));
        assert_eq!(i.topic.as_deref(), Some("domain.tech"));
    }

    #[test]
    fn test_hint_leaving() {
        let (s, ss) = ctx();
        let i = IntentDetector::detect(&mk("gotta go, talk later", &s, &ss));
        assert!(i.pragmatic == PragmaticIntent::Hint(Hint::Leaving));
    }

    #[test]
    fn test_hint_boredom_single_ok() {
        let (s, ss) = ctx();
        let i = IntentDetector::detect(&mk("ok", &s, &ss));
        assert!(i.pragmatic == PragmaticIntent::Hint(Hint::Boredom));
    }

    #[test]
    fn test_hint_urgent() {
        let (s, ss) = ctx();
        let i = IntentDetector::detect(&mk("I need this ASAP", &s, &ss));
        assert!(i.pragmatic == PragmaticIntent::Hint(Hint::Urgent));
    }

    #[test]
    fn test_confidence_scales_with_clarity() {
        let (s, ss) = ctx();
        let ambiguous = IntentDetector::detect(&mk("it", &s, &ss));
        let clear = IntentDetector::detect(&mk("Please calculate the sum of 5 and 7.", &s, &ss));
        assert!(clear.ambiguity < ambiguous.ambiguity);
    }
}
