//! ZetsEngine — unified facade over MmapCore + MmapLangPack + WAL.
//!
//! This is the single public entrypoint for queries. It handles:
//!   - Opening the mmap'd core
//!   - Lazy loading of language packs on demand
//!   - Replaying WAL records to apply learned updates
//!   - High-level queries (surface → concepts, concept → gloss, etc)
//!
//! Typical usage:
//!
//!   let engine = ZetsEngine::open("data/packs").with_langs(&["he", "en"])?;
//!   let concepts = engine.query("en", "dog");
//!   // Record a new learned fact
//!   engine.learn_pos("en", "tree", PosCode::Noun)?;

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::mmap_core::MmapCore;
use crate::mmap_lang::MmapLangPack;
use crate::piece_graph::{pos_code_to_str, pos_str_to_code, ConceptId, LangId};
use crate::wal::{wal_path_for_core, wal_path_for_lang, Record, RecordKind, WalReader, WalWriter};

/// Result of a full query: concept id + metadata.
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub concept_id: ConceptId,
    pub anchor: String,
    pub gloss: String,
    pub pos: String,
    pub edge_count: usize,
}

/// Learned-POS override: what the WAL has taught us about a particular word.
/// Takes precedence over the pack's built-in POS.
#[derive(Debug, Clone, Copy)]
pub struct PosOverride {
    pub lang_id: LangId,
    pub piece_id: u32,
    pub pos: u8,
    pub deleted: bool,
}

/// Learned word-order rule (per language).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordOrder {
    Undetermined,
    AdjFirst,
    NounFirst,
}

impl WordOrder {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::AdjFirst,
            2 => Self::NounFirst,
            _ => Self::Undetermined,
        }
    }
    pub fn as_u8(self) -> u8 {
        match self {
            Self::AdjFirst => 1,
            Self::NounFirst => 2,
            Self::Undetermined => 0,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AdjFirst => "adj_first",
            Self::NounFirst => "noun_first",
            Self::Undetermined => "undetermined",
        }
    }
}

pub struct ZetsEngine {
    pub core: MmapCore,
    pub lang_packs: HashMap<String, MmapLangPack>,
    /// (lang, piece_id) → learned POS override (from WAL)
    pos_overrides: HashMap<(LangId, u32), u8>,
    /// lang_id → learned word-order rule (from WAL)
    word_orders: HashMap<LangId, (WordOrder, u16)>,
    packs_dir: PathBuf,
    /// Open WAL writer for appending new learned updates.
    core_wal: Option<WalWriter>,
}

impl ZetsEngine {
    /// Open the core pack from a directory. Does NOT load any language packs.
    pub fn open<P: AsRef<Path>>(packs_dir: P) -> io::Result<Self> {
        let packs_dir = packs_dir.as_ref().to_path_buf();
        let core_path = packs_dir.join("zets.core");
        let core = MmapCore::open(&core_path)?;
        let mut engine = Self {
            core,
            lang_packs: HashMap::new(),
            pos_overrides: HashMap::new(),
            word_orders: HashMap::new(),
            packs_dir,
            core_wal: None,
        };
        // Replay core WAL if it exists
        engine.replay_core_wal()?;
        Ok(engine)
    }

    /// Load a language pack. Must be called before querying that language.
    /// Also replays the lang-specific WAL.
    pub fn load_lang(&mut self, code: &str) -> io::Result<()> {
        let path = self.packs_dir.join(format!("zets.{}", code));
        let pack = MmapLangPack::open(&path)?;
        self.lang_packs.insert(code.to_string(), pack);
        self.replay_lang_wal(code)?;
        Ok(())
    }

    /// Convenience: chain multiple language loads.
    pub fn with_langs(mut self, codes: &[&str]) -> io::Result<Self> {
        for c in codes {
            self.load_lang(c)?;
        }
        Ok(self)
    }

    /// Replay the core-level WAL (word-order rules + global updates).
    fn replay_core_wal(&mut self) -> io::Result<()> {
        let path = wal_path_for_core(&self.packs_dir);
        if !path.exists() {
            return Ok(());
        }
        let mut reader = WalReader::open(&path)?;
        while let Some(rec) = reader.next_record()? {
            self.apply_record(&rec);
        }
        Ok(())
    }

    /// Replay a per-language WAL.
    fn replay_lang_wal(&mut self, code: &str) -> io::Result<()> {
        let path = wal_path_for_lang(&self.packs_dir, code);
        if !path.exists() {
            return Ok(());
        }
        let mut reader = WalReader::open(&path)?;
        while let Some(rec) = reader.next_record()? {
            self.apply_record(&rec);
        }
        Ok(())
    }

    fn apply_record(&mut self, rec: &Record) {
        match rec.kind {
            RecordKind::LearnPos => {
                if let Some((lang, pid, pos)) = rec.as_learn_pos() {
                    self.pos_overrides.insert((lang, pid), pos);
                }
            }
            RecordKind::LearnOrder => {
                if let Some((lang, rule, conf)) = rec.as_learn_order() {
                    self.word_orders.insert(lang, (WordOrder::from_u8(rule), conf));
                }
            }
            RecordKind::DeletePos => {
                if let Some((lang, pid)) = rec.as_delete_pos() {
                    self.pos_overrides.remove(&(lang, pid));
                }
            }
            _ => {}
        }
    }

    /// Query: given a language + surface string, return all matching concepts.
    pub fn query(&mut self, lang: &str, surface: &str) -> Vec<QueryResult> {
        let Some(pack) = self.lang_packs.get_mut(lang) else {
            return vec![];
        };
        let concepts = pack.concepts_for_surface(&self.core, surface);
        let mut out = Vec::with_capacity(concepts.len());
        for cid in concepts {
            if let Some(node) = self.core.get_concept(cid) {
                let anchor = self.core.get_piece(node.anchor_piece).to_string();
                let gloss = self.core.get_piece(node.gloss_piece).to_string();
                out.push(QueryResult {
                    concept_id: cid,
                    anchor,
                    gloss,
                    pos: pos_code_to_str(node.pos).to_string(),
                    edge_count: node.edge_count,
                });
            }
        }
        out
    }

    /// Get POS for a surface — checks WAL overrides first, then pack.
    pub fn pos_for_surface(&mut self, lang: &str, surface: &str) -> Option<String> {
        let lang_id = self.lang_id_of(lang)?;
        let pack = self.lang_packs.get_mut(lang)?;
        let piece_id = pack.piece_id_of_surface(&self.core, surface)?;
        // WAL override wins
        if let Some(&pos) = self.pos_overrides.get(&(lang_id, piece_id)) {
            return Some(pos_code_to_str(pos).to_string());
        }
        let pos = pack.pos_for(piece_id);
        if pos == 0 {
            None
        } else {
            Some(pos_code_to_str(pos).to_string())
        }
    }

    /// Get word-order rule for a language (from WAL if learned).
    pub fn word_order(&self, lang: &str) -> (WordOrder, u16) {
        let Some(lang_id) = self.lang_id_of(lang) else {
            return (WordOrder::Undetermined, 0);
        };
        self.word_orders
            .get(&lang_id)
            .copied()
            .unwrap_or((WordOrder::Undetermined, 0))
    }

    /// Resolve a lang code to lang_id using the core's registry.
    fn lang_id_of(&self, lang: &str) -> Option<LangId> {
        self.core
            .lang_codes
            .iter()
            .position(|c| c == lang)
            .map(|i| i as LangId)
    }

    /// Ensure the core WAL is open for writing. Creates if needed.
    fn ensure_wal(&mut self) -> io::Result<()> {
        if self.core_wal.is_none() {
            let path = wal_path_for_core(&self.packs_dir);
            self.core_wal = Some(WalWriter::open(&path)?);
        }
        Ok(())
    }

    /// Record a learned POS for a surface. Persists via WAL.
    pub fn learn_pos(&mut self, lang: &str, surface: &str, pos: &str) -> io::Result<()> {
        let lang_id = self.lang_id_of(lang).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("lang not loaded: {}", lang))
        })?;
        let pack = self.lang_packs.get_mut(lang).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("pack not loaded: {}", lang))
        })?;
        let piece_id = pack
            .piece_id_of_surface(&self.core, surface)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("surface '{}' not in pack", surface),
                )
            })?;
        let pos_code = pos_str_to_code(pos);
        self.ensure_wal()?;
        self.core_wal
            .as_mut()
            .unwrap()
            .append(&Record::learn_pos(lang_id, piece_id, pos_code))?;
        self.pos_overrides.insert((lang_id, piece_id), pos_code);
        Ok(())
    }

    /// Record a learned word-order rule. Persists via WAL.
    pub fn learn_order(&mut self, lang: &str, rule: WordOrder, confidence: u16) -> io::Result<()> {
        let lang_id = self.lang_id_of(lang).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("lang not in core: {}", lang))
        })?;
        self.ensure_wal()?;
        self.core_wal
            .as_mut()
            .unwrap()
            .append(&Record::learn_order(lang_id, rule.as_u8(), confidence))?;
        self.word_orders.insert(lang_id, (rule, confidence));
        Ok(())
    }

    /// Flush pending WAL writes to disk.
    pub fn sync(&mut self) -> io::Result<()> {
        if let Some(w) = self.core_wal.as_mut() {
            w.sync()?;
        }
        Ok(())
    }

    /// Diagnostic stats.
    pub fn stats(&self) -> EngineStats {
        EngineStats {
            core_file_bytes: self.core.mmap_len(),
            concept_count: self.core.concept_count as usize,
            piece_count: self.core.piece_count as usize,
            languages_loaded: self.lang_packs.len(),
            languages_registered: self.core.lang_codes.len(),
            pos_overrides: self.pos_overrides.len(),
            word_order_rules: self.word_orders.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EngineStats {
    pub core_file_bytes: usize,
    pub concept_count: usize,
    pub piece_count: usize,
    pub languages_loaded: usize,
    pub languages_registered: usize,
    pub pos_overrides: usize,
    pub word_order_rules: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_order_encoding_roundtrip() {
        for wo in [
            WordOrder::Undetermined,
            WordOrder::AdjFirst,
            WordOrder::NounFirst,
        ] {
            assert_eq!(WordOrder::from_u8(wo.as_u8()), wo);
        }
    }

    #[test]
    fn word_order_as_str() {
        assert_eq!(WordOrder::AdjFirst.as_str(), "adj_first");
        assert_eq!(WordOrder::NounFirst.as_str(), "noun_first");
        assert_eq!(WordOrder::Undetermined.as_str(), "undetermined");
    }
}
