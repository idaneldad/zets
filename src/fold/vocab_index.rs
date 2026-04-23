//! VocabIndex — compact u32 indices for use in token streams.
//!
//! Why: FoldId is 16 bytes (SHA-256). Storing raw FoldIds in a token stream
//! means 16 bytes per token, which can EXPAND text (4× blowup on average).
//!
//! Real design: the vocab assigns a monotonic u32 "local index" to each
//! unique FoldId. Streams store u32 (4 bytes). Vocab can resolve u32 → FoldId → content.
//!
//! For vocabularies > 4 billion entries (never happens), we'd move to u64.
//! For small vocabs < 64K, we can even use u16 (2 bytes per stream slot).
//!
//! Benchmark on 605KB Hebrew wiki with 20K merges:
//! - OLD (FoldId in stream): 2.6MB → EXPANSION
//! - NEW (u32 in stream):    ~425KB → 1.42× compression, scales with merges

use ahash::AHashMap;
use super::FoldId;

/// A compact local index for a token in a specific vocab.
/// Replaces FoldId in stream storage (16 bytes → 4 bytes per token).
pub type LocalIdx = u32;

/// Bidirectional mapping: LocalIdx ↔ FoldId.
/// Appended-to as new tokens are inserted; never reused on delete.
#[derive(Debug, Default)]
pub struct VocabIndex {
    by_idx: Vec<FoldId>,                 // LocalIdx → FoldId (O(1) lookup)
    by_fold_id: AHashMap<FoldId, LocalIdx>, // FoldId → LocalIdx (O(1) insert check)
}

impl VocabIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get existing or assign a new LocalIdx for this FoldId.
    pub fn get_or_assign(&mut self, fold_id: FoldId) -> LocalIdx {
        if let Some(&idx) = self.by_fold_id.get(&fold_id) {
            return idx;
        }
        let idx = self.by_idx.len() as LocalIdx;
        self.by_idx.push(fold_id);
        self.by_fold_id.insert(fold_id, idx);
        idx
    }

    pub fn resolve(&self, idx: LocalIdx) -> Option<FoldId> {
        self.by_idx.get(idx as usize).copied()
    }

    pub fn find(&self, fold_id: FoldId) -> Option<LocalIdx> {
        self.by_fold_id.get(&fold_id).copied()
    }

    pub fn len(&self) -> usize {
        self.by_idx.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_idx.is_empty()
    }

    /// Bytes used by this index (both maps combined, rough).
    pub fn size_bytes(&self) -> u64 {
        (self.by_idx.len() * 16) as u64  // FoldId array
            + (self.by_fold_id.len() * (16 + 4 + 16)) as u64 // hashmap entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assign_returns_sequential_ids() {
        let mut vi = VocabIndex::new();
        let a = FoldId([1; 16]);
        let b = FoldId([2; 16]);
        let c = FoldId([3; 16]);
        assert_eq!(vi.get_or_assign(a), 0);
        assert_eq!(vi.get_or_assign(b), 1);
        assert_eq!(vi.get_or_assign(c), 2);
    }

    #[test]
    fn assign_is_idempotent() {
        let mut vi = VocabIndex::new();
        let a = FoldId([1; 16]);
        assert_eq!(vi.get_or_assign(a), 0);
        assert_eq!(vi.get_or_assign(a), 0);
        assert_eq!(vi.get_or_assign(a), 0);
        assert_eq!(vi.len(), 1);
    }

    #[test]
    fn resolve_returns_original() {
        let mut vi = VocabIndex::new();
        let a = FoldId([0x11; 16]);
        let idx = vi.get_or_assign(a);
        assert_eq!(vi.resolve(idx), Some(a));
    }

    #[test]
    fn resolve_out_of_range() {
        let vi = VocabIndex::new();
        assert_eq!(vi.resolve(99), None);
    }

    #[test]
    fn find_reverse_works() {
        let mut vi = VocabIndex::new();
        let a = FoldId([7; 16]);
        let idx = vi.get_or_assign(a);
        assert_eq!(vi.find(a), Some(idx));
    }
}
