//! # PersonalGraph — scoped identity + relationship storage
//!
//! A PersonalGraph is rooted at one identity (usually a Person) and
//! contains identities and relationships scoped to them.
//!
//! ## Design
//!
//! Each ZETS installation hosts multiple PersonalGraphs:
//!   - Owner's own graph (Idan's personal connections)
//!   - Per-client graphs (each CHOOZ client sees their own view)
//!   - Shared graphs (a workplace shared by multiple Owners)
//!
//! Graphs can overlap: a Company identity might appear in many graphs
//! (Idan's view of CHOOZ, an employee's view, a client's view). The
//! same Identity atom is referenced by all — the RELATIONSHIPS differ.
//!
//! ## Never delete
//!
//! Identities and relationships are never removed. When a connection
//! ends (divorce, job change, death), the record stays with status
//! updated. This lets historical queries work ("who was my employee
//! in 2022?") even after the connection ended.

use std::collections::HashMap;

use super::identity::{Identity, IdentityId, IdentityKind, Lifecycle};
use super::relationship::{RelKind, RelStatus, Relationship};

/// A scoped personal graph.
///
/// Holds identities and relationships belonging to (or visible from)
/// a root identity.
#[derive(Debug, Clone)]
pub struct PersonalGraph {
    /// The identity whose graph this is.
    pub root: IdentityId,
    /// All identities known in this graph, indexed by id.
    identities: HashMap<IdentityId, Identity>,
    /// All relationships. Stored flat; indexed on demand.
    relationships: Vec<Relationship>,
    /// When this graph was created.
    created_ms: i64,
}

impl PersonalGraph {
    pub fn new(root: IdentityId, now_ms: i64) -> Self {
        PersonalGraph {
            root,
            identities: HashMap::new(),
            relationships: Vec::new(),
            created_ms: now_ms,
        }
    }

    /// Add or update an identity.
    pub fn upsert_identity(&mut self, identity: Identity) -> IdentityId {
        let id = identity.id.clone();
        self.identities.insert(id.clone(), identity);
        id
    }

    /// Get an identity by id (regardless of lifecycle).
    pub fn get_identity(&self, id: &IdentityId) -> Option<&Identity> {
        self.identities.get(id)
    }

    /// Get identity mutably.
    pub fn get_identity_mut(&mut self, id: &IdentityId) -> Option<&mut Identity> {
        self.identities.get_mut(id)
    }

    /// Add a relationship.
    pub fn add_relationship(&mut self, rel: Relationship) {
        self.relationships.push(rel);
    }

    /// Find all relationships where `id` appears as source or target,
    /// filtered by current status.
    pub fn relationships_for(&self, id: &IdentityId) -> Vec<&Relationship> {
        self.relationships
            .iter()
            .filter(|r| &r.from == id || &r.to == id)
            .collect()
    }

    /// Find CURRENTLY ACTIVE relationships for an id.
    pub fn current_relationships(&self, id: &IdentityId) -> Vec<&Relationship> {
        self.relationships_for(id)
            .into_iter()
            .filter(|r| r.is_current())
            .collect()
    }

    /// Find relationships of a specific kind for an id (any status).
    pub fn relationships_of_kind(&self, id: &IdentityId, kind: &RelKind) -> Vec<&Relationship> {
        self.relationships_for(id)
            .into_iter()
            .filter(|r| &r.kind == kind)
            .collect()
    }

    /// Find relationships active at a given point in time.
    pub fn relationships_at(&self, id: &IdentityId, ts_ms: i64) -> Vec<&Relationship> {
        self.relationships_for(id)
            .into_iter()
            .filter(|r| r.was_active_at(ts_ms))
            .collect()
    }

    /// End a relationship (find-and-update by (from, to, kind)).
    /// Returns true if found and ended.
    pub fn end_relationship(
        &mut self,
        from: &IdentityId,
        to: &IdentityId,
        kind: &RelKind,
        at_ms: i64,
        status: RelStatus,
    ) -> bool {
        for rel in self.relationships.iter_mut() {
            if &rel.from == from && &rel.to == to && &rel.kind == kind && rel.is_current() {
                rel.end(at_ms, status);
                return true;
            }
        }
        false
    }

    /// Traverse neighbors: get identities linked to `id` by any active edge.
    pub fn active_neighbors(&self, id: &IdentityId) -> Vec<&IdentityId> {
        self.current_relationships(id)
            .into_iter()
            .map(|r| if &r.from == id { &r.to } else { &r.from })
            .collect()
    }

    /// Count of identities by kind (lifecycle filter optional).
    pub fn count_by_kind(&self, kind: IdentityKind, only_active: bool) -> usize {
        self.identities
            .values()
            .filter(|i| i.kind == kind)
            .filter(|i| !only_active || i.lifecycle == Lifecycle::Active)
            .count()
    }

    /// Identities matching predicate.
    pub fn find_identities<F>(&self, pred: F) -> Vec<&Identity>
    where
        F: Fn(&Identity) -> bool,
    {
        self.identities.values().filter(|i| pred(i)).collect()
    }

    /// Mark an identity as having changed lifecycle — convenience method
    /// that also ends all current relationships if archived.
    pub fn transition_identity(
        &mut self,
        id: &IdentityId,
        new_state: Lifecycle,
        at_ms: i64,
        reason: Option<String>,
    ) -> bool {
        // Clone relationships to avoid borrow conflict
        let rel_keys_to_end: Vec<(IdentityId, IdentityId, RelKind)> =
            if new_state == Lifecycle::Archived {
                self.current_relationships(id)
                    .iter()
                    .map(|r| (r.from.clone(), r.to.clone(), r.kind.clone()))
                    .collect()
            } else {
                Vec::new()
            };

        if let Some(ident) = self.get_identity_mut(id) {
            ident.transition(new_state, at_ms, reason);

            // If archived, end all active relationships
            for (from, to, kind) in rel_keys_to_end {
                self.end_relationship(&from, &to, &kind, at_ms, RelStatus::Ended);
            }
            true
        } else {
            false
        }
    }

    /// Stats for quick diagnosis.
    pub fn stats(&self) -> GraphStats {
        GraphStats {
            total_identities: self.identities.len(),
            active_identities: self
                .identities
                .values()
                .filter(|i| i.lifecycle == Lifecycle::Active)
                .count(),
            total_relationships: self.relationships.len(),
            active_relationships: self.relationships.iter().filter(|r| r.is_current()).count(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GraphStats {
    pub total_identities: usize,
    pub active_identities: usize,
    pub total_relationships: usize,
    pub active_relationships: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_person(id: &str, t: i64) -> Identity {
        Identity::new(IdentityKind::Person, id, id, t)
    }
    fn mk_org(id: &str, t: i64) -> Identity {
        Identity::new(IdentityKind::Org, id, id, t)
    }

    #[test]
    fn test_new_graph_empty() {
        let root = IdentityId::new(IdentityKind::Person, "idan");
        let g = PersonalGraph::new(root, 1000);
        assert_eq!(g.stats().total_identities, 0);
    }

    #[test]
    fn test_add_identity_and_find() {
        let root = IdentityId::new(IdentityKind::Person, "idan");
        let mut g = PersonalGraph::new(root, 1000);
        g.upsert_identity(mk_person("idan", 1000));
        g.upsert_identity(mk_org("chooz", 1000));

        assert_eq!(g.stats().total_identities, 2);
        assert_eq!(g.count_by_kind(IdentityKind::Person, false), 1);
        assert_eq!(g.count_by_kind(IdentityKind::Org, false), 1);
    }

    #[test]
    fn test_add_relationship_and_query() {
        let root = IdentityId::new(IdentityKind::Person, "idan");
        let mut g = PersonalGraph::new(root, 1000);
        let idan = g.upsert_identity(mk_person("idan", 1000));
        let chooz = g.upsert_identity(mk_org("chooz", 1000));

        g.add_relationship(Relationship::new(
            idan.clone(),
            chooz.clone(),
            RelKind::WorksAt,
            1000,
            1000,
        ));

        let rels = g.current_relationships(&idan);
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].kind, RelKind::WorksAt);
    }

    #[test]
    fn test_multiple_workplaces() {
        // One person works at 2 different orgs simultaneously
        let root = IdentityId::new(IdentityKind::Person, "idan");
        let mut g = PersonalGraph::new(root, 1000);
        let idan = g.upsert_identity(mk_person("idan", 1000));
        let chooz = g.upsert_identity(mk_org("chooz", 1000));
        let dinio = g.upsert_identity(mk_org("dinio", 1000));

        g.add_relationship(Relationship::new(
            idan.clone(),
            chooz,
            RelKind::WorksAt,
            1000,
            1000,
        ));
        g.add_relationship(Relationship::new(
            idan.clone(),
            dinio,
            RelKind::WorksAt,
            1000,
            1000,
        ));

        assert_eq!(g.current_relationships(&idan).len(), 2);
        assert_eq!(g.active_neighbors(&idan).len(), 2);
    }

    #[test]
    fn test_historical_preserved() {
        // Person leaves one org, joins another — both stay in graph
        let root = IdentityId::new(IdentityKind::Person, "p1");
        let mut g = PersonalGraph::new(root, 1000);
        let person = g.upsert_identity(mk_person("p1", 1000));
        let old_job = g.upsert_identity(mk_org("oldco", 1000));
        let new_job = g.upsert_identity(mk_org("newco", 2000));

        g.add_relationship(Relationship::new(
            person.clone(),
            old_job.clone(),
            RelKind::WorksAt,
            1000,
            1000,
        ));
        g.end_relationship(&person, &old_job, &RelKind::WorksAt, 1800, RelStatus::Ended);
        g.add_relationship(Relationship::new(
            person.clone(),
            new_job,
            RelKind::WorksAt,
            2000,
            2000,
        ));

        // Today: 1 current relationship
        assert_eq!(g.current_relationships(&person).len(), 1);
        // All time: 2 relationships
        assert_eq!(g.relationships_for(&person).len(), 2);
        // At time 1500: was at oldco
        let past = g.relationships_at(&person, 1500);
        assert_eq!(past.len(), 1);
        assert_eq!(past[0].to.local_id(), "oldco");
    }

    #[test]
    fn test_archive_ends_all_relationships() {
        // When a person is archived (e.g. deceased), all their
        // active relationships should end.
        let root = IdentityId::new(IdentityKind::Person, "root");
        let mut g = PersonalGraph::new(root, 1000);
        let person = g.upsert_identity(mk_person("person", 1000));
        let job = g.upsert_identity(mk_org("job", 1000));
        let friend = g.upsert_identity(mk_person("friend", 1000));

        g.add_relationship(Relationship::new(
            person.clone(),
            job,
            RelKind::WorksAt,
            1000,
            1000,
        ));
        g.add_relationship(Relationship::new(
            person.clone(),
            friend,
            RelKind::FriendOf,
            1000,
            1000,
        ));

        g.transition_identity(
            &person,
            Lifecycle::Archived,
            5000,
            Some("passed away".into()),
        );

        assert_eq!(g.current_relationships(&person).len(), 0);
        // But history preserved:
        assert_eq!(g.relationships_for(&person).len(), 2);
    }

    #[test]
    fn test_vehicle_multiple_drivers_across_time() {
        let root = IdentityId::new(IdentityKind::Person, "owner");
        let mut g = PersonalGraph::new(root, 1000);

        let car = g.upsert_identity(Identity::new(
            IdentityKind::Vehicle,
            "car1",
            "Family car",
            1000,
        ));
        let idan = g.upsert_identity(mk_person("idan", 1000));
        let roni = g.upsert_identity(mk_person("roni", 1000));

        g.add_relationship(Relationship::new(
            idan.clone(),
            car.clone(),
            RelKind::Drives,
            1000,
            1000,
        ));
        g.add_relationship(Relationship::new(
            roni.clone(),
            car.clone(),
            RelKind::Drives,
            1000,
            1000,
        ));

        let drivers = g.current_relationships(&car);
        assert_eq!(drivers.len(), 2);
    }

    #[test]
    fn test_find_identities() {
        let root = IdentityId::new(IdentityKind::Person, "r");
        let mut g = PersonalGraph::new(root, 1000);
        g.upsert_identity(mk_person("a", 1000));
        g.upsert_identity(mk_person("b", 1000));
        g.upsert_identity(mk_org("x", 1000));

        let persons = g.find_identities(|i| i.kind == IdentityKind::Person);
        assert_eq!(persons.len(), 2);
    }

    #[test]
    fn test_stats_correct() {
        let root = IdentityId::new(IdentityKind::Person, "r");
        let mut g = PersonalGraph::new(root, 1000);
        g.upsert_identity(mk_person("a", 1000));
        let mut archived = mk_person("b", 1000);
        archived.transition(Lifecycle::Archived, 2000, None);
        g.upsert_identity(archived);

        let s = g.stats();
        assert_eq!(s.total_identities, 2);
        assert_eq!(s.active_identities, 1);
    }
}
