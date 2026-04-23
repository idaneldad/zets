//! # Reading — the result of reading a ReadInput
//!
//! A `Reading` is the structured understanding ZETS has of a particular
//! input. It is the bridge between "raw message" and "compose response".
//!
//! The Reading contains five coordinates:
//!   - `emotion`  — what the source seems to feel
//!   - `intent`   — what they actually mean (vs what they said)
//!   - `style`    — how they communicate (for mirroring)
//!   - `gate`     — whether ZETS should respond, and how carefully
//!   - `directive`— if it should respond, *how*: tone, length, energy
//!
//! These are produced in order by the Reader modules:
//!   emotion.rs → intent.rs → style.rs → gate.rs → directive.rs
//!
//! Each is keyed by atoms in the graph (`emotion_signal`, `intent_kind`,
//! etc.) — the Reading is mostly a lightweight collection of AtomIds +
//! confidences, not a heavyweight data structure.

use std::collections::HashMap;

/// A Reading is the result of Reader::read(input).
///
/// Thin wrapper over AtomIds and confidence scores — all heavy logic
/// lives in the graph traversal, not in this struct.
#[derive(Debug, Clone, Default)]
pub struct Reading {
    /// Perceived emotional state.
    pub emotion: EmotionRead,
    /// Inferred pragmatic intent.
    pub intent: IntentRead,
    /// Communication style of the source.
    pub style: StyleRead,
    /// Whether/how to respond.
    pub gate: GateRead,
    /// If responding, what kind of response.
    pub directive: ResponseDirective,
    /// Overall confidence in this reading (0..1).
    /// Low confidence = consider asking for clarification.
    pub confidence: f32,
}

// ─── Emotion ────────────────────────────────────────────────────────

/// What the source seems to feel.
///
/// The 8 canonical textual signals are each graph atoms; here we just
/// record which were detected and their strength.
#[derive(Debug, Clone, Default)]
pub struct EmotionRead {
    /// Detected signals: sense_key → strength 0..1.
    /// E.g. "emotion_signal.punctuation_intensity" → 0.8
    pub signals: HashMap<String, f32>,
    /// Best-guess primary state.
    pub primary: EmotionalState,
    /// Arousal/energy level (0=calm, 1=high). Independent of valence.
    pub arousal: f32,
    /// Valence (-1=very negative, 0=neutral, +1=very positive).
    pub valence: f32,
}

/// Broad emotional state categories.
///
/// We use a small closed set for internal routing — nuance is captured
/// by `arousal` + `valence` + `signals`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EmotionalState {
    #[default]
    Neutral,
    /// Sad, low, resigned.
    Depressed,
    /// Blocked, irritated, stuck.
    Frustrated,
    /// Worried, uncertain, tense.
    Anxious,
    /// Hot, fighting, offended.
    Angry,
    /// Curious, engaged, alert.
    Engaged,
    /// Up-energy, positive.
    Excited,
    /// Overloaded, too much to hold.
    Overwhelmed,
}

// ─── Intent ─────────────────────────────────────────────────────────

/// What the source actually means (pragmatic intent).
#[derive(Debug, Clone, Default)]
pub struct IntentRead {
    /// The surface (literal) parse.
    pub literal: String,
    /// The pragmatic interpretation.
    pub pragmatic: PragmaticIntent,
    /// Ambiguity (0 = clear, 1 = many equally-plausible readings).
    pub ambiguity: f32,
    /// Topic / domain if identified (e.g. "domain.finance").
    pub topic: Option<String>,
}

/// Pragmatic intent categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PragmaticIntent {
    #[default]
    /// The message means what it says.
    Literal,
    /// A hint the speaker didn't state directly.
    Hint(Hint),
    /// A rhetorical question — answer is implicit.
    Rhetorical,
    /// A test of ZETS (testing limits, probing).
    Probing,
    /// A request for help, even if not framed as one.
    ImplicitHelp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hint {
    Boredom,
    Leaving,
    Urgent,
    Procrastination,
    Disagreement,
    NeedSupport,
}

// ─── Style ──────────────────────────────────────────────────────────

/// How the source communicates — used for tone matching.
#[derive(Debug, Clone, Default)]
pub struct StyleRead {
    /// Big Five inferred scores (0..1 each, unknown = 0.5).
    pub big_five: BigFive,
    /// Formality: 0 = very casual, 1 = very formal.
    pub formality: f32,
    /// Technical density: 0 = layperson, 1 = expert jargon.
    pub tech_density: f32,
    /// Average sentence length (words).
    pub avg_sentence_len: f32,
    /// Question-to-statement ratio.
    pub question_ratio: f32,
    /// Emotional vocabulary intensity (0..1).
    pub emo_intensity: f32,
    /// Hedging level (0 = definite, 1 = very hedged).
    pub hedging: f32,
}

/// Big Five (OCEAN) — each scored 0..1.
///
/// Scores are updated cumulatively across exchanges. Unknown starts at 0.5.
#[derive(Debug, Clone, Copy, Default)]
pub struct BigFive {
    pub openness: f32,
    pub conscientiousness: f32,
    pub extraversion: f32,
    pub agreeableness: f32,
    pub neuroticism: f32,
}

// ─── Gate (quality gate) ────────────────────────────────────────────

/// Should ZETS respond to this input, and with what caution?
///
/// This layer replaces what used to be called Birur. It is a *decision*
/// about the situation: given what I understand about the source, their
/// emotion, their intent, their history — is a direct answer appropriate?
#[derive(Debug, Clone, Default)]
pub struct GateRead {
    /// Decision: one of 4 actions.
    pub action: GateAction,
    /// Short machine-readable reason (for logging and adaptation).
    pub reason: String,
    /// Which precision dimension was weakest.
    pub weak_dim: Option<String>,
    /// 0..42 gates passed (the non-linear score).
    pub gates_passed: u8,
}

/// Four possible actions from the gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GateAction {
    #[default]
    /// Everything looks good — respond directly.
    Pass,
    /// Respond, but add caveats / ask a clarifying question.
    Assisted,
    /// Deliberate more before responding (S2 system).
    Escalate,
    /// Do not respond directly — explain the hesitation.
    Hold,
}

// ─── Response directive ─────────────────────────────────────────────

/// If the gate said to respond, this directive shapes *how*.
#[derive(Debug, Clone, Default)]
pub struct ResponseDirective {
    /// Target energy level (0=calm, 1=high).
    /// Typically = user's energy + delta based on their state.
    pub energy_target: f32,
    /// Target formality.
    pub formality_target: f32,
    /// Rough length target in words.
    pub length_target: u32,
    /// Which 1-2 style dimensions to mirror (atom sense_keys).
    pub mirror_dims: Vec<String>,
    /// Uplift method if needed.
    pub uplift: UpliftMethod,
    /// Patterns to avoid (e.g. "jargon", "long_caveats").
    pub avoid: Vec<String>,
    /// Patterns to include (e.g. "concrete_example", "next_step").
    pub include: Vec<String>,
    /// Whether to reveal that the response was personalized.
    /// Default: false (adaptation is invisible unless asked).
    pub personalization_visible: bool,
}

/// How to elevate energy if the source is low.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UpliftMethod {
    #[default]
    /// No uplift — match current energy.
    None,
    /// Acknowledge difficulty, validate feelings first.
    Validation,
    /// Offer new angle on the same problem.
    Reframe,
    /// Push forward with a small actionable step.
    Momentum,
    /// Break into clear structure to reduce overwhelm.
    Structure,
    /// Highlight what's already working.
    Progress,
}

impl Reading {
    /// Would a human observer say "this response feels right"?
    ///
    /// True iff: gate allowed response AND confidence is reasonable.
    pub fn is_actionable(&self) -> bool {
        self.gate.action != GateAction::Hold && self.confidence > 0.4
    }

    /// Should ZETS ask for clarification before answering?
    pub fn needs_clarification(&self) -> bool {
        self.gate.action == GateAction::Assisted
            || self.intent.ambiguity > 0.6
            || self.confidence < 0.4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_gate_pass() {
        let r = Reading::default();
        assert_eq!(r.gate.action, GateAction::Pass);
    }

    #[test]
    fn test_is_actionable_default_false() {
        // default confidence = 0, so not actionable
        let r = Reading::default();
        assert!(!r.is_actionable());
    }

    #[test]
    fn test_is_actionable_when_confident() {
        let mut r = Reading::default();
        r.confidence = 0.8;
        assert!(r.is_actionable());
    }

    #[test]
    fn test_needs_clarification_on_low_conf() {
        let mut r = Reading::default();
        r.confidence = 0.2;
        assert!(r.needs_clarification());
    }

    #[test]
    fn test_needs_clarification_on_ambiguity() {
        let mut r = Reading::default();
        r.confidence = 0.9;
        r.intent.ambiguity = 0.8;
        assert!(r.needs_clarification());
    }

    #[test]
    fn test_big_five_default_uncertain() {
        let bf = BigFive::default();
        // 0.0 means "unknown" — all default to 0.0 until inferred.
        // Real inference should push toward 0.5 as neutral prior.
        assert_eq!(bf.openness, 0.0);
    }

    #[test]
    fn test_emotional_state_default_neutral() {
        let e = EmotionRead::default();
        assert_eq!(e.primary, EmotionalState::Neutral);
    }

    #[test]
    fn test_hold_is_not_actionable() {
        let mut r = Reading::default();
        r.confidence = 0.9;
        r.gate.action = GateAction::Hold;
        assert!(!r.is_actionable());
    }
}
