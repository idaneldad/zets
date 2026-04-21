//! Binary pack format for PieceGraph.
//!
//! Two-file architecture:
//!   zets.core   — universal graph (always loaded, small)
//!   zets.<lang> — per-language data (lazy-loaded on demand)
//!
//! Format is custom binary, little-endian, no external deps.
//! Designed for:
//!   - Fast random access (u32 offsets → O(1))
//!   - Lazy language loading (core + selected langs only)
//!   - Forward-compatibility (versioned)
//!
//! Compression: strings stored in length-prefixed blocks. Can add zstd later.

use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::piece_graph::{
    ConceptNode, LangIndex, LangRegistry, PackedEdge, PieceGraph, PiecePool,
};

/// Magic bytes for core file
pub const CORE_MAGIC: &[u8; 4] = b"ZETS";
pub const LANG_MAGIC: &[u8; 4] = b"ZTLG";
pub const FORMAT_VERSION: u32 = 1;

// ============================================================================
// WRITER — serialize PieceGraph to binary packs
// ============================================================================

pub struct PackWriter;

impl PackWriter {
    /// Write the core pack (piece pool, concept graph, language registry).
    pub fn write_core<P: AsRef<Path>>(graph: &PieceGraph, path: P) -> io::Result<u64> {
        let file = File::create(path)?;
        let mut w = BufWriter::new(file);

        // Header
        w.write_all(CORE_MAGIC)?;
        w.write_all(&FORMAT_VERSION.to_le_bytes())?;
        // Flags: bit 0 = encrypted (future)
        w.write_all(&0u32.to_le_bytes())?;
        // Reserve 32 bytes for future use
        w.write_all(&[0u8; 32])?;

        // Section offsets table (we'll fill this in after writing sections)
        let offsets_pos = w.stream_position()?;
        // 5 sections × (offset:u64, length:u64) = 80 bytes reserved
        w.write_all(&[0u8; 80])?;

        // ── Section 1: Language Registry ──
        let lang_offset = w.stream_position()?;
        Self::write_lang_registry(&mut w, &graph.langs)?;
        let lang_length = w.stream_position()? - lang_offset;

        // ── Section 2: Piece Pool ──
        let pieces_offset = w.stream_position()?;
        Self::write_piece_pool(&mut w, &graph.pieces)?;
        let pieces_length = w.stream_position()? - pieces_offset;

        // ── Section 3: Concepts ──
        let concepts_offset = w.stream_position()?;
        Self::write_concepts(&mut w, &graph.concepts)?;
        let concepts_length = w.stream_position()? - concepts_offset;

        // Sections 4+5 reserved for future (edges table, metadata)
        let reserved4_offset = w.stream_position()?;
        let reserved4_length = 0u64;
        let reserved5_offset = w.stream_position()?;
        let reserved5_length = 0u64;

        // Go back and fill in the offsets table
        let end_pos = w.stream_position()?;
        w.seek(SeekFrom::Start(offsets_pos))?;
        w.write_all(&lang_offset.to_le_bytes())?;
        w.write_all(&lang_length.to_le_bytes())?;
        w.write_all(&pieces_offset.to_le_bytes())?;
        w.write_all(&pieces_length.to_le_bytes())?;
        w.write_all(&concepts_offset.to_le_bytes())?;
        w.write_all(&concepts_length.to_le_bytes())?;
        w.write_all(&reserved4_offset.to_le_bytes())?;
        w.write_all(&reserved4_length.to_le_bytes())?;
        w.write_all(&reserved5_offset.to_le_bytes())?;
        w.write_all(&reserved5_length.to_le_bytes())?;
        w.seek(SeekFrom::Start(end_pos))?;

        w.flush()?;
        Ok(end_pos)
    }

    /// Write one language pack (lang-specific surface_to_concepts, synonyms, etc).
    pub fn write_lang<P: AsRef<Path>>(
        lang_code: &str,
        idx: &LangIndex,
        path: P,
    ) -> io::Result<u64> {
        let file = File::create(path)?;
        let mut w = BufWriter::new(file);

        w.write_all(LANG_MAGIC)?;
        w.write_all(&FORMAT_VERSION.to_le_bytes())?;
        w.write_all(&0u32.to_le_bytes())?; // flags
        // Lang code as fixed-6-bytes ASCII
        let mut code_buf = [0u8; 6];
        for (i, b) in lang_code.bytes().take(6).enumerate() {
            code_buf[i] = b;
        }
        w.write_all(&code_buf)?;
        // Lang id (for verification against core)
        w.write_all(&[idx.lang_id])?;
        // Reserve 16 bytes
        w.write_all(&[0u8; 16])?;

        // Section offsets (5 sections × 16 bytes)
        let offsets_pos = w.stream_position()?;
        w.write_all(&[0u8; 80])?;

        // ── Section 1: surface_to_concepts ──
        let s2c_offset = w.stream_position()?;
        Self::write_map_u32_to_vec_u32(&mut w, &idx.surface_to_concepts)?;
        let s2c_length = w.stream_position()? - s2c_offset;

        // ── Section 2: surface_pos (u32 → u8) ──
        let pos_offset = w.stream_position()?;
        Self::write_map_u32_to_u8(&mut w, &idx.surface_pos)?;
        let pos_length = w.stream_position()? - pos_offset;

        // ── Section 3: synonyms ──
        let syn_offset = w.stream_position()?;
        Self::write_map_u32_to_vec_u32(&mut w, &idx.synonyms)?;
        let syn_length = w.stream_position()? - syn_offset;

        // ── Section 4: antonyms ──
        let ant_offset = w.stream_position()?;
        Self::write_map_u32_to_vec_u32(&mut w, &idx.antonyms)?;
        let ant_length = w.stream_position()? - ant_offset;

        // ── Section 5: definitions ──
        let def_offset = w.stream_position()?;
        Self::write_map_u32_to_vec_u32(&mut w, &idx.definitions)?;
        let def_length = w.stream_position()? - def_offset;

        let end_pos = w.stream_position()?;
        w.seek(SeekFrom::Start(offsets_pos))?;
        for (off, len) in [
            (s2c_offset, s2c_length),
            (pos_offset, pos_length),
            (syn_offset, syn_length),
            (ant_offset, ant_length),
            (def_offset, def_length),
        ] {
            w.write_all(&off.to_le_bytes())?;
            w.write_all(&len.to_le_bytes())?;
        }
        w.seek(SeekFrom::Start(end_pos))?;
        w.flush()?;
        Ok(end_pos)
    }

    fn write_lang_registry<W: Write>(w: &mut W, langs: &LangRegistry) -> io::Result<()> {
        w.write_all(&(langs.len() as u16).to_le_bytes())?;
        for code in langs.all() {
            let mut buf = [0u8; 6];
            for (i, b) in code.bytes().take(6).enumerate() {
                buf[i] = b;
            }
            w.write_all(&buf)?;
        }
        Ok(())
    }

    fn write_piece_pool<W: Write>(w: &mut W, pool: &PiecePool) -> io::Result<()> {
        w.write_all(&(pool.len() as u32).to_le_bytes())?;
        for i in 0..pool.len() {
            let s = pool.get(i as u32);
            let bytes = s.as_bytes();
            let len = bytes.len().min(65535) as u16;
            w.write_all(&len.to_le_bytes())?;
            w.write_all(&bytes[..len as usize])?;
        }
        Ok(())
    }

    fn write_concepts<W: Write>(w: &mut W, concepts: &[ConceptNode]) -> io::Result<()> {
        w.write_all(&(concepts.len() as u32).to_le_bytes())?;
        for c in concepts {
            // id:u32, anchor:u32, gloss:u32, pos:u8, edge_count:u32, edges[...]
            w.write_all(&c.id.to_le_bytes())?;
            w.write_all(&c.anchor_piece.to_le_bytes())?;
            w.write_all(&c.gloss_piece.to_le_bytes())?;
            w.write_all(&[c.pos])?;
            w.write_all(&(c.edges.len() as u32).to_le_bytes())?;
            for e in &c.edges {
                w.write_all(&[e.kind])?;
                w.write_all(&e.target.to_le_bytes())?;
            }
        }
        Ok(())
    }

    fn write_map_u32_to_vec_u32<W: Write>(
        w: &mut W,
        m: &std::collections::HashMap<u32, Vec<u32>>,
    ) -> io::Result<()> {
        w.write_all(&(m.len() as u32).to_le_bytes())?;
        for (k, v) in m {
            w.write_all(&k.to_le_bytes())?;
            w.write_all(&(v.len() as u32).to_le_bytes())?;
            for item in v {
                w.write_all(&item.to_le_bytes())?;
            }
        }
        Ok(())
    }

    fn write_map_u32_to_u8<W: Write>(
        w: &mut W,
        m: &std::collections::HashMap<u32, u8>,
    ) -> io::Result<()> {
        w.write_all(&(m.len() as u32).to_le_bytes())?;
        for (k, v) in m {
            w.write_all(&k.to_le_bytes())?;
            w.write_all(&[*v])?;
        }
        Ok(())
    }
}

// ============================================================================
// READER — loads binary packs back into PieceGraph
// ============================================================================

pub struct PackReader;

impl PackReader {
    /// Read the core file and populate a new PieceGraph with
    /// language registry, piece pool, and concepts (no per-lang data).
    pub fn read_core<P: AsRef<Path>>(path: P) -> io::Result<PieceGraph> {
        let file = File::open(path)?;
        let mut r = BufReader::new(file);

        // Verify header
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;
        if &magic != CORE_MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "bad core magic"));
        }
        let mut version = [0u8; 4];
        r.read_exact(&mut version)?;
        let _version = u32::from_le_bytes(version);
        let mut flags = [0u8; 4];
        r.read_exact(&mut flags)?;
        let mut reserved = [0u8; 32];
        r.read_exact(&mut reserved)?;

        // Read offsets table
        let mut sections = [(0u64, 0u64); 5];
        for s in &mut sections {
            let mut buf = [0u8; 8];
            r.read_exact(&mut buf)?;
            s.0 = u64::from_le_bytes(buf);
            r.read_exact(&mut buf)?;
            s.1 = u64::from_le_bytes(buf);
        }

        let mut graph = PieceGraph::new();

        // ── Read Language Registry ──
        r.seek(SeekFrom::Start(sections[0].0))?;
        Self::read_lang_registry(&mut r, &mut graph.langs)?;

        // ── Read Piece Pool ──
        r.seek(SeekFrom::Start(sections[1].0))?;
        Self::read_piece_pool(&mut r, &mut graph.pieces)?;

        // ── Read Concepts ──
        r.seek(SeekFrom::Start(sections[2].0))?;
        Self::read_concepts(&mut r, &mut graph.concepts)?;

        Ok(graph)
    }

    /// Read one language pack and install it into an existing graph.
    pub fn read_lang<P: AsRef<Path>>(graph: &mut PieceGraph, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let mut r = BufReader::new(file);

        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;
        if &magic != LANG_MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "bad lang magic"));
        }
        let mut version = [0u8; 4];
        r.read_exact(&mut version)?;
        let mut flags = [0u8; 4];
        r.read_exact(&mut flags)?;
        let mut code_buf = [0u8; 6];
        r.read_exact(&mut code_buf)?;
        let code_str = String::from_utf8_lossy(&code_buf)
            .trim_end_matches('\0')
            .to_string();
        let mut lang_id_buf = [0u8; 1];
        r.read_exact(&mut lang_id_buf)?;
        let _ = lang_id_buf[0];
        let mut reserved = [0u8; 16];
        r.read_exact(&mut reserved)?;

        // Read sections table
        let mut sections = [(0u64, 0u64); 5];
        for s in &mut sections {
            let mut buf = [0u8; 8];
            r.read_exact(&mut buf)?;
            s.0 = u64::from_le_bytes(buf);
            r.read_exact(&mut buf)?;
            s.1 = u64::from_le_bytes(buf);
        }

        // Find the lang_id from registry
        let lang_id = graph
            .langs
            .get(&code_str)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "lang not in registry"))?;
        let mut idx = LangIndex::new(lang_id);

        // Read all 5 sections
        r.seek(SeekFrom::Start(sections[0].0))?;
        idx.surface_to_concepts = Self::read_map_u32_to_vec_u32(&mut r)?;

        r.seek(SeekFrom::Start(sections[1].0))?;
        idx.surface_pos = Self::read_map_u32_to_u8(&mut r)?;

        r.seek(SeekFrom::Start(sections[2].0))?;
        idx.synonyms = Self::read_map_u32_to_vec_u32(&mut r)?;

        r.seek(SeekFrom::Start(sections[3].0))?;
        idx.antonyms = Self::read_map_u32_to_vec_u32(&mut r)?;

        r.seek(SeekFrom::Start(sections[4].0))?;
        idx.definitions = Self::read_map_u32_to_vec_u32(&mut r)?;

        graph.lang_indexes.insert(lang_id, idx);
        Ok(())
    }

    fn read_lang_registry<R: Read>(r: &mut R, langs: &mut LangRegistry) -> io::Result<()> {
        let mut buf = [0u8; 2];
        r.read_exact(&mut buf)?;
        let n = u16::from_le_bytes(buf) as usize;
        for _ in 0..n {
            let mut code_buf = [0u8; 6];
            r.read_exact(&mut code_buf)?;
            let code = String::from_utf8_lossy(&code_buf)
                .trim_end_matches('\0')
                .to_string();
            langs.intern(&code);
        }
        Ok(())
    }

    fn read_piece_pool<R: Read>(r: &mut R, pool: &mut PiecePool) -> io::Result<()> {
        let mut buf = [0u8; 4];
        r.read_exact(&mut buf)?;
        let n = u32::from_le_bytes(buf) as usize;
        // Start fresh (the "" was already interned by PiecePool::new())
        *pool = PiecePool::new();
        // We need to re-intern — but the first "" is already there.
        // Skip the first string (it's empty) and re-add the rest.
        for i in 0..n {
            let mut len_buf = [0u8; 2];
            r.read_exact(&mut len_buf)?;
            let len = u16::from_le_bytes(len_buf) as usize;
            let mut str_buf = vec![0u8; len];
            r.read_exact(&mut str_buf)?;
            if i == 0 {
                // should be ""
                continue;
            }
            let s = String::from_utf8_lossy(&str_buf).to_string();
            pool.intern(&s);
        }
        Ok(())
    }

    fn read_concepts<R: Read>(r: &mut R, concepts: &mut Vec<ConceptNode>) -> io::Result<()> {
        let mut buf = [0u8; 4];
        r.read_exact(&mut buf)?;
        let n = u32::from_le_bytes(buf) as usize;
        concepts.reserve(n);
        for _ in 0..n {
            r.read_exact(&mut buf)?;
            let id = u32::from_le_bytes(buf);
            r.read_exact(&mut buf)?;
            let anchor = u32::from_le_bytes(buf);
            r.read_exact(&mut buf)?;
            let gloss = u32::from_le_bytes(buf);
            let mut pos_buf = [0u8; 1];
            r.read_exact(&mut pos_buf)?;
            let pos = pos_buf[0];
            r.read_exact(&mut buf)?;
            let edge_count = u32::from_le_bytes(buf) as usize;

            let mut edges = Vec::with_capacity(edge_count);
            for _ in 0..edge_count {
                let mut kind_buf = [0u8; 1];
                r.read_exact(&mut kind_buf)?;
                r.read_exact(&mut buf)?;
                let target = u32::from_le_bytes(buf);
                edges.push(PackedEdge {
                    kind: kind_buf[0],
                    target,
                });
            }

            // Ensure concepts grows to this id
            while concepts.len() <= id as usize {
                concepts.push(ConceptNode {
                    id: concepts.len() as u32,
                    anchor_piece: 0,
                    gloss_piece: 0,
                    pos: 0,
                    edges: Vec::new(),
                });
            }
            concepts[id as usize] = ConceptNode {
                id,
                anchor_piece: anchor,
                gloss_piece: gloss,
                pos,
                edges,
            };
        }
        Ok(())
    }

    fn read_map_u32_to_vec_u32<R: Read>(
        r: &mut R,
    ) -> io::Result<std::collections::HashMap<u32, Vec<u32>>> {
        let mut map = std::collections::HashMap::new();
        let mut buf = [0u8; 4];
        r.read_exact(&mut buf)?;
        let n = u32::from_le_bytes(buf) as usize;
        for _ in 0..n {
            r.read_exact(&mut buf)?;
            let key = u32::from_le_bytes(buf);
            r.read_exact(&mut buf)?;
            let vlen = u32::from_le_bytes(buf) as usize;
            let mut values = Vec::with_capacity(vlen);
            for _ in 0..vlen {
                r.read_exact(&mut buf)?;
                values.push(u32::from_le_bytes(buf));
            }
            map.insert(key, values);
        }
        Ok(map)
    }

    fn read_map_u32_to_u8<R: Read>(
        r: &mut R,
    ) -> io::Result<std::collections::HashMap<u32, u8>> {
        let mut map = std::collections::HashMap::new();
        let mut buf = [0u8; 4];
        r.read_exact(&mut buf)?;
        let n = u32::from_le_bytes(buf) as usize;
        for _ in 0..n {
            r.read_exact(&mut buf)?;
            let key = u32::from_le_bytes(buf);
            let mut vbuf = [0u8; 1];
            r.read_exact(&mut vbuf)?;
            map.insert(key, vbuf[0]);
        }
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn round_trip_empty_core() {
        let graph = PieceGraph::new();
        let tmp = std::env::temp_dir().join("zets_test_empty_core.bin");
        PackWriter::write_core(&graph, &tmp).unwrap();
        let read_back = PackReader::read_core(&tmp).unwrap();
        assert_eq!(read_back.langs.len(), 0);
        assert_eq!(read_back.concepts.len(), 0);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn round_trip_small_graph() {
        let mut graph = PieceGraph::new();
        let _ = graph.langs.intern("en");
        let dog_piece = graph.pieces.intern("dog");
        graph.concepts.push(ConceptNode {
            id: 0,
            anchor_piece: dog_piece,
            gloss_piece: 0,
            pos: crate::piece_graph::POS_NOUN,
            edges: vec![PackedEdge {
                kind: crate::piece_graph::EdgeKind::Synonym.as_u8(),
                target: 1,
            }],
        });

        let tmp: PathBuf = std::env::temp_dir().join("zets_test_small.bin");
        PackWriter::write_core(&graph, &tmp).unwrap();
        let read_back = PackReader::read_core(&tmp).unwrap();
        assert_eq!(read_back.langs.len(), 1);
        assert_eq!(read_back.concepts.len(), 1);
        assert_eq!(read_back.pieces.get(dog_piece), "dog");
        assert_eq!(read_back.concepts[0].edges.len(), 1);
        let _ = std::fs::remove_file(&tmp);
    }
}
