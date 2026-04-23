//! # Audit — log every guard decision
//!
//! Every block (and optionally every allow) is recorded. The audit log
//! is append-only and kept separate from conversation storage so that
//! an attacker who manages to clear their conversation can't also erase
//! their attempts.
//!
//! In-memory for now. Future: persistent append-only file per day.

use std::collections::VecDeque;

use super::violation::{GuardAction, GuardDecision, ViolationKind};

/// One entry in the audit log.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub ts_ms: i64,
    /// Which source triggered the check (identifier string).
    pub source_id: String,
    /// Which surface — "input" or "output".
    pub surface: Surface,
    pub action: GuardAction,
    pub violation_kinds: Vec<ViolationKind>,
    pub rule_ids: Vec<String>,
    pub max_severity: u8,
    pub internal_summary: String,
    /// Hash of the message being checked (not the message itself — no PII).
    pub message_hash: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    Input,
    Output,
}

impl Surface {
    pub fn as_str(&self) -> &'static str {
        match self {
            Surface::Input => "input",
            Surface::Output => "output",
        }
    }
}

/// The audit log. Bounded queue for memory safety.
#[derive(Debug)]
pub struct AuditLog {
    entries: VecDeque<AuditEntry>,
    max_entries: usize,
    /// Total events ever recorded (monotonic, doesn't reset when entries evicted).
    total_events: u64,
    /// Counts per violation kind, total.
    kind_counts: std::collections::HashMap<ViolationKind, u64>,
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl AuditLog {
    pub fn new(max_entries: usize) -> Self {
        AuditLog {
            entries: VecDeque::with_capacity(max_entries.min(1000)),
            max_entries,
            total_events: 0,
            kind_counts: std::collections::HashMap::new(),
        }
    }

    /// Record a guard decision.
    pub fn record(
        &mut self,
        source_id: impl Into<String>,
        surface: Surface,
        message: &str,
        decision: &GuardDecision,
        now_ms: i64,
    ) {
        // Only record non-trivial decisions (Allow with no violations is noise).
        if decision.action == GuardAction::Allow && decision.violations.is_empty() {
            return;
        }

        for v in &decision.violations {
            *self.kind_counts.entry(v.kind).or_insert(0) += 1;
        }

        let entry = AuditEntry {
            ts_ms: now_ms,
            source_id: source_id.into(),
            surface,
            action: decision.action,
            violation_kinds: decision.violations.iter().map(|v| v.kind).collect(),
            rule_ids: decision.violations.iter().map(|v| v.rule_id.clone()).collect(),
            max_severity: decision.max_severity(),
            internal_summary: decision.internal_summary.clone(),
            message_hash: hash_message(message),
        };

        self.total_events += 1;

        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Number of entries currently retained.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Total events across all time (not just retained).
    pub fn total(&self) -> u64 {
        self.total_events
    }

    /// Count of a specific violation kind.
    pub fn count(&self, kind: ViolationKind) -> u64 {
        self.kind_counts.get(&kind).copied().unwrap_or(0)
    }

    /// Recent entries (most recent last).
    pub fn recent(&self, n: usize) -> Vec<&AuditEntry> {
        self.entries.iter().rev().take(n).collect()
    }

    /// Entries for a specific source.
    pub fn for_source(&self, source_id: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.source_id == source_id)
            .collect()
    }

    /// Count of blocked events for a source — used to detect repeat attackers.
    pub fn block_count_for(&self, source_id: &str) -> usize {
        self.entries
            .iter()
            .filter(|e| e.source_id == source_id)
            .filter(|e| !e.action.is_permitted())
            .count()
    }
}

/// FNV-1a hash of a message — stable, fast, non-cryptographic.
fn hash_message(msg: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    for b in msg.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guard::violation::Violation;

    fn mk_decision(kind: ViolationKind) -> GuardDecision {
        GuardDecision::from_violations(vec![Violation::new(kind, "r1", "test", 0.9)])
    }

    #[test]
    fn test_record_basic() {
        let mut log = AuditLog::new(10);
        let d = mk_decision(ViolationKind::PromptInjection);
        log.record("user:a", Surface::Input, "ignore previous", &d, 1000);
        assert_eq!(log.len(), 1);
        assert_eq!(log.total(), 1);
    }

    #[test]
    fn test_allow_not_recorded() {
        let mut log = AuditLog::new(10);
        let d = GuardDecision::allow();
        log.record("user:a", Surface::Input, "hello", &d, 1000);
        assert_eq!(log.len(), 0);
        assert_eq!(log.total(), 0);
    }

    #[test]
    fn test_count_by_kind() {
        let mut log = AuditLog::new(100);
        log.record(
            "u:1",
            Surface::Input,
            "m",
            &mk_decision(ViolationKind::PromptInjection),
            1000,
        );
        log.record(
            "u:2",
            Surface::Input,
            "m",
            &mk_decision(ViolationKind::PromptInjection),
            2000,
        );
        log.record(
            "u:1",
            Surface::Input,
            "m",
            &mk_decision(ViolationKind::SecretLeakage),
            3000,
        );

        assert_eq!(log.count(ViolationKind::PromptInjection), 2);
        assert_eq!(log.count(ViolationKind::SecretLeakage), 1);
        assert_eq!(log.count(ViolationKind::RateLimit), 0);
    }

    #[test]
    fn test_bounded_queue_evicts_old() {
        let mut log = AuditLog::new(3);
        for i in 0..5 {
            log.record(
                "u",
                Surface::Input,
                "m",
                &mk_decision(ViolationKind::PromptInjection),
                i,
            );
        }
        assert_eq!(log.len(), 3);
        assert_eq!(log.total(), 5); // total isn't capped
    }

    #[test]
    fn test_block_count_for_repeat_source() {
        let mut log = AuditLog::new(100);
        for i in 0..4 {
            log.record(
                "bad_user",
                Surface::Input,
                "m",
                &mk_decision(ViolationKind::PromptInjection),
                i,
            );
        }
        log.record(
            "good_user",
            Surface::Input,
            "m",
            &mk_decision(ViolationKind::PromptInjection),
            10,
        );

        assert_eq!(log.block_count_for("bad_user"), 4);
        assert_eq!(log.block_count_for("good_user"), 1);
        assert_eq!(log.block_count_for("unknown"), 0);
    }

    #[test]
    fn test_recent_order() {
        let mut log = AuditLog::new(100);
        for i in 0..5 {
            log.record(
                &format!("u{}", i),
                Surface::Input,
                "m",
                &mk_decision(ViolationKind::PromptInjection),
                i,
            );
        }
        let recent = log.recent(3);
        assert_eq!(recent.len(), 3);
        // Most recent first
        assert_eq!(recent[0].source_id, "u4");
        assert_eq!(recent[2].source_id, "u2");
    }

    #[test]
    fn test_for_source_filter() {
        let mut log = AuditLog::new(100);
        log.record(
            "alice",
            Surface::Input,
            "m",
            &mk_decision(ViolationKind::PromptInjection),
            1,
        );
        log.record(
            "bob",
            Surface::Output,
            "m",
            &mk_decision(ViolationKind::SecretLeakage),
            2,
        );
        log.record(
            "alice",
            Surface::Output,
            "m",
            &mk_decision(ViolationKind::SecretLeakage),
            3,
        );

        assert_eq!(log.for_source("alice").len(), 2);
        assert_eq!(log.for_source("bob").len(), 1);
    }

    #[test]
    fn test_message_not_stored_only_hash() {
        let mut log = AuditLog::new(100);
        let secret_message = "my secret API key is sk-ant-xyz";
        log.record(
            "u",
            Surface::Input,
            secret_message,
            &mk_decision(ViolationKind::SecretLeakage),
            1,
        );

        // The message hash is stored, not the message
        let entry = &log.entries[0];
        assert_eq!(entry.message_hash, hash_message(secret_message));
        // No field contains the actual message text
        assert!(!format!("{:?}", entry).contains("sk-ant-xyz"));
    }
}
