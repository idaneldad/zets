//! Sense graph — WordNet-style synset representation.
//!
//! **The שלום / hello problem** (Idan, 23.04.2026):
//!
//! שלום (Hebrew) has at least 3 senses:
//!   - greeting.open   (= hi, hello, bonjour)
//!   - greeting.close  (= goodbye, farewell, au revoir)
//!   - peace.state     (= peace, paix, Frieden)
//!
//! hello (English) has only ONE sense: greeting.open
//!
//! Therefore `word:שלום --SAME_AS--> word:hello` is WRONG.
//! The correct representation:
//!   word:שלום --expresses_sense--> sense:greeting.open  (shared)
//!   word:שלום --expresses_sense--> sense:greeting.close (NOT shared)
//!   word:שלום --expresses_sense--> sense:peace.state    (NOT shared)
//!   word:hello --expresses_sense--> sense:greeting.open (the only one)
//!
//! # Structure
//!
//! - `WordAtom`: a surface form in a specific language (שלום, hello, ciao)
//! - `SenseAtom`: an abstract meaning, language-independent (greeting.open)
//! - `LanguageAtom`: language identifier (he, en, it)
//! - edge `word → sense` (expresses_sense, with optional register/usage flags)
//! - edge `word → language` (in_language)
//! - edge `sense → sense` (broader_than, opposite_of, causes, ...)
//!
//! # Why this matters for cross-lingual search
//!
//! Query "שלום" now returns 3 senses. User picks one (or system infers from
//! context). The chosen sense then matches words in ANY other language that
//! express it. This is how humans actually translate — by meaning, not by
//! surface string.

use std::collections::{HashMap, HashSet};

/// Unique ID for a word atom (surface form in a language).
pub type WordId = u32;

/// Unique ID for a sense atom (abstract meaning).
pub type SenseId = u32;

/// Unique ID for a language atom.
pub type LanguageId = u16;

/// A word atom — surface form in a specific language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordAtom {
    pub id: WordId,
    pub surface: String,       // "שלום", "hello"
    pub language: LanguageId,
}

/// A sense atom — abstract, language-independent meaning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SenseAtom {
    pub id: SenseId,
    pub key: String,           // "greeting.open", "peace.state"
    pub gloss: String,         // human-readable definition
    pub domain: Option<String>, // "greeting", "emotion", "weather", ...
}

/// How a word expresses a sense.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    Neutral,        // standard usage
    Formal,         // "shalom aleichem"
    Informal,       // "hey"
    Literary,       // "farewell"
    Regional,       // dialect-specific
    Archaic,        // obsolete
}

/// Edge from word to sense. A word can have many senses (polysemy).
#[derive(Debug, Clone)]
pub struct ExpressesSense {
    pub word: WordId,
    pub sense: SenseId,
    pub register: Register,
    pub frequency: f32,    // 0.0-1.0: how often this word expresses this sense vs others
}

/// Edge between senses (hierarchical or relational).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SenseRelation {
    /// "animal" is broader than "dog"
    BroaderThan,
    /// "war" is opposite of "peace"
    OppositeOf,
    /// "greeting.open" and "greeting.close" are related by domain
    SameDomain,
    /// "fire" causes "smoke"
    Causes,
    /// "greeting" is a kind of "social_ritual"
    KindOf,
}

/// The sense store — indexes words, senses, languages.
#[derive(Debug, Default)]
pub struct SenseStore {
    words: Vec<WordAtom>,
    senses: Vec<SenseAtom>,
    /// word_id → senses it expresses
    word_to_senses: HashMap<WordId, Vec<ExpressesSense>>,
    /// sense_id → words that express it (across all languages)
    sense_to_words: HashMap<SenseId, Vec<WordId>>,
    /// sense-to-sense relations
    sense_relations: Vec<(SenseId, SenseId, SenseRelation)>,
    /// surface+language → word_id (fast lookup)
    surface_index: HashMap<(String, LanguageId), WordId>,
    /// sense key → sense_id
    sense_index: HashMap<String, SenseId>,
}

impl SenseStore {
    pub fn new() -> Self { Self::default() }

    pub fn add_word(&mut self, surface: String, language: LanguageId) -> WordId {
        let key = (surface.clone(), language);
        if let Some(&id) = self.surface_index.get(&key) {
            return id;
        }
        let id = self.words.len() as WordId;
        self.words.push(WordAtom { id, surface, language });
        self.surface_index.insert(key, id);
        id
    }

    pub fn add_sense(&mut self, key: String, gloss: String,
                     domain: Option<String>) -> SenseId {
        if let Some(&id) = self.sense_index.get(&key) {
            return id;
        }
        let id = self.senses.len() as SenseId;
        self.senses.push(SenseAtom { id, key: key.clone(), gloss, domain });
        self.sense_index.insert(key, id);
        id
    }

    pub fn link_word_to_sense(&mut self, word: WordId, sense: SenseId,
                              register: Register, frequency: f32) {
        self.word_to_senses.entry(word).or_default().push(
            ExpressesSense { word, sense, register, frequency }
        );
        let sense_words = self.sense_to_words.entry(sense).or_default();
        if !sense_words.contains(&word) {
            sense_words.push(word);
        }
    }

    pub fn add_sense_relation(&mut self, a: SenseId, b: SenseId,
                              rel: SenseRelation) {
        self.sense_relations.push((a, b, rel));
    }

    pub fn get_word(&self, id: WordId) -> Option<&WordAtom> {
        self.words.get(id as usize)
    }

    pub fn get_sense(&self, id: SenseId) -> Option<&SenseAtom> {
        self.senses.get(id as usize)
    }

    pub fn find_word(&self, surface: &str, lang: LanguageId) -> Option<WordId> {
        self.surface_index.get(&(surface.to_string(), lang)).copied()
    }

    pub fn find_sense(&self, key: &str) -> Option<SenseId> {
        self.sense_index.get(key).copied()
    }

    /// All senses expressed by a word.
    pub fn senses_of(&self, word: WordId) -> Vec<SenseId> {
        self.word_to_senses.get(&word)
            .map(|v| v.iter().map(|e| e.sense).collect())
            .unwrap_or_default()
    }

    /// All words expressing a sense, optionally in a specific language.
    pub fn words_for_sense(&self, sense: SenseId,
                           lang: Option<LanguageId>) -> Vec<WordId> {
        self.sense_to_words.get(&sense).map(|words| {
            words.iter().copied()
                .filter(|&wid| {
                    match lang {
                        None => true,
                        Some(l) => self.get_word(wid).map(|w| w.language == l).unwrap_or(false),
                    }
                })
                .collect()
        }).unwrap_or_default()
    }

    /// **The key cross-lingual query:**
    /// "What senses do word_a and word_b SHARE?"
    /// Returns the intersection — which is how humans actually compare words.
    pub fn shared_senses(&self, word_a: WordId, word_b: WordId) -> Vec<SenseId> {
        let a: HashSet<SenseId> = self.senses_of(word_a).into_iter().collect();
        let b: HashSet<SenseId> = self.senses_of(word_b).into_iter().collect();
        a.intersection(&b).copied().collect()
    }

    /// True cross-lingual equivalence: words share ALL senses.
    pub fn are_full_synonyms(&self, a: WordId, b: WordId) -> bool {
        let sa: HashSet<SenseId> = self.senses_of(a).into_iter().collect();
        let sb: HashSet<SenseId> = self.senses_of(b).into_iter().collect();
        !sa.is_empty() && sa == sb
    }

    /// Partial overlap: words share SOME but not all senses.
    pub fn are_partial_synonyms(&self, a: WordId, b: WordId) -> bool {
        let shared = self.shared_senses(a, b);
        !shared.is_empty() && !self.are_full_synonyms(a, b)
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        (self.words.len(), self.senses.len(),
         self.word_to_senses.values().map(|v| v.len()).sum())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HE: LanguageId = 0;
    const EN: LanguageId = 1;
    const FR: LanguageId = 2;

    fn setup_shalom_hello_world() -> (SenseStore, WordId, WordId, WordId, WordId) {
        let mut s = SenseStore::new();

        // Senses
        let greet_open = s.add_sense("greeting.open".to_string(),
            "opening a conversation".to_string(), Some("greeting".to_string()));
        let greet_close = s.add_sense("greeting.close".to_string(),
            "ending a conversation".to_string(), Some("greeting".to_string()));
        let peace = s.add_sense("peace.state".to_string(),
            "absence of conflict".to_string(), Some("state".to_string()));

        // Hebrew: שלום covers all 3 senses
        let shalom = s.add_word("שלום".to_string(), HE);
        s.link_word_to_sense(shalom, greet_open, Register::Neutral, 0.5);
        s.link_word_to_sense(shalom, greet_close, Register::Neutral, 0.3);
        s.link_word_to_sense(shalom, peace, Register::Literary, 0.2);

        // English: hello = greet_open only; goodbye = greet_close; peace = peace
        let hello = s.add_word("hello".to_string(), EN);
        s.link_word_to_sense(hello, greet_open, Register::Neutral, 1.0);

        let goodbye = s.add_word("goodbye".to_string(), EN);
        s.link_word_to_sense(goodbye, greet_close, Register::Neutral, 1.0);

        let peace_en = s.add_word("peace".to_string(), EN);
        s.link_word_to_sense(peace_en, peace, Register::Neutral, 1.0);

        (s, shalom, hello, goodbye, peace_en)
    }

    #[test]
    fn shalom_has_three_senses() {
        let (s, shalom, _, _, _) = setup_shalom_hello_world();
        let senses = s.senses_of(shalom);
        assert_eq!(senses.len(), 3,
            "שלום should have 3 senses, got {}", senses.len());
    }

    #[test]
    fn hello_has_one_sense() {
        let (s, _, hello, _, _) = setup_shalom_hello_world();
        let senses = s.senses_of(hello);
        assert_eq!(senses.len(), 1);
    }

    #[test]
    fn shalom_and_hello_share_exactly_one_sense() {
        let (s, shalom, hello, _, _) = setup_shalom_hello_world();
        let shared = s.shared_senses(shalom, hello);
        assert_eq!(shared.len(), 1, "shalom and hello share exactly greeting.open");
        let sense = s.get_sense(shared[0]).unwrap();
        assert_eq!(sense.key, "greeting.open");
    }

    #[test]
    fn shalom_and_hello_are_NOT_full_synonyms() {
        let (s, shalom, hello, _, _) = setup_shalom_hello_world();
        assert!(!s.are_full_synonyms(shalom, hello),
            "שלום and hello must NOT be full synonyms");
    }

    #[test]
    fn shalom_and_hello_ARE_partial_synonyms() {
        let (s, shalom, hello, _, _) = setup_shalom_hello_world();
        assert!(s.are_partial_synonyms(shalom, hello),
            "שלום and hello should be partial synonyms (share greeting.open only)");
    }

    #[test]
    fn shalom_and_goodbye_also_partial() {
        let (s, shalom, _, goodbye, _) = setup_shalom_hello_world();
        // Both share greeting.close but שלום also has other senses
        assert!(s.are_partial_synonyms(shalom, goodbye));
        let shared = s.shared_senses(shalom, goodbye);
        assert_eq!(shared.len(), 1);
        let sense = s.get_sense(shared[0]).unwrap();
        assert_eq!(sense.key, "greeting.close");
    }

    #[test]
    fn words_for_greeting_open_across_languages() {
        let (mut s, _, _, _, _) = setup_shalom_hello_world();
        // Add French: bonjour
        let bonjour = s.add_word("bonjour".to_string(), FR);
        let open = s.find_sense("greeting.open").unwrap();
        s.link_word_to_sense(bonjour, open, Register::Neutral, 1.0);

        let all = s.words_for_sense(open, None);
        assert_eq!(all.len(), 3, "שלום, hello, bonjour all express greeting.open");

        let en_only = s.words_for_sense(open, Some(EN));
        assert_eq!(en_only.len(), 1, "only hello in English");

        let he_only = s.words_for_sense(open, Some(HE));
        assert_eq!(he_only.len(), 1, "only שלום in Hebrew");
    }

    #[test]
    fn sense_relations_can_link_senses() {
        let (mut s, _, _, _, _) = setup_shalom_hello_world();
        let open = s.find_sense("greeting.open").unwrap();
        let close = s.find_sense("greeting.close").unwrap();
        let peace = s.find_sense("peace.state").unwrap();

        // open and close are same-domain (greeting)
        s.add_sense_relation(open, close, SenseRelation::SameDomain);

        // "war" sense opposite to "peace"
        let war = s.add_sense("war.state".to_string(),
            "organized conflict".to_string(), Some("state".to_string()));
        s.add_sense_relation(war, peace, SenseRelation::OppositeOf);

        assert_eq!(s.sense_relations.len(), 2);
    }

    #[test]
    fn add_word_dedup() {
        let mut s = SenseStore::new();
        let a = s.add_word("שלום".to_string(), HE);
        let b = s.add_word("שלום".to_string(), HE);
        assert_eq!(a, b);
        // Same surface in different language is different word
        let c = s.add_word("שלום".to_string(), EN);
        assert_ne!(a, c);
    }

    #[test]
    fn stats_count_correctly() {
        let (s, _, _, _, _) = setup_shalom_hello_world();
        let (words, senses, links) = s.stats();
        assert_eq!(words, 4);  // שלום, hello, goodbye, peace
        assert_eq!(senses, 3); // greeting.open, greeting.close, peace.state
        assert_eq!(links, 6);  // 3+1+1+1
    }

    #[test]
    fn finding_nonexistent_returns_none() {
        let s = SenseStore::new();
        assert!(s.find_word("missing", HE).is_none());
        assert!(s.find_sense("missing.sense").is_none());
    }
}
