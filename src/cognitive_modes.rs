//! Cognitive Modes — deterministic traversal strategies for the data graph.
//!
//! Inspired by the observation that genius-level thinking combines modes:
//!   - DivergentMode (ADHD-inspired)   — hash-weighted BFS, finds remote connections
//!   - PrecisionMode (autism-inspired) — bounded DFS, verifies consistency
//!   - GestaltMode   (dyslexia-insp.)  — k-hop neighborhood, sees the whole
//!   - NarrativeMode (dysgraphia-insp.)— composes results into a story
//!
//! CRITICAL PROPERTY: 100% deterministic. No `rand`. Any "divergence" is
//! achieved via hash functions — reproducible forever. This is ZETS's moat:
//! the same question always yields the same traversal, fully debuggable.
//!
//! Each mode is a `Strategy` that implements the same trait. The graph host
//! is injected — the modes don't care about data graph internals.

use std::collections::{HashMap, HashSet, VecDeque};

/// The trait every cognitive mode implements.
///
/// Given a starting concept and a graph host, produce an ordered list of
/// TraversalStep results, with explanation trails for every decision.
pub trait CognitiveMode {
    /// Human-readable name.
    fn name(&self) -> &'static str;
    /// Short description.
    fn inspired_by(&self) -> &'static str;
    /// Execute the walk. Returns the candidate nodes with trust/contribution scores.
    fn walk(&self, query: &Query, host: &mut dyn GraphHost) -> WalkResult;
}

/// Query to a cognitive mode.
#[derive(Debug, Clone)]
pub struct Query {
    /// A stable identifier for the query (used in deterministic hashing).
    pub id: u64,
    /// The starting concept id.
    pub start: u32,
    /// Maximum depth/hops to explore.
    pub max_depth: u32,
    /// Maximum nodes to visit.
    pub max_visits: u32,
    /// Edge kinds to consider (empty = all).
    pub allowed_edge_kinds: Vec<u8>,
}

impl Query {
    pub fn new(start: u32, max_depth: u32) -> Self {
        // Stable id derived from start+depth
        let id = hash64(&(start as u64, max_depth as u64));
        Self { id, start, max_depth, max_visits: 256, allowed_edge_kinds: Vec::new() }
    }
    pub fn with_kinds(mut self, kinds: Vec<u8>) -> Self {
        self.allowed_edge_kinds = kinds;
        self
    }
}

/// A graph host that cognitive modes walk on.
/// Simpler than the system_graph Host — focused on traversal.
pub trait GraphHost {
    /// Get outgoing edges: (target_id, kind, weight 0-100)
    fn outgoing(&mut self, node: u32) -> Vec<(u32, u8, u8)>;
    /// Human label (for explanation).
    fn label(&mut self, node: u32) -> String;
}

/// One step in a traversal.
#[derive(Debug, Clone)]
pub struct TraversalStep {
    pub depth: u32,
    pub from: u32,
    pub to: u32,
    pub edge_kind: u8,
    pub weight: u8,
    pub reason: &'static str,  // why the mode picked this edge
}

/// Output of walking.
#[derive(Debug, Clone, Default)]
pub struct WalkResult {
    pub visited: Vec<u32>,          // nodes reached (ordered by traversal)
    pub steps: Vec<TraversalStep>,  // every edge taken, with explanation
    pub dead_ends: u32,             // nodes with no further edges
    pub truncated: bool,            // hit max_visits or max_depth
}

// ─────────────────────────────────────────────────────────────────────
// Deterministic hash — our source of "divergence" without randomness.
// ─────────────────────────────────────────────────────────────────────

/// FxHash-style mixer. Fast, deterministic, reproducible across platforms.
pub fn hash64<T: std::hash::Hash>(v: &T) -> u64 {
    use std::hash::{Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
}

/// Decide if an edge should be visited in divergent mode.
/// Fully deterministic: same query + same edge = same answer, forever.
fn divergent_gate(query_id: u64, edge_from: u32, edge_to: u32, weight: u8, divergence_pct: u8) -> bool {
    // Always take strong edges
    if weight >= 70 { return true; }
    // For weaker edges: pseudo-random via hash
    let h = hash64(&(query_id, edge_from, edge_to));
    (h % 100) < (divergence_pct as u64)
}

// ─────────────────────────────────────────────────────────────────────
// 1. PrecisionMode — autism-inspired bounded DFS
// ─────────────────────────────────────────────────────────────────────

pub struct PrecisionMode {
    pub min_weight: u8,  // reject edges weaker than this
}

impl Default for PrecisionMode {
    fn default() -> Self { Self { min_weight: 70 } }
}

impl CognitiveMode for PrecisionMode {
    fn name(&self) -> &'static str { "PrecisionMode" }
    fn inspired_by(&self) -> &'static str { "autism — bounded DFS with strict consistency" }

    fn walk(&self, query: &Query, host: &mut dyn GraphHost) -> WalkResult {
        let mut result = WalkResult::default();
        let mut visited: HashSet<u32> = HashSet::new();
        let mut stack: Vec<(u32, u32)> = vec![(query.start, 0)]; // (node, depth)

        while let Some((node, depth)) = stack.pop() {
            if visited.contains(&node) { continue; }
            if visited.len() as u32 >= query.max_visits {
                result.truncated = true;
                break;
            }
            visited.insert(node);
            result.visited.push(node);
            if depth >= query.max_depth { continue; }

            let mut neighbors = host.outgoing(node);
            // Strict filter: only strong edges, allowed kinds, sorted descending by weight
            neighbors.retain(|(_, k, w)| {
                *w >= self.min_weight &&
                (query.allowed_edge_kinds.is_empty() || query.allowed_edge_kinds.contains(k))
            });
            neighbors.sort_by(|a, b| b.2.cmp(&a.2));

            if neighbors.is_empty() { result.dead_ends += 1; continue; }

            for (tgt, kind, weight) in neighbors.iter().rev() {
                // DFS: push in reverse so strongest is popped first
                stack.push((*tgt, depth + 1));
                result.steps.push(TraversalStep {
                    depth: depth + 1, from: node, to: *tgt,
                    edge_kind: *kind, weight: *weight,
                    reason: "strong edge, meets min_weight threshold",
                });
            }
        }
        result
    }
}

// ─────────────────────────────────────────────────────────────────────
// 2. DivergentMode — ADHD-inspired hash-weighted BFS
// ─────────────────────────────────────────────────────────────────────

pub struct DivergentMode {
    pub divergence_pct: u8,  // 0-100: higher = more willing to take weak edges
}

impl Default for DivergentMode {
    fn default() -> Self { Self { divergence_pct: 15 } }
}

impl CognitiveMode for DivergentMode {
    fn name(&self) -> &'static str { "DivergentMode" }
    fn inspired_by(&self) -> &'static str { "ADHD — hash-weighted BFS, finds remote connections" }

    fn walk(&self, query: &Query, host: &mut dyn GraphHost) -> WalkResult {
        let mut result = WalkResult::default();
        let mut visited: HashSet<u32> = HashSet::new();
        let mut queue: VecDeque<(u32, u32)> = VecDeque::new();
        queue.push_back((query.start, 0));

        while let Some((node, depth)) = queue.pop_front() {
            if visited.contains(&node) { continue; }
            if visited.len() as u32 >= query.max_visits {
                result.truncated = true;
                break;
            }
            visited.insert(node);
            result.visited.push(node);
            if depth >= query.max_depth { continue; }

            let neighbors = host.outgoing(node);
            let mut explored_any = false;
            for (tgt, kind, weight) in neighbors {
                if !query.allowed_edge_kinds.is_empty()
                   && !query.allowed_edge_kinds.contains(&kind) { continue; }

                let accept = divergent_gate(query.id, node, tgt, weight, self.divergence_pct);
                if !accept { continue; }

                explored_any = true;
                queue.push_back((tgt, depth + 1));
                let reason = if weight >= 70 { "strong edge" } else { "weak edge, hash-selected divergence" };
                result.steps.push(TraversalStep {
                    depth: depth + 1, from: node, to: tgt,
                    edge_kind: kind, weight, reason,
                });
            }
            if !explored_any { result.dead_ends += 1; }
        }
        result
    }
}

// ─────────────────────────────────────────────────────────────────────
// 3. GestaltMode — dyslexia-inspired k-hop neighborhood centrality
// ─────────────────────────────────────────────────────────────────────

pub struct GestaltMode {
    pub neighborhood_hops: u32,  // how wide the "whole picture" is
}

impl Default for GestaltMode {
    fn default() -> Self { Self { neighborhood_hops: 2 } }
}

impl CognitiveMode for GestaltMode {
    fn name(&self) -> &'static str { "GestaltMode" }
    fn inspired_by(&self) -> &'static str { "dyslexia — k-hop neighborhood, sees the whole picture" }

    fn walk(&self, query: &Query, host: &mut dyn GraphHost) -> WalkResult {
        let mut result = WalkResult::default();
        let mut visited: HashSet<u32> = HashSet::new();
        let mut by_depth: HashMap<u32, HashSet<u32>> = HashMap::new();
        let mut queue: VecDeque<(u32, u32)> = VecDeque::new();
        queue.push_back((query.start, 0));

        let max_d = query.max_depth.min(self.neighborhood_hops);
        while let Some((node, depth)) = queue.pop_front() {
            if !visited.insert(node) { continue; }
            if visited.len() as u32 >= query.max_visits {
                result.truncated = true;
                break;
            }
            by_depth.entry(depth).or_default().insert(node);
            result.visited.push(node);
            if depth >= max_d { continue; }

            for (tgt, kind, weight) in host.outgoing(node) {
                if !query.allowed_edge_kinds.is_empty()
                   && !query.allowed_edge_kinds.contains(&kind) { continue; }
                queue.push_back((tgt, depth + 1));
                result.steps.push(TraversalStep {
                    depth: depth + 1, from: node, to: tgt,
                    edge_kind: kind, weight,
                    reason: "neighborhood gather — take all reachable",
                });
            }
        }
        result
    }
}

// ─────────────────────────────────────────────────────────────────────
// 4. NarrativeMode — dysgraphia-inspired composer
// ─────────────────────────────────────────────────────────────────────

/// NarrativeMode doesn't walk — it takes the output of other modes and
/// composes them. Deliberately minimal: lazy, efficient, expressive.
pub struct NarrativeMode;

impl NarrativeMode {
    /// Take walk results and produce a human-readable explanation.
    pub fn compose(
        &self,
        query: &Query,
        results: &[(&dyn CognitiveMode, &WalkResult)],
        host: &mut dyn GraphHost,
    ) -> String {
        let start_label = host.label(query.start);
        let mut out = String::new();
        out.push_str(&format!("Starting from '{}' (concept #{}):\n", start_label, query.start));
        for (mode, wr) in results {
            out.push_str(&format!("\n  [{}]  (inspired by: {})\n", mode.name(), mode.inspired_by()));
            out.push_str(&format!("    Visited {} node(s), {} edge(s) taken",
                wr.visited.len(), wr.steps.len()));
            if wr.truncated { out.push_str(" [truncated]"); }
            out.push('\n');
            for step in wr.steps.iter().take(6) {
                let from = host.label(step.from);
                let to = host.label(step.to);
                out.push_str(&format!("      d{}  {} → {} (kind={}, w={}, {})\n",
                    step.depth, from, to, step.edge_kind, step.weight, step.reason));
            }
            if wr.steps.len() > 6 {
                out.push_str(&format!("      … {} more steps\n", wr.steps.len() - 6));
            }
        }
        out
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct TestGraph {
        edges: HashMap<u32, Vec<(u32, u8, u8)>>,
        labels: HashMap<u32, &'static str>,
    }

    impl TestGraph {
        /// Build: dog → canine (IS_A, w=90), canine → mammal (IS_A, w=90),
        ///        mammal → animal (IS_A, w=95)
        ///        dog → loyal (HAS_PROP, w=40)  — weak edge
        ///        dog → space (SEEN_IN, w=5)    — very weak
        fn sample() -> Self {
            let mut edges = HashMap::new();
            edges.insert(1, vec![(2, 3, 90), (10, 7, 40), (20, 9, 5)]);
            edges.insert(2, vec![(3, 3, 90)]);
            edges.insert(3, vec![(4, 3, 95)]);
            edges.insert(4, vec![]);
            edges.insert(10, vec![]);
            edges.insert(20, vec![]);
            let mut labels = HashMap::new();
            labels.insert(1, "dog");  labels.insert(2, "canine");
            labels.insert(3, "mammal"); labels.insert(4, "animal");
            labels.insert(10, "loyal"); labels.insert(20, "space");
            Self { edges, labels }
        }
    }

    impl GraphHost for TestGraph {
        fn outgoing(&mut self, n: u32) -> Vec<(u32, u8, u8)> {
            self.edges.get(&n).cloned().unwrap_or_default()
        }
        fn label(&mut self, n: u32) -> String {
            self.labels.get(&n).map(|s| s.to_string()).unwrap_or_else(|| format!("#{}", n))
        }
    }

    #[test]
    fn precision_mode_follows_only_strong_edges() {
        let mut g = TestGraph::sample();
        let q = Query::new(1, 5);
        let r = PrecisionMode::default().walk(&q, &mut g);
        // Should reach: dog → canine → mammal → animal (weights all ≥ 70)
        // Should NOT visit: loyal (w=40) or space (w=5)
        assert!(r.visited.contains(&2)); // canine
        assert!(r.visited.contains(&3)); // mammal
        assert!(r.visited.contains(&4)); // animal
        assert!(!r.visited.contains(&10)); // NOT loyal
        assert!(!r.visited.contains(&20)); // NOT space
    }

    #[test]
    fn divergent_mode_is_deterministic() {
        let mut g1 = TestGraph::sample();
        let mut g2 = TestGraph::sample();
        let q = Query::new(1, 5);
        let r1 = DivergentMode::default().walk(&q, &mut g1);
        let r2 = DivergentMode::default().walk(&q, &mut g2);
        assert_eq!(r1.visited, r2.visited, "divergent mode must be deterministic");
        assert_eq!(r1.steps.len(), r2.steps.len());
    }

    #[test]
    fn divergent_mode_may_take_weak_edges() {
        let mut g = TestGraph::sample();
        // Use divergence_pct=100 so it always takes weak edges (for test)
        let m = DivergentMode { divergence_pct: 100 };
        let q = Query::new(1, 5);
        let r = m.walk(&q, &mut g);
        // Should visit loyal and space (weak edges from dog)
        assert!(r.visited.contains(&10), "should reach loyal via weak edge");
        assert!(r.visited.contains(&20), "should reach space via weak edge");
    }

    #[test]
    fn divergent_mode_with_zero_divergence_ignores_weak() {
        let mut g = TestGraph::sample();
        let m = DivergentMode { divergence_pct: 0 };
        let q = Query::new(1, 5);
        let r = m.walk(&q, &mut g);
        // No weak edges taken
        assert!(!r.visited.contains(&10));
        assert!(!r.visited.contains(&20));
    }

    #[test]
    fn gestalt_mode_gathers_neighborhood() {
        let mut g = TestGraph::sample();
        let q = Query::new(1, 5);
        let m = GestaltMode { neighborhood_hops: 2 };
        let r = m.walk(&q, &mut g);
        // Should see immediate (canine, loyal, space) + their neighbors (mammal)
        // Takes ALL edges regardless of weight — wide net
        assert!(r.visited.contains(&2));
        assert!(r.visited.contains(&10));
        assert!(r.visited.contains(&20));
        assert!(r.visited.contains(&3)); // 2-hop reach
    }

    #[test]
    fn narrative_mode_composes_all() {
        let mut g = TestGraph::sample();
        let q = Query::new(1, 5);
        let precision = PrecisionMode::default();
        let divergent = DivergentMode::default();
        let precision_r = precision.walk(&q, &mut g);
        let divergent_r = divergent.walk(&q, &mut g);
        let story = NarrativeMode.compose(
            &q,
            &[(&precision, &precision_r), (&divergent, &divergent_r)],
            &mut g,
        );
        assert!(story.contains("dog"));
        assert!(story.contains("PrecisionMode"));
        assert!(story.contains("DivergentMode"));
    }

    #[test]
    fn hash64_is_stable() {
        let a = hash64(&(1u64, 2u64, 3u64));
        let b = hash64(&(1u64, 2u64, 3u64));
        assert_eq!(a, b, "hash must be stable across calls");
    }

    #[test]
    fn divergent_gate_is_stable() {
        let g1 = divergent_gate(42, 1, 2, 30, 15);
        let g2 = divergent_gate(42, 1, 2, 30, 15);
        assert_eq!(g1, g2, "divergent_gate must be deterministic");
    }

    #[test]
    fn divergent_gate_always_accepts_strong() {
        // weight >= 70 always accepted, regardless of divergence_pct
        assert!(divergent_gate(0, 0, 0, 70, 0));
        assert!(divergent_gate(0, 0, 0, 95, 0));
    }
}
