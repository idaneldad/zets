//! # Preferences — per-owner preference store with inference
//!
//! ZETS remembers **how** the owner wants to be served, not just what
//! they know. This module stores and infers behavioral preferences.
//!
//! ## Design
//!
//! Preferences are keyed by `(IdentityId, PreferenceKey)`. Each entry
//! has a value, an origin (explicit vs inferred vs default), a timestamp,
//! and a confidence score. Full history is preserved for audit.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use zets::preferences::{PreferenceStore, StandardKey};
//! use zets::personal_graph::{IdentityId, IdentityKind};
//!
//! let owner = IdentityId::new(IdentityKind::Person, "idan");
//! let mut store = PreferenceStore::new();
//!
//! // Explicit set
//! store.set_explicit(&owner, "tone", "formal", &owner, 1000);
//!
//! // Infer from history
//! store.infer_from_conversation(&owner, &messages, now_ms);
//!
//! // Read with fallback to system default
//! let effective = store.effective(&owner, "length"); // → Some("medium")
//! ```
//!
//! ## Conflict resolution
//!
//! - Explicit > Inferred > Default
//! - Newer Explicit > Older Explicit
//! - Higher-confidence Inferred > Lower-confidence Inferred
//!
//! ## Modules
//!
//! - `key`          — `PreferenceKey` type
//! - `value`        — `PreferenceValue` and `PreferenceOrigin`
//! - `standard_keys`— well-known key enum (`StandardKey`)
//! - `conflict`     — conflict resolution logic
//! - `inference`    — detect preferences from `HistoryEntry` slices
//! - `store`        — `PreferenceStore` — main API

pub mod conflict;
pub mod inference;
pub mod key;
pub mod standard_keys;
pub mod store;
pub mod value;

pub use conflict::PreferenceConflict;
pub use key::PreferenceKey;
pub use standard_keys::StandardKey;
pub use store::PreferenceStore;
pub use value::{PreferenceOrigin, PreferenceValue};
