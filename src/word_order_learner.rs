//! Word-order learner — discovers syntactic rules from concept data alone.
//!
//! CORE INSIGHT: we do not hand-code "Hebrew is NounAdj, English is AdjNoun".
//! Instead, we scan all concept surfaces, find noun+adjective pairs that refer
//! to the same compound meaning, and measure the empirical frequency of each
//! ordering. A language's rule EMERGES from the data.
//!
//! Example pipeline for Hebrew:
//!   - Find concept "big" → surface "גדול"
//!   - Find concept "house" → surface "בית"
//!   - Find two-word surface in Hebrew containing both: "בית גדול"
//!   - Record: in this phrase, NOUN at position 0, ADJ at position 1
//!   - Repeat across 10,000 phrases. Compute P(NounFirst).
//!   - If P(NounFirst) > 0.7, the learned rule is "Hebrew = NounFirst".
//!
//! This is unsupervised learning from corpus statistics. No training labels,
//! no examples Claude wrote — just the observed distribution in the data.

use crate::concepts::ConceptStore;
use std::collections::HashMap;

/// The learned order preference for a language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearnedOrder {
    /// Adjective comes first. E.g. English "big house".
    AdjFirst,
    /// Noun comes first. E.g. Hebrew "בית גדול".
    NounFirst,
    /// No strong preference (<60% consensus). Language may be free-order
    /// or we have insufficient data.
    Undetermined,
}

/// Statistics from observing word orders in a language.
#[derive(Debug, Clone)]
pub struct LanguageOrderStats {
    pub lang: String,
    pub adj_first_count: u64,
    pub noun_first_count: u64,
    pub total_observed: u64,
    /// Empirical probability that the adjective came first.
    pub p_adj_first: f64,
    /// The rule we learned.
    pub learned_rule: LearnedOrder,
    /// How confident we are (0.0 to 1.0).
    pub confidence: f64,
}

/// Discovers word-order rules by scanning compositional phrases in the concept data.
///
/// It looks for concept_surfaces where `surface` contains SPACE-separated tokens
/// whose components are themselves known surfaces of noun/adjective concepts.
pub struct WordOrderLearner<'a> {
    cs: &'a ConceptStore,
    // For each language, build: surface text → (noun_concept, adj_concept)?
    // But we only care about phrases that split into known (noun, adj) component concepts.
    stats: HashMap<String, (u64, u64)>, // lang → (adj_first, noun_first)
    total_per_lang: HashMap<String, u64>,
}

impl<'a> WordOrderLearner<'a> {
    pub fn new(cs: &'a ConceptStore) -> Self {
        Self {
            cs,
            stats: HashMap::new(),
            total_per_lang: HashMap::new(),
        }
    }

    /// Scan all surface entries, find 2-word phrases, determine their pattern.
    pub fn learn(&mut self) -> Vec<LanguageOrderStats> {
        // Build reverse lookup once: (lang, single_word_surface) → Vec<(ConceptId, pos)>
        // We only care about noun and adj concepts.
        let mut single_word_index: HashMap<(String, String), Vec<(u32, String)>> = HashMap::new();

        // Walk through all concepts, looking at their surfaces to build the index
        for cid_u32 in self.cs.all_concept_ids() {
            let Some(concept) = self.cs.get_concept(crate::concepts::ConceptId(cid_u32)) else {
                continue;
            };
            if concept.pos != "noun" && concept.pos != "adj" {
                continue;
            }
            let surfaces = self.cs.surfaces_of(crate::concepts::ConceptId(cid_u32));
            for (lang, surface) in surfaces {
                // Only single-word surfaces for the index
                if !surface.contains(' ') {
                    single_word_index
                        .entry((lang, surface))
                        .or_default()
                        .push((cid_u32, concept.pos.clone()));
                }
            }
        }

        // Now walk all surfaces again, looking for 2-word phrases whose parts are in the index
        for cid_u32 in self.cs.all_concept_ids() {
            let surfaces = self.cs.surfaces_of(crate::concepts::ConceptId(cid_u32));
            for (lang, surface) in surfaces {
                // Only 2-token phrases
                let tokens: Vec<&str> = surface.split_whitespace().collect();
                if tokens.len() != 2 {
                    continue;
                }
                let tok0 = tokens[0].to_string();
                let tok1 = tokens[1].to_string();

                let empty = Vec::new();
                let concepts_0 = single_word_index
                    .get(&(lang.clone(), tok0.clone()))
                    .unwrap_or(&empty);
                let concepts_1 = single_word_index
                    .get(&(lang.clone(), tok1.clone()))
                    .unwrap_or(&empty);
                if concepts_0.is_empty() || concepts_1.is_empty() {
                    continue;
                }

                // Check: is exactly ONE of (tok0, tok1) a noun and the OTHER an adjective?
                let pos_sets_0: Vec<&str> = concepts_0.iter().map(|(_, p)| p.as_str()).collect();
                let pos_sets_1: Vec<&str> = concepts_1.iter().map(|(_, p)| p.as_str()).collect();

                let tok0_has_adj = pos_sets_0.contains(&"adj");
                let tok0_has_noun = pos_sets_0.contains(&"noun");
                let tok1_has_adj = pos_sets_1.contains(&"adj");
                let tok1_has_noun = pos_sets_1.contains(&"noun");

                // Unambiguous case 1: tok0=adj, tok1=noun → "AdjFirst"
                let adj_first = tok0_has_adj && !tok0_has_noun && tok1_has_noun && !tok1_has_adj;
                // Unambiguous case 2: tok0=noun, tok1=adj → "NounFirst"
                let noun_first = tok0_has_noun && !tok0_has_adj && tok1_has_adj && !tok1_has_noun;

                // Skip if both or neither possibility is true (ambiguous)
                if adj_first == noun_first {
                    continue;
                }

                let entry = self.stats.entry(lang.clone()).or_insert((0, 0));
                if adj_first {
                    entry.0 += 1;
                } else {
                    entry.1 += 1;
                }
                *self.total_per_lang.entry(lang).or_insert(0) += 1;
            }
        }

        // Compute results
        let mut results = Vec::new();
        for (lang, &(adj_first, noun_first)) in &self.stats {
            let total = adj_first + noun_first;
            if total == 0 {
                continue;
            }
            let p_adj_first = adj_first as f64 / total as f64;
            let (rule, confidence) = classify_rule(p_adj_first, total);
            results.push(LanguageOrderStats {
                lang: lang.clone(),
                adj_first_count: adj_first,
                noun_first_count: noun_first,
                total_observed: total,
                p_adj_first,
                learned_rule: rule,
                confidence,
            });
        }
        results.sort_by(|a, b| b.total_observed.cmp(&a.total_observed));
        results
    }
}

fn classify_rule(p_adj_first: f64, sample_size: u64) -> (LearnedOrder, f64) {
    // Small sample → low confidence
    let sample_factor = (sample_size as f64 / 100.0).min(1.0);
    let confidence = if p_adj_first >= 0.7 || p_adj_first <= 0.3 {
        (p_adj_first - 0.5).abs() * 2.0 * sample_factor
    } else {
        0.0
    };

    let rule = if p_adj_first >= 0.7 {
        LearnedOrder::AdjFirst
    } else if p_adj_first <= 0.3 {
        LearnedOrder::NounFirst
    } else {
        LearnedOrder::Undetermined
    };

    (rule, confidence)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_strong_adj_first() {
        let (rule, conf) = classify_rule(0.85, 1000);
        assert_eq!(rule, LearnedOrder::AdjFirst);
        assert!(conf > 0.5);
    }

    #[test]
    fn classify_strong_noun_first() {
        let (rule, conf) = classify_rule(0.15, 1000);
        assert_eq!(rule, LearnedOrder::NounFirst);
        assert!(conf > 0.5);
    }

    #[test]
    fn classify_undetermined() {
        let (rule, _) = classify_rule(0.50, 1000);
        assert_eq!(rule, LearnedOrder::Undetermined);
    }

    #[test]
    fn small_sample_lowers_confidence() {
        let (_, conf_big) = classify_rule(0.9, 1000);
        let (_, conf_small) = classify_rule(0.9, 10);
        assert!(conf_big > conf_small);
    }
}
