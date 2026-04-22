//! Testing sandbox — staged learning + simulation.
//!
//! Per Idan's request: when ZETS wants to learn something new (a rule, a route,
//! a fact), it goes to the TESTING scope first. There it can:
//!   - Simulate the impact (run existing queries, see if they still work)
//!   - Generate synthetic test data
//!   - Verify that the change doesn't break anything
//!
//! Only after passing tests does it get promoted to the production scope.
//!
//! This makes ZETS "safely self-modifying" — it can experiment without risk.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// A staged change awaiting verification.
#[derive(Debug, Clone)]
pub struct StagedChange {
    pub change_id: String,
    pub kind: ChangeKind,
    pub description: String,
    pub proposed_at_ms: u64,
    pub test_results: Vec<TestResult>,
    pub status: ChangeStatus,
}

#[derive(Debug, Clone)]
pub enum ChangeKind {
    /// A new concept to add to Data scope.
    AddConcept { anchor: String, gloss: String, pos: u8 },
    /// A new rule for morphology.
    AddMorphologyRule { lang: String, rule_type: String, params: String },
    /// A new route in the system graph.
    AddSystemRoute { name: String, bytecode_len: usize },
    /// A new edge between existing concepts.
    AddEdge { source: u32, target: u32, kind: u8 },
    /// A new fact in user scope.
    AddUserFact { key: String, value: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeStatus {
    /// Just proposed, not tested yet.
    Proposed,
    /// Under test.
    Testing,
    /// Passed all tests, safe to promote.
    Verified,
    /// Failed — hold in testing scope, flag for review.
    Failed,
    /// Promoted to production.
    Promoted,
    /// Rolled back.
    RolledBack,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub detail: String,
}

/// A sandbox for staging changes.
pub struct TestingSandbox {
    staged: HashMap<String, StagedChange>,
    next_id: u64,
}

impl TestingSandbox {
    pub fn new() -> Self {
        Self {
            staged: HashMap::new(),
            next_id: 1,
        }
    }

    /// Stage a new change. Returns its id.
    pub fn stage(&mut self, kind: ChangeKind, description: impl Into<String>) -> String {
        let id = format!("stage-{}", self.next_id);
        self.next_id += 1;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let change = StagedChange {
            change_id: id.clone(),
            kind,
            description: description.into(),
            proposed_at_ms: timestamp,
            test_results: Vec::new(),
            status: ChangeStatus::Proposed,
        };
        self.staged.insert(id.clone(), change);
        id
    }

    /// Record a test result for a staged change.
    pub fn record_test(&mut self, change_id: &str, test: TestResult) {
        if let Some(change) = self.staged.get_mut(change_id) {
            change.status = ChangeStatus::Testing;
            change.test_results.push(test);
        }
    }

    /// Finalize a staged change — decide verified/failed based on results.
    pub fn finalize(&mut self, change_id: &str) {
        if let Some(change) = self.staged.get_mut(change_id) {
            let all_passed = change.test_results.iter().all(|t| t.passed)
                && !change.test_results.is_empty();
            change.status = if all_passed {
                ChangeStatus::Verified
            } else {
                ChangeStatus::Failed
            };
        }
    }

    pub fn mark_promoted(&mut self, change_id: &str) {
        if let Some(change) = self.staged.get_mut(change_id) {
            change.status = ChangeStatus::Promoted;
        }
    }

    pub fn mark_rolled_back(&mut self, change_id: &str) {
        if let Some(change) = self.staged.get_mut(change_id) {
            change.status = ChangeStatus::RolledBack;
        }
    }

    pub fn get(&self, change_id: &str) -> Option<&StagedChange> {
        self.staged.get(change_id)
    }

    pub fn pending_changes(&self) -> Vec<&StagedChange> {
        self.staged
            .values()
            .filter(|c| matches!(c.status, ChangeStatus::Proposed | ChangeStatus::Testing))
            .collect()
    }

    pub fn verified_ready_to_promote(&self) -> Vec<&StagedChange> {
        self.staged
            .values()
            .filter(|c| c.status == ChangeStatus::Verified)
            .collect()
    }

    pub fn failed_changes(&self) -> Vec<&StagedChange> {
        self.staged
            .values()
            .filter(|c| c.status == ChangeStatus::Failed)
            .collect()
    }

    /// Wipe the sandbox (as Idan requested: testing is ephemeral).
    pub fn clear(&mut self) {
        self.staged.clear();
        self.next_id = 1;
    }

    pub fn stats(&self) -> SandboxStats {
        let mut stats = SandboxStats::default();
        for change in self.staged.values() {
            stats.total += 1;
            match change.status {
                ChangeStatus::Proposed => stats.proposed += 1,
                ChangeStatus::Testing => stats.testing += 1,
                ChangeStatus::Verified => stats.verified += 1,
                ChangeStatus::Failed => stats.failed += 1,
                ChangeStatus::Promoted => stats.promoted += 1,
                ChangeStatus::RolledBack => stats.rolled_back += 1,
            }
        }
        stats
    }
}

impl Default for TestingSandbox {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Clone)]
pub struct SandboxStats {
    pub total: usize,
    pub proposed: usize,
    pub testing: usize,
    pub verified: usize,
    pub failed: usize,
    pub promoted: usize,
    pub rolled_back: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_and_test() {
        let mut sb = TestingSandbox::new();
        let id = sb.stage(
            ChangeKind::AddConcept {
                anchor: "DNA".into(),
                gloss: "genetic molecule".into(),
                pos: 1,
            },
            "new concept from article",
        );
        assert!(sb.get(&id).is_some());
        assert_eq!(sb.get(&id).unwrap().status, ChangeStatus::Proposed);

        sb.record_test(&id, TestResult {
            test_name: "conflict_check".into(),
            passed: true,
            detail: "no conflict".into(),
        });
        sb.record_test(&id, TestResult {
            test_name: "query_regression".into(),
            passed: true,
            detail: "all queries still pass".into(),
        });
        sb.finalize(&id);
        assert_eq!(sb.get(&id).unwrap().status, ChangeStatus::Verified);
    }

    #[test]
    fn failed_test_blocks_promotion() {
        let mut sb = TestingSandbox::new();
        let id = sb.stage(
            ChangeKind::AddEdge { source: 1, target: 2, kind: 3 },
            "speculative edge",
        );
        sb.record_test(&id, TestResult {
            test_name: "cycle_check".into(),
            passed: false,
            detail: "would create cycle".into(),
        });
        sb.finalize(&id);
        assert_eq!(sb.get(&id).unwrap().status, ChangeStatus::Failed);
        assert_eq!(sb.failed_changes().len(), 1);
        assert_eq!(sb.verified_ready_to_promote().len(), 0);
    }

    #[test]
    fn promote_workflow() {
        let mut sb = TestingSandbox::new();
        let id = sb.stage(
            ChangeKind::AddConcept { anchor: "test".into(), gloss: "".into(), pos: 1 },
            "test",
        );
        sb.record_test(&id, TestResult {
            test_name: "t".into(), passed: true, detail: "".into(),
        });
        sb.finalize(&id);
        sb.mark_promoted(&id);
        assert_eq!(sb.get(&id).unwrap().status, ChangeStatus::Promoted);
    }

    #[test]
    fn clear_wipes_sandbox() {
        let mut sb = TestingSandbox::new();
        sb.stage(ChangeKind::AddConcept {
            anchor: "x".into(), gloss: "".into(), pos: 1
        }, "test");
        assert_eq!(sb.stats().total, 1);
        sb.clear();
        assert_eq!(sb.stats().total, 0);
    }

    #[test]
    fn stats_track_correctly() {
        let mut sb = TestingSandbox::new();
        let id1 = sb.stage(ChangeKind::AddConcept {
            anchor: "a".into(), gloss: "".into(), pos: 1
        }, "t1");
        let id2 = sb.stage(ChangeKind::AddConcept {
            anchor: "b".into(), gloss: "".into(), pos: 1
        }, "t2");

        sb.record_test(&id1, TestResult {
            test_name: "t".into(), passed: true, detail: "".into(),
        });
        sb.finalize(&id1);

        sb.record_test(&id2, TestResult {
            test_name: "t".into(), passed: false, detail: "".into(),
        });
        sb.finalize(&id2);

        let s = sb.stats();
        assert_eq!(s.total, 2);
        assert_eq!(s.verified, 1);
        assert_eq!(s.failed, 1);
    }
}
