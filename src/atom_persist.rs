//! `atom_persist` — serialize/deserialize AtomStore to a binary file.
//!
//! Idan's requirement: autonomous learning can't work if the graph doesn't
//! survive a restart. This module handles the write/read round-trip.
//!
//! Binary format (little-endian, simple, versioned):
//!
//!   magic:            4 bytes = b"ZETS"
//!   version:          u32 = 1
//!   atom_count:       u64
//!   edge_count:       u64
//!   // atoms section
//!   for each atom:
//!       kind:   u8
//!       len:    u32
//!       data:   `len` bytes
//!       prov:   u64 (content_hash, redundant but useful for audit)
//!   // edges section
//!   for each edge:
//!       from:     u32
//!       to:       u32
//!       relation: u8
//!       weight:   u8
//!       slot:     u16
//!
//! Why not just serde+bincode? We want deterministic byte-level format.
//! Whatever platform writes this file, another can read it byte-for-byte.

use std::io::Write;
use std::path::Path;

use crate::atoms::{AtomKind, AtomStore};

const MAGIC: &[u8; 4] = b"ZETS";
const VERSION: u32 = 1;

#[derive(Debug)]
pub enum PersistError {
    Io(std::io::Error),
    BadMagic,
    UnsupportedVersion(u32),
    TruncatedData,
    InvalidKind(u8),
}

impl From<std::io::Error> for PersistError {
    fn from(e: std::io::Error) -> Self { PersistError::Io(e) }
}

impl std::fmt::Display for PersistError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {}", e),
            Self::BadMagic => write!(f, "not a ZETS file (bad magic)"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported version: {}", v),
            Self::TruncatedData => write!(f, "file truncated"),
            Self::InvalidKind(k) => write!(f, "invalid atom kind: {}", k),
        }
    }
}
impl std::error::Error for PersistError {}

/// Write an AtomStore to a file.
pub fn save_to_file<P: AsRef<Path>>(store: &AtomStore, path: P) -> Result<u64, PersistError> {
    let mut buf = Vec::with_capacity(1024 * 1024);
    serialize(store, &mut buf)?;
    std::fs::write(&path, &buf)?;
    Ok(buf.len() as u64)
}

/// Read an AtomStore from a file.
pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<AtomStore, PersistError> {
    let bytes = std::fs::read(&path)?;
    deserialize(&bytes)
}

/// Serialize the store to a byte buffer.
pub fn serialize<W: Write>(store: &AtomStore, writer: &mut W) -> Result<(), PersistError> {
    writer.write_all(MAGIC)?;
    writer.write_all(&VERSION.to_le_bytes())?;

    let (atoms_snapshot, edges_snapshot) = store.snapshot();

    writer.write_all(&(atoms_snapshot.len() as u64).to_le_bytes())?;
    writer.write_all(&(edges_snapshot.len() as u64).to_le_bytes())?;

    // Atoms
    for atom in atoms_snapshot {
        writer.write_all(&[atom.kind as u8])?;
        writer.write_all(&(atom.data.len() as u32).to_le_bytes())?;
        writer.write_all(&atom.data)?;
        writer.write_all(&(atom.provenance as u64).to_le_bytes())?;
    }

    // Edges
    for edge in edges_snapshot {
        writer.write_all(&edge.from.to_le_bytes())?;
        writer.write_all(&edge.to.to_le_bytes())?;
        writer.write_all(&[edge.relation])?;
        writer.write_all(&[edge.weight])?;
        writer.write_all(&edge.slot.to_le_bytes())?;
    }

    Ok(())
}

/// Deserialize into a fresh AtomStore.
pub fn deserialize(bytes: &[u8]) -> Result<AtomStore, PersistError> {
    if bytes.len() < 20 { return Err(PersistError::TruncatedData); }

    if &bytes[0..4] != MAGIC { return Err(PersistError::BadMagic); }

    let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    if version != VERSION {
        return Err(PersistError::UnsupportedVersion(version));
    }

    let atom_count = u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize;
    let edge_count = u64::from_le_bytes(bytes[16..24].try_into().unwrap()) as usize;

    let mut cursor = 24;
    let mut store = AtomStore::new();

    // Read atoms — use put_raw to preserve atom_id ordering
    for _ in 0..atom_count {
        if cursor + 5 > bytes.len() { return Err(PersistError::TruncatedData); }
        let kind_byte = bytes[cursor]; cursor += 1;
        let kind = atom_kind_from_u8(kind_byte)?;
        let len = u32::from_le_bytes(bytes[cursor..cursor+4].try_into().unwrap()) as usize;
        cursor += 4;
        if cursor + len + 8 > bytes.len() { return Err(PersistError::TruncatedData); }
        let data = bytes[cursor..cursor+len].to_vec();
        cursor += len;
        let _prov = u64::from_le_bytes(bytes[cursor..cursor+8].try_into().unwrap());
        cursor += 8;
        store.put(kind, data);
    }

    // Read edges — reconstruct via store.link (rebuilds outgoing_index)
    for _ in 0..edge_count {
        if cursor + 10 > bytes.len() { return Err(PersistError::TruncatedData); }
        let from = u32::from_le_bytes(bytes[cursor..cursor+4].try_into().unwrap());
        cursor += 4;
        let to = u32::from_le_bytes(bytes[cursor..cursor+4].try_into().unwrap());
        cursor += 4;
        let relation = bytes[cursor]; cursor += 1;
        let weight = bytes[cursor]; cursor += 1;
        let slot = u16::from_le_bytes(bytes[cursor..cursor+2].try_into().unwrap());
        cursor += 2;
        store.link(from, to, relation, weight, slot);
    }

    Ok(store)
}

fn atom_kind_from_u8(b: u8) -> Result<AtomKind, PersistError> {
    // Must match the numeric discriminants in atoms.rs AtomKind enum (0..=8).
    match b {
        0 => Ok(AtomKind::Concept),
        1 => Ok(AtomKind::Text),
        2 => Ok(AtomKind::ImageFrame),
        3 => Ok(AtomKind::AudioChunk),
        4 => Ok(AtomKind::PoseVector),
        5 => Ok(AtomKind::Template),
        6 => Ok(AtomKind::Delta),
        7 => Ok(AtomKind::Composition),
        8 => Ok(AtomKind::Relation),
        _ => Err(PersistError::InvalidKind(b)),
    }
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::{AtomKind, AtomStore};
    use crate::relations;

    fn build_sample() -> AtomStore {
        let mut store = AtomStore::new();
        let a = store.put(AtomKind::Concept, b"alpha".to_vec());
        let b = store.put(AtomKind::Concept, b"beta".to_vec());
        let c = store.put(AtomKind::Concept, b"gamma".to_vec());
        let is_a = relations::by_name("is_a").unwrap().code;
        store.link(a, b, is_a, 85, 0);
        store.link(b, c, is_a, 90, 1);
        store.link(a, c, is_a, 50, 0);
        store
    }

    #[test]
    fn round_trip_preserves_atoms() {
        let src = build_sample();
        let mut buf = Vec::new();
        serialize(&src, &mut buf).unwrap();
        let restored = deserialize(&buf).unwrap();

        let (src_atoms, _) = src.snapshot();
        let (dst_atoms, _) = restored.snapshot();
        assert_eq!(src_atoms.len(), dst_atoms.len());
        for (a, b) in src_atoms.iter().zip(dst_atoms.iter()) {
            assert_eq!(a.kind, b.kind);
            assert_eq!(a.data, b.data);
        }
    }

    #[test]
    fn round_trip_preserves_edges() {
        let src = build_sample();
        let mut buf = Vec::new();
        serialize(&src, &mut buf).unwrap();
        let restored = deserialize(&buf).unwrap();

        let (_, src_edges) = src.snapshot();
        let (_, dst_edges) = restored.snapshot();
        assert_eq!(src_edges.len(), dst_edges.len());
        for (a, b) in src_edges.iter().zip(dst_edges.iter()) {
            assert_eq!(a.from, b.from);
            assert_eq!(a.to, b.to);
            assert_eq!(a.relation, b.relation);
            assert_eq!(a.weight, b.weight);
            assert_eq!(a.slot, b.slot);
        }
    }

    #[test]
    fn file_round_trip() {
        let src = build_sample();
        let tmp = std::env::temp_dir().join("zets_test_roundtrip.bin");
        let written = save_to_file(&src, &tmp).unwrap();
        assert!(written > 20);
        let loaded = load_from_file(&tmp).unwrap();
        let (src_atoms, _) = src.snapshot();
        let (dst_atoms, _) = loaded.snapshot();
        assert_eq!(src_atoms.len(), dst_atoms.len());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn bad_magic_rejected() {
        let bytes = vec![0u8; 50];
        let result = deserialize(&bytes);
        assert!(matches!(result, Err(PersistError::BadMagic)));
    }

    #[test]
    fn truncated_rejected() {
        let bytes = vec![0u8; 10];
        let result = deserialize(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn unsupported_version_rejected() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&999u32.to_le_bytes());
        bytes.extend_from_slice(&0u64.to_le_bytes());
        bytes.extend_from_slice(&0u64.to_le_bytes());
        let result = deserialize(&bytes);
        assert!(matches!(result, Err(PersistError::UnsupportedVersion(999))));
    }

    #[test]
    fn empty_store_round_trip() {
        let store = AtomStore::new();
        let mut buf = Vec::new();
        serialize(&store, &mut buf).unwrap();
        let restored = deserialize(&buf).unwrap();
        let (atoms, edges) = restored.snapshot();
        assert_eq!(atoms.len(), 0);
        assert_eq!(edges.len(), 0);
    }
}
