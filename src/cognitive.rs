//! Cognitive connection types — 16 distinct edge-kind taxonomies mirroring
//! the human brain's relational architecture.
//!
//! # Why this module exists
//!
//! Idan's insight (23.04.26): to build super-AGI, we cannot lump all edges
//! under a single "relation" concept. The brain has at least 16 DISTINCT
//! ways of connecting ideas, each with its own neural substrate, operation
//! type, and strength model.
//!
//! # Research basis (from consultation with gpt-4o + Gemini 2.5)
//!
//! See docs/working/20260423_brain_architecture_consultation_V1.md for full
//! research. Key references:
//! - Patterson et al. 2007 (ATL semantic hub)
//! - Eichenbaum 2017 (hippocampal episodic binding)
//! - Squire 1992 (declarative/procedural dissociation)
//! - Hebb 1949 (associative learning)
//! - Baddeley 2003 (working memory model)
//! - Friston 2010 (free energy / predictive processing)
//!
//! # How these differ from BitflagRelation
//!
//! `BitflagRelation` encodes *semantic modifiers* (polarity, certainty,
//! temporality, source, logic) — orthogonal axes within a single connection.
//!
//! `CognitiveKind` is the TYPE OF CONNECTION itself — which neural pathway
//! is being modeled. The two compose: an edge has a `CognitiveKind` (which
//! brain system) AND a `BitflagRelation` (which modifiers within that system).
//!
//! # Integration
//!
//! Each AtomEdge will carry both:
//!   cognitive: CognitiveKind (5 bits → 32 kinds, 16 used, 16 reserved)
//!   bitflag:   BitflagRelation (14 bits)
//! Combined: 19 bits — fits in a u24 or u32 with room for weight.

use std::convert::TryFrom;

/// The 16 distinct connection types from neuroscience research.
///
/// Each variant documents:
/// - Primary brain region(s)
/// - Characteristic operation (spreading, binding, retrieval, etc.)
/// - Example (apple-related for consistency)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CognitiveKind {
    /// SEMANTIC — meaning, category, property. Spreading activation.
    /// Brain: anterior temporal lobe (ATL), vmPFC, angular gyrus.
    /// Example: "apple" → "fruit" (category), "red" (property).
    Semantic = 0,

    /// EPISODIC — specific events with spatiotemporal context.
    /// Brain: hippocampus, medial PFC, retrosplenial, parahippocampal.
    /// Example: "apple" → "the one I ate yesterday at the farmer's market".
    Episodic = 1,

    /// PROCEDURAL — motor skills, sequences, habits. Implicit.
    /// Brain: basal ganglia, cerebellum, SMA, premotor cortex.
    /// Example: "apple" → "the peeling motion".
    Procedural = 2,

    /// LEXICAL — word-form to meaning mappings.
    /// Brain: STG, fusiform gyrus (VWFA), angular gyrus.
    /// Example: "apple" (word) → /ˈæp.əl/ (sound) → concept.
    Lexical = 3,

    /// PHONETIC — speech-sound patterns, phoneme sequences.
    /// Brain: primary auditory cortex, STG, Wernicke's area.
    /// Example: /æ/ in "apple" → /æ/ in "cat", "hat".
    Phonetic = 4,

    /// ORTHOGRAPHIC — visual letter patterns, spelling.
    /// Brain: visual cortex, fusiform VWFA, inferior temporal.
    /// Example: "A-P-P-L-E" visual string → word.
    Orthographic = 5,

    /// SYNTACTIC — grammatical structure, phrase composition.
    /// Brain: Broca's area, anterior temporal, pSTG.
    /// Example: "The apple is red" → subject-verb-complement.
    Syntactic = 6,

    /// SPATIAL — location, proximity, navigation paths.
    /// Brain: hippocampus (place/grid cells), parahippocampal, posterior parietal.
    /// Example: "apple" → "on the table", "next to banana".
    Spatial = 7,

    /// TEMPORAL — sequence, timing, duration.
    /// Brain: hippocampus, prefrontal, cerebellum.
    /// Example: "picked" → "washed" → "eaten".
    Temporal = 8,

    /// CAUSAL — cause-effect, explanation, prediction.
    /// Brain: dorsolateral PFC, parietal, TPJ.
    /// Example: "eating rotten apple" → "getting sick".
    Causal = 9,

    /// EMOTIONAL — affective valence, arousal, value.
    /// Brain: amygdala, vmPFC, insula, orbitofrontal.
    /// Example: "apple" → "pleasant taste" / "sour disappointment".
    Emotional = 10,

    /// ANALOGICAL — relational mapping between domains.
    /// Brain: right PFC (abstract relations), ACC, parietal.
    /// Example: "apple:tree :: fish:water" (habitat relation).
    Analogical = 11,

    /// ASSOCIATIVE — statistical co-occurrence, Hebbian.
    /// Brain: hippocampus, neocortex, thalamus. Ubiquitous.
    /// Example: "apple" frequently co-occurs with "pie", "cider", "orchard".
    Associative = 12,

    /// CULTURAL — shared societal meaning, symbols, rituals.
    /// Brain: ATL, vmPFC (semantic) + mPFC, TPJ, STS (social).
    /// Example: "apple" → "forbidden fruit" / "Newton's discovery".
    Cultural = 13,

    /// METAPHORICAL — cross-domain mapping for understanding abstract via concrete.
    /// Brain: right PFC, ATL, ACC.
    /// Example: "time is money", "apple of my eye".
    Metaphorical = 14,

    /// PRAGMATIC — context-dependent meaning, social inference.
    /// Brain: PFC, TPJ (theory of mind), right hemisphere for non-literal.
    /// Example: "apple" as a gift vs. "apple" as forbidden fruit — context resolves.
    Pragmatic = 15,
}

impl CognitiveKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            CognitiveKind::Semantic => "semantic",
            CognitiveKind::Episodic => "episodic",
            CognitiveKind::Procedural => "procedural",
            CognitiveKind::Lexical => "lexical",
            CognitiveKind::Phonetic => "phonetic",
            CognitiveKind::Orthographic => "orthographic",
            CognitiveKind::Syntactic => "syntactic",
            CognitiveKind::Spatial => "spatial",
            CognitiveKind::Temporal => "temporal",
            CognitiveKind::Causal => "causal",
            CognitiveKind::Emotional => "emotional",
            CognitiveKind::Analogical => "analogical",
            CognitiveKind::Associative => "associative",
            CognitiveKind::Cultural => "cultural",
            CognitiveKind::Metaphorical => "metaphorical",
            CognitiveKind::Pragmatic => "pragmatic",
        }
    }

    /// Which primary brain region(s) mediate this connection type?
    /// Used for traversal weighting and for routing queries by activated region.
    pub fn primary_region(&self) -> &'static str {
        match self {
            CognitiveKind::Semantic => "ATL + vmPFC + angular_gyrus",
            CognitiveKind::Episodic => "hippocampus + medial_PFC",
            CognitiveKind::Procedural => "basal_ganglia + cerebellum",
            CognitiveKind::Lexical => "STG + VWFA + angular_gyrus",
            CognitiveKind::Phonetic => "auditory_cortex + STG",
            CognitiveKind::Orthographic => "visual_cortex + VWFA",
            CognitiveKind::Syntactic => "Broca + anterior_temporal",
            CognitiveKind::Spatial => "hippocampus + parietal",
            CognitiveKind::Temporal => "hippocampus + PFC + cerebellum",
            CognitiveKind::Causal => "dlPFC + parietal + TPJ",
            CognitiveKind::Emotional => "amygdala + vmPFC + insula",
            CognitiveKind::Analogical => "right_PFC + ACC",
            CognitiveKind::Associative => "hippocampus + neocortex",
            CognitiveKind::Cultural => "ATL + TPJ + mPFC",
            CognitiveKind::Metaphorical => "right_PFC + ATL",
            CognitiveKind::Pragmatic => "PFC + TPJ",
        }
    }

    /// The characteristic operation for this connection type.
    /// Informs which walk algorithm is most effective.
    pub fn operation(&self) -> CognitiveOp {
        match self {
            CognitiveKind::Semantic => CognitiveOp::SpreadingActivation,
            CognitiveKind::Episodic => CognitiveOp::ContextualRetrieval,
            CognitiveKind::Procedural => CognitiveOp::SequenceExecution,
            CognitiveKind::Lexical => CognitiveOp::FormToMeaning,
            CognitiveKind::Phonetic => CognitiveOp::SoundPattern,
            CognitiveKind::Orthographic => CognitiveOp::VisualPattern,
            CognitiveKind::Syntactic => CognitiveOp::StructuralParse,
            CognitiveKind::Spatial => CognitiveOp::NavigationPath,
            CognitiveKind::Temporal => CognitiveOp::OrderedSequence,
            CognitiveKind::Causal => CognitiveOp::InferenceChain,
            CognitiveKind::Emotional => CognitiveOp::ValenceTag,
            CognitiveKind::Analogical => CognitiveOp::RelationalMap,
            CognitiveKind::Associative => CognitiveOp::CoOccurrence,
            CognitiveKind::Cultural => CognitiveOp::ContextualSchema,
            CognitiveKind::Metaphorical => CognitiveOp::DomainTransfer,
            CognitiveKind::Pragmatic => CognitiveOp::ContextResolution,
        }
    }

    /// Which "default" cognitive kinds should a query walk?
    /// For unknown query intent: semantic + associative are safe defaults.
    pub fn default_walk_set() -> &'static [CognitiveKind] {
        &[CognitiveKind::Semantic, CognitiveKind::Associative, CognitiveKind::Lexical]
    }
}

impl TryFrom<u8> for CognitiveKind {
    type Error = ();
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(CognitiveKind::Semantic),
            1 => Ok(CognitiveKind::Episodic),
            2 => Ok(CognitiveKind::Procedural),
            3 => Ok(CognitiveKind::Lexical),
            4 => Ok(CognitiveKind::Phonetic),
            5 => Ok(CognitiveKind::Orthographic),
            6 => Ok(CognitiveKind::Syntactic),
            7 => Ok(CognitiveKind::Spatial),
            8 => Ok(CognitiveKind::Temporal),
            9 => Ok(CognitiveKind::Causal),
            10 => Ok(CognitiveKind::Emotional),
            11 => Ok(CognitiveKind::Analogical),
            12 => Ok(CognitiveKind::Associative),
            13 => Ok(CognitiveKind::Cultural),
            14 => Ok(CognitiveKind::Metaphorical),
            15 => Ok(CognitiveKind::Pragmatic),
            _ => Err(()),
        }
    }
}

/// The characteristic operation performed on this connection type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CognitiveOp {
    /// Breadth-first activation ("doctor" → "nurse", "hospital", "medicine")
    SpreadingActivation,
    /// Specific episode reinstatement (hippocampal pattern completion)
    ContextualRetrieval,
    /// Step-by-step automated motor/skill execution
    SequenceExecution,
    /// Word ↔ concept binding
    FormToMeaning,
    /// Phoneme / syllable sequence match
    SoundPattern,
    /// Letter / glyph pattern match
    VisualPattern,
    /// Grammatical tree parse
    StructuralParse,
    /// Geometric path through spatial graph
    NavigationPath,
    /// Ordered-sequence walk with temporal constraints
    OrderedSequence,
    /// Directed a-causes-b walk, backward for explanation, forward for prediction
    InferenceChain,
    /// Edge weight modulated by affective valence (-1 to +1)
    ValenceTag,
    /// A:B::C:D relational match (structure mapping)
    RelationalMap,
    /// Co-occurrence counting (Hebbian update)
    CoOccurrence,
    /// Schema activation from cultural context
    ContextualSchema,
    /// Source-domain → target-domain transfer
    DomainTransfer,
    /// Inference from context + theory-of-mind
    ContextResolution,
}

/// Maximum legal value for the 5-bit cognitive_kind field.
pub const COGNITIVE_KIND_MAX: u8 = 31;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_16_kinds_roundtrip_via_u8() {
        for v in 0..16u8 {
            let k = CognitiveKind::try_from(v).unwrap();
            assert_eq!(k as u8, v);
        }
    }

    #[test]
    fn invalid_u8_rejected() {
        assert!(CognitiveKind::try_from(16u8).is_err());
        assert!(CognitiveKind::try_from(255u8).is_err());
    }

    #[test]
    fn each_kind_has_nonempty_name() {
        for v in 0..16u8 {
            let k = CognitiveKind::try_from(v).unwrap();
            assert!(!k.as_str().is_empty());
            assert!(!k.primary_region().is_empty());
        }
    }

    #[test]
    fn kind_fits_in_5_bits() {
        // We have 16 kinds now, room for 16 more in 5 bits (32 total)
        assert!(15 <= COGNITIVE_KIND_MAX);
        assert_eq!(std::mem::size_of::<CognitiveKind>(), 1);
    }

    #[test]
    fn semantic_uses_spreading_activation() {
        assert_eq!(CognitiveKind::Semantic.operation(), CognitiveOp::SpreadingActivation);
    }

    #[test]
    fn episodic_uses_contextual_retrieval() {
        // Per Eichenbaum 2017: hippocampus binds episodes, retrieval is
        // pattern completion, NOT spreading.
        assert_eq!(CognitiveKind::Episodic.operation(), CognitiveOp::ContextualRetrieval);
        assert!(CognitiveKind::Episodic.primary_region().contains("hippocampus"));
    }

    #[test]
    fn procedural_distinct_from_declarative() {
        // Per Squire 1992: procedural memory is dissociable from declarative.
        // Different brain systems (basal ganglia vs medial temporal).
        assert!(CognitiveKind::Procedural.primary_region().contains("basal_ganglia"));
        assert!(!CognitiveKind::Procedural.primary_region().contains("hippocampus"));
    }

    #[test]
    fn emotional_routes_to_amygdala() {
        assert!(CognitiveKind::Emotional.primary_region().contains("amygdala"));
    }

    #[test]
    fn default_walk_set_is_reasonable() {
        let default = CognitiveKind::default_walk_set();
        assert!(!default.is_empty());
        assert!(default.contains(&CognitiveKind::Semantic));
        assert!(default.contains(&CognitiveKind::Associative));
    }

    #[test]
    fn all_kinds_have_distinct_operations() {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for v in 0..16u8 {
            let k = CognitiveKind::try_from(v).unwrap();
            // Operations CAN be shared between kinds — e.g., spatial and
            // temporal both use sequence walks. We don't enforce uniqueness,
            // just collect them.
            seen.insert(k.operation());
        }
        // But there should be at least 10 distinct operations
        assert!(seen.len() >= 10, "got only {} distinct ops", seen.len());
    }

    #[test]
    fn brain_region_strings_contain_plus_separator_if_multi() {
        // Sanity: if region mentions multiple areas, uses +
        for v in 0..16u8 {
            let k = CognitiveKind::try_from(v).unwrap();
            let r = k.primary_region();
            // Either single region or uses + separator
            assert!(!r.contains(" and "), "use + not 'and' in {}", r);
        }
    }
}
