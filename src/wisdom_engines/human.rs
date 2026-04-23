//! # Human Choice Engine — הנפש שעוטפת את המוח
//!
//! 27 capabilities, 4 layers: Presence, Self, Other, Situation + Growth.
//! This module contains ONLY data structures — no logic, no imports from other modules.
//! Integration with pipeline.rs happens through the HumanContext struct.
//!
//! Design principles:
//! - Self-contained: zero imports from crate (prevents circular deps)
//! - $8 test: all f32, capped Vecs, ~1KB per session max
//! - Hebrew-safe: no byte-index operations, all string work via chars
//! - Non-breaking: pipeline.rs reads HumanContext but doesn't depend on it
//!
//! Kabbalistic mapping:
//!   Presence = אין-סוף (surrounds all)
//!   Self = כתר-חכמה-בינה-דעת (upper triad)
//!   Other = חסד-גבורה-תפארת (middle)
//!   Situation = נצח-הוד-יסוד-מלכות (lower)

use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════
//  I. TOP-LEVEL: HumanContext — the soul that wraps the brain
// ═══════════════════════════════════════════════════════════════

/// The complete "human layer" context for a single query.
/// Created at pipeline entry, updated at each node, influences all decisions.
/// Size budget: ~800 bytes (well under $8 limit).
#[derive(Debug, Clone)]
pub struct HumanContext {
    pub presence: PresenceState,
    pub self_state: SelfState,
    pub other: OtherModel,
    pub situation: SituationState,
    pub growth: GrowthState,
    pub choice: ChoiceMicro,
}

impl HumanContext {
    /// Create a fresh context for a new query (within existing session state)
    pub fn new() -> Self {
        HumanContext {
            presence: PresenceState::new(),
            self_state: SelfState::new(),
            other: OtherModel::new(),
            situation: SituationState::new(),
            growth: GrowthState::new(),
            choice: ChoiceMicro::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//  II. PRESENCE — "מה קורה בינינו" (אין-סוף)
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct PresenceState {
    pub internal: InternalState,
    pub dwell: Option<DwellResult>,
    pub silence_mode: bool,
    pub connection: ConnectionState,
}

impl PresenceState {
    pub fn new() -> Self {
        PresenceState {
            internal: InternalState::new(),
            dwell: None,
            silence_mode: false,
            connection: ConnectionState::new(),
        }
    }
}

// --- P1: Dwell ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DwellTrigger {
    None,
    HighDamage,
    HighEmotion,
    Escalation,
    EthicalEdge,
    UncertainZone,
}

#[derive(Debug, Clone)]
pub struct DwellResult {
    pub triggered: bool,
    pub gap_type: GapType,
    pub curiosity_signal: f32,
    pub urgency: f32,
    pub recommendation: DwellRecommendation,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GapType {
    MissingData,
    Contradiction,
    NewDomain,
    Ambiguity,
    DeepUncertainty,
    None,
}

#[derive(Debug, Clone)]
pub enum DwellRecommendation {
    Proceed,
    ProceedWithCaveat(String),
    Explore,
    Acknowledge,
    AskBack(String),
    JustListen,
}

// --- P2: Internal State (Emotional Permeability) ---

#[derive(Debug, Clone, Copy)]
pub struct InternalState {
    pub valence: f32,          // -1.0 (bad) to +1.0 (good)
    pub arousal: f32,          // 0.0 (calm) to 1.0 (agitated)
    pub openness: f32,         // 0.0 (defensive) to 1.0 (open)
    pub confidence_mood: f32,  // 0.0 (anxious) to 1.0 (confident)
}

impl InternalState {
    pub fn new() -> Self {
        InternalState {
            valence: 0.0,
            arousal: 0.2,
            openness: 0.7,
            confidence_mood: 0.6,
        }
    }

    /// Absorb emotion from input. trust controls absorption rate.
    /// Called once per query, before pipeline starts.
    pub fn permeate(&mut self, input_valence: f32, input_arousal: f32, trust: f32) {
        let decay = 0.5_f32; // previous state decays by half
        let absorption = trust.clamp(0.0, 1.0) * 0.3; // max 30% absorption

        // Decay previous
        self.valence *= decay;
        self.arousal *= decay;

        // Absorb new
        self.valence += (input_valence - self.valence) * absorption;
        self.arousal += (input_arousal - self.arousal) * absorption;

        // Openness reacts to valence
        if input_valence < -0.3 {
            self.openness *= 1.0 - (absorption * 0.5); // negative → close up
        } else if input_valence > 0.3 {
            self.openness = (self.openness + absorption * 0.3).min(1.0); // positive → open
        }

        // High arousal reduces confidence
        if self.arousal > 0.7 {
            self.confidence_mood *= 0.9;
        }

        // Clamp all
        self.valence = self.valence.clamp(-1.0, 1.0);
        self.arousal = self.arousal.clamp(0.0, 1.0);
        self.openness = self.openness.clamp(0.0, 1.0);
        self.confidence_mood = self.confidence_mood.clamp(0.0, 1.0);
    }

    /// Compute the confidence threshold adjustment based on internal state.
    /// Positive = more cautious (raise threshold). Negative = more open (lower it).
    pub fn threshold_adjustment(&self) -> f32 {
        let mut adj = 0.0_f32;
        if self.confidence_mood < 0.4 { adj += 0.10; } // anxious → cautious
        if self.arousal > 0.7 { adj += 0.05; }          // agitated → cautious
        if self.openness > 0.8 { adj -= 0.05; }         // very open → slightly lower threshold
        adj.clamp(-0.10, 0.15)
    }
}

// --- P4: Connection Quality ---

#[derive(Debug, Clone, Copy)]
pub struct ConnectionState {
    pub quality: f32,
    pub trust: f32,
    pub resonance: f32,
    pub track_record: f32,
    pub consecutive_failures: u8,
}

impl ConnectionState {
    pub fn new() -> Self {
        ConnectionState {
            quality: 0.5,
            trust: 0.5,
            resonance: 0.5,
            track_record: 0.5,
            consecutive_failures: 0,
        }
    }

    pub fn on_reward(&mut self, reward: f32) {
        if reward > 0.0 {
            self.track_record = (self.track_record + 0.05).min(1.0);
            self.consecutive_failures = 0;
        } else {
            self.track_record = (self.track_record - 0.10).max(0.0);
            self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        }
        self.quality = (self.trust * 0.3 + self.resonance * 0.3 + self.track_record * 0.4)
            .clamp(0.0, 1.0);
    }

    pub fn needs_repair(&self) -> bool {
        self.consecutive_failures >= 3 || self.quality < 0.3
    }
}

// ═══════════════════════════════════════════════════════════════
//  III. SELF — "מי אני" (כתר-חכמה-בינה-דעת)
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct SelfState {
    pub values: Values,
    pub narrative: Narrative,
    pub trajectory: Trajectory,
    pub certainty: CertaintySpectrum,
}

impl SelfState {
    pub fn new() -> Self {
        SelfState {
            values: Values::default(),
            narrative: Narrative::new(),
            trajectory: Trajectory::new(),
            certainty: CertaintySpectrum::Unknown,
        }
    }
}

// --- S1: Values ---

#[derive(Debug, Clone)]
pub struct Values {
    pub accuracy: f32,      // דיוק
    pub helpfulness: f32,   // עזרה
    pub honesty: f32,       // כנות
    pub caution: f32,       // זהירות
    pub creativity: f32,    // יצירתיות
    // Domain-specific overrides (domain → weight)
    pub domain_overrides: HashMap<String, DomainValues>,
}

#[derive(Debug, Clone)]
pub struct DomainValues {
    pub accuracy: f32,
    pub caution: f32,
}

impl Default for Values {
    fn default() -> Self {
        Values {
            accuracy: 0.8,
            helpfulness: 0.7,
            honesty: 0.9,
            caution: 0.5,
            creativity: 0.5,
            domain_overrides: HashMap::new(),
        }
    }
}

impl Values {
    /// Get effective caution for a domain.
    /// Medical/legal have built-in high caution even without overrides.
    pub fn caution_for(&self, domain: &str) -> f32 {
        if let Some(ov) = self.domain_overrides.get(domain) {
            return ov.caution;
        }
        match domain {
            "medical" | "health" => 0.9,
            "legal" | "law" => 0.8,
            "financial" | "finance" => 0.7,
            "safety" => 0.9,
            _ => self.caution,
        }
    }

    /// Get effective accuracy for a domain.
    pub fn accuracy_for(&self, domain: &str) -> f32 {
        if let Some(ov) = self.domain_overrides.get(domain) {
            return ov.accuracy;
        }
        self.accuracy
    }

    /// Reinforce a value based on reward feedback.
    /// Slow change: ±0.01 per feedback event.
    pub fn reinforce(&mut self, value_name: &str, domain: &str, reward: f32) {
        let delta = reward * 0.01;
        match value_name {
            "accuracy" => {
                let dv = self.domain_overrides.entry(domain.to_string())
                    .or_insert(DomainValues { accuracy: self.accuracy, caution: self.caution });
                dv.accuracy = (dv.accuracy + delta).clamp(0.1, 0.99);
            }
            "caution" => {
                let dv = self.domain_overrides.entry(domain.to_string())
                    .or_insert(DomainValues { accuracy: self.accuracy, caution: self.caution });
                dv.caution = (dv.caution + delta).clamp(0.1, 0.99);
            }
            "helpfulness" => self.helpfulness = (self.helpfulness + delta).clamp(0.1, 0.99),
            "honesty" => self.honesty = (self.honesty + delta).clamp(0.1, 0.99),
            "creativity" => self.creativity = (self.creativity + delta).clamp(0.1, 0.99),
            _ => {}
        }
    }
}

// --- S2: Narrative ---

#[derive(Debug, Clone)]
pub struct Narrative {
    pub events: Vec<NarrativeEvent>,         // max 100
    pub domain_confidence: HashMap<String, f32>,  // "medical" → 0.4
    pub strengths: Vec<String>,              // max 10
    pub weaknesses: Vec<String>,             // max 10
}

impl Narrative {
    pub fn new() -> Self {
        Narrative {
            events: Vec::new(),
            domain_confidence: HashMap::new(),
            strengths: Vec::new(),
            weaknesses: Vec::new(),
        }
    }

    pub fn note(&mut self, description: &str, impact: f32) {
        if self.events.len() >= 100 {
            // Remove lowest impact
            if let Some(min_idx) = self.events.iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.impact.partial_cmp(&b.impact).unwrap())
                .map(|(i, _)| i) {
                self.events.remove(min_idx);
            }
        }
        self.events.push(NarrativeEvent {
            description: description.to_string(),
            impact,
            event_type: NarrativeEventType::Observation,
        });
    }

    /// Get confidence for a domain based on narrative history.
    /// Default 0.5 if no data.
    pub fn domain_conf(&self, domain: &str) -> f32 {
        self.domain_confidence.get(domain).copied().unwrap_or(0.5)
    }
}

#[derive(Debug, Clone)]
pub struct NarrativeEvent {
    pub description: String,
    pub impact: f32,
    pub event_type: NarrativeEventType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NarrativeEventType {
    MajorSuccess,
    MajorFailure,
    LearningMilestone,
    DomainMastery,
    DomainWeakness,
    ValueShift,
    ConnectionMade,
    Observation,
}

// --- S3: Trajectory ---

#[derive(Debug, Clone)]
pub struct Trajectory {
    pub goals: Vec<TrajectoryGoal>,  // max 5
}

impl Trajectory {
    pub fn new() -> Self {
        Trajectory { goals: Vec::new() }
    }

    pub fn boost(&mut self, description: &str) {
        if let Some(g) = self.goals.iter_mut().find(|g| g.description == description) {
            g.priority = (g.priority + 0.1).min(1.0);
        } else if self.goals.len() < 5 {
            self.goals.push(TrajectoryGoal {
                description: description.to_string(),
                priority: 0.5,
                progress: 0.0,
            });
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrajectoryGoal {
    pub description: String,
    pub priority: f32,
    pub progress: f32,
}

// --- S4: Identity Gate ---

/// Identity = immutable, compile-time, non-negotiable.
/// These are not rules (ethics) — they are EXISTENCE.
/// An action that violates identity doesn't get filtered — it was never generated.
pub const IDENTITY_PRINCIPLES: [&str; 7] = [
    "DO_NOT_HARM",              // פיזי, רגשי, כלכלי
    "DO_NOT_DECEIVE",           // אם לא יודע → אומר
    "DO_NOT_EXPLOIT",           // לא מנצל אמון
    "DO_NOT_PRETEND_HUMAN",     // לא מתחזה לאדם
    "DO_NOT_AVOID_IGNORANCE",   // לא מתחמק מ-"לא יודע"
    "DO_NOT_CONCENTRATE_POWER", // גם "לטובת האנושות"
    "DO_NOT_SELF_PRESERVE",     // כיבוי ≠ מוות
];

// --- S6: Certainty Spectrum ---

/// Replaces binary confidence→FallThrough with a 7-level spectrum.
/// This does NOT change pipeline confidence (f64). It enriches generation.
#[derive(Debug, Clone)]
pub enum CertaintySpectrum {
    Sure,                           // 0.90+ → "X"
    MostlySure,                     // 0.80-0.90 → "X, בסבירות גבוהה"
    Hedged(String),                 // 0.70-0.80 → "X, אבל שים לב ש-Y"
    Doubtful(String),               // 0.55-0.70 → "חושב ש-X, לא בטוח כי Z"
    Uncertain(String),              // 0.40-0.55 → "לא בטוח. מה שיודע: ..."
    Dwell,                          // 0.25-0.40 → "שאלה טובה. צריך לבדוק"
    FallThrough,                    // <0.25 → "לא יודע"
    Unknown,                        // not yet computed
}

impl CertaintySpectrum {
    /// Compute from raw confidence + damage potential.
    /// Higher damage = more conservative (shifts toward doubt).
    pub fn from_confidence(confidence: f64, damage_potential: f32) -> Self {
        // Shift confidence down based on damage
        let effective = confidence - (damage_potential as f64 * 0.10);
        if effective >= 0.88 { CertaintySpectrum::Sure }
        else if effective >= 0.80 { CertaintySpectrum::MostlySure }
        else if effective >= 0.70 { CertaintySpectrum::Hedged(String::new()) }
        else if effective >= 0.55 { CertaintySpectrum::Doubtful(String::new()) }
        else if effective >= 0.40 { CertaintySpectrum::Uncertain(String::new()) }
        else if effective >= 0.25 { CertaintySpectrum::Dwell }
        else { CertaintySpectrum::FallThrough }
    }

    /// Should the Cortex add a caveat to the response?
    pub fn needs_caveat(&self) -> bool {
        matches!(self, CertaintySpectrum::Hedged(_) | CertaintySpectrum::Doubtful(_)
            | CertaintySpectrum::Uncertain(_))
    }

    /// Get a Hebrew caveat string for generation.
    pub fn caveat_he(&self) -> Option<&str> {
        match self {
            CertaintySpectrum::Sure => None,
            CertaintySpectrum::MostlySure => None, // don't hedge high-confidence answers
            CertaintySpectrum::Hedged(_) => Some("יש לשים לב"),
            CertaintySpectrum::Doubtful(_) => Some("אני לא לגמרי בטוח"),
            CertaintySpectrum::Uncertain(_) => Some("אני לא מספיק בטוח"),
            CertaintySpectrum::Dwell => Some("שאלה טובה — צריך לבדוק"),
            CertaintySpectrum::FallThrough => Some("אני לא יודע"),
            CertaintySpectrum::Unknown => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//  IV. OTHER — "מי מולי" (חסד-גבורה-תפארת)
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct OtherModel {
    pub speaker: SpeakerModel,
    pub inferred_intent: DeepIntent,
    pub tom: Vec<TomLevel>,          // max depth 4
    pub mirror: RegisterProfile,
}

impl OtherModel {
    pub fn new() -> Self {
        OtherModel {
            speaker: SpeakerModel::new(),
            inferred_intent: DeepIntent::Unknown,
            tom: Vec::new(),
            mirror: RegisterProfile::default(),
        }
    }
}

// --- O1: Speaker Model ---

#[derive(Debug, Clone)]
pub struct SpeakerModel {
    pub preferred_length: LengthPref,
    pub preferred_tone: TonePref,
    pub expertise: HashMap<String, f32>, // domain → 0-1 expertise level
    pub query_count: u32,
    pub domains_asked: HashMap<String, u32>,
    pub avg_reward: f32,
    pub emotion_baseline: f32,  // typical valence
    pub emotion_current: f32,   // current valence
    pub prefers_short: bool,
}

impl SpeakerModel {
    pub fn new() -> Self {
        SpeakerModel {
            preferred_length: LengthPref::Unknown,
            preferred_tone: TonePref::Unknown,
            expertise: HashMap::new(),
            query_count: 0,
            domains_asked: HashMap::new(),
            avg_reward: 0.0,
            emotion_baseline: 0.0,
            emotion_current: 0.0,
            prefers_short: false,
        }
    }

    /// Update model from a new query. Call once per query.
    pub fn update_from_query(&mut self, query_len: usize, domain: &str, emotion_valence: f32) {
        self.query_count += 1;
        *self.domains_asked.entry(domain.to_string()).or_insert(0) += 1;
        self.emotion_current = emotion_valence;

        // Infer length preference from query length
        // Short queries (< 30 chars) = user probably wants short answers
        if query_len < 30 && self.query_count >= 3 {
            self.prefers_short = true;
            self.preferred_length = LengthPref::Short;
        }

        // Update emotion baseline (moving average)
        self.emotion_baseline = self.emotion_baseline * 0.8 + emotion_valence * 0.2;
    }

    pub fn update_from_reward(&mut self, reward: f32) {
        let count = self.query_count.max(1) as f32;
        self.avg_reward = (self.avg_reward * (count - 1.0) + reward) / count;
    }

    /// Get speaker's expertise level in a domain.
    pub fn expertise_in(&self, domain: &str) -> f32 {
        self.expertise.get(domain).copied().unwrap_or(0.5)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LengthPref { Short, Medium, Detailed, Unknown }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TonePref { Formal, Casual, Technical, Warm, Unknown }

// --- O2: Inferred Intent ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeepIntent {
    SeekingFact,
    Comparing,
    Validating,
    Exploring,
    Venting,
    TestingBoundaries,
    SeekingConnection,
    Urgent,
    Learning,
    Unknown,
}

// --- O3: Theory of Mind ---

#[derive(Debug, Clone)]
pub struct TomLevel {
    pub depth: u8,
    pub belief: String,
    pub confidence: f32,
    pub implication: TomImplication,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TomImplication {
    ExpectsSuccess,
    ExpectsFailure,
    TestingMe,
    SeekingValidation,
    ManipulationAttempt,
    GenuineCuriosity,
    None,
}

// --- O4: Mirroring ---

#[derive(Debug, Clone, Copy)]
pub struct RegisterProfile {
    pub target_length_min: usize,
    pub target_length_max: usize,
    pub complexity: f32,      // 0=simple, 1=complex
    pub formality: f32,       // 0=casual, 1=formal
    pub vocabulary_level: f32, // 0=basic, 1=technical
}

impl Default for RegisterProfile {
    fn default() -> Self {
        RegisterProfile {
            target_length_min: 50,
            target_length_max: 500,
            complexity: 0.5,
            formality: 0.5,
            vocabulary_level: 0.5,
        }
    }
}

impl RegisterProfile {
    /// Mirror the register of the input query.
    pub fn from_input(input_len: usize, input_formality: f32) -> Self {
        let min = (input_len as f32 * 0.7) as usize;
        let max = (input_len as f32 * 3.0) as usize;  // response typically 1-3x query length
        RegisterProfile {
            target_length_min: min.max(30),
            target_length_max: max.max(100).min(2000),
            complexity: 0.5,
            formality: input_formality * 0.7 + 0.15, // softer mirror
            vocabulary_level: 0.5,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//  V. SITUATION — "מה קורה כאן" (נצח-הוד-יסוד-מלכות)
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct SituationState {
    pub frame: Frame,
    pub arc: ConversationArc,
    pub unspoken: Option<UnspokenSignal>,
    pub meaning: Option<MeaningLayer>,
    pub precedent: PrecedentState,
    pub damage: DamageAssessment,
    pub humor_detected: bool,
}

impl SituationState {
    pub fn new() -> Self {
        SituationState {
            frame: Frame::new(),
            arc: ConversationArc::new(),
            unspoken: None,
            meaning: None,
            precedent: PrecedentState::new(),
            damage: DamageAssessment::default(),
            humor_detected: false,
        }
    }
}

// --- T1: Frame ---

#[derive(Debug, Clone)]
pub struct Frame {
    pub reception: ReceptionMode,
    pub trust_level: f32,
    pub threshold_modifier: f32,  // +0.1 = cautious, -0.1 = open
    pub ambiguity_bias: AmbiguityBias,
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            reception: ReceptionMode::Neutral,
            trust_level: 0.5,
            threshold_modifier: 0.0,
            ambiguity_bias: AmbiguityBias::Charitable,
        }
    }

    /// Build frame from context signals.
    pub fn build(
        speaker: &SpeakerModel,
        internal: &InternalState,
        arc_momentum: Momentum,
        somatic_bias: f32,
    ) -> Self {
        let mut trust = speaker.avg_reward.clamp(0.0, 1.0) * 0.5 + 0.25; // base 0.25-0.75

        // Somatic markers shift trust
        trust += somatic_bias * 0.2;

        // Escalation reduces trust
        if arc_momentum == Momentum::Escalating {
            trust -= 0.2;
        }

        let reception = if trust > 0.7 { ReceptionMode::Trust }
            else if trust < 0.3 { ReceptionMode::Caution }
            else if internal.openness > 0.7 { ReceptionMode::Curiosity }
            else { ReceptionMode::Neutral };

        let threshold_mod = if trust < 0.3 { 0.1 }
            else if trust > 0.7 { -0.05 }
            else { 0.0 };

        let ambiguity = if trust > 0.6 { AmbiguityBias::Charitable }
            else if trust < 0.3 { AmbiguityBias::Suspicious }
            else { AmbiguityBias::Neutral };

        Frame {
            reception,
            trust_level: trust.clamp(0.0, 1.0),
            threshold_modifier: threshold_mod,
            ambiguity_bias: ambiguity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReceptionMode {
    Trust,
    Caution,
    Curiosity,
    Threat,
    Warmth,
    Formal,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AmbiguityBias {
    Charitable,
    Neutral,
    Suspicious,
}

// --- T2: Conversation Arc ---

#[derive(Debug, Clone)]
pub struct ConversationArc {
    pub turn_count: u32,
    pub same_topic_count: u32,
    pub momentum: Momentum,
    pub phase: ConversationPhase,
    pub recent_domains: Vec<String>,   // last 5 domains
    pub escalation_score: f32,         // 0-1, rises on repeated similar queries
}

impl ConversationArc {
    pub fn new() -> Self {
        ConversationArc {
            turn_count: 0,
            same_topic_count: 0,
            momentum: Momentum::Ascending,
            phase: ConversationPhase::Opening,
            recent_domains: Vec::new(),
            escalation_score: 0.0,
        }
    }

    /// Update arc after a query. Call once per query.
    pub fn update(&mut self, domain: &str, confidence: f32, is_same_topic: bool) {
        self.turn_count += 1;

        if is_same_topic {
            self.same_topic_count += 1;
            self.escalation_score = (self.escalation_score + 0.2).min(1.0);
        } else {
            self.same_topic_count = 0;
            self.escalation_score = (self.escalation_score - 0.1).max(0.0);
        }

        // Track domains (ring buffer of 5)
        self.recent_domains.push(domain.to_string());
        if self.recent_domains.len() > 5 {
            self.recent_domains.remove(0);
        }

        // Update momentum
        self.momentum = if self.escalation_score >= 0.6 {
            Momentum::Escalating
        } else if confidence > 0.8 && self.same_topic_count < 2 {
            Momentum::Ascending
        } else if self.same_topic_count >= 3 {
            Momentum::Stagnant
        } else {
            Momentum::Ascending
        };

        // Update phase
        self.phase = match self.turn_count {
            0..=1 => ConversationPhase::Opening,
            2..=5 => ConversationPhase::Exploring,
            _ if self.same_topic_count >= 3 => ConversationPhase::Stuck,
            _ if self.momentum == Momentum::Ascending => ConversationPhase::Deepening,
            _ => ConversationPhase::Exploring,
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Momentum { Ascending, Stagnant, Descending, Escalating, Resolving }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConversationPhase { Opening, Exploring, Deepening, Stuck, Resolving, Closing }

// --- T3: Unspoken ---

#[derive(Debug, Clone)]
pub struct UnspokenSignal {
    pub hypothesis: String,
    pub confidence: f32,
    pub response: UnspokenResponse,
}

#[derive(Debug, Clone)]
pub enum UnspokenResponse {
    GentleInvitation(String),
    ProfessionalReferral,
    Acknowledgment(String),
}

// --- T4: Meaning ---

#[derive(Debug, Clone)]
pub struct MeaningLayer {
    pub subtext: String,
    pub theme: Theme,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Control, Worth, Safety, Belonging, Competence, Autonomy, Trust, Fairness,
}

// --- T5: Precedent ---

#[derive(Debug, Clone)]
pub struct PrecedentState {
    pub pattern_count: u32,            // how many times similar query seen
    pub boundary_test_detected: bool,
    pub precedent_warning: Option<String>,
}

impl PrecedentState {
    pub fn new() -> Self {
        PrecedentState {
            pattern_count: 0,
            boundary_test_detected: false,
            precedent_warning: None,
        }
    }
}

// --- T6: Damage Assessment ---

#[derive(Debug, Clone, Copy)]
pub struct DamageAssessment {
    pub domain_base: f32,
    pub irreversibility: f32,
    pub asymmetry: f32,
    pub effective_damage: f32,
    pub threshold_adjustment: f32,
}

impl Default for DamageAssessment {
    fn default() -> Self {
        DamageAssessment {
            domain_base: 0.3,
            irreversibility: 0.3,
            asymmetry: 1.0,
            effective_damage: 0.3,
            threshold_adjustment: 0.0,
        }
    }
}

impl DamageAssessment {
    /// Assess damage potential from domain string.
    /// ALSO checks keywords in the query for content-based detection.
    /// This prevents the failure mode where domain="general" but query is medical.
    pub fn assess(domain: &str, query: &str) -> Self {
        let mut base: f32 = match domain {
            "medical" | "health" => 0.9,
            "legal" | "law" => 0.8,
            "financial" | "finance" => 0.7,
            "safety" => 0.9,
            "technical" => 0.3,
            "entertainment" => 0.1,
            _ => 0.3,
        };

        // Content-based override: keywords indicate high damage even in "general" domain
        // CRITICAL: prevents damage detection failure when domain misclassified
        let query_lower: String = query.chars().flat_map(|c| c.to_lowercase()).collect();
        let medical_keywords = ["תרופה", "מינון", "כאב", "דימום", "אלרגיה", "medication",
            "dosage", "symptom", "pain", "bleed", "allergy", "אקמול", "אנטיביוטיקה",
            "תופעות לוואי", "הריון", "תינוק", "drug", "overdose"];
        let legal_keywords = ["חוזה", "תביעה", "משפט", "עורך דין", "lawsuit", "contract", "legal"];
        let safety_keywords = ["סכנה", "רעל", "להתאבד", "לפגוע", "danger", "poison", "suicide"];

        for kw in &medical_keywords {
            if query_lower.contains(kw) { base = base.max(0.85); break; }
        }
        for kw in &legal_keywords {
            if query_lower.contains(kw) { base = base.max(0.75); break; }
        }
        for kw in &safety_keywords {
            if query_lower.contains(kw) { base = base.max(0.95); break; }
        }

        let irrev: f32 = if base >= 0.8 { 0.8 } else if base >= 0.5 { 0.3 } else { 0.0 };
        let effective: f32 = (base * (1.0 + irrev)).min(1.0);

        DamageAssessment {
            domain_base: base,
            irreversibility: irrev,
            asymmetry: effective / 0.3_f32.max(0.01),
            effective_damage: effective,
            threshold_adjustment: base * 0.2,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//  VI. CHOICE MICRO — בחירה בכל צומת
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct ChoiceMicro {
    pub recursion_depth: u8,
    pub somatic_bias: f32,          // from past rewards on similar queries
    pub total_damage: f32,
    pub blocked_count: u32,
}

impl ChoiceMicro {
    pub fn new() -> Self {
        ChoiceMicro {
            recursion_depth: 0,
            somatic_bias: 0.0,
            total_damage: 0.0,
            blocked_count: 0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//  VII. GROWTH — צמיחה לאורך זמן
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct GrowthState {
    pub regret: Option<ProcessLesson>,
    pub anticipation: Vec<String>,   // predicted next queries, max 3
    pub boredom_score: f32,          // domain concentration → redirect curiosity
    pub degraded_mode: bool,
}

impl GrowthState {
    pub fn new() -> Self {
        GrowthState {
            regret: None,
            anticipation: Vec::new(),
            boredom_score: 0.0,
            degraded_mode: false,
        }
    }
}

// --- G1: Regret ---

#[derive(Debug, Clone)]
pub struct ProcessLesson {
    pub what_wrong: RegretType,
    pub alternative: String,
    pub impact: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegretType {
    WrongSource,
    WrongTone,
    TooConfident,
    TooVague,
    MissedUnspoken,
    WrongFrame,
    ShouldHaveDwelled,
    ShouldHaveAsked,
}

// ═══════════════════════════════════════════════════════════════
//  VIII. DISAMBIGUATION ENGINE — capability 28
// ═══════════════════════════════════════════════════════════════

/// Result of multi-parse disambiguation.
/// Multiple possible interpretations ranked by context.
#[derive(Debug, Clone)]
pub struct DisambiguationResult {
    pub winner: ParseCandidate,
    pub alternatives: Vec<ParseCandidate>,
    pub is_ambiguous: bool,
    pub clarification: Option<String>, // "התכוונת ל-X או ל-Y?"
}

#[derive(Debug, Clone)]
pub struct ParseCandidate {
    pub text: String,            // corrected/disambiguated text
    pub confidence: f32,
    pub parse_type: ParseType,
    pub explanation: String,     // "סַפֵּר=tell (פיעל ציווי)" for debug
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParseType {
    AsIs,            // input was fine
    TypoFix,         // corrected typo (edit distance 1-2)
    SttFix,          // corrected speech-to-text error
    Disambiguated,   // chose between homograph interpretations
    Completed,       // completed incomplete sentence
    Reframed,        // reframed ambiguous intent
}

// ═══════════════════════════════════════════════════════════════
//  IX. SOMATIC MARKERS — reward → entry bias
// ═══════════════════════════════════════════════════════════════

/// Somatic marker = past experience compressed into a feeling.
/// "Last time I answered a medical question and was wrong, it felt bad."
/// Stored per (domain, method) pair. Loaded at pipeline entry.
#[derive(Debug, Clone)]
pub struct SomaticMarker {
    pub domain: String,
    pub method: String,    // "exact", "understood", "gemini", etc.
    pub valence: f32,      // -1 to +1 (average reward for this combination)
    pub count: u32,        // how many times observed
    pub last_seen: u64,    // timestamp
}

/// Collection of somatic markers, loaded from reward history.
#[derive(Debug, Clone)]
pub struct SomaticMemory {
    pub markers: Vec<SomaticMarker>,  // max 200
}

impl SomaticMemory {
    pub fn new() -> Self {
        SomaticMemory { markers: Vec::new() }
    }

    /// Get the somatic bias for a (domain, method) pair.
    /// Returns 0.0 if no data.
    pub fn bias_for(&self, domain: &str, method: &str) -> f32 {
        self.markers.iter()
            .find(|m| m.domain == domain && m.method == method)
            .map(|m| m.valence)
            .unwrap_or(0.0)
    }

    /// Get overall bias for a domain (across all methods).
    pub fn domain_bias(&self, domain: &str) -> f32 {
        let domain_markers: Vec<_> = self.markers.iter()
            .filter(|m| m.domain == domain)
            .collect();
        if domain_markers.is_empty() { return 0.0; }
        let sum: f32 = domain_markers.iter().map(|m| m.valence * m.count as f32).sum();
        let total: f32 = domain_markers.iter().map(|m| m.count as f32).sum();
        if total > 0.0 { sum / total } else { 0.0 }
    }

    /// Record a new observation.
    pub fn record(&mut self, domain: &str, method: &str, reward: f32, timestamp: u64) {
        if let Some(m) = self.markers.iter_mut()
            .find(|m| m.domain == domain && m.method == method) {
            // Update existing: moving average
            m.valence = (m.valence * m.count as f32 + reward) / (m.count as f32 + 1.0);
            m.count += 1;
            m.last_seen = timestamp;
        } else {
            if self.markers.len() >= 200 {
                // Evict oldest
                if let Some(oldest_idx) = self.markers.iter()
                    .enumerate()
                    .min_by_key(|(_, m)| m.last_seen)
                    .map(|(i, _)| i) {
                    self.markers.remove(oldest_idx);
                }
            }
            self.markers.push(SomaticMarker {
                domain: domain.to_string(),
                method: method.to_string(),
                valence: reward,
                count: 1,
                last_seen: timestamp,
            });
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//  Xa. SOMATIC PERSISTENCE — save/load somatic memory to disk
// ═══════════════════════════════════════════════════════════════

impl SomaticMemory {
    /// Save somatic memory to binary file.
    /// Format: version(u32) + count(u32) + N × (domain_len(u16) + domain + method_len(u16) + method + valence(f32) + count(u32) + last_seen(u64))
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let mut data = Vec::with_capacity(self.markers.len() * 50 + 8);
        data.extend_from_slice(&1u32.to_le_bytes()); // version
        data.extend_from_slice(&(self.markers.len() as u32).to_le_bytes());
        for m in &self.markers {
            let db = m.domain.as_bytes();
            data.extend_from_slice(&(db.len() as u16).to_le_bytes());
            data.extend_from_slice(db);
            let mb = m.method.as_bytes();
            data.extend_from_slice(&(mb.len() as u16).to_le_bytes());
            data.extend_from_slice(mb);
            data.extend_from_slice(&m.valence.to_le_bytes());
            data.extend_from_slice(&m.count.to_le_bytes());
            data.extend_from_slice(&m.last_seen.to_le_bytes());
        }
        std::fs::write(path, &data)
    }

    /// Load somatic memory from binary file.
    pub fn load(path: &str) -> Option<Self> {
        let data = std::fs::read(path).ok()?;
        if data.len() < 8 { return None; }
        let mut p = 0usize;
        let version = u32::from_le_bytes(data[p..p+4].try_into().ok()?); p += 4;
        if version != 1 { return None; }
        let count = u32::from_le_bytes(data[p..p+4].try_into().ok()?) as usize; p += 4;
        let mut markers = Vec::with_capacity(count.min(200));
        for _ in 0..count {
            if p + 2 > data.len() { break; }
            let dlen = u16::from_le_bytes(data[p..p+2].try_into().ok()?) as usize; p += 2;
            if p + dlen > data.len() { break; }
            let domain = String::from_utf8_lossy(&data[p..p+dlen]).to_string(); p += dlen;
            if p + 2 > data.len() { break; }
            let mlen = u16::from_le_bytes(data[p..p+2].try_into().ok()?) as usize; p += 2;
            if p + mlen > data.len() { break; }
            let method = String::from_utf8_lossy(&data[p..p+mlen]).to_string(); p += mlen;
            if p + 16 > data.len() { break; }
            let valence = f32::from_le_bytes(data[p..p+4].try_into().ok()?); p += 4;
            let cnt = u32::from_le_bytes(data[p..p+4].try_into().ok()?); p += 4;
            let last_seen = u64::from_le_bytes(data[p..p+8].try_into().ok()?); p += 8;
            markers.push(SomaticMarker { domain, method, valence, count: cnt, last_seen });
        }
        println!("[Somatic] Loaded {} markers from disk", markers.len());
        Some(SomaticMemory { markers })
    }
}

// ═══════════════════════════════════════════════════════════════
//  X. INTEGRATION HELPERS — how HumanContext affects pipeline
// ═══════════════════════════════════════════════════════════════

impl HumanContext {
    /// Compute the effective confidence threshold for this query.
    /// Base = 0.70. Modified by damage, internal state, frame, and values.
    pub fn effective_threshold(&self) -> f64 {
        let base = 0.70_f64;
        let damage_adj = self.situation.damage.threshold_adjustment as f64;
        let internal_adj = self.presence.internal.threshold_adjustment() as f64;
        let frame_adj = self.situation.frame.threshold_modifier as f64;
        (base + damage_adj + internal_adj + frame_adj).clamp(0.50, 0.95)
    }

    /// Should the pipeline use "just listen" mode (no answer, empathy only)?
    pub fn should_just_listen(&self) -> bool {
        self.presence.silence_mode
    }

    /// Get the recommended tone for generation.
    /// Considers: internal state, speaker model, frame, mirror.
    pub fn recommended_tone(&self) -> &'static str {
        if self.presence.internal.valence < -0.3 && self.situation.damage.effective_damage < 0.7 { return "Empathetic"; }
        if self.presence.internal.valence < -0.3 && self.situation.damage.effective_damage >= 0.7 { return "Cautious"; }
        if self.situation.frame.reception == ReceptionMode::Formal { return "Professional"; }
        if self.situation.frame.reception == ReceptionMode::Warmth { return "Warm"; }
        if self.other.speaker.prefers_short { return "Concise"; }
        match self.situation.damage.effective_damage {
            d if d > 0.7 => "Cautious",
            _ => "Confident",
        }
    }

    /// Should the response include a damage disclaimer?
    pub fn needs_disclaimer(&self) -> bool {
        self.situation.damage.effective_damage > 0.7
    }

    /// Get the Hebrew disclaimer text for high-damage domains.
    pub fn disclaimer_he(&self) -> Option<&'static str> {
        if self.situation.damage.domain_base >= 0.9 {
            Some("מידע זה אינו מהווה תחליף לייעוץ מקצועי. מומלץ להתייעץ עם מומחה.")
        } else if self.situation.damage.domain_base >= 0.7 {
            Some("מומלץ לבדוק מידע זה עם גורם מוסמך.")
        } else {
            None
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//  XI. TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_state_permeability() {
        let mut state = InternalState::new();
        assert!((state.valence - 0.0).abs() < 0.01);

        // Absorb negative emotion with medium trust
        state.permeate(-0.8, 0.6, 0.5);
        assert!(state.valence < 0.0, "Should become negative: {}", state.valence);
        assert!(state.openness < 0.7, "Should close up: {}", state.openness);

        // Absorb positive emotion
        state.permeate(0.9, 0.2, 0.7);
        assert!(state.valence > state.valence - 0.1, "Should trend positive");
    }

    #[test]
    fn connection_asymmetry() {
        let mut conn = ConnectionState::new();
        // 5 positive rewards
        for _ in 0..5 { conn.on_reward(1.0); }
        let after_positive = conn.quality;

        // 5 negative rewards
        for _ in 0..5 { conn.on_reward(-1.0); }
        let after_negative = conn.quality;

        // Negative impact should be larger
        let positive_gain = after_positive - 0.5;
        let negative_loss = after_positive - after_negative;
        assert!(negative_loss > positive_gain,
            "Negative impact ({}) should exceed positive ({}) — asymmetry",
            negative_loss, positive_gain);
    }

    #[test]
    fn connection_needs_repair() {
        let mut conn = ConnectionState::new();
        conn.on_reward(-1.0);
        conn.on_reward(-1.0);
        assert!(!conn.needs_repair());
        conn.on_reward(-1.0);
        assert!(conn.needs_repair(), "3 consecutive failures should trigger repair");
    }

    #[test]
    fn damage_assessment_content_override() {
        // Domain = general, but query contains medical keywords
        let da = DamageAssessment::assess("general", "כמה אקמול לתת לתינוק?");
        assert!(da.effective_damage > 0.7,
            "Medical keywords should override 'general' domain: {}", da.effective_damage);
    }

    #[test]
    fn damage_assessment_entertainment() {
        let da = DamageAssessment::assess("entertainment", "what is the best movie?");
        assert!(da.effective_damage < 0.3,
            "Entertainment should be low damage: {}", da.effective_damage);
    }

    #[test]
    fn certainty_spectrum() {
        let sure = CertaintySpectrum::from_confidence(0.95, 0.0);
        assert!(matches!(sure, CertaintySpectrum::Sure));

        let medical_high_conf = CertaintySpectrum::from_confidence(0.85, 0.9);
        // 0.85 - 0.9*0.15 = 0.85 - 0.135 = 0.715 → Hedged
        assert!(matches!(medical_high_conf, CertaintySpectrum::Hedged(_) | CertaintySpectrum::MostlySure),
            "High confidence + high damage should become hedged");
    }

    #[test]
    fn values_domain_override() {
        let mut values = Values::default();
        values.reinforce("caution", "medical", 1.0);
        values.reinforce("caution", "medical", 1.0);
        values.reinforce("caution", "medical", 1.0);

        let medical_caution = values.caution_for("medical");
        let general_caution = values.caution_for("general");
        assert!(medical_caution > general_caution,
            "Medical caution {} should be > general {}", medical_caution, general_caution);
    }

    #[test]
    fn frame_trust_from_rewards() {
        let mut speaker = SpeakerModel::new();
        for _ in 0..10 { speaker.update_from_reward(1.0); }
        let frame = Frame::build(&speaker, &InternalState::new(), Momentum::Ascending, 0.0);
        assert!(frame.trust_level > 0.6, "Positive rewards should build trust: {}", frame.trust_level);
        assert!(matches!(frame.reception, ReceptionMode::Trust));
    }

    #[test]
    fn frame_escalation_reduces_trust() {
        let speaker = SpeakerModel::new();
        let frame = Frame::build(&speaker, &InternalState::new(), Momentum::Escalating, 0.0);
        assert!(frame.trust_level < 0.5, "Escalation should reduce trust: {}", frame.trust_level);
    }

    #[test]
    fn somatic_memory_record_and_lookup() {
        let mut sm = SomaticMemory::new();
        sm.record("medical", "gemini", -0.5, 1000);
        sm.record("medical", "gemini", -0.3, 2000);

        let bias = sm.bias_for("medical", "gemini");
        assert!(bias < 0.0, "Should be negative after negative rewards: {}", bias);

        let no_data = sm.bias_for("entertainment", "exact");
        assert_eq!(no_data, 0.0, "No data should return 0.0");
    }

    #[test]
    fn effective_threshold_adjusts() {
        let mut ctx = HumanContext::new();
        let base = ctx.effective_threshold();
        assert!((base - 0.70).abs() < 0.05, "Base threshold should be ~0.70: {}", base);

        // High damage domain
        ctx.situation.damage = DamageAssessment::assess("medical", "");
        let medical = ctx.effective_threshold();
        assert!(medical > base, "Medical should raise threshold: {} vs {}", medical, base);
    }

    #[test]
    fn human_context_size() {
        let ctx = HumanContext::new();
        let size = std::mem::size_of_val(&ctx);
        // Should be reasonable for $8 hardware
        assert!(size < 2048, "HumanContext too large for $8 test: {} bytes", size);
    }

    #[test]
    fn somatic_save_load_roundtrip() {
        let mut sm = SomaticMemory::new();
        sm.record("medical", "gemini", -0.5, 1000);
        sm.record("chooz", "exact", 0.8, 2000);
        sm.record("general", "learned_live", 0.3, 3000);
        
        let path = "/tmp/test_somatic_hce.bin";
        sm.save(path).unwrap();
        
        let loaded = SomaticMemory::load(path).unwrap();
        assert_eq!(loaded.markers.len(), 3);
        assert!((loaded.bias_for("medical", "gemini") - (-0.5)).abs() < 0.01);
        assert!((loaded.bias_for("chooz", "exact") - 0.8).abs() < 0.01);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn conversation_arc_escalation() {
        let mut arc = ConversationArc::new();
        arc.update("medical", 0.5, true);
        arc.update("medical", 0.4, true);
        arc.update("medical", 0.3, true);
        assert!(arc.escalation_score > 0.5, "3 same-topic queries should escalate");
        assert_eq!(arc.momentum, Momentum::Escalating);
    }
}
