//! Opcodes — the primitive operations every system graph route is composed of.
//!
//! After AGI prep expansion: 32 opcodes covering flow, stack, data graph,
//! text/morph, lists, arithmetic, logic, persistence, metadata.
//!
//! Design: NOT Turing-complete. Bounded recursion. Deterministic.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Opcode {
    // ── Flow control ───────────────────────────────
    Noop = 0,
    Return = 1,
    CallRoute = 2,
    Jump = 3,
    IfEq = 4,
    IfNe = 5,

    // ── Stack/register ─────────────────────────────
    ConstLoad = 10,
    ParamLoad = 11,
    Store = 12,
    Load = 13,
    Dup = 14,
    Swap = 15,
    Pop = 16,

    // ── Data graph ─────────────────────────────────
    ConceptLookup = 20,
    ConceptCreate = 21,
    EdgeAdd = 22,
    EdgeTraverse = 23,

    // ── Text/morphology ────────────────────────────
    MorphAnalyze = 30,
    StringMatch = 31,
    StringSplit = 32,

    // ── List ops ───────────────────────────────────
    ListIndex = 35,
    ListLen = 36,
    ListEmpty = 37,

    // ── Arithmetic / comparison ────────────────────
    Add = 38,
    Sub = 39,
    Lt = 41,
    Gt = 42,

    // ── Logic ──────────────────────────────────────
    Not = 43,
    And = 44,
    Or = 45,

    // ── Persistence ────────────────────────────────
    WalWrite = 40,

    // ── Metadata/metacognition ─────────────────────
    ConfidenceSet = 50,
    TierCheck = 51,
}

impl Opcode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Noop),
            1 => Some(Self::Return),
            2 => Some(Self::CallRoute),
            3 => Some(Self::Jump),
            4 => Some(Self::IfEq),
            5 => Some(Self::IfNe),
            10 => Some(Self::ConstLoad),
            11 => Some(Self::ParamLoad),
            12 => Some(Self::Store),
            13 => Some(Self::Load),
            14 => Some(Self::Dup),
            15 => Some(Self::Swap),
            16 => Some(Self::Pop),
            20 => Some(Self::ConceptLookup),
            21 => Some(Self::ConceptCreate),
            22 => Some(Self::EdgeAdd),
            23 => Some(Self::EdgeTraverse),
            30 => Some(Self::MorphAnalyze),
            31 => Some(Self::StringMatch),
            32 => Some(Self::StringSplit),
            35 => Some(Self::ListIndex),
            36 => Some(Self::ListLen),
            37 => Some(Self::ListEmpty),
            38 => Some(Self::Add),
            39 => Some(Self::Sub),
            40 => Some(Self::WalWrite),
            41 => Some(Self::Lt),
            42 => Some(Self::Gt),
            43 => Some(Self::Not),
            44 => Some(Self::And),
            45 => Some(Self::Or),
            50 => Some(Self::ConfidenceSet),
            51 => Some(Self::TierCheck),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn instruction_size(self) -> usize {
        match self {
            Self::Jump | Self::IfEq | Self::IfNe | Self::CallRoute => 5,
            Self::ConstLoad | Self::ParamLoad | Self::Store | Self::Load => 3,
            _ => 1,
        }
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_opcodes_roundtrip() {
        for v in 0..=255u8 {
            if let Some(op) = Opcode::from_u8(v) {
                assert_eq!(op.as_u8(), v);
            }
        }
    }

    #[test]
    fn total_opcode_count() {
        let count = (0..=255u8).filter(|v| Opcode::from_u8(*v).is_some()).count();
        assert_eq!(count, 33, "expected 33 opcodes for AGI-ready routing");
    }

    #[test]
    fn instruction_sizes_are_correct() {
        assert_eq!(Opcode::Noop.instruction_size(), 1);
        assert_eq!(Opcode::Jump.instruction_size(), 5);
        assert_eq!(Opcode::ConstLoad.instruction_size(), 3);
        assert_eq!(Opcode::Add.instruction_size(), 1);
    }
}
