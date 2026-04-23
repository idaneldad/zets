//! # Capability orchestrator
//!
//! The main entry point for invoking external capabilities. Ties together
//! registry, budget, rate limiting, execution, and audit into a single
//! `invoke()` call with full pre-flight checks and retry logic.

use std::collections::HashSet;
use std::time::Instant;

use crate::personal_graph::IdentityId;

use super::audit::AuditLog;
use super::budget::BudgetTracker;
use super::definition::CapabilityDefinition;
use super::executor::{Executor, StubExecutor};
use super::invocation::CapabilityInvocation;
use super::rate_limit::RateLimiter;
use super::registry::ConnectorRegistry;
use super::result::{CapabilityError, CapabilityResult};

/// Maximum number of retries for transient errors.
const MAX_RETRIES: u32 = 3;

/// The capability orchestrator — single point of entry for invoking
/// external capabilities with budget, rate-limit, ACL, and audit.
///
/// # Usage
///
/// ```ignore
/// let mut orch = CapabilityOrchestrator::new(budget);
/// orch.register(whisper_definition);
/// orch.grant_access(&caller, "whisper.transcribe");
///
/// let result = orch.invoke(invocation)?;
/// ```
#[derive(Debug)]
pub struct CapabilityOrchestrator {
    registry: ConnectorRegistry,
    budget: BudgetTracker,
    rate_limiter: RateLimiter,
    audit: AuditLog,
    executor: Box<dyn Executor>,
    /// ACL: set of (caller, capability_id) pairs that are allowed.
    acl: HashSet<(String, String)>,
    /// Secret resolver stub: maps auth_secret_id → key string.
    /// TODO: Replace with real Vault integration when wiring is done.
    secrets: std::collections::HashMap<String, String>,
}

impl CapabilityOrchestrator {
    /// Create a new orchestrator with the given budget tracker.
    ///
    /// Uses a `StubExecutor` by default. Call `with_executor` to override.
    pub fn new(budget: BudgetTracker) -> Self {
        CapabilityOrchestrator {
            registry: ConnectorRegistry::new(),
            budget,
            rate_limiter: RateLimiter::new(),
            audit: AuditLog::new(),
            executor: Box::new(StubExecutor),
            acl: HashSet::new(),
            secrets: std::collections::HashMap::new(),
        }
    }

    /// Builder: set a custom executor.
    pub fn with_executor(mut self, executor: Box<dyn Executor>) -> Self {
        self.executor = executor;
        self
    }

    // ----- Registration ---------------------------------------------------

    /// Register a capability definition.
    pub fn register(&mut self, definition: CapabilityDefinition) {
        self.rate_limiter
            .configure(&definition.id, definition.rate_limit_per_minute);
        self.registry.register(definition);
    }

    /// Look up a registered capability.
    pub fn lookup(&self, capability_id: &str) -> Option<&CapabilityDefinition> {
        self.registry.lookup(capability_id)
    }

    // ----- ACL ------------------------------------------------------------

    /// Grant a caller permission to invoke a specific capability.
    pub fn grant_access(&mut self, caller: &IdentityId, capability_id: &str) {
        self.acl
            .insert((caller.0.clone(), capability_id.to_string()));
    }

    /// Revoke a caller's access to a capability.
    pub fn revoke_access(&mut self, caller: &IdentityId, capability_id: &str) {
        self.acl
            .remove(&(caller.0.clone(), capability_id.to_string()));
    }

    /// Check if a caller has access to a capability.
    pub fn has_access(&self, caller: &IdentityId, capability_id: &str) -> bool {
        self.acl
            .contains(&(caller.0.clone(), capability_id.to_string()))
    }

    // ----- Secrets (stub) -------------------------------------------------

    /// Register a secret for capability auth (stub — in production this
    /// would be handled by the Vault module).
    /// TODO: Wire to real Vault when integration is done.
    pub fn register_secret(&mut self, secret_id: &str, value: &str) {
        self.secrets.insert(secret_id.to_string(), value.to_string());
    }

    // ----- Invoke ---------------------------------------------------------

    /// Invoke a capability with full pre-flight checks:
    ///
    /// 1. Registry lookup (is this capability registered?)
    /// 2. ACL check (does the caller have permission?)
    /// 3. Budget check (can the caller afford the cost?)
    /// 4. Rate limit check (is the capability's rate limit exhausted?)
    /// 5. Secret resolution (does the required secret exist?)
    /// 6. Execution (call the executor)
    /// 7. Retry on transient errors (up to 3 times)
    /// 8. Audit logging
    ///
    /// Returns `Err(CapabilityError)` for pre-flight failures.
    /// Returns `Ok(CapabilityResult)` for everything else (including
    /// timeout, budget exceeded, rate limited, etc.).
    pub fn invoke(
        &mut self,
        invocation: &CapabilityInvocation,
    ) -> Result<CapabilityResult, CapabilityError> {
        let start = Instant::now();

        // 1. Registry lookup
        let definition = self
            .registry
            .lookup(&invocation.capability_id)
            .ok_or_else(|| CapabilityError::NotRegistered(invocation.capability_id.clone()))?
            .clone();

        // 2. ACL check
        if !self.has_access(&invocation.caller, &invocation.capability_id) {
            return Err(CapabilityError::AccessDenied);
        }

        // 3. Budget check
        if !self.budget.can_afford(
            &invocation.caller,
            definition.cost_per_call_cents,
            invocation.max_budget_cents,
        ) {
            let duration_ms = start.elapsed().as_millis() as u64;
            self.audit.record(
                invocation.caller.clone(),
                invocation.capability_id.clone(),
                duration_ms,
                0,
                "budget_exceeded",
            );
            return Ok(CapabilityResult::BudgetExceeded);
        }

        // 4. Rate limit check
        if let Err(retry_after_ms) = self.rate_limiter.try_acquire(&invocation.capability_id) {
            let duration_ms = start.elapsed().as_millis() as u64;
            self.audit.record(
                invocation.caller.clone(),
                invocation.capability_id.clone(),
                duration_ms,
                0,
                "rate_limited",
            );
            return Ok(CapabilityResult::RateLimited { retry_after_ms });
        }

        // 5. Secret resolution
        let api_key = definition
            .auth_secret_id
            .as_ref()
            .and_then(|sid| self.secrets.get(sid).map(|s| s.as_str()));

        // If a secret is required but missing, that's an error
        if definition.auth_secret_id.is_some() && api_key.is_none() {
            return Err(CapabilityError::Unavailable);
        }

        // 6 + 7. Execute with retry logic
        let result = self.execute_with_retry(&definition, invocation, api_key);

        // Record cost if successful
        let (cost, result_kind) = match &result {
            CapabilityResult::Success { cost_cents, .. } => {
                self.budget.record_spend(&invocation.caller, *cost_cents);
                (*cost_cents, result.kind_label())
            }
            _ => (0, result.kind_label()),
        };

        // 8. Audit
        let duration_ms = start.elapsed().as_millis() as u64;
        self.audit.record(
            invocation.caller.clone(),
            invocation.capability_id.clone(),
            duration_ms,
            cost,
            result_kind,
        );

        Ok(result)
    }

    fn execute_with_retry(
        &self,
        definition: &CapabilityDefinition,
        invocation: &CapabilityInvocation,
        api_key: Option<&str>,
    ) -> CapabilityResult {
        let mut retry_count = 0u32;

        loop {
            let result = self.executor.execute(definition, invocation, api_key);

            match &result {
                CapabilityResult::TransientError { .. } if retry_count < MAX_RETRIES => {
                    retry_count += 1;
                    // In production: exponential backoff sleep here
                    // For now (sync, no tokio): just retry immediately
                    continue;
                }
                CapabilityResult::TransientError { .. } => {
                    // Max retries exhausted
                    return CapabilityResult::TransientError { retry_count };
                }
                // No retry for any other result type
                _ => return result,
            }
        }
    }

    // ----- Query ----------------------------------------------------------

    /// Access the audit log (read-only).
    pub fn audit_log(&self) -> &AuditLog {
        &self.audit
    }

    /// Access the budget tracker (read-only).
    pub fn budget_tracker(&self) -> &BudgetTracker {
        &self.budget
    }

    /// Number of registered capabilities.
    pub fn registered_count(&self) -> usize {
        self.registry.len()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability_runtime::definition::Provider;
    use crate::capability_runtime::executor::MockExecutor;
    use crate::capability_runtime::result::Value;
    use crate::personal_graph::IdentityKind;

    fn idan() -> IdentityId {
        IdentityId::new(IdentityKind::Person, "idan")
    }

    fn roni() -> IdentityId {
        IdentityId::new(IdentityKind::Person, "roni")
    }

    fn whisper_def() -> CapabilityDefinition {
        CapabilityDefinition::new(
            "whisper.transcribe",
            "Transcribe audio",
            Provider::HttpPost,
        )
        .with_cost(3)
        .with_rate_limit(60)
    }

    fn whisper_def_with_secret() -> CapabilityDefinition {
        whisper_def().with_auth_secret("person:idan/api_key/openai")
    }

    fn invoke_whisper(caller: &IdentityId) -> CapabilityInvocation {
        CapabilityInvocation::new("whisper.transcribe", Value::Null, caller.clone())
            .with_budget(100)
    }

    fn setup_orchestrator() -> CapabilityOrchestrator {
        let budget = BudgetTracker::with_default_limit(1000);
        let mut orch = CapabilityOrchestrator::new(budget);
        orch.register(whisper_def());
        orch.grant_access(&idan(), "whisper.transcribe");
        orch
    }

    #[test]
    fn test_invoke_unregistered_fails() {
        let mut orch = setup_orchestrator();
        let inv = CapabilityInvocation::new("nonexistent", Value::Null, idan());
        let result = orch.invoke(&inv);
        assert!(matches!(result, Err(CapabilityError::NotRegistered(_))));
    }

    #[test]
    fn test_acl_blocks_unauthorized() {
        let mut orch = setup_orchestrator();
        // Roni has no access
        let inv = invoke_whisper(&roni());
        let result = orch.invoke(&inv);
        assert!(matches!(result, Err(CapabilityError::AccessDenied)));
    }

    #[test]
    fn test_successful_invocation() {
        let mut orch = setup_orchestrator();
        let inv = invoke_whisper(&idan());
        let result = orch.invoke(&inv).unwrap();
        assert!(result.is_success());
        if let CapabilityResult::Success { cost_cents, .. } = result {
            assert_eq!(cost_cents, 3);
        }
    }

    #[test]
    fn test_budget_exhaustion_blocks() {
        let budget = BudgetTracker::with_default_limit(5); // only 5 cents
        let mut orch = CapabilityOrchestrator::new(budget);
        orch.register(whisper_def()); // costs 3 cents
        orch.grant_access(&idan(), "whisper.transcribe");

        // First call: 3 cents → OK (total: 3)
        let inv = invoke_whisper(&idan());
        let r1 = orch.invoke(&inv).unwrap();
        assert!(r1.is_success());

        // Second call: 3 more → 6 > 5 → BudgetExceeded
        let r2 = orch.invoke(&inv).unwrap();
        assert!(matches!(r2, CapabilityResult::BudgetExceeded));
    }

    #[test]
    fn test_rate_limit_in_orchestrator() {
        let budget = BudgetTracker::new();
        let mut orch = CapabilityOrchestrator::new(budget);
        // Rate limit: 2 per minute
        orch.register(
            CapabilityDefinition::new("limited", "Limited", Provider::Local)
                .with_rate_limit(2)
                .with_cost(0),
        );
        orch.grant_access(&idan(), "limited");

        let inv = CapabilityInvocation::new("limited", Value::Null, idan());

        assert!(orch.invoke(&inv).unwrap().is_success());
        assert!(orch.invoke(&inv).unwrap().is_success());
        let r3 = orch.invoke(&inv).unwrap();
        assert!(matches!(r3, CapabilityResult::RateLimited { .. }));
    }

    #[test]
    fn test_retry_on_transient() {
        let budget = BudgetTracker::new();
        let mut orch = CapabilityOrchestrator::new(budget)
            .with_executor(Box::new(MockExecutor::transient()));
        orch.register(whisper_def());
        orch.grant_access(&idan(), "whisper.transcribe");

        let inv = invoke_whisper(&idan());
        let result = orch.invoke(&inv).unwrap();

        // Should have retried MAX_RETRIES times then returned TransientError
        match result {
            CapabilityResult::TransientError { retry_count } => {
                assert_eq!(retry_count, MAX_RETRIES);
            }
            other => panic!("expected TransientError, got {:?}", other),
        }
    }

    #[test]
    fn test_no_retry_on_permanent() {
        let budget = BudgetTracker::new();
        let mut orch = CapabilityOrchestrator::new(budget)
            .with_executor(Box::new(MockExecutor::permanent("bad key")));
        orch.register(whisper_def());
        orch.grant_access(&idan(), "whisper.transcribe");

        let inv = invoke_whisper(&idan());
        let result = orch.invoke(&inv).unwrap();

        assert!(matches!(
            result,
            CapabilityResult::PermanentError { reason } if reason == "bad key"
        ));
    }

    #[test]
    fn test_no_retry_on_timeout() {
        let budget = BudgetTracker::new();
        let mut orch = CapabilityOrchestrator::new(budget)
            .with_executor(Box::new(MockExecutor::timeout()));
        orch.register(whisper_def());
        orch.grant_access(&idan(), "whisper.transcribe");

        let inv = invoke_whisper(&idan());
        let result = orch.invoke(&inv).unwrap();

        assert!(matches!(result, CapabilityResult::Timeout));
    }

    #[test]
    fn test_audit_records_invocation() {
        let mut orch = setup_orchestrator();
        let inv = invoke_whisper(&idan());
        orch.invoke(&inv).unwrap();

        let log = orch.audit_log();
        assert_eq!(log.len(), 1);
        let entry = &log.entries()[0];
        assert_eq!(entry.caller, idan());
        assert_eq!(entry.capability_id, "whisper.transcribe");
        assert_eq!(entry.result_kind, "success");
        assert_eq!(entry.cost_cents, 3);
    }

    #[test]
    fn test_per_caller_budget_isolated() {
        let budget = BudgetTracker::with_default_limit(10);
        let mut orch = CapabilityOrchestrator::new(budget);
        orch.register(whisper_def()); // costs 3
        orch.grant_access(&idan(), "whisper.transcribe");
        orch.grant_access(&roni(), "whisper.transcribe");

        let inv_idan = invoke_whisper(&idan());
        let inv_roni = invoke_whisper(&roni());

        // Idan: 3, 6, 9 — all OK
        assert!(orch.invoke(&inv_idan).unwrap().is_success());
        assert!(orch.invoke(&inv_idan).unwrap().is_success());
        assert!(orch.invoke(&inv_idan).unwrap().is_success());
        // Idan: 12 > 10 → blocked
        assert!(matches!(
            orch.invoke(&inv_idan).unwrap(),
            CapabilityResult::BudgetExceeded
        ));
        // Roni: 0 + 3 = 3 → still OK
        assert!(orch.invoke(&inv_roni).unwrap().is_success());
    }

    #[test]
    fn test_secret_required_but_missing() {
        let budget = BudgetTracker::new();
        let mut orch = CapabilityOrchestrator::new(budget);
        orch.register(whisper_def_with_secret());
        orch.grant_access(&idan(), "whisper.transcribe");

        let inv = invoke_whisper(&idan());
        let result = orch.invoke(&inv);
        assert!(matches!(result, Err(CapabilityError::Unavailable)));
    }

    #[test]
    fn test_secret_present_allows_invocation() {
        let budget = BudgetTracker::new();
        let mut orch = CapabilityOrchestrator::new(budget);
        orch.register(whisper_def_with_secret());
        orch.grant_access(&idan(), "whisper.transcribe");
        orch.register_secret("person:idan/api_key/openai", "sk-test-12345");

        let inv = invoke_whisper(&idan());
        let result = orch.invoke(&inv).unwrap();
        assert!(result.is_success());
    }

    #[test]
    fn test_grant_revoke_access() {
        let mut orch = setup_orchestrator();

        // Idan has access
        assert!(orch.has_access(&idan(), "whisper.transcribe"));

        // Revoke
        orch.revoke_access(&idan(), "whisper.transcribe");
        assert!(!orch.has_access(&idan(), "whisper.transcribe"));

        // Invoke fails
        let inv = invoke_whisper(&idan());
        assert!(matches!(
            orch.invoke(&inv),
            Err(CapabilityError::AccessDenied)
        ));
    }

    #[test]
    fn test_multiple_capabilities() {
        let budget = BudgetTracker::new();
        let mut orch = CapabilityOrchestrator::new(budget);
        orch.register(whisper_def());
        orch.register(
            CapabilityDefinition::new("gemini.vision", "Analyze images", Provider::HttpPost)
                .with_cost(5),
        );
        orch.grant_access(&idan(), "whisper.transcribe");
        orch.grant_access(&idan(), "gemini.vision");

        assert_eq!(orch.registered_count(), 2);

        let inv1 = invoke_whisper(&idan());
        let inv2 = CapabilityInvocation::new("gemini.vision", Value::Null, idan());

        assert!(orch.invoke(&inv1).unwrap().is_success());
        assert!(orch.invoke(&inv2).unwrap().is_success());

        assert_eq!(orch.audit_log().len(), 2);
    }
}
