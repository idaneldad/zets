#![allow(unsafe_code)]
//! mmap-based reader for per-language packs (`zets.<lang>`).
//!
//! Each language pack has 5 sections:
//!   1. surface_to_concepts (piece_id → Vec<concept_id>)
//!   2. surface_pos (piece_id → pos_code u8)
//!   3. synonyms (piece_id → Vec<piece_id>)
//!   4. antonyms (piece_id → Vec<piece_id>)
//!   5. definitions (piece_id → Vec<piece_id>)
//!
//! We build a small in-memory HashMap of (key_piece → section_offset) for O(1)
//! lookup, but values (the Vec<u32> lists) stay in the mmap until accessed.

use memmap2::Mmap;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;

use crate::pack::{FORMAT_VERSION, LANG_MAGIC};
use crate::piece_graph::{ConceptId, LangId, PieceId};

pub struct MmapLangPack {
    mmap: Mmap,
    pub lang_code: String,
    pub lang_id: LangId,
    // section_offset_table: 5 × (offset, length)
    pub sections: [(u64, u64); 5],
    // Section 0: surface → concepts (list)
    pub surface_to_concepts_index: HashMap<PieceId, u32>, // piece → offset in section
    // Section 1: surface → POS (byte)
    pub surface_pos: HashMap<PieceId, u8>, // small enough to load eagerly
    // Section 2: synonyms index
    pub synonyms_index: HashMap<PieceId, u32>,
    // Section 3: antonyms index
    pub antonyms_index: HashMap<PieceId, u32>,
    // Section 4: definitions index
    pub definitions_index: HashMap<PieceId, u32>,
    /// Reverse lookup: surface string → piece_id.
    /// Built lazily on first query via `build_surface_lookup`.
    surface_lookup: Option<HashMap<String, PieceId>>,
}

impl MmapLangPack {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        if mmap.len() < 53 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "lang file too small"));
        }

        // Header: magic(4) + version(4) + flags(4) + code(6) + lang_id(1) + reserved(16) = 35 bytes
        if &mmap[0..4] != LANG_MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "bad lang magic"));
        }
        let version = u32::from_le_bytes(mmap[4..8].try_into().unwrap());
        if version != FORMAT_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported version {}", version),
            ));
        }
        // flags at 8..12
        let code_bytes = &mmap[12..18];
        let lang_code = String::from_utf8_lossy(code_bytes)
            .trim_end_matches('\0')
            .to_string();
        let lang_id = mmap[18];
        // 16 bytes reserved → header ends at 35
        let section_table_offset = 35usize;

        // Read 5 sections × 16 bytes
        let mut sections = [(0u64, 0u64); 5];
        for (i, s) in sections.iter_mut().enumerate() {
            let off = section_table_offset + i * 16;
            s.0 = u64::from_le_bytes(mmap[off..off + 8].try_into().unwrap());
            s.1 = u64::from_le_bytes(mmap[off + 8..off + 16].try_into().unwrap());
        }

        // Build indexes for each section
        let mut pack = Self {
            mmap,
            lang_code,
            lang_id,
            sections,
            surface_to_concepts_index: HashMap::new(),
            surface_pos: HashMap::new(),
            synonyms_index: HashMap::new(),
            antonyms_index: HashMap::new(),
            definitions_index: HashMap::new(),
            surface_lookup: None,
        };

        // Section 0: surface → list of concepts
        pack.build_list_index(0, |m| &mut m.surface_to_concepts_index);
        // Section 1: surface → pos byte (eagerly loaded since tiny)
        pack.load_surface_pos();
        // Section 2: synonyms
        pack.build_list_index(2, |m| &mut m.synonyms_index);
        // Section 3: antonyms
        pack.build_list_index(3, |m| &mut m.antonyms_index);
        // Section 4: definitions
        pack.build_list_index(4, |m| &mut m.definitions_index);

        Ok(pack)
    }

    fn build_list_index<F>(&mut self, section_idx: usize, selector: F)
    where
        F: FnOnce(&mut Self) -> &mut HashMap<PieceId, u32>,
    {
        let (off, len) = self.sections[section_idx];
        let start = off as usize;
        let end = start + len as usize;
        if start + 4 > end || end > self.mmap.len() {
            return;
        }
        let n = u32::from_le_bytes(self.mmap[start..start + 4].try_into().unwrap()) as usize;
        let mut pos = start + 4;
        // Collect pairs first to avoid borrow conflict
        let mut pairs = Vec::with_capacity(n);
        for _ in 0..n {
            if pos + 8 > end {
                break;
            }
            let key = u32::from_le_bytes(self.mmap[pos..pos + 4].try_into().unwrap());
            let item_pos = (pos + 4) as u32; // points to the u32 count + items
            let count =
                u32::from_le_bytes(self.mmap[pos + 4..pos + 8].try_into().unwrap()) as usize;
            pairs.push((key, item_pos));
            pos += 8 + count * 4;
        }
        let idx = selector(self);
        idx.reserve(pairs.len());
        for (k, v) in pairs {
            idx.insert(k, v);
        }
    }

    fn load_surface_pos(&mut self) {
        let (off, len) = self.sections[1];
        let start = off as usize;
        let end = start + len as usize;
        if start + 4 > end || end > self.mmap.len() {
            return;
        }
        let n = u32::from_le_bytes(self.mmap[start..start + 4].try_into().unwrap()) as usize;
        let mut pos = start + 4;
        self.surface_pos.reserve(n);
        for _ in 0..n {
            if pos + 5 > end {
                break;
            }
            let key = u32::from_le_bytes(self.mmap[pos..pos + 4].try_into().unwrap());
            let val = self.mmap[pos + 4];
            self.surface_pos.insert(key, val);
            pos += 5;
        }
    }

    /// Read a list at a given offset — zero-copy iter over u32 items.
    fn read_list_at(&self, offset_to_count: u32) -> &[u8] {
        let pos = offset_to_count as usize;
        if pos + 4 > self.mmap.len() {
            return &[];
        }
        let count = u32::from_le_bytes(self.mmap[pos..pos + 4].try_into().unwrap()) as usize;
        let start = pos + 4;
        let end = start + count * 4;
        if end > self.mmap.len() {
            return &[];
        }
        &self.mmap[start..end]
    }

    /// Get concepts that use a given surface piece.
    pub fn concepts_for_surface_piece(&self, piece: PieceId) -> Vec<ConceptId> {
        let Some(&off) = self.surface_to_concepts_index.get(&piece) else {
            return vec![];
        };
        let bytes = self.read_list_at(off);
        bytes
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
            .collect()
    }

    /// Get synonyms of a surface piece (as piece_ids).
    pub fn synonyms_for(&self, piece: PieceId) -> Vec<PieceId> {
        let Some(&off) = self.synonyms_index.get(&piece) else {
            return vec![];
        };
        let bytes = self.read_list_at(off);
        bytes
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
            .collect()
    }

    /// Get antonyms of a surface piece.
    pub fn antonyms_for(&self, piece: PieceId) -> Vec<PieceId> {
        let Some(&off) = self.antonyms_index.get(&piece) else {
            return vec![];
        };
        let bytes = self.read_list_at(off);
        bytes
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
            .collect()
    }

    /// Get definitions (as gloss piece_ids) for a surface.
    pub fn definitions_for(&self, piece: PieceId) -> Vec<PieceId> {
        let Some(&off) = self.definitions_index.get(&piece) else {
            return vec![];
        };
        let bytes = self.read_list_at(off);
        bytes
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
            .collect()
    }

    /// Get POS code for a surface piece.
    pub fn pos_for(&self, piece: PieceId) -> u8 {
        self.surface_pos.get(&piece).copied().unwrap_or(0)
    }

    pub fn mmap_len(&self) -> usize {
        self.mmap.len()
    }

    pub fn surface_count(&self) -> usize {
        self.surface_to_concepts_index.len()
    }

    /// Build reverse lookup: surface string → piece_id.
    /// Called once on first `piece_id_of_surface` call. Cost: ~O(surface_count)
    /// strings fetched from the core mmap (not owned by this pack).
    /// Typical: 50K-300K entries per language pack = 2-15 MB RAM.
    fn ensure_surface_lookup(&mut self, core: &crate::mmap_core::MmapCore) {
        if self.surface_lookup.is_some() {
            return;
        }
        let mut map = HashMap::with_capacity(self.surface_to_concepts_index.len());
        for &piece_id in self.surface_to_concepts_index.keys() {
            let s = core.get_piece(piece_id);
            if !s.is_empty() {
                map.insert(s.to_string(), piece_id);
            }
        }
        self.surface_lookup = Some(map);
    }

    /// Look up surface → piece_id in O(1) after first call.
    pub fn piece_id_of_surface(&mut self, core: &crate::mmap_core::MmapCore, surface: &str) -> Option<PieceId> {
        self.ensure_surface_lookup(core);
        self.surface_lookup.as_ref().and_then(|m| m.get(surface).copied())
    }

    /// Full query: surface string → concept_ids (no scan).
    pub fn concepts_for_surface(&mut self, core: &crate::mmap_core::MmapCore, surface: &str) -> Vec<ConceptId> {
        let piece_id = match self.piece_id_of_surface(core, surface) {
            Some(p) => p,
            None => return vec![],
        };
        self.concepts_for_surface_piece(piece_id)
    }
}
