//! Appraisal — how events become emotions.
//!
//! Based on Appraisal Theory (Lazarus, Scherer, Frijda): emotions don't arise
//! directly from events. They arise from:
//!   event → appraisal (significance + threat + controllability)
//!         → self_schema (what does this mean to ME?)
//!         → coping capacity (can I handle this?)
//!         → emotion
//!         → regulation strategy
//!
//! Example:
//!   EVENT: "dog ran away"
//!   APPRAISAL: loss (high importance, low controllability)
//!   SELF_SCHEMA: "I failed to protect what I care about"
//!   COPING: searching, crying, calling
//!   EMOTION: grief (intensity 0-7)
//!   REGULATION: sister hugs child (strategy=social_support)

use std::collections::HashMap;

use crate::atoms::AtomId;

/// Appraisal dimensions (Lazarus/Scherer).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Appraisal {
    /// How significant is this event to the self? 0-100.
    pub importance: u8,
    /// 0=loss, 1=threat, 2=opportunity, 3=irrelevant
    pub valence: AppraisalValence,
    /// How much control do I have? 0-100.
    pub controllability: u8,
    /// Am I responsible? Others? No one? 0=self, 1=other, 2=circumstance
    pub attribution: Attribution,
    /// Can I cope with this? 0-100.
    pub coping_capacity: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AppraisalValence {
    Loss = 0,
    Threat = 1,
    Opportunity = 2,
    Irrelevant = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Attribution {
    Self_ = 0,
    Other = 1,
    Circumstance = 2,
    Uncertain = 3,
}

/// Emotion types — not exhaustive, covers the major families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmotionKind {
    // Positive
    Joy,
    Love,
    Pride,
    Hope,
    Gratitude,
    Serenity,
    // Negative
    Sadness,
    Fear,
    Anger,
    Shame,
    Guilt,
    Disgust,
    // Complex
    Grief,
    Anxiety,
    Envy,
    Empathy,
    Relief,
    Loneliness,
}

impl EmotionKind {
    pub fn hebrew(self) -> &'static str {
        match self {
            Self::Joy => "שמחה",
            Self::Love => "אהבה",
            Self::Pride => "גאווה",
            Self::Hope => "תקווה",
            Self::Gratitude => "הכרת תודה",
            Self::Serenity => "שלווה",
            Self::Sadness => "עצב",
            Self::Fear => "פחד",
            Self::Anger => "כעס",
            Self::Shame => "בושה",
            Self::Guilt => "אשמה",
            Self::Disgust => "גועל",
            Self::Grief => "אבל",
            Self::Anxiety => "חרדה",
            Self::Envy => "קנאה",
            Self::Empathy => "אמפתיה",
            Self::Relief => "הקלה",
            Self::Loneliness => "בדידות",
        }
    }
}

/// Derive likely emotion from appraisal dimensions.
///
/// This is a rule-based DETERMINISTIC mapping based on appraisal theory.
/// Same appraisal → same emotion, always.
pub fn derive_emotion(a: &Appraisal) -> Option<(EmotionKind, u8)> {
    use AppraisalValence::*;
    use Attribution::*;

    let imp = a.importance as u32;
    let cope_inverse = 100u32.saturating_sub(a.coping_capacity as u32);
    let ctrl_inverse = 100u32.saturating_sub(a.controllability as u32);
    let raw = (imp + cope_inverse + ctrl_inverse) / 3;  // average 0-100
    let intensity = ((raw * 7 / 100).min(7)) as u8;

    let emotion = match (a.valence, a.attribution) {
        (Loss, _) if a.importance >= 80 => EmotionKind::Grief,
        (Loss, _) if a.importance >= 50 => EmotionKind::Sadness,
        (Loss, _) => EmotionKind::Sadness,

        (Threat, _) if a.controllability < 20 => EmotionKind::Fear,
        (Threat, Self_) if a.importance >= 60 => EmotionKind::Anxiety,
        (Threat, Other) if a.importance >= 60 => EmotionKind::Anger,
        (Threat, _) => EmotionKind::Anxiety,

        (Opportunity, _) if a.coping_capacity >= 70 && a.importance >= 50 => EmotionKind::Hope,
        (Opportunity, _) if a.importance >= 80 => EmotionKind::Joy,
        (Opportunity, _) => EmotionKind::Serenity,

        (Irrelevant, _) => return None,
    };
    Some((emotion, intensity))
}

/// An emotional event — the full causal chain from event to emotion to regulation.
#[derive(Debug, Clone)]
pub struct EmotionalEvent {
    pub event_atom: AtomId,
    pub subject_atom: AtomId,  // who experiences it
    pub appraisal: Appraisal,
    pub self_schema_triggered: Option<AtomId>,
    pub emotion: Option<(EmotionKind, u8)>,
    pub regulation_strategy: Option<AtomId>,
}

impl EmotionalEvent {
    pub fn new(event: AtomId, subject: AtomId, appraisal: Appraisal) -> Self {
        let emotion = derive_emotion(&appraisal);
        Self {
            event_atom: event,
            subject_atom: subject,
            appraisal,
            self_schema_triggered: None,
            emotion,
            regulation_strategy: None,
        }
    }

    pub fn with_self_schema(mut self, schema: AtomId) -> Self {
        self.self_schema_triggered = Some(schema);
        self
    }

    pub fn with_regulation(mut self, strategy: AtomId) -> Self {
        self.regulation_strategy = Some(strategy);
        self
    }
}

/// An emotion registry — tracks emotional events for a subject over time.
/// Useful for detecting patterns (Idan's "relation invention engine").
pub struct EmotionalHistory {
    events: Vec<EmotionalEvent>,
    /// count of each emotion kind (for pattern detection)
    emotion_counts: HashMap<EmotionKind, u32>,
}

impl EmotionalHistory {
    pub fn new() -> Self {
        Self { events: Vec::new(), emotion_counts: HashMap::new() }
    }

    pub fn record(&mut self, event: EmotionalEvent) {
        if let Some((kind, _)) = event.emotion {
            *self.emotion_counts.entry(kind).or_insert(0) += 1;
        }
        self.events.push(event);
    }

    pub fn dominant_emotion(&self) -> Option<(EmotionKind, u32)> {
        self.emotion_counts.iter()
            .max_by_key(|(_, &c)| c)
            .map(|(&k, &c)| (k, c))
    }

    pub fn events(&self) -> &[EmotionalEvent] { &self.events }

    /// Detect recurring (event_pattern, emotion) pairs.
    /// This feeds the "relation invention engine" — if pattern appears 3+ times,
    /// we have a candidate for a new derived relation.
    pub fn recurring_patterns(&self, min_count: u32) -> Vec<(EmotionKind, u32)> {
        self.emotion_counts.iter()
            .filter(|(_, &c)| c >= min_count)
            .map(|(&k, &c)| (k, c))
            .collect()
    }
}

impl Default for EmotionalHistory {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loss_with_high_importance_produces_grief() {
        let a = Appraisal {
            importance: 90,
            valence: AppraisalValence::Loss,
            controllability: 10,
            attribution: Attribution::Circumstance,
            coping_capacity: 30,
        };
        let (emotion, intensity) = derive_emotion(&a).unwrap();
        assert_eq!(emotion, EmotionKind::Grief);
        assert!(intensity >= 4);
    }

    #[test]
    fn opportunity_with_capability_produces_hope() {
        let a = Appraisal {
            importance: 70,
            valence: AppraisalValence::Opportunity,
            controllability: 80,
            attribution: Attribution::Self_,
            coping_capacity: 80,
        };
        let (emotion, _) = derive_emotion(&a).unwrap();
        assert_eq!(emotion, EmotionKind::Hope);
    }

    #[test]
    fn threat_uncontrollable_produces_fear() {
        let a = Appraisal {
            importance: 80,
            valence: AppraisalValence::Threat,
            controllability: 10,
            attribution: Attribution::Circumstance,
            coping_capacity: 40,
        };
        let (emotion, _) = derive_emotion(&a).unwrap();
        assert_eq!(emotion, EmotionKind::Fear);
    }

    #[test]
    fn threat_from_other_produces_anger() {
        let a = Appraisal {
            importance: 70,
            valence: AppraisalValence::Threat,
            controllability: 50,
            attribution: Attribution::Other,
            coping_capacity: 60,
        };
        let (emotion, _) = derive_emotion(&a).unwrap();
        assert_eq!(emotion, EmotionKind::Anger);
    }

    #[test]
    fn irrelevant_produces_no_emotion() {
        let a = Appraisal {
            importance: 5,
            valence: AppraisalValence::Irrelevant,
            controllability: 50,
            attribution: Attribution::Circumstance,
            coping_capacity: 50,
        };
        assert!(derive_emotion(&a).is_none());
    }

    #[test]
    fn same_appraisal_always_same_emotion() {
        // Determinism: critical for ZETS
        let a = Appraisal {
            importance: 80, valence: AppraisalValence::Loss,
            controllability: 20, attribution: Attribution::Self_,
            coping_capacity: 30,
        };
        let first = derive_emotion(&a);
        for _ in 0..100 {
            assert_eq!(derive_emotion(&a), first);
        }
    }

    #[test]
    fn history_tracks_dominant_emotion() {
        let mut h = EmotionalHistory::new();
        let a_loss = Appraisal {
            importance: 80, valence: AppraisalValence::Loss,
            controllability: 30, attribution: Attribution::Circumstance,
            coping_capacity: 40,
        };
        for _ in 0..5 {
            h.record(EmotionalEvent::new(1, 2, a_loss));
        }
        let a_hope = Appraisal {
            importance: 60, valence: AppraisalValence::Opportunity,
            controllability: 70, attribution: Attribution::Self_,
            coping_capacity: 80,
        };
        h.record(EmotionalEvent::new(3, 2, a_hope));
        let (dominant, count) = h.dominant_emotion().unwrap();
        assert!(matches!(dominant, EmotionKind::Sadness | EmotionKind::Grief));
        assert_eq!(count, 5);
    }

    #[test]
    fn recurring_patterns_detected() {
        let mut h = EmotionalHistory::new();
        let a = Appraisal {
            importance: 90, valence: AppraisalValence::Loss,
            controllability: 10, attribution: Attribution::Circumstance,
            coping_capacity: 30,
        };
        for _ in 0..4 {
            h.record(EmotionalEvent::new(1, 2, a));
        }
        let patterns = h.recurring_patterns(3);
        assert!(!patterns.is_empty(), "should detect recurring grief");
    }

    #[test]
    fn emotion_has_hebrew_label() {
        assert_eq!(EmotionKind::Joy.hebrew(), "שמחה");
        assert_eq!(EmotionKind::Grief.hebrew(), "אבל");
        assert_eq!(EmotionKind::Loneliness.hebrew(), "בדידות");
    }
}
