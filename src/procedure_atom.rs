//! Procedure Atom — OpenClaw-inspired skill model, native to ZETS graph.
//!
//! A procedure is an atom in the graph describing HOW to do something.
//! Matches OpenClaw's SKILL.md contract (name/description/allowed-tools/
//! when-to-use/when-not-to-use/install) but stored as graph atoms + DAG of
//! steps, so ZETS can (a) find them via sense matching, (b) compose them
//! (paths of paths), (c) gate them with trust/permission layers.
//!
//! # Example — "send WhatsApp via GreenAPI"
//!
//! ```text
//! procedure:send_whatsapp_via_greenapi  (atom)
//! ├── step 1 → procedure:resolve_contact  (atom → sub-procedure)
//! ├── step 2 → procedure:check_permission (atom → sub-procedure)
//! ├── step 3 → procedure:compose_hebrew_text
//! ├── step 4 → procedure:http_post
//! │           ├── step 4a → procedure:fetch_with_ssrf_guard
//! │           └── step 4b → procedure:parse_json_response
//! └── step 5 → procedure:log_to_graph
//! ```
//!
//! Every step IS an atom. Every atom that is itself a procedure can be
//! walked into. Paths of paths.
//!
//! # Permission model (inspired by OpenClaw's 5 layers)
//!
//! 1. `trust_level` — System/OwnerVerified/Learned/Experimental
//! 2. `allowed_tools` — whitelist of tool IDs this procedure may invoke
//! 3. `required_permissions` — capabilities the caller must hold
//! 4. `invocation_source` — which identity trust levels may invoke (Owner/Paired/Any)
//! 5. `rate_limit` — per-caller per-window ceiling

use std::collections::HashSet;

/// Trust level — enforced by the VM before execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TrustLevel {
    /// Hardcoded in Rust, cannot be modified at runtime.
    System = 0,
    /// Owner (עידן) approved this procedure explicitly.
    OwnerVerified = 1,
    /// Extracted from corpus / learned, runs in sandbox only until verified.
    Learned = 2,
    /// Simulation-only — returns what WOULD happen, does not execute.
    Experimental = 3,
}

impl TrustLevel {
    /// Procedures at or ABOVE this level may execute directly.
    /// Below → sandbox or simulation.
    pub fn allows_direct_execution(self, policy_min: TrustLevel) -> bool {
        (self as u8) <= (policy_min as u8)
    }
}

/// Who may invoke this procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvocationSource {
    /// Only the owner (עידן).
    OwnerOnly,
    /// Owner + peers who have completed pairing.
    Paired,
    /// Anyone — including unknown channels. Use sparingly.
    Any,
    /// Only other procedures at System trust level (internal).
    SystemOnly,
}

/// An install spec — how to set up external deps before this procedure works.
#[derive(Debug, Clone)]
pub enum InstallSpec {
    Brew { formula: String },
    Apt { package: String },
    Npm { spec: String },
    Cargo { crate_name: String, version: Option<String> },
    Python { package: String },
    /// Fetch a file from a URL (must pass SSRF guard).
    Fetch { url: String, dest: String, sha256: String },
    /// No external install needed.
    None,
}

/// Precondition expressed as a graph query pattern.
#[derive(Debug, Clone)]
pub struct Precondition {
    pub description: String,
    /// A graph query — if it matches, precondition holds.
    /// Simplified here to a string; real impl would use a proper GraphQuery.
    pub query: String,
}

/// Rate limit per identity.
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub max_calls: u32,
    pub window_secs: u32,
}

/// A single step in a procedure — either a call to another procedure or a
/// primitive bytecode sequence.
#[derive(Debug, Clone)]
pub enum ProcedureStep {
    /// Call another procedure by ID. The args map source-step slots to
    /// target procedure's parameters.
    CallProcedure {
        procedure_id: u32,
        on_success: Option<u16>,
        on_failure: Option<u16>,
    },
    /// Execute primitive bytecode (from the 32 opcodes).
    /// Kept as bytes here; would decode into Vec<Opcode> in VM.
    ExecuteBytecode { bytecode: Vec<u8> },
    /// Conditional branch.
    IfElse {
        condition_query: String,
        if_true_step: u16,
        if_false_step: u16,
    },
    /// Bounded loop (max iterations enforced).
    Loop {
        condition_query: String,
        body_step: u16,
        max_iterations: u32,
    },
    /// Parallel execution — all steps run, join according to JoinType.
    Parallel {
        step_ids: Vec<u16>,
        join: JoinType,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    /// Wait for all to complete.
    AllComplete,
    /// First to complete wins.
    FirstComplete,
    /// At least N must complete.
    AtLeast(u8),
}

/// The full procedure atom.
#[derive(Debug, Clone)]
pub struct ProcedureAtom {
    /// Unique ID within the ProcedureStore.
    pub id: u32,
    /// Human-readable name (e.g. "send_whatsapp_via_greenapi").
    pub name: String,
    /// Short description (mirrors OpenClaw SKILL.md `description` field).
    pub description: String,
    /// Sense keys that match this procedure (for intent → procedure lookup).
    pub when_to_use: Vec<String>,
    /// When NOT to use (prevents mis-routing).
    pub when_not_to_use: Vec<String>,
    /// Tool IDs this procedure may invoke. Enforced before each CallTool.
    pub allowed_tools: HashSet<u32>,
    /// Capabilities the caller must hold.
    pub required_permissions: Vec<String>,
    /// Who may invoke this procedure.
    pub invocation_source: InvocationSource,
    /// Preconditions evaluated before execution.
    pub preconditions: Vec<Precondition>,
    /// Expected post-state (for verification).
    pub postconditions: Vec<Precondition>,
    /// How to install external deps.
    pub install: InstallSpec,
    /// Ordered steps (the DAG).
    pub steps: Vec<ProcedureStep>,
    /// Trust level — gates execution mode.
    pub trust_level: TrustLevel,
    /// Who added this procedure (None = system).
    pub owner: Option<u32>,
    /// Monotonic version (for updates).
    pub version: u32,
    /// Rate limit per caller.
    pub rate_limit: Option<RateLimit>,
    /// Max execution cost (opcodes). Bounded VM enforces this.
    pub max_cost: u32,
}

impl ProcedureAtom {
    /// Check whether a given caller may invoke this procedure.
    ///
    /// Returns Ok(()) if allowed, Err(reason) if denied. This is the PRIMARY
    /// entry point for permission enforcement.
    pub fn check_invocation(
        &self,
        caller_trust: CallerTrust,
        held_permissions: &HashSet<String>,
    ) -> Result<(), String> {
        // Layer 1: invocation source
        match (self.invocation_source, caller_trust) {
            (InvocationSource::OwnerOnly, CallerTrust::Owner) => {},
            (InvocationSource::OwnerOnly, _) => {
                return Err("procedure is owner-only".to_string())
            },
            (InvocationSource::Paired, CallerTrust::Owner | CallerTrust::Paired) => {},
            (InvocationSource::Paired, _) => {
                return Err("procedure requires paired identity".to_string())
            },
            (InvocationSource::SystemOnly, CallerTrust::System) => {},
            (InvocationSource::SystemOnly, _) => {
                return Err("procedure is system-internal only".to_string())
            },
            (InvocationSource::Any, _) => {},
        }

        // Layer 2: required permissions
        for required in &self.required_permissions {
            if !held_permissions.contains(required) {
                return Err(format!("missing permission: {}", required));
            }
        }

        Ok(())
    }

    /// Can this procedure execute directly, or must it run in sandbox/sim?
    pub fn execution_mode(&self) -> ExecutionMode {
        match self.trust_level {
            TrustLevel::System | TrustLevel::OwnerVerified => ExecutionMode::Direct,
            TrustLevel::Learned => ExecutionMode::Sandboxed,
            TrustLevel::Experimental => ExecutionMode::SimulationOnly,
        }
    }
}

/// Trust level of the caller of a procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallerTrust {
    /// Owner of the ZETS instance.
    Owner,
    /// Paired peer (via pairing code).
    Paired,
    /// Unknown external peer (usually denied at gateway).
    Unknown,
    /// Another procedure at System trust level (internal dispatch).
    System,
}

/// How a procedure execution runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Direct execution — side effects applied to main graph.
    Direct,
    /// Sandboxed — side effects to transaction, committed on success.
    Sandboxed,
    /// Simulation — no side effects, returns trace only.
    SimulationOnly,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_owner_only_proc(trust: TrustLevel) -> ProcedureAtom {
        ProcedureAtom {
            id: 1,
            name: "test_proc".into(),
            description: "test".into(),
            when_to_use: vec![],
            when_not_to_use: vec![],
            allowed_tools: HashSet::new(),
            required_permissions: vec![],
            invocation_source: InvocationSource::OwnerOnly,
            preconditions: vec![],
            postconditions: vec![],
            install: InstallSpec::None,
            steps: vec![],
            trust_level: trust,
            owner: None,
            version: 1,
            rate_limit: None,
            max_cost: 1000,
        }
    }

    #[test]
    fn owner_only_rejects_unknown() {
        let p = make_owner_only_proc(TrustLevel::System);
        let held = HashSet::new();
        let result = p.check_invocation(CallerTrust::Unknown, &held);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("owner-only"));
    }

    #[test]
    fn owner_only_accepts_owner() {
        let p = make_owner_only_proc(TrustLevel::System);
        let held = HashSet::new();
        assert!(p.check_invocation(CallerTrust::Owner, &held).is_ok());
    }

    #[test]
    fn paired_accepts_paired() {
        let mut p = make_owner_only_proc(TrustLevel::System);
        p.invocation_source = InvocationSource::Paired;
        let held = HashSet::new();
        assert!(p.check_invocation(CallerTrust::Paired, &held).is_ok());
        assert!(p.check_invocation(CallerTrust::Owner, &held).is_ok());
        assert!(p.check_invocation(CallerTrust::Unknown, &held).is_err());
    }

    #[test]
    fn missing_permission_denies() {
        let mut p = make_owner_only_proc(TrustLevel::System);
        p.required_permissions = vec!["send_whatsapp".into()];
        let held = HashSet::new();
        let result = p.check_invocation(CallerTrust::Owner, &held);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("send_whatsapp"));
    }

    #[test]
    fn held_permission_allows() {
        let mut p = make_owner_only_proc(TrustLevel::System);
        p.required_permissions = vec!["send_whatsapp".into()];
        let mut held = HashSet::new();
        held.insert("send_whatsapp".into());
        assert!(p.check_invocation(CallerTrust::Owner, &held).is_ok());
    }

    #[test]
    fn system_procedure_executes_direct() {
        let p = make_owner_only_proc(TrustLevel::System);
        assert_eq!(p.execution_mode(), ExecutionMode::Direct);
    }

    #[test]
    fn owner_verified_executes_direct() {
        let p = make_owner_only_proc(TrustLevel::OwnerVerified);
        assert_eq!(p.execution_mode(), ExecutionMode::Direct);
    }

    #[test]
    fn learned_runs_sandboxed() {
        let p = make_owner_only_proc(TrustLevel::Learned);
        assert_eq!(p.execution_mode(), ExecutionMode::Sandboxed);
    }

    #[test]
    fn experimental_is_simulation_only() {
        let p = make_owner_only_proc(TrustLevel::Experimental);
        assert_eq!(p.execution_mode(), ExecutionMode::SimulationOnly);
    }

    #[test]
    fn trust_level_ordering() {
        assert!((TrustLevel::System as u8) < (TrustLevel::OwnerVerified as u8));
        assert!((TrustLevel::OwnerVerified as u8) < (TrustLevel::Learned as u8));
        assert!((TrustLevel::Learned as u8) < (TrustLevel::Experimental as u8));
    }

    #[test]
    fn system_procedure_allows_direct_at_any_policy() {
        let p = make_owner_only_proc(TrustLevel::System);
        assert!(p.trust_level.allows_direct_execution(TrustLevel::Experimental));
        assert!(p.trust_level.allows_direct_execution(TrustLevel::OwnerVerified));
    }

    #[test]
    fn experimental_denied_at_strict_policy() {
        let p = make_owner_only_proc(TrustLevel::Experimental);
        assert!(!p.trust_level.allows_direct_execution(TrustLevel::OwnerVerified));
    }
}
