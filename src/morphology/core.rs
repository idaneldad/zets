//! MorphologyCore — the unified analyzer.
//!
//! One struct, one analyze() function, drives ALL languages.
//! Per-language behavior is determined entirely by the rules installed.

use std::collections::HashMap;

use super::rules::{IrregularForm, PrefixFamily, PrefixRule, SuffixRule};

/// Grammatical features detectable by morphology.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Feature {
    // Number
    Singular, Plural, Dual,
    // Gender
    Masculine, Feminine, Neuter,
    // Person
    Person1, Person2, Person3,
    // Tense
    Past, Present, Future, Imperfect, Preterite, Conditional, Subjunctive,
    // Aspect
    Continuous, Perfect,
    // Mood
    Imperative,
    // Case
    Nominative, Accusative, Genitive, Dative,
    // Affix semantics (Semitic/Arabic)
    DefiniteArticle, Locative, Directional, From, As, Relativizer, Conjunction, Possessive,
    // Register
    Formal, Colloquial, Dialectal,
    // Escape hatch
    Custom(String),
}

/// Language typology — describes the morphological style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Typology {
    /// Hebrew, Arabic — root-and-pattern + rich prefix system.
    Semitic,
    /// Spanish, French, Italian — fusional verb conjugation.
    Romance,
    /// English, German, Dutch — moderate inflection.
    Germanic,
    /// Vietnamese, Mandarin — no word-level inflection, particles only.
    Isolating,
    /// Agglutinative languages (Turkish, Finnish, Japanese).
    Agglutinative,
    /// Slavic — case + aspect rich.
    Slavic,
}

impl Typology {
    pub fn description(self) -> &'static str {
        match self {
            Self::Semitic => "Semitic: root-and-pattern, stacking prefixes",
            Self::Romance => "Romance: fusional verb conjugation, gender on adj",
            Self::Germanic => "Germanic: moderate inflection, -s/-ed/-ing",
            Self::Isolating => "Isolating: no word-level inflection, particles only",
            Self::Agglutinative => "Agglutinative: long suffix chains",
            Self::Slavic => "Slavic: case + aspect + gender",
        }
    }
}

/// Result of analyzing a surface word.
#[derive(Debug, Clone)]
pub struct Analysis {
    pub lemma: String,
    pub features: Vec<Feature>,
    pub confidence: u8, // 0-100
}

impl Analysis {
    pub fn identity(word: &str) -> Self {
        Self {
            lemma: word.to_string(),
            features: Vec::new(),
            confidence: 100,
        }
    }
}

/// The unified morphology engine.
/// One instance per language, built by a factory function.
#[derive(Debug, Clone)]
pub struct MorphologyCore {
    pub lang_code: &'static str,
    pub typology: Typology,

    pub prefix_rules: Vec<PrefixRule>,
    pub suffix_rules: Vec<SuffixRule>,
    pub irregulars: HashMap<String, IrregularForm>,

    /// If the analyzer should lowercase before processing (English yes, Hebrew no).
    pub case_fold: bool,
    /// Minimum word length before we attempt any stripping.
    pub min_word_chars: usize,
    /// Allow stacking of multiple prefixes (Hebrew does, English doesn't).
    pub stack_prefixes: bool,
}

impl MorphologyCore {
    /// Build an empty core for a given language + typology.
    pub fn new(lang_code: &'static str, typology: Typology) -> Self {
        Self {
            lang_code,
            typology,
            prefix_rules: Vec::new(),
            suffix_rules: Vec::new(),
            irregulars: HashMap::new(),
            case_fold: true,
            min_word_chars: 3,
            stack_prefixes: false,
        }
    }

    /// Convenient: add a prefix rule. Returns self for chaining.
    pub fn with_prefix(mut self, r: PrefixRule) -> Self {
        self.prefix_rules.push(r);
        self
    }

    /// Add many prefix rules.
    pub fn with_prefixes(mut self, rs: Vec<PrefixRule>) -> Self {
        self.prefix_rules.extend(rs);
        self
    }

    pub fn with_suffix(mut self, r: SuffixRule) -> Self {
        self.suffix_rules.push(r);
        self
    }

    pub fn with_suffixes(mut self, rs: Vec<SuffixRule>) -> Self {
        self.suffix_rules.extend(rs);
        self
    }

    pub fn with_irregulars(mut self, rs: Vec<IrregularForm>) -> Self {
        for r in rs {
            self.irregulars.insert(r.surface.to_string(), r);
        }
        self
    }

    pub fn stacking(mut self, enable: bool) -> Self {
        self.stack_prefixes = enable;
        self
    }

    pub fn no_case_fold(mut self) -> Self {
        self.case_fold = false;
        self
    }

    /// Add a learned rule at runtime (called when WAL replays).
    pub fn learn_suffix(&mut self, r: SuffixRule) {
        self.suffix_rules.push(r);
        // Re-sort by priority after adding
        self.suffix_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Add a learned prefix rule at runtime.
    pub fn learn_prefix(&mut self, r: PrefixRule) {
        self.prefix_rules.push(r);
        self.prefix_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Seal rules: sort by priority so analyze() tries most-specific first.
    pub fn finalize(mut self) -> Self {
        self.suffix_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.prefix_rules.sort_by(|a, b| {
            // Sort by stacking order first (outer→inner), then priority
            a.family.stacking_order()
                .cmp(&b.family.stacking_order())
                .then(b.priority.cmp(&a.priority))
        });
        self
    }

    // ========================================================================
    // ANALYZE — the single entry point
    // ========================================================================

    pub fn analyze(&self, surface: &str) -> Vec<Analysis> {
        let word = if self.case_fold {
            surface.to_lowercase()
        } else {
            surface.to_string()
        };

        // Short words: identity only
        if word.chars().count() < self.min_word_chars {
            return vec![Analysis::identity(surface)];
        }

        let mut results: Vec<Analysis> = Vec::new();

        // 1) Irregular form lookup — if matched, this is the most confident.
        if let Some(irr) = self.irregulars.get(&word) {
            results.push(Analysis {
                lemma: irr.lemma.to_string(),
                features: irr.features.clone(),
                confidence: 95,
            });
        }

        // 2) Suffix stripping — try each rule
        for rule in &self.suffix_rules {
            if let Some(analysis) = self.try_suffix(&word, rule) {
                results.push(analysis);
            }
        }

        // 3) Prefix stripping — Semitic may stack
        if self.stack_prefixes {
            if let Some(analysis) = self.try_stacked_prefixes(&word) {
                results.push(analysis);
            }
        } else {
            for rule in &self.prefix_rules {
                if let Some(analysis) = self.try_single_prefix(&word, rule) {
                    results.push(analysis);
                }
            }
        }

        // 4) Fallback — identity at lower confidence if nothing matched
        if results.is_empty() {
            results.push(Analysis {
                lemma: surface.to_string(),
                features: Vec::new(),
                confidence: 60,
            });
        } else {
            // Always include identity as a secondary candidate
            results.push(Analysis {
                lemma: surface.to_string(),
                features: Vec::new(),
                confidence: 50,
            });
        }

        results
    }

    fn try_suffix(&self, word: &str, rule: &SuffixRule) -> Option<Analysis> {
        if !word.ends_with(rule.suffix) {
            return None;
        }
        let chars: Vec<char> = word.chars().collect();
        let suffix_chars = rule.suffix.chars().count();
        if chars.len() < suffix_chars + rule.min_stem_chars {
            return None;
        }
        let stem_chars: &[char] = &chars[..chars.len() - suffix_chars];

        // Check stem ending requirement
        if let Some(allowed) = rule.requires_stem_ending {
            if let Some(&last) = stem_chars.last() {
                if !allowed.contains(&last) {
                    return None;
                }
            } else {
                return None;
            }
        }

        let mut stem: String = stem_chars.iter().collect();

        // Handle double-consonant hint (running → run)
        if rule.double_consonant_hint && stem.len() >= 2 {
            let bs: Vec<char> = stem.chars().collect();
            let len = bs.len();
            if bs[len - 1] == bs[len - 2]
                && is_consonant(bs[len - 1])
                && !matches!(bs[len - 1], 'l' | 's' | 'z')
            {
                stem = bs[..len - 1].iter().collect();
            }
        }

        // Append the add_back characters
        if !rule.add_back.is_empty() {
            stem.push_str(rule.add_back);
        }

        Some(Analysis {
            lemma: stem,
            features: rule.features.clone(),
            confidence: rule.priority.min(95),
        })
    }

    fn try_single_prefix(&self, word: &str, rule: &PrefixRule) -> Option<Analysis> {
        if !word.starts_with(rule.prefix) {
            return None;
        }
        let prefix_chars = rule.prefix.chars().count();
        let total_chars = word.chars().count();
        if total_chars < prefix_chars + rule.min_stem_chars {
            return None;
        }
        let stem: String = word.chars().skip(prefix_chars).collect();

        Some(Analysis {
            lemma: stem,
            features: rule.features.clone(),
            confidence: rule.priority.min(90),
        })
    }

    /// Semitic-style: try to strip multiple stacked prefixes in order.
    /// Conjunction → Preposition → DefiniteArticle → stem
    fn try_stacked_prefixes(&self, word: &str) -> Option<Analysis> {
        let mut remaining = word.to_string();
        let mut features: Vec<Feature> = Vec::new();
        let mut any_stripped = false;

        // Try each family in order. Only one rule per family can match.
        for family in [PrefixFamily::Conjunction, PrefixFamily::Preposition, PrefixFamily::DefiniteArticle] {
            for rule in self.prefix_rules.iter().filter(|r| r.family == family) {
                if remaining.starts_with(rule.prefix) {
                    let prefix_chars = rule.prefix.chars().count();
                    let after_chars = remaining.chars().count() - prefix_chars;
                    if after_chars >= rule.min_stem_chars {
                        remaining = remaining.chars().skip(prefix_chars).collect();
                        features.extend(rule.features.clone());
                        any_stripped = true;
                        break;
                    }
                }
            }
        }

        if any_stripped {
            Some(Analysis {
                lemma: remaining,
                features,
                confidence: 85,
            })
        } else {
            None
        }
    }
}

fn is_consonant(c: char) -> bool {
    !matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphology::rules::{PrefixFamily, PrefixRule, SuffixRule, IrregularForm};

    #[test]
    fn core_empty_returns_identity() {
        let m = MorphologyCore::new("xx", Typology::Isolating);
        let a = m.analyze("hello");
        assert_eq!(a[0].lemma, "hello");
    }

    #[test]
    fn core_with_suffix_strips_correctly() {
        let m = MorphologyCore::new("en", Typology::Germanic)
            .with_suffix(SuffixRule {
                suffix: "s",
                features: vec![Feature::Plural],
                add_back: "",
                priority: 75,
                min_stem_chars: 2,
                requires_stem_ending: None,
                double_consonant_hint: false,
            })
            .finalize();
        let a = m.analyze("dogs");
        assert!(a.iter().any(|x| x.lemma == "dog" && x.features.contains(&Feature::Plural)));
    }

    #[test]
    fn core_with_irregular_wins_over_suffix() {
        let m = MorphologyCore::new("en", Typology::Germanic)
            .with_suffix(SuffixRule {
                suffix: "t",
                features: vec![Feature::Past],
                add_back: "",
                priority: 75,
                min_stem_chars: 2,
                requires_stem_ending: None,
                double_consonant_hint: false,
            })
            .with_irregulars(vec![IrregularForm::new("went", "go", vec![Feature::Past])])
            .finalize();
        let a = m.analyze("went");
        // Irregular should be first (highest confidence)
        assert_eq!(a[0].lemma, "go");
        assert_eq!(a[0].confidence, 95);
    }

    #[test]
    fn stacking_prefixes_compose() {
        let m = MorphologyCore::new("he", Typology::Semitic)
            .with_prefix(PrefixRule {
                prefix: "ו",
                features: vec![Feature::Conjunction],
                priority: 80,
                family: PrefixFamily::Conjunction,
                min_stem_chars: 2,
            })
            .with_prefix(PrefixRule {
                prefix: "מ",
                features: vec![Feature::From],
                priority: 70,
                family: PrefixFamily::Preposition,
                min_stem_chars: 2,
            })
            .with_prefix(PrefixRule {
                prefix: "ה",
                features: vec![Feature::DefiniteArticle],
                priority: 90,
                family: PrefixFamily::DefiniteArticle,
                min_stem_chars: 2,
            })
            .stacking(true)
            .no_case_fold()
            .finalize();
        let results = m.analyze("ומהבית");
        // Should find: stem=בית, features=[Conj, From, DefArt]
        let stacked = results.iter().find(|a| a.lemma == "בית");
        assert!(stacked.is_some(), "expected lemma=בית, got {:?}", results);
        let stacked = stacked.unwrap();
        assert!(stacked.features.contains(&Feature::Conjunction));
        assert!(stacked.features.contains(&Feature::From));
        assert!(stacked.features.contains(&Feature::DefiniteArticle));
    }

    #[test]
    fn runtime_learning_adds_rule() {
        let mut m = MorphologyCore::new("en", Typology::Germanic);
        m.learn_suffix(SuffixRule {
            suffix: "xyz",
            features: vec![Feature::Plural],
            add_back: "",
            priority: 75,
            min_stem_chars: 2,
            requires_stem_ending: None,
            double_consonant_hint: false,
        });
        let a = m.analyze("catxyz");
        assert!(a.iter().any(|x| x.lemma == "cat" && x.features.contains(&Feature::Plural)));
    }

    #[test]
    fn double_consonant_hint_works() {
        let m = MorphologyCore::new("en", Typology::Germanic)
            .with_suffix(SuffixRule {
                suffix: "ing",
                features: vec![Feature::Continuous],
                add_back: "",
                priority: 85,
                min_stem_chars: 3,
                requires_stem_ending: None,
                double_consonant_hint: true,
            })
            .finalize();
        let a = m.analyze("running");
        assert!(a.iter().any(|x| x.lemma == "run" && x.features.contains(&Feature::Continuous)));
    }

    #[test]
    fn short_word_unchanged() {
        let m = MorphologyCore::new("he", Typology::Semitic);
        let a = m.analyze("ה");
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].lemma, "ה");
    }
}
