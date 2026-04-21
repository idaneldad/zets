//! Per-language syntactic composition rules.
//!
//! Encodes WHICH order to place adjective vs noun, etc.
//! Same concept pair → different surface strings, by language rule.
//!
//! Example:
//!   Concept(BIG) + Concept(HOUSE)
//!     en: "big house"     (AdjNoun)
//!     he: "בית גדול"       (NounAdj)
//!     de: "großes Haus"   (AdjNoun)
//!     fr: "grande maison" (AdjNoun — dominant for size adjectives)
//!     it: "casa grande"   (NounAdj — dominant)

/// Order of adjective relative to the noun it modifies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjOrder {
    /// Adjective BEFORE noun: "big house"
    AdjNoun,
    /// Adjective AFTER noun: "בית גדול", "casa grande"
    NounAdj,
}

/// Grammatical gender marker (for morphological agreement).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
    /// Not marked in this language (e.g. English)
    Unmarked,
}

/// Syntactic profile per language.
#[derive(Debug, Clone)]
pub struct LangSyntax {
    pub code: &'static str,
    pub name: &'static str,
    pub adj_order: AdjOrder,
    /// Does this language mark gender on adjectives? (agreement required)
    pub has_gender_agreement: bool,
    /// Reading direction — affects display but not composition
    pub rtl: bool,
    /// Separator between composed tokens. Usually " " but CJK uses "".
    pub separator: &'static str,
}

/// The canonical syntax table for all supported languages.
/// Extensively documented with grammar citations in mind.
pub const LANG_SYNTAX: &[LangSyntax] = &[
    // Germanic: adjective before noun
    LangSyntax {
        code: "en",
        name: "English",
        adj_order: AdjOrder::AdjNoun,
        has_gender_agreement: false,
        rtl: false,
        separator: " ",
    },
    LangSyntax {
        code: "de",
        name: "German",
        adj_order: AdjOrder::AdjNoun,
        has_gender_agreement: true,
        rtl: false,
        separator: " ",
    },
    LangSyntax {
        code: "nl",
        name: "Dutch",
        adj_order: AdjOrder::AdjNoun,
        has_gender_agreement: true,
        rtl: false,
        separator: " ",
    },
    // Slavic: adjective before noun
    LangSyntax {
        code: "ru",
        name: "Russian",
        adj_order: AdjOrder::AdjNoun,
        has_gender_agreement: true,
        rtl: false,
        separator: " ",
    },
    // Semitic: noun before adjective
    LangSyntax {
        code: "he",
        name: "Hebrew",
        adj_order: AdjOrder::NounAdj,
        has_gender_agreement: true,
        rtl: true,
        separator: " ",
    },
    LangSyntax {
        code: "ar",
        name: "Arabic",
        adj_order: AdjOrder::NounAdj,
        has_gender_agreement: true,
        rtl: true,
        separator: " ",
    },
    // Romance: noun before adjective (dominant)
    // Note: in all Romance languages, MANY adjectives (size, beauty, age) can
    // appear before the noun with meaning change. We use the dominant pattern.
    LangSyntax {
        code: "fr",
        name: "French",
        adj_order: AdjOrder::NounAdj,
        has_gender_agreement: true,
        rtl: false,
        separator: " ",
    },
    LangSyntax {
        code: "es",
        name: "Spanish",
        adj_order: AdjOrder::NounAdj,
        has_gender_agreement: true,
        rtl: false,
        separator: " ",
    },
    LangSyntax {
        code: "it",
        name: "Italian",
        adj_order: AdjOrder::NounAdj,
        has_gender_agreement: true,
        rtl: false,
        separator: " ",
    },
    LangSyntax {
        code: "pt",
        name: "Portuguese",
        adj_order: AdjOrder::NounAdj,
        has_gender_agreement: true,
        rtl: false,
        separator: " ",
    },
];

/// Look up the syntactic profile for a language.
/// Returns default English-like syntax if not found.
pub fn syntax_for(lang_code: &str) -> &'static LangSyntax {
    LANG_SYNTAX
        .iter()
        .find(|s| s.code == lang_code)
        .unwrap_or(&LANG_SYNTAX[0])
}

/// Compose a noun + adjective pair in a specific language, applying word order.
///
/// # Examples
/// - `compose_noun_adj("en", "house", "big")` → `"big house"`
/// - `compose_noun_adj("he", "בית", "גדול")` → `"בית גדול"`
/// - `compose_noun_adj("es", "casa", "grande")` → `"casa grande"`
pub fn compose_noun_adj(lang: &str, noun: &str, adj: &str) -> String {
    let syntax = syntax_for(lang);
    match syntax.adj_order {
        AdjOrder::AdjNoun => format!("{adj}{sep}{noun}", sep = syntax.separator),
        AdjOrder::NounAdj => format!("{noun}{sep}{adj}", sep = syntax.separator),
    }
}

/// Parse a two-word NP surface and return (noun, adj) pair based on language rule.
/// Returns None if the input does not have exactly two tokens.
pub fn split_noun_adj<'a>(lang: &str, phrase: &'a str) -> Option<(&'a str, &'a str)> {
    let syntax = syntax_for(lang);
    let parts: Vec<&str> = phrase.split(syntax.separator).collect();
    if parts.len() != 2 {
        return None;
    }
    Some(match syntax.adj_order {
        AdjOrder::AdjNoun => (parts[1], parts[0]), // noun = [1], adj = [0]
        AdjOrder::NounAdj => (parts[0], parts[1]), // noun = [0], adj = [1]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_order_english() {
        assert_eq!(compose_noun_adj("en", "house", "big"), "big house");
        assert_eq!(compose_noun_adj("en", "dog", "small"), "small dog");
    }

    #[test]
    fn word_order_hebrew() {
        assert_eq!(compose_noun_adj("he", "בית", "גדול"), "בית גדול");
        assert_eq!(compose_noun_adj("he", "כלב", "קטן"), "כלב קטן");
    }

    #[test]
    fn word_order_italian_and_spanish() {
        assert_eq!(compose_noun_adj("it", "casa", "grande"), "casa grande");
        assert_eq!(compose_noun_adj("es", "casa", "grande"), "casa grande");
    }

    #[test]
    fn word_order_german() {
        assert_eq!(compose_noun_adj("de", "Haus", "groß"), "groß Haus");
        // (ignoring morphology like "großes" — that's a separate concern)
    }

    #[test]
    fn all_ten_languages_covered() {
        for lang in &["en", "de", "fr", "es", "it", "he", "ar", "ru", "nl", "pt"] {
            let s = syntax_for(lang);
            assert_eq!(s.code, *lang);
        }
    }

    #[test]
    fn parse_english_np() {
        let result = split_noun_adj("en", "big house");
        assert_eq!(result, Some(("house", "big")));
    }

    #[test]
    fn parse_hebrew_np() {
        let result = split_noun_adj("he", "בית גדול");
        assert_eq!(result, Some(("בית", "גדול")));
    }

    #[test]
    fn parse_italian_np() {
        let result = split_noun_adj("it", "casa grande");
        assert_eq!(result, Some(("casa", "grande")));
    }

    #[test]
    fn rtl_detection() {
        assert!(syntax_for("he").rtl);
        assert!(syntax_for("ar").rtl);
        assert!(!syntax_for("en").rtl);
        assert!(!syntax_for("de").rtl);
    }
}
