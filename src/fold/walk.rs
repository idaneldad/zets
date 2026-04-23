//! Walk — recursive unfolding with depth guard and materialization cache.
//!
//! Given a FoldId, unfold back to the original byte stream.
//!
//! Performance notes:
//! - Each step is a hashmap lookup → potentially a cache miss (~100ns on modern hw)
//! - We cap depth at MAX_FOLD_DEPTH (default 8) per measured latency impact
//! - Hot atoms get materialized (full unfold cached) per `tier` module
//! - We collect into a single Vec<u8> to avoid intermediate allocations

use super::vocab::{Vocab, TokenContent};
use super::FoldId;

/// Error type for walk operations.
#[derive(Debug)]
pub enum WalkError {
    /// FoldId not found in vocab (broken DAG — shouldn't happen in practice).
    NotFound(FoldId),
    /// Depth exceeded safe limit — walk aborted.
    DepthExceeded(u8),
}

impl std::fmt::Display for WalkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalkError::NotFound(id) => write!(f, "FoldId not found: {}", id),
            WalkError::DepthExceeded(d) => write!(f, "depth exceeded: {}", d),
        }
    }
}

impl std::error::Error for WalkError {}

/// Fully unfold a FoldId to its original byte sequence.
/// Returns an error if the FoldId is missing or depth exceeds `max_depth`.
pub fn unfold(vocab: &Vocab, id: FoldId, max_depth: u8) -> Result<Vec<u8>, WalkError> {
    let mut out = Vec::new();
    unfold_into(vocab, id, &mut out, 0, max_depth)?;
    Ok(out)
}

/// Unfold into an existing buffer (avoids allocation).
fn unfold_into(
    vocab: &Vocab,
    id: FoldId,
    out: &mut Vec<u8>,
    depth: u8,
    max_depth: u8,
) -> Result<(), WalkError> {
    if depth > max_depth {
        return Err(WalkError::DepthExceeded(depth));
    }

    let content = vocab.lookup(id).ok_or(WalkError::NotFound(id))?;

    match content {
        TokenContent::Leaf(bytes) => {
            out.extend_from_slice(bytes);
            Ok(())
        }
        TokenContent::Merge(left, right) => {
            // Clone the FoldIds so we can drop the borrow before recursing.
            let l = *left;
            let r = *right;
            unfold_into(vocab, l, out, depth + 1, max_depth)?;
            unfold_into(vocab, r, out, depth + 1, max_depth)?;
            Ok(())
        }
    }
}

/// Unfold a full token stream (e.g., the output of BPE) back to the original text.
/// Concatenates all unfolded tokens.
pub fn unfold_stream(vocab: &Vocab, tokens: &[FoldId], max_depth: u8) -> Result<Vec<u8>, WalkError> {
    let mut out = Vec::with_capacity(tokens.len() * 4); // rough estimate
    for &id in tokens {
        unfold_into(vocab, id, &mut out, 0, max_depth)?;
    }
    Ok(out)
}

/// Unfold to a UTF-8 string (returns an error if not valid UTF-8).
pub fn unfold_to_string(vocab: &Vocab, id: FoldId, max_depth: u8) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = unfold(vocab, id, max_depth)?;
    Ok(String::from_utf8(bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::bpe::{BpeConfig, fold_text};
    use super::super::MAX_FOLD_DEPTH;

    #[test]
    fn unfold_leaf_returns_bytes() {
        let mut v = Vocab::new();
        let id = v.get_or_insert_leaf(b"hello");
        let out = unfold(&v, id, MAX_FOLD_DEPTH).unwrap();
        assert_eq!(out, b"hello");
    }

    #[test]
    fn unfold_merge_concatenates() {
        let mut v = Vocab::new();
        let a = v.get_or_insert_leaf(b"ab");
        let b = v.get_or_insert_leaf(b"cd");
        let ab = v.get_or_insert_merge(a, b);
        let out = unfold(&v, ab, MAX_FOLD_DEPTH).unwrap();
        assert_eq!(out, b"abcd");
    }

    #[test]
    fn unfold_deep_stacks() {
        let mut v = Vocab::new();
        let a = v.get_or_insert_leaf(b"1");
        let b = v.get_or_insert_leaf(b"2");
        let c = v.get_or_insert_leaf(b"3");
        let d = v.get_or_insert_leaf(b"4");
        let ab = v.get_or_insert_merge(a, b);
        let cd = v.get_or_insert_merge(c, d);
        let abcd = v.get_or_insert_merge(ab, cd);
        let out = unfold(&v, abcd, MAX_FOLD_DEPTH).unwrap();
        assert_eq!(out, b"1234");
    }

    #[test]
    fn unfold_respects_depth_cap() {
        let mut v = Vocab::new();
        let a = v.get_or_insert_leaf(b"a");
        let b = v.get_or_insert_leaf(b"b");
        let ab = v.get_or_insert_merge(a, b);
        let ab2 = v.get_or_insert_merge(ab, ab);
        let ab4 = v.get_or_insert_merge(ab2, ab2);

        // depth-3 walk with max_depth=2 should fail
        let result = unfold(&v, ab4, 2);
        assert!(matches!(result, Err(WalkError::DepthExceeded(_))));
    }

    #[test]
    fn unfold_stream_concatenates_tokens() {
        let mut v = Vocab::new();
        let a = v.get_or_insert_leaf(b"Hello ");
        let b = v.get_or_insert_leaf(b"World");
        let out = unfold_stream(&v, &[a, b], MAX_FOLD_DEPTH).unwrap();
        assert_eq!(out, b"Hello World");
    }

    #[test]
    fn roundtrip_bpe_then_unfold() {
        // Input → BPE fold → stream of FoldIds → walk → original input
        let mut v = Vocab::new();
        let input = "hello world hello world hello";
        let (tokens, _) = fold_text(input, &mut v, &BpeConfig::default());

        let unfolded = unfold_stream(&v, &tokens, MAX_FOLD_DEPTH).unwrap();
        let recovered = String::from_utf8(unfolded).unwrap();
        assert_eq!(recovered, input, "BPE fold + walk must be lossless");
    }

    #[test]
    fn roundtrip_on_hebrew() {
        let mut v = Vocab::new();
        let input = "שלום עולם שלום חברים שלום";
        let (tokens, _) = fold_text(input, &mut v, &BpeConfig::default());
        let unfolded = unfold_stream(&v, &tokens, MAX_FOLD_DEPTH).unwrap();
        let recovered = String::from_utf8(unfolded).unwrap();
        assert_eq!(recovered, input);
    }

    #[test]
    fn roundtrip_preserves_unicode_exactly() {
        let mut v = Vocab::new();
        // Mix of Hebrew, ASCII, emoji — ensure lossless roundtrip
        let input = "Hello שלום 你好 🙂 مرحبا";
        let (tokens, _) = fold_text(input, &mut v, &BpeConfig::default());
        let unfolded = unfold_stream(&v, &tokens, MAX_FOLD_DEPTH).unwrap();
        assert_eq!(String::from_utf8(unfolded).unwrap(), input);
    }

    #[test]
    fn unfold_to_string_invalid_utf8_fails() {
        let mut v = Vocab::new();
        // Inject invalid UTF-8 sequence
        let id = v.get_or_insert_leaf(&[0xFF, 0xFE]);
        let r = unfold_to_string(&v, id, MAX_FOLD_DEPTH);
        assert!(r.is_err());
    }
}
