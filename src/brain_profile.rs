//! Brain profiles — accessibility-first, strength-aware user modeling.
//!
//! # Philosophy
//!
//! Per Idan (23.04.26): "Understand users with disabilities so we serve them
//! excellently, AND leverage the compensation strengths they developed."
//!
//! This is NOT a disability taxonomy. It's a TRAIT COMPENSATION MAP — which
//! cognitive channels are constrained, which are amplified, and how the
//! query/response pipeline should adapt.
//!
//! # Research basis
//!
//! See docs/working/20260423_brain_architecture_consultation_V1.md
//!
//! Key findings applied here:
//! - Blind users develop enhanced spatial-auditory processing (Kujala et al.)
//! - Deaf signers have heightened visual attention, spatial reasoning (Bavelier et al.)
//! - ADHD: broader attentional sampling, creative divergent thinking (White 2006)
//! - Autism: enhanced pattern recognition, systematic thinking (Mottron 2006)
//! - Synesthesia: cross-modal linking strengthens episodic memory (Smilek 2002)
//! - Aphasia: intact non-verbal reasoning, semantic memory via images (Klein 2014)
//!
//! # Key design decisions
//!
//! 1. Profiles are ATOMS in the graph — editable by user, self-chosen.
//! 2. Profile affects:
//!    - Which CognitiveKind edges to prefer in walks
//!    - Output modality (text, spoken, visual, haptic descriptions)
//!    - Response pacing and depth
//!    - Strength-channel amplification
//! 3. NO diagnostic labels are stored — user says what they need.
//! 4. Profile composes with SearchStrategy — different concerns.

use super::cognitive::CognitiveKind;

/// Primary modality for input/output.
/// User may use multiple — this is the PREFERRED channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modality {
    /// Standard text-in/text-out.
    Textual,
    /// Spoken interaction — needs phonetic precision, longer gaps between ideas.
    Auditory,
    /// Sign language / gestural — visual-spatial patterns.
    VisualGestural,
    /// Tactile / Braille — text-encoded but delivered via touch.
    Tactile,
    /// Visual-rich — diagrams, images, video (for deaf, visual learners).
    VisualRich,
}

/// Attention/processing style. Behavioral descriptors, NOT diagnostic labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttentionStyle {
    /// Sustained focused: prefers deep single-thread exploration.
    /// Matches well with DeepDive search strategy.
    Sustained,
    /// Broad parallel: explores many options in parallel, fast context switching.
    /// Matches RapidIteration / Exploratory strategies.
    Broad,
    /// Pattern-sensitive: strong at detecting structural regularities,
    /// prefers well-organized information.
    PatternSensitive,
    /// Context-adaptive: switches between focused and broad based on task.
    /// The default for most users.
    Adaptive,
}

/// How content should be structured in responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InformationPacing {
    /// Short bursts with clear breaks. Better for broad-attention profiles.
    Chunked,
    /// Deep continuous prose. Better for sustained-attention profiles.
    Continuous,
    /// Step-by-step enumeration. Better for pattern-sensitive profiles.
    Enumerated,
    /// Mixed / standard. Default.
    Standard,
}

/// Which cognitive channels are CONSTRAINED and which are AMPLIFIED
/// for this user. Used to weight walks and route to strength channels.
///
/// Weight: 0.0 = unavailable, 0.5 = neutral, 1.0 = normal, 1.5 = enhanced.
#[derive(Debug, Clone)]
pub struct ChannelStrengths {
    pub semantic: f32,
    pub episodic: f32,
    pub procedural: f32,
    pub lexical: f32,
    pub phonetic: f32,
    pub orthographic: f32,
    pub syntactic: f32,
    pub spatial: f32,
    pub temporal: f32,
    pub causal: f32,
    pub emotional: f32,
    pub analogical: f32,
    pub associative: f32,
    pub cultural: f32,
    pub metaphorical: f32,
    pub pragmatic: f32,
}

impl Default for ChannelStrengths {
    fn default() -> Self {
        Self {
            semantic: 1.0, episodic: 1.0, procedural: 1.0, lexical: 1.0,
            phonetic: 1.0, orthographic: 1.0, syntactic: 1.0, spatial: 1.0,
            temporal: 1.0, causal: 1.0, emotional: 1.0, analogical: 1.0,
            associative: 1.0, cultural: 1.0, metaphorical: 1.0, pragmatic: 1.0,
        }
    }
}

impl ChannelStrengths {
    /// Weight for a given cognitive kind.
    pub fn weight(&self, kind: CognitiveKind) -> f32 {
        match kind {
            CognitiveKind::Semantic => self.semantic,
            CognitiveKind::Episodic => self.episodic,
            CognitiveKind::Procedural => self.procedural,
            CognitiveKind::Lexical => self.lexical,
            CognitiveKind::Phonetic => self.phonetic,
            CognitiveKind::Orthographic => self.orthographic,
            CognitiveKind::Syntactic => self.syntactic,
            CognitiveKind::Spatial => self.spatial,
            CognitiveKind::Temporal => self.temporal,
            CognitiveKind::Causal => self.causal,
            CognitiveKind::Emotional => self.emotional,
            CognitiveKind::Analogical => self.analogical,
            CognitiveKind::Associative => self.associative,
            CognitiveKind::Cultural => self.cultural,
            CognitiveKind::Metaphorical => self.metaphorical,
            CognitiveKind::Pragmatic => self.pragmatic,
        }
    }

    /// Return the top N channels by strength (for routing).
    pub fn top_n(&self, n: usize) -> Vec<(CognitiveKind, f32)> {
        let all: [(CognitiveKind, f32); 16] = [
            (CognitiveKind::Semantic, self.semantic),
            (CognitiveKind::Episodic, self.episodic),
            (CognitiveKind::Procedural, self.procedural),
            (CognitiveKind::Lexical, self.lexical),
            (CognitiveKind::Phonetic, self.phonetic),
            (CognitiveKind::Orthographic, self.orthographic),
            (CognitiveKind::Syntactic, self.syntactic),
            (CognitiveKind::Spatial, self.spatial),
            (CognitiveKind::Temporal, self.temporal),
            (CognitiveKind::Causal, self.causal),
            (CognitiveKind::Emotional, self.emotional),
            (CognitiveKind::Analogical, self.analogical),
            (CognitiveKind::Associative, self.associative),
            (CognitiveKind::Cultural, self.cultural),
            (CognitiveKind::Metaphorical, self.metaphorical),
            (CognitiveKind::Pragmatic, self.pragmatic),
        ];
        let mut v: Vec<_> = all.into_iter().collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        v.truncate(n);
        v
    }
}

/// A complete brain profile for a user.
#[derive(Debug, Clone)]
pub struct BrainProfile {
    pub profile_id: String,
    pub modality_preferred: Modality,
    pub modality_avoided: Option<Modality>,
    pub attention: AttentionStyle,
    pub pacing: InformationPacing,
    pub strengths: ChannelStrengths,
    pub description: String,
}

/// Built-in profiles derived from published cognitive research.
/// Users pick any and customize — or build entirely custom.
impl BrainProfile {
    /// Sighted + hearing + typical attention. The baseline.
    pub fn neurotypical_baseline() -> Self {
        Self {
            profile_id: "baseline".into(),
            modality_preferred: Modality::Textual,
            modality_avoided: None,
            attention: AttentionStyle::Adaptive,
            pacing: InformationPacing::Standard,
            strengths: ChannelStrengths::default(),
            description: "Default profile — all channels at normal weight.".into(),
        }
    }

    /// Blind user: vision unavailable, spatial-auditory enhanced.
    /// Per Kujala et al.: sighted-vs-blind show enhanced auditory cortex
    /// activation to sound, enhanced parietal spatial reasoning from touch.
    pub fn blind() -> Self {
        let mut s = ChannelStrengths::default();
        s.orthographic = 0.0;  // no visual word form
        s.phonetic = 1.4;       // enhanced auditory
        s.spatial = 1.3;        // enhanced spatial-by-sound navigation
        s.temporal = 1.2;       // sequence timing sharper
        Self {
            profile_id: "blind".into(),
            modality_preferred: Modality::Auditory,
            modality_avoided: Some(Modality::VisualRich),
            attention: AttentionStyle::Sustained,
            pacing: InformationPacing::Continuous,
            strengths: s,
            description: "Vision unavailable — spatial-auditory channels \
                          amplified. Prefer descriptive prose for spoken \
                          delivery, avoid visual-only content.".into(),
        }
    }

    /// Deaf signer: visual-spatial channels dominant, sign = gestural.
    /// Per Bavelier et al.: deaf signers show enhanced peripheral visual
    /// attention, superior spatial working memory.
    pub fn deaf_signer() -> Self {
        let mut s = ChannelStrengths::default();
        s.phonetic = 0.2;        // minimal (lip-read or text preferred)
        s.spatial = 1.5;         // enhanced — sign language is spatial
        s.orthographic = 1.2;    // reading is often very strong
        s.syntactic = 1.1;       // sign grammar is spatial-temporal
        Self {
            profile_id: "deaf_signer".into(),
            modality_preferred: Modality::VisualGestural,
            modality_avoided: Some(Modality::Auditory),
            attention: AttentionStyle::PatternSensitive,
            pacing: InformationPacing::Chunked,
            strengths: s,
            description: "Deaf signer — visual-spatial dominant. Prefer \
                          diagrams, signed-video, or chunked text. Avoid \
                          audio-only content.".into(),
        }
    }

    /// Broad-attention profile (sometimes called ADHD-adjacent).
    /// Per White 2006, Abraham 2006: broad attentional filter correlates
    /// with creative divergent thinking.
    pub fn broad_attention() -> Self {
        let mut s = ChannelStrengths::default();
        s.analogical = 1.3;     // cross-domain connections strong
        s.associative = 1.3;     // rapid co-occurrence detection
        s.metaphorical = 1.2;    // unusual mappings
        // Sustained focus can be briefly lower — but this isn't a deficit,
        // it's a different allocation. Keep at 1.0.
        Self {
            profile_id: "broad_attention".into(),
            modality_preferred: Modality::Textual,
            modality_avoided: None,
            attention: AttentionStyle::Broad,
            pacing: InformationPacing::Chunked,
            strengths: s,
            description: "Broad parallel attention — strong at cross-domain \
                          links and divergent ideas. Prefer chunked, visual, \
                          or frequently-new content.".into(),
        }
    }

    /// Deep-focus profile (sometimes autism-adjacent).
    /// Per Mottron 2006: enhanced perceptual functioning, strong systemizing,
    /// detail-oriented pattern detection.
    pub fn deep_focus() -> Self {
        let mut s = ChannelStrengths::default();
        s.syntactic = 1.3;      // pattern-rich structures
        s.orthographic = 1.2;    // detail in written form
        s.procedural = 1.2;      // systematic sequence learning
        s.analogical = 1.2;      // deep structure mapping
        Self {
            profile_id: "deep_focus".into(),
            modality_preferred: Modality::Textual,
            modality_avoided: None,
            attention: AttentionStyle::PatternSensitive,
            pacing: InformationPacing::Enumerated,
            strengths: s,
            description: "Pattern-sensitive deep focus — strong at detail, \
                          systematic structure, deep analysis. Prefer \
                          enumerated or well-organized content.".into(),
        }
    }

    /// Aphasia / language-impaired but non-verbally intact.
    /// Per Klein 2014: semantic memory can be accessed via pictures even
    /// when verbal retrieval is impaired.
    pub fn aphasia_visual() -> Self {
        let mut s = ChannelStrengths::default();
        s.lexical = 0.4;         // word retrieval impaired
        s.syntactic = 0.5;
        s.semantic = 1.1;        // meaning intact via non-verbal route
        s.spatial = 1.2;
        s.orthographic = 0.8;    // reading sometimes preserved
        Self {
            profile_id: "aphasia_visual".into(),
            modality_preferred: Modality::VisualRich,
            modality_avoided: None,
            attention: AttentionStyle::Sustained,
            pacing: InformationPacing::Chunked,
            strengths: s,
            description: "Language production/comprehension impaired — \
                          semantic understanding strong via non-verbal \
                          route. Prefer diagrams, symbols, short clear \
                          sentences.".into(),
        }
    }

    /// Synesthesia — cross-modal linking enhances memory and creativity.
    /// Per Smilek 2002: grapheme-color synesthetes show superior memory
    /// for sequences linked by concurrent colors.
    pub fn synesthete() -> Self {
        let mut s = ChannelStrengths::default();
        s.episodic = 1.3;       // cross-modal tags strengthen binding
        s.associative = 1.4;     // rich cross-channel co-occurrence
        s.metaphorical = 1.3;    // native cross-domain mapping
        s.emotional = 1.1;
        Self {
            profile_id: "synesthete".into(),
            modality_preferred: Modality::VisualRich,
            modality_avoided: None,
            attention: AttentionStyle::PatternSensitive,
            pacing: InformationPacing::Continuous,
            strengths: s,
            description: "Cross-modal linking native — episodic and \
                          metaphorical channels amplified. Richer \
                          associative recall.".into(),
        }
    }

    /// Color-blind user — hue discrimination reduced, contrast/pattern enhanced.
    pub fn color_blind() -> Self {
        let mut s = ChannelStrengths::default();
        s.spatial = 1.1;         // stronger shape/contrast reliance
        s.orthographic = 1.1;
        Self {
            profile_id: "color_blind".into(),
            modality_preferred: Modality::VisualRich,
            modality_avoided: None,
            attention: AttentionStyle::PatternSensitive,
            pacing: InformationPacing::Standard,
            strengths: s,
            description: "Color-blind (any type) — avoid hue-only \
                          distinctions. Use patterns, shapes, labels, \
                          contrast instead.".into(),
        }
    }

    /// Return all built-in profiles.
    pub fn all_builtin() -> Vec<BrainProfile> {
        vec![
            Self::neurotypical_baseline(),
            Self::blind(),
            Self::deaf_signer(),
            Self::broad_attention(),
            Self::deep_focus(),
            Self::aphasia_visual(),
            Self::synesthete(),
            Self::color_blind(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_builtin_profiles_are_valid() {
        for p in BrainProfile::all_builtin() {
            assert!(!p.profile_id.is_empty());
            assert!(!p.description.is_empty());
            // No strength should be negative
            let tops = p.strengths.top_n(16);
            for (_, s) in &tops {
                assert!(*s >= 0.0, "strengths must be non-negative");
                assert!(*s <= 2.0, "strengths capped at 2.0 (double strong)");
            }
        }
    }

    #[test]
    fn blind_profile_has_no_orthographic() {
        let p = BrainProfile::blind();
        assert_eq!(p.strengths.orthographic, 0.0);
        assert!(p.strengths.phonetic > 1.0);  // amplified
        assert_eq!(p.modality_preferred, Modality::Auditory);
    }

    #[test]
    fn deaf_signer_has_strong_spatial() {
        let p = BrainProfile::deaf_signer();
        assert!(p.strengths.spatial > 1.2);   // amplified
        assert!(p.strengths.phonetic < 0.5);   // minimal
        assert_eq!(p.modality_preferred, Modality::VisualGestural);
    }

    #[test]
    fn broad_attention_amplifies_analogical() {
        let p = BrainProfile::broad_attention();
        assert!(p.strengths.analogical > 1.0);
        assert!(p.strengths.associative > 1.0);
        assert_eq!(p.attention, AttentionStyle::Broad);
    }

    #[test]
    fn deep_focus_amplifies_syntactic() {
        let p = BrainProfile::deep_focus();
        assert!(p.strengths.syntactic > 1.0);
        assert!(p.strengths.procedural > 1.0);
        assert_eq!(p.attention, AttentionStyle::PatternSensitive);
    }

    #[test]
    fn aphasia_reduces_lexical_preserves_semantic() {
        let p = BrainProfile::aphasia_visual();
        assert!(p.strengths.lexical < 0.5);    // impaired
        assert!(p.strengths.semantic >= 1.0);   // preserved or better
    }

    #[test]
    fn top_n_returns_sorted_descending() {
        let p = BrainProfile::blind();
        let top3 = p.strengths.top_n(3);
        assert_eq!(top3.len(), 3);
        assert!(top3[0].1 >= top3[1].1);
        assert!(top3[1].1 >= top3[2].1);
    }

    #[test]
    fn baseline_is_truly_neutral() {
        let p = BrainProfile::neurotypical_baseline();
        // Every strength = 1.0
        for kind in 0..16u8 {
            let k = CognitiveKind::try_from(kind).unwrap();
            assert_eq!(p.strengths.weight(k), 1.0);
        }
    }

    #[test]
    fn no_profile_has_diagnostic_term_in_id() {
        // Per established rule — no diagnostic labels. Use behavioral.
        let forbidden = [
            "adhd", "autistic", "autism", "disorder",
            "deficit", "disability",
        ];
        for p in BrainProfile::all_builtin() {
            let id = p.profile_id.to_lowercase();
            for f in &forbidden {
                assert!(!id.contains(f),
                    "profile id '{}' uses diagnostic term '{}'", id, f);
            }
        }
    }

    #[test]
    fn synesthete_amplifies_cross_modal_channels() {
        let p = BrainProfile::synesthete();
        assert!(p.strengths.associative > 1.0);
        assert!(p.strengths.metaphorical > 1.0);
        assert!(p.strengths.episodic > 1.0);
    }

    #[test]
    fn weight_returns_correct_channel_value() {
        let p = BrainProfile::blind();
        assert_eq!(p.strengths.weight(CognitiveKind::Orthographic), 0.0);
        assert!(p.strengths.weight(CognitiveKind::Phonetic) > 1.0);
    }

    #[test]
    fn all_profiles_have_description() {
        for p in BrainProfile::all_builtin() {
            assert!(p.description.len() > 20,
                "description for '{}' is too short", p.profile_id);
        }
    }
}
