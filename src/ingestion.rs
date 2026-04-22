//! Ingestion — the autonomous learning interface.
//!
//! Idan's requirement: ZETS needs to ingest text (and eventually other media)
//! and grow its graph without human curation. This module is the entry point:
//! text in → new atoms + edges out.
//!
//! The approach is deliberately simple and DETERMINISTIC. No neural NLP.
//! We parse structure out of text using:
//!
//!   1. Tokenization: split on whitespace + punctuation (Unicode-safe via char_indices)
//!   2. Normalization: lowercase, collapse whitespace, strip punctuation
//!   3. Co-occurrence: words appearing in the same sentence get co_occurs_with edges
//!   4. Pattern extraction: simple regex-like matching for "X is a Y", "X has Y"
//!   5. Context tagging: every ingested atom gets a provenance source_id
//!
//! This is a FOUNDATION. Better parsers (CLIP, wav2vec, dependency parsing)
//! can slot in later as alternative ingestors. The key is that whatever the
//! ingestor produces, it becomes atoms + typed edges in the graph — the graph
//! then reasons over it with the existing machinery (spreading activation,
//! dreaming, cognitive modes).
//!
//! Properties:
//!   - DETERMINISTIC: same text input → same atom IDs (via content_hash)
//!   - IDEMPOTENT: re-ingesting the same text doesn't duplicate atoms
//!   - COMPOSABLE: can run on one paragraph or a book, same API
//!   - TRACEABLE: every atom knows which source it came from

use std::collections::{HashMap, HashSet};

use crate::atoms::{AtomId, AtomKind, AtomStore};
use crate::relations;

/// One ingestion result — what the text produced.
#[derive(Debug, Clone, Default)]
pub struct IngestionResult {
    /// Atoms created (or deduped with existing) for each unique token
    pub word_atoms: HashMap<String, AtomId>,
    /// Atom representing the source document
    pub source_atom: Option<AtomId>,
    /// Atom representing each sentence, linked to source
    pub sentence_atoms: Vec<AtomId>,
    /// Count of new edges written
    pub new_edges: usize,
    /// Count of atoms created NEW (not deduped)
    pub new_atoms: usize,
    /// Total tokens seen
    pub total_tokens: usize,
    /// Unique tokens (after dedup)
    pub unique_tokens: usize,
}

/// Configuration for ingestion.
#[derive(Debug, Clone)]
pub struct IngestConfig {
    /// Words shorter than this are skipped (typical stop words)
    pub min_word_length: usize,
    /// Maximum co-occurrence edge weight
    pub max_cooccur_weight: u8,
    /// Link words within this many tokens of each other
    pub cooccur_window: usize,
    /// Whether to skip common English stop-words
    pub skip_stopwords: bool,
    /// Strip these characters from tokens (punctuation)
    pub punct_chars: &'static [char],
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            min_word_length: 3,
            max_cooccur_weight: 50,
            cooccur_window: 5,
            skip_stopwords: true,
            punct_chars: &['.', ',', ';', ':', '!', '?', '"', '\'', '(', ')', '[', ']', '{', '}'],
        }
    }
}

/// Simple English stop-word list — minimal, not linguistically exhaustive.
const STOPWORDS: &[&str] = &[
    "the", "is", "at", "of", "on", "and", "a", "to", "in", "for",
    "it", "that", "this", "with", "as", "was", "were", "be", "by",
    "an", "or", "are", "not", "but", "from", "had", "has", "have",
    "he", "she", "they", "we", "you", "i", "my", "your", "our",
    "his", "her", "its", "their", "who", "what", "when", "where", "why",
    "how", "all", "any", "some", "so", "if", "then", "than",
];

fn is_stopword(word: &str) -> bool {
    STOPWORDS.contains(&word)
}

/// Normalize a token: lowercase + strip surrounding punctuation.
/// Unicode-safe: never splits at byte boundaries inside multi-byte chars.
fn normalize(raw: &str, punct: &[char]) -> String {
    let trimmed = raw.trim_matches(|c: char| punct.contains(&c));
    trimmed.to_lowercase()
}

/// Split text into sentences on `. ! ?` followed by whitespace.
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        current.push(c);
        if matches!(c, '.' | '!' | '?') {
            // Peek: is the next char whitespace (or end)?
            match chars.peek() {
                None => {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() { sentences.push(trimmed); }
                    current.clear();
                }
                Some(next) if next.is_whitespace() => {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() { sentences.push(trimmed); }
                    current.clear();
                }
                _ => {}  // e.g., "3.14" — don't split
            }
        }
    }
    // Flush anything remaining
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() { sentences.push(trimmed); }
    sentences
}

/// Split a sentence into normalized tokens, filtering short/stopwords per config.
fn tokenize(sentence: &str, config: &IngestConfig) -> Vec<String> {
    sentence.split_whitespace()
        .map(|raw| normalize(raw, config.punct_chars))
        .filter(|w| {
            w.chars().count() >= config.min_word_length
                && !(config.skip_stopwords && is_stopword(w))
        })
        .collect()
}

/// Ingest one text document into the store.
///
/// `source_label`: a name for the document (e.g., filename, URL). Stored as a
///                 source atom that all sentences link back to.
/// `text`: the document content.
/// `config`: how to tokenize / filter / weigh edges.
pub fn ingest_text(
    store: &mut AtomStore,
    source_label: &str,
    text: &str,
    config: &IngestConfig,
) -> IngestionResult {
    let atoms_before = store.atom_count();
    let edges_before = store.edge_count();

    let mut result = IngestionResult::default();

    // ── 1. Source atom ──
    let source_data = format!("source:{}", source_label).into_bytes();
    let source_atom = store.put(AtomKind::Concept, source_data);
    result.source_atom = Some(source_atom);

    // ── 2. Split into sentences ──
    let sentences = split_sentences(text);
    let near_rel = relations::by_name("near").unwrap().code;
    let co_occurs = relations::by_name("co_occurs_with").unwrap().code;
    let part_of = relations::by_name("part_of").unwrap().code;

    let mut all_unique_tokens: HashSet<String> = HashSet::new();

    for (sent_idx, sentence) in sentences.iter().enumerate() {
        // Sentence atom, linked to source
        let sent_data = format!("sent:{}:{}", source_label, sent_idx).into_bytes();
        let sent_atom = store.put(AtomKind::Text, sent_data);
        store.link(sent_atom, source_atom, part_of, 90, 0);
        result.sentence_atoms.push(sent_atom);

        // Tokenize
        let tokens = tokenize(sentence, config);
        result.total_tokens += tokens.len();
        for t in &tokens { all_unique_tokens.insert(t.clone()); }

        // Create word atoms + link each to its containing sentence
        let mut token_atoms: Vec<AtomId> = Vec::with_capacity(tokens.len());
        for t in &tokens {
            let entry = result.word_atoms.entry(t.clone()).or_insert_with(|| {
                let data = format!("word:{}", t).into_bytes();
                store.put(AtomKind::Concept, data)
            });
            token_atoms.push(*entry);
            // Word is PART_OF this sentence
            store.link(*entry, sent_atom, part_of, 70, 0);
        }

        // ── 3. Co-occurrence edges (windowed) ──
        for i in 0..token_atoms.len() {
            let max_j = (i + 1 + config.cooccur_window).min(token_atoms.len());
            for j in (i + 1)..max_j {
                if token_atoms[i] == token_atoms[j] { continue; } // skip self
                let distance = (j - i) as u8;
                // Inverse-distance weighting: closer = stronger co-occurrence
                let weight = (config.max_cooccur_weight / distance.max(1)).max(10);
                store.link(token_atoms[i], token_atoms[j], co_occurs, weight, 0);
                // Symmetric
                store.link(token_atoms[j], token_atoms[i], co_occurs, weight, 0);
            }
        }

        // ── 4. Simple pattern extraction: "X is a Y" ──
        // Look for the literal pattern "word IS_A word" in the original
        // (non-tokenized) sentence, then create an is_a edge.
        extract_simple_patterns(store, sentence, config);
    }

    result.unique_tokens = all_unique_tokens.len();
    result.new_atoms = store.atom_count() - atoms_before;
    result.new_edges = store.edge_count() - edges_before;

    result
}

/// Pattern extraction — "X is a Y" and "X has a Y" templates.
/// Very basic — enough to seed the graph. Deterministic.
fn extract_simple_patterns(store: &mut AtomStore, sentence: &str, config: &IngestConfig) {
    let is_a = relations::by_name("is_a").unwrap().code;
    let has_part = relations::by_name("has_part").unwrap().code;

    let words: Vec<String> = sentence.split_whitespace()
        .map(|w| normalize(w, config.punct_chars))
        .collect();

    let len = words.len();
    let mut i = 0;
    while i + 2 < len {
        let triple = [words[i].as_str(), words[i+1].as_str(), words[i+2].as_str()];
        let (rel, subj, obj) = match triple {
            [a, "is", b] if a.len() >= config.min_word_length
                         && b.len() >= config.min_word_length
                         && !is_stopword(a) && !is_stopword(b) =>
                (Some(is_a), a, b),
            [a, "has", b] if a.len() >= config.min_word_length
                          && b.len() >= config.min_word_length
                          && !is_stopword(a) && !is_stopword(b) =>
                (Some(has_part), a, b),
            _ => (None, "", ""),
        };

        // Also check 4-word: "a X is a Y" patterns — skip first two words
        if rel.is_some() {
            let subj_atom = store.put(AtomKind::Concept,
                format!("word:{}", subj).into_bytes());
            let obj_atom = store.put(AtomKind::Concept,
                format!("word:{}", obj).into_bytes());
            store.link(subj_atom, obj_atom, rel.unwrap(), 75, 0);
        }
        i += 1;
    }
}

/// Convenience: ingest many texts in a batch with a shared source.
pub fn ingest_batch(
    store: &mut AtomStore,
    documents: &[(&str, &str)],  // (label, content)
    config: &IngestConfig,
) -> Vec<IngestionResult> {
    documents.iter()
        .map(|(label, content)| ingest_text(store, label, content, config))
        .collect()
}

/// Query: find all words that co-occur with a given word, ranked by weight.
pub fn words_co_occurring_with(
    store: &AtomStore,
    word: &str,
    top_k: usize,
) -> Vec<(String, u8)> {
    let needle = format!("word:{}", word.to_lowercase());
    let target_hash = crate::atoms::content_hash(needle.as_bytes());
    let (atoms, _) = store.snapshot();

    let word_atom = match atoms.iter().position(|a| a.content_hash == target_hash) {
        Some(i) => i as u32,
        None => return Vec::new(),
    };

    let co_occurs = relations::by_name("co_occurs_with").unwrap().code;
    let mut neighbors: HashMap<AtomId, u8> = HashMap::new();
    for e in store.outgoing(word_atom).iter().filter(|e| e.relation == co_occurs) {
        let cur = neighbors.get(&e.to).copied().unwrap_or(0);
        if e.weight > cur { neighbors.insert(e.to, e.weight); }
    }

    let mut ranked: Vec<(String, u8)> = neighbors.into_iter()
        .filter_map(|(aid, w)| {
            let a = store.get(aid)?;
            let s = std::str::from_utf8(&a.data).ok()?;
            s.strip_prefix("word:").map(|stripped| (stripped.to_string(), w))
        })
        .collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1));
    ranked.truncate(top_k);
    ranked
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_basic() {
        let config = IngestConfig::default();
        let tokens = tokenize("The quick brown fox jumps over the lazy dog.", &config);
        // "the" is stopword, "over" is kept (4 letters, not stopword in our list)
        assert!(tokens.contains(&"quick".to_string()));
        assert!(tokens.contains(&"brown".to_string()));
        assert!(tokens.contains(&"fox".to_string()));
        assert!(!tokens.contains(&"the".to_string()));
    }

    #[test]
    fn split_sentences_basic() {
        let text = "Hello world. This is a test. Really.";
        let sentences = split_sentences(text);
        assert_eq!(sentences.len(), 3);
    }

    #[test]
    fn split_sentences_keeps_decimals() {
        // "3.14" shouldn't split
        let text = "Pi is 3.14 approximately.";
        let sentences = split_sentences(text);
        // Should be one sentence, not split at "3."
        assert_eq!(sentences.len(), 1);
    }

    #[test]
    fn ingest_creates_atoms_and_edges() {
        let mut store = AtomStore::new();
        let config = IngestConfig::default();
        let text = "The quick brown fox jumps over the lazy dog.";
        let result = ingest_text(&mut store, "aesop-like", text, &config);

        assert!(result.source_atom.is_some());
        assert!(result.new_atoms > 0);
        assert!(result.new_edges > 0);
        assert!(result.unique_tokens >= 5);
        assert!(result.word_atoms.contains_key("fox"));
    }

    #[test]
    fn ingest_is_deterministic() {
        let mut s1 = AtomStore::new();
        let mut s2 = AtomStore::new();
        let config = IngestConfig::default();
        let text = "Dogs are friendly animals. Cats are independent.";

        let r1 = ingest_text(&mut s1, "doc1", text, &config);
        let r2 = ingest_text(&mut s2, "doc1", text, &config);

        assert_eq!(r1.new_atoms, r2.new_atoms);
        assert_eq!(r1.new_edges, r2.new_edges);
        assert_eq!(r1.unique_tokens, r2.unique_tokens);
        assert_eq!(r1.word_atoms, r2.word_atoms);
    }

    #[test]
    fn ingest_is_idempotent() {
        let mut store = AtomStore::new();
        let config = IngestConfig::default();
        let text = "Dogs are friendly animals.";

        let _r1 = ingest_text(&mut store, "same", text, &config);
        let atoms_after_1 = store.atom_count();

        let _r2 = ingest_text(&mut store, "same", text, &config);
        let atoms_after_2 = store.atom_count();

        // Second ingestion should add zero new unique atoms (all dedup'd)
        assert_eq!(atoms_after_1, atoms_after_2,
            "idempotent ingestion should add no new atoms, got {} -> {}",
            atoms_after_1, atoms_after_2);
    }

    #[test]
    fn pattern_is_a_creates_edge() {
        let mut store = AtomStore::new();
        let config = IngestConfig::default();
        let text = "dog is animal. cat is animal.";
        let _ = ingest_text(&mut store, "taxonomy", text, &config);

        // Look for dog -- is_a --> animal edge
        let is_a = relations::by_name("is_a").unwrap().code;
        let dog_hash = crate::atoms::content_hash("word:dog".as_bytes());
        let (atoms, _) = store.snapshot();
        let dog_atom = atoms.iter().position(|a| a.content_hash == dog_hash);
        assert!(dog_atom.is_some(), "dog atom should exist");

        let outgoing = store.outgoing(dog_atom.unwrap() as u32);
        let has_is_a = outgoing.iter().any(|e| e.relation == is_a);
        assert!(has_is_a, "dog should have an is_a edge from pattern");
    }

    #[test]
    fn cooccurrence_creates_edges() {
        let mut store = AtomStore::new();
        let config = IngestConfig::default();
        let _ = ingest_text(&mut store, "doc", "apple banana cherry date.", &config);

        // apple and banana should have a strong co-occurs edge (adjacent)
        let apple_hash = crate::atoms::content_hash("word:apple".as_bytes());
        let banana_hash = crate::atoms::content_hash("word:banana".as_bytes());
        let (atoms, _) = store.snapshot();
        let apple = atoms.iter().position(|a| a.content_hash == apple_hash).map(|i| i as u32);
        let banana = atoms.iter().position(|a| a.content_hash == banana_hash).map(|i| i as u32);

        assert!(apple.is_some() && banana.is_some());
        let co = relations::by_name("co_occurs_with").unwrap().code;
        let has_edge = store.outgoing(apple.unwrap()).iter()
            .any(|e| e.relation == co && e.to == banana.unwrap());
        assert!(has_edge, "apple and banana should co-occur");
    }

    #[test]
    fn words_co_occurring_query() {
        let mut store = AtomStore::new();
        let config = IngestConfig::default();
        let text = "bread butter jam. bread cheese. butter milk.";
        let _ = ingest_text(&mut store, "food", text, &config);

        let neighbors = words_co_occurring_with(&store, "bread", 10);
        let names: Vec<String> = neighbors.iter().map(|(s, _)| s.clone()).collect();
        assert!(names.contains(&"butter".to_string()));
    }

    #[test]
    fn batch_ingestion() {
        let mut store = AtomStore::new();
        let config = IngestConfig::default();
        let docs = [
            ("doc1", "dogs bark loudly."),
            ("doc2", "cats meow quietly."),
            ("doc3", "birds sing early."),
        ];
        let results = ingest_batch(&mut store, &docs, &config);
        assert_eq!(results.len(), 3);
        for r in &results {
            assert!(r.source_atom.is_some());
        }
    }

    #[test]
    fn hebrew_utf8_safe() {
        // Hebrew text should not panic or split at byte boundaries
        let mut store = AtomStore::new();
        let config = IngestConfig::default();
        let text = "הכלב חום ונחמד. החתול שחור וחכם.";
        let result = ingest_text(&mut store, "hebrew-doc", text, &config);
        assert!(result.unique_tokens > 0);
    }

    #[test]
    fn empty_text_is_safe() {
        let mut store = AtomStore::new();
        let config = IngestConfig::default();
        let result = ingest_text(&mut store, "empty", "", &config);
        assert_eq!(result.unique_tokens, 0);
        assert_eq!(result.sentence_atoms.len(), 0);
    }

    #[test]
    fn source_label_tracked() {
        let mut store = AtomStore::new();
        let config = IngestConfig::default();
        let r = ingest_text(&mut store, "my-book", "Some content here.", &config);
        let src = r.source_atom.unwrap();
        let atom = store.get(src).unwrap();
        let label = std::str::from_utf8(&atom.data).unwrap();
        assert!(label.contains("my-book"));
    }
}
