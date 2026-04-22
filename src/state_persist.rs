//! Volatile state persistence — serialize Session and MetaLearner as JSON bytes.
//!
//! Idan's requirement: conversations and meta-learned preferences must
//! survive a restart. Otherwise "autonomous learning" loses everything
//! between runs.
//!
//! We use a simple binary format instead of serde_json to avoid adding
//! the serde dependency. The format is explicit and deterministic.
//!
//! SessionContext format (LE):
//!   magic: b"ZSES" (4 bytes)
//!   version: u32 = 1
//!   current_turn: u64
//!   boost, decay_per_turn, prune_threshold: f32 (12 bytes)
//!   max_size: u32
//!   active_count: u32
//!   for each active:
//!       atom_id: u32
//!       activation: f32
//!       last_mentioned_turn: u64
//!
//! MetaLearner format:
//!   magic: b"ZMLR" (4 bytes)
//!   version: u32 = 1
//!   global.alpha: [f32; 4] = 16 bytes
//!   global.total_observations: u64
//!   context_count: u32
//!   for each context:
//!       label_len: u32
//!       label: utf8 bytes
//!       alpha: [f32; 4]
//!       total_observations: u64

use std::io::Write;
use std::path::Path;

use crate::meta_learning::{MetaLearner, ModeWeights};
use crate::session::{ActiveAtom, SessionContext};

const SES_MAGIC: &[u8; 4] = b"ZSES";
const MLR_MAGIC: &[u8; 4] = b"ZMLR";
const VERSION: u32 = 1;

#[derive(Debug)]
pub enum StateError {
    Io(std::io::Error),
    BadMagic,
    UnsupportedVersion(u32),
    Truncated,
    InvalidUtf8,
}

impl From<std::io::Error> for StateError {
    fn from(e: std::io::Error) -> Self { Self::Io(e) }
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io: {}", e),
            Self::BadMagic => write!(f, "bad magic"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported version {}", v),
            Self::Truncated => write!(f, "truncated"),
            Self::InvalidUtf8 => write!(f, "invalid utf8 label"),
        }
    }
}
impl std::error::Error for StateError {}

// ────────────────────────────────────────────────────────────────
// SessionContext
// ────────────────────────────────────────────────────────────────

pub fn session_serialize<W: Write>(s: &SessionContext, w: &mut W) -> Result<(), StateError> {
    w.write_all(SES_MAGIC)?;
    w.write_all(&VERSION.to_le_bytes())?;
    w.write_all(&s.current_turn.to_le_bytes())?;
    w.write_all(&s.boost.to_le_bytes())?;
    w.write_all(&s.decay_per_turn.to_le_bytes())?;
    w.write_all(&s.prune_threshold.to_le_bytes())?;
    w.write_all(&(s.max_size as u32).to_le_bytes())?;
    w.write_all(&(s.active.len() as u32).to_le_bytes())?;

    // Sort by atom_id for deterministic output
    let mut entries: Vec<&ActiveAtom> = s.active.values().collect();
    entries.sort_by_key(|e| e.atom_id);
    for e in entries {
        w.write_all(&e.atom_id.to_le_bytes())?;
        w.write_all(&e.activation.to_le_bytes())?;
        w.write_all(&e.last_mentioned_turn.to_le_bytes())?;
    }
    Ok(())
}

pub fn session_deserialize(bytes: &[u8]) -> Result<SessionContext, StateError> {
    if bytes.len() < 36 { return Err(StateError::Truncated); }
    if &bytes[0..4] != SES_MAGIC { return Err(StateError::BadMagic); }
    let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    if version != VERSION { return Err(StateError::UnsupportedVersion(version)); }

    let current_turn = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let boost = f32::from_le_bytes(bytes[16..20].try_into().unwrap());
    let decay = f32::from_le_bytes(bytes[20..24].try_into().unwrap());
    let prune = f32::from_le_bytes(bytes[24..28].try_into().unwrap());
    let max_size = u32::from_le_bytes(bytes[28..32].try_into().unwrap()) as usize;
    let active_count = u32::from_le_bytes(bytes[32..36].try_into().unwrap()) as usize;

    let mut s = SessionContext::with_params(boost, decay, prune, max_size);
    s.current_turn = current_turn;

    let mut cursor = 36;
    for _ in 0..active_count {
        if cursor + 16 > bytes.len() { return Err(StateError::Truncated); }
        let atom_id = u32::from_le_bytes(bytes[cursor..cursor+4].try_into().unwrap());
        cursor += 4;
        let activation = f32::from_le_bytes(bytes[cursor..cursor+4].try_into().unwrap());
        cursor += 4;
        let last_mentioned_turn = u64::from_le_bytes(bytes[cursor..cursor+8].try_into().unwrap());
        cursor += 8;
        s.active.insert(atom_id, ActiveAtom { atom_id, activation, last_mentioned_turn });
    }
    Ok(s)
}

pub fn session_save_file<P: AsRef<Path>>(s: &SessionContext, p: P) -> Result<(), StateError> {
    let mut buf = Vec::with_capacity(512);
    session_serialize(s, &mut buf)?;
    std::fs::write(p, buf)?;
    Ok(())
}

pub fn session_load_file<P: AsRef<Path>>(p: P) -> Result<SessionContext, StateError> {
    let bytes = std::fs::read(p)?;
    session_deserialize(&bytes)
}

// ────────────────────────────────────────────────────────────────
// MetaLearner
// ────────────────────────────────────────────────────────────────

fn write_mode_weights<W: Write>(w: &mut W, mw: &ModeWeights) -> Result<(), StateError> {
    for &a in &mw.alpha {
        w.write_all(&a.to_le_bytes())?;
    }
    w.write_all(&mw.total_observations.to_le_bytes())?;
    Ok(())
}

fn read_mode_weights(bytes: &[u8], cursor: &mut usize) -> Result<ModeWeights, StateError> {
    if *cursor + 24 > bytes.len() { return Err(StateError::Truncated); }
    let mut alpha = [0.0f32; 4];
    for i in 0..4 {
        alpha[i] = f32::from_le_bytes(bytes[*cursor..*cursor+4].try_into().unwrap());
        *cursor += 4;
    }
    let total_observations = u64::from_le_bytes(bytes[*cursor..*cursor+8].try_into().unwrap());
    *cursor += 8;
    Ok(ModeWeights { alpha, total_observations })
}

pub fn meta_serialize<W: Write>(ml: &MetaLearner, w: &mut W) -> Result<(), StateError> {
    w.write_all(MLR_MAGIC)?;
    w.write_all(&VERSION.to_le_bytes())?;

    // Global
    write_mode_weights(w, &ml.global)?;

    // Contexts (sorted by label for determinism)
    let mut contexts: Vec<(&String, &ModeWeights)> = ml.per_context.iter().collect();
    contexts.sort_by_key(|(k, _)| k.as_str().to_string());

    w.write_all(&(contexts.len() as u32).to_le_bytes())?;
    for (label, mw) in contexts {
        let lb = label.as_bytes();
        w.write_all(&(lb.len() as u32).to_le_bytes())?;
        w.write_all(lb)?;
        write_mode_weights(w, mw)?;
    }
    Ok(())
}

pub fn meta_deserialize(bytes: &[u8]) -> Result<MetaLearner, StateError> {
    if bytes.len() < 8 + 24 + 4 { return Err(StateError::Truncated); }
    if &bytes[0..4] != MLR_MAGIC { return Err(StateError::BadMagic); }
    let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    if version != VERSION { return Err(StateError::UnsupportedVersion(version)); }

    let mut cursor = 8;
    let global = read_mode_weights(bytes, &mut cursor)?;

    if cursor + 4 > bytes.len() { return Err(StateError::Truncated); }
    let context_count = u32::from_le_bytes(bytes[cursor..cursor+4].try_into().unwrap()) as usize;
    cursor += 4;

    let mut ml = MetaLearner { global, per_context: std::collections::HashMap::new() };
    for _ in 0..context_count {
        if cursor + 4 > bytes.len() { return Err(StateError::Truncated); }
        let label_len = u32::from_le_bytes(bytes[cursor..cursor+4].try_into().unwrap()) as usize;
        cursor += 4;
        if cursor + label_len > bytes.len() { return Err(StateError::Truncated); }
        let label = std::str::from_utf8(&bytes[cursor..cursor+label_len])
            .map_err(|_| StateError::InvalidUtf8)?.to_string();
        cursor += label_len;
        let mw = read_mode_weights(bytes, &mut cursor)?;
        ml.per_context.insert(label, mw);
    }
    Ok(ml)
}

pub fn meta_save_file<P: AsRef<Path>>(ml: &MetaLearner, p: P) -> Result<(), StateError> {
    let mut buf = Vec::with_capacity(512);
    meta_serialize(ml, &mut buf)?;
    std::fs::write(p, buf)?;
    Ok(())
}

pub fn meta_load_file<P: AsRef<Path>>(p: P) -> Result<MetaLearner, StateError> {
    let bytes = std::fs::read(p)?;
    meta_deserialize(&bytes)
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta_learning::CognitiveMode;

    #[test]
    fn session_round_trip() {
        let mut s = SessionContext::new();
        s.mention(100);
        s.advance_turn();
        s.mention(200);
        s.mention(300);

        let mut buf = Vec::new();
        session_serialize(&s, &mut buf).unwrap();
        let restored = session_deserialize(&buf).unwrap();

        assert_eq!(restored.current_turn, s.current_turn);
        assert_eq!(restored.size(), s.size());
        for (aid, entry) in &s.active {
            let r = restored.active.get(aid).unwrap();
            assert_eq!(r.activation, entry.activation);
        }
    }

    #[test]
    fn session_empty_round_trip() {
        let s = SessionContext::new();
        let mut buf = Vec::new();
        session_serialize(&s, &mut buf).unwrap();
        let restored = session_deserialize(&buf).unwrap();
        assert!(restored.is_empty());
    }

    #[test]
    fn session_bad_magic() {
        let bytes = vec![0u8; 100];
        assert!(matches!(session_deserialize(&bytes), Err(StateError::BadMagic)));
    }

    #[test]
    fn session_file_round_trip() {
        let mut s = SessionContext::new();
        s.mention(42);
        let tmp = std::env::temp_dir().join("zets_session_test.bin");
        session_save_file(&s, &tmp).unwrap();
        let restored = session_load_file(&tmp).unwrap();
        assert!(restored.is_active(42));
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn meta_round_trip() {
        let mut ml = MetaLearner::new();
        ml.record("factual", CognitiveMode::Precision, 1.0);
        ml.record("factual", CognitiveMode::Precision, 0.8);
        ml.record("creative", CognitiveMode::Divergent, 1.0);

        let mut buf = Vec::new();
        meta_serialize(&ml, &mut buf).unwrap();
        let restored = meta_deserialize(&buf).unwrap();

        assert_eq!(restored.global.alpha, ml.global.alpha);
        assert_eq!(restored.global.total_observations, ml.global.total_observations);
        assert_eq!(restored.per_context.len(), ml.per_context.len());

        let r_factual = restored.per_context.get("factual").unwrap();
        let o_factual = ml.per_context.get("factual").unwrap();
        assert_eq!(r_factual.alpha, o_factual.alpha);
    }

    #[test]
    fn meta_file_round_trip() {
        let mut ml = MetaLearner::new();
        ml.record("test", CognitiveMode::Gestalt, 1.0);
        let tmp = std::env::temp_dir().join("zets_meta_test.bin");
        meta_save_file(&ml, &tmp).unwrap();
        let restored = meta_load_file(&tmp).unwrap();
        assert!(restored.per_context.contains_key("test"));
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn meta_empty_round_trip() {
        let ml = MetaLearner::new();
        let mut buf = Vec::new();
        meta_serialize(&ml, &mut buf).unwrap();
        let restored = meta_deserialize(&buf).unwrap();
        assert_eq!(restored.per_context.len(), 0);
    }

    #[test]
    fn serialization_is_deterministic() {
        // Same state → same bytes, always (critical for checksums / audit)
        let mut ml1 = MetaLearner::new();
        let mut ml2 = MetaLearner::new();
        for ctx in ["alpha", "beta", "gamma"] {
            ml1.record(ctx, CognitiveMode::Narrative, 1.0);
            ml2.record(ctx, CognitiveMode::Narrative, 1.0);
        }
        let mut b1 = Vec::new();
        let mut b2 = Vec::new();
        meta_serialize(&ml1, &mut b1).unwrap();
        meta_serialize(&ml2, &mut b2).unwrap();
        assert_eq!(b1, b2);
    }

    #[test]
    fn meta_unsupported_version_rejected() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MLR_MAGIC);
        bytes.extend_from_slice(&999u32.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 24]);  // mode weights
        bytes.extend_from_slice(&0u32.to_le_bytes());  // count
        assert!(matches!(meta_deserialize(&bytes), Err(StateError::UnsupportedVersion(999))));
    }
}
