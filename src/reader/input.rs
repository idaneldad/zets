//! # ReadInput — everything that enters ZETS for reading
//!
//! Whether from a human, an API, a peer ZETS, or ZETS itself, every input
//! that needs to be understood flows through `ReadInput`. The `Reader`
//! consumes `ReadInput` and produces a `Reading`.
//!
//! Design principle: thin struct, heavy context pointers.
//! The payload (`message`) is owned; the context (`history`, `profile`)
//! is borrowed — the Reader doesn't own conversation state.

use super::source::Source;

/// Raw input that the Reader will analyze.
///
/// All data ZETS needs to understand "what just came in and from whom".
/// The Reader resolves this into a `Reading`.
pub struct ReadInput<'a> {
    /// The textual payload. For non-textual inputs (e.g. API JSON),
    /// this is a canonical string representation.
    pub message: &'a str,

    /// Where this input came from.
    pub source: &'a Source,

    /// Conversation history with this source, most recent last.
    /// Empty for first interaction.
    pub history: &'a [HistoryEntry],

    /// The session in which this input arrived.
    pub session: &'a SessionContext,

    /// Optional: extra structured metadata the source provided.
    /// E.g. for ExternalApi, this might be the raw JSON payload.
    pub metadata: Option<&'a str>,
}

/// One prior exchange in a conversation.
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// Unix timestamp in milliseconds.
    pub ts_ms: i64,
    /// Who sent it — either the current source, or ZETS itself.
    pub who: Author,
    /// What was said.
    pub content: String,
}

/// Author of a history entry — either a Source, or ZETS replying.
#[derive(Debug, Clone, PartialEq)]
pub enum Author {
    /// Came from the Source we are reading.
    FromSource,
    /// A reply ZETS produced.
    FromZets,
    /// A system event (e.g. session started, profile updated).
    System(String),
}

/// Context of the current session.
///
/// A session is a bounded conversation — usually capped at idle time or
/// explicit close. Two messages from the same `Source` at different times
/// may belong to different sessions.
#[derive(Debug, Clone)]
pub struct SessionContext {
    /// Unique session identifier.
    pub session_id: String,
    /// When the session started.
    pub started_ms: i64,
    /// How many exchanges have occurred in this session.
    pub turn_count: u32,
    /// Aggregated session signals, updated each turn.
    pub signals: SessionSignals,
}

/// Running aggregate of session-level signals.
///
/// These are cheap rolling counts the Reader updates each turn —
/// used for detecting drift, engagement decline, topic shift, etc.
#[derive(Debug, Clone, Default)]
pub struct SessionSignals {
    /// Average message length over the session.
    pub avg_msg_len: f32,
    /// Ratio of short messages (< 10 words). High = disengagement signal.
    pub short_msg_ratio: f32,
    /// Ratio of question marks. High = exploration or confusion.
    pub question_ratio: f32,
    /// Topic-shift count (approximate).
    pub topic_shifts: u32,
    /// Whether the source asked a clarifying question back to ZETS.
    pub pushed_back: bool,
}

impl<'a> ReadInput<'a> {
    /// Build a new ReadInput. All fields required except metadata.
    pub fn new(
        message: &'a str,
        source: &'a Source,
        history: &'a [HistoryEntry],
        session: &'a SessionContext,
    ) -> Self {
        ReadInput {
            message,
            source,
            history,
            session,
            metadata: None,
        }
    }

    /// Add metadata to the input (for API sources, structured payloads).
    pub fn with_metadata(mut self, metadata: &'a str) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Is this the first interaction from this source?
    pub fn is_first_contact(&self) -> bool {
        self.history.is_empty()
    }

    /// How many prior exchanges in this session.
    pub fn turn_number(&self) -> u32 {
        self.session.turn_count
    }

    /// Message length in graphemes (not bytes — Hebrew-safe).
    pub fn message_length(&self) -> usize {
        self.message.chars().count()
    }

    /// Quick word count.
    pub fn word_count(&self) -> usize {
        self.message.split_whitespace().count()
    }
}

impl SessionContext {
    /// Build a fresh session (turn 0).
    pub fn new(session_id: impl Into<String>, started_ms: i64) -> Self {
        SessionContext {
            session_id: session_id.into(),
            started_ms,
            turn_count: 0,
            signals: SessionSignals::default(),
        }
    }

    /// Advance the session by one turn.
    pub fn advance(&mut self) {
        self.turn_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::source::{Source, UserRole};

    fn mk_source() -> Source {
        Source::User {
            id: "idan".into(),
            role: UserRole::Owner,
        }
    }

    fn mk_session() -> SessionContext {
        SessionContext::new("s1", 1745400000000)
    }

    #[test]
    fn test_first_contact() {
        let src = mk_source();
        let sess = mk_session();
        let input = ReadInput::new("hello", &src, &[], &sess);
        assert!(input.is_first_contact());
        assert_eq!(input.turn_number(), 0);
    }

    #[test]
    fn test_not_first_contact() {
        let src = mk_source();
        let sess = mk_session();
        let history = vec![HistoryEntry {
            ts_ms: 1745399999000,
            who: Author::FromSource,
            content: "previous".into(),
        }];
        let input = ReadInput::new("follow up", &src, &history, &sess);
        assert!(!input.is_first_contact());
    }

    #[test]
    fn test_message_length_hebrew() {
        let src = mk_source();
        let sess = mk_session();
        let input = ReadInput::new("שלום", &src, &[], &sess);
        // 4 Hebrew chars, not 8 bytes
        assert_eq!(input.message_length(), 4);
    }

    #[test]
    fn test_word_count() {
        let src = mk_source();
        let sess = mk_session();
        let input = ReadInput::new("hello world today", &src, &[], &sess);
        assert_eq!(input.word_count(), 3);
    }

    #[test]
    fn test_session_advance() {
        let mut sess = mk_session();
        assert_eq!(sess.turn_count, 0);
        sess.advance();
        sess.advance();
        assert_eq!(sess.turn_count, 2);
    }

    #[test]
    fn test_with_metadata() {
        let src = mk_source();
        let sess = mk_session();
        let meta = r#"{"webhook":"zapier"}"#;
        let input = ReadInput::new("event", &src, &[], &sess).with_metadata(meta);
        assert_eq!(input.metadata, Some(meta));
    }
}
