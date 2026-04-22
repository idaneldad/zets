//! Language families — factory functions that return pre-filled MorphologyCores.
//!
//! These are the "base classes" in class-inheritance terms, but implemented as
//! pure functions. Per-language factories call one of these and extend.
//!
//! NONE of these are registered as actual languages. They are only called
//! internally by the `languages::*` factories.

use super::core::{MorphologyCore, Typology};
#[allow(unused_imports)]
use super::rules::{PrefixFamily, PrefixRule, SuffixRule};
use super::core::Feature;

/// Semitic base — stacking prefix support on, no case-fold (Hebrew/Arabic scripts).
/// Does NOT add any specific letters — each language (Hebrew, Arabic) adds its own.
pub fn semitic(lang_code: &'static str) -> MorphologyCore {
    MorphologyCore::new(lang_code, Typology::Semitic)
        .stacking(true)
        .no_case_fold()
}

/// Romance base — gender/number endings common to the family.
/// Per-language factories override/extend the conjugation endings.
pub fn romance(lang_code: &'static str) -> MorphologyCore {
    MorphologyCore::new(lang_code, Typology::Romance).with_suffixes(vec![
        // Adverb formation (shared across Romance)
        SuffixRule {
            suffix: "mente",
            features: vec![Feature::Custom("adverb".into())],
            add_back: "",
            priority: 85,
            min_stem_chars: 3,
            requires_stem_ending: None,
            double_consonant_hint: false,
        },
    ])
}

/// Germanic base — light inflection typical of the family.
/// English/German/Dutch extend with their specific irregulars and endings.
pub fn germanic(lang_code: &'static str) -> MorphologyCore {
    MorphologyCore::new(lang_code, Typology::Germanic).with_suffixes(vec![
        // Present continuous / gerund
        SuffixRule {
            suffix: "ing",
            features: vec![Feature::Continuous],
            add_back: "",
            priority: 85,
            min_stem_chars: 3,
            requires_stem_ending: None,
            double_consonant_hint: true,
        },
        // Past / past participle
        SuffixRule {
            suffix: "ed",
            features: vec![Feature::Past],
            add_back: "",
            priority: 80,
            min_stem_chars: 3,
            requires_stem_ending: None,
            double_consonant_hint: true,
        },
        // Plural
        SuffixRule {
            suffix: "s",
            features: vec![Feature::Plural],
            add_back: "",
            priority: 70,
            min_stem_chars: 3,
            requires_stem_ending: None,
            double_consonant_hint: false,
        },
    ])
}

/// Isolating base — no affixes at all.
/// Vietnamese/Mandarin use particle_feature() helpers at the sentence level.
pub fn isolating(lang_code: &'static str) -> MorphologyCore {
    MorphologyCore::new(lang_code, Typology::Isolating)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semitic_has_stacking_enabled() {
        let m = semitic("he");
        assert!(m.stack_prefixes);
        assert!(!m.case_fold);
    }

    #[test]
    fn germanic_has_basic_suffixes() {
        let m = germanic("en").finalize();
        // Should match "-ing", "-ed", "-s"
        let a = m.analyze("running");
        assert!(a.iter().any(|x| x.lemma == "run"));
        let a = m.analyze("walked");
        assert!(a.iter().any(|x| x.lemma == "walk"));
        let a = m.analyze("cats");
        assert!(a.iter().any(|x| x.lemma == "cat"));
    }

    #[test]
    fn isolating_has_no_rules() {
        let m = isolating("vi");
        assert!(m.prefix_rules.is_empty());
        assert!(m.suffix_rules.is_empty());
    }

    #[test]
    fn romance_has_mente() {
        let m = romance("es").finalize();
        let a = m.analyze("rápidamente");
        assert!(a.iter().any(|x| x.lemma == "rápida"));
    }
}
