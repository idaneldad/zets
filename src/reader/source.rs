//! # Source — who/what is sending input to ZETS
//!
//! Every input has a source. The source determines:
//!   - default trust level
//!   - which tone profiles apply
//!   - which gates are relevant
//!   - whether Owner-configured overrides kick in
//!
//! This is intentionally broader than "user" — inputs can come from humans,
//! from external APIs, from other ZETS instances, or from ZETS itself
//! (self-initiated learning, curiosity, background tasks).

use std::fmt;

/// The kind of entity sending input.
///
/// Stored as an atom in the graph under the `source_kind` namespace.
/// Edges from a `Source` atom encode its default trust, tone priors,
/// and relationship to the owner.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Source {
    /// A human user — the one who owns or configures this ZETS instance,
    /// or someone acting on their behalf.
    User {
        id: String,
        role: UserRole,
    },

    /// A business or personal client — someone the Owner serves.
    /// The `owner` field points back to the User who owns the relationship.
    Client {
        id: String,
        owner: String,
        role: ClientRole,
    },

    /// An anonymous or first-contact entity. No id tracked.
    Guest {
        session: String,
    },

    /// A machine or service contacting ZETS via API (Zapier, webhook, ...).
    ExternalApi {
        id: String,
        kind: ApiKind,
    },

    /// Another ZETS instance (peer-to-peer, same tier).
    PeerZets {
        instance_id: String,
    },

    /// The master orchestration ZETS (higher-tier, trusted).
    ZetsMaster {
        instance_id: String,
    },

    /// ZETS itself initiated this input (background learning, curiosity,
    /// self-reflection, scheduled tasks).
    SelfInitiated {
        origin: SelfOrigin,
    },
}

/// Role of a User.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UserRole {
    /// The person who configured this instance. Highest authority.
    Owner,
    /// A collaborator with the Owner — can configure parts of the system.
    Collaborator,
    /// An admin who helps operate but does not own.
    Admin,
}

/// Role of a Client (relative to the Owner).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClientRole {
    /// A direct business or service client.
    Direct,
    /// A family member or related person of a client (acting on their behalf).
    Related,
    /// A potential client who has not yet converted.
    Prospect,
    /// Someone who referred a client.
    Referrer,
}

/// Kind of external API source.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ApiKind {
    /// General automation (Zapier, n8n, Make).
    Automation,
    /// A webhook fired by another service.
    Webhook,
    /// A scheduled batch job.
    Batch,
    /// Integration with a specific third-party product.
    Integration(String),
}

/// Why ZETS initiated this itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SelfOrigin {
    /// Curiosity — ZETS detected a knowledge gap.
    Curiosity,
    /// Learning — scheduled knowledge expansion.
    Learning,
    /// Reflection — reviewing past interactions.
    Reflection,
    /// Maintenance — housekeeping, indexing, compaction.
    Maintenance,
}

impl Source {
    /// Returns the sense-key used to look up this source's atom in the graph.
    ///
    /// Format: `source.{kind}.{role_or_subkind}` — hierarchical for
    /// prefix-based walks.
    pub fn sense_key(&self) -> String {
        match self {
            Source::User { role, .. } => format!("source.user.{}", role.as_str()),
            Source::Client { role, .. } => format!("source.client.{}", role.as_str()),
            Source::Guest { .. } => "source.guest".to_string(),
            Source::ExternalApi { kind, .. } => {
                format!("source.external_api.{}", kind.as_str())
            }
            Source::PeerZets { .. } => "source.peer_zets".to_string(),
            Source::ZetsMaster { .. } => "source.zets_master".to_string(),
            Source::SelfInitiated { origin } => {
                format!("source.self_initiated.{}", origin.as_str())
            }
        }
    }

    /// Default trust tier for this source kind.
    ///
    /// Can be overridden per-instance (Owner configures) — this is just
    /// the out-of-the-box default.
    pub fn default_trust(&self) -> TrustTier {
        match self {
            Source::User { role: UserRole::Owner, .. } => TrustTier::Full,
            Source::User { role: UserRole::Collaborator, .. } => TrustTier::High,
            Source::User { role: UserRole::Admin, .. } => TrustTier::High,
            Source::Client { role: ClientRole::Direct, .. } => TrustTier::Known,
            Source::Client { role: ClientRole::Related, .. } => TrustTier::Mid,
            Source::Client { role: ClientRole::Prospect, .. } => TrustTier::Cautious,
            Source::Client { role: ClientRole::Referrer, .. } => TrustTier::Mid,
            Source::Guest { .. } => TrustTier::Cautious,
            Source::ExternalApi { .. } => TrustTier::Limited,
            Source::PeerZets { .. } => TrustTier::Known,
            Source::ZetsMaster { .. } => TrustTier::Full,
            Source::SelfInitiated { .. } => TrustTier::Full,
        }
    }

    /// Is this source a human? (affects tone adaptation — no point
    /// mirroring sentence-length for an API).
    pub fn is_human(&self) -> bool {
        matches!(
            self,
            Source::User { .. } | Source::Client { .. } | Source::Guest { .. }
        )
    }

    /// Does this source get personality-based adaptation?
    pub fn deserves_adaptation(&self) -> bool {
        self.is_human()
    }

    /// Identifier for logging and tracing.
    pub fn identifier(&self) -> String {
        match self {
            Source::User { id, .. } => format!("user:{}", id),
            Source::Client { id, owner, .. } => format!("client:{}@{}", id, owner),
            Source::Guest { session } => format!("guest:{}", session),
            Source::ExternalApi { id, .. } => format!("api:{}", id),
            Source::PeerZets { instance_id } => format!("peer:{}", instance_id),
            Source::ZetsMaster { instance_id } => format!("master:{}", instance_id),
            Source::SelfInitiated { origin } => format!("self:{}", origin.as_str()),
        }
    }
}

/// Trust tier — how much latitude this source gets.
///
/// Distinct from `TrustLevel` on procedures (System/OwnerVerified/Learned/
/// Experimental) — that is about code, this is about the **speaker**.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TrustTier {
    /// Anonymous or highly restricted.
    Cautious,
    /// Limited access — external APIs, unknown.
    Limited,
    /// Mid — known but not deeply trusted.
    Mid,
    /// Known — we recognize this entity, moderate trust.
    Known,
    /// High — trusted collaborator.
    High,
    /// Full — the owner, or ZETS itself.
    Full,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Owner => "owner",
            UserRole::Collaborator => "collaborator",
            UserRole::Admin => "admin",
        }
    }
}

impl ClientRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClientRole::Direct => "direct",
            ClientRole::Related => "related",
            ClientRole::Prospect => "prospect",
            ClientRole::Referrer => "referrer",
        }
    }
}

impl ApiKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiKind::Automation => "automation",
            ApiKind::Webhook => "webhook",
            ApiKind::Batch => "batch",
            ApiKind::Integration(_) => "integration",
        }
    }
}

impl SelfOrigin {
    pub fn as_str(&self) -> &'static str {
        match self {
            SelfOrigin::Curiosity => "curiosity",
            SelfOrigin::Learning => "learning",
            SelfOrigin::Reflection => "reflection",
            SelfOrigin::Maintenance => "maintenance",
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.identifier())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sense_key_owner() {
        let s = Source::User {
            id: "idan".into(),
            role: UserRole::Owner,
        };
        assert_eq!(s.sense_key(), "source.user.owner");
    }

    #[test]
    fn test_sense_key_client() {
        let s = Source::Client {
            id: "c42".into(),
            owner: "idan".into(),
            role: ClientRole::Direct,
        };
        assert_eq!(s.sense_key(), "source.client.direct");
    }

    #[test]
    fn test_sense_key_self() {
        let s = Source::SelfInitiated {
            origin: SelfOrigin::Curiosity,
        };
        assert_eq!(s.sense_key(), "source.self_initiated.curiosity");
    }

    #[test]
    fn test_trust_owner_full() {
        let s = Source::User {
            id: "idan".into(),
            role: UserRole::Owner,
        };
        assert_eq!(s.default_trust(), TrustTier::Full);
    }

    #[test]
    fn test_trust_guest_cautious() {
        let s = Source::Guest {
            session: "x".into(),
        };
        assert_eq!(s.default_trust(), TrustTier::Cautious);
    }

    #[test]
    fn test_trust_api_limited() {
        let s = Source::ExternalApi {
            id: "zapier".into(),
            kind: ApiKind::Automation,
        };
        assert_eq!(s.default_trust(), TrustTier::Limited);
    }

    #[test]
    fn test_is_human() {
        let human = Source::User {
            id: "a".into(),
            role: UserRole::Owner,
        };
        let api = Source::ExternalApi {
            id: "b".into(),
            kind: ApiKind::Webhook,
        };
        assert!(human.is_human());
        assert!(!api.is_human());
    }

    #[test]
    fn test_deserves_adaptation() {
        let client = Source::Client {
            id: "c".into(),
            owner: "idan".into(),
            role: ClientRole::Direct,
        };
        let master = Source::ZetsMaster {
            instance_id: "m".into(),
        };
        assert!(client.deserves_adaptation());
        assert!(!master.deserves_adaptation());
    }

    #[test]
    fn test_identifier_format() {
        let s = Source::Client {
            id: "c42".into(),
            owner: "idan".into(),
            role: ClientRole::Direct,
        };
        assert_eq!(s.identifier(), "client:c42@idan");
    }

    #[test]
    fn test_trust_ordering() {
        assert!(TrustTier::Full > TrustTier::High);
        assert!(TrustTier::High > TrustTier::Known);
        assert!(TrustTier::Cautious < TrustTier::Limited);
    }
}
