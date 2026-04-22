//! Registry — the single entry point for callers.
//!
//! `MorphologyRegistry::new()` builds all supported language cores once.
//! `registry.analyze("he", "בבית")` → returns Vec<Analysis>.
//! Zero dyn dispatch — each language is just a stored MorphologyCore.

use std::collections::HashMap;

use super::core::{Analysis, MorphologyCore};
use super::languages::{arabic, english, hebrew, spanish, vietnamese};

pub struct MorphologyRegistry {
    cores: HashMap<&'static str, MorphologyCore>,
}

impl MorphologyRegistry {
    pub fn new() -> Self {
        let mut cores = HashMap::new();
        cores.insert("he", hebrew());
        cores.insert("ar", arabic());
        cores.insert("en", english());
        cores.insert("es", spanish());
        cores.insert("vi", vietnamese());
        Self { cores }
    }

    /// Analyze a surface word — returns candidates sorted by confidence.
    pub fn analyze(&self, lang: &str, surface: &str) -> Vec<Analysis> {
        if let Some(core) = self.cores.get(lang) {
            core.analyze(surface)
        } else {
            // Unknown language — return identity
            vec![Analysis::identity(surface)]
        }
    }

    /// Direct access (for learning: engine needs to mutate the core).
    pub fn core_mut(&mut self, lang: &str) -> Option<&mut MorphologyCore> {
        self.cores.get_mut(lang)
    }

    pub fn core(&self, lang: &str) -> Option<&MorphologyCore> {
        self.cores.get(lang)
    }

    pub fn has(&self, lang: &str) -> bool {
        self.cores.contains_key(lang)
    }

    pub fn supported_langs(&self) -> Vec<&'static str> {
        let mut v: Vec<_> = self.cores.keys().copied().collect();
        v.sort();
        v
    }

    /// Short typology summary.
    pub fn typology_summary(&self) -> String {
        let mut out = String::new();
        for lang in self.supported_langs() {
            let core = self.cores.get(lang).unwrap();
            out.push_str(&format!(
                "{:<4} {:<14} prefixes={} suffixes={} irregulars={}\n",
                lang,
                format!("{:?}", core.typology),
                core.prefix_rules.len(),
                core.suffix_rules.len(),
                core.irregulars.len(),
            ));
        }
        out
    }
}

impl Default for MorphologyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_all_5_langs() {
        let r = MorphologyRegistry::new();
        for lang in ["he", "ar", "en", "es", "vi"] {
            assert!(r.has(lang), "missing {}", lang);
        }
    }

    #[test]
    fn unknown_lang_returns_identity() {
        let r = MorphologyRegistry::new();
        let a = r.analyze("zz", "hello");
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].lemma, "hello");
    }

    #[test]
    fn registry_hebrew_works() {
        let r = MorphologyRegistry::new();
        let a = r.analyze("he", "בבית");
        assert!(a.iter().any(|x| x.lemma == "בית"));
    }

    #[test]
    fn registry_english_works() {
        let r = MorphologyRegistry::new();
        let a = r.analyze("en", "cats");
        assert!(a.iter().any(|x| x.lemma == "cat"));
    }
}
