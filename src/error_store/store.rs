//! # ErrorStore — central sink for everything that goes wrong
//!
//! Append-only. Aggregates by kind for trend detection.
//! Deduplicates recurring errors: if the same ErrorKind fires within a
//! short window from the same context, occurrence_count goes up instead
//! of a new entry.

use std::collections::HashMap;

use super::entry::{ErrorEntry, ErrorId, ErrorKind, Severity};

/// Dedup window — errors of the same kind+context within this window
/// bump an existing entry's count instead of creating a new one.
const DEDUP_WINDOW_MS: i64 = 60 * 1000; // 1 minute

/// The store.
#[derive(Debug, Default)]
pub struct ErrorStore {
    entries: HashMap<ErrorId, ErrorEntry>,
    /// ordered ids for chronological access
    order: Vec<ErrorId>,
    /// per-kind count for quick stats
    kind_counts: HashMap<String, u32>,
    /// autoincrement for generating ids
    next_num: u64,
}

impl ErrorStore {
    pub fn new() -> Self {
        ErrorStore::default()
    }

    /// Record an error. Returns the ErrorId (new or existing-bumped).
    pub fn record(&mut self, mut entry: ErrorEntry) -> ErrorId {
        // Dedup: same kind + same context + within window → bump count
        for existing_id in self.order.iter().rev().take(20) {
            if let Some(existing) = self.entries.get_mut(existing_id) {
                if existing.kind == entry.kind
                    && existing.context == entry.context
                    && existing.source == entry.source
                    && (entry.occurred_ms - existing.occurred_ms) < DEDUP_WINDOW_MS
                    && existing.is_open()
                {
                    existing.occurrence_count += 1;
                    existing.occurred_ms = entry.occurred_ms; // bump to most recent
                    return existing.id.clone();
                }
            }
        }

        // Fresh entry
        self.next_num += 1;
        let id = ErrorId(format!("e{:09}", self.next_num));
        entry.id = id.clone();

        let kind_key = entry.kind.sense_key();
        *self.kind_counts.entry(kind_key).or_insert(0) += 1;

        self.order.push(id.clone());
        self.entries.insert(id.clone(), entry);
        id
    }

    pub fn get(&self, id: &ErrorId) -> Option<&ErrorEntry> {
        self.entries.get(id)
    }

    pub fn get_mut(&mut self, id: &ErrorId) -> Option<&mut ErrorEntry> {
        self.entries.get_mut(id)
    }

    /// Open (unresolved) errors, most recent first.
    pub fn open_errors(&self) -> Vec<&ErrorEntry> {
        self.order
            .iter()
            .rev()
            .filter_map(|id| self.entries.get(id))
            .filter(|e| e.is_open())
            .collect()
    }

    /// Errors of a given severity (open or all).
    pub fn by_severity(&self, min_severity: Severity, only_open: bool) -> Vec<&ErrorEntry> {
        self.order
            .iter()
            .rev()
            .filter_map(|id| self.entries.get(id))
            .filter(|e| e.severity >= min_severity)
            .filter(|e| !only_open || e.is_open())
            .collect()
    }

    /// Errors relating to a particular source identifier.
    pub fn for_source(&self, source: &str, only_open: bool) -> Vec<&ErrorEntry> {
        self.order
            .iter()
            .rev()
            .filter_map(|id| self.entries.get(id))
            .filter(|e| e.source.as_deref() == Some(source))
            .filter(|e| !only_open || e.is_open())
            .collect()
    }

    /// Resolve an error by id.
    pub fn resolve(&mut self, id: &ErrorId, now_ms: i64, note: Option<String>) -> bool {
        if let Some(e) = self.entries.get_mut(id) {
            e.resolve(now_ms, note);
            true
        } else {
            false
        }
    }

    /// Counts by kind sense_key — for trend dashboards.
    pub fn kind_histogram(&self) -> &HashMap<String, u32> {
        &self.kind_counts
    }

    /// Top N kinds by count.
    pub fn top_kinds(&self, n: usize) -> Vec<(String, u32)> {
        let mut v: Vec<(String, u32)> =
            self.kind_counts.iter().map(|(k, v)| (k.clone(), *v)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v.into_iter().take(n).collect()
    }

    pub fn total(&self) -> usize {
        self.entries.len()
    }

    pub fn open_count(&self) -> usize {
        self.entries.values().filter(|e| e.is_open()).count()
    }

    pub fn stats(&self) -> StoreStats {
        let mut by_sev: HashMap<&'static str, u32> = HashMap::new();
        let mut open = 0u32;
        for e in self.entries.values() {
            *by_sev.entry(e.severity.as_str()).or_insert(0) += 1;
            if e.is_open() {
                open += 1;
            }
        }
        StoreStats {
            total: self.entries.len() as u32,
            open,
            info: *by_sev.get("info").unwrap_or(&0),
            warn: *by_sev.get("warn").unwrap_or(&0),
            error: *by_sev.get("error").unwrap_or(&0),
            critical: *by_sev.get("critical").unwrap_or(&0),
            security: *by_sev.get("security").unwrap_or(&0),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StoreStats {
    pub total: u32,
    pub open: u32,
    pub info: u32,
    pub warn: u32,
    pub error: u32,
    pub critical: u32,
    pub security: u32,
}

/// Quick helpers so calling code doesn't construct ErrorEntry by hand.
impl ErrorStore {
    pub fn record_gate_hold(
        &mut self,
        reason: impl Into<String>,
        source: impl Into<String>,
        context: impl Into<String>,
        now_ms: i64,
    ) -> ErrorId {
        let reason_str = reason.into();
        let entry = ErrorEntry::new(
            "",
            ErrorKind::GateHold {
                reason: reason_str.clone(),
            },
            Severity::Info,
            format!("gate hold: {}", reason_str),
            now_ms,
        )
        .with_source(source)
        .with_context(context);
        self.record(entry)
    }

    pub fn record_procedure_failure(
        &mut self,
        procedure_id: impl Into<String>,
        message: impl Into<String>,
        source: Option<String>,
        now_ms: i64,
    ) -> ErrorId {
        let mut entry = ErrorEntry::new(
            "",
            ErrorKind::ProcedureFailure {
                procedure_id: procedure_id.into(),
            },
            Severity::Error,
            message,
            now_ms,
        );
        if let Some(s) = source {
            entry = entry.with_source(s);
        }
        self.record(entry)
    }

    pub fn record_llm_failure(
        &mut self,
        provider: impl Into<String>,
        message: impl Into<String>,
        now_ms: i64,
    ) -> ErrorId {
        let entry = ErrorEntry::new(
            "",
            ErrorKind::LlmFailure {
                provider: provider.into(),
            },
            Severity::Error,
            message,
            now_ms,
        );
        self.record(entry)
    }

    pub fn record_security(
        &mut self,
        message: impl Into<String>,
        source: Option<String>,
        now_ms: i64,
    ) -> ErrorId {
        let mut entry = ErrorEntry::new(
            "",
            ErrorKind::AccessDenied {
                subject: source.clone().unwrap_or_else(|| "unknown".into()),
            },
            Severity::Security,
            message,
            now_ms,
        );
        if let Some(s) = source {
            entry = entry.with_source(s);
        }
        self.record(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_get() {
        let mut s = ErrorStore::new();
        let id = s.record_gate_hold("low_confidence", "person:idan", "ctx1", 1000);
        assert!(s.get(&id).is_some());
        assert_eq!(s.total(), 1);
    }

    #[test]
    fn test_dedup_within_window() {
        let mut s = ErrorStore::new();
        let id1 = s.record_gate_hold("reason_x", "person:idan", "ctx1", 1000);
        let id2 = s.record_gate_hold("reason_x", "person:idan", "ctx1", 1500);
        let id3 = s.record_gate_hold("reason_x", "person:idan", "ctx1", 2000);

        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
        assert_eq!(s.total(), 1);
        assert_eq!(s.get(&id1).unwrap().occurrence_count, 3);
    }

    #[test]
    fn test_dedup_window_expires() {
        let mut s = ErrorStore::new();
        let id1 = s.record_gate_hold("reason", "p", "c", 1000);
        // > 60 sec later → new entry
        let id2 = s.record_gate_hold("reason", "p", "c", 1000 + 70_000);

        assert_ne!(id1, id2);
        assert_eq!(s.total(), 2);
    }

    #[test]
    fn test_different_context_no_dedup() {
        let mut s = ErrorStore::new();
        let id1 = s.record_gate_hold("same_reason", "p", "ctxA", 1000);
        let id2 = s.record_gate_hold("same_reason", "p", "ctxB", 1001);
        assert_ne!(id1, id2);
        assert_eq!(s.total(), 2);
    }

    #[test]
    fn test_open_vs_resolved() {
        let mut s = ErrorStore::new();
        let id1 = s.record_procedure_failure("p1", "broke", None, 1000);
        let _id2 = s.record_procedure_failure(
            "p2",
            "broke too",
            Some("src".into()),
            2000,
        );

        assert_eq!(s.open_count(), 2);
        s.resolve(&id1, 3000, Some("fixed".into()));
        assert_eq!(s.open_count(), 1);
    }

    #[test]
    fn test_by_severity() {
        let mut s = ErrorStore::new();
        s.record_procedure_failure("p1", "err", None, 1000);
        s.record_llm_failure("openai", "timeout", 2000);
        s.record_security("denied", Some("stranger".into()), 3000);

        let crit = s.by_severity(Severity::Critical, false);
        assert_eq!(crit.len(), 1); // only security is ≥ Critical
        assert_eq!(crit[0].severity, Severity::Security);

        let errs = s.by_severity(Severity::Error, false);
        assert_eq!(errs.len(), 3); // all three are ≥ Error
    }

    #[test]
    fn test_for_source() {
        let mut s = ErrorStore::new();
        s.record_procedure_failure("p1", "e1", Some("person:a".into()), 1000);
        s.record_procedure_failure("p2", "e2", Some("person:b".into()), 2000);
        s.record_procedure_failure("p3", "e3", Some("person:a".into()), 3000);

        let a_errs = s.for_source("person:a", false);
        assert_eq!(a_errs.len(), 2);
        // ordered most-recent first
        assert_eq!(a_errs[0].occurred_ms, 3000);
    }

    #[test]
    fn test_kind_histogram() {
        let mut s = ErrorStore::new();
        s.record_llm_failure("openai", "e1", 1000);
        s.record_llm_failure("gemini", "e2", 1_000_000); // different kind NO — same kind, different provider
        s.record_procedure_failure("p1", "e3", None, 1_000_000);

        let hist = s.kind_histogram();
        // Note: llm_failure with different providers are DIFFERENT ErrorKind variants
        // (because provider is part of the enum payload), so dedup WILL create 2 entries
        // for the 2 llm_failures here (different occurred_ms > window).
        // But kind_histogram groups by sense_key which is just "error_kind.llm_failure"
        assert!(hist.get("error_kind.llm_failure").is_some());
        assert!(hist.get("error_kind.procedure_failure").is_some());
    }

    #[test]
    fn test_stats() {
        let mut s = ErrorStore::new();
        s.record_procedure_failure("p1", "e", None, 1000);
        s.record_security("bad", Some("attacker".into()), 2000);
        let stats = s.stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.open, 2);
        assert_eq!(stats.error, 1);
        assert_eq!(stats.security, 1);
    }

    #[test]
    fn test_top_kinds() {
        let mut s = ErrorStore::new();
        for i in 0..5 {
            // spread across time to avoid dedup
            s.record_procedure_failure(
                format!("p{}", i),
                "fail",
                None,
                1000 + (i as i64) * 100_000,
            );
        }
        for i in 0..2 {
            s.record_llm_failure("openai", "t", 1000 + (i as i64) * 100_000);
        }

        let top = s.top_kinds(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "error_kind.procedure_failure");
        assert_eq!(top[0].1, 5);
    }
}
