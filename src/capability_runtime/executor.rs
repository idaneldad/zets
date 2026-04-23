//! # Capability executor
//!
//! Trait for executing capability invocations and a stub implementation
//! for v1. The real HTTP executor will be added when `reqwest` is available.

use super::definition::CapabilityDefinition;
use super::invocation::CapabilityInvocation;
use super::result::{CapabilityResult, Value};

/// Trait for executing a capability invocation.
///
/// Implementations handle the actual work: HTTP calls, local process
/// invocation, or custom logic. The orchestrator calls `execute` after
/// all pre-checks (ACL, budget, rate limit) pass.
pub trait Executor: std::fmt::Debug {
    /// Execute the invocation against the given capability definition.
    ///
    /// - `api_key`: the resolved secret (if any) from the vault.
    /// - Returns a `CapabilityResult` (never a `CapabilityError` — those
    ///   are caught before execution).
    fn execute(
        &self,
        definition: &CapabilityDefinition,
        invocation: &CapabilityInvocation,
        api_key: Option<&str>,
    ) -> CapabilityResult;
}

/// Stub executor that always returns `Success` with a dummy output.
///
/// Used for testing and as a placeholder until real HTTP execution is added.
/// TODO: Replace with HttpExecutor when reqwest is added to Cargo.toml.
#[derive(Debug)]
pub struct StubExecutor;

impl Executor for StubExecutor {
    fn execute(
        &self,
        definition: &CapabilityDefinition,
        _invocation: &CapabilityInvocation,
        _api_key: Option<&str>,
    ) -> CapabilityResult {
        CapabilityResult::Success {
            output: Value::Map(vec![
                ("stub".into(), Value::Bool(true)),
                (
                    "capability".into(),
                    Value::Text(definition.id.clone()),
                ),
            ]),
            cost_cents: definition.cost_per_call_cents,
            duration_ms: 1, // instant stub
        }
    }
}

/// Executor that returns a configurable result (for testing).
#[derive(Debug)]
pub struct MockExecutor {
    result: CapabilityResult,
}

impl MockExecutor {
    pub fn new(result: CapabilityResult) -> Self {
        MockExecutor { result }
    }

    /// Creates a mock that always returns a transient error.
    pub fn transient() -> Self {
        MockExecutor {
            result: CapabilityResult::TransientError { retry_count: 0 },
        }
    }

    /// Creates a mock that always returns a permanent error.
    pub fn permanent(reason: impl Into<String>) -> Self {
        MockExecutor {
            result: CapabilityResult::PermanentError {
                reason: reason.into(),
            },
        }
    }

    /// Creates a mock that always times out.
    pub fn timeout() -> Self {
        MockExecutor {
            result: CapabilityResult::Timeout,
        }
    }
}

impl Executor for MockExecutor {
    fn execute(
        &self,
        _definition: &CapabilityDefinition,
        _invocation: &CapabilityInvocation,
        _api_key: Option<&str>,
    ) -> CapabilityResult {
        self.result.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability_runtime::definition::Provider;
    use crate::personal_graph::{IdentityId, IdentityKind};

    fn test_def() -> CapabilityDefinition {
        CapabilityDefinition::new("test.cap", "Test capability", Provider::Local).with_cost(5)
    }

    fn test_inv() -> CapabilityInvocation {
        CapabilityInvocation::new(
            "test.cap",
            Value::Null,
            IdentityId::new(IdentityKind::Person, "tester"),
        )
    }

    #[test]
    fn test_stub_executor_returns_success() {
        let exec = StubExecutor;
        let result = exec.execute(&test_def(), &test_inv(), None);
        assert!(result.is_success());
        if let CapabilityResult::Success { cost_cents, .. } = result {
            assert_eq!(cost_cents, 5);
        }
    }

    #[test]
    fn test_mock_executor_transient() {
        let exec = MockExecutor::transient();
        let result = exec.execute(&test_def(), &test_inv(), None);
        assert!(matches!(result, CapabilityResult::TransientError { .. }));
    }

    #[test]
    fn test_mock_executor_permanent() {
        let exec = MockExecutor::permanent("API key invalid");
        let result = exec.execute(&test_def(), &test_inv(), None);
        assert!(matches!(result, CapabilityResult::PermanentError { .. }));
    }
}
