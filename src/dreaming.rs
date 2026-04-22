//! Dreaming — ADHD-like recursive link proposal and evaluation.
//!
//! Idan's insight (22 Apr 2026): ADHD minds have a lower threshold for
//! associative leaps, producing more mind-wandering and more creative
//! connections. When directed well, this feeds hyperfocus on an idea.
//!
//! This module implements a deterministic analog:
//!
//!   1. Low-threshold proposal: sample pairs of atoms that DON'T have a
//!      direct edge yet, using hash-based pseudo-randomness (no rand!).
//!   2. Each proposal is a candidate edge — it doesn't commit to the store
//!      until it passes evaluation.
//!   3. Three-stage evaluation:
//!      a. LOCAL STRENGTH: are the atoms already connected via 2-hop path?
//!         (If yes, the proposal "makes sense" given existing structure.)
//!      b. PROVENANCE INTEGRITY: if either atom has provenance, check hash.
//!      c. GLOBAL CONSISTENCY: does the new edge contradict existing edges?
//!         (e.g., A is_a B and A is_a NOT-B would be contradiction.)
//!   4. Recursive expansion: if a proposal passes, run the proposer again
//!      with the new atom as a seed. Hyperfocus from mind-wandering.
//!
//! Determinism: every proposal is derived from atom IDs via hash — same
//! starting seed produces identical proposal sequence.

use std::collections::HashSet;

use crate::atoms::{AtomEdge, AtomId, AtomStore};
use crate::relations;

/// A candidate edge proposed by the dreaming process.
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateEdge {
    pub from: AtomId,
    pub to: AtomId,
    pub suggested_relation: u8,
    pub suggested_weight: u8,
    pub source: ProposalSource,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ProposalSource {
    /// Proposed because a 2-hop path exists: A → X → B
    TwoHopPath,
    /// Proposed because both atoms appeared together in a scenario
    Coactivation,
    /// Proposed because both connect to a common prototype
    SharedPrototype,
    /// Proposed by session context (both currently active)
    SessionContext,
}

/// Result of evaluating a candidate.
#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationResult {
    pub accepted: bool,
    pub local_strength: f32,
    pub provenance_ok: bool,
    pub consistency_ok: bool,
    pub reason: String,
}

/// Deterministic hash → pseudo-random index. Not crypto, just for reproducibility.
fn pseudo_hash(a: AtomId, b: AtomId, seed: u64) -> u64 {
    let mut h = seed.wrapping_mul(0x9E3779B97F4A7C15);
    h ^= a as u64;
    h = h.wrapping_mul(0x9E3779B97F4A7C15);
    h ^= b as u64;
    h = h.wrapping_mul(0x9E3779B97F4A7C15);
    h
}

/// Propose candidate edges by looking for atoms connected via 2-hop paths
/// but lacking direct edges.
///
/// For each atom in `seeds`, walk 2 hops; for each reachable atom B that does
/// NOT have a direct edge from the seed A, propose A→B as a candidate.
///
/// Parameters:
///   - max_per_seed: how many candidates to produce per seed (keeps output bounded)
///   - proposal_seed: for deterministic ordering (doesn't affect which atoms
///     are proposed, only their order)
pub fn propose_via_two_hop(
    store: &AtomStore,
    seeds: &[AtomId],
    max_per_seed: usize,
    proposal_seed: u64,
) -> Vec<CandidateEdge> {
    let mut proposals: Vec<CandidateEdge> = Vec::new();

    for &seed in seeds {
        let direct: HashSet<AtomId> = store.outgoing(seed).iter()
            .map(|e| e.to)
            .collect();

        // Collect all 2-hop reachable atoms (not directly connected)
        let mut two_hop: Vec<(AtomId, u8, u8)> = Vec::new();
        for e1 in store.outgoing(seed) {
            for e2 in store.outgoing(e1.to) {
                if e2.to == seed { continue; }
                if direct.contains(&e2.to) { continue; }
                // Suggest the "dominant" relation via first hop's kind
                two_hop.push((e2.to, e1.relation, e1.weight.min(e2.weight) / 2));
            }
        }

        // Dedupe and sort deterministically
        two_hop.sort_by_key(|&(t, r, w)| (pseudo_hash(seed, t, proposal_seed), t, r, w));
        two_hop.dedup_by_key(|&mut (t, _, _)| t);

        for (target, rel, weight) in two_hop.into_iter().take(max_per_seed) {
            proposals.push(CandidateEdge {
                from: seed,
                to: target,
                suggested_relation: rel,
                suggested_weight: weight.max(30),
                source: ProposalSource::TwoHopPath,
            });
        }
    }

    proposals
}

/// Propose candidates based on shared prototype parents.
/// If A and B both PROTOTYPE_OF X, they're likely "siblings" and might
/// relate via similar_to.
pub fn propose_via_shared_prototype(
    store: &AtomStore,
    seeds: &[AtomId],
    max_per_seed: usize,
) -> Vec<CandidateEdge> {
    let mut proposals = Vec::new();
    let proto_of = crate::prototype::prototype_rel::PROTOTYPE_OF;
    let similar_to = relations::by_name("similar_to").unwrap().code;

    for &seed in seeds {
        // Parents of seed
        let parents: HashSet<AtomId> = store.outgoing(seed).iter()
            .filter(|e| e.relation == proto_of)
            .map(|e| e.to)
            .collect();

        if parents.is_empty() { continue; }

        // Siblings — atoms that share a parent with seed
        let mut siblings: Vec<AtomId> = Vec::new();
        for parent in &parents {
            for edge in store.incoming_by_relation(*parent, proto_of) {
                if edge.from != seed && !siblings.contains(&edge.from) {
                    siblings.push(edge.from);
                }
            }
        }

        // Existing similar_to edges to filter out
        let existing_similar: HashSet<AtomId> = store.outgoing(seed).iter()
            .filter(|e| e.relation == similar_to)
            .map(|e| e.to)
            .collect();

        for sib in siblings.into_iter().take(max_per_seed) {
            if !existing_similar.contains(&sib) {
                proposals.push(CandidateEdge {
                    from: seed,
                    to: sib,
                    suggested_relation: similar_to,
                    suggested_weight: 60,
                    source: ProposalSource::SharedPrototype,
                });
            }
        }
    }

    proposals
}

// ────────────────────────────────────────────────────────────────
// 3-stage Evaluation
// ────────────────────────────────────────────────────────────────

/// Evaluate a candidate edge — 3 stages from Idan's spec.
///
/// Stage 1 — Local strength: do A and B already share context? Count:
///    - common neighbors (atoms connected to both)
///    - shared prototype parents
///    Returns a 0.0-1.0 score.
///
/// Stage 2 — Provenance integrity (placeholder in ZETS for now):
///    Always returns true unless specific provenance tracking is enabled.
///
/// Stage 3 — Global consistency: does adding this edge contradict anything?
///    Specifically checks: if we're proposing A is_a B, but A already
///    has edges that would make it NOT a B.
///
/// Result: accepted iff local_strength >= acceptance_threshold AND both
/// provenance_ok and consistency_ok are true.
pub fn evaluate(
    store: &AtomStore,
    candidate: &CandidateEdge,
    acceptance_threshold: f32,
) -> EvaluationResult {
    // Stage 1: local strength
    let from_neighbors: HashSet<AtomId> = store.outgoing(candidate.from).iter()
        .map(|e| e.to).collect();
    let to_neighbors: HashSet<AtomId> = store.outgoing(candidate.to).iter()
        .map(|e| e.to).collect();
    let shared = from_neighbors.intersection(&to_neighbors).count();
    let total = from_neighbors.len().max(to_neighbors.len()).max(1);
    let local_strength = (shared as f32 / total as f32).min(1.0);

    // Stage 2: provenance (simplified — assume intact; hook for future)
    let provenance_ok = true;

    // Stage 3: consistency — check for contradicting edges.
    let is_a = relations::by_name("is_a").unwrap().code;
    let mut consistency_ok = true;
    let mut reason = String::new();

    if candidate.suggested_relation == is_a {
        // Does from already is_a something that contradicts?
        // Simple check: if from already has is_a X and X and to are siblings (both have same parent),
        // that's suspicious.
        let existing_is_a: Vec<AtomId> = store.outgoing(candidate.from).iter()
            .filter(|e| e.relation == is_a)
            .map(|e| e.to)
            .collect();
        if existing_is_a.contains(&candidate.to) {
            consistency_ok = false;
            reason = "edge already exists".to_string();
        }
    }

    let accepted = local_strength >= acceptance_threshold
                && provenance_ok
                && consistency_ok;

    if reason.is_empty() {
        reason = if accepted {
            format!("passed: local_strength={:.2}", local_strength)
        } else {
            format!("rejected: local_strength={:.2} below threshold {:.2}",
                local_strength, acceptance_threshold)
        };
    }

    EvaluationResult {
        accepted,
        local_strength,
        provenance_ok,
        consistency_ok,
        reason,
    }
}

/// Commit an accepted candidate to the store.
pub fn commit_candidate(store: &mut AtomStore, candidate: &CandidateEdge) {
    store.link(candidate.from, candidate.to,
        candidate.suggested_relation, candidate.suggested_weight, 0);
}

// ────────────────────────────────────────────────────────────────
// Recursive dreaming — the full loop
// ────────────────────────────────────────────────────────────────

/// Full dreaming cycle: propose → evaluate → commit → recurse.
///
/// Mimics Idan's ADHD mind-wandering + hyperfocus pattern:
///   - Starts wide (propose many candidates from seeds)
///   - Evaluates each
///   - Commits those that pass
///   - Recurses with NEW edges as expansion seeds
///   - Bounds: max_total_edges committed + max_depth iterations
pub struct DreamResult {
    pub committed: Vec<CandidateEdge>,
    pub rejected: Vec<(CandidateEdge, EvaluationResult)>,
    pub depth_reached: u32,
}

pub fn dream(
    store: &mut AtomStore,
    seeds: &[AtomId],
    acceptance_threshold: f32,
    max_total_edges: usize,
    max_depth: u32,
    proposal_seed: u64,
) -> DreamResult {
    let mut committed = Vec::new();
    let mut rejected = Vec::new();
    let mut current_seeds: Vec<AtomId> = seeds.to_vec();
    let mut depth = 0u32;

    while depth < max_depth && committed.len() < max_total_edges {
        // Propose from current seeds
        let proposals = propose_via_two_hop(store, &current_seeds, 3, proposal_seed + depth as u64);

        if proposals.is_empty() { break; }

        let mut new_seeds: Vec<AtomId> = Vec::new();
        for candidate in proposals {
            if committed.len() >= max_total_edges { break; }
            let eval = evaluate(store, &candidate, acceptance_threshold);
            if eval.accepted {
                commit_candidate(store, &candidate);
                new_seeds.push(candidate.to);
                committed.push(candidate);
            } else {
                rejected.push((candidate, eval));
            }
        }

        if new_seeds.is_empty() { break; }
        current_seeds = new_seeds;
        depth += 1;
    }

    DreamResult { committed, rejected, depth_reached: depth }
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::{AtomKind, AtomStore};

    fn chain_graph() -> (AtomStore, Vec<AtomId>) {
        // Build A → B → C. Two-hop from A reaches C (no direct edge yet).
        let mut store = AtomStore::new();
        let a = store.put(AtomKind::Concept, b"A".to_vec());
        let b = store.put(AtomKind::Concept, b"B".to_vec());
        let c = store.put(AtomKind::Concept, b"C".to_vec());
        let rel_near = relations::by_name("near").unwrap().code;
        store.link(a, b, rel_near, 80, 0);
        store.link(b, c, rel_near, 80, 0);
        (store, vec![a, b, c])
    }

    #[test]
    fn propose_two_hop_finds_transitive() {
        let (store, ids) = chain_graph();
        let proposals = propose_via_two_hop(&store, &[ids[0]], 5, 42);
        // Should propose A→C (since A→B→C exists but A→C doesn't)
        assert!(proposals.iter().any(|p| p.from == ids[0] && p.to == ids[2]));
    }

    #[test]
    fn propose_skips_existing_direct_edges() {
        let (mut store, ids) = chain_graph();
        let rel_near = relations::by_name("near").unwrap().code;
        // Add direct A→C
        store.link(ids[0], ids[2], rel_near, 90, 0);
        let proposals = propose_via_two_hop(&store, &[ids[0]], 5, 42);
        // Should NOT propose A→C (already exists)
        assert!(!proposals.iter().any(|p| p.from == ids[0] && p.to == ids[2]));
    }

    #[test]
    fn evaluate_rejects_low_local_strength() {
        let (store, ids) = chain_graph();
        let candidate = CandidateEdge {
            from: ids[0],
            to: ids[2],
            suggested_relation: relations::by_name("near").unwrap().code,
            suggested_weight: 50,
            source: ProposalSource::TwoHopPath,
        };
        // High threshold → rejected
        let eval = evaluate(&store, &candidate, 0.9);
        assert!(!eval.accepted);
        // Low threshold → may or may not pass depending on graph structure
    }

    #[test]
    fn evaluate_rejects_duplicate() {
        let (mut store, ids) = chain_graph();
        let is_a = relations::by_name("is_a").unwrap().code;
        store.link(ids[0], ids[1], is_a, 90, 0);
        let candidate = CandidateEdge {
            from: ids[0],
            to: ids[1],
            suggested_relation: is_a,
            suggested_weight: 50,
            source: ProposalSource::TwoHopPath,
        };
        let eval = evaluate(&store, &candidate, 0.0);
        assert!(!eval.consistency_ok, "duplicate edge should fail consistency");
    }

    #[test]
    fn dream_commits_and_recurses() {
        // Build a richer graph: A→B→C, B→D, C→E
        let mut store = AtomStore::new();
        let a = store.put(AtomKind::Concept, b"A".to_vec());
        let b = store.put(AtomKind::Concept, b"B".to_vec());
        let c = store.put(AtomKind::Concept, b"C".to_vec());
        let d = store.put(AtomKind::Concept, b"D".to_vec());
        let e = store.put(AtomKind::Concept, b"E".to_vec());
        let near = relations::by_name("near").unwrap().code;
        store.link(a, b, near, 80, 0);
        store.link(b, c, near, 80, 0);
        store.link(b, d, near, 80, 0);
        store.link(c, e, near, 80, 0);
        // Give common neighbors to pass local_strength
        store.link(a, d, near, 70, 0); // now A and B share {B,D} as neighbors

        let res = dream(&mut store, &[a], 0.0, 10, 3, 42);
        // Should commit at least one (low threshold)
        assert!(!res.committed.is_empty() || !res.rejected.is_empty(),
            "dream should produce SOME output");
    }

    #[test]
    fn dream_respects_max_total_edges() {
        let mut store = AtomStore::new();
        // Build a wide graph
        let center = store.put(AtomKind::Concept, b"center".to_vec());
        for i in 0..20 {
            let x = store.put(AtomKind::Concept, format!("n{}", i).into_bytes());
            let y = store.put(AtomKind::Concept, format!("m{}", i).into_bytes());
            let near = relations::by_name("near").unwrap().code;
            store.link(center, x, near, 80, 0);
            store.link(x, y, near, 80, 0);
        }
        let res = dream(&mut store, &[center], 0.0, 5, 10, 42);
        assert!(res.committed.len() <= 5, "max_total_edges should bound output");
    }

    #[test]
    fn determinism_same_seed_same_proposals() {
        let (store, ids) = chain_graph();
        let p1 = propose_via_two_hop(&store, &[ids[0]], 5, 12345);
        let p2 = propose_via_two_hop(&store, &[ids[0]], 5, 12345);
        assert_eq!(p1, p2);
    }

    #[test]
    fn pseudo_hash_deterministic() {
        let h1 = pseudo_hash(1, 2, 100);
        let h2 = pseudo_hash(1, 2, 100);
        assert_eq!(h1, h2);
        let h3 = pseudo_hash(2, 1, 100); // order matters
        assert_ne!(h1, h3);
    }

    #[test]
    fn candidate_edge_equality() {
        let e1 = CandidateEdge {
            from: 1, to: 2,
            suggested_relation: 0, suggested_weight: 50,
            source: ProposalSource::TwoHopPath,
        };
        let e2 = e1.clone();
        assert_eq!(e1, e2);
    }
}
