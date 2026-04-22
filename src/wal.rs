//! Write-Ahead Log — append-only log of learned updates.
//!
//! Why: the main zets.core pack is read-only and should not be rewritten for
//! every small learning step. Instead, when SelfLearner discovers "X is noun"
//! or word-order rules, we append a record to a WAL file. On startup, we replay
//! the WAL over the mmap'd graph in memory (no disk writes to the pack).
//!
//! File layout:
//!   magic(4): "ZWAL"
//!   version(u32)
//!   [records...]
//!
//! Record format (variable length):
//!   kind (u8)       — RecordKind
//!   timestamp (u64) — UNIX ms
//!   payload_len (u16)
//!   payload (bytes) — kind-specific
//!   checksum (u16)  — sum of (kind + timestamp bytes + payload) mod 65536
//!
//! All records are append-only. Delete is represented as a DeletePos / DeleteEdge
//! record rather than physical removal.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const WAL_MAGIC: &[u8; 4] = b"ZWAL";
pub const WAL_VERSION: u32 = 1;

/// The kind of record written to the WAL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecordKind {
    /// Learner asserted a POS for a surface: lang_id u8, piece_id u32, pos u8.
    LearnPos = 1,
    /// Learner discovered a word-order rule: lang_id u8, rule u8, confidence u16.
    /// rule: 0=undetermined, 1=adj_first, 2=noun_first
    LearnOrder = 2,
    /// User annotation: concept_id u32, field u8, payload (variable).
    UserNote = 3,
    /// Delete a learned POS: lang_id u8, piece_id u32.
    DeletePos = 4,
    /// Unknown/reserved.
    Unknown = 255,
}

impl RecordKind {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::LearnPos,
            2 => Self::LearnOrder,
            3 => Self::UserNote,
            4 => Self::DeletePos,
            _ => Self::Unknown,
        }
    }
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// A single WAL record.
#[derive(Debug, Clone)]
pub struct Record {
    pub kind: RecordKind,
    pub timestamp_ms: u64,
    pub payload: Vec<u8>,
}

impl Record {
    pub fn new(kind: RecordKind, payload: Vec<u8>) -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Self {
            kind,
            timestamp_ms: ts,
            payload,
        }
    }

    /// Build a LearnPos record.
    pub fn learn_pos(lang_id: u8, piece_id: u32, pos: u8) -> Self {
        let mut p = Vec::with_capacity(6);
        p.push(lang_id);
        p.extend_from_slice(&piece_id.to_le_bytes());
        p.push(pos);
        Self::new(RecordKind::LearnPos, p)
    }

    /// Build a LearnOrder record.
    pub fn learn_order(lang_id: u8, rule: u8, confidence: u16) -> Self {
        let mut p = Vec::with_capacity(4);
        p.push(lang_id);
        p.push(rule);
        p.extend_from_slice(&confidence.to_le_bytes());
        Self::new(RecordKind::LearnOrder, p)
    }

    pub fn delete_pos(lang_id: u8, piece_id: u32) -> Self {
        let mut p = Vec::with_capacity(5);
        p.push(lang_id);
        p.extend_from_slice(&piece_id.to_le_bytes());
        Self::new(RecordKind::DeletePos, p)
    }

    /// Parse a LearnPos payload back into (lang_id, piece_id, pos).
    pub fn as_learn_pos(&self) -> Option<(u8, u32, u8)> {
        if self.kind != RecordKind::LearnPos || self.payload.len() != 6 {
            return None;
        }
        let lang = self.payload[0];
        let pid = u32::from_le_bytes(self.payload[1..5].try_into().ok()?);
        let pos = self.payload[5];
        Some((lang, pid, pos))
    }

    pub fn as_learn_order(&self) -> Option<(u8, u8, u16)> {
        if self.kind != RecordKind::LearnOrder || self.payload.len() != 4 {
            return None;
        }
        let lang = self.payload[0];
        let rule = self.payload[1];
        let conf = u16::from_le_bytes(self.payload[2..4].try_into().ok()?);
        Some((lang, rule, conf))
    }

    pub fn as_delete_pos(&self) -> Option<(u8, u32)> {
        if self.kind != RecordKind::DeletePos || self.payload.len() != 5 {
            return None;
        }
        let lang = self.payload[0];
        let pid = u32::from_le_bytes(self.payload[1..5].try_into().ok()?);
        Some((lang, pid))
    }

    /// Simple checksum: sum of all bytes mod 65536.
    fn checksum(&self) -> u16 {
        let mut sum: u32 = self.kind.as_u8() as u32;
        for b in self.timestamp_ms.to_le_bytes() {
            sum = sum.wrapping_add(b as u32);
        }
        for b in &self.payload {
            sum = sum.wrapping_add(*b as u32);
        }
        (sum & 0xFFFF) as u16
    }
}

/// Append-only WAL writer.
pub struct WalWriter {
    file: File,
}

impl WalWriter {
    /// Open (or create) a WAL file for appending.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let existed = path.as_ref().exists();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&path)?;

        let mut w = Self { file };
        if !existed {
            w.write_header()?;
        } else {
            w.verify_header()?;
        }
        Ok(w)
    }

    fn write_header(&mut self) -> io::Result<()> {
        self.file.write_all(WAL_MAGIC)?;
        self.file.write_all(&WAL_VERSION.to_le_bytes())?;
        Ok(())
    }

    fn verify_header(&mut self) -> io::Result<()> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut magic = [0u8; 4];
        self.file.read_exact(&mut magic)?;
        if &magic != WAL_MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "bad WAL magic"));
        }
        let mut ver = [0u8; 4];
        self.file.read_exact(&mut ver)?;
        self.file.seek(SeekFrom::End(0))?;
        Ok(())
    }

    /// Append a record. Format: kind(1) + ts(8) + len(2) + payload + checksum(2).
    pub fn append(&mut self, r: &Record) -> io::Result<()> {
        let checksum = r.checksum();
        self.file.write_all(&[r.kind.as_u8()])?;
        self.file.write_all(&r.timestamp_ms.to_le_bytes())?;
        self.file
            .write_all(&(r.payload.len() as u16).to_le_bytes())?;
        self.file.write_all(&r.payload)?;
        self.file.write_all(&checksum.to_le_bytes())?;
        // For safety we don't fsync after every append — we sync on close.
        Ok(())
    }

    pub fn sync(&mut self) -> io::Result<()> {
        self.file.sync_all()
    }

    pub fn file_size(&self) -> io::Result<u64> {
        Ok(self.file.metadata()?.len())
    }
}

/// Sequential reader — replays a WAL from start.
pub struct WalReader {
    file: File,
}

impl WalReader {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if &magic != WAL_MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "bad WAL magic"));
        }
        let mut ver = [0u8; 4];
        file.read_exact(&mut ver)?;
        Ok(Self { file })
    }

    /// Read the next record, or Ok(None) at EOF or torn-write boundary.
    ///
    /// CRASH SAFETY: If a record is incomplete (power loss mid-write or
    /// checksum mismatch), we return Ok(None) rather than error. This is
    /// standard WAL semantics — all records up to the first torn one are
    /// valid; everything after is discarded as if the crash never happened.
    /// Caller can re-truncate the file to drop the torn suffix if desired.
    pub fn next_record(&mut self) -> io::Result<Option<Record>> {
        // Try to read 1 byte — 0 means clean EOF.
        let mut kind_buf = [0u8; 1];
        match self.file.read(&mut kind_buf)? {
            0 => return Ok(None),
            1 => {}
            _ => unreachable!(),
        }

        // Helper: try to read exact; torn write returns Ok(None)
        let mut ts_buf = [0u8; 8];
        if !read_or_eof(&mut self.file, &mut ts_buf)? {
            return Ok(None);
        }
        let mut len_buf = [0u8; 2];
        if !read_or_eof(&mut self.file, &mut len_buf)? {
            return Ok(None);
        }
        let plen = u16::from_le_bytes(len_buf) as usize;
        // Sanity: payload length shouldn't be absurd
        if plen > 1 << 20 {
            // Torn or corrupt — treat as EOF
            return Ok(None);
        }
        let mut payload = vec![0u8; plen];
        if !read_or_eof(&mut self.file, &mut payload)? {
            return Ok(None);
        }
        let mut cs_buf = [0u8; 2];
        if !read_or_eof(&mut self.file, &mut cs_buf)? {
            return Ok(None);
        }
        let cs_read = u16::from_le_bytes(cs_buf);

        let r = Record {
            kind: RecordKind::from_u8(kind_buf[0]),
            timestamp_ms: u64::from_le_bytes(ts_buf),
            payload,
        };
        // Checksum mismatch = torn write or bit flip. Skip rest of file.
        if r.checksum() != cs_read {
            return Ok(None);
        }
        Ok(Some(r))
    }

    /// Read all records into a Vec.
    pub fn read_all(&mut self) -> io::Result<Vec<Record>> {
        let mut out = Vec::new();
        while let Some(r) = self.next_record()? {
            out.push(r);
        }
        Ok(out)
    }
}

/// Read exactly `buf.len()` bytes. Returns Ok(true) on success, Ok(false) on EOF
/// or partial read (torn write). Never returns an error for short reads.
fn read_or_eof(f: &mut File, buf: &mut [u8]) -> io::Result<bool> {
    let mut total = 0;
    while total < buf.len() {
        match f.read(&mut buf[total..])? {
            0 => return Ok(false), // EOF / torn write
            n => total += n,
        }
    }
    Ok(true)
}

/// Convenience: default WAL location for a given lang code.
pub fn wal_path_for_lang<P: AsRef<Path>>(packs_dir: P, lang: &str) -> PathBuf {
    packs_dir.as_ref().join(format!("zets.{}.wal", lang))
}

/// Convenience: default WAL location for core updates.
pub fn wal_path_for_core<P: AsRef<Path>>(packs_dir: P) -> PathBuf {
    packs_dir.as_ref().join("zets.core.wal")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_learn_pos_roundtrip() {
        let r = Record::learn_pos(3, 12345, 1);
        let (lang, pid, pos) = r.as_learn_pos().expect("parse");
        assert_eq!(lang, 3);
        assert_eq!(pid, 12345);
        assert_eq!(pos, 1);
    }

    #[test]
    fn record_learn_order_roundtrip() {
        let r = Record::learn_order(5, 1, 95);
        let (lang, rule, conf) = r.as_learn_order().expect("parse");
        assert_eq!(lang, 5);
        assert_eq!(rule, 1);
        assert_eq!(conf, 95);
    }

    #[test]
    fn wal_write_and_read() {
        let dir = std::env::temp_dir().join(format!("zets_wal_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.wal");

        {
            let mut w = WalWriter::open(&path).unwrap();
            w.append(&Record::learn_pos(0, 100, 1)).unwrap();
            w.append(&Record::learn_pos(0, 200, 2)).unwrap();
            w.append(&Record::learn_order(0, 1, 90)).unwrap();
            w.append(&Record::delete_pos(0, 100)).unwrap();
            w.sync().unwrap();
        }

        let mut r = WalReader::open(&path).unwrap();
        let records = r.read_all().unwrap();
        assert_eq!(records.len(), 4);
        assert_eq!(records[0].kind, RecordKind::LearnPos);
        assert_eq!(records[1].kind, RecordKind::LearnPos);
        assert_eq!(records[2].kind, RecordKind::LearnOrder);
        assert_eq!(records[3].kind, RecordKind::DeletePos);
        assert_eq!(records[2].as_learn_order(), Some((0u8, 1u8, 90u16)));

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }


    #[test]
    fn wal_torn_write_recovery() {
        // Write 3 records, truncate mid-last, reopen — should recover first 2.
        let dir = std::env::temp_dir().join(format!("zets_wal_torn_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("torn.wal");

        {
            let mut w = WalWriter::open(&path).unwrap();
            w.append(&Record::learn_pos(0, 1, 1)).unwrap();
            w.append(&Record::learn_pos(0, 2, 2)).unwrap();
            w.append(&Record::learn_pos(0, 3, 3)).unwrap();
            w.sync().unwrap();
        }

        // Truncate last 3 bytes to simulate interrupted write
        let full = std::fs::read(&path).unwrap();
        std::fs::write(&path, &full[..full.len() - 3]).unwrap();

        let mut r = WalReader::open(&path).unwrap();
        let records = r.read_all().unwrap();
        assert_eq!(records.len(), 2, "should recover the 2 complete records");
        assert_eq!(records[0].as_learn_pos(), Some((0u8, 1u32, 1u8)));
        assert_eq!(records[1].as_learn_pos(), Some((0u8, 2u32, 2u8)));

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn wal_empty_file_after_header() {
        // Just header, no records — should read cleanly as empty.
        let dir = std::env::temp_dir().join(format!("zets_wal_empty_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("empty.wal");
        {
            let _ = WalWriter::open(&path).unwrap();
        }
        let mut r = WalReader::open(&path).unwrap();
        let records = r.read_all().unwrap();
        assert_eq!(records.len(), 0);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn wal_append_after_reopen() {
        let dir = std::env::temp_dir().join(format!("zets_wal_test2_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("append.wal");

        {
            let mut w = WalWriter::open(&path).unwrap();
            w.append(&Record::learn_pos(0, 1, 1)).unwrap();
            w.sync().unwrap();
        }

        {
            let mut w = WalWriter::open(&path).unwrap();
            w.append(&Record::learn_pos(0, 2, 2)).unwrap();
            w.sync().unwrap();
        }

        let mut r = WalReader::open(&path).unwrap();
        let records = r.read_all().unwrap();
        assert_eq!(records.len(), 2);

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }
}
