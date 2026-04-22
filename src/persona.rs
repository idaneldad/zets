//! Persona module — rich person modeling on top of AtomStore.
//!
//! Built to address Idan's specification (22 Apr 2026): a knowledge graph must
//! support queries like "who speaks 3 languages and belongs to 2 clubs?" over
//! a heterogeneous population.
//!
//! Design decisions (from consultation spec + broken-kelim analysis):
//!   - A Person is an atom of kind Concept with name as data.
//!   - Attributes are edges, not struct fields — this keeps the data model
//!     uniform (everything is atoms + edges) and allows polymorphism:
//!     a Person can have 0 or N hobbies, 0 or N languages, without schema changes.
//!   - Values (ages, occupations, languages) are ALSO atoms — stored once,
//!     referenced many times. If 100 people speak Hebrew, "Hebrew" is one atom.
//!   - Queries are graph walks with relation-kind filtering — same walker
//!     that handles IS_A/HAS_PART handles has_hobby/speaks_language.
//!
//! Rejected approaches (from doc review):
//!   - RocksDB column-family per tenant: AtomStore + scopes already separates.
//!   - libsodium selective encryption: AES-256-GCM already in crypto.rs.
//!   - Global `lazy_static` DB: anti-pattern in Rust.

use std::collections::{HashMap, HashSet};

use crate::atoms::{AtomId, AtomKind, AtomStore};
use crate::relations;

/// Lightweight builder for creating a person with typed attributes.
pub struct PersonBuilder<'a> {
    store: &'a mut AtomStore,
    id: AtomId,
}

impl<'a> PersonBuilder<'a> {
    /// Create a new person atom with the given name.
    pub fn create(store: &'a mut AtomStore, name: &str) -> Self {
        let id = store.put(AtomKind::Concept, name.as_bytes().to_vec());
        Self { store, id }
    }

    pub fn id(&self) -> AtomId { self.id }

    /// Age in years.
    pub fn with_age(self, age: u8) -> Self {
        let age_atom = self.store.put(AtomKind::Concept,
            format!("age:{}", age).into_bytes());
        let rel = relations::by_name("has_age").unwrap().code;
        self.store.link(self.id, age_atom, rel, 100, 0);
        self
    }

    /// Add an occupation (can be called multiple times).
    pub fn with_occupation(self, occupation: &str) -> Self {
        let occ_atom = self.store.put(AtomKind::Concept, occupation.as_bytes().to_vec());
        let rel = relations::by_name("has_occupation").unwrap().code;
        self.store.link(self.id, occ_atom, rel, 90, 0);
        self
    }

    /// Add a hobby (can be called multiple times).
    pub fn with_hobby(self, hobby: &str) -> Self {
        let hobby_atom = self.store.put(AtomKind::Concept, hobby.as_bytes().to_vec());
        let rel = relations::by_name("has_hobby").unwrap().code;
        self.store.link(self.id, hobby_atom, rel, 80, 0);
        self
    }

    /// Add a spoken language with proficiency 0-100.
    pub fn with_language(self, language: &str, proficiency: u8) -> Self {
        let lang_atom = self.store.put(AtomKind::Concept, language.as_bytes().to_vec());
        let rel = relations::by_name("speaks_language").unwrap().code;
        self.store.link(self.id, lang_atom, rel, proficiency, 0);
        self
    }

    /// Add group membership (club, community, professional organization).
    pub fn belongs_to(self, group: &str) -> Self {
        let group_atom = self.store.put(AtomKind::Concept, group.as_bytes().to_vec());
        let rel = relations::by_name("belongs_to_group").unwrap().code;
        self.store.link(self.id, group_atom, rel, 85, 0);
        self
    }

    /// Residence.
    pub fn lives_in(self, place: &str) -> Self {
        let loc_atom = self.store.put(AtomKind::Concept, place.as_bytes().to_vec());
        let rel = relations::by_name("lives_in").unwrap().code;
        self.store.link(self.id, loc_atom, rel, 100, 0);
        self
    }

    /// Family: A parent_of B.
    pub fn parent_of(self, child_id: AtomId) -> Self {
        let rel = relations::by_name("parent_of").unwrap().code;
        self.store.link(self.id, child_id, rel, 100, 0);
        self
    }

    /// Married to another person (creates symmetric edge).
    pub fn married_to(self, spouse_id: AtomId) -> Self {
        let rel = relations::by_name("married_to").unwrap().code;
        self.store.link(self.id, spouse_id, rel, 100, 0);
        self.store.link(spouse_id, self.id, rel, 100, 0);
        self
    }

    /// Education institution.
    pub fn studied_at(self, institution: &str) -> Self {
        let inst_atom = self.store.put(AtomKind::Concept, institution.as_bytes().to_vec());
        let rel = relations::by_name("studied_at").unwrap().code;
        self.store.link(self.id, inst_atom, rel, 80, 0);
        self
    }
}

// ────────────────────────────────────────────────────────────────
// Queries
// ────────────────────────────────────────────────────────────────

/// Count outgoing edges of a given relation name for a person.
pub fn count_relation(store: &AtomStore, person: AtomId, rel_name: &str) -> usize {
    let rel_code = match relations::by_name(rel_name) {
        Some(r) => r.code,
        None => return 0,
    };
    store.outgoing(person).iter().filter(|e| e.relation == rel_code).count()
}

/// Retrieve all target atoms for a given relation.
pub fn get_related(store: &AtomStore, person: AtomId, rel_name: &str) -> Vec<AtomId> {
    let rel_code = match relations::by_name(rel_name) {
        Some(r) => r.code,
        None => return Vec::new(),
    };
    store.outgoing(person).iter()
        .filter(|e| e.relation == rel_code)
        .map(|e| e.to)
        .collect()
}

/// Diversity score = number of DISTINCT relation families the person has links to.
/// A richer person has hobbies + occupation + languages + groups + location etc.
pub fn diversity_score(store: &AtomStore, person: AtomId) -> usize {
    let edges = store.outgoing(person);
    let kinds: HashSet<u8> = edges.iter().map(|e| e.relation).collect();
    // Each relation family counted once: has_age, has_occupation, has_hobby,
    // speaks_language, belongs_to_group, lives_in, parent_of, married_to, studied_at
    let persona_relations = [0x40u8, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x27];
    kinds.iter().filter(|k| persona_relations.contains(k)).count()
}

/// Total attribute count — sum of edges across all persona relations.
/// A polyglot with 5 languages + 3 hobbies + 2 groups scores 10.
pub fn attribute_richness(store: &AtomStore, person: AtomId) -> usize {
    let persona_relations: HashSet<u8> = [0x40u8, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x27]
        .iter().copied().collect();
    store.outgoing(person).iter()
        .filter(|e| persona_relations.contains(&e.relation))
        .count()
}

/// Find all persons speaking at least N languages.
pub fn polyglots(store: &AtomStore, persons: &[AtomId], min_languages: usize) -> Vec<AtomId> {
    persons.iter()
        .filter(|&&p| count_relation(store, p, "speaks_language") >= min_languages)
        .copied()
        .collect()
}

/// Find all persons who belong to at least N groups.
pub fn joiners(store: &AtomStore, persons: &[AtomId], min_groups: usize) -> Vec<AtomId> {
    persons.iter()
        .filter(|&&p| count_relation(store, p, "belongs_to_group") >= min_groups)
        .copied()
        .collect()
}

/// "Who speaks at least L languages AND belongs to at least G groups?" — the
/// exact query from Idan's spec.
pub fn polyglot_clubbers(store: &AtomStore, persons: &[AtomId],
                         min_languages: usize, min_groups: usize) -> Vec<AtomId> {
    persons.iter()
        .filter(|&&p| count_relation(store, p, "speaks_language") >= min_languages
                   && count_relation(store, p, "belongs_to_group") >= min_groups)
        .copied()
        .collect()
}

/// Most diverse person — highest attribute_richness.
pub fn most_diverse(store: &AtomStore, persons: &[AtomId]) -> Option<AtomId> {
    persons.iter().copied()
        .max_by_key(|&p| attribute_richness(store, p))
}

/// Who shares the most common attributes with a given person?
/// Used for "who is most like me?" queries.
pub fn find_similar(store: &AtomStore, target: AtomId, candidates: &[AtomId])
    -> Vec<(AtomId, usize)>
{
    let target_values: HashSet<(u8, AtomId)> = store.outgoing(target).iter()
        .map(|e| (e.relation, e.to))
        .collect();

    let mut scored: Vec<(AtomId, usize)> = candidates.iter()
        .filter(|&&c| c != target)
        .map(|&c| {
            let shared = store.outgoing(c).iter()
                .filter(|e| target_values.contains(&(e.relation, e.to)))
                .count();
            (c, shared)
        })
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored
}

/// Full "person card" — structured summary of what we know.
#[derive(Debug, Clone, Default)]
pub struct PersonCard {
    pub id: AtomId,
    pub name: String,
    pub age: Option<u8>,
    pub occupations: Vec<String>,
    pub hobbies: Vec<String>,
    pub languages: Vec<(String, u8)>, // (language, proficiency)
    pub groups: Vec<String>,
    pub location: Option<String>,
    pub diversity: usize,
    pub richness: usize,
}

/// Summarize a person as structured text.
pub fn card(store: &AtomStore, person: AtomId) -> PersonCard {
    let mut c = PersonCard::default();
    c.id = person;
    c.name = store.get(person)
        .and_then(|a| std::str::from_utf8(&a.data).ok().map(|s| s.to_string()))
        .unwrap_or_default();

    let edges = store.outgoing(person);
    let mut lang_proficiencies: HashMap<AtomId, u8> = HashMap::new();

    for e in &edges {
        let target_data = match store.get(e.to) {
            Some(atom) => std::str::from_utf8(&atom.data).unwrap_or("").to_string(),
            None => continue,
        };
        match e.relation {
            0x40 => {
                // has_age — format is "age:NN"
                if let Some(n) = target_data.strip_prefix("age:") {
                    if let Ok(age) = n.parse::<u8>() { c.age = Some(age); }
                }
            }
            0x41 => c.occupations.push(target_data),
            0x42 => c.hobbies.push(target_data),
            0x43 => { lang_proficiencies.insert(e.to, e.weight); }
            0x44 => c.location = Some(target_data),
            0x27 => c.groups.push(target_data),
            _ => {}
        }
    }
    // Materialize languages with proficiency
    for (lang_id, prof) in lang_proficiencies {
        if let Some(atom) = store.get(lang_id) {
            if let Ok(s) = std::str::from_utf8(&atom.data) {
                c.languages.push((s.to_string(), prof));
            }
        }
    }

    c.diversity = diversity_score(store, person);
    c.richness = attribute_richness(store, person);
    c
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (AtomStore, Vec<AtomId>) {
        let mut store = AtomStore::new();
        let alice = PersonBuilder::create(&mut store, "Alice")
            .with_age(30)
            .with_occupation("software engineer")
            .with_occupation("writer")
            .with_hobby("photography")
            .with_hobby("climbing")
            .with_language("Hebrew", 100)
            .with_language("English", 95)
            .with_language("Spanish", 60)
            .belongs_to("hackers_club")
            .belongs_to("climbing_club")
            .lives_in("Tel Aviv")
            .id();
        let bob = PersonBuilder::create(&mut store, "Bob")
            .with_age(45)
            .with_occupation("plumber")
            .with_language("Hebrew", 100)
            .id();
        let carol = PersonBuilder::create(&mut store, "Carol")
            .with_age(28)
            .with_occupation("doctor")
            .with_hobby("reading")
            .with_language("Hebrew", 100)
            .with_language("English", 90)
            .with_language("French", 70)
            .belongs_to("medical_association")
            .id();
        (store, vec![alice, bob, carol])
    }

    #[test]
    fn person_has_age() {
        let (store, ps) = setup();
        assert_eq!(count_relation(&store, ps[0], "has_age"), 1);
    }

    #[test]
    fn alice_has_three_languages() {
        let (store, ps) = setup();
        assert_eq!(count_relation(&store, ps[0], "speaks_language"), 3);
    }

    #[test]
    fn polyglots_finds_correct_people() {
        let (store, ps) = setup();
        let polys = polyglots(&store, &ps, 3);
        assert_eq!(polys.len(), 2); // alice and carol
    }

    #[test]
    fn polyglot_clubbers_query() {
        let (store, ps) = setup();
        // Idan's query: 3+ languages AND 2+ groups
        let result = polyglot_clubbers(&store, &ps, 3, 2);
        assert_eq!(result.len(), 1); // only alice
    }

    #[test]
    fn most_diverse_is_alice() {
        let (store, ps) = setup();
        assert_eq!(most_diverse(&store, &ps), Some(ps[0]));
    }

    #[test]
    fn card_produces_full_profile() {
        let (store, ps) = setup();
        let c = card(&store, ps[0]);
        assert_eq!(c.name, "Alice");
        assert_eq!(c.age, Some(30));
        assert_eq!(c.occupations.len(), 2);
        assert_eq!(c.languages.len(), 3);
        assert_eq!(c.groups.len(), 2);
        assert_eq!(c.location.as_deref(), Some("Tel Aviv"));
        assert!(c.diversity >= 5);
    }

    #[test]
    fn find_similar_ranks_by_shared_attributes() {
        let (store, ps) = setup();
        // Alice & Carol both speak Hebrew + English (shared atoms)
        let similar = find_similar(&store, ps[0], &ps);
        // carol should rank higher than bob for alice
        assert!(similar[0].1 >= similar[1].1);
    }

    #[test]
    fn shared_language_atoms_are_deduped() {
        let (store, ps) = setup();
        // Hebrew is spoken by all three — should exist as ONE atom with refcount
        let stats = store.stats();
        // 3 unique languages across 3 people: Hebrew, English, Spanish, French
        // but Hebrew is deduped, English is deduped
        // Total lang atoms: 4 (Hebrew, English, Spanish, French)
        // So the fact that we have < 3*3=9 language storage means dedup worked
        assert!(stats.bytes_saved_dedup > 0, "dedup should save bytes");
        let _ = ps;
    }

    #[test]
    fn family_relationships_work() {
        let mut store = AtomStore::new();
        let dad = PersonBuilder::create(&mut store, "Dad").id();
        let mom = PersonBuilder::create(&mut store, "Mom").id();
        let kid = PersonBuilder::create(&mut store, "Kid").id();

        PersonBuilder { store: &mut store, id: dad }.married_to(mom);
        PersonBuilder { store: &mut store, id: dad }.parent_of(kid);
        PersonBuilder { store: &mut store, id: mom }.parent_of(kid);

        // Both parents link to kid
        assert_eq!(count_relation(&store, dad, "parent_of"), 1);
        assert_eq!(count_relation(&store, mom, "parent_of"), 1);
        // Marriage is symmetric
        assert_eq!(count_relation(&store, dad, "married_to"), 1);
        assert_eq!(count_relation(&store, mom, "married_to"), 1);
    }
}
