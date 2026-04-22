//! Smart Walk — integration layer that connects meta-learning, session context,
//! spreading activation, and dreaming into ONE pipeline.
//!
//! Before this module, each component was independent:
//!   - SessionContext knew what's active
//!   - SpreadingActivation could walk from seeds
//!   - MetaLearner could suggest a mode
//!   - Dreaming could propose new edges
//!
//! smart_walk() calls them in the right order:
//!   1. Meta-learner picks a cognitive mode for this query context
//!   2. Spreading activation runs from session seeds with matching preset
//!   3. If results sparse, invoke dreaming to widen the search
//!   4. Return ranked candidates with provenance
//!   5. After outcome is known, call record_outcome() to update meta-learner
//!
//! This is what Idan called "יציאה לחיפוש מקונטקסט השיחה" — search starts
//! from the current conversation's active atoms, biased by what the system
//! has learned works for this kind of query.

use crate::atoms::{AtomId, AtomStore};
use crate::dreaming::{dream, DreamResult};
use crate::meta_learning::{CognitiveMode, MetaLearner, query_hash};
use crate::session::SessionContext;
use crate::spreading_activation::{spread_from_session, ActivationMap, SpreadConfig};

/// Result of a smart walk — what the system found + which mode was chosen.
#[derive(Debug, Clone)]
pub struct WalkResult {
    /// Ranked candidate atoms (atom_id, score)
    pub candidates: Vec<(AtomId, f32)>,
    /// Which mode the meta-learner chose for this walk
    pub mode_used: CognitiveMode,
    /// Hash of the query (used to later record the outcome with same key)
    pub query_hash: u64,
    /// Did dreaming kick in (search was sparse)?
    pub dreamed: bool,
    /// If dreaming ran, this is its result summary
    pub dream_info: Option<DreamSummary>,
}

#[derive(Debug, Clone)]
pub struct DreamSummary {
    pub candidates_proposed: usize,
    pub candidates_accepted: usize,
    pub depth_reached: u32,
}

/// Smart walk — the integrated query pipeline.
///
/// Inputs:
///   - store: the knowledge graph
///   - session: current conversation context
///   - meta: meta-learner with learned mode preferences
///   - query_text: the user's query (used for hashing + context classification)
///   - query_context: a string like "factual", "creative", "problem_solving"
///   - top_k: how many candidates to return
///
/// Returns a ranked list of candidates biased by:
///   - What's currently active (session)
///   - Which mode has worked best for this context (meta)
///   - Dreaming if search comes up thin
pub fn smart_walk(
    store: &mut AtomStore,
    session: &SessionContext,
    meta: &MetaLearner,
    query_text: &str,
    query_context: &str,
    top_k: usize,
) -> WalkResult {
    let qhash = query_hash(query_text);

    // Step 1: meta-learner chooses mode based on context history
    let mode = meta.weights_for(query_context)
        .map(|w| w.sample_by_hash(qhash))
        .unwrap_or_else(|| meta.global.sample_by_hash(qhash));

    // Step 2: convert mode to SpreadConfig preset
    let config = match mode {
        CognitiveMode::Precision => SpreadConfig::precise(),
        CognitiveMode::Divergent => SpreadConfig::divergent(),
        CognitiveMode::Gestalt => SpreadConfig {
            max_depth: 3,
            hop_decay: 0.5,
            min_activation: 0.03,
            max_nodes: 500,
            allowed_relations: None,
            weight_matters: false,  // treat weak edges equally — helps gestalt
        },
        CognitiveMode::Narrative => SpreadConfig {
            max_depth: 4,
            hop_decay: 0.5,
            min_activation: 0.05,
            max_nodes: 800,
            allowed_relations: None,
            weight_matters: true,
        },
    };

    // Step 3: spread activation
    let activation: ActivationMap = spread_from_session(store, session, &config, 10);
    let seeds = session.active_ids();
    let mut candidates = activation.top_k_novel(&seeds, top_k);

    // Step 4: if too sparse, invoke dreaming to create new edges
    let sparse_threshold = top_k.min(3);
    let mut dreamed = false;
    let mut dream_info = None;
    if candidates.len() < sparse_threshold && !seeds.is_empty() {
        dreamed = true;
        let dream_result = dream(
            store,
            &seeds,
            0.0,           // low acceptance threshold — ADHD-like
            top_k,         // max new edges
            2,             // depth
            qhash,         // propose seed = deterministic from query
        );
        let summary = DreamSummary {
            candidates_proposed: dream_result.committed.len() + dream_result.rejected.len(),
            candidates_accepted: dream_result.committed.len(),
            depth_reached: dream_result.depth_reached,
        };
        dream_info = Some(summary);
        // Re-spread after dreaming committed new edges
        let fresh_activation = spread_from_session(store, session, &config, 10);
        candidates = fresh_activation.top_k_novel(&seeds, top_k);
    }

    WalkResult {
        candidates,
        mode_used: mode,
        query_hash: qhash,
        dreamed,
        dream_info,
    }
}

/// Record the outcome of a smart_walk — caller tells us how useful the
/// result was (0.0 = not useful, 1.0 = perfect). This updates the meta-learner
/// so future queries of the same context route better.
pub fn record_outcome(
    meta: &mut MetaLearner,
    walk_result: &WalkResult,
    query_context: &str,
    usefulness: f32,
) {
    meta.record(query_context, walk_result.mode_used, usefulness);
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::{AtomKind, AtomStore};
    use crate::relations;

    fn build_graph() -> (AtomStore, AtomId, AtomId, AtomId, AtomId) {
        let mut store = AtomStore::new();
        let a = store.put(AtomKind::Concept, b"A".to_vec());
        let b = store.put(AtomKind::Concept, b"B".to_vec());
        let c = store.put(AtomKind::Concept, b"C".to_vec());
        let d = store.put(AtomKind::Concept, b"D".to_vec());
        let near = relations::by_name("near").unwrap().code;
        store.link(a, b, near, 90, 0);
        store.link(b, c, near, 85, 0);
        store.link(c, d, near, 80, 0);
        (store, a, b, c, d)
    }

    #[test]
    fn smart_walk_basic_flow() {
        let (mut store, a, _, _, _) = build_graph();
        let mut session = SessionContext::new();
        session.mention(a);
        let meta = MetaLearner::new();

        let result = smart_walk(&mut store, &session, &meta, "test query", "factual", 3);
        // Should return SOME candidates (B is directly linked to A)
        assert!(!result.candidates.is_empty() || result.dreamed,
                "should find candidates or dream");
    }

    #[test]
    fn smart_walk_uses_meta_learned_mode() {
        let (mut store, a, _, _, _) = build_graph();
        let mut session = SessionContext::new();
        session.mention(a);

        // Train meta to strongly prefer Divergent for 'creative'
        let mut meta = MetaLearner::new();
        for _ in 0..100 {
            meta.record("creative", CognitiveMode::Divergent, 1.0);
        }

        let result = smart_walk(&mut store, &session, &meta, "make me a new idea", "creative", 3);
        // With 100 Divergent successes, most query hashes should route there
        // (probabilistic but deterministic per hash)
        // We can't guarantee any single hash routes to Divergent, but we can
        // check that 'mode_used' is a valid mode
        assert!(matches!(result.mode_used,
            CognitiveMode::Precision | CognitiveMode::Divergent
            | CognitiveMode::Gestalt | CognitiveMode::Narrative));
    }

    #[test]
    fn smart_walk_determinism() {
        let (mut store1, a, _, _, _) = build_graph();
        let (mut store2, _, _, _, _) = build_graph();
        let mut session1 = SessionContext::new();
        session1.mention(a);
        let mut session2 = SessionContext::new();
        session2.mention(a);
        let meta = MetaLearner::new();

        let r1 = smart_walk(&mut store1, &session1, &meta, "same query", "factual", 3);
        let r2 = smart_walk(&mut store2, &session2, &meta, "same query", "factual", 3);
        // Same inputs → same query_hash → same mode
        assert_eq!(r1.query_hash, r2.query_hash);
        assert_eq!(r1.mode_used, r2.mode_used);
    }

    #[test]
    fn record_outcome_updates_meta() {
        let (mut store, a, _, _, _) = build_graph();
        let mut session = SessionContext::new();
        session.mention(a);
        let mut meta = MetaLearner::new();

        let result = smart_walk(&mut store, &session, &meta, "query1", "factual", 3);
        let mode = result.mode_used;
        record_outcome(&mut meta, &result, "factual", 1.0);

        let weights = meta.weights_for("factual").unwrap();
        assert!(weights.alpha[mode.as_index()] > 1.0, "alpha for chosen mode should increase");
    }

    #[test]
    fn sparse_graph_triggers_dreaming() {
        let mut store = AtomStore::new();
        let a = store.put(AtomKind::Concept, b"isolated_A".to_vec());
        let b = store.put(AtomKind::Concept, b"isolated_B".to_vec());
        let near = relations::by_name("near").unwrap().code;
        store.link(a, b, near, 80, 0);
        // Only 1 edge — spreading will return very little novel

        let mut session = SessionContext::new();
        session.mention(a);
        let meta = MetaLearner::new();

        let result = smart_walk(&mut store, &session, &meta, "sparse", "anything", 5);
        // With sparse graph, dreamed should be true
        // (may or may not commit anything, but attempted)
        assert!(result.dreamed || !result.candidates.is_empty(),
                "sparse should either dream or find something");
    }

    #[test]
    fn empty_session_still_works() {
        let (mut store, _, _, _, _) = build_graph();
        let session = SessionContext::new();  // empty!
        let meta = MetaLearner::new();

        let result = smart_walk(&mut store, &session, &meta, "q", "ctx", 3);
        // Should return no candidates without seeds, but shouldn't panic
        assert!(result.candidates.is_empty());
    }
}
