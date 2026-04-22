//! Learning Layer — cognitive training vs asserted knowledge.
//!
//! Idan's insight (22 Apr 2026): ZETS today treats every ingested atom as
//! "asserted fact". But when ingesting 1000 Reddit posts to learn "sad
//! expression patterns", the posts themselves aren't facts — they're
//! training data. Humans handle this via episodic/semantic memory split
//! (Tulving 1972). ZETS needs the same.
//!
//! This module is PHASE A: ADDITIVE. It does NOT modify AtomKind, AtomEdge,
//! or atom_persist format. It adds:
//!
//!   - Provenance tag per edge: Asserted | Observed | Learned | Hypothesis
//!     stored in side-car HashMap<EdgeKey, ProvenanceRecord>
//!   - Prototype / Pattern / Exemplar wrapper types over existing atoms
//!   - ProtoBank — registry of learned prototypes
//!
//! Once Phase A is validated, Phase B will migrate to first-class enum
//! variants in AtomKind (format_version: 2).
//!
//! Validated by triangulation with Gemini Pro + Groq + Gemini Flash.
//! Strategy doc: docs/working/20260422_learning_layer_V1.md

use std::collections::HashMap;

use crate::atoms::AtomId;

/// How this piece of knowledge came to be in the graph.
///
/// Critical: this is a PROPERTY of an edge, not a graph partition.
/// The same atom can be reached by edges of different provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provenance {
    /// Stated as truth — from a textbook, encyclopedia, or user assertion.
    /// Trusted for Precision mode reasoning.
    Asserted = 0,

    /// Happened once in training data — a specific dialogue turn,
    /// one image, one Reddit post. Not a claim about truth, just data.
    Observed = 1,

    /// Generalization distilled from N observations.
    /// Has a confidence score; gains trust as N grows and drift shrinks.
    Learned = 2,

    /// Proposed by dreaming/exploration, not yet verified.
    /// Used by Divergent mode; filtered out by Precision mode.
    Hypothesis = 3,
}

impl Provenance {
    pub fn as_u8(self) -> u8 { self as u8 }

    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Asserted),
            1 => Some(Self::Observed),
            2 => Some(Self::Learned),
            3 => Some(Self::Hypothesis),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Asserted => "asserted",
            Self::Observed => "observed",
            Self::Learned => "learned",
            Self::Hypothesis => "hypothesis",
        }
    }
}

/// Key for looking up edge metadata. We use (from, to, relation) triple
/// since the AtomEdge struct doesn't have a stable edge-id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeKey {
    pub from: AtomId,
    pub to: AtomId,
    pub relation: u8,
}

impl EdgeKey {
    pub fn new(from: AtomId, to: AtomId, relation: u8) -> Self {
        Self { from, to, relation }
    }
}


/// Register / formality level for an edge (see docs/working/20260422_formality_diglossia_design.md).
///
/// Strict-register languages (Arabic MSA vs colloquial) MUST NOT mix levels on
/// the same walk. Non-strict languages (Hebrew, English) may mix but prefer
/// same-register edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RegisterLevel {
    /// Sacred / religious texts (Tanakh, Quran, Vedas).
    Sacred = 0,
    /// Literary / poetic (Shakespeare, Bialik, classical Arabic fusha).
    Literary = 1,
    /// Formal / journalistic / academic.
    Formal = 2,
    /// Neutral — default. Works in most contexts.
    Neutral = 3,
    /// Colloquial — everyday spoken / WhatsApp-level.
    Colloquial = 4,
    /// Slang — rapidly-changing informal terms.
    Slang = 5,
    /// Child-friendly — baby talk, simplified terms.
    Child = 6,
}

impl RegisterLevel {
    pub fn as_u8(self) -> u8 { self as u8 }
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Sacred),
            1 => Some(Self::Literary),
            2 => Some(Self::Formal),
            3 => Some(Self::Neutral),
            4 => Some(Self::Colloquial),
            5 => Some(Self::Slang),
            6 => Some(Self::Child),
            _ => None,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Sacred => "sacred",
            Self::Literary => "literary",
            Self::Formal => "formal",
            Self::Neutral => "neutral",
            Self::Colloquial => "colloquial",
            Self::Slang => "slang",
            Self::Child => "child",
        }
    }
    /// Is this register considered safe for children?
    /// Child mode (Yam, Ben) filters out registers beyond this threshold.
    pub fn is_child_safe(self) -> bool {
        // Sacred/Literary/Formal/Neutral/Child — all ok. Colloquial borderline. Slang excluded.
        (self as u8) != Self::Slang as u8
    }
    /// Is this register suitable for formal output (journalism, contracts)?
    pub fn is_formal_appropriate(self) -> bool {
        matches!(self, Self::Sacred | Self::Literary | Self::Formal | Self::Neutral)
    }
}

impl Default for RegisterLevel {
    fn default() -> Self { Self::Neutral }
}

/// Provenance + confidence for a single edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProvenanceRecord {
    pub provenance: Provenance,
    /// 0-255: how confident are we in this edge's validity?
    /// For Asserted: usually 240+ (trusted source).
    /// For Observed: usually 100-200 (one observation).
    /// For Learned: proportional to cluster size + stability (e.g., 200+ if stable).
    /// For Hypothesis: 50-150 (dreaming quality).
    pub confidence: u8,
    /// Register / formality level. Defaults to Neutral.
    pub register: RegisterLevel,
}

impl ProvenanceRecord {
    pub fn asserted() -> Self {
        Self { provenance: Provenance::Asserted, confidence: 240, register: RegisterLevel::Neutral }
    }

    pub fn asserted_with_register(register: RegisterLevel) -> Self {
        Self { provenance: Provenance::Asserted, confidence: 240, register }
    }
    pub fn observed() -> Self {
        Self { provenance: Provenance::Observed, confidence: 128, register: RegisterLevel::Neutral }
    }
    pub fn learned(confidence: u8) -> Self {
        Self { provenance: Provenance::Learned, confidence, register: RegisterLevel::Neutral }
    }
    pub fn hypothesis() -> Self {
        Self { provenance: Provenance::Hypothesis, confidence: 100, register: RegisterLevel::Neutral }
    }
}

/// Side-car store for edge provenance. Kept separate from AtomStore so
/// we don't break existing persistence format during Phase A.
#[derive(Debug, Default, Clone)]
pub struct ProvenanceLog {
    records: HashMap<EdgeKey, ProvenanceRecord>,
}

impl ProvenanceLog {
    pub fn new() -> Self { Self::default() }

    pub fn tag(&mut self, key: EdgeKey, record: ProvenanceRecord) {
        self.records.insert(key, record);
    }

    pub fn get(&self, key: &EdgeKey) -> Option<&ProvenanceRecord> {
        self.records.get(key)
    }

    /// Get provenance with a default — if edge was tagged, return that;
    /// otherwise treat untagged edges as Asserted (backward compat with
    /// pre-learning-layer atoms).
    pub fn get_or_asserted(&self, key: &EdgeKey) -> ProvenanceRecord {
        self.records.get(key).copied().unwrap_or_else(ProvenanceRecord::asserted)
    }

    pub fn len(&self) -> usize { self.records.len() }

    pub fn is_empty(&self) -> bool { self.records.is_empty() }

    /// Filter edges by provenance type (e.g., for Precision mode).
    pub fn filter<F>(&self, predicate: F) -> Vec<(EdgeKey, ProvenanceRecord)>
    where F: Fn(&ProvenanceRecord) -> bool {
        self.records.iter()
            .filter(|(_, r)| predicate(r))
            .map(|(k, r)| (*k, *r))
            .collect()
    }

    /// Count by provenance kind — useful for audit.
    pub fn counts(&self) -> HashMap<Provenance, usize> {
        let mut counts = HashMap::new();
        for record in self.records.values() {
            *counts.entry(record.provenance).or_insert(0) += 1;
        }
        counts
    }
}

// ────────────────────────────────────────────────────────────────
// Prototype — the Platonic ideal of a learned pattern
// ────────────────────────────────────────────────────────────────

/// A learned cluster centroid, stored as a wrapper over an AtomId.
///
/// In Phase A we don't have FeatureVector atoms in AtomKind yet, so
/// the "centroid" is represented as metadata in the ProtoBank.
/// Phase B will make this a first-class AtomKind::Prototype.
#[derive(Debug, Clone)]
pub struct Prototype {
    /// The atom that represents this prototype (Concept atom for now)
    pub atom_id: AtomId,
    /// Human-readable name: "sadness", "dialogue:greeting", "visual:labrador"
    pub name: String,
    /// Domain: "emotion", "dialogue_pattern", "visual_concept", ...
    pub domain: String,
    /// How many observations contributed to this prototype
    pub observation_count: u32,
    /// Cluster drift — 0 = stable, higher = moving. Threshold for merge/split.
    pub drift: f32,
    /// 2-3 preserved canonical example atoms (not deleted during distillation)
    pub exemplars: Vec<AtomId>,
    /// ISO date when prototype was last redistilled
    pub last_distilled: String,
}

/// Registry of all learned prototypes in the graph.
#[derive(Debug, Default, Clone)]
pub struct ProtoBank {
    prototypes: Vec<Prototype>,
}

impl ProtoBank {
    pub fn new() -> Self { Self::default() }

    pub fn register(&mut self, proto: Prototype) -> usize {
        self.prototypes.push(proto);
        self.prototypes.len() - 1
    }

    pub fn get(&self, idx: usize) -> Option<&Prototype> { self.prototypes.get(idx) }

    pub fn by_name(&self, name: &str) -> Option<&Prototype> {
        self.prototypes.iter().find(|p| p.name == name)
    }

    pub fn by_domain(&self, domain: &str) -> Vec<&Prototype> {
        self.prototypes.iter().filter(|p| p.domain == domain).collect()
    }

    pub fn len(&self) -> usize { self.prototypes.len() }

    pub fn is_empty(&self) -> bool { self.prototypes.is_empty() }

    /// Find the most similar prototype to a given atom (by observation_count as crude proxy
    /// for popularity — Phase B will use real feature-vector similarity).
    pub fn nearest_in_domain(&self, domain: &str) -> Option<&Prototype> {
        self.by_domain(domain).into_iter()
            .max_by_key(|p| p.observation_count)
    }
}

// ────────────────────────────────────────────────────────────────
// Pattern — dialogue template with slots
// ────────────────────────────────────────────────────────────────

/// A learned dialogue pattern: "I just want to [VERB]"
#[derive(Debug, Clone)]
pub struct DialoguePattern {
    /// Atom representing this pattern
    pub atom_id: AtomId,
    /// Template text with slot markers: "I just want to [VERB]"
    pub template: String,
    /// Slot names in order: ["VERB"]
    pub slots: Vec<String>,
    /// Inferred intent: "request", "complaint", "empathize", "deflect"
    pub intent: String,
    /// Typical emotional context: prototype id in ProtoBank
    pub typical_emotion_proto: Option<usize>,
    /// Number of observations supporting this pattern
    pub observation_count: u32,
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::AtomStore;

    #[test]
    fn provenance_enum_roundtrip() {
        for p in [Provenance::Asserted, Provenance::Observed,
                   Provenance::Learned, Provenance::Hypothesis] {
            let b = p.as_u8();
            let back = Provenance::from_u8(b).unwrap();
            assert_eq!(p, back);
        }
    }

    #[test]
    fn provenance_labels() {
        assert_eq!(Provenance::Asserted.label(), "asserted");
        assert_eq!(Provenance::Observed.label(), "observed");
        assert_eq!(Provenance::Learned.label(), "learned");
        assert_eq!(Provenance::Hypothesis.label(), "hypothesis");
    }

    #[test]
    fn provenance_log_tag_and_get() {
        let mut log = ProvenanceLog::new();
        let key = EdgeKey::new(1, 2, 0x10);
        log.tag(key, ProvenanceRecord::learned(210));

        let record = log.get(&key).unwrap();
        assert_eq!(record.provenance, Provenance::Learned);
        assert_eq!(record.confidence, 210);
    }

    #[test]
    fn get_or_asserted_default() {
        let log = ProvenanceLog::new();
        let key = EdgeKey::new(99, 100, 0x20);
        // Untagged edge returns Asserted by default (backward compat)
        let record = log.get_or_asserted(&key);
        assert_eq!(record.provenance, Provenance::Asserted);
        assert_eq!(record.confidence, 240);
    }

    #[test]
    fn provenance_log_counts() {
        let mut log = ProvenanceLog::new();
        log.tag(EdgeKey::new(1, 2, 0), ProvenanceRecord::asserted());
        log.tag(EdgeKey::new(2, 3, 0), ProvenanceRecord::observed());
        log.tag(EdgeKey::new(3, 4, 0), ProvenanceRecord::observed());
        log.tag(EdgeKey::new(4, 5, 0), ProvenanceRecord::learned(180));
        log.tag(EdgeKey::new(5, 6, 0), ProvenanceRecord::hypothesis());

        let counts = log.counts();
        assert_eq!(counts.get(&Provenance::Asserted), Some(&1));
        assert_eq!(counts.get(&Provenance::Observed), Some(&2));
        assert_eq!(counts.get(&Provenance::Learned), Some(&1));
        assert_eq!(counts.get(&Provenance::Hypothesis), Some(&1));
    }

    #[test]
    fn filter_by_provenance() {
        let mut log = ProvenanceLog::new();
        log.tag(EdgeKey::new(1, 2, 0), ProvenanceRecord::asserted());
        log.tag(EdgeKey::new(2, 3, 0), ProvenanceRecord::observed());
        log.tag(EdgeKey::new(3, 4, 0), ProvenanceRecord::learned(220));

        let precision_mode: Vec<_> = log.filter(|r|
            r.provenance == Provenance::Asserted
            || (r.provenance == Provenance::Learned && r.confidence >= 200)
        );
        assert_eq!(precision_mode.len(), 2);

        let divergent_mode: Vec<_> = log.filter(|r|
            r.provenance != Provenance::Asserted
        );
        assert_eq!(divergent_mode.len(), 2);
    }

    #[test]
    fn proto_bank_register_and_query() {
        let mut bank = ProtoBank::new();
        let sad_idx = bank.register(Prototype {
            atom_id: 42,
            name: "sadness".to_string(),
            domain: "emotion".to_string(),
            observation_count: 87,
            drift: 0.03,
            exemplars: vec![10, 11, 12],
            last_distilled: "2026-04-22".to_string(),
        });
        let joy_idx = bank.register(Prototype {
            atom_id: 43,
            name: "joy".to_string(),
            domain: "emotion".to_string(),
            observation_count: 124,
            drift: 0.02,
            exemplars: vec![20, 21],
            last_distilled: "2026-04-22".to_string(),
        });

        assert_eq!(sad_idx, 0);
        assert_eq!(joy_idx, 1);
        assert_eq!(bank.len(), 2);

        let sad = bank.by_name("sadness").unwrap();
        assert_eq!(sad.observation_count, 87);

        let emotions = bank.by_domain("emotion");
        assert_eq!(emotions.len(), 2);

        // Nearest (by count) in emotion domain = joy (124 > 87)
        let nearest = bank.nearest_in_domain("emotion").unwrap();
        assert_eq!(nearest.name, "joy");
    }

    #[test]
    fn proto_bank_empty_queries() {
        let bank = ProtoBank::new();
        assert!(bank.is_empty());
        assert_eq!(bank.len(), 0);
        assert!(bank.by_name("anything").is_none());
        assert!(bank.by_domain("any").is_empty());
        assert!(bank.nearest_in_domain("any").is_none());
    }

    #[test]
    fn dialogue_pattern_basic() {
        let pat = DialoguePattern {
            atom_id: 100,
            template: "I just want to [VERB]".to_string(),
            slots: vec!["VERB".to_string()],
            intent: "request".to_string(),
            typical_emotion_proto: Some(0),
            observation_count: 45,
        };
        assert_eq!(pat.slots.len(), 1);
        assert_eq!(pat.intent, "request");
    }

    #[test]
    fn provenance_record_constructors() {
        let a = ProvenanceRecord::asserted();
        assert_eq!(a.provenance, Provenance::Asserted);
        assert_eq!(a.confidence, 240);

        let o = ProvenanceRecord::observed();
        assert_eq!(o.provenance, Provenance::Observed);

        let l = ProvenanceRecord::learned(200);
        assert_eq!(l.provenance, Provenance::Learned);
        assert_eq!(l.confidence, 200);

        let h = ProvenanceRecord::hypothesis();
        assert_eq!(h.provenance, Provenance::Hypothesis);
    }

    #[test]
    fn edge_key_equality() {
        let k1 = EdgeKey::new(1, 2, 0x10);
        let k2 = EdgeKey::new(1, 2, 0x10);
        let k3 = EdgeKey::new(1, 2, 0x11);  // different relation
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
    }

    #[test]
    fn mixed_provenance_workflow() {
        // Simulate a dialogue ingestion workflow:
        // 1. Assert a fact: "sadness is an emotion"
        // 2. Observe: "user said 'I lost my job'"
        // 3. Learn: "job_loss correlates with sadness" (from 50 obs)
        // 4. Hypothesize: "offering comfort reduces sadness" (dreaming)

        let mut store = AtomStore::new();
        let mut log = ProvenanceLog::new();

        let sadness = store.put(crate::atoms::AtomKind::Concept, b"emotion:sadness".to_vec());
        let emotion = store.put(crate::atoms::AtomKind::Concept, b"category:emotion".to_vec());
        let utterance = store.put(crate::atoms::AtomKind::Text, b"I lost my job".to_vec());
        let job_loss = store.put(crate::atoms::AtomKind::Concept, b"event:job_loss".to_vec());
        let comfort = store.put(crate::atoms::AtomKind::Concept, b"action:comfort".to_vec());

        // 1. Asserted: sadness is_a emotion
        let is_a = crate::relations::by_name("is_a").unwrap().code;
        store.link(sadness, emotion, is_a, 95, 0);
        log.tag(EdgeKey::new(sadness, emotion, is_a), ProvenanceRecord::asserted());

        // 2. Observed: utterance expresses sadness
        let expresses = crate::relations::by_name("expresses_emotion")
            .map(|r| r.code).unwrap_or(is_a);
        store.link(utterance, sadness, expresses, 70, 0);
        log.tag(EdgeKey::new(utterance, sadness, expresses), ProvenanceRecord::observed());

        // 3. Learned: job_loss correlates_with sadness
        let correlates = crate::relations::by_name("co_occurs_with").unwrap().code;
        store.link(job_loss, sadness, correlates, 80, 0);
        log.tag(EdgeKey::new(job_loss, sadness, correlates), ProvenanceRecord::learned(210));

        // 4. Hypothesis: comfort reduces sadness
        let near = crate::relations::by_name("near").unwrap().code;
        store.link(comfort, sadness, near, 60, 0);
        log.tag(EdgeKey::new(comfort, sadness, near), ProvenanceRecord::hypothesis());

        // Audit:
        let counts = log.counts();
        assert_eq!(counts.get(&Provenance::Asserted), Some(&1));
        assert_eq!(counts.get(&Provenance::Observed), Some(&1));
        assert_eq!(counts.get(&Provenance::Learned), Some(&1));
        assert_eq!(counts.get(&Provenance::Hypothesis), Some(&1));

        // Precision mode: only Asserted + high-confidence Learned
        let precision_edges = log.filter(|r|
            r.provenance == Provenance::Asserted
            || (r.provenance == Provenance::Learned && r.confidence >= 200)
        );
        assert_eq!(precision_edges.len(), 2);  // asserted + learned(210)
    }
}
