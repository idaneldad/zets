//! # CapabilityRuntime — invoke external capabilities safely
//!
//! This module provides a framework for invoking external APIs (Whisper,
//! Gemini Vision, Midjourney, Suno, Sora, etc.) with:
//!
//! - **Registry**: register and look up capability definitions
//! - **ACL**: per-caller permission checks
//! - **Budget tracking**: per-caller monotonic cost accounting
//! - **Rate limiting**: token-bucket per capability
//! - **Retry logic**: exponential backoff for transient errors
//! - **Audit logging**: append-only, PII-free invocation records
//!
//! ## Quick start
//!
//! ```ignore
//! use zets::capability_runtime::*;
//!
//! let budget = BudgetTracker::with_default_limit(1000); // $10 max per caller
//! let mut orch = CapabilityOrchestrator::new(budget);
//!
//! // Register a capability
//! orch.register(
//!     CapabilityDefinition::new("whisper.transcribe", "Transcribe audio", Provider::HttpPost)
//!         .with_cost(3)
//!         .with_rate_limit(60),
//! );
//!
//! // Grant access
//! orch.grant_access(&caller_id, "whisper.transcribe");
//!
//! // Invoke
//! let result = orch.invoke(&invocation)?;
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                CapabilityOrchestrator                │
//! │                                                     │
//! │  invoke(invocation)                                 │
//! │    ├── 1. Registry lookup                           │
//! │    ├── 2. ACL check                                 │
//! │    ├── 3. Budget check         ← BudgetTracker      │
//! │    ├── 4. Rate limit check     ← RateLimiter        │
//! │    ├── 5. Secret resolution    ← Vault (stub)       │
//! │    ├── 6. Execute              ← Executor trait      │
//! │    ├── 7. Retry on transient   (up to 3×)           │
//! │    └── 8. Audit log            ← AuditLog           │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Module structure
//!
//! - `result` — `Value`, `CapabilityResult`, `CapabilityError`
//! - `definition` — `Provider`, `CapabilityDefinition`
//! - `invocation` — `CapabilityInvocation`
//! - `registry` — `ConnectorRegistry`
//! - `budget` — `BudgetTracker`
//! - `rate_limit` — `RateLimiter`
//! - `executor` — `Executor` trait, `StubExecutor`, `MockExecutor`
//! - `audit` — `AuditLog`, `AuditEntry`
//! - `orchestrator` — `CapabilityOrchestrator`

pub mod audit;
pub mod budget;
pub mod definition;
pub mod executor;
pub mod invocation;
pub mod orchestrator;
pub mod rate_limit;
pub mod registry;
pub mod result;

// Re-exports for convenience
pub use audit::{AuditEntry, AuditLog};
pub use budget::BudgetTracker;
pub use definition::{CapabilityDefinition, Provider};
pub use executor::{Executor, MockExecutor, StubExecutor};
pub use invocation::CapabilityInvocation;
pub use orchestrator::CapabilityOrchestrator;
pub use rate_limit::RateLimiter;
pub use registry::ConnectorRegistry;
pub use result::{CapabilityError, CapabilityResult, Value};
