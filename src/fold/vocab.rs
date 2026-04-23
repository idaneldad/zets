//! Vocabulary — the mutable dictionary of folded tokens.
//!
//! Structure:
//! - Leaves: raw byte sequences → FoldId
//! - Merges: (left_id, right_id) → FoldId (parent)
//! - Reverse: FoldId → (leaf_bytes | merge_pair) for unfolding
//!
//! Hierarchical vocab support (Gemini's recommendation):
//! - `frozen_core` — universal tokens shipped with ZETS (e.g., common words)
//! - `mutable_domain` — per-session or per-topic additions (growable)
//!
//! This avoids expensive re-encoding when new domain data arrives.

use ahash::AHashMap;
use super::FoldId;
use super::merkle::{hash_leaf, hash_merge};

/// A folded token's content — either a leaf with raw bytes or a merge of two children.
#[derive(Debug, Clone)]
pub enum TokenContent {
    /// A raw byte sequence (the original, normalized).
    Leaf(Vec<u8>),
    /// A merge of two children by their FoldIds.
    Merge(FoldId, FoldId),
}

impl TokenContent {
    pub fn is_leaf(&self) -> bool {
        matches!(self, TokenContent::Leaf(_))
    }

    pub fn leaf_bytes(&self) -> Option<&[u8]> {
        match self {
            TokenContent::Leaf(b) => Some(b),
            _ => None,
        }
    }

    pub fn children(&self) -> Option<(FoldId, FoldId)> {
        match self {
            TokenContent::Merge(l, r) => Some((*l, *r)),
            _ => None,
        }
    }

    /// Depth of this token (0 for leaves, else max(left, right) + 1).
    /// Requires a vocab to look up children's depths. See `Vocab::depth_of`.
    pub fn on_disk_size(&self) -> usize {
        match self {
            TokenContent::Leaf(b) => b.len() + 1,       // +1 for kind tag
            TokenContent::Merge(_, _) => 16 + 16 + 1,   // two FoldIds + tag
        }
    }
}

/// A mutable vocabulary of folded tokens.
///
/// This is the full-optimized implementation — uses ahash (AES-NI) for fast
/// lookup, separates frozen/mutable layers, tracks usage counts for tiering.
#[derive(Debug, Default)]
pub struct Vocab {
    /// Frozen core — shipped with ZETS, never removed, read-only after load.
    frozen: AHashMap<FoldId, TokenContent>,

    /// Mutable domain — learned from current content, can grow or be pruned.
    mutable: AHashMap<FoldId, TokenContent>,

    /// Access counter — used by `tier` module for hot/cold classification.
    /// Stored separately to keep the main maps cache-friendly.
    access_counts: AHashMap<FoldId, u32>,

    /// Depth cache for quick walk limiting.
    depth_cache: AHashMap<FoldId, u8>,

    /// Leaf content reverse-lookup (content bytes → FoldId) for dedup during BPE.
    leaf_index: AHashMap<Vec<u8>, FoldId>,

    /// Merge reverse-lookup ((left, right) → FoldId) for dedup during BPE.
    merge_index: AHashMap<(FoldId, FoldId), FoldId>,
}

impl Vocab {
    /// Create a new empty vocab.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a leaf (raw content). If already present, returns existing FoldId (hash consing).
    pub fn get_or_insert_leaf(&mut self, content: &[u8]) -> FoldId {
        if let Some(&id) = self.leaf_index.get(content) {
            return id;
        }
        let id = hash_leaf(content);
        self.mutable.insert(id, TokenContent::Leaf(content.to_vec()));
        self.leaf_index.insert(content.to_vec(), id);
        self.depth_cache.insert(id, 0);
        id
    }

    /// Insert a merge node. If (left, right) already merged, returns existing FoldId.
    pub fn get_or_insert_merge(&mut self, left: FoldId, right: FoldId) -> FoldId {
        if let Some(&id) = self.merge_index.get(&(left, right)) {
            return id;
        }
        let id = hash_merge(left, right);
        self.mutable.insert(id, TokenContent::Merge(left, right));
        self.merge_index.insert((left, right), id);

        // Cache depth: 1 + max(left_depth, right_depth)
        let d = 1 + self.depth_of(left).max(self.depth_of(right));
        self.depth_cache.insert(id, d);

        id
    }

    /// Look up a token by FoldId. Checks mutable first, then frozen.
    pub fn lookup(&self, id: FoldId) -> Option<&TokenContent> {
        self.mutable.get(&id).or_else(|| self.frozen.get(&id))
    }

    /// Look up and record access (for tier stats).
    pub fn lookup_tracked(&mut self, id: FoldId) -> Option<&TokenContent> {
        if self.mutable.contains_key(&id) || self.frozen.contains_key(&id) {
            *self.access_counts.entry(id).or_insert(0) += 1;
        }
        self.mutable.get(&id).or_else(|| self.frozen.get(&id))
    }

    /// Depth of a token (0 for leaves, else max child depth + 1).
    pub fn depth_of(&self, id: FoldId) -> u8 {
        *self.depth_cache.get(&id).unwrap_or(&0)
    }

    /// Access count for tier classification (hot if in top 10%).
    pub fn access_count(&self, id: FoldId) -> u32 {
        *self.access_counts.get(&id).unwrap_or(&0)
    }

    /// Total number of tokens (leaves + merges, mutable + frozen).
    pub fn len(&self) -> usize {
        self.mutable.len() + self.frozen.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// How many tokens are in the mutable (growable) layer.
    pub fn mutable_size(&self) -> usize {
        self.mutable.len()
    }

    /// How many tokens are frozen (read-only base).
    pub fn frozen_size(&self) -> usize {
        self.frozen.len()
    }

    /// Promote entries from mutable → frozen (typically after BPE training converges).
    /// Frees the mutable-specific indices for those entries to save memory.
    pub fn freeze_mutable(&mut self) {
        let drained: Vec<_> = self.mutable.drain().collect();
        for (id, content) in drained {
            self.frozen.insert(id, content);
        }
        // Keep the reverse indices — they work for both layers.
    }

    /// Total bytes stored (approximate). Useful for compression ratio tracking.
    pub fn total_bytes(&self) -> u64 {
        let mut b: u64 = 0;
        for content in self.mutable.values().chain(self.frozen.values()) {
            b += content.on_disk_size() as u64 + 16;  // +16 for the key (FoldId)
        }
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaf_dedup_same_content() {
        let mut v = Vocab::new();
        let a = v.get_or_insert_leaf(b"hello");
        let b = v.get_or_insert_leaf(b"hello");
        assert_eq!(a, b);
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn merge_dedup_same_pair() {
        let mut v = Vocab::new();
        let a = v.get_or_insert_leaf(b"alpha");
        let b = v.get_or_insert_leaf(b"beta");
        let m1 = v.get_or_insert_merge(a, b);
        let m2 = v.get_or_insert_merge(a, b);
        assert_eq!(m1, m2);
        assert_eq!(v.len(), 3); // 2 leaves + 1 merge
    }

    #[test]
    fn depth_of_leaves_is_zero() {
        let mut v = Vocab::new();
        let id = v.get_or_insert_leaf(b"x");
        assert_eq!(v.depth_of(id), 0);
    }

    #[test]
    fn depth_of_merge_is_max_child_plus_one() {
        let mut v = Vocab::new();
        let a = v.get_or_insert_leaf(b"a");
        let b = v.get_or_insert_leaf(b"b");
        let ab = v.get_or_insert_merge(a, b);
        assert_eq!(v.depth_of(ab), 1);

        let c = v.get_or_insert_leaf(b"c");
        let abc = v.get_or_insert_merge(ab, c);
        assert_eq!(v.depth_of(abc), 2);

        let d = v.get_or_insert_leaf(b"d");
        let cd = v.get_or_insert_merge(c, d);
        let abcd = v.get_or_insert_merge(ab, cd);
        assert_eq!(v.depth_of(abcd), 2); // max(depth(ab)=1, depth(cd)=1) + 1
    }

    #[test]
    fn access_count_tracks_lookups() {
        let mut v = Vocab::new();
        let id = v.get_or_insert_leaf(b"x");
        assert_eq!(v.access_count(id), 0);
        v.lookup_tracked(id);
        v.lookup_tracked(id);
        v.lookup_tracked(id);
        assert_eq!(v.access_count(id), 3);
    }

    #[test]
    fn freeze_moves_mutable_to_frozen() {
        let mut v = Vocab::new();
        v.get_or_insert_leaf(b"x");
        v.get_or_insert_leaf(b"y");
        assert_eq!(v.mutable_size(), 2);
        assert_eq!(v.frozen_size(), 0);
        v.freeze_mutable();
        assert_eq!(v.mutable_size(), 0);
        assert_eq!(v.frozen_size(), 2);
    }

    #[test]
    fn lookup_after_freeze_still_works() {
        let mut v = Vocab::new();
        let id = v.get_or_insert_leaf(b"x");
        v.freeze_mutable();
        assert!(v.lookup(id).is_some());
    }
}
