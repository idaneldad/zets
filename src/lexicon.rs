//! Multilingual lexicon — loaded at runtime from TSV packs.
//! Each entry keyed by (lang, surface) so "Gift"[EN] != "Gift"[DE].

use std::collections::HashMap;
use std::fs;
use std::path::Path;

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

pub struct Lexicon {
    entries: HashMap<LexKey, LexEntry>,
    languages: Vec<String>,
}

impl Lexicon {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            languages: Vec::new(),
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
            self.languages.push(lang.clone());
            self.load_language(&lang, &entry.path(), &mut stats)?;
        }
        Ok(stats)
    }

    fn load_language(
        &mut self,
        lang: &str,
        dir: &Path,
        stats: &mut LoadStats,
    ) -> std::io::Result<()> {
        let def_path = dir.join("definitions.tsv");
        if def_path.exists() {
            for line in fs::read_to_string(&def_path)?.lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let mut parts = line.splitn(2, '\t');
                if let (Some(word), Some(def)) = (parts.next(), parts.next()) {
                    self.entries
                        .entry(LexKey::new(lang, word))
                        .or_default()
                        .definitions
                        .push(def.to_string());
                    stats.definitions += 1;
                }
            }
        }
        let syn_path = dir.join("synonyms.tsv");
        if syn_path.exists() {
            for line in fs::read_to_string(&syn_path)?.lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let mut parts = line.splitn(2, '\t');
                if let (Some(word), Some(syn)) = (parts.next(), parts.next()) {
                    self.entries
                        .entry(LexKey::new(lang, word))
                        .or_default()
                        .synonyms
                        .push(syn.to_string());
                    stats.synonyms += 1;
                }
            }
        }
        let ant_path = dir.join("antonyms.tsv");
        if ant_path.exists() {
            for line in fs::read_to_string(&ant_path)?.lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let mut parts = line.splitn(2, '\t');
                if let (Some(word), Some(ant)) = (parts.next(), parts.next()) {
                    self.entries
                        .entry(LexKey::new(lang, word))
                        .or_default()
                        .antonyms
                        .push(ant.to_string());
                    stats.antonyms += 1;
                }
            }
        }
        let pos_path = dir.join("pos.tsv");
        if pos_path.exists() {
            for line in fs::read_to_string(&pos_path)?.lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let mut parts = line.splitn(2, '\t');
                if let (Some(word), Some(pos)) = (parts.next(), parts.next()) {
                    let entry = self.entries.entry(LexKey::new(lang, word)).or_default();
                    if entry.pos.is_none() {
                        entry.pos = Some(pos.to_string());
                        stats.pos += 1;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get(&self, lang: &str, surface: &str) -> Option<&LexEntry> {
        self.entries.get(&LexKey::new(lang, surface))
    }

    pub fn find_all_languages(&self, surface: &str) -> Vec<(String, &LexEntry)> {
        let mut result = Vec::new();
        for (key, entry) in &self.entries {
            if key.surface == surface {
                result.push((key.lang.clone(), entry));
            }
        }
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn languages(&self) -> &[String] {
        &self.languages
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
