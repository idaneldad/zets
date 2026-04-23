//! Pattern dictionary — frequent N-tuples of edges compressed to short codes.
//!
//! This is the DIRECT answer to Idan's question:
//!   "Can common co-occurring edge states share a short bit-code?"
//!
//! Algorithm:
//! 1. Group edges by source_id
//! 2. For each source, compute the multiset of relations
//! 3. Count occurrences of each multiset (= pattern)
//! 4. Top N patterns get Huffman-coded pattern IDs
//! 5. When encoding, match source's edge set to a pattern and emit pattern_id + variable targets

use ahash::AHashMap;
use super::RawEdge;

/// A pattern = sorted multiset of relation codes.
pub type Pattern = Vec<u8>;

/// Pattern statistics after mining.
#[derive(Debug, Default)]
pub struct PatternDictionary {
    patterns: Vec<(Pattern, u64)>,   // (pattern, count), sorted by count desc
    pattern_idx: AHashMap<Pattern, u32>,
}

impl PatternDictionary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mine patterns from edges. Only keeps patterns occurring ≥ min_freq times.
    pub fn mine(edges: &[RawEdge], min_freq: u64) -> Self {
        let mut by_source: AHashMap<u32, Vec<u8>> = AHashMap::new();
        for e in edges {
            by_source.entry(e.source).or_default().push(e.relation);
        }

        let mut pattern_counts: AHashMap<Pattern, u64> = AHashMap::new();
        for (_, rels) in by_source.into_iter() {
            let mut p = rels;
            p.sort_unstable();
            *pattern_counts.entry(p).or_insert(0) += 1;
        }

        let mut patterns: Vec<(Pattern, u64)> = pattern_counts.into_iter()
            .filter(|(_, c)| *c >= min_freq)
            .collect();
        patterns.sort_by(|a, b| b.1.cmp(&a.1));

        let pattern_idx: AHashMap<Pattern, u32> = patterns.iter().enumerate()
            .map(|(i, (p, _))| (p.clone(), i as u32))
            .collect();

        Self { patterns, pattern_idx }
    }

    pub fn lookup(&self, pattern: &[u8]) -> Option<u32> {
        self.pattern_idx.get(pattern).copied()
    }

    pub fn get_pattern(&self, idx: u32) -> Option<&Pattern> {
        self.patterns.get(idx as usize).map(|(p, _)| p)
    }

    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    pub fn top_n(&self, n: usize) -> &[(Pattern, u64)] {
        &self.patterns[..self.patterns.len().min(n)]
    }
}

/// Compute savings estimate for using this dictionary on edges.
pub fn estimate_savings(dict: &PatternDictionary, edges: &[RawEdge]) -> (u64, u64) {
    // Returns (naive_bytes, compressed_bytes_estimate)
    let naive = (edges.len() * RawEdge::NAIVE_BYTES) as u64;

    let mut by_source: AHashMap<u32, Vec<u8>> = AHashMap::new();
    for e in edges {
        by_source.entry(e.source).or_default().push(e.relation);
    }

    let pattern_bits = if dict.len() > 0 {
        (64 - (dict.len() as u64).leading_zeros() as u64).max(1)
    } else {
        0
    };

    let mut compressed = 0u64;
    for (_, rels) in by_source {
        let mut p = rels.clone();
        p.sort_unstable();
        if dict.lookup(&p).is_some() {
            // pattern code + variable targets (avg 2 bytes per varint)
            compressed += pattern_bits / 8 + 1 + (rels.len() as u64) * 2;
        } else {
            // baseline
            compressed += (rels.len() as u64) * RawEdge::NAIVE_BYTES as u64;
        }
    }

    (naive, compressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mines_repeated_patterns() {
        // 1000 sources, all with pattern [0, 0, 1]
        let mut edges = Vec::new();
        for src in 0..1000u32 {
            edges.push(RawEdge { source: src, target: 100, relation: 0, weight: 1.0 });
            edges.push(RawEdge { source: src, target: 101, relation: 0, weight: 1.0 });
            edges.push(RawEdge { source: src, target: 102, relation: 1, weight: 1.0 });
        }
        let dict = PatternDictionary::mine(&edges, 100);
        assert!(dict.len() >= 1, "should find at least one pattern");
        let top = &dict.top_n(1)[0];
        assert_eq!(top.0, vec![0, 0, 1]);
        assert_eq!(top.1, 1000);
    }

    #[test]
    fn rare_patterns_excluded() {
        // Only 1 source with a unique pattern
        let edges = vec![
            RawEdge { source: 0, target: 1, relation: 5, weight: 1.0 },
            RawEdge { source: 0, target: 2, relation: 6, weight: 1.0 },
        ];
        let dict = PatternDictionary::mine(&edges, 100);
        assert_eq!(dict.len(), 0);
    }

    #[test]
    fn savings_estimate_beats_naive() {
        let mut edges = Vec::new();
        for src in 0..1000u32 {
            for r in 0u8..4 {
                edges.push(RawEdge { source: src, target: src + r as u32 * 1000, relation: r, weight: 1.0 });
            }
        }
        let dict = PatternDictionary::mine(&edges, 100);
        let (naive, compressed) = estimate_savings(&dict, &edges);
        println!("naive: {}  compressed: {}  ratio: {:.2}×",
                 naive, compressed, naive as f64 / compressed as f64);
        assert!(compressed < naive);
    }
}
