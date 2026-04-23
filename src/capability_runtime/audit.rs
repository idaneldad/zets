//! # Audit log
//!
//! Append-only in-memory log of capability invocations. Each entry records
//! WHO called WHAT, WHEN, HOW LONG it took, and the OUTCOME — but never
//! the actual content (PII-free by design).

use std::time::SystemTime;

use crate::personal_graph::IdentityId;

/// A single audit log entry — records one capability invocation.
///
/// Contains no message content, arguments, or output — only metadata.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// When the invocation happened (milliseconds since UNIX epoch).
    pub timestamp_ms: u64,
    /// Who invoked the capability.
    pub caller: IdentityId,
    /// Which capability was invoked.
    pub capability_id: String,
    /// How long the invocation took (milliseconds).
    pub duration_ms: u64,
    /// Cost charged for this invocation (cents).
    pub cost_cents: u32,
    /// Outcome label (e.g. "success", "timeout", "rate_limited").
    pub result_kind: String,
}

/// Append-only in-memory audit log.
///
/// Query by caller, capability, or time range. Never stores PII.
#[derive(Debug, Default)]
pub struct AuditLog {
    entries: Vec<AuditEntry>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an invocation.
    pub fn record(
        &mut self,
        caller: IdentityId,
        capability_id: String,
        duration_ms: u64,
        cost_cents: u32,
        result_kind: &str,
    ) {
        let timestamp_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        self.entries.push(AuditEntry {
            timestamp_ms,
            caller,
            capability_id,
            duration_ms,
            cost_cents,
            result_kind: result_kind.to_string(),
        });
    }

    /// Record with an explicit timestamp (for testing).
    pub fn record_at(
        &mut self,
        timestamp_ms: u64,
        caller: IdentityId,
        capability_id: String,
        duration_ms: u64,
        cost_cents: u32,
        result_kind: &str,
    ) {
        self.entries.push(AuditEntry {
            timestamp_ms,
            caller,
            capability_id,
            duration_ms,
            cost_cents,
            result_kind: result_kind.to_string(),
        });
    }

    /// All entries (newest last).
    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// Total number of recorded invocations.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Filter entries by caller.
    pub fn by_caller(&self, caller: &IdentityId) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| &e.caller == caller).collect()
    }

    /// Filter entries by capability ID.
    pub fn by_capability(&self, capability_id: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.capability_id == capability_id)
            .collect()
    }

    /// Filter entries within a time range (inclusive).
    pub fn by_time_range(&self, from_ms: u64, to_ms: u64) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp_ms >= from_ms && e.timestamp_ms <= to_ms)
            .collect()
    }

    /// Total cost across all entries.
    pub fn total_cost_cents(&self) -> u64 {
        self.entries.iter().map(|e| e.cost_cents as u64).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::personal_graph::IdentityKind;

    fn idan() -> IdentityId {
        IdentityId::new(IdentityKind::Person, "idan")
    }

    fn roni() -> IdentityId {
        IdentityId::new(IdentityKind::Person, "roni")
    }

    #[test]
    fn test_audit_record_and_query() {
        let mut log = AuditLog::new();
        log.record_at(1000, idan(), "whisper.transcribe".into(), 500, 3, "success");
        log.record_at(2000, roni(), "gemini.vision".into(), 200, 5, "success");
        log.record_at(3000, idan(), "gemini.vision".into(), 300, 5, "timeout");

        assert_eq!(log.len(), 3);

        // By caller
        assert_eq!(log.by_caller(&idan()).len(), 2);
        assert_eq!(log.by_caller(&roni()).len(), 1);

        // By capability
        assert_eq!(log.by_capability("whisper.transcribe").len(), 1);
        assert_eq!(log.by_capability("gemini.vision").len(), 2);

        // By time range
        assert_eq!(log.by_time_range(1500, 2500).len(), 1);
        assert_eq!(log.by_time_range(0, 5000).len(), 3);
    }

    #[test]
    fn test_audit_total_cost() {
        let mut log = AuditLog::new();
        log.record_at(1000, idan(), "cap1".into(), 100, 3, "success");
        log.record_at(2000, idan(), "cap2".into(), 200, 5, "success");
        assert_eq!(log.total_cost_cents(), 8);
    }

    #[test]
    fn test_audit_empty() {
        let log = AuditLog::new();
        assert!(log.is_empty());
        assert_eq!(log.total_cost_cents(), 0);
    }
}
