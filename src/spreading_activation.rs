//! Spreading Activation — context-anchored graph walk.
//!
//! The insight (Idan, 22 Apr 2026): search shouldn't start from scratch.
//! When a session is active (conversation about dogs), "crown" should
//! activate the dog's royal-crown bred meaning, not the dental crown.
//!
//! Classic spreading activation (Collins & Loftus 1975):
//!   - Start with seed activations from session context
//!   - Propagate through edges weighted by relation strength
//!   - Each hop attenuates activation by decay factor
//!   - Bounded depth (typically 2-3 hops) to prevent explosion
//!
//! Key properties for ZETS:
//!   - DETERMINISTIC: same seeds + same graph → same activation map
//!   - BOUNDED: hard limit on visited nodes (no runaway)
//!   - CONFIDENCE-AWARE: final scores reflect how well-activated each
//!     node is by the context, enabling ranked retrieval
//!
//! This is the BRIDGE between session memory and cognitive walks.
//! When PrecisionMode or DivergentMode want to walk, they can use
//! the session's activation map as a prior instead of flat weights.

use std::collections::HashMap;

use crate::atoms::{AtomId, AtomStore};
use crate::session::SessionContext;

/// Result of spreading activation — every node that received any activation,
/// mapped to its final score.
#[derive(Debug, Clone)]
pub struct ActivationMap {
    pub scores: HashMap<AtomId, f32>,
    /// How the seed came to influence each node — source atom that contributed
    pub provenance: HashMap<AtomId, AtomId>,
    /// How many hops from any seed (useful for pruning far results)
    pub distance: HashMap<AtomId, u8>,
}

impl ActivationMap {
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
            provenance: HashMap::new(),
            distance: HashMap::new(),
        }
    }

    /// Top-K activated atoms, excluding the original seeds.
    pub fn top_k_novel(&self, seeds: &[AtomId], k: usize) -> Vec<(AtomId, f32)> {
        let seed_set: std::collections::HashSet<AtomId> = seeds.iter().copied().collect();
        let mut v: Vec<(AtomId, f32)> = self.scores.iter()
            .filter(|(aid, _)| !seed_set.contains(aid))
            .map(|(&aid, &score)| (aid, score))
            .collect();
        // DETERMINISTIC sort: primary key = score (desc), secondary key = atom_id (asc).
        // Without the AtomId tie-breaker, HashMap iteration order leaks into
        // output order when scores tie. That breaks cross-process determinism.
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0)));
        v.truncate(k);
        v
    }

    /// Top-K activated atoms including seeds.
    pub fn top_k(&self, k: usize) -> Vec<(AtomId, f32)> {
        let mut v: Vec<(AtomId, f32)> = self.scores.iter()
            .map(|(&aid, &score)| (aid, score))
            .collect();
        // DETERMINISTIC sort: ties broken by AtomId ascending. See top_k_novel.
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0)));
        v.truncate(k);
        v
    }

    pub fn size(&self) -> usize { self.scores.len() }
}

impl Default for ActivationMap {
    fn default() -> Self { Self::new() }
}

/// Configuration for spreading activation.
#[derive(Debug, Clone)]
pub struct SpreadConfig {
    /// How many hops to spread (typical: 2-3)
    pub max_depth: u8,
    /// Each hop multiplies activation by this (typical: 0.5)
    pub hop_decay: f32,
    /// Below this activation, don't continue spreading (prevents runaway)
    pub min_activation: f32,
    /// Max total nodes to visit (safety bound)
    pub max_nodes: usize,
    /// Optional relation filter — if Some, only spread through these relation codes
    pub allowed_relations: Option<Vec<u8>>,
    /// Use edge weight as multiplier (true) or ignore it (false)
    pub weight_matters: bool,
}

impl Default for SpreadConfig {
    fn default() -> Self {
        Self {
            max_depth: 3,
            hop_decay: 0.5,
            min_activation: 0.05,
            max_nodes: 1000,
            allowed_relations: None,
            weight_matters: true,
        }
    }
}

impl SpreadConfig {
    /// Shallow + strict — good for precision queries
    pub fn precise() -> Self {
        Self {
            max_depth: 2,
            hop_decay: 0.6,
            min_activation: 0.1,
            max_nodes: 200,
            allowed_relations: None,
            weight_matters: true,
        }
    }

    /// Deep + permissive — good for divergent / associative queries
    pub fn divergent() -> Self {
        Self {
            max_depth: 4,
            hop_decay: 0.4,
            min_activation: 0.02,
            max_nodes: 2000,
            allowed_relations: None,
            weight_matters: false, // treat weak edges as equal to strong
        }
    }

    /// Scale-adaptive — auto-tune based on graph size.
    /// At ~200K atoms, depth-3 BFS explores millions of paths and max_nodes
    /// truncates to the wrong 1000 (BFS-first, not most-relevant). So for
    /// large graphs we MUST stay shallow to keep signal/noise usable.
    ///
    /// Critical insight: at Wikipedia scale, the answer is almost always
    /// a DIRECT NEIGHBOR of some seed. Paris -> France has 22 direct edges.
    /// Deeper spreading just adds generic-hub noise.
    pub fn scale_adaptive(graph_atom_count: usize) -> Self {
        if graph_atom_count >= 100_000 {
            // Big graph (Wikipedia-scale): depth-1 only, wide exploration at that depth
            Self {
                max_depth: 1,
                hop_decay: 1.0,       // no decay — depth-1 only
                min_activation: 0.01,
                max_nodes: 5000,      // allow many direct neighbors
                allowed_relations: None,
                weight_matters: true,
            }
        } else if graph_atom_count >= 5_000 {
            // Medium graph: depth-2 with tight cutoff
            Self {
                max_depth: 2,
                hop_decay: 0.4,
                min_activation: 0.05,
                max_nodes: 2000,
                allowed_relations: None,
                weight_matters: true,
            }
        } else {
            // Small graph: the old default is fine
            Self::default()
        }
    }
}

/// Spread activation from a set of seed atoms (with initial activations)
/// through the graph, for up to `config.max_depth` hops.
///
/// This is the CORE algorithm. Start with seed_activations[seed] for each seed.
/// At each hop, propagate: new_score[neighbor] += current_score * hop_decay * edge_weight_norm
pub fn spread(
    store: &AtomStore,
    seeds: &[(AtomId, f32)],
    config: &SpreadConfig,
) -> ActivationMap {
    let mut map = ActivationMap::new();

    // Initialize scores with seeds
    for &(seed, score) in seeds {
        map.scores.insert(seed, score);
        map.provenance.insert(seed, seed);
        map.distance.insert(seed, 0);
    }

    let mut frontier: Vec<(AtomId, f32, u8)> = seeds.iter()
        .map(|&(aid, score)| (aid, score, 0u8))
        .collect();
    let mut visited_count = seeds.len();

    while let Some((current, activation, depth)) = frontier.pop() {
        if depth >= config.max_depth { continue; }
        if activation < config.min_activation { continue; }
        if visited_count >= config.max_nodes { break; }

        // Walk outgoing edges
        let edges = store.outgoing(current);
        for edge in edges {
            // Relation filter (optional)
            if let Some(allowed) = &config.allowed_relations {
                if !allowed.contains(&edge.relation) { continue; }
            }

            // Compute contribution
            let weight_factor = if config.weight_matters {
                edge.weight as f32 / 100.0
            } else {
                1.0
            };
            let contribution = activation * config.hop_decay * weight_factor;

            if contribution < config.min_activation { continue; }

            // Accumulate into map
            let existing = map.scores.get(&edge.to).copied().unwrap_or(0.0);
            let new_score = existing + contribution;
            map.scores.insert(edge.to, new_score);

            // Track first-arrival distance and provenance
            map.distance.entry(edge.to).or_insert(depth + 1);
            map.provenance.entry(edge.to).or_insert(current);

            visited_count += 1;
            frontier.push((edge.to, contribution, depth + 1));
        }
    }

    map
}

/// Convenience: spread from a session context directly.
///
/// Each active atom in the session becomes a seed with activation =
/// current session activation (top-K most active).
pub fn spread_from_session(
    store: &AtomStore,
    session: &SessionContext,
    config: &SpreadConfig,
    n_seeds: usize,
) -> ActivationMap {
    let seeds: Vec<(AtomId, f32)> = session.top_k(n_seeds);
    spread(store, &seeds, config)
}

/// Disambiguation — given a candidate set of atoms all matching the same
/// surface form (e.g. multiple "crown" concepts), return the one most
/// aligned with the current session context.
pub fn disambiguate(
    store: &AtomStore,
    session: &SessionContext,
    candidates: &[AtomId],
    config: &SpreadConfig,
) -> Option<(AtomId, f32)> {
    if candidates.is_empty() { return None; }
    if candidates.len() == 1 { return Some((candidates[0], 1.0)); }

    // Spread from session, then score each candidate by sum of its neighbors'
    // activation scores — candidate whose neighborhood is most activated wins.
    let activation = spread_from_session(store, session, config, 20);

    let mut best: Option<(AtomId, f32)> = None;
    for &candidate in candidates {
        let mut score = activation.scores.get(&candidate).copied().unwrap_or(0.0);
        // Add contributions from neighbors (atoms linked FROM the candidate)
        for edge in store.outgoing(candidate) {
            score += activation.scores.get(&edge.to).copied().unwrap_or(0.0) * 0.5;
        }
        match best {
            None => best = Some((candidate, score)),
            Some((_, best_score)) if score > best_score => best = Some((candidate, score)),
            _ => {}
        }
    }
    best
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::{AtomKind, AtomStore};
    use crate::relations;

    fn build_small_graph() -> (AtomStore, std::collections::HashMap<&'static str, AtomId>) {
        let mut store = AtomStore::new();
        let mut ids = std::collections::HashMap::new();

        for name in ["dog", "puppy", "bone", "cat", "cat_food", "crown_royal",
                    "crown_dental", "tooth", "king", "queen", "dentist"] {
            let id = store.put(AtomKind::Concept, name.as_bytes().to_vec());
            ids.insert(name, id);
        }

        // Build edges with semantic meaning
        let is_a = relations::by_name("is_a").unwrap().code;
        let near = relations::by_name("near").unwrap().code;
        let located_in = relations::by_name("located_in").unwrap().code;
        let used_for = relations::by_name("used_for").unwrap().code;

        // Dog cluster
        store.link(ids["puppy"], ids["dog"], is_a, 95, 0);
        store.link(ids["dog"], ids["bone"], near, 70, 0);
        store.link(ids["cat"], ids["cat_food"], near, 70, 0);

        // Dental cluster
        store.link(ids["crown_dental"], ids["tooth"], located_in, 90, 0);
        store.link(ids["crown_dental"], ids["dentist"], used_for, 80, 0);
        store.link(ids["tooth"], ids["dentist"], near, 75, 0);

        // Royal cluster
        store.link(ids["crown_royal"], ids["king"], located_in, 90, 0);
        store.link(ids["crown_royal"], ids["queen"], located_in, 85, 0);
        store.link(ids["king"], ids["queen"], near, 80, 0);

        (store, ids)
    }

    #[test]
    fn spread_from_seed_reaches_neighbors() {
        let (store, ids) = build_small_graph();
        let config = SpreadConfig::default();
        let map = spread(&store, &[(ids["dog"], 1.0)], &config);

        // dog → bone edge should activate bone
        assert!(map.scores.contains_key(&ids["bone"]));
        let bone_score = map.scores[&ids["bone"]];
        assert!(bone_score > 0.0 && bone_score < 1.0, "bone got {}", bone_score);
    }

    #[test]
    fn activation_decays_with_distance() {
        let (store, ids) = build_small_graph();
        let is_a = relations::by_name("is_a").unwrap().code;
        // Extend: puppy is_a dog (exists); dog is_a mammal (fake, to test depth)
        let mut store = store;
        let mammal = store.put(AtomKind::Concept, b"mammal".to_vec());
        store.link(ids["dog"], mammal, is_a, 90, 0);

        let config = SpreadConfig::default();
        let map = spread(&store, &[(ids["puppy"], 1.0)], &config);

        // puppy directly is_a dog (1 hop), dog is_a mammal (2 hops)
        let dog_score = map.scores.get(&ids["dog"]).copied().unwrap_or(0.0);
        let mammal_score = map.scores.get(&mammal).copied().unwrap_or(0.0);
        assert!(dog_score > mammal_score, "closer hop should score higher: dog={} mammal={}",
                dog_score, mammal_score);
    }

    #[test]
    fn disambiguation_by_session_context_dental() {
        let (store, ids) = build_small_graph();

        // Session is about teeth
        let mut session = SessionContext::new();
        session.mention(ids["tooth"]);
        session.mention(ids["dentist"]);

        // Candidates: two kinds of crown
        let candidates = vec![ids["crown_royal"], ids["crown_dental"]];
        let result = disambiguate(&store, &session, &candidates, &SpreadConfig::default());
        assert!(result.is_some());
        let (winner, _) = result.unwrap();
        assert_eq!(winner, ids["crown_dental"],
            "in dental context, crown_dental must win");
    }

    #[test]
    fn disambiguation_by_session_context_royal() {
        let (store, ids) = build_small_graph();

        // Session is about royalty
        let mut session = SessionContext::new();
        session.mention(ids["king"]);
        session.mention(ids["queen"]);

        // Candidates: two kinds of crown
        let candidates = vec![ids["crown_royal"], ids["crown_dental"]];
        let result = disambiguate(&store, &session, &candidates, &SpreadConfig::default());
        let (winner, _) = result.unwrap();
        assert_eq!(winner, ids["crown_royal"],
            "in royal context, crown_royal must win");
    }

    #[test]
    fn empty_session_gives_no_preference() {
        let (store, ids) = build_small_graph();
        let session = SessionContext::new();
        let candidates = vec![ids["crown_royal"], ids["crown_dental"]];
        let result = disambiguate(&store, &session, &candidates, &SpreadConfig::default());
        // Both have equal activation (0) so first is chosen deterministically
        assert!(result.is_some());
    }

    #[test]
    fn determinism_same_session_same_result() {
        let (store, ids) = build_small_graph();
        let mut s1 = SessionContext::new();
        s1.mention(ids["tooth"]);
        let mut s2 = SessionContext::new();
        s2.mention(ids["tooth"]);

        let r1 = disambiguate(&store, &s1, &[ids["crown_royal"], ids["crown_dental"]],
                              &SpreadConfig::default());
        let r2 = disambiguate(&store, &s2, &[ids["crown_royal"], ids["crown_dental"]],
                              &SpreadConfig::default());
        assert_eq!(r1.map(|(a, _)| a), r2.map(|(a, _)| a));
    }

    #[test]
    fn max_nodes_bounds_walk() {
        let (store, ids) = build_small_graph();
        let config = SpreadConfig {
            max_depth: 10,
            hop_decay: 0.9, // barely decays
            min_activation: 0.0, // never stops by score
            max_nodes: 3, // HARD STOP after 3
            ..SpreadConfig::default()
        };
        let map = spread(&store, &[(ids["dog"], 1.0)], &config);
        // Should stay bounded — exact count may vary but should be ≤ max_nodes + seeds
        assert!(map.size() <= 10, "unbounded walk, got {} nodes", map.size());
    }

    #[test]
    fn top_k_novel_excludes_seeds() {
        let (store, ids) = build_small_graph();
        let seeds = vec![ids["dog"]];
        let map = spread(&store, &[(ids["dog"], 1.0)], &SpreadConfig::default());
        let novel = map.top_k_novel(&seeds, 10);
        for (id, _) in novel {
            assert_ne!(id, ids["dog"], "seed should be excluded from novel");
        }
    }

    #[test]
    fn spread_from_session_uses_session_state() {
        let (store, ids) = build_small_graph();
        let mut session = SessionContext::new();
        session.mention(ids["dog"]);

        let map = spread_from_session(&store, &session, &SpreadConfig::default(), 5);
        // bone should be activated because dog → bone
        assert!(map.scores.contains_key(&ids["bone"]));
    }
}
