//! Background — WAL-triggered batch folding (LSM tree compaction pattern).
//!
//! Design:
//! - User writes go to an append-only WAL (Write-Ahead Log) — fast, never folded
//! - Every N atoms OR T seconds, a background task scans the WAL and folds
//! - After folding, WAL segment is promoted to the main fold store
//! - User queries never wait on fold — they read from WAL + folded store together
//!
//! This matches RocksDB/Cassandra compaction: writes stay cheap, background
//! does the heavy work. Reads are slightly slower but still fast.

use super::vocab::Vocab;
use super::bpe::{BpeConfig, fold_text};

use super::FoldStats;

/// Trigger policy for background fold.
#[derive(Debug, Clone)]
pub struct FoldTrigger {
    /// Fold when this many un-folded atoms accumulate.
    pub atoms_threshold: u32,
    /// Fold after this many seconds since last fold.
    pub time_threshold_sec: u64,
}

impl Default for FoldTrigger {
    fn default() -> Self {
        Self {
            atoms_threshold: 100_000,
            time_threshold_sec: 600,   // 10 minutes
        }
    }
}

/// A pending batch of text blobs waiting to be folded.
#[derive(Debug, Default)]
pub struct FoldBatch {
    items: Vec<String>,
    total_bytes: u64,
}

impl FoldBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, text: String) {
        self.total_bytes += text.len() as u64;
        self.items.push(text);
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    pub fn drain(&mut self) -> Vec<String> {
        self.total_bytes = 0;
        std::mem::take(&mut self.items)
    }
}

/// Fold an entire batch of text into a vocab, returning stats.
/// This is the main entry point for the background task.
pub fn fold_batch(batch: &[String], vocab: &mut Vocab, config: &BpeConfig) -> FoldStats {
    let mut stats = FoldStats::default();

    for text in batch {
        stats.raw_bytes += text.len() as u64;
        let vocab_before = vocab.len();
        let (_tokens, merges) = fold_text(text, vocab, config);
        stats.merge_count += merges as u64;
        stats.atom_count += (vocab.len() - vocab_before) as u64;
    }

    stats.folded_bytes = vocab.total_bytes();
    stats
}

/// Batch processor that tracks triggers and decides when to fold.
/// Not thread-safe — wrap in Mutex if needed across threads.
pub struct BackgroundFolder {
    batch: FoldBatch,
    trigger: FoldTrigger,
    last_fold: std::time::Instant,
    total_merges: u64,
    total_folds: u32,
}

impl BackgroundFolder {
    pub fn new(trigger: FoldTrigger) -> Self {
        Self {
            batch: FoldBatch::new(),
            trigger,
            last_fold: std::time::Instant::now(),
            total_merges: 0,
            total_folds: 0,
        }
    }

    pub fn submit(&mut self, text: String) {
        self.batch.push(text);
    }

    /// Check if we should fold now based on trigger policy.
    pub fn should_fold(&self) -> bool {
        let atom_hit = self.batch.len() as u32 >= self.trigger.atoms_threshold;
        let time_hit = self.last_fold.elapsed().as_secs() >= self.trigger.time_threshold_sec;
        (atom_hit || time_hit) && !self.batch.is_empty()
    }

    /// Run the fold if conditions are met. Returns stats if it ran.
    pub fn maybe_fold(&mut self, vocab: &mut Vocab, config: &BpeConfig) -> Option<FoldStats> {
        if !self.should_fold() {
            return None;
        }
        let items = self.batch.drain();
        let stats = fold_batch(&items, vocab, config);
        self.total_merges += stats.merge_count;
        self.total_folds += 1;
        self.last_fold = std::time::Instant::now();
        Some(stats)
    }

    /// Force a fold regardless of triggers.
    pub fn force_fold(&mut self, vocab: &mut Vocab, config: &BpeConfig) -> FoldStats {
        let items = self.batch.drain();
        let stats = fold_batch(&items, vocab, config);
        self.total_merges += stats.merge_count;
        self.total_folds += 1;
        self.last_fold = std::time::Instant::now();
        stats
    }

    pub fn total_folds(&self) -> u32 { self.total_folds }
    pub fn total_merges(&self) -> u64 { self.total_merges }
    pub fn pending_items(&self) -> usize { self.batch.len() }
    pub fn pending_bytes(&self) -> u64 { self.batch.total_bytes() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_accumulates() {
        let mut b = FoldBatch::new();
        b.push("hello".to_string());
        b.push("world".to_string());
        assert_eq!(b.len(), 2);
        assert_eq!(b.total_bytes(), 10);
    }

    #[test]
    fn batch_drain_resets() {
        let mut b = FoldBatch::new();
        b.push("hello".to_string());
        let drained = b.drain();
        assert_eq!(drained.len(), 1);
        assert_eq!(b.len(), 0);
        assert_eq!(b.total_bytes(), 0);
    }

    #[test]
    fn fold_batch_produces_stats() {
        let mut v = Vocab::new();
        let items = vec![
            "hello world hello world".to_string(),
            "foo bar foo bar foo".to_string(),
        ];
        let stats = fold_batch(&items, &mut v, &BpeConfig::default());
        assert!(stats.raw_bytes > 0);
        assert!(stats.merge_count > 0);
        assert!(stats.folded_bytes > 0);
    }

    #[test]
    fn background_folder_triggers_on_threshold() {
        let trigger = FoldTrigger { atoms_threshold: 3, time_threshold_sec: 3600 };
        let mut bf = BackgroundFolder::new(trigger);

        bf.submit("a".to_string());
        bf.submit("b".to_string());
        assert!(!bf.should_fold());

        bf.submit("c".to_string());
        assert!(bf.should_fold());

        let mut v = Vocab::new();
        let stats = bf.maybe_fold(&mut v, &BpeConfig::default());
        assert!(stats.is_some());
        assert_eq!(bf.pending_items(), 0);
    }

    #[test]
    fn background_folder_doesnt_trigger_on_empty() {
        let trigger = FoldTrigger { atoms_threshold: 1, time_threshold_sec: 0 };
        let bf = BackgroundFolder::new(trigger);
        assert!(!bf.should_fold());  // empty batch = no trigger even though thresholds low
    }

    #[test]
    fn force_fold_runs_regardless() {
        let mut bf = BackgroundFolder::new(FoldTrigger::default());
        bf.submit("hello world hello world".to_string());
        let mut v = Vocab::new();
        let stats = bf.force_fold(&mut v, &BpeConfig::default());
        assert!(stats.raw_bytes > 0);
        assert_eq!(bf.pending_items(), 0);
    }
}
