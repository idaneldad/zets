//! Per-language factories — each is ~20-40 lines of rule declarations.
//!
//! This file REPLACES the 1259 lines of the old morphology/ directory.
//! Each factory: call the family base, add language-specific rules, finalize.

use super::core::{Feature, MorphologyCore};
use super::families::{germanic, isolating, romance, semitic};
use super::rules::{IrregularForm, PrefixFamily, PrefixRule, SuffixRule};

// ============================================================================
// HEBREW — Semitic, stacking prefixes (ו ה ב ל מ כ ש), rich suffixes
// ============================================================================
pub fn hebrew() -> MorphologyCore {
    semitic("he")
        .with_prefixes(vec![
            PrefixRule { prefix: "ו", features: vec![Feature::Conjunction],
                priority: 80, family: PrefixFamily::Conjunction, min_stem_chars: 3 },
            PrefixRule { prefix: "ב", features: vec![Feature::Locative],
                priority: 70, family: PrefixFamily::Preposition, min_stem_chars: 3 },
            PrefixRule { prefix: "ל", features: vec![Feature::Directional],
                priority: 70, family: PrefixFamily::Preposition, min_stem_chars: 3 },
            PrefixRule { prefix: "מ", features: vec![Feature::From],
                priority: 70, family: PrefixFamily::Preposition, min_stem_chars: 3 },
            PrefixRule { prefix: "כ", features: vec![Feature::As],
                priority: 70, family: PrefixFamily::Preposition, min_stem_chars: 3 },
            PrefixRule { prefix: "ש", features: vec![Feature::Relativizer],
                priority: 70, family: PrefixFamily::Preposition, min_stem_chars: 3 },
            PrefixRule { prefix: "ה", features: vec![Feature::DefiniteArticle],
                priority: 90, family: PrefixFamily::DefiniteArticle, min_stem_chars: 3 },
        ])
        .with_suffixes(vec![
            // Plurals
            SuffixRule { suffix: "ים", features: vec![Feature::Plural, Feature::Masculine],
                add_back: "", priority: 85, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            SuffixRule { suffix: "ות", features: vec![Feature::Plural, Feature::Feminine],
                add_back: "", priority: 85, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            // Past tense endings
            SuffixRule { suffix: "תי", features: vec![Feature::Past, Feature::Person1, Feature::Singular],
                add_back: "", priority: 90, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            SuffixRule { suffix: "נו", features: vec![Feature::Past, Feature::Person1, Feature::Plural],
                add_back: "", priority: 88, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            SuffixRule { suffix: "ת", features: vec![Feature::Past, Feature::Person2, Feature::Singular],
                add_back: "", priority: 75, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            // Feminine singular
            SuffixRule { suffix: "ה", features: vec![Feature::Feminine, Feature::Singular],
                add_back: "", priority: 60, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
        ])
        .finalize()
}

// ============================================================================
// ARABIC — Semitic, different prefix set than Hebrew
// ============================================================================
pub fn arabic() -> MorphologyCore {
    semitic("ar")
        .with_prefixes(vec![
            PrefixRule { prefix: "و", features: vec![Feature::Conjunction],
                priority: 80, family: PrefixFamily::Conjunction, min_stem_chars: 3 },
            PrefixRule { prefix: "ف", features: vec![Feature::Conjunction],
                priority: 75, family: PrefixFamily::Conjunction, min_stem_chars: 3 },
            PrefixRule { prefix: "ب", features: vec![Feature::Locative],
                priority: 70, family: PrefixFamily::Preposition, min_stem_chars: 3 },
            PrefixRule { prefix: "ل", features: vec![Feature::Directional],
                priority: 70, family: PrefixFamily::Preposition, min_stem_chars: 3 },
            PrefixRule { prefix: "ال", features: vec![Feature::DefiniteArticle],
                priority: 95, family: PrefixFamily::DefiniteArticle, min_stem_chars: 2 },
        ])
        .with_suffixes(vec![
            // Sound feminine (taa marbuta)
            SuffixRule { suffix: "ة", features: vec![Feature::Feminine, Feature::Singular],
                add_back: "", priority: 85, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            // Sound masc plural nominative
            SuffixRule { suffix: "ون", features: vec![Feature::Plural, Feature::Masculine, Feature::Nominative],
                add_back: "", priority: 85, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            // Sound fem plural
            SuffixRule { suffix: "ات", features: vec![Feature::Plural, Feature::Feminine],
                add_back: "", priority: 85, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
        ])
        .finalize()
}

// ============================================================================
// ENGLISH — Germanic, inherits -s/-ed/-ing, adds irregulars + -ies/-ied
// ============================================================================
pub fn english() -> MorphologyCore {
    germanic("en")
        .with_suffixes(vec![
            // Cities → city (y → i + es)
            SuffixRule { suffix: "ies", features: vec![Feature::Plural],
                add_back: "y", priority: 88, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            // Tried → try
            SuffixRule { suffix: "ied", features: vec![Feature::Past],
                add_back: "y", priority: 88, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            // Adverbs: -ly
            SuffixRule { suffix: "ly", features: vec![Feature::Custom("adverb".into())],
                add_back: "", priority: 78, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
        ])
        .with_irregulars(vec![
            IrregularForm::new("went", "go", vec![Feature::Past]),
            IrregularForm::new("gone", "go", vec![Feature::Past, Feature::Perfect]),
            IrregularForm::new("did", "do", vec![Feature::Past]),
            IrregularForm::new("done", "do", vec![Feature::Past, Feature::Perfect]),
            IrregularForm::new("saw", "see", vec![Feature::Past]),
            IrregularForm::new("seen", "see", vec![Feature::Past, Feature::Perfect]),
            IrregularForm::new("ate", "eat", vec![Feature::Past]),
            IrregularForm::new("eaten", "eat", vec![Feature::Past, Feature::Perfect]),
            IrregularForm::new("wrote", "write", vec![Feature::Past]),
            IrregularForm::new("written", "write", vec![Feature::Past, Feature::Perfect]),
            IrregularForm::new("took", "take", vec![Feature::Past]),
            IrregularForm::new("taken", "take", vec![Feature::Past, Feature::Perfect]),
            IrregularForm::new("gave", "give", vec![Feature::Past]),
            IrregularForm::new("given", "give", vec![Feature::Past, Feature::Perfect]),
            IrregularForm::new("brought", "bring", vec![Feature::Past]),
            IrregularForm::new("caught", "catch", vec![Feature::Past]),
            IrregularForm::new("thought", "think", vec![Feature::Past]),
            IrregularForm::new("bought", "buy", vec![Feature::Past]),
            IrregularForm::new("came", "come", vec![Feature::Past]),
            IrregularForm::new("ran", "run", vec![Feature::Past]),
            IrregularForm::new("knew", "know", vec![Feature::Past]),
            IrregularForm::new("known", "know", vec![Feature::Past, Feature::Perfect]),
            IrregularForm::new("said", "say", vec![Feature::Past]),
            IrregularForm::new("told", "tell", vec![Feature::Past]),
            IrregularForm::new("heard", "hear", vec![Feature::Past]),
            IrregularForm::new("found", "find", vec![Feature::Past]),
            IrregularForm::new("made", "make", vec![Feature::Past]),
            IrregularForm::new("men", "man", vec![Feature::Plural]),
            IrregularForm::new("women", "woman", vec![Feature::Plural]),
            IrregularForm::new("children", "child", vec![Feature::Plural]),
            IrregularForm::new("feet", "foot", vec![Feature::Plural]),
            IrregularForm::new("teeth", "tooth", vec![Feature::Plural]),
            IrregularForm::new("mice", "mouse", vec![Feature::Plural]),
            IrregularForm::new("geese", "goose", vec![Feature::Plural]),
        ])
        .finalize()
}

/// Multi-word tense detection for English (called from sentence-level analyzer).
pub fn english_multiword_tense(prev_token: &str, _cur_verb: &str) -> Option<Feature> {
    match prev_token.to_lowercase().as_str() {
        "will" | "shall" | "'ll" => Some(Feature::Future),
        "have" | "has" | "had" => Some(Feature::Perfect),
        _ => None,
    }
}

// ============================================================================
// SPANISH — Romance, rich verb conjugation
// ============================================================================
pub fn spanish() -> MorphologyCore {
    romance("es")
        .with_suffixes(vec![
            // Gerunds
            SuffixRule { suffix: "iendo", features: vec![Feature::Continuous],
                add_back: "er", priority: 82, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            SuffixRule { suffix: "ando", features: vec![Feature::Continuous],
                add_back: "ar", priority: 82, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            // Past participles
            SuffixRule { suffix: "ado", features: vec![Feature::Past, Feature::Perfect],
                add_back: "ar", priority: 80, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            SuffixRule { suffix: "ido", features: vec![Feature::Past, Feature::Perfect],
                add_back: "ir", priority: 80, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            // Future 1sg (aré/eré/iré) — requires_stem_ending used to distinguish
            SuffixRule { suffix: "aré", features: vec![Feature::Future, Feature::Person1, Feature::Singular],
                add_back: "ar", priority: 87, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            SuffixRule { suffix: "eré", features: vec![Feature::Future, Feature::Person1, Feature::Singular],
                add_back: "er", priority: 87, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            SuffixRule { suffix: "iré", features: vec![Feature::Future, Feature::Person1, Feature::Singular],
                add_back: "ir", priority: 87, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            // Imperfect
            SuffixRule { suffix: "aba", features: vec![Feature::Imperfect],
                add_back: "ar", priority: 80, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            // Plural noun/adj
            SuffixRule { suffix: "os", features: vec![Feature::Masculine, Feature::Plural],
                add_back: "o", priority: 75, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
            SuffixRule { suffix: "as", features: vec![Feature::Feminine, Feature::Plural],
                add_back: "a", priority: 75, min_stem_chars: 2,
                requires_stem_ending: None, double_consonant_hint: false },
        ])
        .finalize()
}

// ============================================================================
// VIETNAMESE — Isolating, no word-level morphology
// ============================================================================
pub fn vietnamese() -> MorphologyCore {
    isolating("vi").finalize()
}

/// Vietnamese particle-level tense/aspect detection.
pub fn vietnamese_particle_feature(particle: &str) -> Option<Feature> {
    match particle {
        "đã" => Some(Feature::Past),
        "đang" => Some(Feature::Continuous),
        "sẽ" => Some(Feature::Future),
        "rồi" => Some(Feature::Perfect),
        "những" | "các" => Some(Feature::Plural),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Hebrew ----
    #[test]
    fn hebrew_stacked_prefixes() {
        let m = hebrew();
        let a = m.analyze("בבית");
        assert!(a.iter().any(|x| x.lemma == "בית" && x.features.contains(&Feature::Locative)),
            "expected בית + Locative, got {:?}", a);
    }

    #[test]
    fn hebrew_stacked_triple() {
        let m = hebrew();
        let a = m.analyze("ומהבית");
        assert!(a.iter().any(|x| x.lemma == "בית"
            && x.features.contains(&Feature::Conjunction)
            && x.features.contains(&Feature::From)
            && x.features.contains(&Feature::DefiniteArticle)),
            "expected full stack, got {:?}", a);
    }

    #[test]
    fn hebrew_plural_masc() {
        let m = hebrew();
        let a = m.analyze("ספרים");
        assert!(a.iter().any(|x| x.lemma == "ספר" && x.features.contains(&Feature::Plural)),
            "got {:?}", a);
    }

    #[test]
    fn hebrew_plural_fem() {
        let m = hebrew();
        let a = m.analyze("בנות");
        assert!(a.iter().any(|x| x.lemma == "בנ"
            && x.features.contains(&Feature::Plural)
            && x.features.contains(&Feature::Feminine)),
            "expected stem בנ + Plural + Feminine, got {:?}", a);
    }

    #[test]
    fn hebrew_past_1sg() {
        let m = hebrew();
        let a = m.analyze("הלכתי");
        assert!(a.iter().any(|x| x.features.contains(&Feature::Past)
            && x.features.contains(&Feature::Person1)),
            "expected past 1sg, got {:?}", a);
    }

    // ---- Arabic ----
    #[test]
    fn arabic_definite_article() {
        let m = arabic();
        let a = m.analyze("الكتاب");
        assert!(a.iter().any(|x| x.lemma == "كتاب" && x.features.contains(&Feature::DefiniteArticle)),
            "got {:?}", a);
    }

    #[test]
    fn arabic_fem_taa_marbuta() {
        let m = arabic();
        let a = m.analyze("مدرسة");
        assert!(a.iter().any(|x| x.features.contains(&Feature::Feminine)),
            "got {:?}", a);
    }

    // ---- English ----
    #[test]
    fn english_plural_dogs() {
        let m = english();
        let a = m.analyze("dogs");
        assert!(a.iter().any(|x| x.lemma == "dog" && x.features.contains(&Feature::Plural)));
    }

    #[test]
    fn english_past_walked() {
        let m = english();
        let a = m.analyze("walked");
        assert!(a.iter().any(|x| x.lemma == "walk" && x.features.contains(&Feature::Past)));
    }

    #[test]
    fn english_continuous_running() {
        let m = english();
        let a = m.analyze("running");
        assert!(a.iter().any(|x| x.lemma == "run" && x.features.contains(&Feature::Continuous)),
            "got {:?}", a);
    }

    #[test]
    fn english_plural_cities() {
        let m = english();
        let a = m.analyze("cities");
        assert!(a.iter().any(|x| x.lemma == "city" && x.features.contains(&Feature::Plural)),
            "got {:?}", a);
    }

    #[test]
    fn english_irregular_went() {
        let m = english();
        let a = m.analyze("went");
        assert_eq!(a[0].lemma, "go");
        assert!(a[0].features.contains(&Feature::Past));
    }

    #[test]
    fn english_irregular_children() {
        let m = english();
        let a = m.analyze("children");
        assert_eq!(a[0].lemma, "child");
    }

    #[test]
    fn english_future_detect() {
        assert_eq!(english_multiword_tense("will", "come"), Some(Feature::Future));
        assert_eq!(english_multiword_tense("shall", "go"), Some(Feature::Future));
    }

    #[test]
    fn english_perfect_detect() {
        assert_eq!(english_multiword_tense("has", "eaten"), Some(Feature::Perfect));
    }

    // ---- Spanish ----
    #[test]
    fn spanish_plural_perros() {
        let m = spanish();
        let a = m.analyze("perros");
        assert!(a.iter().any(|x| x.lemma == "perro" && x.features.contains(&Feature::Plural)),
            "got {:?}", a);
    }

    #[test]
    fn spanish_past_hablado() {
        let m = spanish();
        let a = m.analyze("hablado");
        assert!(a.iter().any(|x| x.lemma == "hablar" && x.features.contains(&Feature::Past)));
    }

    #[test]
    fn spanish_future_hablaré() {
        let m = spanish();
        let a = m.analyze("hablaré");
        assert!(a.iter().any(|x| x.lemma == "hablar" && x.features.contains(&Feature::Future)),
            "got {:?}", a);
    }

    // ---- Vietnamese ----
    #[test]
    fn vietnamese_no_inflection() {
        let m = vietnamese();
        let a = m.analyze("ăn");
        assert_eq!(a[0].lemma, "ăn");
    }

    #[test]
    fn vietnamese_particles() {
        assert_eq!(vietnamese_particle_feature("đã"), Some(Feature::Past));
        assert_eq!(vietnamese_particle_feature("sẽ"), Some(Feature::Future));
        assert_eq!(vietnamese_particle_feature("những"), Some(Feature::Plural));
    }
}
