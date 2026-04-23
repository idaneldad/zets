//! graph_v4::persist — binary format לsave/load של Graph.
//!
//! Binary layout (little-endian):
//!   magic:        4 bytes = b"ZV4\0"
//!   version:      u32 = 1
//!   atom_count:   u64
//!   edge_count:   u64
//!   phrase_count: u64   (for quick stats)
//!   sent_count:   u64   (for quick stats)
//!
//!   for each atom (ordered by id):
//!       kind:       u8
//!       key_len:    u32
//!       key:        key_len bytes (utf-8)
//!       count:      u32   (phrase count, 0 for others)
//!       text_len:   u32   (0 if None)
//!       text:       text_len bytes (utf-8, only for sentences)
//!
//!   for each edge:
//!       from:       u32
//!       to:         u32
//!       relation:   u8
//!       weight:     u8
//!       pos:        u16
//!
//! Edge record = 12 bytes (packed). Atom avg = 30-80 bytes.

use super::types::{Atom, AtomKind, Edge, Graph, Relation};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

const MAGIC: &[u8; 4] = b"ZV4\0";
const VERSION: u32 = 1;

#[derive(Debug)]
pub enum PersistError {
    Io(std::io::Error),
    BadMagic,
    UnsupportedVersion(u32),
    InvalidKind(u8),
    InvalidRelation(u8),
    InvalidUtf8,
}

impl From<std::io::Error> for PersistError {
    fn from(e: std::io::Error) -> Self { PersistError::Io(e) }
}

impl std::fmt::Display for PersistError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {}", e),
            Self::BadMagic => write!(f, "not a ZV4 file"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported version: {}", v),
            Self::InvalidKind(k) => write!(f, "invalid atom kind: {}", k),
            Self::InvalidRelation(r) => write!(f, "invalid relation: {}", r),
            Self::InvalidUtf8 => write!(f, "invalid utf-8"),
        }
    }
}

impl std::error::Error for PersistError {}

pub fn save<P: AsRef<Path>>(g: &Graph, path: P) -> Result<(), PersistError> {
    let f = File::create(path)?;
    let mut w = BufWriter::new(f);

    w.write_all(MAGIC)?;
    w.write_all(&VERSION.to_le_bytes())?;
    w.write_all(&(g.atoms.len() as u64).to_le_bytes())?;
    w.write_all(&(g.edges.len() as u64).to_le_bytes())?;
    let phrase_ct = g.atoms.iter().filter(|a| a.kind == AtomKind::Phrase).count() as u64;
    let sent_ct = g.atoms.iter().filter(|a| a.kind == AtomKind::Sentence).count() as u64;
    w.write_all(&phrase_ct.to_le_bytes())?;
    w.write_all(&sent_ct.to_le_bytes())?;

    for atom in &g.atoms {
        w.write_all(&[atom.kind as u8])?;
        let key_bytes = atom.key.as_bytes();
        w.write_all(&(key_bytes.len() as u32).to_le_bytes())?;
        w.write_all(key_bytes)?;
        w.write_all(&atom.count.to_le_bytes())?;
        match &atom.text {
            Some(t) => {
                let tb = t.as_bytes();
                w.write_all(&(tb.len() as u32).to_le_bytes())?;
                w.write_all(tb)?;
            }
            None => {
                w.write_all(&0u32.to_le_bytes())?;
            }
        }
    }

    for e in &g.edges {
        w.write_all(&e.from.to_le_bytes())?;
        w.write_all(&e.to.to_le_bytes())?;
        w.write_all(&[e.relation as u8])?;
        w.write_all(&[e.weight])?;
        w.write_all(&e.pos.to_le_bytes())?;
    }

    w.flush()?;
    Ok(())
}

fn read_u8(r: &mut impl Read) -> std::io::Result<u8> {
    let mut b = [0u8; 1]; r.read_exact(&mut b)?; Ok(b[0])
}
fn read_u16(r: &mut impl Read) -> std::io::Result<u16> {
    let mut b = [0u8; 2]; r.read_exact(&mut b)?; Ok(u16::from_le_bytes(b))
}
fn read_u32(r: &mut impl Read) -> std::io::Result<u32> {
    let mut b = [0u8; 4]; r.read_exact(&mut b)?; Ok(u32::from_le_bytes(b))
}
fn read_u64(r: &mut impl Read) -> std::io::Result<u64> {
    let mut b = [0u8; 8]; r.read_exact(&mut b)?; Ok(u64::from_le_bytes(b))
}

pub fn load<P: AsRef<Path>>(path: P) -> Result<Graph, PersistError> {
    let f = File::open(path)?;
    let mut r = BufReader::new(f);

    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;
    if &magic != MAGIC { return Err(PersistError::BadMagic); }

    let version = read_u32(&mut r)?;
    if version != VERSION { return Err(PersistError::UnsupportedVersion(version)); }

    let atom_count = read_u64(&mut r)? as usize;
    let edge_count = read_u64(&mut r)? as usize;
    let _phrase_ct = read_u64(&mut r)?;
    let _sent_ct = read_u64(&mut r)?;

    let mut g = Graph::new();
    g.atoms.reserve(atom_count);

    for _ in 0..atom_count {
        let kind_byte = read_u8(&mut r)?;
        let kind = AtomKind::from_byte(kind_byte).ok_or(PersistError::InvalidKind(kind_byte))?;
        let key_len = read_u32(&mut r)? as usize;
        let mut key_buf = vec![0u8; key_len];
        r.read_exact(&mut key_buf)?;
        let key = String::from_utf8(key_buf).map_err(|_| PersistError::InvalidUtf8)?;
        let count = read_u32(&mut r)?;
        let text_len = read_u32(&mut r)? as usize;
        let text = if text_len == 0 {
            None
        } else {
            let mut tb = vec![0u8; text_len];
            r.read_exact(&mut tb)?;
            Some(String::from_utf8(tb).map_err(|_| PersistError::InvalidUtf8)?)
        };
        let id = g.atoms.len() as u32;
        g.by_key.insert((kind, key.clone()), id);
        g.atoms.push(Atom { kind, key, text, count });
    }

    g.edges.reserve(edge_count);
    for _ in 0..edge_count {
        let from = read_u32(&mut r)?;
        let to = read_u32(&mut r)?;
        let rel_byte = read_u8(&mut r)?;
        let relation = Relation::from_byte(rel_byte).ok_or(PersistError::InvalidRelation(rel_byte))?;
        let weight = read_u8(&mut r)?;
        let pos = read_u16(&mut r)?;
        g.edges.push(Edge { from, to, relation, weight, pos });
    }

    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_v4::{build_graph, BuildConfig};

    #[test]
    fn round_trip() {
        let articles = vec![
            ("Test".to_string(),
             "The heart beats. The heart is strong. The heart works.".to_string())
        ];
        let g1 = build_graph(&articles, &BuildConfig::default());
        let path = "/tmp/test_v4_persist.zv4";
        save(&g1, path).unwrap();
        let g2 = load(path).unwrap();
        assert_eq!(g1.atoms.len(), g2.atoms.len());
        assert_eq!(g1.edges.len(), g2.edges.len());
        for (a1, a2) in g1.atoms.iter().zip(g2.atoms.iter()) {
            assert_eq!(a1.kind, a2.kind);
            assert_eq!(a1.key, a2.key);
            assert_eq!(a1.text, a2.text);
            assert_eq!(a1.count, a2.count);
        }
        std::fs::remove_file(path).ok();
    }
}
