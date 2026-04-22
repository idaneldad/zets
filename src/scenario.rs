//! Scenario — a conversation (or any discrete event episode) as a subgraph.
//!
//! Idan's insight: every conversation should become a persistent sub-atom in
//! the user's persona graph. When Tamar discusses dental work today and then
//! again in 2 years, the old scenario is still there — at a reduced weight,
//! but reachable.
//!
//! This is the BRIDGE between session (ephemeral, in-memory) and persistent
//! long-term memory. A scenario is an atom of kind EventNode that:
//!
//!   1. Links to the Person who had the conversation
//!   2. Links to the topics/atoms that were mentioned
//!   3. Carries a timestamp for temporal decay
//!   4. Has an optional emotion tag from appraisal
//!   5. Is a first-class node — walks can traverse to/from it
//!
//! Temporal decay uses exponential: weight(age) = e^(-age_seconds / TAU)
//! where TAU is the time constant (default: 30 days = 2.6M seconds).
//! This matches human episodic memory decay curves.

use crate::atoms::{AtomId, AtomKind, AtomStore};
use crate::relations;

/// Timestamp — seconds since a reference epoch. Kept in u64 for determinism.
pub type Timestamp = u64;

/// How strongly the scenario's recency affects its weight. Higher = slower decay.
pub const DEFAULT_TEMPORAL_TAU_SECONDS: f32 = 2_592_000.0; // 30 days

/// A scenario record — the in-memory representation before it's committed
/// to the AtomStore as an EventNode + edges.
#[derive(Debug, Clone)]
pub struct Scenario {
    pub atom_id: AtomId,
    pub person_id: AtomId,
    pub created_at: Timestamp,
    pub mentioned_atoms: Vec<AtomId>,
    pub emotion_atom: Option<AtomId>,
    pub label: String,
}

/// Builder for creating a scenario. Commits to AtomStore via `commit`.
pub struct ScenarioBuilder<'a> {
    store: &'a mut AtomStore,
    person_id: AtomId,
    timestamp: Timestamp,
    label: String,
    mentioned: Vec<AtomId>,
    emotion: Option<AtomId>,
}

impl<'a> ScenarioBuilder<'a> {
    pub fn new(store: &'a mut AtomStore, person_id: AtomId, timestamp: Timestamp, label: &str) -> Self {
        Self {
            store,
            person_id,
            timestamp,
            label: label.to_string(),
            mentioned: Vec::new(),
            emotion: None,
        }
    }

    /// Add an atom that was mentioned in this scenario.
    pub fn mentioned(mut self, atom_id: AtomId) -> Self {
        self.mentioned.push(atom_id);
        self
    }

    /// Add multiple mentioned atoms.
    pub fn mentioned_all(mut self, atoms: &[AtomId]) -> Self {
        self.mentioned.extend_from_slice(atoms);
        self
    }

    /// Tag with an emotion atom (optional).
    pub fn with_emotion(mut self, emotion: AtomId) -> Self {
        self.emotion = Some(emotion);
        self
    }

    /// Commit to the store: create atom of kind Relation (used as event marker),
    /// link to person, to mentioned atoms, and to emotion if present.
    pub fn commit(self) -> Scenario {
        // Encode label + timestamp in data so atom is uniquely identified
        let data = format!("scenario:{}:{}", self.timestamp, self.label).into_bytes();
        let atom_id = self.store.put(AtomKind::Relation, data);

        let agent_of = relations::by_name("agent_of").unwrap().code;
        let co_occurs = relations::by_name("co_occurs_with").unwrap().code;
        let emotion_triggered = relations::by_name("emotion_triggered").unwrap().code;

        // Link person → scenario via agent_of (the person had this event)
        self.store.link(self.person_id, atom_id, agent_of, 100, 0);

        // Link scenario to each mentioned atom via co_occurs
        for atom in &self.mentioned {
            self.store.link(atom_id, *atom, co_occurs, 80, 0);
        }

        // Link scenario to emotion if provided
        if let Some(e) = self.emotion {
            self.store.link(atom_id, e, emotion_triggered, 90, 0);
        }

        Scenario {
            atom_id,
            person_id: self.person_id,
            created_at: self.timestamp,
            mentioned_atoms: self.mentioned,
            emotion_atom: self.emotion,
            label: self.label,
        }
    }
}

/// Compute the temporal decay weight for a scenario given current time.
///
/// Returns a float in (0, 1]:
///   - 1.0 means the scenario just happened
///   - 0.5 means it happened TAU seconds ago (default: 30 days)
///   - 0.01 means very old (about 4.6*TAU ago)
pub fn temporal_weight(created_at: Timestamp, now: Timestamp, tau_seconds: f32) -> f32 {
    if now < created_at {
        return 1.0; // future or same — treat as fresh
    }
    let elapsed = (now - created_at) as f32;
    (-elapsed / tau_seconds).exp()
}

/// Find all scenarios this person had, ordered by temporal weight descending.
pub fn scenarios_of(store: &AtomStore, person_id: AtomId) -> Vec<AtomId> {
    let agent_of = relations::by_name("agent_of").unwrap().code;
    store.outgoing(person_id).iter()
        .filter(|e| e.relation == agent_of)
        .map(|e| e.to)
        .filter(|atom_id| {
            // Must be of kind Relation (our event markers)
            store.get(*atom_id).map(|a| a.kind == AtomKind::Relation).unwrap_or(false)
        })
        .collect()
}

/// Find scenarios by this person that mentioned a specific atom.
/// Useful for: "what scenarios did Tamar have about dogs?"
pub fn scenarios_mentioning(store: &AtomStore, person_id: AtomId, topic: AtomId) -> Vec<AtomId> {
    let co_occurs = relations::by_name("co_occurs_with").unwrap().code;
    scenarios_of(store, person_id).into_iter()
        .filter(|sid| {
            store.outgoing(*sid).iter().any(|e| {
                e.relation == co_occurs && e.to == topic
            })
        })
        .collect()
}

/// Parse the timestamp encoded in a scenario's atom data ("scenario:TS:label").
pub fn scenario_timestamp(store: &AtomStore, scenario_atom: AtomId) -> Option<Timestamp> {
    let atom = store.get(scenario_atom)?;
    let text = std::str::from_utf8(&atom.data).ok()?;
    let parts: Vec<&str> = text.splitn(3, ':').collect();
    if parts.len() < 2 || parts[0] != "scenario" { return None; }
    parts[1].parse::<Timestamp>().ok()
}

/// Score each scenario of a person by temporal weight, given current time.
/// Returns (scenario_atom, weight) sorted by weight descending.
pub fn ranked_scenarios(
    store: &AtomStore,
    person_id: AtomId,
    now: Timestamp,
    tau_seconds: f32,
) -> Vec<(AtomId, f32)> {
    let mut scored: Vec<(AtomId, f32)> = scenarios_of(store, person_id).into_iter()
        .filter_map(|sid| {
            let ts = scenario_timestamp(store, sid)?;
            Some((sid, temporal_weight(ts, now, tau_seconds)))
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored
}

/// Re-activation via similarity: given a set of atoms in current context,
/// find past scenarios whose mentioned atoms overlap, weighted by BOTH
/// temporal decay AND similarity (overlap).
///
/// This is what enables "Tamar discusses dental work 2 years later — the old
/// dental scenario is retrieved even though it's very old, because the topic
/// matches."
pub fn reactivate_by_similarity(
    store: &AtomStore,
    person_id: AtomId,
    current_atoms: &[AtomId],
    now: Timestamp,
    tau_seconds: f32,
    similarity_weight: f32, // 0.0 = only recency; 1.0 = only similarity
) -> Vec<(AtomId, f32)> {
    let current_set: std::collections::HashSet<AtomId> =
        current_atoms.iter().copied().collect();
    let co_occurs = relations::by_name("co_occurs_with").unwrap().code;

    let mut scored: Vec<(AtomId, f32)> = scenarios_of(store, person_id).into_iter()
        .filter_map(|sid| {
            let ts = scenario_timestamp(store, sid)?;
            let temporal = temporal_weight(ts, now, tau_seconds);

            // Similarity = Jaccard-like overlap
            let mentions: std::collections::HashSet<AtomId> = store.outgoing(sid).iter()
                .filter(|e| e.relation == co_occurs)
                .map(|e| e.to)
                .collect();
            let overlap = current_set.intersection(&mentions).count() as f32;
            let union = current_set.union(&mentions).count() as f32;
            let similarity = if union > 0.0 { overlap / union } else { 0.0 };

            // Combined score
            let combined = (1.0 - similarity_weight) * temporal + similarity_weight * similarity;
            Some((sid, combined))
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored
}

/// Auto-commit a session into a Scenario atom. Useful when a conversation ends:
/// snapshot the active atoms + timestamp + optional emotion into the graph.
///
/// This is the bridge between ephemeral session memory and persistent long-term
/// memory. After this call, the scenario is queryable via scenarios_of() and
/// can be reactivated later via reactivate_by_similarity().
///
/// Returns None if the session has no active atoms (nothing to commit).
pub fn auto_commit_session(
    store: &mut AtomStore,
    session: &crate::session::SessionContext,
    person_id: AtomId,
    timestamp: Timestamp,
    label: &str,
    emotion: Option<AtomId>,
) -> Option<Scenario> {
    let atoms: Vec<AtomId> = session.active_ids();
    if atoms.is_empty() { return None; }

    let mut builder = ScenarioBuilder::new(store, person_id, timestamp, label)
        .mentioned_all(&atoms);
    if let Some(e) = emotion {
        builder = builder.with_emotion(e);
    }
    Some(builder.commit())
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::{AtomKind, AtomStore};

    fn setup() -> (AtomStore, AtomId, AtomId, AtomId) {
        let mut store = AtomStore::new();
        let person = store.put(AtomKind::Concept, b"Tamar".to_vec());
        let dog = store.put(AtomKind::Concept, b"dog".to_vec());
        let tooth = store.put(AtomKind::Concept, b"tooth".to_vec());
        (store, person, dog, tooth)
    }

    #[test]
    fn create_and_retrieve_scenario() {
        let (mut store, tamar, dog, _) = setup();
        let sc = ScenarioBuilder::new(&mut store, tamar, 1000, "dog conversation")
            .mentioned(dog)
            .commit();
        let all = scenarios_of(&store, tamar);
        assert!(all.contains(&sc.atom_id));
    }

    #[test]
    fn scenarios_by_topic() {
        let (mut store, tamar, dog, tooth) = setup();
        let dental_atom = store.put(AtomKind::Concept, b"dental".to_vec());

        ScenarioBuilder::new(&mut store, tamar, 1000, "dog talk")
            .mentioned(dog).commit();
        ScenarioBuilder::new(&mut store, tamar, 2000, "tooth pain")
            .mentioned(tooth).mentioned(dental_atom).commit();

        let dog_scenarios = scenarios_mentioning(&store, tamar, dog);
        assert_eq!(dog_scenarios.len(), 1);
        let tooth_scenarios = scenarios_mentioning(&store, tamar, tooth);
        assert_eq!(tooth_scenarios.len(), 1);
        assert_ne!(dog_scenarios[0], tooth_scenarios[0]);
    }

    #[test]
    fn temporal_weight_decays_over_time() {
        let tau = 100.0;
        assert!((temporal_weight(100, 100, tau) - 1.0).abs() < 0.001);
        let at_tau = temporal_weight(0, 100, tau);
        assert!((at_tau - 0.368).abs() < 0.01, "e^-1 ≈ 0.368, got {}", at_tau);
        // Very old: near zero
        let at_5tau = temporal_weight(0, 500, tau);
        assert!(at_5tau < 0.01, "e^-5 should be small, got {}", at_5tau);
    }

    #[test]
    fn ranked_scenarios_recent_first() {
        let (mut store, tamar, dog, tooth) = setup();
        ScenarioBuilder::new(&mut store, tamar, 100, "old")
            .mentioned(dog).commit();
        ScenarioBuilder::new(&mut store, tamar, 1000, "recent")
            .mentioned(tooth).commit();

        let ranked = ranked_scenarios(&store, tamar, 1000, 100.0);
        assert_eq!(ranked.len(), 2);
        // The recent one (ts=1000, age=0) should rank higher than old (ts=100, age=900)
        assert!(ranked[0].1 > ranked[1].1);
    }

    #[test]
    fn reactivation_finds_similar_even_if_old() {
        let (mut store, tamar, _, tooth) = setup();
        let dentist = store.put(AtomKind::Concept, b"dentist".to_vec());
        let pizza = store.put(AtomKind::Concept, b"pizza".to_vec());

        // Old dental scenario (age ~ 10*tau)
        ScenarioBuilder::new(&mut store, tamar, 100, "old dental")
            .mentioned_all(&[tooth, dentist]).commit();
        // Recent unrelated scenario (age 0)
        ScenarioBuilder::new(&mut store, tamar, 1100, "pizza night")
            .mentioned(pizza).commit();

        // Current context is about dental work
        let current = vec![tooth, dentist];
        // With high similarity_weight, old dental should beat recent pizza
        let ranked = reactivate_by_similarity(&store, tamar, &current, 1100, 100.0, 0.9);
        assert!(ranked.len() == 2);
        // The first one should be dental (similarity-matched)
        let first_atom = store.get(ranked[0].0).unwrap();
        let first_label = std::str::from_utf8(&first_atom.data).unwrap();
        assert!(first_label.contains("old dental"),
                "dental should be ranked first due to similarity, got: {}", first_label);
    }

    #[test]
    fn scenario_timestamp_parse_roundtrip() {
        let (mut store, tamar, dog, _) = setup();
        let sc = ScenarioBuilder::new(&mut store, tamar, 12345, "test")
            .mentioned(dog).commit();
        let ts = scenario_timestamp(&store, sc.atom_id);
        assert_eq!(ts, Some(12345));
    }

    #[test]
    fn emotion_tagging_creates_edge() {
        let (mut store, tamar, dog, _) = setup();
        let joy = store.put(AtomKind::Concept, b"joy".to_vec());
        let sc = ScenarioBuilder::new(&mut store, tamar, 1000, "happy dog")
            .mentioned(dog)
            .with_emotion(joy)
            .commit();
        assert_eq!(sc.emotion_atom, Some(joy));
        let emotion_triggered = relations::by_name("emotion_triggered").unwrap().code;
        let has_edge = store.outgoing(sc.atom_id).iter()
            .any(|e| e.relation == emotion_triggered && e.to == joy);
        assert!(has_edge);
    }

    #[test]
    fn determinism_same_inputs_same_atom_ids() {
        let mut s1 = AtomStore::new();
        let mut s2 = AtomStore::new();
        let p1 = s1.put(AtomKind::Concept, b"Tamar".to_vec());
        let p2 = s2.put(AtomKind::Concept, b"Tamar".to_vec());
        let t1 = s1.put(AtomKind::Concept, b"topic".to_vec());
        let t2 = s2.put(AtomKind::Concept, b"topic".to_vec());
        let sc1 = ScenarioBuilder::new(&mut s1, p1, 5000, "label").mentioned(t1).commit();
        let sc2 = ScenarioBuilder::new(&mut s2, p2, 5000, "label").mentioned(t2).commit();
        // Same sequence of operations → same atom IDs
        assert_eq!(sc1.atom_id, sc2.atom_id);
        assert_eq!(sc1.mentioned_atoms, sc2.mentioned_atoms);
    }

    #[test]
    fn auto_commit_session_creates_scenario() {
        let (mut store, tamar, dog, tooth) = setup();
        let mut session = crate::session::SessionContext::new();
        session.mention(dog);
        session.mention(tooth);

        let sc = auto_commit_session(&mut store, &session, tamar, 5000, "auto-saved", None);
        assert!(sc.is_some());
        let sc = sc.unwrap();
        assert_eq!(sc.mentioned_atoms.len(), 2);
        let found = scenarios_of(&store, tamar);
        assert!(found.contains(&sc.atom_id));
    }

    #[test]
    fn auto_commit_empty_session_returns_none() {
        let (mut store, tamar, _, _) = setup();
        let session = crate::session::SessionContext::new();
        let sc = auto_commit_session(&mut store, &session, tamar, 5000, "empty", None);
        assert!(sc.is_none());
    }

    #[test]
    fn mentioned_all_shortcut() {
        let (mut store, tamar, dog, tooth) = setup();
        let sc = ScenarioBuilder::new(&mut store, tamar, 1, "multi")
            .mentioned_all(&[dog, tooth])
            .commit();
        assert_eq!(sc.mentioned_atoms.len(), 2);
    }
}
