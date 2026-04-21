//! Phrase-level composition using concepts + per-language word-order rules.
//!
//! This is the module that realizes Idan's vision:
//! "תיק גדול" (Hebrew) and "big bag" (English) are the SAME phrase concept,
//! just rendered differently based on each language's syntactic rules.

use crate::concepts::{ConceptId, ConceptStore};
use crate::syntax_rules::compose_noun_adj;

/// A compositional phrase made of two concepts (noun + adjective).
#[derive(Debug, Clone)]
pub struct Phrase {
    pub noun_concept: ConceptId,
    pub adj_concept: ConceptId,
}

impl Phrase {
    pub fn new(noun: ConceptId, adj: ConceptId) -> Self {
        Self {
            noun_concept: noun,
            adj_concept: adj,
        }
    }
}

/// The rendering of a Phrase in a specific language.
#[derive(Debug, Clone)]
pub struct PhraseRealization {
    pub lang: String,
    pub text: String,
    pub noun_surface: String,
    pub adj_surface: String,
}

/// Compose a phrase (noun + adjective) in any loaded language.
pub struct PhraseComposer<'a> {
    concepts: &'a ConceptStore,
}

impl<'a> PhraseComposer<'a> {
    pub fn new(concepts: &'a ConceptStore) -> Self {
        Self { concepts }
    }

    /// Render the phrase in one specific language.
    /// Returns None if either concept has no surface in this language.
    pub fn realize_in(&self, phrase: &Phrase, lang: &str) -> Option<PhraseRealization> {
        let noun_surfaces = self.concepts.surfaces_of_in(phrase.noun_concept, lang);
        let adj_surfaces = self.concepts.surfaces_of_in(phrase.adj_concept, lang);
        let noun = noun_surfaces.first()?.clone();
        let adj = adj_surfaces.first()?.clone();
        let text = compose_noun_adj(lang, &noun, &adj);
        Some(PhraseRealization {
            lang: lang.to_string(),
            text,
            noun_surface: noun,
            adj_surface: adj,
        })
    }

    /// Render the phrase in all 10 supported languages.
    /// Returns a Vec in canonical language order.
    pub fn realize_all(&self, phrase: &Phrase) -> Vec<PhraseRealization> {
        const LANGS: &[&str] = &[
            "en", "he", "de", "fr", "es", "it", "ar", "ru", "nl", "pt",
        ];
        LANGS
            .iter()
            .filter_map(|l| self.realize_in(phrase, l))
            .collect()
    }

    /// Compose a phrase from words: "big house" [en] → try to build a Phrase.
    ///
    /// Uses POS-filtered concept lookup AND picks the concept with most language
    /// coverage (canonical meaning) over rare specialized senses.
    pub fn compose_from_words(
        &self,
        lang: &str,
        noun_word: &str,
        adj_word: &str,
    ) -> Option<Phrase> {
        // Pick noun: POS-filtered + most-covered
        let noun = self.concepts.best_concept_for_pos(lang, noun_word, "noun")?;
        // Pick adjective: POS-filtered + most-covered
        let adj = self.concepts.best_concept_for_pos(lang, adj_word, "adj")?;
        Some(Phrase::new(noun, adj))
    }

    /// Translate a two-word phrase by parsing it in source language's word order,
    /// resolving to concepts, and rendering in target language's word order.
    ///
    /// Example:
    ///   translate_phrase("en", "big house", "he") → "בית גדול"
    ///   translate_phrase("he", "בית גדול", "en") → "big house"
    pub fn translate_phrase(
        &self,
        from_lang: &str,
        phrase: &str,
        to_lang: &str,
    ) -> Option<PhraseRealization> {
        use crate::syntax_rules::split_noun_adj;
        let (noun_word, adj_word) = split_noun_adj(from_lang, phrase)?;
        let p = self.compose_from_words(from_lang, noun_word, adj_word)?;
        self.realize_in(&p, to_lang)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // tests that don't need data are already covered by syntax_rules::tests;
    // tests for PhraseComposer require a loaded ConceptStore → integration test.
}
