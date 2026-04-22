//! Scopes — the 6 graph tiers of ZETS.
//!
//! After architectural review with Idan, ZETS organizes data into 6 named
//! scopes, each with its own security tier, storage strategy, and purpose.
//!
//! Routing between scopes is driven by the system graph itself — making
//! ZETS a "graph of graphs" where the registry is part of the knowledge
//! the system reflects on.
//!
//! ┌──────────────┬──────────────┬─────────────────────────────────────┐
//! │ Scope        │ Encryption   │ Content                             │
//! ├──────────────┼──────────────┼─────────────────────────────────────┤
//! │ System       │ Paranoid     │ Routes, rules, opcodes (the "IP")   │
//! │ User         │ Paranoid     │ Personal facts, history, prefs      │
//! │ Log          │ Signed       │ Decision trail, audit, reasoning    │
//! │ Testing      │ Plain        │ Sandboxed simulations, staging      │
//! │ Language     │ Signed       │ Per-lang lexicon + morphology       │
//! │ Data         │ Plain        │ Universal facts (public knowledge)  │
//! └──────────────┴──────────────┴─────────────────────────────────────┘

use std::path::PathBuf;

pub mod registry;
pub mod router;

pub use registry::{GraphScope, ScopeRegistry};
pub use router::{CascadeResult, ScopeRouter};

/// Identifies which graph scope a piece of data belongs to.
/// Stored as u8 for compactness (we have room for 256 future scopes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ScopeId {
    /// Routes and rules — the "how we think" part of ZETS.
    /// Highest encryption, IP protection.
    System = 0,

    /// Universal facts — the "what we all know" baseline.
    /// Plain (public knowledge, no privacy concern).
    Data = 1,

    /// Per-language lexicon + morphology rules.
    /// Signed but not encrypted (open format, tamper-detect).
    Language = 2,

    /// Personal — my facts, preferences, history.
    /// Paranoid encryption (owner-only access).
    User = 3,

    /// Decision trail — how did I arrive at each answer?
    /// Signed, append-only, reviewable.
    Log = 4,

    /// Sandbox for simulations, staged learning, A/B tests.
    /// Plain; ephemeral; wipeable anytime.
    Testing = 5,

    /// Shared with a group (family/team/org).
    /// Tier depends on group policy.
    Shared = 6,
}

impl ScopeId {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::System),
            1 => Some(Self::Data),
            2 => Some(Self::Language),
            3 => Some(Self::User),
            4 => Some(Self::Log),
            5 => Some(Self::Testing),
            6 => Some(Self::Shared),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Data => "data",
            Self::Language => "language",
            Self::User => "user",
            Self::Log => "log",
            Self::Testing => "testing",
            Self::Shared => "shared",
        }
    }

    /// Default encryption tier for this scope.
    pub fn default_encryption(self) -> EncryptionTier {
        match self {
            Self::System | Self::User => EncryptionTier::Paranoid,
            Self::Language | Self::Log => EncryptionTier::Signed,
            Self::Data | Self::Testing | Self::Shared => EncryptionTier::None,
        }
    }

    /// Can writes happen to this scope in normal operation?
    pub fn is_writable(self) -> bool {
        match self {
            Self::System => false,      // updated via signed bundle install
            Self::Data => false,        // updated via signed bundle install
            Self::Language => false,    // updated via signed bundle install
            Self::User => true,         // user owns this
            Self::Log => true,          // append-only
            Self::Testing => true,      // sandbox
            Self::Shared => true,       // group members can write
        }
    }

    /// Cascade priority — lower number = checked first.
    /// Used by ScopeRouter for query resolution.
    pub fn cascade_priority(self) -> u8 {
        match self {
            Self::Testing => 0,   // sandbox highest priority (current experiment)
            Self::User => 1,      // personal facts next
            Self::Shared => 2,    // group data
            Self::Language => 3,  // morphology help
            Self::Data => 4,      // universal facts
            Self::System => 5,    // meta-knowledge last
            Self::Log => 10,      // excluded from cascade (not for facts)
        }
    }
}

/// Encryption strength per scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionTier {
    /// No encryption, no signature.
    None,
    /// Ed25519 signed, verifiable but readable.
    Signed,
    /// AES-256-GCM with Argon2-hardened key.
    Paranoid,
}

impl EncryptionTier {
    pub fn description(self) -> &'static str {
        match self {
            Self::None => "plain",
            Self::Signed => "Ed25519 signed, readable",
            Self::Paranoid => "AES-256-GCM + Argon2",
        }
    }
}

/// Path conventions for a ZETS instance.
pub struct ScopePaths {
    pub root: PathBuf,
}

impl ScopePaths {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    pub fn system(&self) -> PathBuf {
        self.root.join("packs").join("zets.system")
    }

    pub fn data_core(&self) -> PathBuf {
        self.root.join("packs").join("zets.core")
    }

    pub fn language(&self, lang: &str) -> PathBuf {
        self.root.join("packs").join(format!("zets.{}", lang))
    }

    pub fn user(&self, user_id: &str) -> PathBuf {
        self.root.join("user").join(format!("{}.graph", user_id))
    }

    pub fn log(&self) -> PathBuf {
        self.root.join("log").join("decisions.log")
    }

    pub fn testing(&self, test_id: &str) -> PathBuf {
        self.root.join("testing").join(format!("{}.graph", test_id))
    }

    pub fn shared(&self, group_id: &str) -> PathBuf {
        self.root.join("shared").join(format!("{}.graph", group_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_roundtrip() {
        for s in [
            ScopeId::System, ScopeId::Data, ScopeId::Language,
            ScopeId::User, ScopeId::Log, ScopeId::Testing, ScopeId::Shared,
        ] {
            assert_eq!(ScopeId::from_u8(s.as_u8()), Some(s));
        }
    }

    #[test]
    fn system_and_user_are_paranoid() {
        assert_eq!(ScopeId::System.default_encryption(), EncryptionTier::Paranoid);
        assert_eq!(ScopeId::User.default_encryption(), EncryptionTier::Paranoid);
    }

    #[test]
    fn data_and_testing_are_plain() {
        assert_eq!(ScopeId::Data.default_encryption(), EncryptionTier::None);
        assert_eq!(ScopeId::Testing.default_encryption(), EncryptionTier::None);
    }

    #[test]
    fn language_and_log_are_signed() {
        assert_eq!(ScopeId::Language.default_encryption(), EncryptionTier::Signed);
        assert_eq!(ScopeId::Log.default_encryption(), EncryptionTier::Signed);
    }

    #[test]
    fn system_not_writable() {
        assert!(!ScopeId::System.is_writable());
        assert!(!ScopeId::Data.is_writable());
        assert!(ScopeId::User.is_writable());
        assert!(ScopeId::Log.is_writable());
        assert!(ScopeId::Testing.is_writable());
    }

    #[test]
    fn cascade_testing_first() {
        assert!(ScopeId::Testing.cascade_priority() < ScopeId::User.cascade_priority());
        assert!(ScopeId::User.cascade_priority() < ScopeId::Data.cascade_priority());
        assert!(ScopeId::Data.cascade_priority() < ScopeId::System.cascade_priority());
    }

    #[test]
    fn paths_construct_correctly() {
        let p = ScopePaths::new("/tmp/zets");
        assert_eq!(p.system().to_str().unwrap(), "/tmp/zets/packs/zets.system");
        assert_eq!(p.data_core().to_str().unwrap(), "/tmp/zets/packs/zets.core");
        assert_eq!(p.language("he").to_str().unwrap(), "/tmp/zets/packs/zets.he");
        assert_eq!(p.user("idan").to_str().unwrap(), "/tmp/zets/user/idan.graph");
    }
}
