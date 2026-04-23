//! # Budget tracker
//!
//! Per-caller (IdentityId) monotonic cost tracking. Rejects invocations
//! that would exceed either the per-call ceiling or the caller's global budget.

use std::collections::HashMap;

use crate::personal_graph::IdentityId;

/// Tracks cumulative cost (in cents) per caller identity.
///
/// Budget limits are enforced at two levels:
/// 1. **Per-call**: `CapabilityInvocation.max_budget_cents` — the caller's
///    ceiling for a single invocation.
/// 2. **Global per-caller**: an optional total-spend cap set via
///    `set_caller_limit`.
#[derive(Debug)]
pub struct BudgetTracker {
    /// Cumulative cents spent per caller.
    spent: HashMap<IdentityId, u32>,
    /// Optional per-caller spending limit (cents). If not set, no global limit.
    limits: HashMap<IdentityId, u32>,
    /// Default global limit for callers without a specific limit (0 = unlimited).
    default_limit_cents: u32,
}

impl BudgetTracker {
    /// Create a new tracker with no default limit (unlimited).
    pub fn new() -> Self {
        BudgetTracker {
            spent: HashMap::new(),
            limits: HashMap::new(),
            default_limit_cents: 0,
        }
    }

    /// Create a tracker with a default global limit for all callers.
    pub fn with_default_limit(limit_cents: u32) -> Self {
        BudgetTracker {
            spent: HashMap::new(),
            limits: HashMap::new(),
            default_limit_cents: limit_cents,
        }
    }

    /// Set a spending limit for a specific caller.
    pub fn set_caller_limit(&mut self, caller: &IdentityId, limit_cents: u32) {
        self.limits.insert(caller.clone(), limit_cents);
    }

    /// Check if the caller can afford `cost_cents` without exceeding
    /// either the per-call budget or their global limit.
    ///
    /// - `per_call_budget`: the caller's max_budget_cents for this invocation.
    /// - `cost_cents`: the cost of the capability.
    pub fn can_afford(
        &self,
        caller: &IdentityId,
        cost_cents: u32,
        per_call_budget: u32,
    ) -> bool {
        // Per-call budget check
        if cost_cents > per_call_budget {
            return false;
        }
        // Global limit check
        let limit = self.effective_limit(caller);
        if limit == 0 {
            return true; // unlimited
        }
        let current = self.spent.get(caller).copied().unwrap_or(0);
        current.saturating_add(cost_cents) <= limit
    }

    /// Record that `cost_cents` was spent by `caller`. Monotonic — never decreases.
    pub fn record_spend(&mut self, caller: &IdentityId, cost_cents: u32) {
        let entry = self.spent.entry(caller.clone()).or_insert(0);
        *entry = entry.saturating_add(cost_cents);
    }

    /// Total cents spent by a caller.
    pub fn total_spent(&self, caller: &IdentityId) -> u32 {
        self.spent.get(caller).copied().unwrap_or(0)
    }

    /// Remaining budget for a caller. Returns `None` if unlimited.
    pub fn remaining(&self, caller: &IdentityId) -> Option<u32> {
        let limit = self.effective_limit(caller);
        if limit == 0 {
            return None; // unlimited
        }
        let spent = self.total_spent(caller);
        Some(limit.saturating_sub(spent))
    }

    /// Reset a caller's spend counter (for testing or admin).
    pub fn reset(&mut self, caller: &IdentityId) {
        self.spent.remove(caller);
    }

    /// Reset all spend counters.
    pub fn reset_all(&mut self) {
        self.spent.clear();
    }

    fn effective_limit(&self, caller: &IdentityId) -> u32 {
        self.limits
            .get(caller)
            .copied()
            .unwrap_or(self.default_limit_cents)
    }
}

impl Default for BudgetTracker {
    fn default() -> Self {
        Self::new()
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
    fn test_unlimited_budget() {
        let tracker = BudgetTracker::new();
        assert!(tracker.can_afford(&idan(), 1000, 1000));
        assert!(tracker.remaining(&idan()).is_none());
    }

    #[test]
    fn test_per_call_budget_exceeded() {
        let tracker = BudgetTracker::new();
        // cost (10) > per_call_budget (5)
        assert!(!tracker.can_afford(&idan(), 10, 5));
    }

    #[test]
    fn test_global_limit_enforcement() {
        let mut tracker = BudgetTracker::with_default_limit(100);
        tracker.record_spend(&idan(), 80);
        assert!(tracker.can_afford(&idan(), 20, 100)); // 80 + 20 = 100, OK
        assert!(!tracker.can_afford(&idan(), 21, 100)); // 80 + 21 = 101, exceeded
        assert_eq!(tracker.remaining(&idan()), Some(20));
    }

    #[test]
    fn test_per_caller_budget_isolated() {
        let mut tracker = BudgetTracker::with_default_limit(100);
        tracker.record_spend(&idan(), 90);
        // Idan is almost out, but Roni hasn't spent anything
        assert!(!tracker.can_afford(&idan(), 20, 100));
        assert!(tracker.can_afford(&roni(), 20, 100));
        assert_eq!(tracker.total_spent(&idan()), 90);
        assert_eq!(tracker.total_spent(&roni()), 0);
    }

    #[test]
    fn test_caller_specific_limit() {
        let mut tracker = BudgetTracker::with_default_limit(100);
        tracker.set_caller_limit(&idan(), 500);
        tracker.record_spend(&idan(), 200);
        assert!(tracker.can_afford(&idan(), 200, 500)); // 200 + 200 = 400 ≤ 500
        assert_eq!(tracker.remaining(&idan()), Some(300));
    }

    #[test]
    fn test_reset() {
        let mut tracker = BudgetTracker::with_default_limit(100);
        tracker.record_spend(&idan(), 90);
        assert_eq!(tracker.total_spent(&idan()), 90);
        tracker.reset(&idan());
        assert_eq!(tracker.total_spent(&idan()), 0);
        assert!(tracker.can_afford(&idan(), 100, 100));
    }

    #[test]
    fn test_monotonic_spending() {
        let mut tracker = BudgetTracker::new();
        tracker.record_spend(&idan(), 10);
        tracker.record_spend(&idan(), 20);
        tracker.record_spend(&idan(), 5);
        assert_eq!(tracker.total_spent(&idan()), 35);
    }
}
