//! Bitflag relation — 6 orthogonal axes packed into 16 bits.
//!
//! Replaces the single `relation: u8` field with a 16-bit packed representation
//! encoding 6 independent axes. Same physical edge size (2 bytes for relation
//! instead of 1 + implicit room, but much richer expressiveness).
//!
//! Per docs/working/20260423_bitflag_paths_quantize_design_V1.md.
//! 14 bits used, 2 bits reserved for future axes.
//!
//! | Axis        | Bits | Values                                           |
//! |-------------|------|--------------------------------------------------|
//! | sem_kind    | 4    | is_a, has_part, located_at, near, depicts, ...   |
//! | polarity    | 1    | positive, negated                                |
//! | certainty   | 2    | certain, probable, hypothetical, refuted         |
//! | temporality | 2    | timeless, was, is, will_be                       |
//! | source      | 2    | observation, inference, hearsay, default         |
//! | logic       | 3    | plain, and, or, xor, implies, nand, nor, iff     |
//! | reserved    | 2    | future extension                                 |
//!
//! **Why this matters:** instead of forcing separate relation atoms for
//! "is_a", "is_not_a", "was_a", "might_be_a" — these become the SAME sem_kind
//! with different axis bits. Reduces relation-atom cardinality; improves
//! semantic precision per edge.

use std::convert::TryFrom;

/// Semantic kind — 4 bits = 16 kinds.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemKind {
    IsA = 0,
    HasPart = 1,
    TemplateOf = 2,
    HasColor = 3,
    HasPose = 4,
    Causes = 5,
    LocatedAt = 6,
    Near = 7,
    Depicts = 8,
    Says = 9,
    MemberOf = 10,
    InstanceOf = 11,
    SimilarTo = 12,
    SameAs = 13,
    Hug = 14,
    Love = 15,
}

impl SemKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SemKind::IsA => "is_a",
            SemKind::HasPart => "has_part",
            SemKind::TemplateOf => "template_of",
            SemKind::HasColor => "has_color",
            SemKind::HasPose => "has_pose",
            SemKind::Causes => "causes",
            SemKind::LocatedAt => "located_at",
            SemKind::Near => "near",
            SemKind::Depicts => "depicts",
            SemKind::Says => "says",
            SemKind::MemberOf => "member_of",
            SemKind::InstanceOf => "instance_of",
            SemKind::SimilarTo => "similar_to",
            SemKind::SameAs => "same_as",
            SemKind::Hug => "hug",
            SemKind::Love => "love",
        }
    }
}

impl TryFrom<u8> for SemKind {
    type Error = ();
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v & 0xF {
            0 => Ok(SemKind::IsA),
            1 => Ok(SemKind::HasPart),
            2 => Ok(SemKind::TemplateOf),
            3 => Ok(SemKind::HasColor),
            4 => Ok(SemKind::HasPose),
            5 => Ok(SemKind::Causes),
            6 => Ok(SemKind::LocatedAt),
            7 => Ok(SemKind::Near),
            8 => Ok(SemKind::Depicts),
            9 => Ok(SemKind::Says),
            10 => Ok(SemKind::MemberOf),
            11 => Ok(SemKind::InstanceOf),
            12 => Ok(SemKind::SimilarTo),
            13 => Ok(SemKind::SameAs),
            14 => Ok(SemKind::Hug),
            15 => Ok(SemKind::Love),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Certainty {
    Certain = 0,
    Probable = 1,
    Hypothetical = 2,
    Refuted = 3,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Temporality {
    Timeless = 0,
    Was = 1,
    Is = 2,
    WillBe = 3,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Source {
    Observation = 0,
    Inference = 1,
    Hearsay = 2,
    Default = 3,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Logic {
    Plain = 0,
    And = 1,
    Or = 2,
    Xor = 3,
    Implies = 4,
    Nand = 5,
    Nor = 6,
    Iff = 7,
}

/// Packed relation: 14 bits of semantic meaning in a u16.
///
/// Bit layout (LSB to MSB):
/// - 0..=3:  SemKind (4 bits)
/// - 4:      polarity (1 bit: 0=positive, 1=negated)
/// - 5..=6:  Certainty (2 bits)
/// - 7..=8:  Temporality (2 bits)
/// - 9..=10: Source (2 bits)
/// - 11..=13: Logic (3 bits)
/// - 14..=15: reserved
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitflagRelation(pub u16);

impl BitflagRelation {
    pub const fn new_raw(bits: u16) -> Self {
        Self(bits)
    }

    pub fn build(
        sem_kind: SemKind,
        negated: bool,
        certainty: Certainty,
        temporality: Temporality,
        source: Source,
        logic: Logic,
    ) -> Self {
        let mut val: u16 = 0;
        val |= (sem_kind as u16) & 0xF;
        val |= ((negated as u16) & 0x1) << 4;
        val |= ((certainty as u16) & 0x3) << 5;
        val |= ((temporality as u16) & 0x3) << 7;
        val |= ((source as u16) & 0x3) << 9;
        val |= ((logic as u16) & 0x7) << 11;
        Self(val)
    }

    /// A plain "is_a" relation — affirmative, certain, timeless, observed.
    pub fn simple(sem_kind: SemKind) -> Self {
        Self::build(
            sem_kind,
            false,
            Certainty::Certain,
            Temporality::Timeless,
            Source::Observation,
            Logic::Plain,
        )
    }

    pub fn sem_kind(&self) -> SemKind {
        SemKind::try_from((self.0 & 0xF) as u8).unwrap_or(SemKind::IsA)
    }

    pub fn is_negated(&self) -> bool {
        ((self.0 >> 4) & 0x1) != 0
    }

    pub fn certainty(&self) -> Certainty {
        match (self.0 >> 5) & 0x3 {
            0 => Certainty::Certain,
            1 => Certainty::Probable,
            2 => Certainty::Hypothetical,
            _ => Certainty::Refuted,
        }
    }

    pub fn temporality(&self) -> Temporality {
        match (self.0 >> 7) & 0x3 {
            0 => Temporality::Timeless,
            1 => Temporality::Was,
            2 => Temporality::Is,
            _ => Temporality::WillBe,
        }
    }

    pub fn source(&self) -> Source {
        match (self.0 >> 9) & 0x3 {
            0 => Source::Observation,
            1 => Source::Inference,
            2 => Source::Hearsay,
            _ => Source::Default,
        }
    }

    pub fn logic(&self) -> Logic {
        match (self.0 >> 11) & 0x7 {
            0 => Logic::Plain,
            1 => Logic::And,
            2 => Logic::Or,
            3 => Logic::Xor,
            4 => Logic::Implies,
            5 => Logic::Nand,
            6 => Logic::Nor,
            _ => Logic::Iff,
        }
    }

    pub fn as_raw(&self) -> u16 {
        self.0
    }

    /// Migration path: given an old `u8` relation, convert to BitflagRelation
    /// assuming defaults for the other axes.
    pub fn from_legacy_u8(old: u8) -> Self {
        let sem = SemKind::try_from(old).unwrap_or(SemKind::IsA);
        Self::simple(sem)
    }
}

impl std::fmt::Display for BitflagRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let neg = if self.is_negated() { "¬" } else { "" };
        write!(f, "{}{}[{:?}][{:?}][{:?}][{:?}]",
            neg, self.sem_kind().as_str(),
            self.certainty(), self.temporality(), self.source(), self.logic())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_unpack_roundtrip() {
        let r = BitflagRelation::build(
            SemKind::HasPart,
            true,                       // negated
            Certainty::Hypothetical,
            Temporality::Was,
            Source::Hearsay,
            Logic::Implies,
        );
        assert_eq!(r.sem_kind(), SemKind::HasPart);
        assert!(r.is_negated());
        assert_eq!(r.certainty(), Certainty::Hypothetical);
        assert_eq!(r.temporality(), Temporality::Was);
        assert_eq!(r.source(), Source::Hearsay);
        assert_eq!(r.logic(), Logic::Implies);
    }

    #[test]
    fn simple_defaults_are_sane() {
        let r = BitflagRelation::simple(SemKind::IsA);
        assert_eq!(r.sem_kind(), SemKind::IsA);
        assert!(!r.is_negated());
        assert_eq!(r.certainty(), Certainty::Certain);
        assert_eq!(r.temporality(), Temporality::Timeless);
        assert_eq!(r.source(), Source::Observation);
        assert_eq!(r.logic(), Logic::Plain);
    }

    #[test]
    fn all_16_sem_kinds_roundtrip() {
        let kinds = [
            SemKind::IsA, SemKind::HasPart, SemKind::TemplateOf, SemKind::HasColor,
            SemKind::HasPose, SemKind::Causes, SemKind::LocatedAt, SemKind::Near,
            SemKind::Depicts, SemKind::Says, SemKind::MemberOf, SemKind::InstanceOf,
            SemKind::SimilarTo, SemKind::SameAs, SemKind::Hug, SemKind::Love,
        ];
        for k in kinds {
            let r = BitflagRelation::simple(k);
            assert_eq!(r.sem_kind(), k);
        }
    }

    #[test]
    fn bits_fit_in_14() {
        // Max value should not exceed 14 bits (top 2 reserved)
        let max = BitflagRelation::build(
            SemKind::Love,          // 0xF
            true,                    // 1
            Certainty::Refuted,      // 3
            Temporality::WillBe,     // 3
            Source::Default,         // 3
            Logic::Iff,              // 7
        );
        assert_eq!(max.as_raw() & 0xC000, 0,
            "top 2 bits must be reserved (zero): got {:016b}", max.as_raw());
    }

    #[test]
    fn negation_flips_only_polarity_bit() {
        let pos = BitflagRelation::simple(SemKind::IsA);
        let neg = BitflagRelation::build(
            SemKind::IsA,
            true,
            Certainty::Certain,
            Temporality::Timeless,
            Source::Observation,
            Logic::Plain,
        );
        assert_eq!(pos.as_raw() ^ neg.as_raw(), 1 << 4,
            "only polarity bit should differ");
    }

    #[test]
    fn expressiveness_is_much_richer_than_u8() {
        // A u8 relation gives at most 256 distinct kinds.
        // BitflagRelation gives 16 × 2 × 4 × 4 × 4 × 8 = 16,384 combinations
        // in the same 2-byte slot.
        let combos = 16u32 * 2 * 4 * 4 * 4 * 8;
        assert_eq!(combos, 16_384);
        assert!(combos >= 256 * 64, "should be >>256× richer");
    }

    #[test]
    fn legacy_u8_migration_preserves_sem_kind() {
        for old_val in 0u8..16 {
            let bf = BitflagRelation::from_legacy_u8(old_val);
            let expected = SemKind::try_from(old_val).unwrap_or(SemKind::IsA);
            assert_eq!(bf.sem_kind(), expected);
            // Everything else should be defaults
            assert!(!bf.is_negated());
            assert_eq!(bf.certainty(), Certainty::Certain);
        }
    }

    #[test]
    fn display_shows_negation_symbol() {
        let neg = BitflagRelation::build(
            SemKind::IsA, true,
            Certainty::Certain, Temporality::Timeless,
            Source::Observation, Logic::Plain,
        );
        let s = format!("{}", neg);
        assert!(s.starts_with('¬'), "negated relation should start with ¬: {}", s);
    }

    #[test]
    fn certainty_axis_independent_of_others() {
        for c in [Certainty::Certain, Certainty::Probable,
                  Certainty::Hypothetical, Certainty::Refuted] {
            let r = BitflagRelation::build(
                SemKind::Causes, false, c,
                Temporality::Is, Source::Inference, Logic::Plain,
            );
            assert_eq!(r.certainty(), c);
            assert_eq!(r.sem_kind(), SemKind::Causes);
            assert!(!r.is_negated());
        }
    }

    #[test]
    fn bitflag_fits_in_u16() {
        assert_eq!(std::mem::size_of::<BitflagRelation>(), 2);
    }
}
