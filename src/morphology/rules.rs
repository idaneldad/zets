//! Rule types — the data that drives morphological analysis.
//!
//! All rules are plain data (Clone-able, can be serialized to WAL).
//! The engine iterates over these — no code generation, no compilation.

use super::core::Feature;

/// A prefix-strip rule.
/// When a word starts with `prefix`, remove it and add `features` to the analysis.
/// Higher `priority` = tried first.
#[derive(Debug, Clone)]
pub struct PrefixRule {
    pub prefix: &'static str,
    pub features: Vec<Feature>,
    pub priority: u8,        // 100 = definite article, 80 = conjunction, 60 = preposition
    pub family: PrefixFamily, // for stacking order logic
    /// Minimum stem length AFTER stripping (to avoid stripping real word starts).
    pub min_stem_chars: usize,
}

/// Family classification — used for stacking order (Hebrew ו-מ-ה must come in this order).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixFamily {
    /// "and" - must be outermost (ו, و)
    Conjunction,
    /// Inseparable prepositions - between conjunction and article (ב/ל/מ/כ/ש, ب/ل)
    Preposition,
    /// Definite article - innermost before stem (ה, ال)
    DefiniteArticle,
    /// Any other
    Other,
}

impl PrefixFamily {
    /// Return the stacking order: lower number = outermost.
    pub fn stacking_order(self) -> u8 {
        match self {
            Self::Conjunction => 0,
            Self::Preposition => 1,
            Self::DefiniteArticle => 2,
            Self::Other => 3,
        }
    }
}

/// A suffix-strip rule.
/// When a word ends with `suffix`, remove it and emit the lemma with `features`.
/// `add_back` appends characters to the stem after strip (e.g. cities → city: strip "ies", add "y").
#[derive(Debug, Clone)]
pub struct SuffixRule {
    pub suffix: &'static str,
    pub features: Vec<Feature>,
    pub add_back: &'static str, // characters to append after stripping
    pub priority: u8,
    /// Minimum stem length AFTER stripping.
    pub min_stem_chars: usize,
    /// If set, only apply if the stripped stem's LAST character matches one of these.
    pub requires_stem_ending: Option<&'static [char]>,
    /// If true, the engine may double the last consonant before stripping (running→run).
    pub double_consonant_hint: bool,
}

impl SuffixRule {
    /// Simple constructor for common case (no add_back, no special conditions).
    pub const fn simple(suffix: &'static str, features: Vec<Feature>, priority: u8) -> Self {
        Self {
            suffix,
            features,
            add_back: "",
            priority,
            min_stem_chars: 1,
            requires_stem_ending: None,
            double_consonant_hint: false,
        }
    }
}

/// An irregular form — direct lookup.
/// "went" → ("go", [Past]), no rule-based stripping.
#[derive(Debug, Clone)]
pub struct IrregularForm {
    pub surface: &'static str,
    pub lemma: &'static str,
    pub features: Vec<Feature>,
}

impl IrregularForm {
    pub fn new(surface: &'static str, lemma: &'static str, features: Vec<Feature>) -> Self {
        Self { surface, lemma, features }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_stacking_order_is_correct() {
        assert!(PrefixFamily::Conjunction.stacking_order()
              < PrefixFamily::Preposition.stacking_order());
        assert!(PrefixFamily::Preposition.stacking_order()
              < PrefixFamily::DefiniteArticle.stacking_order());
    }
}
