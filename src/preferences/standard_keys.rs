//! # StandardKey — well-known preference keys
//!
//! Rather than using free-form strings everywhere, callers should use
//! these canonical keys. Free-form strings still work (for custom prefs)
//! but standard keys guarantee consistency across the system.

use crate::preferences::key::PreferenceKey;

/// Well-known preference keys with stable string representations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardKey {
    /// Desired response tone. Values: "formal" | "casual" | "direct" | "warm"
    Tone,
    /// Desired response length. Values: "short" | "medium" | "long"
    Length,
    /// Desired response format. Values: "bullet_list" | "prose" | "code" | "table"
    Format,
    /// Preferred language. Values: "he" | "en" | "auto"
    Language,
    /// Level of detail. Values: "brief" | "standard" | "detailed" | "exhaustive"
    DetailLevel,
    /// How ZETS should handle responses. Values: "question_back" | "confirm" | "suggest" | "command"
    ResponseStyle,
    /// Humor level. Values: "none" | "subtle" | "frequent"
    HumorLevel,
    /// Whether hedging language is acceptable. Values: "true" | "false"
    HedgingAllowed,
    /// Whether ZETS may use emoji. Values: "true" | "false"
    UseEmoji,
    /// Default programming language for code examples.
    CodeLanguage,
    /// How to address the owner. Values: "first" | "full" | "nickname"
    NameForm,
}

impl StandardKey {
    /// The canonical string key for this preference.
    pub fn key_str(&self) -> &'static str {
        match self {
            StandardKey::Tone => "tone",
            StandardKey::Length => "length",
            StandardKey::Format => "format",
            StandardKey::Language => "language",
            StandardKey::DetailLevel => "detail_level",
            StandardKey::ResponseStyle => "response_style",
            StandardKey::HumorLevel => "humor_level",
            StandardKey::HedgingAllowed => "hedging_allowed",
            StandardKey::UseEmoji => "use_emoji",
            StandardKey::CodeLanguage => "code.language",
            StandardKey::NameForm => "name_form",
        }
    }

    /// Convert to a PreferenceKey.
    pub fn as_key(&self) -> PreferenceKey {
        PreferenceKey::new(self.key_str())
    }

    /// System default value, if any.
    pub fn default_value(&self) -> Option<&'static str> {
        match self {
            StandardKey::Tone => Some("direct"),
            StandardKey::Length => Some("medium"),
            StandardKey::Format => Some("prose"),
            StandardKey::Language => Some("auto"),
            StandardKey::DetailLevel => Some("standard"),
            StandardKey::ResponseStyle => Some("suggest"),
            StandardKey::HumorLevel => Some("none"),
            StandardKey::HedgingAllowed => Some("true"),
            StandardKey::UseEmoji => Some("false"),
            StandardKey::CodeLanguage => None,
            StandardKey::NameForm => Some("first"),
        }
    }

    /// Parse a key string back to a StandardKey, if it's one of the known ones.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "tone" => Some(StandardKey::Tone),
            "length" => Some(StandardKey::Length),
            "format" => Some(StandardKey::Format),
            "language" => Some(StandardKey::Language),
            "detail_level" => Some(StandardKey::DetailLevel),
            "response_style" => Some(StandardKey::ResponseStyle),
            "humor_level" => Some(StandardKey::HumorLevel),
            "hedging_allowed" => Some(StandardKey::HedgingAllowed),
            "use_emoji" => Some(StandardKey::UseEmoji),
            "code.language" => Some(StandardKey::CodeLanguage),
            "name_form" => Some(StandardKey::NameForm),
            _ => None,
        }
    }

    /// All standard keys.
    pub fn all() -> &'static [StandardKey] {
        &[
            StandardKey::Tone,
            StandardKey::Length,
            StandardKey::Format,
            StandardKey::Language,
            StandardKey::DetailLevel,
            StandardKey::ResponseStyle,
            StandardKey::HumorLevel,
            StandardKey::HedgingAllowed,
            StandardKey::UseEmoji,
            StandardKey::CodeLanguage,
            StandardKey::NameForm,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_keys_have_defaults() {
        // Most standard keys should have a default
        let keys_with_defaults = StandardKey::all()
            .iter()
            .filter(|k| k.default_value().is_some())
            .count();
        // At minimum Tone, Length, Format, Language, DetailLevel, ResponseStyle,
        // HumorLevel, HedgingAllowed, UseEmoji, NameForm — 10 out of 11
        assert!(keys_with_defaults >= 10);
    }

    #[test]
    fn test_standard_key_round_trip() {
        for key in StandardKey::all() {
            let s = key.key_str();
            let parsed = StandardKey::from_str(s);
            assert_eq!(parsed, Some(*key), "Round-trip failed for {:?}", key);
        }
    }

    #[test]
    fn test_as_key_creates_valid_preference_key() {
        for key in StandardKey::all() {
            let pk = key.as_key();
            assert!(pk.is_valid(), "Invalid key for {:?}: {:?}", key, pk);
        }
    }

    #[test]
    fn test_language_default_is_auto() {
        assert_eq!(StandardKey::Language.default_value(), Some("auto"));
    }

    #[test]
    fn test_length_default_is_medium() {
        assert_eq!(StandardKey::Length.default_value(), Some("medium"));
    }

    #[test]
    fn test_code_language_has_no_default() {
        // CodeLanguage is personal — no universal default
        assert_eq!(StandardKey::CodeLanguage.default_value(), None);
    }
}
