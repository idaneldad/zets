//! Opcodes — the 20 primitive operations that every process/method
//! in the system graph is composed of.
//!
//! Design principle: these are NOT Turing-complete. No unbounded loops,
//! no arbitrary recursion. A route must terminate in bounded steps.
//! This preserves determinism — a hallmark of ZETS.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Opcode {
    // Flow control
    Noop = 0,
    Return = 1,
    CallRoute = 2,    // invoke another route (bounded depth)
    Jump = 3,         // unconditional branch to offset
    IfEq = 4,         // branch if top-two stack values equal
    IfNe = 5,         // branch if not equal

    // Stack/register manipulation
    ConstLoad = 10,   // push a constant (from constant pool)
    ParamLoad = 11,   // push a parameter by index
    Store = 12,       // pop top, store to register N
    Load = 13,        // push from register N

    // Data graph interaction
    ConceptLookup = 20,   // pop surface, push concept_id (or 0)
    ConceptCreate = 21,   // pop gloss,anchor,pos -> push new concept_id
    EdgeAdd = 22,         // pop source,target,kind -> add edge
    EdgeTraverse = 23,    // pop concept_id,kind -> push Vec<target>

    // Text / morphology
    MorphAnalyze = 30,    // pop lang,surface -> push lemma (most confident)
    StringMatch = 31,     // pop text,pattern -> push 1/0
    StringSplit = 32,     // pop text,sep -> push Vec<String>

    // Persistence
    WalWrite = 40,        // pop record_kind,payload -> write to WAL

    // Metadata
    ConfidenceSet = 50,   // pop value -> set confidence of current result
    TierCheck = 51,       // pop tier -> check if we're allowed at this tier
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
            20 => Some(Self::ConceptLookup),
            21 => Some(Self::ConceptCreate),
            22 => Some(Self::EdgeAdd),
            23 => Some(Self::EdgeTraverse),
            30 => Some(Self::MorphAnalyze),
            31 => Some(Self::StringMatch),
            32 => Some(Self::StringSplit),
            40 => Some(Self::WalWrite),
            50 => Some(Self::ConfidenceSet),
            51 => Some(Self::TierCheck),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Number of bytes this opcode uses including inline immediate operands.
    /// Most opcodes = 1 byte. Some (jumps, calls) have u32 operands.
    pub fn instruction_size(self) -> usize {
        match self {
            Self::Jump | Self::IfEq | Self::IfNe | Self::CallRoute => 5, // 1 + u32
            Self::ConstLoad | Self::ParamLoad | Self::Store | Self::Load => 3, // 1 + u16
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
    fn opcode_roundtrip() {
        for op in [
            Opcode::Noop,
            Opcode::ConceptLookup,
            Opcode::CallRoute,
            Opcode::MorphAnalyze,
            Opcode::WalWrite,
        ] {
            assert_eq!(Opcode::from_u8(op.as_u8()), Some(op));
        }
    }

    #[test]
    fn invalid_opcode_returns_none() {
        assert!(Opcode::from_u8(255).is_none());
        assert!(Opcode::from_u8(99).is_none());
    }

    #[test]
    fn instruction_sizes_are_correct() {
        assert_eq!(Opcode::Noop.instruction_size(), 1);
        assert_eq!(Opcode::Jump.instruction_size(), 5);
        assert_eq!(Opcode::ConstLoad.instruction_size(), 3);
    }
}
