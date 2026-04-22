//! Cognitive Brain Relations — 64 typed relations in 9 families.
//!
//! Built from the deep spec consultation with Idan (22 Apr 2026).
//! Each relation has:
//!   - Stable 6-bit code (0-63)
//!   - Family (Ontological/Structural/Perceptual/Functional/Causal/Temporal/
//!     Spatial/Social/Emotional/Identity/Therapeutic/Creative/Epistemic)
//!   - Direction (directed or symmetric)
//!   - Transitivity (yes/no/unknown)
//!   - Brain region it primarily lives in
//!   - Cognitive mode affinity (which walk modes prefer this relation)
//!
//! This is a REGISTRY. The atom store edges continue to use a u8 relation
//! tag — this registry explains WHAT each tag means and HOW to reason with it.

use std::collections::HashMap;

/// Direction of a relation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// A → B is meaningful, B → A is not the same relation.
    Directed,
    /// A ↔ B, the relation holds both ways.
    Symmetric,
}

/// Transitivity — can we infer A r C from (A r B, B r C)?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transitivity {
    /// Always transitive (A is_a B, B is_a C ⇒ A is_a C).
    Always,
    /// Never transitive (A near B, B near C does NOT imply A near C).
    Never,
    /// Context-dependent.
    Sometimes,
}

/// Brain region — where this type of knowledge lives.
/// Loosely inspired by Idan's 9-region cognitive architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrainRegion {
    CoreReality,       // is_a, part_of — the stable cortex
    Perceptual,        // looks_like, abstracted_from
    EventNarrative,    // agent_of, narrative_before
    SocialMind,        // trusts, cares_for
    EmotionAppraisal,  // appraised_as, emotion_triggered
    SelfSchema,        // core_belief, identity_threatened_by
    GrowthTherapy,     // regulated_by, coping_strategy_for
    Creative,          // analogous_to, remote_association_to
    MetaCognition,     // supported_by, contradicted_by
}

impl BrainRegion {
    pub fn name(self) -> &'static str {
        match self {
            Self::CoreReality => "core_reality",
            Self::Perceptual => "perceptual",
            Self::EventNarrative => "event_narrative",
            Self::SocialMind => "social_mind",
            Self::EmotionAppraisal => "emotion_appraisal",
            Self::SelfSchema => "self_schema",
            Self::GrowthTherapy => "growth_therapy",
            Self::Creative => "creative",
            Self::MetaCognition => "meta_cognition",
        }
    }
}

/// Which cognitive mode prefers walking this relation type.
/// Multiple modes can prefer the same relation (represented as flags).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeAffinity {
    pub precision: bool, // strict logical chains
    pub divergent: bool, // brainstorming, associations
    pub gestalt: bool,   // neighborhood / wholeness
    pub narrative: bool, // story composition
}

impl ModeAffinity {
    pub const fn all() -> Self { Self { precision: true, divergent: true, gestalt: true, narrative: true } }
    pub const fn precise() -> Self { Self { precision: true, divergent: false, gestalt: true, narrative: true } }
    pub const fn creative() -> Self { Self { precision: false, divergent: true, gestalt: true, narrative: true } }
    pub const fn story() -> Self { Self { precision: true, divergent: false, gestalt: false, narrative: true } }
    pub const fn emotional() -> Self { Self { precision: false, divergent: true, gestalt: true, narrative: true } }
}

/// The full relation definition.
#[derive(Debug, Clone)]
pub struct RelationDef {
    pub code: u8,
    pub name: &'static str,
    pub description: &'static str,
    pub region: BrainRegion,
    pub direction: Direction,
    pub transitivity: Transitivity,
    pub affinity: ModeAffinity,
    /// Hebrew template hint for narrative composition: use {A} and {B}
    pub hebrew_template: &'static str,
}

// ────────────────────────────────────────────────────────────────
// The 64 relation registry.
// Each category occupies a contiguous code range for easy filtering.
// ────────────────────────────────────────────────────────────────

pub const ALL_RELATIONS: &[RelationDef] = &[
    // ── Ontological (0x00-0x03) ──
    RelationDef { code: 0x00, name: "is_a", description: "Taxonomic subtype",
        region: BrainRegion::CoreReality, direction: Direction::Directed,
        transitivity: Transitivity::Always, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} הוא {B}" },
    RelationDef { code: 0x01, name: "instance_of", description: "Individual of a class",
        region: BrainRegion::CoreReality, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} הוא דוגמה של {B}" },
    RelationDef { code: 0x02, name: "variant_of", description: "Variation of a base",
        region: BrainRegion::CoreReality, direction: Direction::Directed,
        transitivity: Transitivity::Sometimes, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} הוא וריאציה של {B}" },
    RelationDef { code: 0x03, name: "prototype_of", description: "Canonical example of a category",
        region: BrainRegion::CoreReality, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} הוא אבטיפוס של {B}" },

    // ── Structural / Part-Whole (0x04-0x08) ──
    RelationDef { code: 0x04, name: "has_part", description: "Contains as part",
        region: BrainRegion::Perceptual, direction: Direction::Directed,
        transitivity: Transitivity::Sometimes, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} כולל את {B}" },
    RelationDef { code: 0x05, name: "part_of", description: "Inverse of has_part",
        region: BrainRegion::Perceptual, direction: Direction::Directed,
        transitivity: Transitivity::Sometimes, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} חלק מ{B}" },
    RelationDef { code: 0x06, name: "fills_slot", description: "Piece fills a structural slot",
        region: BrainRegion::Perceptual, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} ממלא את החריץ {B}" },
    RelationDef { code: 0x07, name: "slot_of", description: "Structural slot of a whole",
        region: BrainRegion::Perceptual, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} חריץ של {B}" },
    RelationDef { code: 0x08, name: "replaces_part", description: "Overrides inherited part",
        region: BrainRegion::Perceptual, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} מחליף את {B}" },

    // ── Perceptual (0x09-0x0D) ──
    RelationDef { code: 0x09, name: "looks_like", description: "Visual similarity",
        region: BrainRegion::Perceptual, direction: Direction::Symmetric,
        transitivity: Transitivity::Sometimes, affinity: ModeAffinity::all(),
        hebrew_template: "{A} נראה כמו {B}" },
    RelationDef { code: 0x0A, name: "sounds_like", description: "Auditory similarity",
        region: BrainRegion::Perceptual, direction: Direction::Symmetric,
        transitivity: Transitivity::Sometimes, affinity: ModeAffinity::all(),
        hebrew_template: "{A} נשמע כמו {B}" },
    RelationDef { code: 0x0B, name: "moves_like", description: "Motion pattern similarity",
        region: BrainRegion::Perceptual, direction: Direction::Symmetric,
        transitivity: Transitivity::Sometimes, affinity: ModeAffinity::all(),
        hebrew_template: "{A} זז כמו {B}" },
    RelationDef { code: 0x0C, name: "abstracted_from", description: "Derived abstraction from raw media",
        region: BrainRegion::Perceptual, direction: Direction::Directed,
        transitivity: Transitivity::Sometimes, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} הופשט מ{B}" },
    RelationDef { code: 0x0D, name: "has_attribute", description: "Has a named attribute value",
        region: BrainRegion::CoreReality, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::all(),
        hebrew_template: "ל{A} יש {B}" },

    // ── Functional (0x0E-0x12) ──
    RelationDef { code: 0x0E, name: "used_for", description: "Typical function",
        region: BrainRegion::CoreReality, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} משמש ל{B}" },
    RelationDef { code: 0x0F, name: "affords", description: "What object enables actor to do",
        region: BrainRegion::CoreReality, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} מאפשר {B}" },
    RelationDef { code: 0x10, name: "enables", description: "Causal enabler of capability",
        region: BrainRegion::CoreReality, direction: Direction::Directed,
        transitivity: Transitivity::Sometimes, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} מאפשר {B}" },
    RelationDef { code: 0x11, name: "blocks", description: "Prevents action",
        region: BrainRegion::CoreReality, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} חוסם {B}" },
    RelationDef { code: 0x12, name: "requires", description: "Necessary condition",
        region: BrainRegion::CoreReality, direction: Direction::Directed,
        transitivity: Transitivity::Sometimes, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} דורש {B}" },

    // ── Causal (0x13-0x16) ──
    RelationDef { code: 0x13, name: "causes", description: "Direct causation",
        region: BrainRegion::CoreReality, direction: Direction::Directed,
        transitivity: Transitivity::Sometimes, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} גורם ל{B}" },
    RelationDef { code: 0x14, name: "prevents", description: "Stops outcome",
        region: BrainRegion::CoreReality, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} מונע {B}" },
    RelationDef { code: 0x15, name: "signals", description: "Weak indicator / symptom",
        region: BrainRegion::EventNarrative, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::all(),
        hebrew_template: "{A} מסמן {B}" },
    RelationDef { code: 0x16, name: "leads_to", description: "Indirect causal outcome",
        region: BrainRegion::CoreReality, direction: Direction::Directed,
        transitivity: Transitivity::Sometimes, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} מוביל ל{B}" },

    // ── Temporal (0x17-0x1A) ──
    RelationDef { code: 0x17, name: "before", description: "Temporal precedence",
        region: BrainRegion::EventNarrative, direction: Direction::Directed,
        transitivity: Transitivity::Always, affinity: ModeAffinity::story(),
        hebrew_template: "{A} לפני {B}" },
    RelationDef { code: 0x18, name: "after", description: "Temporal follower",
        region: BrainRegion::EventNarrative, direction: Direction::Directed,
        transitivity: Transitivity::Always, affinity: ModeAffinity::story(),
        hebrew_template: "{A} אחרי {B}" },
    RelationDef { code: 0x19, name: "during", description: "Overlapping time",
        region: BrainRegion::EventNarrative, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::story(),
        hebrew_template: "{A} במהלך {B}" },
    RelationDef { code: 0x1A, name: "narrative_before", description: "Earlier in story causation",
        region: BrainRegion::EventNarrative, direction: Direction::Directed,
        transitivity: Transitivity::Sometimes, affinity: ModeAffinity::story(),
        hebrew_template: "{A} קדם ל{B} בסיפור" },

    // ── Spatial (0x1B-0x1E) ──
    RelationDef { code: 0x1B, name: "located_in", description: "Contained spatially",
        region: BrainRegion::CoreReality, direction: Direction::Directed,
        transitivity: Transitivity::Always, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} נמצא ב{B}" },
    RelationDef { code: 0x1C, name: "near", description: "Spatial proximity",
        region: BrainRegion::Perceptual, direction: Direction::Symmetric,
        transitivity: Transitivity::Never, affinity: ModeAffinity::all(),
        hebrew_template: "{A} קרוב ל{B}" },
    RelationDef { code: 0x1D, name: "inside", description: "Enclosed within",
        region: BrainRegion::CoreReality, direction: Direction::Directed,
        transitivity: Transitivity::Always, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} בתוך {B}" },
    RelationDef { code: 0x1E, name: "visible_from", description: "Observable from vantage",
        region: BrainRegion::Perceptual, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::all(),
        hebrew_template: "{A} נראה מ{B}" },

    // ── Event / Narrative (0x1F-0x23) ──
    RelationDef { code: 0x1F, name: "agent_of", description: "Performs the action",
        region: BrainRegion::EventNarrative, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::story(),
        hebrew_template: "{A} מבצע את {B}" },
    RelationDef { code: 0x20, name: "patient_of", description: "Receives the action",
        region: BrainRegion::EventNarrative, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::story(),
        hebrew_template: "{A} חווה את {B}" },
    RelationDef { code: 0x21, name: "reacts_to", description: "Responds to event",
        region: BrainRegion::EventNarrative, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::story(),
        hebrew_template: "{A} מגיב ל{B}" },
    RelationDef { code: 0x22, name: "explains_reaction", description: "Connects reaction to cause",
        region: BrainRegion::EventNarrative, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::story(),
        hebrew_template: "{A} מסביר את התגובה ל{B}" },
    RelationDef { code: 0x23, name: "co_occurs_with", description: "Happens at the same scene",
        region: BrainRegion::EventNarrative, direction: Direction::Symmetric,
        transitivity: Transitivity::Never, affinity: ModeAffinity::all(),
        hebrew_template: "{A} מתרחש יחד עם {B}" },

    // ── Social Mind (0x24-0x29) ──
    RelationDef { code: 0x24, name: "cares_for", description: "Loves / watches over",
        region: BrainRegion::SocialMind, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::emotional(),
        hebrew_template: "{A} דואג ל{B}" },
    RelationDef { code: 0x25, name: "trusts", description: "Relies on",
        region: BrainRegion::SocialMind, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::emotional(),
        hebrew_template: "{A} בוטח ב{B}" },
    RelationDef { code: 0x26, name: "fears", description: "Perceives as threat",
        region: BrainRegion::SocialMind, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::emotional(),
        hebrew_template: "{A} מפחד מ{B}" },
    RelationDef { code: 0x27, name: "belongs_to_group", description: "Social group membership",
        region: BrainRegion::SocialMind, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::all(),
        hebrew_template: "{A} שייך ל{B}" },
    RelationDef { code: 0x28, name: "role_toward", description: "Social role relative to another",
        region: BrainRegion::SocialMind, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::all(),
        hebrew_template: "{A} משמש ב{B}" },
    RelationDef { code: 0x29, name: "imitates", description: "Behavioral imitation",
        region: BrainRegion::SocialMind, direction: Direction::Directed,
        transitivity: Transitivity::Sometimes, affinity: ModeAffinity::all(),
        hebrew_template: "{A} מחקה את {B}" },

    // ── Emotion / Appraisal (0x2A-0x31) — THE KEY REGION for AGI ──
    RelationDef { code: 0x2A, name: "emotion_triggered", description: "Event triggered this emotion",
        region: BrainRegion::EmotionAppraisal, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::emotional(),
        hebrew_template: "{A} מפעיל {B}" },
    RelationDef { code: 0x2B, name: "appraised_as_loss", description: "Event interpreted as loss",
        region: BrainRegion::EmotionAppraisal, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::emotional(),
        hebrew_template: "{A} מוערך כאובדן" },
    RelationDef { code: 0x2C, name: "appraised_as_threat", description: "Event interpreted as danger",
        region: BrainRegion::EmotionAppraisal, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::emotional(),
        hebrew_template: "{A} מוערך כאיום" },
    RelationDef { code: 0x2D, name: "appraised_as_opportunity", description: "Event interpreted as gain",
        region: BrainRegion::EmotionAppraisal, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::emotional(),
        hebrew_template: "{A} מוערך כהזדמנות" },
    RelationDef { code: 0x2E, name: "coping_capacity", description: "Ability to handle the event",
        region: BrainRegion::EmotionAppraisal, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::emotional(),
        hebrew_template: "{A} מאפשר התמודדות עם {B}" },
    RelationDef { code: 0x2F, name: "reappraised_as", description: "Emotion regulation by reframing",
        region: BrainRegion::EmotionAppraisal, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::emotional(),
        hebrew_template: "{A} הוערך מחדש כ{B}" },
    RelationDef { code: 0x30, name: "self_schema_triggered_by", description: "Event activates self-schema",
        region: BrainRegion::SelfSchema, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::emotional(),
        hebrew_template: "{A} מפעיל סכמת עצמי של {B}" },
    RelationDef { code: 0x31, name: "regulation_strategy_used", description: "Regulation applied to emotion",
        region: BrainRegion::GrowthTherapy, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::emotional(),
        hebrew_template: "{A} משתמש בוויסות {B}" },

    // ── Self-Schema / Identity (0x32-0x35) ──
    RelationDef { code: 0x32, name: "self_identifies_as", description: "Person sees self as X",
        region: BrainRegion::SelfSchema, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::emotional(),
        hebrew_template: "{A} מזדהה כ{B}" },
    RelationDef { code: 0x33, name: "core_belief", description: "Fundamental conviction",
        region: BrainRegion::SelfSchema, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::emotional(),
        hebrew_template: "אמונה יסוד של {A}: {B}" },
    RelationDef { code: 0x34, name: "value_attached_to", description: "Personal value on this concept",
        region: BrainRegion::SelfSchema, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::emotional(),
        hebrew_template: "{A} מעניק ערך ל{B}" },
    RelationDef { code: 0x35, name: "identity_threatened_by", description: "Weakens sense of self",
        region: BrainRegion::SelfSchema, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::emotional(),
        hebrew_template: "הזהות של {A} מאוימת על-ידי {B}" },

    // ── Growth / Therapy (0x36-0x38) ──
    RelationDef { code: 0x36, name: "regulated_by", description: "Emotion soothed by method",
        region: BrainRegion::GrowthTherapy, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::emotional(),
        hebrew_template: "{A} מוסדר על-ידי {B}" },
    RelationDef { code: 0x37, name: "coping_strategy_for", description: "Method to handle emotion",
        region: BrainRegion::GrowthTherapy, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::emotional(),
        hebrew_template: "{A} אסטרטגיית התמודדות עבור {B}" },
    RelationDef { code: 0x38, name: "improved_by_habit", description: "Grows through practice",
        region: BrainRegion::GrowthTherapy, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::all(),
        hebrew_template: "{A} משתפר דרך {B}" },

    // ── Creative / Association (0x39-0x3D) ──
    RelationDef { code: 0x39, name: "similar_to", description: "Semantic similarity",
        region: BrainRegion::Creative, direction: Direction::Symmetric,
        transitivity: Transitivity::Sometimes, affinity: ModeAffinity::all(),
        hebrew_template: "{A} דומה ל{B}" },
    RelationDef { code: 0x3A, name: "analogous_to", description: "Deep structural analogy",
        region: BrainRegion::Creative, direction: Direction::Symmetric,
        transitivity: Transitivity::Never, affinity: ModeAffinity::creative(),
        hebrew_template: "{A} אנלוגי ל{B}" },
    RelationDef { code: 0x3B, name: "remote_association_to", description: "Weak but valuable link",
        region: BrainRegion::Creative, direction: Direction::Symmetric,
        transitivity: Transitivity::Never, affinity: ModeAffinity::creative(),
        hebrew_template: "{A} מתקשר מרחוק ל{B}" },
    RelationDef { code: 0x3C, name: "conceptual_blend_of", description: "Novel combination of concepts",
        region: BrainRegion::Creative, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::creative(),
        hebrew_template: "{A} שילוב מושגי של {B}" },
    RelationDef { code: 0x3D, name: "metaphorically_maps_to", description: "Metaphorical mapping",
        region: BrainRegion::Creative, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::creative(),
        hebrew_template: "{A} מיפוי מטאפורי ל{B}" },

    // ── Meta-Cognition / Epistemic (0x3E-0x3F) ──
    RelationDef { code: 0x3E, name: "supported_by", description: "Evidence supporting claim",
        region: BrainRegion::MetaCognition, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} נתמך על-ידי {B}" },
    RelationDef { code: 0x3F, name: "contradicted_by", description: "Evidence against claim",
        region: BrainRegion::MetaCognition, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} נסתר על-ידי {B}" },

    // ── Persona / Social Identity (0x40-0x47) — added for rich person modeling ──
    RelationDef { code: 0x40, name: "has_age", description: "Age of a person in years",
        region: BrainRegion::SocialMind, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::precise(),
        hebrew_template: "ל{A} יש גיל {B}" },
    RelationDef { code: 0x41, name: "has_occupation", description: "Profession / job / role",
        region: BrainRegion::SocialMind, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} עובד כ{B}" },
    RelationDef { code: 0x42, name: "has_hobby", description: "Leisure pursuit",
        region: BrainRegion::SocialMind, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::all(),
        hebrew_template: "{A} נהנה מ{B}" },
    RelationDef { code: 0x43, name: "speaks_language", description: "Can speak a language",
        region: BrainRegion::SocialMind, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} מדבר {B}" },
    RelationDef { code: 0x44, name: "lives_in", description: "Residence location",
        region: BrainRegion::SocialMind, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::precise(),
        hebrew_template: "{A} גר ב{B}" },
    RelationDef { code: 0x45, name: "parent_of", description: "Parent-child relation",
        region: BrainRegion::SocialMind, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::story(),
        hebrew_template: "{A} הורה של {B}" },
    RelationDef { code: 0x46, name: "married_to", description: "Spouse relationship",
        region: BrainRegion::SocialMind, direction: Direction::Symmetric,
        transitivity: Transitivity::Never, affinity: ModeAffinity::story(),
        hebrew_template: "{A} נשוי ל{B}" },
    RelationDef { code: 0x47, name: "studied_at", description: "Educational institution",
        region: BrainRegion::SocialMind, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::story(),
        hebrew_template: "{A} למד ב{B}" },

    // ── Growth / Capability (0x48-0x4B) ──
    RelationDef { code: 0x48, name: "has_skill", description: "Person has acquired skill (weight = proficiency 0-100)",
        region: BrainRegion::GrowthTherapy, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::all(),
        hebrew_template: "{A} יודע {B}" },
    RelationDef { code: 0x49, name: "exercises_skill", description: "Using a skill right now",
        region: BrainRegion::GrowthTherapy, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::all(),
        hebrew_template: "{A} מפעיל {B}" },
    RelationDef { code: 0x4A, name: "teaches", description: "Can teach this skill/topic",
        region: BrainRegion::SocialMind, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::story(),
        hebrew_template: "{A} מלמד {B}" },
    RelationDef { code: 0x4B, name: "learning_goal", description: "Aspires to acquire this skill",
        region: BrainRegion::GrowthTherapy, direction: Direction::Directed,
        transitivity: Transitivity::Never, affinity: ModeAffinity::all(),
        hebrew_template: "{A} שואף ללמוד {B}" },
];

/// Lookup by code.
pub fn get(code: u8) -> Option<&'static RelationDef> {
    ALL_RELATIONS.iter().find(|r| r.code == code)
}

/// Lookup by name.
pub fn by_name(name: &str) -> Option<&'static RelationDef> {
    ALL_RELATIONS.iter().find(|r| r.name == name)
}

/// All relations in a given brain region.
pub fn by_region(region: BrainRegion) -> Vec<&'static RelationDef> {
    ALL_RELATIONS.iter().filter(|r| r.region == region).collect()
}

/// Relations preferred by a cognitive mode.
pub fn by_mode(f: impl Fn(&ModeAffinity) -> bool) -> Vec<&'static RelationDef> {
    ALL_RELATIONS.iter().filter(|r| f(&r.affinity)).collect()
}

/// Statistics about the registry.
pub struct RegistryStats {
    pub total: usize,
    pub by_region: HashMap<BrainRegion, usize>,
}

pub fn stats() -> RegistryStats {
    let mut by_region = HashMap::new();
    for r in ALL_RELATIONS {
        *by_region.entry(r.region).or_insert(0) += 1;
    }
    RegistryStats { total: ALL_RELATIONS.len(), by_region }
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seventy_six_relations_defined() {
        assert_eq!(ALL_RELATIONS.len(), 76, "must have exactly 76 relations (72 before + 4 growth/capability)");
    }

    #[test]
    fn all_codes_unique_and_sequential() {
        let mut seen = std::collections::HashSet::new();
        for r in ALL_RELATIONS {
            assert!(seen.insert(r.code), "duplicate code {} ({})", r.code, r.name);
            assert!(r.code < 128, "code {} exceeds 7-bit range", r.code);
        }
    }

    #[test]
    fn all_names_unique() {
        let mut seen = std::collections::HashSet::new();
        for r in ALL_RELATIONS {
            assert!(seen.insert(r.name), "duplicate name {}", r.name);
        }
    }

    #[test]
    fn lookup_by_code_works() {
        assert!(get(0x00).is_some());
        assert_eq!(get(0x00).unwrap().name, "is_a");
        assert_eq!(get(0x2A).unwrap().name, "emotion_triggered");
        assert!(get(128).is_none()); // 7-bit range
        assert!(get(0x40).is_some());
        assert_eq!(get(0x40).unwrap().name, "has_age");
    }

    #[test]
    fn lookup_by_name_works() {
        assert_eq!(by_name("is_a").unwrap().code, 0x00);
        assert_eq!(by_name("appraised_as_loss").unwrap().code, 0x2B);
        assert!(by_name("nonexistent").is_none());
    }

    #[test]
    fn brain_regions_covered() {
        let s = stats();
        // Every brain region must have at least one relation
        for region in [BrainRegion::CoreReality, BrainRegion::Perceptual,
                       BrainRegion::EventNarrative, BrainRegion::SocialMind,
                       BrainRegion::EmotionAppraisal, BrainRegion::SelfSchema,
                       BrainRegion::GrowthTherapy, BrainRegion::Creative,
                       BrainRegion::MetaCognition] {
            assert!(s.by_region.get(&region).is_some(),
                "region {:?} has no relations", region);
        }
    }

    #[test]
    fn emotion_region_has_rich_coverage() {
        let emo = by_region(BrainRegion::EmotionAppraisal);
        // Appraisal theory requires at least: loss, threat, opportunity,
        // coping, emotion_triggered, reappraised_as
        assert!(emo.len() >= 6, "emotion/appraisal region underpopulated");
    }

    #[test]
    fn precision_mode_filters_subset() {
        let p = by_mode(|a| a.precision);
        let d = by_mode(|a| a.divergent);
        assert!(p.len() > 0);
        assert!(d.len() > 0);
        // Precision should NOT include remote_association_to
        assert!(!p.iter().any(|r| r.name == "remote_association_to"));
        // Divergent should
        assert!(d.iter().any(|r| r.name == "remote_association_to"));
    }

    #[test]
    fn is_a_is_transitive() {
        assert_eq!(by_name("is_a").unwrap().transitivity, Transitivity::Always);
        assert_eq!(by_name("before").unwrap().transitivity, Transitivity::Always);
        // near is NOT transitive
        assert_eq!(by_name("near").unwrap().transitivity, Transitivity::Never);
    }

    #[test]
    fn directed_vs_symmetric() {
        assert_eq!(by_name("is_a").unwrap().direction, Direction::Directed);
        assert_eq!(by_name("similar_to").unwrap().direction, Direction::Symmetric);
        assert_eq!(by_name("near").unwrap().direction, Direction::Symmetric);
    }
}
