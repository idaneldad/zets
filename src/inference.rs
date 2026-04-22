//! Inference by analogy — the "capybara has 4 legs" problem.
//!
//! Given: `poodle is_a dog`, `dog is_a mammal`, `mammal has_attribute four_legs`.
//! Ask:   what attributes does `poodle` have?
//! Answer: `four_legs` — inferred by walking `is_a` upward and collecting
//!         `has_attribute` edges from ancestors.
//!
//! Key discipline:
//!   1. Inferred attributes are returned as `Provenance::Hypothesis` with
//!      a derivation chain — never silently mixed with asserted facts.
//!   2. Max 3 hops of `is_a` by default — prevents runaway inference.
//!   3. Never chain through Hypothesis edges — prevents hypothesis creep
//!      (inferring on top of inferences).
//!   4. Confidence decays with distance from source (200 → 160 → 128).
//!
//! This does NOT persist anything. It's ephemeral per-query. If you want
//! to remember inferences, the caller decides whether to tag them in a
//! ProvenanceLog using the InferredAttribute data.
//!
//! Why module instead of walk-mode? Because it's a pure reader over
//! AtomStore — no side effects. Callers can compose it with smart_walk
//! or use it standalone.

use crate::atoms::{AtomId, AtomStore};
use crate::learning_layer::{EdgeKey, Provenance, ProvenanceLog};
use crate::relations;

/// Default max is_a hops when inferring.
pub const DEFAULT_MAX_HOPS: u32 = 3;

/// Starting confidence for a 1-hop inference (is_a direct parent).
pub const BASE_CONFIDENCE: u8 = 200;

/// Confidence decay factor per hop (0.80 = 20% decay).
/// Hop 1 → 200, hop 2 → 160, hop 3 → 128.
pub const DECAY_PER_HOP: f32 = 0.80;

/// A single inferred attribute with its full derivation chain.
#[derive(Debug, Clone)]
pub struct InferredAttribute {
    /// The attribute atom (e.g., the atom for "four_legs").
    pub attribute: AtomId,
    /// Which ancestor of `subject` owned this attribute.
    /// Example: for poodle, this might be the atom for "mammal".
    pub source_ancestor: AtomId,
    /// The chain of is_a edges from subject up to source_ancestor.
    /// `chain[0].from == subject`, `chain.last().to == source_ancestor`.
    pub is_a_chain: Vec<EdgeKey>,
    /// The has_attribute edge at the source_ancestor itself.
    pub attribute_edge: EdgeKey,
    /// Confidence 0-255. Decays with hops.
    pub confidence: u8,
    /// True if all edges along the chain are non-Hypothesis.
    /// If false, this inference chained through a Hypothesis and is
    /// REJECTED by infer_attributes (we don't emit hypothesis-on-hypothesis).
    pub is_grounded: bool,
}

impl InferredAttribute {
    /// Number of is_a hops from subject to the source_ancestor.
    pub fn hops(&self) -> u32 {
        self.is_a_chain.len() as u32
    }

    /// Human-readable trace like "poodle → dog → mammal + has_attribute → four_legs".
    pub fn trace(&self, store: &AtomStore) -> String {
        let mut parts: Vec<String> = Vec::new();
        for (i, edge) in self.is_a_chain.iter().enumerate() {
            if i == 0 {
                parts.push(atom_label(store, edge.from));
            }
            parts.push(atom_label(store, edge.to));
        }
        let attr_label = atom_label(store, self.attribute);
        format!(
            "{}  +has_attribute→ {}  (conf {})",
            parts.join(" →is_a→ "),
            attr_label,
            self.confidence
        )
    }

    /// Compact provenance string: list of edge keys.
    pub fn provenance(&self) -> String {
        let chain: Vec<String> = self
            .is_a_chain
            .iter()
            .map(|e| format!("E({},{},{})", e.from, e.to, e.relation))
            .collect();
        format!(
            "Hypothesis(DerivedFrom: is_a=[{}], has_attribute=E({},{},{}))",
            chain.join(","),
            self.attribute_edge.from,
            self.attribute_edge.to,
            self.attribute_edge.relation
        )
    }
}

fn atom_label(store: &AtomStore, id: AtomId) -> String {
    store
        .get(id)
        .map(|a| String::from_utf8_lossy(&a.data).to_string())
        .unwrap_or_else(|| format!("atom#{}", id))
}

/// Infer attributes of `subject` by walking is_a edges upward and collecting
/// has_attribute edges from each ancestor.
///
/// Returns inferences in order of confidence (highest first).
///
/// If `prov` is Some, edges tagged as Hypothesis are SKIPPED during the walk
/// (we don't infer on top of inferences).
pub fn infer_attributes(
    store: &AtomStore,
    subject: AtomId,
    max_hops: u32,
    prov: Option<&ProvenanceLog>,
) -> Vec<InferredAttribute> {
    let is_a_code = match relations::by_name("is_a") {
        Some(r) => r.code,
        None => return Vec::new(),
    };
    let has_attr_code = match relations::by_name("has_attribute") {
        Some(r) => r.code,
        None => return Vec::new(),
    };

    // First: is the subject itself known? If not, return nothing.
    if store.get(subject).is_none() {
        return Vec::new();
    }

    let mut results: Vec<InferredAttribute> = Vec::new();

    // BFS up the is_a chain. Track (current_atom, path_to_here).
    // We use BFS to get shortest chain to each ancestor.
    let mut visited: std::collections::HashSet<AtomId> = std::collections::HashSet::new();
    visited.insert(subject);

    // Queue items: (atom, path_of_is_a_edges_so_far)
    let mut queue: std::collections::VecDeque<(AtomId, Vec<EdgeKey>)> =
        std::collections::VecDeque::new();
    queue.push_back((subject, Vec::new()));

    while let Some((current, path)) = queue.pop_front() {
        let hops = path.len() as u32;
        if hops > max_hops {
            continue;
        }

        // 1. Collect has_attribute edges at this node (if hops > 0 — don't
        //    re-emit the subject's own attributes; those aren't inferred).
        if hops > 0 {
            let confidence = compute_confidence(hops);
            for edge in store.outgoing(current) {
                if edge.relation != has_attr_code {
                    continue;
                }
                let attr_edge_key = EdgeKey {
                    from: current,
                    to: edge.to,
                    relation: has_attr_code,
                };

                // Filter: skip this has_attribute if it's itself Hypothesis.
                if let Some(log) = prov {
                    if let Some(rec) = log.get(&attr_edge_key) {
                        if rec.provenance == Provenance::Hypothesis {
                            continue;
                        }
                    }
                }

                results.push(InferredAttribute {
                    attribute: edge.to,
                    source_ancestor: current,
                    is_a_chain: path.clone(),
                    attribute_edge: attr_edge_key,
                    confidence,
                    is_grounded: true,
                });
            }
        }

        // 2. Expand via is_a edges to the next level.
        if hops < max_hops {
            for edge in store.outgoing(current) {
                if edge.relation != is_a_code {
                    continue;
                }
                if visited.contains(&edge.to) {
                    continue;
                }

                // Filter: skip this is_a if it's itself Hypothesis.
                let is_a_key = EdgeKey {
                    from: current,
                    to: edge.to,
                    relation: is_a_code,
                };
                if let Some(log) = prov {
                    if let Some(rec) = log.get(&is_a_key) {
                        if rec.provenance == Provenance::Hypothesis {
                            continue;
                        }
                    }
                }

                visited.insert(edge.to);
                let mut new_path = path.clone();
                new_path.push(is_a_key);
                queue.push_back((edge.to, new_path));
            }
        }
    }

    // Sort by confidence descending, then by hops ascending (closer ancestors first).
    results.sort_by(|a, b| {
        b.confidence
            .cmp(&a.confidence)
            .then(a.hops().cmp(&b.hops()))
    });
    results
}

fn compute_confidence(hops: u32) -> u8 {
    // confidence = BASE * DECAY^(hops-1)
    if hops == 0 {
        return BASE_CONFIDENCE;
    }
    let conf = BASE_CONFIDENCE as f32 * DECAY_PER_HOP.powi(hops as i32 - 1);
    conf.round().clamp(0.0, 255.0) as u8
}

// ════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::{AtomKind, AtomStore};
    use crate::learning_layer::{ProvenanceLog, ProvenanceRecord};

    fn setup_mammal_graph() -> (AtomStore, u32, u32, u32, u32, u32) {
        // poodle is_a dog, dog is_a mammal, mammal has_attribute four_legs
        // mammal has_attribute milk
        let mut store = AtomStore::new();
        let poodle = store.put(AtomKind::Concept, b"poodle".to_vec());
        let dog = store.put(AtomKind::Concept, b"dog".to_vec());
        let mammal = store.put(AtomKind::Concept, b"mammal".to_vec());
        let four_legs = store.put(AtomKind::Concept, b"four_legs".to_vec());
        let milk = store.put(AtomKind::Concept, b"milk".to_vec());

        let is_a = relations::by_name("is_a").unwrap().code;
        let has_attr = relations::by_name("has_attribute").unwrap().code;

        store.link(poodle, dog, is_a, 240, 0);
        store.link(dog, mammal, is_a, 240, 0);
        store.link(mammal, four_legs, has_attr, 240, 0);
        store.link(mammal, milk, has_attr, 240, 0);

        (store, poodle, dog, mammal, four_legs, milk)
    }

    #[test]
    fn poodle_infers_four_legs() {
        let (store, poodle, _dog, mammal, four_legs, _milk) = setup_mammal_graph();
        let results = infer_attributes(&store, poodle, 3, None);

        // Should find both four_legs and milk, via the 2-hop chain poodle→dog→mammal.
        assert_eq!(results.len(), 2, "expected 2 attributes, got {:?}", results.len());

        for inf in &results {
            assert_eq!(inf.source_ancestor, mammal);
            assert_eq!(inf.hops(), 2);
            // 2-hop confidence: 200 * 0.80^1 = 160
            assert_eq!(inf.confidence, 160);
            assert!(inf.is_grounded);
        }

        let attrs: std::collections::HashSet<AtomId> =
            results.iter().map(|r| r.attribute).collect();
        assert!(attrs.contains(&four_legs));
    }

    #[test]
    fn confidence_decays_with_hops() {
        assert_eq!(compute_confidence(0), 200);
        assert_eq!(compute_confidence(1), 200);
        assert_eq!(compute_confidence(2), 160);
        assert_eq!(compute_confidence(3), 128);
    }

    #[test]
    fn max_hops_bounds_the_walk() {
        let (store, poodle, _dog, _mammal, _four_legs, _milk) = setup_mammal_graph();

        // With max_hops=1, we only reach `dog` — which has NO has_attribute.
        // So we get 0 inferences.
        let results = infer_attributes(&store, poodle, 1, None);
        assert_eq!(results.len(), 0);

        // With max_hops=2, we reach `mammal` and get 2 attributes.
        let results = infer_attributes(&store, poodle, 2, None);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn never_chains_through_hypothesis() {
        // Set up: poodle is_a dog (Hypothesis), dog is_a mammal, mammal has_attribute X.
        // Because poodle→dog is Hypothesis, we must NOT use it → 0 inferences.
        let mut store = AtomStore::new();
        let poodle = store.put(AtomKind::Concept, b"poodle".to_vec());
        let dog = store.put(AtomKind::Concept, b"dog".to_vec());
        let mammal = store.put(AtomKind::Concept, b"mammal".to_vec());
        let x = store.put(AtomKind::Concept, b"X".to_vec());

        let is_a = relations::by_name("is_a").unwrap().code;
        let has_attr = relations::by_name("has_attribute").unwrap().code;

        store.link(poodle, dog, is_a, 240, 0);
        store.link(dog, mammal, is_a, 240, 0);
        store.link(mammal, x, has_attr, 240, 0);

        let mut prov = ProvenanceLog::new();
        prov.tag(
            EdgeKey { from: poodle, to: dog, relation: is_a },
            ProvenanceRecord::hypothesis(),
        );

        let results = infer_attributes(&store, poodle, 3, Some(&prov));
        assert_eq!(results.len(), 0,
            "must not chain through Hypothesis is_a edge");

        // Without prov filter, we would find X.
        let results = infer_attributes(&store, poodle, 3, None);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn skips_hypothesis_has_attribute() {
        let (store, poodle, _dog, mammal, four_legs, milk) = setup_mammal_graph();

        // Tag `mammal has_attribute four_legs` as Hypothesis.
        let mut prov = ProvenanceLog::new();
        prov.tag(
            EdgeKey { from: mammal, to: four_legs, relation: relations::by_name("has_attribute").unwrap().code },
            ProvenanceRecord::hypothesis(),
        );

        let results = infer_attributes(&store, poodle, 3, Some(&prov));
        assert_eq!(results.len(), 1, "four_legs should be filtered out");
        assert_eq!(results[0].attribute, milk);
    }

    #[test]
    fn unknown_subject_returns_empty() {
        let (store, _, _, _, _, _) = setup_mammal_graph();
        let results = infer_attributes(&store, 99999, 3, None);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn results_sorted_by_confidence_then_hops() {
        // Build: A is_a B, B is_a C, C has_attribute X.
        //        A has_attribute Y (direct, not inferred).
        //        B has_attribute Z.
        // From A, we should infer Z (1 hop) then X (2 hops).
        let mut store = AtomStore::new();
        let a = store.put(AtomKind::Concept, b"A".to_vec());
        let b = store.put(AtomKind::Concept, b"B".to_vec());
        let c = store.put(AtomKind::Concept, b"C".to_vec());
        let x = store.put(AtomKind::Concept, b"X".to_vec());
        let z = store.put(AtomKind::Concept, b"Z".to_vec());

        let is_a = relations::by_name("is_a").unwrap().code;
        let has_attr = relations::by_name("has_attribute").unwrap().code;

        store.link(a, b, is_a, 240, 0);
        store.link(b, c, is_a, 240, 0);
        store.link(b, z, has_attr, 240, 0);
        store.link(c, x, has_attr, 240, 0);

        let results = infer_attributes(&store, a, 3, None);
        assert_eq!(results.len(), 2);
        // First (higher conf) should be Z from B (1 hop).
        assert_eq!(results[0].attribute, z);
        assert_eq!(results[0].hops(), 1);
        // confidence at hops=1 is 200 (no decay applied for first hop)
        assert_eq!(results[0].confidence, 200);
        // Second is X from C (2 hops, confidence 160).
        assert_eq!(results[1].attribute, x);
        assert_eq!(results[1].hops(), 2);
        assert_eq!(results[1].confidence, 160);
    }
}
