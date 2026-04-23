//! # Identity — the atomic entity in PersonalGraph
//!
//! An Identity is ANY entity that can be referred to: a person, organization,
//! vehicle, group, family, or any bounded subject. Identities are atoms in
//! the graph; their properties and relationships are edges.
//!
//! Design principle: identities are NEVER deleted. When something ceases
//! to exist (person dies, company closes, group ends), the identity stays
//! but its `lifecycle` changes. This preserves historical queries.

use std::fmt;

/// A unique reference to an identity in the graph.
///
/// Format: `{kind}:{id}` — e.g. `person:idan`, `org:chooz`, `vehicle:abc-123`.
/// The format is parseable, sortable, and human-readable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IdentityId(pub String);

impl IdentityId {
    pub fn new(kind: IdentityKind, id: impl Into<String>) -> Self {
        IdentityId(format!("{}:{}", kind.as_str(), id.into()))
    }

    pub fn kind(&self) -> Option<IdentityKind> {
        let prefix = self.0.split(':').next()?;
        IdentityKind::from_str(prefix)
    }

    pub fn local_id(&self) -> &str {
        self.0.split(':').nth(1).unwrap_or(&self.0)
    }
}

impl fmt::Display for IdentityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The kind of entity — determines what relationships and profiles apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentityKind {
    /// A human being.
    Person,
    /// A business, organization, company, non-profit.
    Org,
    /// A vehicle (car, truck, boat) — has owner + possible drivers.
    Vehicle,
    /// A bounded group (club, course, study group, sports team).
    Group,
    /// A family unit (distinct from Group — has kinship relationships).
    Family,
    /// A workplace (specific department within an Org).
    Workplace,
    /// A role-based entity (e.g. "CEO of CHOOZ") that can change holders.
    Role,
    /// A physical or virtual place (office, store, domain).
    Place,
    /// A device (phone, laptop, IoT device) — distinct from Vehicle.
    Device,
}

impl IdentityKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            IdentityKind::Person => "person",
            IdentityKind::Org => "org",
            IdentityKind::Vehicle => "vehicle",
            IdentityKind::Group => "group",
            IdentityKind::Family => "family",
            IdentityKind::Workplace => "workplace",
            IdentityKind::Role => "role",
            IdentityKind::Place => "place",
            IdentityKind::Device => "device",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "person" => Some(IdentityKind::Person),
            "org" => Some(IdentityKind::Org),
            "vehicle" => Some(IdentityKind::Vehicle),
            "group" => Some(IdentityKind::Group),
            "family" => Some(IdentityKind::Family),
            "workplace" => Some(IdentityKind::Workplace),
            "role" => Some(IdentityKind::Role),
            "place" => Some(IdentityKind::Place),
            "device" => Some(IdentityKind::Device),
            _ => None,
        }
    }

    /// Is this kind a human entity (Person, Family)?
    pub fn is_human(&self) -> bool {
        matches!(self, IdentityKind::Person | IdentityKind::Family)
    }

    /// Can this kind have multiple holders over time? (e.g. a Role,
    /// a Vehicle can have multiple drivers).
    pub fn supports_multi_holder(&self) -> bool {
        matches!(
            self,
            IdentityKind::Role | IdentityKind::Vehicle | IdentityKind::Device
        )
    }
}

/// Lifecycle of an identity — never deleted, status changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Lifecycle {
    /// Active, in current use.
    #[default]
    Active,
    /// Suspended — temporarily not in use (vacation, leave).
    Suspended,
    /// Inactive — not in current use but could become active again
    /// (ex-employee, former group member).
    Inactive,
    /// Archived — definitively ended (deceased, dissolved, permanently closed).
    Archived,
}

impl Lifecycle {
    pub fn as_str(&self) -> &'static str {
        match self {
            Lifecycle::Active => "active",
            Lifecycle::Suspended => "suspended",
            Lifecycle::Inactive => "inactive",
            Lifecycle::Archived => "archived",
        }
    }

    /// Can this identity still receive new interactions?
    pub fn accepts_interaction(&self) -> bool {
        matches!(self, Lifecycle::Active | Lifecycle::Suspended)
    }

    /// Should this identity appear in "current" lookups by default?
    pub fn is_current(&self) -> bool {
        matches!(self, Lifecycle::Active)
    }
}

/// An Identity — the atomic entity in PersonalGraph.
#[derive(Debug, Clone)]
pub struct Identity {
    pub id: IdentityId,
    pub kind: IdentityKind,
    /// Display name (for humans to read in UIs, logs).
    pub display_name: String,
    /// Current lifecycle status.
    pub lifecycle: Lifecycle,
    /// When this identity was created in ZETS (Unix ms).
    pub created_ms: i64,
    /// When lifecycle last changed (Unix ms).
    pub lifecycle_changed_ms: i64,
    /// Optional: when archived, why. Free-text.
    pub archive_reason: Option<String>,
    /// Scope — which PersonalGraph this belongs to, if any.
    /// None = global (e.g. a public organization, shared knowledge).
    pub scope: Option<ScopeRef>,
}

/// Which PersonalGraph an identity belongs to.
///
/// A Person's data lives under their own scope. An Organization's data
/// might be scoped to the Owner who created the entry, or be global
/// (publicly known company).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScopeRef {
    /// The root identity (usually a Person) whose graph this belongs to.
    pub root: IdentityId,
    /// Visibility level.
    pub visibility: Visibility,
}

/// How widely this identity/relationship is visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Visibility {
    /// Only visible within the owning scope.
    #[default]
    Private,
    /// Visible to collaborators invited by the owner.
    Shared,
    /// Visible to the Owner's workplace or family.
    Unit,
    /// Public — anyone can see (e.g. a public company entry).
    Public,
}

impl Visibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Visibility::Private => "private",
            Visibility::Shared => "shared",
            Visibility::Unit => "unit",
            Visibility::Public => "public",
        }
    }
}

impl Identity {
    pub fn new(
        kind: IdentityKind,
        local_id: impl Into<String>,
        display_name: impl Into<String>,
        now_ms: i64,
    ) -> Self {
        let id = IdentityId::new(kind, local_id);
        Identity {
            id,
            kind,
            display_name: display_name.into(),
            lifecycle: Lifecycle::Active,
            created_ms: now_ms,
            lifecycle_changed_ms: now_ms,
            archive_reason: None,
            scope: None,
        }
    }

    /// Change lifecycle, recording the moment.
    pub fn transition(&mut self, new_state: Lifecycle, now_ms: i64, reason: Option<String>) {
        self.lifecycle = new_state;
        self.lifecycle_changed_ms = now_ms;
        if new_state == Lifecycle::Archived {
            self.archive_reason = reason;
        }
    }

    /// Set or update scope.
    pub fn in_scope(mut self, scope: ScopeRef) -> Self {
        self.scope = Some(scope);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_id_format() {
        let id = IdentityId::new(IdentityKind::Person, "idan");
        assert_eq!(id.0, "person:idan");
        assert_eq!(id.kind(), Some(IdentityKind::Person));
        assert_eq!(id.local_id(), "idan");
    }

    #[test]
    fn test_identity_id_org() {
        let id = IdentityId::new(IdentityKind::Org, "chooz");
        assert_eq!(id.0, "org:chooz");
    }

    #[test]
    fn test_identity_kind_is_human() {
        assert!(IdentityKind::Person.is_human());
        assert!(IdentityKind::Family.is_human());
        assert!(!IdentityKind::Org.is_human());
        assert!(!IdentityKind::Vehicle.is_human());
    }

    #[test]
    fn test_multi_holder() {
        assert!(IdentityKind::Vehicle.supports_multi_holder());
        assert!(IdentityKind::Role.supports_multi_holder());
        assert!(!IdentityKind::Person.supports_multi_holder());
    }

    #[test]
    fn test_lifecycle_transitions() {
        let mut id = Identity::new(IdentityKind::Person, "test", "Test", 1000);
        assert_eq!(id.lifecycle, Lifecycle::Active);

        id.transition(Lifecycle::Inactive, 2000, None);
        assert_eq!(id.lifecycle, Lifecycle::Inactive);
        assert_eq!(id.lifecycle_changed_ms, 2000);

        id.transition(
            Lifecycle::Archived,
            3000,
            Some("deceased 2025".into()),
        );
        assert_eq!(id.lifecycle, Lifecycle::Archived);
        assert_eq!(id.archive_reason.as_deref(), Some("deceased 2025"));
    }

    #[test]
    fn test_lifecycle_interaction() {
        assert!(Lifecycle::Active.accepts_interaction());
        assert!(Lifecycle::Suspended.accepts_interaction());
        assert!(!Lifecycle::Inactive.accepts_interaction());
        assert!(!Lifecycle::Archived.accepts_interaction());
    }

    #[test]
    fn test_lifecycle_current() {
        assert!(Lifecycle::Active.is_current());
        assert!(!Lifecycle::Suspended.is_current());
        assert!(!Lifecycle::Inactive.is_current());
        assert!(!Lifecycle::Archived.is_current());
    }

    #[test]
    fn test_scope_visibility() {
        let id = Identity::new(IdentityKind::Person, "idan", "Idan", 1000).in_scope(
            ScopeRef {
                root: IdentityId::new(IdentityKind::Person, "idan"),
                visibility: Visibility::Private,
            },
        );
        assert!(id.scope.is_some());
        assert_eq!(id.scope.unwrap().visibility, Visibility::Private);
    }

    #[test]
    fn test_round_trip_kind() {
        for kind in [
            IdentityKind::Person,
            IdentityKind::Org,
            IdentityKind::Vehicle,
            IdentityKind::Group,
            IdentityKind::Family,
            IdentityKind::Workplace,
            IdentityKind::Role,
            IdentityKind::Place,
            IdentityKind::Device,
        ] {
            let s = kind.as_str();
            assert_eq!(IdentityKind::from_str(s), Some(kind));
        }
    }
}
