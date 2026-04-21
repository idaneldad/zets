//! Multilingual lexicon — concept-centric, memory-efficient.
//!
//! Final architecture:
//!   - Languages encoded as u8 (16 fixed slots, no String allocation per entry)
//!   - Surfaces interned in StringPool (HashMap key holds u32 id, not String)
//!   - Glosses interned (huge dedup — same definition shared across languages)
//!   - Compact entries: (u8 lang, u32 surface_id) → 4-byte u32 ids only

use std::collections::HashMap;
use std::fs;
use std::path::Path;

const POS_VALUES: &[&str] = &[
    "", "noun", "verb", "adj", "adv", "interj", "prep", "conj",
    "phrase", "det", "pron", "num", "particle", "art", "aux", "other",
];

fn pos_to_id(pos: &str) -> u8 {
    POS_VALUES.iter().position(|p| *p == pos).unwrap_or(15) as u8
}

fn pos_from_id(id: u8) -> &'static str {
    POS_VALUES.get(id as usize).copied().unwrap_or("")
}

/// Language codes registered as `u8` to avoid String overhead per entry.
struct LangRegistry {
    codes: Vec<String>,         // index → "en", "he", ...
    by_name: HashMap<String, u8>,
}

impl LangRegistry {
    fn new() -> Self {
        Self {
            codes: Vec::new(),
            by_name: HashMap::new(),
        }
    }
    fn intern(&mut self, code: &str) -> u8 {
        if let Some(&id) = self.by_name.get(code) {
            return id;
        }
        let id = self.codes.len() as u8;
        self.codes.push(code.to_string());
        self.by_name.insert(code.to_string(), id);
        id
    }
    fn get(&self, code: &str) -> Option<u8> {
        self.by_name.get(code).copied()
    }
    fn name(&self, id: u8) -> &str {
        self.codes.get(id as usize).map(|s| s.as_str()).unwrap_or("")
    }
}

struct StringPool {
    strings: Vec<String>,
    interner: HashMap<String, u32>,
}

impl StringPool {
    fn new() -> Self {
        Self {
            strings: Vec::new(),
            interner: HashMap::new(),
        }
    }
    fn intern(&mut self, s: &str) -> u32 {
        if let Some(&id) = self.interner.get(s) {
            return id;
        }
        let id = self.strings.len() as u32;
        self.strings.push(s.to_string());
        self.interner.insert(s.to_string(), id);
        id
    }
    fn lookup_id(&self, s: &str) -> Option<u32> {
        self.interner.get(s).copied()
    }
    fn get(&self, id: u32) -> &str {
        self.strings.get(id as usize).map(|s| s.as_str()).unwrap_or("")
    }
    fn len(&self) -> usize {
        self.strings.len()
    }
}

/// Public API kept identical for backward compat with compose/ask/phrase.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LexKey {
    pub lang: String,
    pub surface: String,
}

impl LexKey {
    pub fn new(lang: &str, surface: &str) -> Self {
        Self {
            lang: lang.to_string(),
            surface: surface.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LexEntry {
    pub definitions: Vec<String>,
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
    pub pos: Option<String>,
}

impl LexEntry {
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
            && self.synonyms.is_empty()
            && self.antonyms.is_empty()
            && self.pos.is_none()
    }
}

/// Internal compact entry: ~24 bytes payload (no String allocations)
#[derive(Debug, Clone, Default)]
struct CompactEntry {
    definition_ids: Vec<u32>,
    synonym_ids: Vec<u32>,
    antonym_ids: Vec<u32>,
    pos: u8,
}

/// Compact key: 8 bytes total (u8 + u32 + 3 padding)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LangSurfaceKey {
    lang_id: u8,
    surface_id: u32,
}

pub struct Lexicon {
    langs: LangRegistry,
    gloss_pool: StringPool,
    surface_pool: StringPool,
    entries: HashMap<LangSurfaceKey, CompactEntry>,
}

impl Lexicon {
    pub fn new() -> Self {
        Self {
            langs: LangRegistry::new(),
            gloss_pool: StringPool::new(),
            surface_pool: StringPool::new(),
            entries: HashMap::new(),
        }
    }

    pub fn load_from_dir<P: AsRef<Path>>(&mut self, base: P) -> std::io::Result<LoadStats> {
        let mut stats = LoadStats::default();
        let base = base.as_ref();
        for entry in fs::read_dir(base)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let lang = entry.file_name().to_string_lossy().to_string();
            if lang.len() > 6 {
                continue;
            }
            let lang_id = self.langs.intern(&lang);
            self.load_language(lang_id, &entry.path(), &mut stats)?;
        }
        // Free the interner maps after loading — saves ~30% RAM
        self.gloss_pool.interner.clear();
        self.gloss_pool.interner.shrink_to_fit();
        self.surface_pool.interner.clear();
        self.surface_pool.interner.shrink_to_fit();
        self.entries.shrink_to_fit();
        Ok(stats)
    }

    fn load_language(
        &mut self,
        lang_id: u8,
        dir: &Path,
        stats: &mut LoadStats,
    ) -> std::io::Result<()> {
        let def_path = dir.join("definitions.tsv");
        if def_path.exists() {
            let content = fs::read_to_string(&def_path)?;
            for line in content.lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let mut parts = line.splitn(2, '\t');
                if let (Some(word), Some(def)) = (parts.next(), parts.next()) {
                    let surface_id = self.surface_pool.intern(word);
                    let gloss_id = self.gloss_pool.intern(def);
                    self.entries
                        .entry(LangSurfaceKey { lang_id, surface_id })
                        .or_default()
                        .definition_ids
                        .push(gloss_id);
                    stats.definitions += 1;
                }
            }
        }

        let syn_path = dir.join("synonyms.tsv");
        if syn_path.exists() {
            let content = fs::read_to_string(&syn_path)?;
            for line in content.lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let mut parts = line.splitn(2, '\t');
                if let (Some(word), Some(syn)) = (parts.next(), parts.next()) {
                    let surface_id = self.surface_pool.intern(word);
                    let s_id = self.surface_pool.intern(syn);
                    self.entries
                        .entry(LangSurfaceKey { lang_id, surface_id })
                        .or_default()
                        .synonym_ids
                        .push(s_id);
                    stats.synonyms += 1;
                }
            }
        }

        let ant_path = dir.join("antonyms.tsv");
        if ant_path.exists() {
            let content = fs::read_to_string(&ant_path)?;
            for line in content.lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let mut parts = line.splitn(2, '\t');
                if let (Some(word), Some(ant)) = (parts.next(), parts.next()) {
                    let surface_id = self.surface_pool.intern(word);
                    let a_id = self.surface_pool.intern(ant);
                    self.entries
                        .entry(LangSurfaceKey { lang_id, surface_id })
                        .or_default()
                        .antonym_ids
                        .push(a_id);
                    stats.antonyms += 1;
                }
            }
        }

        let pos_path = dir.join("pos.tsv");
        if pos_path.exists() {
            let content = fs::read_to_string(&pos_path)?;
            for line in content.lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let mut parts = line.splitn(2, '\t');
                if let (Some(word), Some(pos)) = (parts.next(), parts.next()) {
                    let surface_id = self.surface_pool.intern(word);
                    let entry = self
                        .entries
                        .entry(LangSurfaceKey { lang_id, surface_id })
                        .or_default();
                    if entry.pos == 0 {
                        entry.pos = pos_to_id(pos);
                        stats.pos += 1;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get(&self, lang: &str, surface: &str) -> Option<LexEntry> {
        let lang_id = self.langs.get(lang)?;
        let surface_id = self.surface_pool.lookup_id(surface)?;
        let compact = self.entries.get(&LangSurfaceKey { lang_id, surface_id })?;
        Some(self.materialize(compact))
    }

    fn materialize(&self, compact: &CompactEntry) -> LexEntry {
        LexEntry {
            definitions: compact
                .definition_ids
                .iter()
                .map(|id| self.gloss_pool.get(*id).to_string())
                .collect(),
            synonyms: compact
                .synonym_ids
                .iter()
                .map(|id| self.surface_pool.get(*id).to_string())
                .collect(),
            antonyms: compact
                .antonym_ids
                .iter()
                .map(|id| self.surface_pool.get(*id).to_string())
                .collect(),
            pos: if compact.pos == 0 {
                None
            } else {
                Some(pos_from_id(compact.pos).to_string())
            },
        }
    }

    pub fn find_all_languages(&self, surface: &str) -> Vec<(String, LexEntry)> {
        let surface_id = match self.surface_pool.lookup_id(surface) {
            Some(id) => id,
            None => return Vec::new(),
        };
        let mut result = Vec::new();
        for (key, compact) in &self.entries {
            if key.surface_id == surface_id {
                result.push((self.langs.name(key.lang_id).to_string(), self.materialize(compact)));
            }
        }
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn languages(&self) -> Vec<String> {
        self.langs.codes.clone()
    }

    pub fn pool_stats(&self) -> (usize, usize, usize) {
        (
            self.gloss_pool.len(),
            self.surface_pool.len(),
            self.entries.len(),
        )
    }
}

impl Default for Lexicon {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Clone)]
pub struct LoadStats {
    pub definitions: usize,
    pub synonyms: usize,
    pub antonyms: usize,
    pub pos: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_dedups_identical_strings() {
        let mut p = StringPool::new();
        let a = p.intern("hello");
        let b = p.intern("hello");
        let c = p.intern("world");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn pos_encoding_roundtrip() {
        for &p in POS_VALUES {
            let id = pos_to_id(p);
            assert_eq!(pos_from_id(id), p);
        }
    }

    #[test]
    fn lang_registry_works() {
        let mut r = LangRegistry::new();
        let en = r.intern("en");
        let he = r.intern("he");
        let en2 = r.intern("en");
        assert_eq!(en, en2);
        assert_ne!(en, he);
        assert_eq!(r.name(en), "en");
    }

    #[test]
    fn round_trip_entry() {
        let mut l = Lexicon::new();
        let lang_id = l.langs.intern("en");
        let surface_id = l.surface_pool.intern("dog");
        let gloss_id = l.gloss_pool.intern("a four-legged animal");
        l.entries
            .entry(LangSurfaceKey { lang_id, surface_id })
            .or_default()
            .definition_ids
            .push(gloss_id);
        let entry = l.get("en", "dog").expect("entry");
        assert_eq!(entry.definitions, vec!["a four-legged animal".to_string()]);
    }

    #[test]
    fn unknown_lookup_returns_none() {
        let l = Lexicon::new();
        assert!(l.get("xx", "yy").is_none());
    }
}
