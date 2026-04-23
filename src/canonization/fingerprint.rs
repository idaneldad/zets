//! Fingerprint — content identity detection across languages and versions.
//!
//! A fingerprint has three components:
//!   - **Structural**: sentence boundaries, length ratios, punctuation patterns.
//!     Captures the "shape" of the text independent of language.
//!   - **Semantic**: concept-level hashes via the sense graph.
//!     Captures meaning regardless of surface wording.
//!   - **Length**: word count tier for fast pre-filtering.
//!
//! Design: stable across minor wording differences, sensitive to structural
//! changes. Two translations of the same verse should produce similar
//! fingerprints; a summary of a long text should not.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::sense_graph::{LanguageId, SenseStore};

/// Content fingerprint for variant detection.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fingerprint {
    /// Per-sentence structural hashes (sentence length ratios, punctuation).
    pub structural: Vec<u64>,
    /// Per-sentence semantic hashes (concept IDs from sense graph).
    pub semantic: Vec<u64>,
    /// Word count tier: 0 = ≤100, 1 = 101-1K, 2 = 1K-10K, 3 = 10K+.
    pub length: u32,
}

impl Fingerprint {
    pub fn empty() -> Self {
        Self {
            structural: Vec::new(),
            semantic: Vec::new(),
            length: 0,
        }
    }
}

/// Split text into sentences using basic punctuation boundaries.
/// Handles: '.', '!', '?', '׃' (Hebrew sof pasuk), '।' (Devanagari danda).
pub fn split_sentences(text: &str) -> Vec<&str> {
    let mut sentences = Vec::new();
    let mut start = 0;

    for (i, ch) in text.char_indices() {
        if ch == '.' || ch == '!' || ch == '?' || ch == '׃' || ch == '।' {
            let end = i + ch.len_utf8();
            let segment = text[start..end].trim();
            if !segment.is_empty() {
                sentences.push(segment);
            }
            start = end;
        }
    }

    // Trailing text without terminal punctuation
    let tail = text[start..].trim();
    if !tail.is_empty() {
        sentences.push(tail);
    }

    sentences
}

/// Count words in text (split on whitespace).
fn word_count(text: &str) -> u32 {
    text.split_whitespace().count() as u32
}

/// Map word count to a tier for fast pre-filtering.
fn length_tier(wc: u32) -> u32 {
    match wc {
        0..=100 => 0,
        101..=1000 => 1,
        1001..=10000 => 2,
        _ => 3,
    }
}

/// Compute a structural hash for a single sentence.
/// Captures: word count, average word length, punctuation density, first/last char class.
fn structural_hash(sentence: &str) -> u64 {
    let words: Vec<&str> = sentence.split_whitespace().collect();
    let wc = words.len() as u64;
    let total_chars: usize = words.iter().map(|w| w.chars().count()).sum();
    let avg_word_len = if wc > 0 { total_chars as u64 / wc } else { 0 };

    // Punctuation density (count of punct chars / total chars), quantized
    let punct_count = sentence.chars().filter(|c| c.is_ascii_punctuation()).count() as u64;
    let char_count = sentence.chars().count().max(1) as u64;
    let punct_ratio_q = (punct_count * 100) / char_count;

    let mut h = DefaultHasher::new();
    wc.hash(&mut h);
    avg_word_len.hash(&mut h);
    punct_ratio_q.hash(&mut h);
    h.finish()
}

/// Compute a semantic hash for a single sentence using the sense graph.
/// For each word, look up its senses and collect the sense IDs.
/// Words NOT in the sense store are skipped — the semantic hash captures
/// only resolved meanings, making it truly cross-lingual.
fn semantic_hash(sentence: &str, language: LanguageId, store: &SenseStore) -> u64 {
    let mut sense_ids: Vec<u32> = Vec::new();

    for word in sentence.split_whitespace() {
        // Strip punctuation from word edges
        let clean: String = word.chars()
            .filter(|c| !c.is_ascii_punctuation())
            .collect();
        if clean.is_empty() {
            continue;
        }

        if let Some(wid) = store.find_word(&clean, language) {
            let senses = store.senses_of(wid);
            sense_ids.extend(senses);
        }
        // Words not in sense store are intentionally skipped.
        // They contribute to structural fingerprint (word count, etc.)
        // but not to semantic fingerprint, preserving cross-lingual comparison.
    }

    sense_ids.sort();
    let mut h = DefaultHasher::new();
    sense_ids.hash(&mut h);
    h.finish()
}

/// Compute a full fingerprint for the given text.
pub fn compute_fingerprint(
    text: &str,
    language: LanguageId,
    store: &SenseStore,
) -> Fingerprint {
    let sentences = split_sentences(text);
    let wc = word_count(text);

    let structural: Vec<u64> = sentences.iter()
        .map(|s| structural_hash(s))
        .collect();

    let semantic: Vec<u64> = sentences.iter()
        .map(|s| semantic_hash(s, language, store))
        .collect();

    Fingerprint {
        structural,
        semantic,
        length: length_tier(wc),
    }
}

/// Compare two fingerprints and return a similarity score in [0.0, 1.0].
///
/// Algorithm:
///   similarity = structural_sim * 0.4 + semantic_sim * 0.6
///
/// Each component uses a set-overlap approach on the hash vectors:
/// |intersection| / |union| (Jaccard-like, but on multisets via sorted merge).
pub fn fingerprint_similarity(a: &Fingerprint, b: &Fingerprint) -> f32 {
    let struct_sim = hash_vec_similarity(&a.structural, &b.structural);
    let sem_sim = hash_vec_similarity(&a.semantic, &b.semantic);

    // Length tier mismatch penalty
    let length_penalty = if a.length == b.length { 1.0 } else { 0.85 };

    (struct_sim * 0.4 + sem_sim * 0.6) * length_penalty
}

/// Jaccard similarity on sorted hash vectors.
fn hash_vec_similarity(a: &[u64], b: &[u64]) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let mut a_sorted = a.to_vec();
    let mut b_sorted = b.to_vec();
    a_sorted.sort();
    b_sorted.sort();

    let mut i = 0;
    let mut j = 0;
    let mut intersection = 0u64;
    let mut union = 0u64;

    while i < a_sorted.len() && j < b_sorted.len() {
        if a_sorted[i] == b_sorted[j] {
            intersection += 1;
            union += 1;
            i += 1;
            j += 1;
        } else if a_sorted[i] < b_sorted[j] {
            union += 1;
            i += 1;
        } else {
            union += 1;
            j += 1;
        }
    }
    union += (a_sorted.len() - i + b_sorted.len() - j) as u64;

    if union == 0 { 1.0 } else { intersection as f32 / union as f32 }
}
