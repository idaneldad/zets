//! # Secrets — API keys, tokens, passwords, business codes
//!
//! ## Architecture
//!
//! Secrets split into TWO parts, by design:
//!
//! 1. **SecretRef** (in the graph) — metadata only:
//!    - Who owns it
//!    - Who can access it (ACL)
//!    - When it was last rotated
//!    - Status (active/revoked/expired)
//!    - NEVER the value itself
//!
//! 2. **Vault** (separate encrypted file) — the values:
//!    - AES-GCM encrypted (placeholder XOR in v1 — replace before prod)
//!    - 0600 permissions on disk
//!    - Master key from env var or OS keychain
//!    - Never logged, never dumped, never written to graph
//!
//! ## Why split
//!
//! If the graph is compromised (dump, debug trace, backup leak), secrets
//! remain safe. The graph can reveal "Idan has an OpenAI key" — but not
//! the key itself.
//!
//! ## Scoping (per Idan's ask)
//!
//! Secrets are owned by either a Person (User) or an Org (Company).
//! An employee has personal secrets; the company has company-wide secrets;
//! the employee may be ON the ACL of some company secrets via the
//! `WorksAt` relationship in PersonalGraph.
//!
//! Example:
//!   - `person:idan/api_key/openai` — Idan's personal key
//!   - `org:chooz/oauth/gmail` — CHOOZ's company Gmail integration
//!   - `person:employee1/api_key/their_own_api` — employee's personal key
//!
//! These are distinct atoms, different SecretIds, never mixed.
//!
//! ## Usage pattern
//!
//! ```ignore
//! let vault = Vault::open("/secure/vault.enc", master_key)?;
//! let sref = graph.get_secret_ref("person:idan/api_key/openai")?;
//! let value = vault.get(&sref, &caller)?;  // ACL checked
//! // use value...
//! // value is cloned bytes; zeroize after use recommended
//! ```

pub mod secret_ref;
pub mod vault;

pub use secret_ref::{SecretId, SecretKind, SecretRef, SecretStatus};
pub use vault::{Vault, VaultError};
