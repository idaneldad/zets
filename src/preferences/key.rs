//! # PreferenceKey — hierarchical, parseable preference keys
//!
//! Keys follow a dot-separated hierarchy:
//!   `tone`, `format.bullet_list`, `code.language`
//!
//! This allows prefix-based lookup ("all format preferences") and
//! is interoperable with the StandardKey enum.

use std::fmt;

/// A preference key — normalized, dot-separated, lowercase.
///
/// Examples: `"tone"`, `"format"`, `"length"`, `"language"`,
///           `"code.language"`, `"response.style"`
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PreferenceKey(pub String);

impl PreferenceKey {
    /// Create from any string — normalizes to lowercase.
    pub fn new(key: impl Into<String>) -> Self {
        PreferenceKey(key.into().to_lowercase())
    }

    /// Access the raw string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Top-level prefix (first segment before dot).
    pub fn prefix(&self) -> &str {
        self.0.split('.').next().unwrap_or(&self.0)
    }

    /// Remaining path after the first segment, if any.
    pub fn suffix(&self) -> Option<&str> {
        let dot = self.0.find('.')?;
        Some(&self.0[dot + 1..])
    }

    /// Whether this key starts with the given prefix.
    pub fn starts_with_prefix(&self, prefix: &str) -> bool {
        self.0 == prefix || self.0.starts_with(&format!("{}.", prefix))
    }

    /// Check that the key is valid (non-empty, only lowercase alphanumeric + dots + underscores).
    pub fn is_valid(&self) -> bool {
        !self.0.is_empty()
            && self
                .0
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_')
    }
}

impl fmt::Display for PreferenceKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for PreferenceKey {
    fn from(s: &str) -> Self {
        PreferenceKey::new(s)
    }
}

impl From<String> for PreferenceKey {
    fn from(s: String) -> Self {
        PreferenceKey::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_normalized_to_lowercase() {
        let k = PreferenceKey::new("Tone");
        assert_eq!(k.as_str(), "tone");
    }

    #[test]
    fn test_key_prefix_and_suffix() {
        let k = PreferenceKey::new("code.language");
        assert_eq!(k.prefix(), "code");
        assert_eq!(k.suffix(), Some("language"));
    }

    #[test]
    fn test_key_no_suffix() {
        let k = PreferenceKey::new("tone");
        assert_eq!(k.prefix(), "tone");
        assert_eq!(k.suffix(), None);
    }

    #[test]
    fn test_key_starts_with_prefix() {
        let k = PreferenceKey::new("format.bullet_list");
        assert!(k.starts_with_prefix("format"));
        assert!(!k.starts_with_prefix("tone"));
    }

    #[test]
    fn test_key_validity() {
        assert!(PreferenceKey::new("tone").is_valid());
        assert!(PreferenceKey::new("code.language").is_valid());
        assert!(PreferenceKey::new("response_style").is_valid());
        assert!(!PreferenceKey::new("").is_valid());
        assert!(!PreferenceKey::new("Bad Key!").is_valid());
    }

    #[test]
    fn test_key_display() {
        let k = PreferenceKey::new("language");
        assert_eq!(format!("{}", k), "language");
    }

    #[test]
    fn test_key_from_str() {
        let k: PreferenceKey = "length".into();
        assert_eq!(k.as_str(), "length");
    }
}
