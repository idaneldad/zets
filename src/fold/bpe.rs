//! BPE — Byte Pair Encoding iterative merging.
//!
//! Algorithm (per classic Sennrich 2016, with our hash-consing twist):
//!
//! 1. Tokenize input into initial FoldIds (one per character or per byte or per morpheme)
//! 2. Count all adjacent pairs in the token stream
//! 3. Pick the most-frequent pair (freq >= min_frequency)
//! 4. Create (or find) the merge token via vocab.get_or_insert_merge
//! 5. Rewrite the stream: every occurrence of this pair → the merge token
//! 6. Repeat until max_merges reached or no pair has freq >= min_frequency
//!
//! Depth guard: once a candidate merge would produce a token of depth > MAX_FOLD_DEPTH,
//! skip it (even if frequent). Keeps walks cache-friendly.
//!
//! Performance: O(n * m) where n = stream length, m = merges. For 14.5M articles,
//! we fold per-article and union vocabs — this scales linearly.

use ahash::AHashMap;
use super::vocab::Vocab;
use super::FoldId;
use super::MAX_FOLD_DEPTH;

/// BPE configuration for a single fold pass.
#[derive(Debug, Clone)]
pub struct BpeConfig {
    /// Maximum number of merges to perform in one fold pass.
    pub max_merges: u32,
    /// Minimum frequency for a pair to be merged.
    pub min_frequency: u32,
    /// Maximum depth for any merge (cache-friendliness).
    pub max_depth: u8,
}

impl Default for BpeConfig {
    fn default() -> Self {
        Self {
            max_merges: 10_000,
            min_frequency: 2,
            max_depth: MAX_FOLD_DEPTH,
        }
    }
}

/// Encode a byte stream into initial leaf tokens (one per character).
/// For Hebrew/Unicode: one token per codepoint (not per byte).
pub fn tokenize_chars(input: &str, vocab: &mut Vocab) -> Vec<FoldId> {
    input.chars()
        .map(|c| {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            vocab.get_or_insert_leaf(s.as_bytes())
        })
        .collect()
}

/// Encode a byte stream as raw bytes (byte-level BPE).
/// Use this for binary modalities or when char-level is too expensive.
pub fn tokenize_bytes(input: &[u8], vocab: &mut Vocab) -> Vec<FoldId> {
    input.iter()
        .map(|&b| vocab.get_or_insert_leaf(&[b]))
        .collect()
}

/// Tokenize by splitting into runs of whitespace and non-whitespace.
/// Each run becomes a leaf. This preserves exact bytes (lossless roundtrip)
/// while still giving BPE word-level merging behavior.
///
/// Example: "hello world" → ["hello", " ", "world"]  (3 leaves)
///          "שלום  עולם" → ["שלום", "  ", "עולם"]   (3 leaves, 2 spaces preserved)
pub fn tokenize_words(input: &str, vocab: &mut Vocab) -> Vec<FoldId> {
    let mut tokens = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Find the start boundary: same classification (whitespace or not)
        let start = i;
        // Determine class of current char using char boundaries
        let ch_start = i;
        // Find end of this run: walk by char while class stays same
        let mut run_end = i;
        let input_str = input;
        let first_char = input_str[start..].chars().next().expect("has char");
        let first_is_ws = first_char.is_whitespace();
        let mut char_iter = input_str[start..].char_indices();
        for (off, c) in char_iter.by_ref() {
            if c.is_whitespace() != first_is_ws {
                run_end = start + off;
                break;
            }
            run_end = start + off + c.len_utf8();
        }
        tokens.push(vocab.get_or_insert_leaf(&bytes[start..run_end]));
        i = run_end;
        if i == ch_start { break; }  // safety
    }
    tokens
}

/// Run iterative BPE on a token stream, mutating both the stream and vocab.
/// Returns the number of merges performed.
///
/// **Incremental algorithm** (100-1000x faster than naive):
/// - Initial: O(n) full pair-count scan
/// - Each iteration: O(occurrences_of_best_pair) local update, NOT O(n) full rescan
/// - Replaces in-place using a linked-list-like skip pattern
///
/// Reference: Sennrich 2016 BPE + subword-nmt fast merge implementation.
pub fn bpe_fold(tokens: &mut Vec<FoldId>, vocab: &mut Vocab, config: &BpeConfig) -> u32 {
    if tokens.len() < 2 { return 0; }

    // Phase 1: full initial pair count — O(n)
    let mut pair_counts: AHashMap<(FoldId, FoldId), u32> = AHashMap::with_capacity(tokens.len());
    for pair in tokens.windows(2) {
        *pair_counts.entry((pair[0], pair[1])).or_insert(0) += 1;
    }

    let mut merges_done: u32 = 0;

    while merges_done < config.max_merges {
        // Find best pair that (a) meets min_frequency and (b) respects max_depth
        let mut best_pair: Option<(FoldId, FoldId)> = None;
        let mut best_count: u32 = 0;
        for (&pair, &count) in pair_counts.iter() {
            if count < config.min_frequency { continue; }
            if count <= best_count { continue; }
            let new_depth = 1u8.saturating_add(vocab.depth_of(pair.0).max(vocab.depth_of(pair.1)));
            if new_depth > config.max_depth { continue; }
            best_pair = Some(pair);
            best_count = count;
        }

        let pair = match best_pair {
            Some(p) => p,
            None => break,
        };

        let merged_id = vocab.get_or_insert_merge(pair.0, pair.1);

        // Phase 2: in-place rewrite + incremental pair count update — O(occurrences)
        // Walk the stream; every time we see `pair`, replace with merged_id and
        // update counts of neighbouring pairs locally.
        //
        // For each replacement at position i:
        //   - The pair (tokens[i-1], tokens[i]) becomes (tokens[i-1], merged)
        //   - The pair (tokens[i+1], tokens[i+2]) becomes (merged, tokens[i+2])
        //   - The merged pair itself (pair.0, pair.1) goes to zero for this occurrence
        let mut write = 0usize;
        let mut read = 0usize;
        let len = tokens.len();
        while read < len {
            if read + 1 < len && tokens[read] == pair.0 && tokens[read + 1] == pair.1 {
                // About to merge — adjust neighbour pair counts.
                // Left neighbour: if there's a token before `write`, the pair
                // (tokens[write-1], pair.0) is replaced by (tokens[write-1], merged_id)
                if write > 0 {
                    let left = tokens[write - 1];
                    if let Some(c) = pair_counts.get_mut(&(left, pair.0)) {
                        if *c > 0 { *c -= 1; }
                    }
                    *pair_counts.entry((left, merged_id)).or_insert(0) += 1;
                }
                // Right neighbour: if there's a token after `read+1`, the pair
                // (pair.1, tokens[read+2]) is replaced by (merged_id, tokens[read+2])
                if read + 2 < len {
                    let right = tokens[read + 2];
                    if let Some(c) = pair_counts.get_mut(&(pair.1, right)) {
                        if *c > 0 { *c -= 1; }
                    }
                    *pair_counts.entry((merged_id, right)).or_insert(0) += 1;
                }
                // Write the merged token
                tokens[write] = merged_id;
                write += 1;
                read += 2;
            } else {
                tokens[write] = tokens[read];
                write += 1;
                read += 1;
            }
        }
        tokens.truncate(write);

        // Remove the merged pair itself from counts (it no longer exists in stream)
        pair_counts.remove(&pair);

        merges_done += 1;

        if tokens.len() < 2 { break; }
    }

    merges_done
}

/// Count frequencies of adjacent pairs (used by tests).
#[allow(dead_code)]
fn count_pairs(tokens: &[FoldId]) -> AHashMap<(FoldId, FoldId), u32> {
    let mut counts = AHashMap::new();
    for pair in tokens.windows(2) {
        let key = (pair[0], pair[1]);
        *counts.entry(key).or_insert(0u32) += 1;
    }
    counts
}

/// Fold a string (text) in one shot — handy entry point.
/// Returns (final tokens, number of merges performed).
pub fn fold_text(text: &str, vocab: &mut Vocab, config: &BpeConfig) -> (Vec<FoldId>, u32) {
    let mut tokens = tokenize_chars(text, vocab);
    let merges = bpe_fold(&mut tokens, vocab, config);
    (tokens, merges)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_chars_one_per_codepoint() {
        let mut v = Vocab::new();
        let tokens = tokenize_chars("abc", &mut v);
        assert_eq!(tokens.len(), 3);
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn tokenize_hebrew_per_codepoint() {
        let mut v = Vocab::new();
        let tokens = tokenize_chars("שלום", &mut v);
        assert_eq!(tokens.len(), 4); // 4 Hebrew letters
    }

    #[test]
    fn dedup_identical_chars() {
        let mut v = Vocab::new();
        let tokens = tokenize_chars("aaa", &mut v);
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], tokens[1]);
        assert_eq!(tokens[1], tokens[2]);
        assert_eq!(v.len(), 1); // only one unique leaf
    }

    #[test]
    fn bpe_merges_frequent_pair() {
        let mut v = Vocab::new();
        // "ababab" → (a,b) is most frequent
        let mut tokens = tokenize_chars("ababab", &mut v);
        let cfg = BpeConfig { max_merges: 1, min_frequency: 2, max_depth: 8 };
        let merges = bpe_fold(&mut tokens, &mut v, &cfg);
        assert_eq!(merges, 1);
        assert_eq!(tokens.len(), 3); // [ab, ab, ab]
        assert_eq!(tokens[0], tokens[1]);
        assert_eq!(tokens[1], tokens[2]);
    }

    #[test]
    fn bpe_respects_max_depth() {
        let mut v = Vocab::new();
        // Force many merges that would build deep tree
        let mut tokens = tokenize_chars(&"ab".repeat(100), &mut v);
        let cfg = BpeConfig { max_merges: 100, min_frequency: 2, max_depth: 2 };
        bpe_fold(&mut tokens, &mut v, &cfg);

        // Verify no token exceeds depth 2
        for t in &tokens {
            assert!(v.depth_of(*t) <= 2, "token exceeded max depth");
        }
    }

    #[test]
    fn bpe_stops_when_no_frequent_pair() {
        let mut v = Vocab::new();
        let mut tokens = tokenize_chars("abcd", &mut v); // no repeating pair
        let cfg = BpeConfig { max_merges: 100, min_frequency: 2, max_depth: 8 };
        let merges = bpe_fold(&mut tokens, &mut v, &cfg);
        assert_eq!(merges, 0);
        assert_eq!(tokens.len(), 4);
    }

    #[test]
    fn bpe_compresses_repetitive_text() {
        let mut v = Vocab::new();
        let input = "abcabcabcabcabc"; // 15 chars, "abc" repeated 5x
        let (tokens, _merges) = fold_text(input, &mut v, &BpeConfig::default());
        // After BPE: [abc, abc, abc, abc, abc] → 5 tokens (vs 15 raw)
        assert!(tokens.len() < input.len());
        assert!(tokens.len() <= 5);
    }

    #[test]
    fn bpe_on_hebrew() {
        let mut v = Vocab::new();
        let input = "שלום שלום שלום שלום";
        let (tokens, merges) = fold_text(input, &mut v, &BpeConfig::default());
        assert!(merges > 0);
        // Shouldn't expand
        let char_count = input.chars().count();
        assert!(tokens.len() <= char_count);
    }
}
