//! Beam search — the actual walk executor.
//!
//! This is a sync implementation first. Async wrapper with tokio::spawn
//! comes after we verify correctness. Python prototype measured 30-550×
//! speedup even single-threaded (the win comes from beam=parallel-paths
//! exploration, not just threading).
//!
//! Graph-agnostic: takes a closure that returns neighbors for a given
//! node ID, so this can plug into any ZETS graph store.

use super::cancel::CancelToken;
use super::strategy::SearchStrategy;
use std::collections::HashSet;

/// A path through the graph.
pub type Path = Vec<u64>;

/// Result of a beam search.
#[derive(Debug, Clone)]
pub struct BeamResult {
    pub found: bool,
    pub answer_node: Option<u64>,
    pub path: Path,
    pub walks_executed: usize,
    pub steps_total: usize,
    pub cancelled_early: bool,
}

impl BeamResult {
    pub fn not_found() -> Self {
        Self {
            found: false,
            answer_node: None,
            path: vec![],
            walks_executed: 0,
            steps_total: 0,
            cancelled_early: false,
        }
    }
}

/// A single frontier entry: current node + the path used to reach it +
/// a heuristic score guiding which entries to expand first.
#[derive(Debug, Clone)]
struct FrontierEntry {
    node: u64,
    path: Path,
    score: f32,
}

/// Run a beam search.
///
/// Args:
/// - `start_nodes`: seed points. Should be ≥ 1 and ≤ strategy.beam_width.
/// - `strategy`: beam_width, max_depth, etc.
/// - `is_answer`: predicate that returns confidence [0,1] for each node.
///   If >= strategy.confidence_threshold, we stop.
/// - `neighbors`: returns list of (neighbor_id, heuristic_score) for a node.
/// - `cancel`: token to observe for early termination.
pub fn beam_search<F, N>(
    start_nodes: &[u64],
    strategy: &SearchStrategy,
    mut is_answer: F,
    mut neighbors: N,
    cancel: &CancelToken,
) -> BeamResult
where
    F: FnMut(u64) -> f32,
    N: FnMut(u64) -> Vec<(u64, f32)>,
{
    let strategy = strategy.clone().clamped();
    let mut visited: HashSet<u64> = HashSet::new();
    let mut steps_total: usize = 0;

    // Initial frontier: start_nodes with empty paths.
    let mut frontier: Vec<FrontierEntry> = start_nodes.iter()
        .take(strategy.beam_width)
        .map(|&n| FrontierEntry { node: n, path: vec![], score: 0.0 })
        .collect();

    for _depth in 0..strategy.max_depth {
        if cancel.is_cancelled() || frontier.is_empty() {
            break;
        }

        let mut next_frontier: Vec<FrontierEntry> = Vec::with_capacity(strategy.beam_width * 4);

        for entry in frontier.drain(..) {
            if cancel.is_cancelled() { break; }
            if visited.contains(&entry.node) { continue; }
            visited.insert(entry.node);

            steps_total += 1;
            let mut new_path = entry.path.clone();
            new_path.push(entry.node);

            // Check if this node is the answer.
            let conf = is_answer(entry.node);
            if conf >= strategy.confidence_threshold {
                cancel.cancel();
                return BeamResult {
                    found: true,
                    answer_node: Some(entry.node),
                    path: new_path,
                    walks_executed: start_nodes.len().min(strategy.beam_width),
                    steps_total,
                    cancelled_early: false,
                };
            }

            // Expand neighbors.
            for (nb, score) in neighbors(entry.node) {
                if !visited.contains(&nb) {
                    next_frontier.push(FrontierEntry {
                        node: nb,
                        path: new_path.clone(),
                        score,
                    });
                }
            }
        }

        // Keep top beam_width by heuristic score.
        next_frontier.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        next_frontier.truncate(strategy.beam_width);
        frontier = next_frontier;
    }

    BeamResult {
        found: false,
        answer_node: None,
        path: vec![],
        walks_executed: start_nodes.len().min(strategy.beam_width),
        steps_total,
        cancelled_early: cancel.is_cancelled(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::strategy::{StrategyLabel, default_strategies};

    /// Build a tiny tree graph:
    ///   0 -> [1, 2]
    ///   1 -> [3, 4]
    ///   2 -> [5, 6]
    ///   ... target at node 7
    fn mini_graph_with_target(target: u64, size: u64) -> impl FnMut(u64) -> Vec<(u64, f32)> {
        move |n: u64| -> Vec<(u64, f32)> {
            let mut r = Vec::new();
            // simple: parent n has children 2n+1, 2n+2 if in range
            let c1 = 2*n + 1;
            let c2 = 2*n + 2;
            if c1 < size {
                let score = if c1 == target { 1.0 }
                            else if c1 == target / 2 { 0.7 }
                            else { 0.3 };
                r.push((c1, score));
            }
            if c2 < size {
                let score = if c2 == target { 1.0 }
                            else if c2 == target / 2 { 0.7 }
                            else { 0.3 };
                r.push((c2, score));
            }
            r
        }
    }

    #[test]
    fn beam_finds_root() {
        let s = &default_strategies()[&StrategyLabel::Standard7x7];
        let cancel = CancelToken::new();
        let r = beam_search(
            &[0],
            s,
            |n| if n == 0 { 1.0 } else { 0.0 },
            mini_graph_with_target(999, 100),
            &cancel,
        );
        assert!(r.found);
        assert_eq!(r.answer_node, Some(0));
    }

    #[test]
    fn beam_finds_nearby_target() {
        let s = &default_strategies()[&StrategyLabel::Standard7x7];
        let cancel = CancelToken::new();
        let target = 13u64;
        let r = beam_search(
            &[0],
            s,
            |n| if n == target { 1.0 } else { 0.0 },
            mini_graph_with_target(target, 100),
            &cancel,
        );
        assert!(r.found, "should find target 13 in depth-7 beam search of binary tree");
        assert_eq!(r.answer_node, Some(target));
    }

    #[test]
    fn beam_respects_max_depth() {
        // Narrow strategy with depth=2 from root 0 in binary tree:
        // can reach 0, 1, 2, 3, 4, 5, 6 — so target 10 is unreachable.
        let s = super::super::strategy::SearchStrategy {
            label: StrategyLabel::Custom,
            beam_width: 2, max_depth: 2, retry_waves: 1,
            confidence_threshold: 0.9, description: "test",
        };
        let cancel = CancelToken::new();
        let r = beam_search(
            &[0],
            &s,
            |n| if n == 10 { 1.0 } else { 0.0 },
            mini_graph_with_target(10, 100),
            &cancel,
        );
        assert!(!r.found, "target should be unreachable within depth 2");
    }

    #[test]
    fn cancellation_stops_search() {
        let s = &default_strategies()[&StrategyLabel::Exhaustive];
        let cancel = CancelToken::new();
        cancel.cancel();  // cancel immediately
        let r = beam_search(
            &[0],
            s,
            |_| 0.0,
            mini_graph_with_target(99, 1000),
            &cancel,
        );
        assert!(!r.found);
        assert!(r.cancelled_early);
    }

    #[test]
    fn confidence_threshold_respected() {
        // Target returns 0.6; threshold is 0.9 → NOT found.
        let s = super::super::strategy::SearchStrategy {
            label: StrategyLabel::Custom,
            beam_width: 4, max_depth: 5, retry_waves: 1,
            confidence_threshold: 0.9, description: "test",
        };
        let cancel = CancelToken::new();
        let r = beam_search(
            &[0],
            &s,
            |n| if n == 5 { 0.6 } else { 0.0 },  // partial match only
            mini_graph_with_target(5, 50),
            &cancel,
        );
        assert!(!r.found, "0.6 < 0.9 threshold; should not count as found");
    }

    #[test]
    fn retry_waves_doesnt_affect_single_beam_search() {
        // beam_search itself doesn't loop over waves — that's a higher layer.
        let s = &default_strategies()[&StrategyLabel::Standard7x7];
        let cancel = CancelToken::new();
        let r = beam_search(
            &[0],
            s,
            |_| 0.0,
            |_| vec![],  // dead-end immediately
            &cancel,
        );
        assert!(!r.found);
        assert_eq!(r.walks_executed, 1);
    }
}
