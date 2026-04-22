//! Skills — tree 10: knowledge, procedures, and capabilities grow with need.
//!
//! Idan's insight: when the user solves a problem (writes code, regulates
//! emotion, learns a technique), the SKILL used to solve it should be a
//! first-class node in the graph. Each successful use strengthens the skill;
//! each failure weakens it. Skills become queryable ("what can I do?").
//!
//! Three skill categories:
//!   1. Technical (code patterns, algorithms, frameworks)
//!   2. Procedural (regulation strategies, planning habits, workflows)
//!   3. Conceptual (analogies, explanatory frames, mental models)
//!
//! Skills DON'T live in a special table. They're atoms of kind Concept
//! with label prefix "skill:" and linked via typed relations:
//!
//!   person → has_skill → skill_atom
//!   skill_atom → used_for → problem_atom      (what it solves)
//!   skill_atom → requires → prerequisite_atom (what you need to know first)
//!   skill_atom → improved_by_habit → practice_atom (how to strengthen)
//!
//! The proficiency is encoded in the EDGE WEIGHT of has_skill (0-100).
//! Each successful use bumps the weight; each failure decays it.

use std::collections::HashMap;

use crate::atoms::{AtomId, AtomKind, AtomStore};
use crate::relations;

/// How much to strengthen a skill on successful use.
pub const SUCCESS_BOOST: u8 = 5;
/// How much to weaken a skill on failed use.
pub const FAILURE_DECAY: u8 = 3;
/// Weight at which a skill is considered "proficient".
pub const PROFICIENT_THRESHOLD: u8 = 60;
/// Weight at which a skill is considered "mastered".
pub const MASTERED_THRESHOLD: u8 = 85;

/// Proficiency bucket based on current edge weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proficiency {
    Novice,      // 0-30
    Developing,  // 31-60
    Proficient,  // 61-85
    Mastered,    // 86-100
}

impl Proficiency {
    pub fn from_weight(w: u8) -> Self {
        match w {
            0..=30 => Self::Novice,
            31..=60 => Self::Developing,
            61..=85 => Self::Proficient,
            _ => Self::Mastered,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Novice => "novice",
            Self::Developing => "developing",
            Self::Proficient => "proficient",
            Self::Mastered => "mastered",
        }
    }
}

/// Register a new skill atom. Returns the atom_id. If a skill with the same
/// canonical label already exists, returns the existing id (dedup via content hash).
pub fn register_skill(store: &mut AtomStore, label: &str) -> AtomId {
    let data = format!("skill:{}", label).into_bytes();
    store.put(AtomKind::Concept, data)
}

/// Link a person to a skill via has_skill with an initial weight.
/// If the edge exists, this is a no-op (weights must be updated via reinforce/weaken).
pub fn attach_skill(
    store: &mut AtomStore,
    person: AtomId,
    skill: AtomId,
    initial_weight: u8,
) {
    let has_skill_rel = get_has_skill_code();
    // Check if already linked
    let existing = store.outgoing(person).iter()
        .find(|e| e.relation == has_skill_rel && e.to == skill)
        .copied();
    if existing.is_none() {
        store.link(person, skill, has_skill_rel, initial_weight, 0);
    }
}

/// Link skill → problem via used_for (the problem the skill addresses).
pub fn skill_solves(store: &mut AtomStore, skill: AtomId, problem: AtomId) {
    let used_for = relations::by_name("used_for").unwrap().code;
    store.link(skill, problem, used_for, 75, 0);
}

/// Link skill → prerequisite via requires.
pub fn skill_requires(store: &mut AtomStore, skill: AtomId, prerequisite: AtomId) {
    let requires = relations::by_name("requires").unwrap().code;
    store.link(skill, prerequisite, requires, 90, 0);
}

/// Link skill → practice habit via improved_by_habit.
pub fn skill_improved_by(store: &mut AtomStore, skill: AtomId, habit: AtomId) {
    let improved_by = relations::by_name("improved_by_habit").unwrap().code;
    store.link(skill, habit, improved_by, 70, 0);
}

/// Record successful use — boosts the proficiency edge weight.
///
/// Because AtomStore doesn't yet support edge weight mutation directly,
/// we emulate via RE-LINK semantics: add a stronger edge on top. Future
/// walks will see the higher weight dominate.
///
/// This also adds provenance: a new "use event" atom linked to both person
/// and skill, so the growth is traceable.
pub fn reinforce_skill(
    store: &mut AtomStore,
    person: AtomId,
    skill: AtomId,
    success: bool,
    timestamp: u64,
) -> u8 {
    let has_skill_rel = get_has_skill_code();

    // Compute new weight based on existing edges
    let current_weight: u8 = store.outgoing(person).iter()
        .filter(|e| e.relation == has_skill_rel && e.to == skill)
        .map(|e| e.weight)
        .max()
        .unwrap_or(0);

    let new_weight = if success {
        current_weight.saturating_add(SUCCESS_BOOST).min(100)
    } else {
        current_weight.saturating_sub(FAILURE_DECAY)
    };

    // Emit an additional edge with the new weight — the new weight is
    // what counts because our queries take max.
    store.link(person, skill, has_skill_rel, new_weight, 0);

    // Add a reinforcement event atom for audit trail
    let event_data = format!(
        "skill_event:{}:p{}:s{}:{}:w{}",
        timestamp,
        person,
        skill,
        if success { "success" } else { "failure" },
        new_weight
    ).into_bytes();
    store.put(AtomKind::Relation, event_data);

    new_weight
}

/// Query: what skills does this person have? Returns (skill_atom, weight).
pub fn skills_of(store: &AtomStore, person: AtomId) -> Vec<(AtomId, u8)> {
    let has_skill_rel = get_has_skill_code();
    let mut by_skill: HashMap<AtomId, u8> = HashMap::new();

    for edge in store.outgoing(person).iter().filter(|e| e.relation == has_skill_rel) {
        // Take max weight per skill (latest reinforcement wins)
        let current = by_skill.get(&edge.to).copied().unwrap_or(0);
        if edge.weight > current {
            by_skill.insert(edge.to, edge.weight);
        }
    }

    let mut out: Vec<(AtomId, u8)> = by_skill.into_iter().collect();
    out.sort_by(|a, b| b.1.cmp(&a.1));
    out
}

/// Query: what's the proficiency level for a specific skill?
pub fn proficiency_of(store: &AtomStore, person: AtomId, skill: AtomId) -> Option<Proficiency> {
    let has_skill_rel = get_has_skill_code();
    let weight = store.outgoing(person).iter()
        .filter(|e| e.relation == has_skill_rel && e.to == skill)
        .map(|e| e.weight)
        .max()?;
    Some(Proficiency::from_weight(weight))
}

/// Query: what skills solve this problem? Returns skills linked via used_for.
pub fn skills_for_problem(store: &AtomStore, problem: AtomId) -> Vec<AtomId> {
    let used_for = relations::by_name("used_for").unwrap().code;
    store.incoming_by_relation(problem, used_for).iter()
        .map(|e| e.from)
        .filter(|aid| {
            store.get(*aid).map(|a| {
                a.data.starts_with(b"skill:")
            }).unwrap_or(false)
        })
        .collect()
}

/// Query: which people are proficient (or better) in this skill?
pub fn people_with_skill(
    store: &AtomStore,
    skill: AtomId,
    min_proficiency: Proficiency,
) -> Vec<(AtomId, u8)> {
    let has_skill_rel = get_has_skill_code();
    let min_weight = match min_proficiency {
        Proficiency::Novice => 0,
        Proficiency::Developing => 31,
        Proficiency::Proficient => 61,
        Proficiency::Mastered => 86,
    };

    // Use incoming_by_relation to find everyone linked TO this skill via has_skill
    let edges = store.incoming_by_relation(skill, has_skill_rel);
    let mut best_per_person: HashMap<AtomId, u8> = HashMap::new();
    for e in edges {
        let cur = best_per_person.get(&e.from).copied().unwrap_or(0);
        if e.weight > cur {
            best_per_person.insert(e.from, e.weight);
        }
    }
    let mut out: Vec<(AtomId, u8)> = best_per_person.into_iter()
        .filter(|&(_, w)| w >= min_weight)
        .collect();
    out.sort_by(|a, b| b.1.cmp(&a.1));
    out
}

/// Extract the human-readable skill name from a skill atom.
pub fn skill_label(store: &AtomStore, skill: AtomId) -> Option<String> {
    let atom = store.get(skill)?;
    let text = std::str::from_utf8(&atom.data).ok()?;
    text.strip_prefix("skill:").map(|s| s.to_string())
}

/// Get the relation code for has_skill. Now first-class in the registry (0x48).
fn get_has_skill_code() -> u8 {
    relations::by_name("has_skill")
        .expect("has_skill relation must be registered (code 0x48)")
        .code
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::{AtomKind, AtomStore};

    #[test]
    fn register_and_attach_skill() {
        let mut store = AtomStore::new();
        let person = store.put(AtomKind::Concept, b"Idan".to_vec());
        let skill = register_skill(&mut store, "laravel_service_container");
        attach_skill(&mut store, person, skill, 40);

        let skills = skills_of(&store, person);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].0, skill);
        assert_eq!(skills[0].1, 40);
    }

    #[test]
    fn reinforcement_increases_weight() {
        let mut store = AtomStore::new();
        let person = store.put(AtomKind::Concept, b"Idan".to_vec());
        let skill = register_skill(&mut store, "python_asyncio");
        attach_skill(&mut store, person, skill, 50);

        let w1 = reinforce_skill(&mut store, person, skill, true, 1000);
        assert_eq!(w1, 55);
        let w2 = reinforce_skill(&mut store, person, skill, true, 2000);
        assert_eq!(w2, 60);
    }

    #[test]
    fn failure_decreases_weight() {
        let mut store = AtomStore::new();
        let person = store.put(AtomKind::Concept, b"Idan".to_vec());
        let skill = register_skill(&mut store, "rust_lifetimes");
        attach_skill(&mut store, person, skill, 50);

        let w = reinforce_skill(&mut store, person, skill, false, 1000);
        assert_eq!(w, 47);
    }

    #[test]
    fn proficiency_buckets() {
        assert_eq!(Proficiency::from_weight(0), Proficiency::Novice);
        assert_eq!(Proficiency::from_weight(30), Proficiency::Novice);
        assert_eq!(Proficiency::from_weight(31), Proficiency::Developing);
        assert_eq!(Proficiency::from_weight(60), Proficiency::Developing);
        assert_eq!(Proficiency::from_weight(61), Proficiency::Proficient);
        assert_eq!(Proficiency::from_weight(85), Proficiency::Proficient);
        assert_eq!(Proficiency::from_weight(86), Proficiency::Mastered);
        assert_eq!(Proficiency::from_weight(100), Proficiency::Mastered);
    }

    #[test]
    fn skill_solves_links_to_problem() {
        let mut store = AtomStore::new();
        let skill = register_skill(&mut store, "binary_search");
        let problem = store.put(AtomKind::Concept, b"find_item_in_sorted_list".to_vec());
        skill_solves(&mut store, skill, problem);

        let skills = skills_for_problem(&store, problem);
        assert!(skills.contains(&skill));
    }

    #[test]
    fn people_with_skill_filters_by_proficiency() {
        let mut store = AtomStore::new();
        let novice = store.put(AtomKind::Concept, b"Alice".to_vec());
        let proficient = store.put(AtomKind::Concept, b"Bob".to_vec());
        let mastered = store.put(AtomKind::Concept, b"Carol".to_vec());
        let skill = register_skill(&mut store, "sql_window_functions");

        attach_skill(&mut store, novice, skill, 20);
        attach_skill(&mut store, proficient, skill, 70);
        attach_skill(&mut store, mastered, skill, 95);

        let proficient_or_better = people_with_skill(&store, skill, Proficiency::Proficient);
        let ids: Vec<AtomId> = proficient_or_better.iter().map(|(a, _)| *a).collect();
        assert!(ids.contains(&proficient));
        assert!(ids.contains(&mastered));
        assert!(!ids.contains(&novice));
    }

    #[test]
    fn skill_label_roundtrip() {
        let mut store = AtomStore::new();
        let skill = register_skill(&mut store, "mindfulness_breathing");
        assert_eq!(skill_label(&store, skill), Some("mindfulness_breathing".to_string()));
    }

    #[test]
    fn proficiency_of_returns_bucket() {
        let mut store = AtomStore::new();
        let person = store.put(AtomKind::Concept, b"Idan".to_vec());
        let skill = register_skill(&mut store, "graph_algorithms");
        attach_skill(&mut store, person, skill, 75);

        let prof = proficiency_of(&store, person, skill);
        assert_eq!(prof, Some(Proficiency::Proficient));
    }

    #[test]
    fn weight_caps_at_100() {
        let mut store = AtomStore::new();
        let person = store.put(AtomKind::Concept, b"Idan".to_vec());
        let skill = register_skill(&mut store, "unicode_handling");
        attach_skill(&mut store, person, skill, 98);

        let w = reinforce_skill(&mut store, person, skill, true, 1000);
        assert_eq!(w, 100);
        // Another success — still capped
        let w2 = reinforce_skill(&mut store, person, skill, true, 2000);
        assert_eq!(w2, 100);
    }

    #[test]
    fn weight_floors_at_zero() {
        let mut store = AtomStore::new();
        let person = store.put(AtomKind::Concept, b"Idan".to_vec());
        let skill = register_skill(&mut store, "punch_card_programming");
        attach_skill(&mut store, person, skill, 2);

        let w = reinforce_skill(&mut store, person, skill, false, 1000);
        assert_eq!(w, 0);
    }

    #[test]
    fn register_skill_is_idempotent() {
        let mut store = AtomStore::new();
        let s1 = register_skill(&mut store, "react_hooks");
        let s2 = register_skill(&mut store, "react_hooks");
        // Same label → content-hash dedup → same atom_id
        assert_eq!(s1, s2);
    }
}
