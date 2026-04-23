//! # Capability invocation request
//!
//! `CapabilityInvocation` represents a single request to invoke a capability,
//! including caller identity, arguments, timeout, and budget ceiling.

use crate::personal_graph::IdentityId;

use super::result::Value;

/// A request to invoke a registered capability.
#[derive(Debug, Clone)]
pub struct CapabilityInvocation {
    /// Which capability to invoke, e.g. `"whisper.transcribe"`.
    pub capability_id: String,
    /// Capability-specific arguments.
    pub args: Value,
    /// Who is calling (for audit + ACL).
    pub caller: IdentityId,
    /// Per-call timeout in milliseconds.
    pub max_timeout_ms: u64,
    /// Cost ceiling in cents — reject if the call would exceed this.
    pub max_budget_cents: u32,
}

impl CapabilityInvocation {
    pub fn new(
        capability_id: impl Into<String>,
        args: Value,
        caller: IdentityId,
    ) -> Self {
        CapabilityInvocation {
            capability_id: capability_id.into(),
            args,
            caller,
            max_timeout_ms: 30_000, // 30s default
            max_budget_cents: 100,   // $1.00 default
        }
    }

    /// Builder: set timeout.
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.max_timeout_ms = ms;
        self
    }

    /// Builder: set budget ceiling.
    pub fn with_budget(mut self, cents: u32) -> Self {
        self.max_budget_cents = cents;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::personal_graph::IdentityKind;

    #[test]
    fn test_invocation_builder() {
        let caller = IdentityId::new(IdentityKind::Person, "idan");
        let inv = CapabilityInvocation::new(
            "whisper.transcribe",
            Value::Map(vec![("file".into(), Value::Text("audio.wav".into()))]),
            caller,
        )
        .with_timeout(10_000)
        .with_budget(50);

        assert_eq!(inv.capability_id, "whisper.transcribe");
        assert_eq!(inv.max_timeout_ms, 10_000);
        assert_eq!(inv.max_budget_cents, 50);
    }
}
