//! # ConversationStore — in-memory + file-backed persistent store
//!
//! Holds all sessions and messages, per source.

use std::collections::HashMap;
use std::io;

use super::session::{Session, SessionBoundary};
use crate::reader::input::HistoryEntry;

#[derive(Debug)]
pub enum StoreError {
    Io(io::Error),
    SessionNotFound,
    SessionAlreadyEnded,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "I/O: {}", e),
            StoreError::SessionNotFound => write!(f, "session not found"),
            StoreError::SessionAlreadyEnded => write!(f, "session already ended"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<io::Error> for StoreError {
    fn from(e: io::Error) -> Self {
        StoreError::Io(e)
    }
}

/// The main store. In-memory for now — persistence is future work.
#[derive(Debug, Default)]
pub struct ConversationStore {
    /// All sessions by session_id.
    sessions: HashMap<String, Session>,
    /// Active session id per source.
    active_by_source: HashMap<String, String>,
    /// Counter for generating session ids.
    next_session_num: u64,
    /// Idle threshold in ms. Default: 30 minutes.
    idle_threshold_ms: i64,
}

impl ConversationStore {
    pub fn new() -> Self {
        ConversationStore {
            sessions: HashMap::new(),
            active_by_source: HashMap::new(),
            next_session_num: 1,
            idle_threshold_ms: 30 * 60 * 1000,
        }
    }

    pub fn with_idle_threshold(mut self, ms: i64) -> Self {
        self.idle_threshold_ms = ms;
        self
    }

    /// Append an entry to the active session for a source, opening a
    /// new session if none is active or if the previous one went idle.
    pub fn append(&mut self, source_id: &str, entry: HistoryEntry) -> &str {
        // Check if existing session is still active
        let needs_new_session = match self.active_by_source.get(source_id) {
            None => true,
            Some(sid) => {
                let sid = sid.clone();
                if let Some(sess) = self.sessions.get(&sid) {
                    if !sess.is_active() {
                        true
                    } else if sess.is_idle(entry.ts_ms, self.idle_threshold_ms) {
                        // close old session, start new
                        if let Some(s) = self.sessions.get_mut(&sid) {
                            s.end(entry.ts_ms, SessionBoundary::IdleTimeout);
                        }
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
        };

        let session_id = if needs_new_session {
            let new_id = format!("s{:06}", self.next_session_num);
            self.next_session_num += 1;
            let new_session = Session::new(&new_id, source_id, entry.ts_ms);
            self.sessions.insert(new_id.clone(), new_session);
            self.active_by_source.insert(source_id.to_string(), new_id.clone());
            new_id
        } else {
            self.active_by_source[source_id].clone()
        };

        // Append to session
        if let Some(sess) = self.sessions.get_mut(&session_id) {
            sess.append(entry);
        }

        // Return by looking up once more
        self.active_by_source[source_id].as_str()
    }

    /// Get the full history for a source, across all sessions, in order.
    pub fn history_for(&self, source_id: &str) -> Vec<&HistoryEntry> {
        let mut all: Vec<(&i64, &HistoryEntry)> = self
            .sessions
            .values()
            .filter(|s| s.source_id == source_id)
            .flat_map(|s| s.entries.iter().map(|e| (&e.ts_ms, e)))
            .collect();
        all.sort_by_key(|&(ts, _)| ts);
        all.into_iter().map(|(_, e)| e).collect()
    }

    /// Get only the last N entries for a source.
    pub fn recent(&self, source_id: &str, n: usize) -> Vec<&HistoryEntry> {
        let full = self.history_for(source_id);
        let len = full.len();
        full.into_iter().skip(len.saturating_sub(n)).collect()
    }

    /// Get the active session for a source.
    pub fn active_session(&self, source_id: &str) -> Option<&Session> {
        self.active_by_source
            .get(source_id)
            .and_then(|sid| self.sessions.get(sid))
    }

    /// Explicitly close the active session for a source.
    pub fn close_session(
        &mut self,
        source_id: &str,
        now_ms: i64,
        reason: SessionBoundary,
    ) -> Result<(), StoreError> {
        let sid = self
            .active_by_source
            .remove(source_id)
            .ok_or(StoreError::SessionNotFound)?;
        let sess = self
            .sessions
            .get_mut(&sid)
            .ok_or(StoreError::SessionNotFound)?;
        if !sess.is_active() {
            return Err(StoreError::SessionAlreadyEnded);
        }
        sess.end(now_ms, reason);
        Ok(())
    }

    /// Total messages stored (across all sources).
    pub fn total_entries(&self) -> usize {
        self.sessions.values().map(|s| s.entries.len()).sum()
    }

    /// Number of sessions (active + ended).
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// All sessions for a source, oldest first.
    pub fn sessions_for(&self, source_id: &str) -> Vec<&Session> {
        let mut sessions: Vec<&Session> = self
            .sessions
            .values()
            .filter(|s| s.source_id == source_id)
            .collect();
        sessions.sort_by_key(|s| s.started_ms);
        sessions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::input::Author;

    fn entry(ts: i64, content: &str) -> HistoryEntry {
        HistoryEntry {
            ts_ms: ts,
            who: Author::FromSource,
            content: content.into(),
        }
    }

    #[test]
    fn test_append_creates_session() {
        let mut store = ConversationStore::new();
        store.append("person:idan", entry(1000, "hello"));

        assert_eq!(store.session_count(), 1);
        assert_eq!(store.total_entries(), 1);
        assert!(store.active_session("person:idan").is_some());
    }

    #[test]
    fn test_append_continues_session() {
        let mut store = ConversationStore::new();
        store.append("person:idan", entry(1000, "msg1"));
        store.append("person:idan", entry(2000, "msg2"));
        store.append("person:idan", entry(3000, "msg3"));

        assert_eq!(store.session_count(), 1);
        assert_eq!(store.total_entries(), 3);
    }

    #[test]
    fn test_idle_creates_new_session() {
        let mut store = ConversationStore::new().with_idle_threshold(1000); // 1sec idle
        store.append("person:idan", entry(1000, "msg1"));
        store.append("person:idan", entry(2500, "msg2")); // 1.5s gap → new session

        assert_eq!(store.session_count(), 2);
        assert_eq!(store.total_entries(), 2);
    }

    #[test]
    fn test_sessions_separate_per_source() {
        let mut store = ConversationStore::new();
        store.append("person:idan", entry(1000, "idan msg"));
        store.append("client:c1@idan", entry(1100, "client msg"));

        assert_eq!(store.session_count(), 2);
        assert_eq!(store.history_for("person:idan").len(), 1);
        assert_eq!(store.history_for("client:c1@idan").len(), 1);
    }

    #[test]
    fn test_history_ordered() {
        let mut store = ConversationStore::new().with_idle_threshold(100);
        store.append("person:a", entry(1000, "first"));
        store.append("person:a", entry(2000, "second")); // new session (idle > 100)
        store.append("person:a", entry(3000, "third"));  // new session

        let h = store.history_for("person:a");
        assert_eq!(h.len(), 3);
        assert_eq!(h[0].content, "first");
        assert_eq!(h[2].content, "third");
    }

    #[test]
    fn test_recent() {
        let mut store = ConversationStore::new();
        for i in 0..10 {
            store.append("p:a", entry(1000 + i * 10, &format!("msg{}", i)));
        }
        let recent = store.recent("p:a", 3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].content, "msg7");
        assert_eq!(recent[2].content, "msg9");
    }

    #[test]
    fn test_explicit_close() {
        let mut store = ConversationStore::new();
        store.append("person:idan", entry(1000, "hi"));
        store
            .close_session("person:idan", 2000, SessionBoundary::ExplicitClose)
            .unwrap();

        assert!(store.active_session("person:idan").is_none());
        // But history still there
        assert_eq!(store.history_for("person:idan").len(), 1);

        // New message creates new session
        store.append("person:idan", entry(3000, "back again"));
        assert_eq!(store.session_count(), 2);
    }

    #[test]
    fn test_close_without_session() {
        let mut store = ConversationStore::new();
        let result =
            store.close_session("person:nobody", 1000, SessionBoundary::ExplicitClose);
        assert!(result.is_err());
    }
}
