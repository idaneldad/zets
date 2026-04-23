//! # PersonalGraph — people, places, organizations, and their relationships
//!
//! A scoped graph layer distinct from ZETS's main knowledge graph.
//! Where the main graph holds facts about the world, the PersonalGraph
//! holds facts about **specific identities** — people, businesses,
//! vehicles, groups, families — and how they connect.
//!
//! ## Why a separate layer?
//!
//! Personal data has different properties:
//!   - **Scoped visibility**: Idan's client list is not public knowledge.
//!   - **Mutable relationships**: jobs change, families form and dissolve,
//!     groups end — but history must be preserved for audit and memory.
//!   - **Multi-holder patterns**: vehicles have drivers; roles have holders;
//!     workplaces have employees — the same identity connects to many.
//!   - **Cross-graph reference**: an organization might appear in
//!     hundreds of personal graphs with different facets visible.
//!
//! ## Multiplicity
//!
//! Each ZETS installation holds many PersonalGraphs:
//!   - The Owner's (Idan's personal connections)
//!   - Per-client views (each client gets their own angle)
//!   - Shared graphs (team workspaces)
//!
//! Cross-graph syncing (for globally-known orgs, public figures) happens
//! via reference — not copy. Same IdentityId in multiple graphs = same
//! entity; different relationship sets = different personal views.
//!
//! ## Never-delete principle
//!
//! Identities and relationships are archived, not erased. A deceased
//! person, a closed company, an ended friendship — all stay as history.
//! This enables:
//!   - "Who was on my team last year?"
//!   - "Which clients were active during Q3 2024?"
//!   - Reliable memory across lifecycle changes.
//!
//! ## Relationship to Reader
//!
//! The Reader (src/reader/) uses PersonalGraph to resolve Source metadata:
//!   - Who is this client? → lookup in Owner's PersonalGraph
//!   - What's their relationship to me? → active relationships
//!   - Should I trust this caller? → based on known relationships
//!
//! PersonalGraph feeds the Reader; the Reader writes back observations
//! (new connections discovered, relationships strengthened).

pub mod graph;
pub mod identity;
pub mod relationship;

pub use graph::{GraphStats, PersonalGraph};
pub use identity::{Identity, IdentityId, IdentityKind, Lifecycle, ScopeRef, Visibility};
pub use relationship::{RelKind, RelStatus, Relationship};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_scenario_owner_with_clients_and_family() {
        // Scenario: Idan has CHOOZ (his company), family, and clients.
        let root_id = IdentityId::new(IdentityKind::Person, "idan");
        let mut g = PersonalGraph::new(root_id.clone(), 1000);

        // Identities
        g.upsert_identity(Identity::new(IdentityKind::Person, "idan", "Idan", 1000));
        g.upsert_identity(Identity::new(IdentityKind::Person, "roni", "Roni", 1000));
        g.upsert_identity(Identity::new(IdentityKind::Org, "chooz", "CHOOZ", 1000));
        g.upsert_identity(Identity::new(
            IdentityKind::Family,
            "eldad",
            "Eldad Family",
            1000,
        ));
        g.upsert_identity(Identity::new(IdentityKind::Person, "client1", "Client 1", 1000));
        g.upsert_identity(Identity::new(IdentityKind::Person, "client2", "Client 2", 1000));

        // Relationships
        let idan = IdentityId::new(IdentityKind::Person, "idan");
        let roni = IdentityId::new(IdentityKind::Person, "roni");
        let chooz = IdentityId::new(IdentityKind::Org, "chooz");
        let family = IdentityId::new(IdentityKind::Family, "eldad");
        let c1 = IdentityId::new(IdentityKind::Person, "client1");
        let c2 = IdentityId::new(IdentityKind::Person, "client2");

        // Owner relationships
        g.add_relationship(Relationship::new(
            idan.clone(),
            chooz.clone(),
            RelKind::Founded,
            1000,
            1000,
        ));
        g.add_relationship(Relationship::new(
            idan.clone(),
            chooz.clone(),
            RelKind::Owns,
            1000,
            1000,
        ));

        // Family
        g.add_relationship(Relationship::new(
            idan.clone(),
            roni.clone(),
            RelKind::PartnerOf,
            1000,
            1000,
        ));
        g.add_relationship(Relationship::new(
            idan.clone(),
            family.clone(),
            RelKind::MemberOfFamily,
            1000,
            1000,
        ));
        g.add_relationship(Relationship::new(
            roni,
            family,
            RelKind::MemberOfFamily,
            1000,
            1000,
        ));

        // Clients
        g.add_relationship(Relationship::new(
            c1,
            chooz.clone(),
            RelKind::ClientOf,
            1000,
            1000,
        ));
        g.add_relationship(Relationship::new(
            c2,
            chooz,
            RelKind::ClientOf,
            1000,
            1000,
        ));

        // Verify: Idan has 4 active direct relationships
        let idan_rels = g.current_relationships(&idan);
        assert_eq!(idan_rels.len(), 4); // Founded + Owns + PartnerOf + MemberOfFamily

        // CHOOZ has 2 clients currently
        let chooz_id = IdentityId::new(IdentityKind::Org, "chooz");
        let chooz_clients: Vec<_> = g
            .relationships_of_kind(&chooz_id, &RelKind::ClientOf)
            .into_iter()
            .filter(|r| r.is_current())
            .collect();
        assert_eq!(chooz_clients.len(), 2);

        // Stats
        let s = g.stats();
        assert_eq!(s.total_identities, 6);
        assert_eq!(s.active_identities, 6);
    }

    #[test]
    fn test_client_leaves_but_history_remembered() {
        let root_id = IdentityId::new(IdentityKind::Person, "idan");
        let mut g = PersonalGraph::new(root_id, 1000);

        let client = g.upsert_identity(Identity::new(IdentityKind::Person, "c", "Client", 1000));
        let chooz = g.upsert_identity(Identity::new(IdentityKind::Org, "chooz", "CHOOZ", 1000));

        g.add_relationship(Relationship::new(
            client.clone(),
            chooz.clone(),
            RelKind::ClientOf,
            1000,
            1000,
        ));

        // Current: 1 client
        let chooz_id = IdentityId::new(IdentityKind::Org, "chooz");
        assert_eq!(
            g.relationships_of_kind(&chooz_id, &RelKind::ClientOf)
                .iter()
                .filter(|r| r.is_current())
                .count(),
            1
        );

        // Client leaves at time 5000
        g.end_relationship(&client, &chooz, &RelKind::ClientOf, 5000, RelStatus::Ended);

        // Current: 0 clients
        assert_eq!(
            g.relationships_of_kind(&chooz_id, &RelKind::ClientOf)
                .iter()
                .filter(|r| r.is_current())
                .count(),
            0
        );

        // But history: query at time 3000 still shows the client
        let historical = g.relationships_at(&chooz_id, 3000);
        assert_eq!(historical.len(), 1);
    }

    #[test]
    fn test_person_in_multiple_groups() {
        let root_id = IdentityId::new(IdentityKind::Person, "person");
        let mut g = PersonalGraph::new(root_id.clone(), 1000);

        let person = g.upsert_identity(Identity::new(
            IdentityKind::Person,
            "person",
            "Person",
            1000,
        ));
        let yoga = g.upsert_identity(Identity::new(
            IdentityKind::Group,
            "yoga",
            "Yoga class",
            1000,
        ));
        let book_club = g.upsert_identity(Identity::new(
            IdentityKind::Group,
            "books",
            "Book club",
            1000,
        ));
        let team = g.upsert_identity(Identity::new(
            IdentityKind::Group,
            "soccer",
            "Soccer team",
            1000,
        ));

        // Joins all 3
        for group in [&yoga, &book_club, &team] {
            g.add_relationship(Relationship::new(
                person.clone(),
                group.clone(),
                RelKind::MemberOf,
                1000,
                1000,
            ));
        }

        // Yoga class ends
        g.transition_identity(
            &yoga,
            Lifecycle::Archived,
            5000,
            Some("instructor retired".into()),
        );

        // Person leaves book club (but book club keeps going)
        g.end_relationship(
            &person,
            &book_club,
            &RelKind::MemberOf,
            6000,
            RelStatus::Ended,
        );

        // Active groups for person: just soccer
        let active = g.current_relationships(&person);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].to.local_id(), "soccer");

        // Historically participated in 3
        assert_eq!(g.relationships_for(&person).len(), 3);
    }
}
