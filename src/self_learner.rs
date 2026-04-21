//! SelfLearner — fully unsupervised graph builder.
//!
//! Given:
//!   1. A small `seed` lexicon (~100 words per language, POS labeled by hand)
//!   2. A surface corpus (multi-word phrases observed in real text)
//!
//! The learner builds a complete graph by **propagating** POS labels through
//! the data. Algorithm:
//!
//!   ROUND 1: All seed words have known POS.
//!   ROUND 2: For each 2-word phrase where both words are POS-known, learn
//!            the language's word-order rule (statistical majority).
//!   ROUND 3: For each phrase where ONE word is POS-known, use the learned
//!            rule to infer the other word's POS.
//!   REPEAT until convergence (no new labels added).
//!
//! Key property: the same algorithm works for ANY language, including
//! languages never seen before — provided we have a small seed.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// A single observation: word + language + inferred POS + how confident we are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnedWord {
    pub lang: String,
    pub word: String,
    pub pos: String,
    /// 1.0 = from seed (gold), <1.0 = inferred
    pub confidence: u8, // 0-100 scale (avoid float in keys)
}

/// The graph that gets built completely from scratch by the learner.
/// This is "Graph_B" — comparable against the original Graph_A (extracted by us).
#[derive(Debug, Default)]
pub struct LearnedGraph {
    /// (lang, word) → (pos, confidence 0-100)
    pub words: HashMap<(String, String), (String, u8)>,
    /// lang → discovered word-order rule
    /// Possible values: "adj_first", "noun_first", "undetermined"
    pub word_order: HashMap<String, String>,
    /// Per-language stats for the rule
    pub order_stats: HashMap<String, (u64, u64)>, // (adj_first_count, noun_first_count)
}

impl LearnedGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn word_count(&self) -> usize {
        self.words.len()
    }

    pub fn pos_for(&self, lang: &str, word: &str) -> Option<&str> {
        self.words
            .get(&(lang.to_string(), word.to_string()))
            .map(|(p, _)| p.as_str())
    }

    pub fn confidence_for(&self, lang: &str, word: &str) -> Option<u8> {
        self.words
            .get(&(lang.to_string(), word.to_string()))
            .map(|(_, c)| *c)
    }

    /// Insert a word only if not already present, OR upgrade if higher confidence.
    fn upsert(&mut self, lang: &str, word: &str, pos: &str, confidence: u8) -> bool {
        let key = (lang.to_string(), word.to_string());
        match self.words.get(&key) {
            Some((existing_pos, existing_conf)) => {
                if existing_pos == pos {
                    // Same POS → reinforce, take higher confidence
                    if confidence > *existing_conf {
                        self.words.insert(key, (pos.to_string(), confidence));
                        true
                    } else {
                        false
                    }
                } else {
                    // Conflict — only override if newer confidence is much higher
                    if confidence > existing_conf + 20 {
                        self.words.insert(key, (pos.to_string(), confidence));
                        true
                    } else {
                        false
                    }
                }
            }
            None => {
                self.words.insert(key, (pos.to_string(), confidence));
                true
            }
        }
    }
}

/// The learner itself: orchestrates rounds of propagation.
pub struct SelfLearner {
    pub graph: LearnedGraph,
    /// (lang, surface) → list of corpus phrases this surface appeared in
    /// For now we just keep the surfaces themselves.
    corpus_phrases: Vec<(String, String)>, // (lang, multi-word phrase)
    /// Round count for diagnostics
    pub rounds_run: u32,
}

impl SelfLearner {
    pub fn new() -> Self {
        Self {
            graph: LearnedGraph::new(),
            corpus_phrases: Vec::new(),
            rounds_run: 0,
        }
    }

    /// Load seed: words with known POS for one language.
    pub fn load_seed<P: AsRef<Path>>(&mut self, lang: &str, path: P) -> std::io::Result<usize> {
        let mut count = 0;
        for line in fs::read_to_string(path)?.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(2, '\t').collect();
            if parts.len() != 2 {
                continue;
            }
            let word = parts[0].trim();
            let pos = parts[1].trim();
            if word.is_empty() || pos.is_empty() {
                continue;
            }
            self.graph.upsert(lang, word, pos, 100);
            count += 1;
        }
        Ok(count)
    }

    /// Load corpus: surface phrases for analysis.
    /// Source: concept_surfaces.tsv (we use only the multi-word ones).
    pub fn load_corpus<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<usize> {
        let mut count = 0;
        for line in fs::read_to_string(path)?.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(3, '\t').collect();
            if parts.len() != 3 {
                continue;
            }
            let lang = parts[1].trim();
            let surface = parts[2].trim();
            // Only keep 2-word phrases (the simplest learnable structure)
            if surface.split_whitespace().count() == 2 {
                self.corpus_phrases.push((lang.to_string(), surface.to_string()));
                count += 1;
            }
        }
        Ok(count)
    }

    /// Run propagation rounds until convergence.
    pub fn learn(&mut self) -> LearnReport {
        let mut report = LearnReport::default();
        report.seed_size = self.graph.word_count();
        report.corpus_phrases = self.corpus_phrases.len();

        loop {
            let before = self.graph.word_count();

            // Step A: discover word-order rules from currently-known phrases
            self.discover_word_order();

            // Step B: propagate POS labels using known rules
            let added = self.propagate_pos();

            self.rounds_run += 1;
            report.rounds = self.rounds_run;

            if added == 0 || self.rounds_run > 10 {
                report.final_word_count = self.graph.word_count();
                report.words_inferred = report.final_word_count - report.seed_size;
                break;
            }

            let _ = before; // silence unused
        }

        // Final word-order discovery with all the words we now know
        self.discover_word_order();
        for (lang, (af, nf)) in &self.graph.order_stats {
            report.order_per_lang.push((lang.clone(), *af, *nf));
        }
        report
    }

    /// Discover order rule: count (adj noun) vs (noun adj) frequencies.
    fn discover_word_order(&mut self) {
        let mut counts: HashMap<String, (u64, u64)> = HashMap::new();

        for (lang, phrase) in &self.corpus_phrases {
            let toks: Vec<&str> = phrase.split_whitespace().collect();
            if toks.len() != 2 {
                continue;
            }
            let pos0 = self.graph.pos_for(lang, toks[0]);
            let pos1 = self.graph.pos_for(lang, toks[1]);
            match (pos0, pos1) {
                (Some("adj"), Some("noun")) => counts.entry(lang.clone()).or_insert((0, 0)).0 += 1,
                (Some("noun"), Some("adj")) => counts.entry(lang.clone()).or_insert((0, 0)).1 += 1,
                _ => {}
            }
        }

        for (lang, (af, nf)) in counts {
            let total = af + nf;
            if total == 0 {
                continue;
            }
            let p_adj = af as f64 / total as f64;
            let rule = if p_adj >= 0.65 {
                "adj_first"
            } else if p_adj <= 0.35 {
                "noun_first"
            } else {
                "undetermined"
            };
            self.graph.word_order.insert(lang.clone(), rule.to_string());
            self.graph.order_stats.insert(lang, (af, nf));
        }
    }

    /// Use known word-order rules to infer POS for unknown words.
    /// Returns number of new words added.
    fn propagate_pos(&mut self) -> usize {
        let mut new_inferences: Vec<(String, String, String, u8)> = Vec::new();

        for (lang, phrase) in &self.corpus_phrases {
            let rule = match self.graph.word_order.get(lang) {
                Some(r) if r != "undetermined" => r.clone(),
                _ => continue,
            };
            let toks: Vec<&str> = phrase.split_whitespace().collect();
            if toks.len() != 2 {
                continue;
            }
            let pos0 = self.graph.pos_for(lang, toks[0]).map(String::from);
            let pos1 = self.graph.pos_for(lang, toks[1]).map(String::from);

            match (pos0.as_deref(), pos1.as_deref(), rule.as_str()) {
                // adj_first language, position 0 known as adj → position 1 must be noun
                (Some("adj"), None, "adj_first") => {
                    new_inferences.push((lang.clone(), toks[1].to_string(), "noun".to_string(), 70));
                }
                // adj_first language, position 1 known as noun → position 0 must be adj
                (None, Some("noun"), "adj_first") => {
                    new_inferences.push((lang.clone(), toks[0].to_string(), "adj".to_string(), 70));
                }
                // noun_first language, position 0 known as noun → position 1 must be adj
                (Some("noun"), None, "noun_first") => {
                    new_inferences.push((lang.clone(), toks[1].to_string(), "adj".to_string(), 70));
                }
                // noun_first language, position 1 known as adj → position 0 must be noun
                (None, Some("adj"), "noun_first") => {
                    new_inferences.push((lang.clone(), toks[0].to_string(), "noun".to_string(), 70));
                }
                _ => {}
            }
        }

        // Apply all inferences
        let mut added = 0;
        for (lang, word, pos, conf) in new_inferences {
            if self.graph.upsert(&lang, &word, &pos, conf) {
                added += 1;
            }
        }
        added
    }
}

impl Default for SelfLearner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Clone)]
pub struct LearnReport {
    pub seed_size: usize,
    pub corpus_phrases: usize,
    pub rounds: u32,
    pub final_word_count: usize,
    pub words_inferred: usize,
    pub order_per_lang: Vec<(String, u64, u64)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_inserts_new() {
        let mut g = LearnedGraph::new();
        assert!(g.upsert("en", "dog", "noun", 100));
        assert_eq!(g.pos_for("en", "dog"), Some("noun"));
    }

    #[test]
    fn upsert_no_dup_same_pos() {
        let mut g = LearnedGraph::new();
        g.upsert("en", "dog", "noun", 100);
        // Same POS, lower confidence → no update
        assert!(!g.upsert("en", "dog", "noun", 70));
    }

    #[test]
    fn upsert_resists_low_conflict() {
        let mut g = LearnedGraph::new();
        g.upsert("en", "dog", "noun", 100);
        // Different POS, lower confidence → don't override
        assert!(!g.upsert("en", "dog", "verb", 70));
        assert_eq!(g.pos_for("en", "dog"), Some("noun"));
    }

    #[test]
    fn discover_works_on_clear_pattern() {
        let mut l = SelfLearner::new();
        // Seed
        l.graph.upsert("en", "big", "adj", 100);
        l.graph.upsert("en", "house", "noun", 100);
        l.graph.upsert("en", "small", "adj", 100);
        l.graph.upsert("en", "car", "noun", 100);
        // Phrases that show adj first
        l.corpus_phrases
            .push(("en".to_string(), "big house".to_string()));
        l.corpus_phrases
            .push(("en".to_string(), "small car".to_string()));
        l.discover_word_order();
        assert_eq!(l.graph.word_order.get("en"), Some(&"adj_first".to_string()));
    }

    #[test]
    fn propagate_infers_from_rule() {
        let mut l = SelfLearner::new();
        // Seed: only "big" known
        l.graph.upsert("en", "big", "adj", 100);
        // Rule already established
        l.graph.word_order.insert("en".to_string(), "adj_first".to_string());
        // Phrase: big + UNKNOWN
        l.corpus_phrases
            .push(("en".to_string(), "big tree".to_string()));
        let added = l.propagate_pos();
        assert_eq!(added, 1);
        assert_eq!(l.graph.pos_for("en", "tree"), Some("noun"));
    }
}
