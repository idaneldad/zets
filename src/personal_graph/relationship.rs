//! # Relationship — edges between identities in PersonalGraph
//!
//! A Relationship is a time-bounded edge: "identity A is in relationship
//! R to identity B, from time T1 to T2 (or ongoing), with status S".
//!
//! Relationships are NEVER deleted — when a connection ends, the
//! relationship's `status` becomes `Ended` and `valid_until` is set.
//! This preserves history: "who worked at CHOOZ in 2020?" still works.
//!
//! Multi-holder patterns are naturally expressed: a Vehicle can have
//! multiple `DrivenBy` relationships simultaneously (one per driver),
//! each with its own time window.

use super::identity::IdentityId;

/// A single relationship between two identities.
#[derive(Debug, Clone)]
pub struct Relationship {
    pub from: IdentityId,
    pub to: IdentityId,
    pub kind: RelKind,
    /// Status of this specific connection.
    pub status: RelStatus,
    /// When the relationship began (Unix ms).
    pub valid_from: i64,
    /// When it ended. None = ongoing.
    pub valid_until: Option<i64>,
    /// Optional strength (0..1) — how strong is this connection?
    pub strength: f32,
    /// Optional: free-text context.
    pub note: Option<String>,
    /// When this record was created in ZETS.
    pub recorded_ms: i64,
}

/// Canonical relationship kinds.
///
/// Stored as atoms in the graph under `rel_kind.*`. Custom kinds can be
/// added via `Custom(String)` for domain-specific connections.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RelKind {
    // ─── Employment ─────────────────────────────
    /// A person works at an org / workplace.
    WorksAt,
    /// A person holds a role at an org.
    HoldsRole,
    /// An org employs a person.
    Employs,
    /// A person founded an org.
    Founded,
    /// A person owns an org / vehicle / device.
    Owns,
    /// Former workplace, now ended.
    WorkedAt,

    // ─── Client/Service ─────────────────────────
    /// An entity is a client of an org.
    ClientOf,
    /// An entity is a supplier to an org.
    SupplierOf,
    /// A person referred another to a service.
    Referred,

    // ─── Family ─────────────────────────────────
    /// Parent-child.
    ParentOf,
    /// Child-parent.
    ChildOf,
    /// Siblings.
    SiblingOf,
    /// Spouse/partner.
    PartnerOf,
    /// Member of a family unit.
    MemberOfFamily,

    // ─── Groups & Organizations ─────────────────
    /// Member of a group, club, or association.
    MemberOf,
    /// Leads/runs a group or org.
    Leads,
    /// Attends/participates in an event or class.
    Attends,

    // ─── Vehicles & Devices ─────────────────────
    /// Drives a vehicle (one of possibly many drivers).
    Drives,
    /// Uses a device (owner or authorized user).
    Uses,

    // ─── Places ─────────────────────────────────
    /// Located at a place (workplace, home).
    LocatedAt,
    /// Originates from a place (country of birth, hometown).
    OriginatesFrom,

    // ─── Social ─────────────────────────────────
    /// Friend.
    FriendOf,
    /// Knows (acquaintance, weaker than friend).
    Knows,
    /// Trusts (explicit trust assertion).
    Trusts,
    /// Has mentored / been mentored by.
    Mentors,

    // ─── Custom ─────────────────────────────────
    /// Domain-specific relationship — sense_key in the graph.
    Custom(String),
}

impl RelKind {
    pub fn sense_key(&self) -> String {
        let base = match self {
            RelKind::WorksAt => "works_at",
            RelKind::HoldsRole => "holds_role",
            RelKind::Employs => "employs",
            RelKind::Founded => "founded",
            RelKind::Owns => "owns",
            RelKind::WorkedAt => "worked_at",
            RelKind::ClientOf => "client_of",
            RelKind::SupplierOf => "supplier_of",
            RelKind::Referred => "referred",
            RelKind::ParentOf => "parent_of",
            RelKind::ChildOf => "child_of",
            RelKind::SiblingOf => "sibling_of",
            RelKind::PartnerOf => "partner_of",
            RelKind::MemberOfFamily => "member_of_family",
            RelKind::MemberOf => "member_of",
            RelKind::Leads => "leads",
            RelKind::Attends => "attends",
            RelKind::Drives => "drives",
            RelKind::Uses => "uses",
            RelKind::LocatedAt => "located_at",
            RelKind::OriginatesFrom => "originates_from",
            RelKind::FriendOf => "friend_of",
            RelKind::Knows => "knows",
            RelKind::Trusts => "trusts",
            RelKind::Mentors => "mentors",
            RelKind::Custom(s) => return format!("rel_kind.custom.{}", s),
        };
        format!("rel_kind.{}", base)
    }

    /// Is this a symmetric relationship? (if A→B, then B→A automatically)
    pub fn is_symmetric(&self) -> bool {
        matches!(
            self,
            RelKind::SiblingOf
                | RelKind::PartnerOf
                | RelKind::FriendOf
                | RelKind::Knows
        )
    }

    /// Returns the inverse relationship, if defined.
    /// E.g. ParentOf ↔ ChildOf, Employs ↔ WorksAt.
    pub fn inverse(&self) -> Option<RelKind> {
        match self {
            RelKind::ParentOf => Some(RelKind::ChildOf),
            RelKind::ChildOf => Some(RelKind::ParentOf),
            RelKind::Employs => Some(RelKind::WorksAt),
            RelKind::WorksAt => Some(RelKind::Employs),
            RelKind::ClientOf => Some(RelKind::SupplierOf),
            RelKind::SupplierOf => Some(RelKind::ClientOf),
            RelKind::Mentors => Some(RelKind::Custom("mentored_by".to_string())),
            _ if self.is_symmetric() => Some(self.clone()),
            _ => None,
        }
    }
}

/// Status of a relationship at a point in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RelStatus {
    /// Ongoing, currently in force.
    #[default]
    Active,
    /// Paused, can resume (leave, sabbatical).
    Suspended,
    /// Ended normally (quit, graduated).
    Ended,
    /// Ended abnormally (fired, expelled, died).
    Terminated,
    /// Was tentative, never materialized.
    Canceled,
}

impl RelStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelStatus::Active => "active",
            RelStatus::Suspended => "suspended",
            RelStatus::Ended => "ended",
            RelStatus::Terminated => "terminated",
            RelStatus::Canceled => "canceled",
        }
    }

    pub fn is_current(&self) -> bool {
        matches!(self, RelStatus::Active)
    }

    pub fn is_historical(&self) -> bool {
        matches!(
            self,
            RelStatus::Ended | RelStatus::Terminated | RelStatus::Canceled
        )
    }
}

impl Relationship {
    pub fn new(
        from: IdentityId,
        to: IdentityId,
        kind: RelKind,
        valid_from: i64,
        now_ms: i64,
    ) -> Self {
        Relationship {
            from,
            to,
            kind,
            status: RelStatus::Active,
            valid_from,
            valid_until: None,
            strength: 0.5,
            note: None,
            recorded_ms: now_ms,
        }
    }

    /// End this relationship at the given time.
    pub fn end(&mut self, at_ms: i64, status: RelStatus) {
        self.status = status;
        self.valid_until = Some(at_ms);
    }

    /// Was this relationship valid at the given point in time?
    pub fn was_active_at(&self, ts_ms: i64) -> bool {
        if ts_ms < self.valid_from {
            return false;
        }
        match self.valid_until {
            None => true, // ongoing
            Some(end) => ts_ms < end,
        }
    }

    /// Is this relationship currently active?
    pub fn is_current(&self) -> bool {
        self.status == RelStatus::Active && self.valid_until.is_none()
    }

    pub fn with_strength(mut self, strength: f32) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::identity::IdentityKind;

    fn p(id: &str) -> IdentityId {
        IdentityId::new(IdentityKind::Person, id)
    }
    fn o(id: &str) -> IdentityId {
        IdentityId::new(IdentityKind::Org, id)
    }

    #[test]
    fn test_rel_kind_sense_key() {
        assert_eq!(RelKind::WorksAt.sense_key(), "rel_kind.works_at");
        assert_eq!(RelKind::ParentOf.sense_key(), "rel_kind.parent_of");
    }

    #[test]
    fn test_rel_kind_custom() {
        let k = RelKind::Custom("advisor_to".into());
        assert_eq!(k.sense_key(), "rel_kind.custom.advisor_to");
    }

    #[test]
    fn test_inverse() {
        assert_eq!(RelKind::ParentOf.inverse(), Some(RelKind::ChildOf));
        assert_eq!(RelKind::Employs.inverse(), Some(RelKind::WorksAt));
        assert_eq!(RelKind::SiblingOf.inverse(), Some(RelKind::SiblingOf));
    }

    #[test]
    fn test_symmetric() {
        assert!(RelKind::SiblingOf.is_symmetric());
        assert!(RelKind::FriendOf.is_symmetric());
        assert!(!RelKind::ParentOf.is_symmetric());
    }

    #[test]
    fn test_relationship_lifecycle() {
        let mut r = Relationship::new(p("idan"), o("chooz"), RelKind::WorksAt, 1000, 1000);
        assert!(r.is_current());
        assert!(r.was_active_at(1500));

        r.end(2000, RelStatus::Ended);
        assert!(!r.is_current());
        assert!(r.was_active_at(1500)); // was active between 1000-2000
        assert!(!r.was_active_at(2500)); // ended at 2000
    }

    #[test]
    fn test_strength_clamped() {
        let r = Relationship::new(p("a"), p("b"), RelKind::FriendOf, 0, 0)
            .with_strength(1.5);
        assert_eq!(r.strength, 1.0);
    }

    #[test]
    fn test_multi_driver_scenario() {
        // Vehicle has 3 drivers at different times
        let vehicle = IdentityId::new(IdentityKind::Vehicle, "abc-123");

        let driver1 = Relationship::new(p("idan"), vehicle.clone(), RelKind::Drives, 1000, 1000);
        let mut driver2 = Relationship::new(p("roni"), vehicle.clone(), RelKind::Drives, 1500, 1500);
        driver2.end(2000, RelStatus::Ended);
        let driver3 = Relationship::new(p("shai"), vehicle, RelKind::Drives, 2500, 2500);

        // At time 1500: idan + roni both driving
        assert!(driver1.was_active_at(1500));
        assert!(driver2.was_active_at(1500));
        assert!(!driver3.was_active_at(1500));

        // At time 2500: only idan + shai
        assert!(driver1.was_active_at(2500));
        assert!(!driver2.was_active_at(2500));
        assert!(driver3.was_active_at(2500));
    }

    #[test]
    fn test_status_classification() {
        assert!(RelStatus::Active.is_current());
        assert!(!RelStatus::Ended.is_current());
        assert!(RelStatus::Ended.is_historical());
        assert!(RelStatus::Terminated.is_historical());
        assert!(!RelStatus::Suspended.is_historical());
    }

    #[test]
    fn test_was_active_before_start() {
        let r = Relationship::new(p("a"), o("b"), RelKind::WorksAt, 5000, 5000);
        assert!(!r.was_active_at(1000)); // before it started
        assert!(r.was_active_at(6000));
    }
}
