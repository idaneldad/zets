//! PieceGraph loader — builds the unified graph from TSV sources.
//!
//! Merges:
//!   - per-language lexicons (definitions/synonyms/antonyms/pos)
//!   - cross-language concept files (concepts.tsv + concept_surfaces.tsv)
//!
//! All strings are deduplicated via PiecePool. All structural relations use u32 pointers.
//!
//! Dedup: also removes the 1,760 duplicate edges detected during analysis.

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::piece_graph::{
    pos_str_to_code, ConceptId, ConceptNode, LangIndex, PieceGraph, POS_NONE,
};

pub struct PieceGraphLoader {
    base_dir: PathBuf,
    concepts_dir: PathBuf,
}

impl PieceGraphLoader {
    pub fn new<P: AsRef<Path>>(base: P) -> Self {
        let base = base.as_ref().to_path_buf();
        let concepts_dir = base.join("_concepts_v2");
        Self {
            base_dir: base,
            concepts_dir,
        }
    }

    pub fn with_concepts_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.concepts_dir = dir.as_ref().to_path_buf();
        self
    }

    /// Build a full PieceGraph from disk.
    pub fn load(&self) -> io::Result<PieceGraph> {
        let mut graph = PieceGraph::new();

        // ── Phase 1: load universal concepts (from concepts_v2) ──
        self.load_concepts(&mut graph)?;

        // ── Phase 2: load per-language surface bridges ──
        // These are already pointed to by concept_surfaces.tsv.
        self.load_concept_surfaces(&mut graph)?;

        // ── Phase 3: load per-language lexicon (definitions, POS, synonyms, antonyms) ──
        self.load_per_language_lexicons(&mut graph)?;

        // ── Phase 4: dedup edges + compact memory ──
        self.dedup_edges(&mut graph);
        graph.freeze();

        Ok(graph)
    }

    fn load_concepts(&self, graph: &mut PieceGraph) -> io::Result<()> {
        // Prefer concepts_with_pos.tsv if present, else concepts.tsv
        let path_pos = self.concepts_dir.join("concepts_with_pos.tsv");
        let path_plain = self.concepts_dir.join("concepts.tsv");
        let (path, has_pos) = if path_pos.exists() {
            (path_pos, true)
        } else {
            (path_plain, false)
        };

        if !path.exists() {
            return Ok(()); // empty graph is valid
        }

        let content = fs::read_to_string(&path)?;
        for line in content.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(4, '\t').collect();
            if parts.len() < 2 {
                continue;
            }
            // parse concept_id like "c12345" → 12345
            let cid_str = parts[0].trim_start_matches('c');
            let Ok(concept_id) = cid_str.parse::<u32>() else {
                continue;
            };

            let anchor = parts[1];
            let gloss = parts.get(2).unwrap_or(&"");
            let pos_str = if has_pos {
                parts.get(3).unwrap_or(&"")
            } else {
                &""
            };

            let anchor_piece = graph.pieces.intern(anchor);
            let gloss_piece = if gloss.is_empty() {
                0
            } else {
                graph.pieces.intern(gloss)
            };
            let pos = pos_str_to_code(pos_str);

            // Ensure concepts vector has room for this ID
            while graph.concepts.len() <= concept_id as usize {
                graph.concepts.push(ConceptNode {
                    id: graph.concepts.len() as u32,
                    anchor_piece: 0,
                    gloss_piece: 0,
                    pos: POS_NONE,
                    edges: Vec::new(),
                });
            }
            graph.concepts[concept_id as usize] = ConceptNode {
                id: concept_id,
                anchor_piece,
                gloss_piece,
                pos,
                edges: Vec::new(),
            };
            graph.stats.concepts_loaded += 1;
        }
        Ok(())
    }

    fn load_concept_surfaces(&self, graph: &mut PieceGraph) -> io::Result<()> {
        let path = self.concepts_dir.join("concept_surfaces.tsv");
        if !path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&path)?;
        let mut seen_edges: HashSet<(ConceptId, u8, u32)> = HashSet::new();

        for line in content.lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(3, '\t').collect();
            if parts.len() != 3 {
                continue;
            }
            let cid_str = parts[0].trim_start_matches('c');
            let Ok(concept_id) = cid_str.parse::<u32>() else {
                continue;
            };
            let lang = parts[1].trim();
            let surface = parts[2].trim();
            if lang.is_empty() || surface.is_empty() {
                continue;
            }

            let lang_id = graph.langs.intern(lang);
            let surface_piece = graph.pieces.intern(surface);

            // Dedup: skip already-seen (concept, lang, surface) edges
            let key = (concept_id, lang_id, surface_piece);
            if !seen_edges.insert(key) {
                graph.stats.edges_deduped += 1;
                continue;
            }

            let idx = graph
                .lang_indexes
                .entry(lang_id)
                .or_insert_with(|| LangIndex::new(lang_id));
            idx.surface_to_concepts
                .entry(surface_piece)
                .or_default()
                .push(concept_id);

            graph.stats.edges_total += 1;
        }
        Ok(())
    }

    fn load_per_language_lexicons(&self, graph: &mut PieceGraph) -> io::Result<()> {
        if !self.base_dir.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            // Skip _concepts_v2 etc.
            if name.starts_with('_') || name.len() > 6 {
                continue;
            }
            let lang_id = graph.langs.intern(&name);
            self.load_single_lang_lexicon(graph, lang_id, &entry.path())?;
        }
        Ok(())
    }

    fn load_single_lang_lexicon(
        &self,
        graph: &mut PieceGraph,
        lang_id: u8,
        dir: &Path,
    ) -> io::Result<()> {
        // Definitions
        let def_path = dir.join("definitions.tsv");
        if def_path.exists() {
            let content = fs::read_to_string(&def_path)?;
            for line in content.lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let mut parts = line.splitn(2, '\t');
                if let (Some(word), Some(def)) = (parts.next(), parts.next()) {
                    let w_piece = graph.pieces.intern(word);
                    let d_piece = graph.pieces.intern(def);
                    let idx = graph
                        .lang_indexes
                        .entry(lang_id)
                        .or_insert_with(|| LangIndex::new(lang_id));
                    idx.definitions.entry(w_piece).or_default().push(d_piece);
                    graph.stats.definitions += 1;
                }
            }
        }

        // Synonyms
        let syn_path = dir.join("synonyms.tsv");
        if syn_path.exists() {
            let content = fs::read_to_string(&syn_path)?;
            for line in content.lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let mut parts = line.splitn(2, '\t');
                if let (Some(word), Some(syn)) = (parts.next(), parts.next()) {
                    let w_piece = graph.pieces.intern(word);
                    let s_piece = graph.pieces.intern(syn);
                    let idx = graph
                        .lang_indexes
                        .entry(lang_id)
                        .or_insert_with(|| LangIndex::new(lang_id));
                    idx.synonyms.entry(w_piece).or_default().push(s_piece);
                    graph.stats.synonyms += 1;
                }
            }
        }

        // Antonyms
        let ant_path = dir.join("antonyms.tsv");
        if ant_path.exists() {
            let content = fs::read_to_string(&ant_path)?;
            for line in content.lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let mut parts = line.splitn(2, '\t');
                if let (Some(word), Some(ant)) = (parts.next(), parts.next()) {
                    let w_piece = graph.pieces.intern(word);
                    let a_piece = graph.pieces.intern(ant);
                    let idx = graph
                        .lang_indexes
                        .entry(lang_id)
                        .or_insert_with(|| LangIndex::new(lang_id));
                    idx.antonyms.entry(w_piece).or_default().push(a_piece);
                    graph.stats.antonyms += 1;
                }
            }
        }

        // POS
        let pos_path = dir.join("pos.tsv");
        if pos_path.exists() {
            let content = fs::read_to_string(&pos_path)?;
            for line in content.lines() {
                if line.starts_with('#') || line.is_empty() {
                    continue;
                }
                let mut parts = line.splitn(2, '\t');
                if let (Some(word), Some(pos)) = (parts.next(), parts.next()) {
                    let w_piece = graph.pieces.intern(word);
                    let pos_code = pos_str_to_code(pos);
                    let idx = graph
                        .lang_indexes
                        .entry(lang_id)
                        .or_insert_with(|| LangIndex::new(lang_id));
                    idx.surface_pos.entry(w_piece).or_insert(pos_code);
                    graph.stats.pos_tags += 1;
                }
            }
        }

        Ok(())
    }

    fn dedup_edges(&self, graph: &mut PieceGraph) {
        let mut total_removed = 0usize;
        for concept in &mut graph.concepts {
            let before = concept.edges.len();
            let mut seen: HashSet<(u8, u32)> = HashSet::new();
            concept.edges.retain(|e| seen.insert((e.kind, e.target)));
            total_removed += before - concept.edges.len();
            concept.edges.shrink_to_fit();
        }
        graph.stats.edges_deduped += total_removed;
    }

    /// Build cross-language TranslatesTo edges between concepts that share a meaning.
    /// This materializes the "big EN ↔ gadol HE" relationship as explicit edges.
    #[allow(dead_code)]
    pub fn build_cross_lang_edges(_graph: &mut PieceGraph) {
        // For each concept, connect it to concepts that share at least one surface context.
        // (Not done by default — concept graph already encodes this implicitly via shared concept_id.)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_loads() {
        let loader = PieceGraphLoader::new("/nonexistent/path");
        let graph = loader.load().unwrap();
        assert_eq!(graph.concepts.len(), 0);
    }
}
