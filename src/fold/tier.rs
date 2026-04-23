//! Tier — hot/cold classification with materialization cache.
//!
//! Per consultation: "tiered storage" is a known pattern.
//! - Hot atoms (top 10%): stay shallow-folded (depth ≤ 2), materialized copies cached
//! - Cold atoms (90%):    aggressively folded (depth up to 8)
//!
//! The materialization cache is a simple size-bounded LRU of pre-unfolded hot atoms.

use ahash::AHashMap;
use std::collections::VecDeque;
use super::vocab::Vocab;
use super::FoldId;

use super::walk::unfold;

/// Classification of a token by access frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Top 10% — high access count.
    Hot,
    /// Bottom 90% — rarely accessed.
    Cold,
}

/// A simple LRU cache of materialized (fully-unfolded) hot atoms.
/// Stores raw bytes of the unfold result, keyed by FoldId.
#[derive(Debug)]
pub struct Materializer {
    cache: AHashMap<FoldId, Vec<u8>>,
    order: VecDeque<FoldId>,
    max_entries: usize,
    max_total_bytes: usize,
    current_bytes: usize,
}

impl Materializer {
    pub fn new(max_entries: usize, max_total_bytes: usize) -> Self {
        Self {
            cache: AHashMap::new(),
            order: VecDeque::new(),
            max_entries,
            max_total_bytes,
            current_bytes: 0,
        }
    }

    /// Default: 10K entries, 100 MB total.
    pub fn default_limits() -> Self {
        Self::new(10_000, 100 * 1024 * 1024)
    }

    /// Lookup a materialized atom; returns None if not cached.
    /// Moves to front (LRU policy).
    pub fn get(&mut self, id: FoldId) -> Option<&[u8]> {
        if !self.cache.contains_key(&id) {
            return None;
        }
        // Move to back (most-recently-used)
        if let Some(pos) = self.order.iter().position(|x| *x == id) {
            self.order.remove(pos);
            self.order.push_back(id);
        }
        self.cache.get(&id).map(|v| v.as_slice())
    }

    /// Insert a materialized value; evicts LRU if full.
    pub fn insert(&mut self, id: FoldId, value: Vec<u8>) {
        let size = value.len();

        // Eviction loop
        while (self.cache.len() >= self.max_entries ||
               self.current_bytes + size > self.max_total_bytes) && !self.order.is_empty() {
            if let Some(evict_id) = self.order.pop_front() {
                if let Some(evicted) = self.cache.remove(&evict_id) {
                    self.current_bytes -= evicted.len();
                }
            }
        }

        self.cache.insert(id, value);
        self.order.push_back(id);
        self.current_bytes += size;
    }

    /// Get or materialize + cache.
    /// Returns the unfolded bytes. Internal walk is capped at max_depth.
    pub fn get_or_materialize(
        &mut self,
        vocab: &Vocab,
        id: FoldId,
        max_depth: u8,
    ) -> Option<Vec<u8>> {
        if let Some(bytes) = self.get(id) {
            return Some(bytes.to_vec());
        }
        if let Ok(bytes) = unfold(vocab, id, max_depth) {
            let clone = bytes.clone();
            self.insert(id, bytes);
            return Some(clone);
        }
        None
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub fn total_bytes(&self) -> usize {
        self.current_bytes
    }
}

/// Classify a token as hot or cold based on its access count and
/// a threshold (typically 90th-percentile access count).
pub fn classify(vocab: &Vocab, id: FoldId, hot_threshold: u32) -> Tier {
    if vocab.access_count(id) >= hot_threshold {
        Tier::Hot
    } else {
        Tier::Cold
    }
}

/// Compute the 90th percentile of access counts across all tokens.
/// Used as the hot/cold threshold.
pub fn compute_hot_threshold(_vocab: &Vocab) -> u32 {
    // Walk vocab counting accesses per id.
    // For simplicity, scan public API: we don't have iter on private access_counts.
    // For a proper impl we'd expose an iterator; here we use a heuristic.
    // TODO: expose `Vocab::iter_access_counts()` for real percentile.
    // Fallback: any access > 0 counts as hot. Improve in V2.
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::MAX_FOLD_DEPTH;

    #[test]
    fn materializer_stores_and_retrieves() {
        let mut m = Materializer::new(10, 1024);
        let id = FoldId::ZERO;
        m.insert(id, b"hello".to_vec());
        assert_eq!(m.get(id).map(|b| b.to_vec()), Some(b"hello".to_vec()));
    }

    #[test]
    fn materializer_evicts_lru() {
        let mut m = Materializer::new(2, 1024);
        let id1 = FoldId([1; 16]);
        let id2 = FoldId([2; 16]);
        let id3 = FoldId([3; 16]);
        m.insert(id1, b"a".to_vec());
        m.insert(id2, b"b".to_vec());
        // Access id1 to make it more-recently-used than id2
        m.get(id1);
        // Insert id3 should evict id2 (LRU)
        m.insert(id3, b"c".to_vec());
        assert!(m.get(id1).is_some());
        assert!(m.get(id2).is_none(), "id2 should have been evicted");
        assert!(m.get(id3).is_some());
    }

    #[test]
    fn materializer_respects_byte_cap() {
        let mut m = Materializer::new(100, 10); // 10-byte cap
        let id1 = FoldId([1; 16]);
        let id2 = FoldId([2; 16]);
        m.insert(id1, vec![0u8; 8]); // 8 bytes
        m.insert(id2, vec![0u8; 5]); // 5 more → total 13 > 10, evicts id1
        assert!(m.get(id1).is_none());
        assert!(m.get(id2).is_some());
    }

    #[test]
    fn get_or_materialize_populates_cache() {
        let mut v = Vocab::new();
        let a = v.get_or_insert_leaf(b"hello");
        let b = v.get_or_insert_leaf(b"world");
        let ab = v.get_or_insert_merge(a, b);

        let mut m = Materializer::default_limits();
        let bytes1 = m.get_or_materialize(&v, ab, MAX_FOLD_DEPTH).unwrap();
        assert_eq!(bytes1, b"helloworld");

        // Second call hits cache
        let bytes2 = m.get_or_materialize(&v, ab, MAX_FOLD_DEPTH).unwrap();
        assert_eq!(bytes2, b"helloworld");
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn classify_by_access_count() {
        let mut v = Vocab::new();
        let a = v.get_or_insert_leaf(b"frequent");
        let b = v.get_or_insert_leaf(b"rare");

        // Access 'a' many times
        for _ in 0..10 {
            v.lookup_tracked(a);
        }

        assert_eq!(classify(&v, a, 5), Tier::Hot);
        assert_eq!(classify(&v, b, 5), Tier::Cold);
    }
}
