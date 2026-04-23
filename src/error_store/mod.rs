//! # ErrorStore — central place for everything that went wrong
//!
//! Without this, Phase B (benchmarking ZETS) is impossible: we can't
//! measure how well the system behaves if failures aren't captured.
//!
//! Design principles:
//!
//! 1. **Append-only.** Errors are never deleted, only marked resolved.
//! 2. **Dedup within short window.** Same kind + context + source within
//!    1 minute bumps occurrence_count instead of new entry.
//! 3. **Severity-tiered.** Info / Warn / Error / Critical / Security.
//!    Security class triggers separate handling (alerts, audit).
//! 4. **Kind-taxonomized.** Every error is one of 10 canonical kinds;
//!    unclassified goes to Other(String).
//! 5. **Trend-friendly.** Kind histogram + top_kinds() for dashboards.
//!
//! What feeds into this store:
//!   - Reader's GateHold decisions (not errors per se, but worth tracking)
//!   - Procedure execution failures
//!   - External API timeouts / errors
//!   - LLM call failures
//!   - ACL denials from the vault
//!   - User thumbs-down feedback
//!   - Ingestion pipeline failures (Phase C)
//!   - Graph inconsistencies found during walks

pub mod entry;
pub mod store;

pub use entry::{ErrorEntry, ErrorId, ErrorKind, Resolution, Severity};
pub use store::{ErrorStore, StoreStats};
