//! # Session — a bounded conversation
//!
//! A Session groups consecutive messages from the same source into a
//! coherent "conversation". Sessions end when:
//!   - Idle too long (default: 30 minutes)
//!   - Explicit close (user/system signal)
//!   - Topic shift so large that continuity broke

use crate::reader::input::HistoryEntry;

/// A Session — a bounded, named conversation between ZETS and a Source.
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub source_id: String,
    pub started_ms: i64,
    pub last_activity_ms: i64,
    pub ended_ms: Option<i64>,
    pub end_reason: Option<SessionBoundary>,
    pub entries: Vec<HistoryEntry>,
}

/// Why a session ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionBoundary {
    /// No activity for the idle threshold.
    IdleTimeout,
    /// Caller explicitly closed.
    ExplicitClose,
    /// System detected a major topic shift, started a new session.
    TopicShift,
    /// Daily rollover (end of day cuts session).
    DayBoundary,
    /// Process restart, session lost.
    Restart,
}

impl SessionBoundary {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionBoundary::IdleTimeout => "idle_timeout",
            SessionBoundary::ExplicitClose => "explicit_close",
            SessionBoundary::TopicShift => "topic_shift",
            SessionBoundary::DayBoundary => "day_boundary",
            SessionBoundary::Restart => "restart",
        }
    }
}

impl Session {
    pub fn new(id: impl Into<String>, source_id: impl Into<String>, now_ms: i64) -> Self {
        Session {
            id: id.into(),
            source_id: source_id.into(),
            started_ms: now_ms,
            last_activity_ms: now_ms,
            ended_ms: None,
            end_reason: None,
            entries: Vec::new(),
        }
    }

    pub fn append(&mut self, entry: HistoryEntry) {
        self.last_activity_ms = entry.ts_ms;
        self.entries.push(entry);
    }

    pub fn end(&mut self, now_ms: i64, reason: SessionBoundary) {
        self.ended_ms = Some(now_ms);
        self.end_reason = Some(reason);
    }

    pub fn is_active(&self) -> bool {
        self.ended_ms.is_none()
    }

    pub fn turn_count(&self) -> u32 {
        self.entries.len() as u32
    }

    /// Duration in milliseconds (live if still active).
    pub fn duration_ms(&self, now_ms: i64) -> i64 {
        let end = self.ended_ms.unwrap_or(now_ms);
        end - self.started_ms
    }

    /// Has this session been idle longer than the threshold?
    pub fn is_idle(&self, now_ms: i64, idle_threshold_ms: i64) -> bool {
        self.is_active() && (now_ms - self.last_activity_ms) > idle_threshold_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::input::Author;

    fn entry(ts: i64, content: &str, from_source: bool) -> HistoryEntry {
        HistoryEntry {
            ts_ms: ts,
            who: if from_source { Author::FromSource } else { Author::FromZets },
            content: content.into(),
        }
    }

    #[test]
    fn test_session_lifecycle() {
        let mut s = Session::new("s1", "person:idan", 1000);
        assert!(s.is_active());
        assert_eq!(s.turn_count(), 0);

        s.append(entry(1100, "hello", true));
        s.append(entry(1200, "hi back", false));
        assert_eq!(s.turn_count(), 2);
        assert_eq!(s.last_activity_ms, 1200);

        s.end(2000, SessionBoundary::ExplicitClose);
        assert!(!s.is_active());
        assert_eq!(s.end_reason, Some(SessionBoundary::ExplicitClose));
    }

    #[test]
    fn test_idle_detection() {
        let mut s = Session::new("s1", "person:a", 1000);
        s.append(entry(1100, "msg", true));

        let thirty_min_ms = 30 * 60 * 1000;
        assert!(!s.is_idle(1100 + thirty_min_ms - 1, thirty_min_ms));
        assert!(s.is_idle(1100 + thirty_min_ms + 1, thirty_min_ms));
    }

    #[test]
    fn test_ended_session_never_idle() {
        let mut s = Session::new("s1", "person:a", 1000);
        s.append(entry(1100, "msg", true));
        s.end(2000, SessionBoundary::ExplicitClose);

        // Even way later, ended session is not "idle" — it's ended
        assert!(!s.is_idle(9999999, 1000));
    }

    #[test]
    fn test_duration_calc() {
        let mut s = Session::new("s1", "person:a", 1000);
        assert_eq!(s.duration_ms(1500), 500);

        s.end(2000, SessionBoundary::IdleTimeout);
        assert_eq!(s.duration_ms(9999), 1000); // uses ended_ms, not now
    }
}
