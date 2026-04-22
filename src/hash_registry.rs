//! Hash registry — cross-graph content-addressable lookup.
//!
//! ZETS has two data systems (AtomStore + PieceGraph). Rather than merge them,
//! this module lets them SHARE a hash space. Every atom (regardless of graph)
//! is identified by its `content_hash: u64` (FNV-1a). The registry maps:
//!
//!     content_hash → Vec<GraphRef>
//!
//! where each GraphRef says "System X has this hash as local_id Y". Queries
//! can aggregate across systems; duplication is prevented at insertion time.
//!
//! This is exactly Idan's insight (22 Apr 2026): since we already have a
//! deterministic hash function, we don't need to merge the graphs. Each graph
//! contributes its knowledge; the hash is the bridge.
//!
//! Cross-graph voting:
//!   - If both graphs contain hash H, they MAY vote on label/kind.
//!   - Mismatch = schema drift, flagged for resolution.
//!   - Agreement = confidence boost (both systems independently computed same).
//!
//! This also answers "how do we add language #17?" — new atoms contribute
//! to the shared hash space; anything already known (by hash) is referenced,
//! not copied.

use std::collections::HashMap;

/// Which graph a reference belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphKind {
    /// System A — AtomStore (atoms.rs, atom_persist.rs).
    AtomStore,
    /// System B — PieceGraph (piece_graph.rs, pack.rs, mmap_*.rs).
    PieceGraph,
    /// Future: per-client / per-persona graphs.
    Personal(u16),
    /// Future: per-language overlay (e.g., en-GB overlay on en).
    DialectOverlay(u16),
}

/// A reference to a concept inside one graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphRef {
    pub graph: GraphKind,
    /// Local id inside that graph (AtomId for A, ConceptId for B, etc.)
    pub local_id: u32,
    /// How confident is this graph that its local entry represents the hash?
    /// 0-255. Higher = more authoritative.
    pub confidence: u8,
}

/// Cross-graph hash registry.
///
/// For each content_hash, tracks which graphs contain it and with what
/// confidence. Never stores the content itself — that stays in the owning
/// graph. This keeps memory footprint tiny (just the hash map).
#[derive(Debug, Default, Clone)]
pub struct HashRegistry {
    entries: HashMap<u64, Vec<GraphRef>>,
    /// Count of known hashes — for quick stats.
    total_hashes: usize,
    /// Count of hashes that appear in 2+ graphs (cross-referenced).
    shared_hashes: usize,
}

impl HashRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register that a graph contains a given content_hash.
    /// Idempotent — registering the same (hash, graph, local_id) twice is a no-op.
    pub fn register(&mut self, content_hash: u64, gref: GraphRef) {
        let entry = self.entries.entry(content_hash).or_insert_with(Vec::new);
        // Prevent duplicates within same graph for same local_id
        for existing in entry.iter() {
            if existing.graph == gref.graph && existing.local_id == gref.local_id {
                return;
            }
        }
        let was_single = entry.len() == 1;
        entry.push(gref);
        if entry.len() == 1 {
            self.total_hashes += 1;
        } else if was_single {
            // Just became shared
            self.shared_hashes += 1;
        }
    }

    /// Get all graph refs for a content_hash.
    pub fn lookup(&self, content_hash: u64) -> Option<&[GraphRef]> {
        self.entries.get(&content_hash).map(|v| v.as_slice())
    }

    /// How many graphs contain this hash?
    pub fn graphs_containing(&self, content_hash: u64) -> usize {
        self.entries.get(&content_hash).map(|v| v.len()).unwrap_or(0)
    }

    /// Cross-vote: check whether two specific graphs AGREE that they both
    /// know this hash. If both register it, agreement = true.
    pub fn cross_vote(
        &self,
        content_hash: u64,
        a: GraphKind,
        b: GraphKind,
    ) -> CrossVote {
        let refs = match self.entries.get(&content_hash) {
            Some(r) => r,
            None => return CrossVote::Unknown,
        };
        let a_has = refs.iter().any(|r| r.graph == a);
        let b_has = refs.iter().any(|r| r.graph == b);
        match (a_has, b_has) {
            (true, true) => CrossVote::Agreement,
            (true, false) => CrossVote::OnlyA,
            (false, true) => CrossVote::OnlyB,
            (false, false) => CrossVote::Unknown,
        }
    }

    /// Find all content_hashes known by one graph but not another.
    /// Used to measure "what does A know that B doesn't?" — the learning gap.
    pub fn gap(&self, has: GraphKind, lacks: GraphKind) -> Vec<u64> {
        let mut gap = Vec::new();
        for (&hash, refs) in self.entries.iter() {
            let has_present = refs.iter().any(|r| r.graph == has);
            let lacks_present = refs.iter().any(|r| r.graph == lacks);
            if has_present && !lacks_present {
                gap.push(hash);
            }
        }
        gap
    }

    pub fn total_hashes(&self) -> usize { self.total_hashes }
    pub fn shared_hashes(&self) -> usize { self.shared_hashes }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}

/// Result of a cross-graph vote on whether two graphs know the same hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossVote {
    /// Both graphs have this hash — they independently confirm the concept exists.
    Agreement,
    /// Only graph A has it.
    OnlyA,
    /// Only graph B has it.
    OnlyB,
    /// Neither has it (or the hash is not registered).
    Unknown,
}

// ════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::content_hash;

    #[test]
    fn same_content_hashes_equal() {
        let h1 = content_hash(b"cat");
        let h2 = content_hash(b"cat");
        assert_eq!(h1, h2);
    }

    #[test]
    fn dedup_within_same_graph() {
        let mut reg = HashRegistry::new();
        let h = content_hash(b"cat");
        reg.register(h, GraphRef {
            graph: GraphKind::AtomStore,
            local_id: 100,
            confidence: 200,
        });
        // Registering same (graph, local_id) twice — should be no-op
        reg.register(h, GraphRef {
            graph: GraphKind::AtomStore,
            local_id: 100,
            confidence: 200,
        });
        assert_eq!(reg.graphs_containing(h), 1);
    }

    #[test]
    fn cross_graph_agreement() {
        let mut reg = HashRegistry::new();
        let h = content_hash(b"cat");
        reg.register(h, GraphRef { graph: GraphKind::AtomStore, local_id: 1, confidence: 200 });
        reg.register(h, GraphRef { graph: GraphKind::PieceGraph, local_id: 99, confidence: 220 });

        assert_eq!(reg.graphs_containing(h), 2);
        assert_eq!(
            reg.cross_vote(h, GraphKind::AtomStore, GraphKind::PieceGraph),
            CrossVote::Agreement
        );
        assert_eq!(reg.shared_hashes(), 1);
    }

    #[test]
    fn gap_detects_what_one_knows_other_doesnt() {
        let mut reg = HashRegistry::new();
        let h_cat = content_hash(b"cat");
        let h_fish = content_hash(b"fish");

        // AtomStore has both
        reg.register(h_cat, GraphRef { graph: GraphKind::AtomStore, local_id: 1, confidence: 200 });
        reg.register(h_fish, GraphRef { graph: GraphKind::AtomStore, local_id: 2, confidence: 200 });
        // PieceGraph has only cat
        reg.register(h_cat, GraphRef { graph: GraphKind::PieceGraph, local_id: 99, confidence: 200 });

        let gap = reg.gap(GraphKind::AtomStore, GraphKind::PieceGraph);
        assert_eq!(gap.len(), 1);
        assert_eq!(gap[0], h_fish);
    }

    #[test]
    fn multiple_personal_graphs_coexist() {
        let mut reg = HashRegistry::new();
        let h = content_hash(b"pizza");
        reg.register(h, GraphRef { graph: GraphKind::AtomStore, local_id: 1, confidence: 200 });
        reg.register(h, GraphRef { graph: GraphKind::Personal(3251), local_id: 5, confidence: 180 });
        reg.register(h, GraphRef { graph: GraphKind::Personal(3265), local_id: 12, confidence: 150 });
        assert_eq!(reg.graphs_containing(h), 3);
    }

    #[test]
    fn vote_on_absent_hash() {
        let reg = HashRegistry::new();
        let h = content_hash(b"ghost_concept");
        assert_eq!(reg.cross_vote(h, GraphKind::AtomStore, GraphKind::PieceGraph),
                   CrossVote::Unknown);
    }
}
