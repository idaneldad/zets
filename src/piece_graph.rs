//! PieceGraph — unified graph store with pointer-based architecture.
//!
//! Idan's observation (22.04.2026):
//! "אם פוינטר זול אז כל אובייקט שמצביע יחסוך לנו...
//!  אפשר שהוא יצביע לאובייקט מקור כמו פיסה/צומת?"
//!
//! Architecture:
//!
//!   Piece Pool        ← every distinct string stored ONCE
//!   └─ piece_id: u32
//!
//!   Concept Nodes     ← universal graph, shared across all languages
//!   └─ concept_id: u32 → (gloss_piece, pos, edges)
//!
//!   Language Indexes  ← per-lang: (lang_id, surface_piece) → concept_refs
//!   └─ stored separately per language for lazy loading later
//!
//! Everything is u32 indexes — 4 bytes per pointer, never owned strings.

use std::collections::HashMap;

/// A Piece is an atomic content unit — a string, stored ONCE in the pool.
/// Pieces are content-addressed: identical strings → same piece_id.
pub type PieceId = u32;

/// A Concept is a universal meaning, independent of language.
pub type ConceptId = u32;

/// A language is identified by a u8 code (up to 256 languages).
pub type LangId = u8;

/// Edge types — encoded as small integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    /// Cross-language: this concept is expressed by surface X in language Y.
    SurfaceOf = 0,
    /// Semantic relation: A is a synonym of B.
    Synonym = 1,
    /// Semantic relation: A is antonym of B.
    Antonym = 2,
    /// Hierarchy: A is a kind of B (hypernym).
    IsA = 3,
    /// Composition: A is part of B.
    PartOf = 4,
    /// Translation: A in lang X translates to B in lang Y.
    TranslatesTo = 5,
    /// Generic relation.
    Other = 255,
}

impl EdgeKind {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::SurfaceOf,
            1 => Self::Synonym,
            2 => Self::Antonym,
            3 => Self::IsA,
            4 => Self::PartOf,
            5 => Self::TranslatesTo,
            _ => Self::Other,
        }
    }
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// POS (part-of-speech) code — fits in u8.
pub const POS_NONE: u8 = 0;
pub const POS_NOUN: u8 = 1;
pub const POS_VERB: u8 = 2;
pub const POS_ADJ: u8 = 3;
pub const POS_ADV: u8 = 4;
pub const POS_DET: u8 = 5;
pub const POS_PREP: u8 = 6;
pub const POS_CONJ: u8 = 7;
pub const POS_INTERJ: u8 = 8;
pub const POS_PRON: u8 = 9;
pub const POS_NUM: u8 = 10;
pub const POS_PARTICLE: u8 = 11;
pub const POS_PHRASE: u8 = 12;
pub const POS_OTHER: u8 = 15;

pub fn pos_code_to_str(code: u8) -> &'static str {
    match code {
        POS_NONE => "",
        POS_NOUN => "noun",
        POS_VERB => "verb",
        POS_ADJ => "adj",
        POS_ADV => "adv",
        POS_DET => "det",
        POS_PREP => "prep",
        POS_CONJ => "conj",
        POS_INTERJ => "interj",
        POS_PRON => "pron",
        POS_NUM => "num",
        POS_PARTICLE => "particle",
        POS_PHRASE => "phrase",
        _ => "other",
    }
}

pub fn pos_str_to_code(s: &str) -> u8 {
    match s {
        "" => POS_NONE,
        "noun" => POS_NOUN,
        "verb" => POS_VERB,
        "adj" | "adjective" => POS_ADJ,
        "adv" | "adverb" => POS_ADV,
        "det" | "determiner" => POS_DET,
        "prep" | "preposition" => POS_PREP,
        "conj" | "conjunction" => POS_CONJ,
        "interj" | "interjection" => POS_INTERJ,
        "pron" | "pronoun" => POS_PRON,
        "num" => POS_NUM,
        "particle" => POS_PARTICLE,
        "phrase" => POS_PHRASE,
        _ => POS_OTHER,
    }
}

/// A Piece is an interned string. PiecePool deduplicates identical strings.
pub struct PiecePool {
    strings: Vec<String>,
    interner: HashMap<String, PieceId>,
    /// Total bytes saved by dedup (for metrics)
    pub bytes_saved: u64,
    pub bytes_stored: u64,
}

impl PiecePool {
    pub fn new() -> Self {
        // piece_id 0 is reserved for "empty/null"
        let mut p = Self {
            strings: Vec::new(),
            interner: HashMap::new(),
            bytes_saved: 0,
            bytes_stored: 0,
        };
        p.intern("");
        p
    }

    pub fn intern(&mut self, s: &str) -> PieceId {
        if let Some(&id) = self.interner.get(s) {
            self.bytes_saved += s.len() as u64;
            return id;
        }
        let id = self.strings.len() as PieceId;
        self.bytes_stored += s.len() as u64;
        self.strings.push(s.to_string());
        self.interner.insert(s.to_string(), id);
        id
    }

    pub fn lookup(&self, s: &str) -> Option<PieceId> {
        self.interner.get(s).copied()
    }

    pub fn get(&self, id: PieceId) -> &str {
        self.strings
            .get(id as usize)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    pub fn len(&self) -> usize {
        self.strings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Shrink-to-fit both maps for compact representation.
    /// NOTE: we keep the interner because lookup() depends on it.
    /// For truly minimal memory, use a sorted Vec + binary search instead.
    pub fn freeze(&mut self) {
        self.strings.shrink_to_fit();
        self.interner.shrink_to_fit();
    }
}

impl Default for PiecePool {
    fn default() -> Self {
        Self::new()
    }
}

/// Concept Node — universal meaning. All languages reference it.
#[derive(Debug, Clone)]
pub struct ConceptNode {
    pub id: ConceptId,
    /// The English anchor word (piece_id) — for debugging / display
    pub anchor_piece: PieceId,
    /// Short description (piece_id)
    pub gloss_piece: PieceId,
    /// Part of speech
    pub pos: u8,
    /// Outgoing edges: (kind, target concept_id)
    /// Stored as Vec of packed u40 entries (u8 kind + u32 target) for compactness.
    pub edges: Vec<PackedEdge>,
}

/// Packed edge: 5 bytes (u8 kind + u32 target).
/// Using repr(C, packed) to avoid alignment padding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed)]
pub struct PackedEdge {
    pub kind: u8,
    pub target: u32,
}

impl PackedEdge {
    pub fn new(kind: EdgeKind, target: ConceptId) -> Self {
        Self {
            kind: kind.as_u8(),
            target,
        }
    }
    pub fn kind_enum(&self) -> EdgeKind {
        EdgeKind::from_u8(self.kind)
    }
    pub fn target_id(&self) -> ConceptId {
        self.target
    }
}

/// Language registry — u8 codes for up to 256 languages.
pub struct LangRegistry {
    codes: Vec<String>,
    by_code: HashMap<String, LangId>,
}

impl LangRegistry {
    pub fn new() -> Self {
        Self {
            codes: Vec::new(),
            by_code: HashMap::new(),
        }
    }

    pub fn intern(&mut self, code: &str) -> LangId {
        if let Some(&id) = self.by_code.get(code) {
            return id;
        }
        let id = self.codes.len() as LangId;
        self.codes.push(code.to_string());
        self.by_code.insert(code.to_string(), id);
        id
    }

    pub fn get(&self, code: &str) -> Option<LangId> {
        self.by_code.get(code).copied()
    }

    pub fn name(&self, id: LangId) -> &str {
        self.codes
            .get(id as usize)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    pub fn all(&self) -> &[String] {
        &self.codes
    }

    pub fn len(&self) -> usize {
        self.codes.len()
    }
}

impl Default for LangRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Language-specific data: surface form → [concept_ids].
/// One instance per language. Can be loaded lazily.
pub struct LangIndex {
    pub lang_id: LangId,
    /// surface_piece_id → list of concept_ids that use this surface
    pub surface_to_concepts: HashMap<PieceId, Vec<ConceptId>>,
    /// surface_piece_id → POS code (if known from per-lang pos.tsv)
    pub surface_pos: HashMap<PieceId, u8>,
    /// Synonyms within this language: surface_piece → [synonym_surface_pieces]
    pub synonyms: HashMap<PieceId, Vec<PieceId>>,
    /// Antonyms within this language
    pub antonyms: HashMap<PieceId, Vec<PieceId>>,
    /// Per-surface definitions: surface → [gloss_piece_ids] in this language
    pub definitions: HashMap<PieceId, Vec<PieceId>>,
}

impl LangIndex {
    pub fn new(lang_id: LangId) -> Self {
        Self {
            lang_id,
            surface_to_concepts: HashMap::new(),
            surface_pos: HashMap::new(),
            synonyms: HashMap::new(),
            antonyms: HashMap::new(),
            definitions: HashMap::new(),
        }
    }

    pub fn entry_count(&self) -> usize {
        // Count all distinct surface pieces that have some data
        let mut all: std::collections::HashSet<&crate::piece_graph::PieceId> = std::collections::HashSet::new();
        all.extend(self.surface_to_concepts.keys());
        all.extend(self.surface_pos.keys());
        all.extend(self.synonyms.keys());
        all.extend(self.antonyms.keys());
        all.extend(self.definitions.keys());
        all.len()
    }

    pub fn shrink(&mut self) {
        self.surface_to_concepts.shrink_to_fit();
        self.surface_pos.shrink_to_fit();
        self.synonyms.shrink_to_fit();
        self.antonyms.shrink_to_fit();
        self.definitions.shrink_to_fit();
    }
}

/// The main unified graph store.
pub struct PieceGraph {
    pub pieces: PiecePool,
    pub langs: LangRegistry,
    pub concepts: Vec<ConceptNode>,
    /// Per-language indexes (can be loaded lazily in stage 2)
    pub lang_indexes: HashMap<LangId, LangIndex>,
    /// Stats
    pub stats: GraphStats,
}

#[derive(Debug, Default, Clone)]
pub struct GraphStats {
    pub pieces_interned: usize,
    pub concepts_loaded: usize,
    pub edges_total: usize,
    pub edges_deduped: usize,
    pub definitions: usize,
    pub synonyms: usize,
    pub antonyms: usize,
    pub pos_tags: usize,
}

impl PieceGraph {
    pub fn new() -> Self {
        Self {
            pieces: PiecePool::new(),
            langs: LangRegistry::new(),
            concepts: Vec::new(),
            lang_indexes: HashMap::new(),
            stats: GraphStats::default(),
        }
    }

    /// Freeze all pools — drop interners after loading is done.
    pub fn freeze(&mut self) {
        self.pieces.freeze();
        for idx in self.lang_indexes.values_mut() {
            idx.shrink();
        }
        self.concepts.shrink_to_fit();
    }

    pub fn get_concept(&self, id: ConceptId) -> Option<&ConceptNode> {
        self.concepts.get(id as usize)
    }

    pub fn concept_count(&self) -> usize {
        self.concepts.len()
    }

    /// Lookup concepts that have a given surface in a given language.
    pub fn concepts_for_surface(&self, lang: &str, surface: &str) -> Vec<ConceptId> {
        let Some(lang_id) = self.langs.get(lang) else {
            return vec![];
        };
        let Some(idx) = self.lang_indexes.get(&lang_id) else {
            return vec![];
        };
        let Some(piece_id) = self.pieces.lookup(surface) else {
            return vec![];
        };
        idx.surface_to_concepts
            .get(&piece_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Lookup surfaces for a concept in a given language.
    pub fn surfaces_of_concept(&self, concept_id: ConceptId, lang: &str) -> Vec<String> {
        let Some(lang_id) = self.langs.get(lang) else {
            return vec![];
        };
        let Some(idx) = self.lang_indexes.get(&lang_id) else {
            return vec![];
        };
        let mut result = Vec::new();
        for (piece_id, concept_list) in &idx.surface_to_concepts {
            if concept_list.contains(&concept_id) {
                result.push(self.pieces.get(*piece_id).to_string());
            }
        }
        result
    }
}

impl Default for PieceGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piece_pool_dedups() {
        let mut pool = PiecePool::new();
        let a = pool.intern("hello");
        let b = pool.intern("hello");
        let c = pool.intern("world");
        assert_eq!(a, b);
        assert_ne!(a, c);
        // pool.len() includes the empty string at index 0
        assert_eq!(pool.len(), 3);
    }

    #[test]
    fn piece_pool_empty_is_zero() {
        let pool = PiecePool::new();
        assert_eq!(pool.lookup(""), Some(0));
    }

    #[test]
    fn lang_registry_reuses_ids() {
        let mut r = LangRegistry::new();
        let en = r.intern("en");
        let he = r.intern("he");
        let en2 = r.intern("en");
        assert_eq!(en, en2);
        assert_ne!(en, he);
        assert_eq!(r.name(en), "en");
    }

    #[test]
    fn pos_codes_roundtrip() {
        for code in 0..=15u8 {
            let s = pos_code_to_str(code);
            let back = pos_str_to_code(s);
            if code <= POS_PHRASE || code == POS_OTHER {
                // valid codes should roundtrip
                assert_eq!(
                    back,
                    if s.is_empty() { POS_NONE } else { code },
                    "code {} → {:?} → {}",
                    code,
                    s,
                    back
                );
            }
        }
    }

    #[test]
    fn packed_edge_size_is_minimal() {
        // Due to repr(C, packed), size should be 5 bytes
        assert_eq!(std::mem::size_of::<PackedEdge>(), 5);
    }

    #[test]
    fn edge_kind_roundtrip() {
        for kind in &[
            EdgeKind::SurfaceOf,
            EdgeKind::Synonym,
            EdgeKind::Antonym,
            EdgeKind::IsA,
            EdgeKind::TranslatesTo,
        ] {
            let v = kind.as_u8();
            assert_eq!(EdgeKind::from_u8(v), *kind);
        }
    }

    #[test]
    fn concept_lookup_empty() {
        let g = PieceGraph::new();
        assert!(g.get_concept(0).is_none());
        assert!(g.concepts_for_surface("en", "dog").is_empty());
    }
}
