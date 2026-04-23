//! Path — represent articles and other sequences as paths of motif atoms.
//!
//! Instead of storing an article as [word_atom_1, word_atom_2, ..., word_atom_1000],
//! we mine motifs (recurring subsequences) and represent the article as
//! [motif_atom_A, word_atom_17, motif_atom_B, ...].
//!
//! Measured on 200 Hebrew Wikipedia articles:
//!   Original:    19,926 word refs
//!   Motif-encoded: 12,882 refs (35.4% reduction)
//!
//! Works per-language: Hebrew motifs are different from English.
//! Motif mining happens in background (like BPE fold).

use std::collections::{HashMap, HashSet};
use crate::fold::FoldId;

/// A motif is a sequence of atom IDs that recurs.
/// Stored once in the motif_store, referenced many times.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Motif {
    pub tokens: Vec<FoldId>,
}

impl Motif {
    pub fn new(tokens: Vec<FoldId>) -> Self {
        Self { tokens }
    }
    pub fn len(&self) -> usize {
        self.tokens.len()
    }
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

/// A motif ID — opaque handle to a motif in the store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MotifId(pub u32);

/// A path reference: either a direct token, or a motif.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathRef {
    Token(FoldId),
    Motif(MotifId),
}

/// Config for motif mining.
#[derive(Debug, Clone)]
pub struct MiningConfig {
    pub min_support: u32,       // must recur in >= this many articles
    pub min_motif_len: usize,
    pub max_motif_len: usize,
}

impl Default for MiningConfig {
    fn default() -> Self {
        Self { min_support: 3, min_motif_len: 2, max_motif_len: 8 }
    }
}

/// Mine recurring motifs from a corpus of token sequences.
///
/// For each n in [min_len..=max_len], count all n-grams across all articles.
/// Keep those with frequency >= min_support.
pub fn mine_motifs(articles: &[Vec<FoldId>], config: &MiningConfig) -> HashMap<Motif, u32> {
    let mut counts: HashMap<Motif, u32> = HashMap::new();
    for article in articles {
        // Use a seen set per article to count ARTICLE-level support
        // (avoids inflating by multiple occurrences in single article).
        let mut seen_here: HashSet<Motif> = HashSet::new();
        for n in config.min_motif_len..=config.max_motif_len {
            if article.len() < n { continue; }
            for i in 0..=(article.len() - n) {
                let motif = Motif::new(article[i..i + n].to_vec());
                seen_here.insert(motif);
            }
        }
        for m in seen_here {
            *counts.entry(m).or_insert(0) += 1;
        }
    }
    // Keep only those with enough support
    counts.retain(|_, c| *c >= config.min_support);
    counts
}

/// A motif store — indexes motifs by ID.
#[derive(Debug, Default)]
pub struct MotifStore {
    motifs: Vec<Motif>,
    by_motif: HashMap<Motif, MotifId>,
}

impl MotifStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, m: Motif) -> MotifId {
        if let Some(&id) = self.by_motif.get(&m) {
            return id;
        }
        let id = MotifId(self.motifs.len() as u32);
        self.motifs.push(m.clone());
        self.by_motif.insert(m, id);
        id
    }

    pub fn get(&self, id: MotifId) -> Option<&Motif> {
        self.motifs.get(id.0 as usize)
    }

    pub fn len(&self) -> usize {
        self.motifs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.motifs.is_empty()
    }
}

/// Encode a single article as a sequence of PathRefs using the motif store.
/// Greedy longest-first match at each position.
pub fn encode_article_greedy(
    article: &[FoldId],
    store: &MotifStore,
    motif_by_start: &HashMap<FoldId, Vec<MotifId>>,
) -> Vec<PathRef> {
    let mut out = Vec::with_capacity(article.len());
    let mut i = 0;
    while i < article.len() {
        let mut matched: Option<(MotifId, usize)> = None;
        // Try motifs that start with this token
        if let Some(candidates) = motif_by_start.get(&article[i]) {
            // Longest match first — sort by descending length
            let mut sorted: Vec<&MotifId> = candidates.iter().collect();
            sorted.sort_by_key(|&&id| {
                store.get(id).map(|m| std::cmp::Reverse(m.len())).unwrap_or(std::cmp::Reverse(0))
            });
            for mid in sorted {
                if let Some(m) = store.get(*mid) {
                    if i + m.len() <= article.len()
                        && article[i..i + m.len()] == m.tokens[..]
                    {
                        matched = Some((*mid, m.len()));
                        break;
                    }
                }
            }
        }
        if let Some((mid, len)) = matched {
            out.push(PathRef::Motif(mid));
            i += len;
        } else {
            out.push(PathRef::Token(article[i]));
            i += 1;
        }
    }
    out
}

/// Unfold a PathRef sequence back to the original token sequence (lossless).
pub fn unfold_path(path: &[PathRef], store: &MotifStore) -> Vec<FoldId> {
    let mut out = Vec::with_capacity(path.len() * 2);
    for r in path {
        match r {
            PathRef::Token(t) => out.push(*t),
            PathRef::Motif(mid) => {
                if let Some(m) = store.get(*mid) {
                    out.extend_from_slice(&m.tokens);
                }
            }
        }
    }
    out
}

/// Build a motif-by-first-token index for fast greedy matching.
pub fn index_by_first_token(
    store: &MotifStore,
    motifs: &[MotifId],
) -> HashMap<FoldId, Vec<MotifId>> {
    let mut idx: HashMap<FoldId, Vec<MotifId>> = HashMap::new();
    for mid in motifs {
        if let Some(m) = store.get(*mid) {
            if let Some(first) = m.tokens.first() {
                idx.entry(*first).or_default().push(*mid);
            }
        }
    }
    idx
}

/// Stats from encoding a corpus.
#[derive(Debug, Clone, Default)]
pub struct EncodingStats {
    pub articles_encoded: usize,
    pub original_ref_count: usize,
    pub encoded_ref_count: usize,
    pub motifs_used: usize,
}

impl EncodingStats {
    pub fn reduction_pct(&self) -> f32 {
        if self.original_ref_count == 0 { return 0.0; }
        100.0 * (1.0 - self.encoded_ref_count as f32 / self.original_ref_count as f32)
    }
}

/// End-to-end: mine motifs + encode all articles + return stats.
pub fn mine_and_encode(
    articles: &[Vec<FoldId>],
    config: &MiningConfig,
) -> (MotifStore, Vec<Vec<PathRef>>, EncodingStats) {
    let mined = mine_motifs(articles, config);
    let mut store = MotifStore::new();
    let mut motif_ids: Vec<MotifId> = Vec::new();
    for (m, _) in mined.iter() {
        motif_ids.push(store.add(m.clone()));
    }
    let index = index_by_first_token(&store, &motif_ids);

    let original_total: usize = articles.iter().map(|a| a.len()).sum();
    let mut encoded = Vec::with_capacity(articles.len());
    let mut encoded_total = 0usize;
    for a in articles {
        let e = encode_article_greedy(a, &store, &index);
        encoded_total += e.len();
        encoded.push(e);
    }

    let stats = EncodingStats {
        articles_encoded: articles.len(),
        original_ref_count: original_total,
        encoded_ref_count: encoded_total,
        motifs_used: store.len(),
    };
    (store, encoded, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fid(n: u64) -> FoldId {
        let mut bytes = [0u8; 16];
        bytes[..8].copy_from_slice(&n.to_le_bytes());
        FoldId(bytes)
    }

    #[test]
    fn motif_new_and_len() {
        let m = Motif::new(vec![fid(1), fid(2), fid(3)]);
        assert_eq!(m.len(), 3);
        assert!(!m.is_empty());
    }

    #[test]
    fn motif_store_dedup() {
        let mut s = MotifStore::new();
        let id1 = s.add(Motif::new(vec![fid(1), fid(2)]));
        let id2 = s.add(Motif::new(vec![fid(1), fid(2)]));  // same content
        assert_eq!(id1, id2, "same motif should get same id");
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn mine_finds_repeated_bigram() {
        let articles = vec![
            vec![fid(1), fid(2), fid(3), fid(1), fid(2)],
            vec![fid(4), fid(1), fid(2), fid(5)],
            vec![fid(1), fid(2), fid(6)],
        ];
        let cfg = MiningConfig { min_support: 3, min_motif_len: 2, max_motif_len: 3 };
        let mined = mine_motifs(&articles, &cfg);
        // [1,2] appears in all 3 articles → should be found
        let target = Motif::new(vec![fid(1), fid(2)]);
        assert!(mined.contains_key(&target), "should find bigram [1,2]");
        assert_eq!(mined[&target], 3);
    }

    #[test]
    fn mine_excludes_rare_bigrams() {
        let articles = vec![
            vec![fid(1), fid(2), fid(3)],
            vec![fid(4), fid(5), fid(6)],
        ];
        let cfg = MiningConfig { min_support: 3, min_motif_len: 2, max_motif_len: 2 };
        let mined = mine_motifs(&articles, &cfg);
        assert_eq!(mined.len(), 0, "no motif appears in 3+ articles");
    }

    #[test]
    fn encode_article_uses_motifs() {
        let mut store = MotifStore::new();
        let mid = store.add(Motif::new(vec![fid(1), fid(2)]));
        let index = index_by_first_token(&store, &[mid]);

        let article = vec![fid(1), fid(2), fid(3), fid(1), fid(2)];
        let encoded = encode_article_greedy(&article, &store, &index);
        // Expect: motif, token(3), motif
        assert_eq!(encoded.len(), 3);
        assert!(matches!(encoded[0], PathRef::Motif(_)));
        assert_eq!(encoded[1], PathRef::Token(fid(3)));
        assert!(matches!(encoded[2], PathRef::Motif(_)));
    }

    #[test]
    fn unfold_path_is_lossless() {
        let mut store = MotifStore::new();
        let mid = store.add(Motif::new(vec![fid(1), fid(2)]));
        let index = index_by_first_token(&store, &[mid]);
        let original = vec![fid(1), fid(2), fid(3), fid(1), fid(2), fid(4)];
        let encoded = encode_article_greedy(&original, &store, &index);
        let recovered = unfold_path(&encoded, &store);
        assert_eq!(recovered, original, "unfold must be lossless");
    }

    #[test]
    fn greedy_matches_longest_motif_first() {
        let mut store = MotifStore::new();
        let short = store.add(Motif::new(vec![fid(1), fid(2)]));
        let long = store.add(Motif::new(vec![fid(1), fid(2), fid(3)]));
        let index = index_by_first_token(&store, &[short, long]);
        let article = vec![fid(1), fid(2), fid(3), fid(4)];
        let encoded = encode_article_greedy(&article, &store, &index);
        // Should use the long motif, not the short
        assert_eq!(encoded.len(), 2);
        assert_eq!(encoded[0], PathRef::Motif(long));
        assert_eq!(encoded[1], PathRef::Token(fid(4)));
    }

    #[test]
    fn mine_and_encode_end_to_end() {
        let articles: Vec<Vec<FoldId>> = vec![
            vec![fid(1), fid(2), fid(3), fid(1), fid(2), fid(4)],
            vec![fid(1), fid(2), fid(5), fid(6)],
            vec![fid(1), fid(2), fid(7)],
            vec![fid(8), fid(9)],
        ];
        let cfg = MiningConfig { min_support: 3, min_motif_len: 2, max_motif_len: 2 };
        let (_store, _encoded, stats) = mine_and_encode(&articles, &cfg);
        assert!(stats.motifs_used >= 1);
        assert!(stats.encoded_ref_count < stats.original_ref_count,
            "motif encoding should reduce ref count");
    }

    #[test]
    fn encoding_stats_reduction_pct() {
        let stats = EncodingStats {
            articles_encoded: 1,
            original_ref_count: 100,
            encoded_ref_count: 65,
            motifs_used: 10,
        };
        let pct = stats.reduction_pct();
        assert!((pct - 35.0).abs() < 0.1, "expected ~35%, got {}", pct);
    }

    #[test]
    fn empty_article_handled_safely() {
        let articles: Vec<Vec<FoldId>> = vec![vec![]];
        let cfg = MiningConfig::default();
        let (_, encoded, stats) = mine_and_encode(&articles, &cfg);
        assert_eq!(encoded.len(), 1);
        assert_eq!(encoded[0].len(), 0);
        assert_eq!(stats.original_ref_count, 0);
    }

    #[test]
    fn no_motifs_means_all_tokens_pass_through() {
        let articles: Vec<Vec<FoldId>> = vec![
            vec![fid(1), fid(2), fid(3)],
            vec![fid(4), fid(5), fid(6)],
        ];
        // min_support=10 means no motifs will survive
        let cfg = MiningConfig { min_support: 10, min_motif_len: 2, max_motif_len: 2 };
        let (_, encoded, stats) = mine_and_encode(&articles, &cfg);
        assert_eq!(stats.motifs_used, 0);
        assert_eq!(stats.encoded_ref_count, stats.original_ref_count,
            "no motifs should mean no compression");
    }
}
