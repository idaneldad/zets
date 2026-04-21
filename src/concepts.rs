//! Cross-language concept layer.
//!
//! A `Concept` is a language-neutral meaning identifier.
//! Surface forms in different languages point to the same Concept via
//! `ConceptSurface` bridge edges.
//!
//! Example: Concept(c42) = "dog" → surfaces in he, en, de, fr, es, it.
//! Query "כלב" or "dog" or "Hund" all resolve to the same Concept.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Language-neutral meaning identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ConceptId(pub u32);

impl ConceptId {
    pub fn from_str(s: &str) -> Option<Self> {
        // "c42" → ConceptId(42)
        s.strip_prefix('c')
            .and_then(|n| n.parse::<u32>().ok())
            .map(ConceptId)
    }
}

/// A concept groups related surface forms across languages.
#[derive(Debug, Clone)]
pub struct Concept {
    pub id: ConceptId,
    /// English "anchor" word — for debugging, not semantically canonical.
    pub english_anchor: String,
    /// Short gloss describing this sense (e.g. "domesticated animal").
    pub gloss: String,
    /// Inferred part-of-speech: "noun", "verb", "adj", "adv", or "" if unknown.
    pub pos: String,
}

/// Bridge edge: a surface form in a specific language points to a concept.
#[derive(Debug, Clone)]
pub struct SurfaceLink {
    pub lang: String,
    pub surface: String,
    pub concept: ConceptId,
}

/// The concept store. Allows bidirectional lookup:
/// - `find_by_surface("dog", "en")` → list of ConceptIds
/// - `surfaces_of(ConceptId, "he")` → list of Hebrew surfaces for that concept
pub struct ConceptStore {
    concepts: HashMap<ConceptId, Concept>,
    // Forward: (lang, surface) → concepts (a word may belong to multiple concepts/senses)
    surface_to_concepts: HashMap<(String, String), Vec<ConceptId>>,
    // Reverse: concept → all (lang, surface) pairs
    concept_to_surfaces: HashMap<ConceptId, Vec<(String, String)>>,
}

impl ConceptStore {
    pub fn new() -> Self {
        Self {
            concepts: HashMap::new(),
            surface_to_concepts: HashMap::new(),
            concept_to_surfaces: HashMap::new(),
        }
    }

    /// Load concepts + surface bridges from a directory.
    /// Expects `concepts.tsv` and `concept_surfaces.tsv`.
    pub fn load_from_dir<P: AsRef<Path>>(&mut self, base: P) -> std::io::Result<ConceptLoadStats> {
        let base = base.as_ref();
        let mut stats = ConceptLoadStats::default();

        // Prefer concepts_with_pos.tsv if it exists (4 columns)
        let cpath_pos = base.join("concepts_with_pos.tsv");
        let cpath_plain = base.join("concepts.tsv");
        let (cpath, has_pos) = if cpath_pos.exists() {
            (cpath_pos, true)
        } else {
            (cpath_plain, false)
        };

        if cpath.exists() {
            for line in fs::read_to_string(&cpath)?.lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let parts: Vec<&str> = line.splitn(4, '\t').collect();
                if parts.len() < 2 {
                    continue;
                }
                let Some(cid) = ConceptId::from_str(parts[0]) else {
                    continue;
                };
                let english = parts[1].to_string();
                let gloss = parts.get(2).map(|s| s.to_string()).unwrap_or_default();
                let pos = if has_pos {
                    parts.get(3).map(|s| s.to_string()).unwrap_or_default()
                } else {
                    String::new()
                };
                self.concepts.insert(
                    cid,
                    Concept {
                        id: cid,
                        english_anchor: english,
                        gloss,
                        pos,
                    },
                );
                stats.concepts += 1;
            }
        }

        // concept_surfaces.tsv: concept_id<TAB>lang<TAB>surface
        let spath = base.join("concept_surfaces.tsv");
        if spath.exists() {
            for line in fs::read_to_string(&spath)?.lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let parts: Vec<&str> = line.splitn(3, '\t').collect();
                if parts.len() != 3 {
                    continue;
                }
                let Some(cid) = ConceptId::from_str(parts[0]) else {
                    continue;
                };
                let lang = parts[1].to_string();
                let surface = parts[2].to_string();

                self.surface_to_concepts
                    .entry((lang.clone(), surface.clone()))
                    .or_default()
                    .push(cid);
                self.concept_to_surfaces
                    .entry(cid)
                    .or_default()
                    .push((lang, surface));
                stats.surface_links += 1;
            }
        }
        Ok(stats)
    }

    /// Find all concepts matching a surface in a specific language.
    pub fn concepts_for(&self, lang: &str, surface: &str) -> Vec<ConceptId> {
        self.surface_to_concepts
            .get(&(lang.to_string(), surface.to_string()))
            .cloned()
            .unwrap_or_default()
    }

    /// Find concepts matching a surface AND matching a specific POS.
    /// Example: concepts_for_pos("en", "house", "noun") returns only noun senses.
    pub fn concepts_for_pos(&self, lang: &str, surface: &str, target_pos: &str) -> Vec<ConceptId> {
        self.concepts_for(lang, surface)
            .into_iter()
            .filter(|cid| {
                self.concepts
                    .get(cid)
                    .map(|c| c.pos == target_pos)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Pick the "best" concept from a list — the one with coverage in the most languages.
    /// This prefers canonical/common meanings over rare specialized senses.
    pub fn best_concept(&self, concepts: &[ConceptId]) -> Option<ConceptId> {
        concepts
            .iter()
            .max_by_key(|cid| {
                self.concept_to_surfaces
                    .get(cid)
                    .map(|v| {
                        let mut langs = std::collections::HashSet::new();
                        for (lang, _) in v {
                            langs.insert(lang.clone());
                        }
                        langs.len()
                    })
                    .unwrap_or(0)
            })
            .copied()
    }

    /// Best concept matching POS (with graceful fallback to any-POS most-covered).
    pub fn best_concept_for_pos(
        &self,
        lang: &str,
        surface: &str,
        target_pos: &str,
    ) -> Option<ConceptId> {
        let candidates = self.concepts_for_pos(lang, surface, target_pos);
        if !candidates.is_empty() {
            self.best_concept(&candidates)
        } else {
            let all = self.concepts_for(lang, surface);
            self.best_concept(&all)
        }
    }

    /// Get all (language, surface) forms for a concept.
    pub fn surfaces_of(&self, cid: ConceptId) -> Vec<(String, String)> {
        self.concept_to_surfaces
            .get(&cid)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all (language, surface) forms for a concept in a specific target language.
    pub fn surfaces_of_in(&self, cid: ConceptId, target_lang: &str) -> Vec<String> {
        self.concept_to_surfaces
            .get(&cid)
            .map(|v| {
                v.iter()
                    .filter(|(lang, _)| lang == target_lang)
                    .map(|(_, s)| s.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Given a surface in lang A, find all surfaces in lang B via shared concepts.
    /// This is the "translate" operation — without a translation dictionary.
    pub fn cross_translate(
        &self,
        from_lang: &str,
        surface: &str,
        to_lang: &str,
    ) -> Vec<String> {
        let mut result = Vec::new();
        for cid in self.concepts_for(from_lang, surface) {
            for s in self.surfaces_of_in(cid, to_lang) {
                if !result.contains(&s) {
                    result.push(s);
                }
            }
        }
        result
    }

    pub fn get_concept(&self, cid: ConceptId) -> Option<&Concept> {
        self.concepts.get(&cid)
    }

    pub fn concept_count(&self) -> usize {
        self.concepts.len()
    }

    pub fn link_count(&self) -> usize {
        self.surface_to_concepts
            .values()
            .map(|v| v.len())
            .sum()
    }

    /// Iterator over all concept IDs (as u32 for convenience).
    pub fn all_concept_ids(&self) -> Vec<u32> {
        self.concepts.keys().map(|cid| cid.0).collect()
    }
}

impl Default for ConceptStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ConceptLoadStats {
    pub concepts: usize,
    pub surface_links: usize,
}
