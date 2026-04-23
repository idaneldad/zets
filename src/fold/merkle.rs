//! Merkle DAG content addressing via SHA-256 truncated to 128 bits.
//!
//! Why SHA-256 not FNV-1a?
//! - FNV-1a 64-bit: birthday collision expected at ~2^32 atoms = 4 billion.
//!   Measured 2.7% at 10^9 atoms per Python prototype test [2].
//! - SHA-256 truncated to 128-bit: birthday collision expected at 2^64 atoms.
//!   Practically collision-free for any realistic ZETS deployment.
//!
//! Why 128-bit (not full 256)?
//! - 16 bytes per ID is enough for uniqueness
//! - 32 bytes doubles storage for no benefit at our scale
//! - Matches Git short-hash convention (though Git uses 160-bit full)

use sha2::{Digest, Sha256};
use super::FoldId;

/// Compute the FoldId (SHA-256 truncated to 16 bytes) of content bytes.
pub fn hash_leaf(content: &[u8]) -> FoldId {
    let mut hasher = Sha256::new();
    hasher.update(b"LEAF:");  // domain separator prevents collision with merge nodes
    hasher.update(content);
    let digest = hasher.finalize();
    let mut out = [0u8; 16];
    out.copy_from_slice(&digest[..16]);
    FoldId(out)
}

/// Compute the FoldId of a merge node — identified by its two children.
///
/// Hash-consing property: same (left, right) → same FoldId always.
/// This is what enables automatic deduplication of folded subgraphs.
pub fn hash_merge(left: FoldId, right: FoldId) -> FoldId {
    let mut hasher = Sha256::new();
    hasher.update(b"MERGE:");  // domain separator
    hasher.update(left.as_bytes());
    hasher.update(right.as_bytes());
    let digest = hasher.finalize();
    let mut out = [0u8; 16];
    out.copy_from_slice(&digest[..16]);
    FoldId(out)
}

/// Fat atom hash — for N children (Gemini's "fat atom" suggestion).
/// Reduces depth by branching wider (B-tree style).
pub fn hash_merge_n(children: &[FoldId]) -> FoldId {
    let mut hasher = Sha256::new();
    hasher.update(b"MERGE_N:");
    let n = children.len() as u32;
    hasher.update(&n.to_le_bytes());
    for c in children {
        hasher.update(c.as_bytes());
    }
    let digest = hasher.finalize();
    let mut out = [0u8; 16];
    out.copy_from_slice(&digest[..16]);
    FoldId(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_content_same_hash() {
        let a = hash_leaf(b"hello world");
        let b = hash_leaf(b"hello world");
        assert_eq!(a, b, "same content must produce same FoldId");
    }

    #[test]
    fn different_content_different_hash() {
        let a = hash_leaf(b"hello world");
        let b = hash_leaf(b"hello world!");
        assert_ne!(a, b);
    }

    #[test]
    fn merge_is_commutative_only_if_we_say_so() {
        // By design: merge(A, B) ≠ merge(B, A) — order carries meaning (fold is sequential)
        let a = hash_leaf(b"alpha");
        let b = hash_leaf(b"beta");
        let ab = hash_merge(a, b);
        let ba = hash_merge(b, a);
        assert_ne!(ab, ba, "merge is order-sensitive by design");
    }

    #[test]
    fn hash_consing_property() {
        // If we compute merge(A, B) twice, same result — enables CAS dedup
        let a = hash_leaf(b"left");
        let b = hash_leaf(b"right");
        let ab1 = hash_merge(a, b);
        let ab2 = hash_merge(a, b);
        assert_eq!(ab1, ab2);
    }

    #[test]
    fn domain_separator_prevents_collision() {
        // If we had used the same hash for leaf and merge, a leaf containing
        // exactly 32 bytes of merge-looking content could collide. Domain
        // separator prevents this.
        let leaf_a = hash_leaf(b"a");
        let leaf_b = hash_leaf(b"b");
        let merge_ab = hash_merge(leaf_a, leaf_b);

        // Also try: leaf whose content IS the concat of leaf_a + leaf_b bytes
        let mut trick_bytes = Vec::new();
        trick_bytes.extend_from_slice(leaf_a.as_bytes());
        trick_bytes.extend_from_slice(leaf_b.as_bytes());
        let trick_leaf = hash_leaf(&trick_bytes);

        assert_ne!(merge_ab, trick_leaf,
                   "domain separators must prevent leaf/merge collisions");
    }

    #[test]
    fn fat_atom_hash_varies_with_arity() {
        let a = hash_leaf(b"a");
        let b = hash_leaf(b"b");
        let c = hash_leaf(b"c");

        let pair = hash_merge_n(&[a, b]);
        let triple = hash_merge_n(&[a, b, c]);
        assert_ne!(pair, triple);
    }

    #[test]
    fn collision_resistance_at_scale() {
        // Generate 100K hashes; zero collisions expected.
        let mut seen = std::collections::HashSet::new();
        for i in 0..100_000 {
            let s = format!("atom_{}", i);
            let h = hash_leaf(s.as_bytes());
            assert!(seen.insert(h), "collision at iteration {}", i);
        }
    }
}
