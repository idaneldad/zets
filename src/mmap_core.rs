#![allow(unsafe_code)]
//! mmap-based reader for zets.core — OS-level lazy loading.
//!
//! Instead of reading the whole core file into memory, we memory-map it.
//! The OS loads pages (4KB) only when we actually access them.
//! RAM usage reflects only "hot" portions of the data.
//!
//! API:
//!   let reader = MmapCore::open("data/packs/zets.core")?;
//!   let concept = reader.get_concept(1625)?;
//!   let str = reader.get_piece(piece_id)?;
//!
//! The mmap handle lives as long as MmapCore does.

use memmap2::Mmap;
use std::fs::File;
use std::io::{self};
use std::path::Path;

use crate::pack::{CORE_MAGIC, FORMAT_VERSION};
use crate::piece_graph::{ConceptId, PieceId};

/// Section offsets stored in the core header, 5 sections × (offset, length).
#[derive(Debug, Clone, Copy)]
pub struct SectionOffsets {
    pub lang_registry: (u64, u64),
    pub pieces: (u64, u64),
    pub concepts: (u64, u64),
    pub reserved4: (u64, u64),
    pub reserved5: (u64, u64),
}

/// mmap-backed reader for zets.core.
///
/// The entire file is memory-mapped. Accessing data triggers page faults
/// that the OS services lazily — only pages you touch end up in RAM.
pub struct MmapCore {
    mmap: Mmap,
    #[allow(dead_code)]
    sections: SectionOffsets,
    /// Decoded language registry (small, eager read)
    pub lang_codes: Vec<String>,
    /// Number of pieces (known from header, but strings not eagerly decoded)
    pub piece_count: u32,
    /// Offsets of each piece within the pieces section (lazy lookup)
    piece_offsets: Vec<u32>,
    /// Number of concepts
    pub concept_count: u32,
    /// Offsets of each concept (by concept_id) within the concepts section
    concept_offsets: Vec<u32>,
}

impl MmapCore {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        // Safety: mmap is inherently unsafe because external modifications to the file
        // can invalidate the map. We treat the core file as read-only during the lifetime
        // of this process.
        let mmap = unsafe { Mmap::map(&file)? };

        if mmap.len() < 16 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "core file too small",
            ));
        }

        // Verify header
        if &mmap[0..4] != CORE_MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "bad core magic"));
        }
        let version = u32::from_le_bytes(mmap[4..8].try_into().unwrap());
        if version != FORMAT_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported version {}", version),
            ));
        }
        let _flags = u32::from_le_bytes(mmap[8..12].try_into().unwrap());

        // Skip 32 reserved bytes → section table at offset 44
        let section_table_offset = 44usize;
        if mmap.len() < section_table_offset + 80 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "header truncated",
            ));
        }

        let read_u64 = |off: usize| u64::from_le_bytes(mmap[off..off + 8].try_into().unwrap());
        let sections = SectionOffsets {
            lang_registry: (
                read_u64(section_table_offset),
                read_u64(section_table_offset + 8),
            ),
            pieces: (
                read_u64(section_table_offset + 16),
                read_u64(section_table_offset + 24),
            ),
            concepts: (
                read_u64(section_table_offset + 32),
                read_u64(section_table_offset + 40),
            ),
            reserved4: (
                read_u64(section_table_offset + 48),
                read_u64(section_table_offset + 56),
            ),
            reserved5: (
                read_u64(section_table_offset + 64),
                read_u64(section_table_offset + 72),
            ),
        };

        // Read language registry (small, always eager)
        let mut lang_codes = Vec::new();
        let lang_start = sections.lang_registry.0 as usize;
        if lang_start + 2 <= mmap.len() {
            let n = u16::from_le_bytes(mmap[lang_start..lang_start + 2].try_into().unwrap()) as usize;
            let mut pos = lang_start + 2;
            for _ in 0..n {
                if pos + 6 > mmap.len() {
                    break;
                }
                let code_bytes = &mmap[pos..pos + 6];
                let code = String::from_utf8_lossy(code_bytes)
                    .trim_end_matches('\0')
                    .to_string();
                lang_codes.push(code);
                pos += 6;
            }
        }

        // Build piece_offsets index (scan once — small compared to actual strings)
        let pieces_start = sections.pieces.0 as usize;
        let pieces_end = pieces_start + sections.pieces.1 as usize;
        let piece_count = u32::from_le_bytes(mmap[pieces_start..pieces_start + 4].try_into().unwrap());
        let mut piece_offsets = Vec::with_capacity(piece_count as usize);
        let mut pos = pieces_start + 4;
        for _ in 0..piece_count {
            if pos + 2 > pieces_end {
                break;
            }
            piece_offsets.push(pos as u32);
            let len = u16::from_le_bytes(mmap[pos..pos + 2].try_into().unwrap()) as usize;
            pos += 2 + len;
        }

        // Build concept_offsets index
        let concepts_start = sections.concepts.0 as usize;
        let concepts_end = concepts_start + sections.concepts.1 as usize;
        let concept_count =
            u32::from_le_bytes(mmap[concepts_start..concepts_start + 4].try_into().unwrap());
        // Since concepts are written with their IDs (and we auto-grow to accommodate),
        // we need a sparse map: concept_id → file offset.
        // Max concept_id could be up to concept_count (but sparse).
        // We'll read sequentially once to build the index.
        let mut concept_offsets = Vec::new();
        let mut pos = concepts_start + 4;
        while pos + 17 <= concepts_end {
            let id_bytes: [u8; 4] = mmap[pos..pos + 4].try_into().unwrap();
            let cid = u32::from_le_bytes(id_bytes);
            // anchor:4 + gloss:4 + pos:1 + edge_count:4
            let record_start = pos;
            let edge_count_pos = pos + 13; // after id(4)+anchor(4)+gloss(4)+pos(1)
            let edge_count =
                u32::from_le_bytes(mmap[edge_count_pos..edge_count_pos + 4].try_into().unwrap())
                    as usize;
            // total record: 17 bytes + edge_count*5
            let record_size = 17 + edge_count * 5;
            // Extend index to fit cid
            while concept_offsets.len() <= cid as usize {
                concept_offsets.push(u32::MAX); // not-present sentinel
            }
            concept_offsets[cid as usize] = record_start as u32;
            pos += record_size;
        }

        Ok(Self {
            mmap,
            sections,
            lang_codes,
            piece_count,
            piece_offsets,
            concept_count,
            concept_offsets,
        })
    }

    /// Get a piece string by id — zero-copy slice of the mmap.
    /// Returns empty string if id is invalid.
    pub fn get_piece(&self, id: PieceId) -> &str {
        let Some(&off) = self.piece_offsets.get(id as usize) else {
            return "";
        };
        let off = off as usize;
        if off + 2 > self.mmap.len() {
            return "";
        }
        let len = u16::from_le_bytes(self.mmap[off..off + 2].try_into().unwrap()) as usize;
        let start = off + 2;
        let end = start + len;
        if end > self.mmap.len() {
            return "";
        }
        std::str::from_utf8(&self.mmap[start..end]).unwrap_or("")
    }

    /// Get concept metadata (anchor, gloss, pos, edges) for a given concept_id.
    pub fn get_concept(&self, cid: ConceptId) -> Option<MmapConcept<'_>> {
        let off = self.concept_offsets.get(cid as usize).copied()?;
        if off == u32::MAX {
            return None;
        }
        let off = off as usize;
        if off + 17 > self.mmap.len() {
            return None;
        }
        let id = u32::from_le_bytes(self.mmap[off..off + 4].try_into().unwrap());
        let anchor = u32::from_le_bytes(self.mmap[off + 4..off + 8].try_into().unwrap());
        let gloss = u32::from_le_bytes(self.mmap[off + 8..off + 12].try_into().unwrap());
        let pos = self.mmap[off + 12];
        let edge_count =
            u32::from_le_bytes(self.mmap[off + 13..off + 17].try_into().unwrap()) as usize;
        let edges_slice = &self.mmap[off + 17..off + 17 + edge_count * 5];
        Some(MmapConcept {
            id,
            anchor_piece: anchor,
            gloss_piece: gloss,
            pos,
            edge_count,
            edges_bytes: edges_slice,
        })
    }

    pub fn mmap_len(&self) -> usize {
        self.mmap.len()
    }
}

/// Lightweight concept view — holds a slice into the mmap.
pub struct MmapConcept<'a> {
    pub id: ConceptId,
    pub anchor_piece: PieceId,
    pub gloss_piece: PieceId,
    pub pos: u8,
    pub edge_count: usize,
    pub edges_bytes: &'a [u8],
}

impl<'a> MmapConcept<'a> {
    /// Iterate edges without allocating.
    pub fn edges(&self) -> impl Iterator<Item = (u8, ConceptId)> + '_ {
        (0..self.edge_count).map(move |i| {
            let off = i * 5;
            let kind = self.edges_bytes[off];
            let target =
                u32::from_le_bytes(self.edges_bytes[off + 1..off + 5].try_into().unwrap());
            (kind, target)
        })
    }
}

// Keep std::io::Read in scope to avoid "unused import" warning in some builds
#[allow(unused_imports)]
use std::fs;
