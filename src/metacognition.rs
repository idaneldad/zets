//! Meta-cognition — ZETS reasons about its own knowledge.
//!
//! Core AGI property: knowing what you DON'T know.
//!
//! Provides:
//!   - ConfidenceLevel (how sure we are about an answer)
//!   - KnowledgeGap (things we identified as unknown)
//!   - LearningProposal (candidates for the sandbox)
//!
//! Idea: when a query returns with low confidence or "not found",
//! we register a gap. The sandbox can then prioritize learning to
//! fill these gaps — active learning driven by experience.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Discrete confidence levels — avoids spurious precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Confidence {
    /// "No idea" — 0-20
    Unknown = 0,
    /// "Guess" — 20-40
    Weak = 1,
    /// "Probably" — 40-60
    Moderate = 2,
    /// "Pretty sure" — 60-80
    Strong = 3,
    /// "Certain, multiple sources agree" — 80-100
    Certain = 4,
}

impl Confidence {
    pub fn from_score(score: u8) -> Self {
        match score {
            0..=20 => Self::Unknown,
            21..=40 => Self::Weak,
            41..=60 => Self::Moderate,
            61..=80 => Self::Strong,
            _ => Self::Certain,
        }
    }
    pub fn as_score(self) -> u8 {
        match self {
            Self::Unknown => 10,
            Self::Weak => 30,
            Self::Moderate => 50,
            Self::Strong => 70,
            Self::Certain => 90,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Weak => "weak",
            Self::Moderate => "moderate",
            Self::Strong => "strong",
            Self::Certain => "certain",
        }
    }
}

/// A detected gap — something we don't know or aren't confident about.
#[derive(Debug, Clone)]
pub struct KnowledgeGap {
    pub topic: String,
    pub query_text: String,
    pub confidence: Confidence,
    pub detected_at_ms: u64,
    pub occurrence_count: u32,
    pub kind: GapKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GapKind {
    /// We had no concept for this.
    NoConceptFound,
    /// We had the concept but no relevant edges.
    NoEdgesFound,
    /// Multiple contradictory answers from different sources.
    Contradiction { sources: Vec<String> },
    /// Confidence was below threshold.
    LowConfidence,
    /// We were asked to reason and exceeded recursion depth.
    ReasoningDepthExceeded,
}

/// A proposal for what to learn next, based on detected gaps.
#[derive(Debug, Clone)]
pub struct LearningProposal {
    pub proposal_id: String,
    pub target_topic: String,
    pub justification: String,
    pub priority: u8, // 0-100, higher = more important
    pub suggested_sources: Vec<String>,
}

/// The metacognitive state of ZETS — tracks gaps, proposes learning.
pub struct MetaCognition {
    gaps: HashMap<String, KnowledgeGap>,
    proposals: Vec<LearningProposal>,
    query_count: u64,
    low_confidence_count: u64,
    not_found_count: u64,
}

impl MetaCognition {
    pub fn new() -> Self {
        Self {
            gaps: HashMap::new(),
            proposals: Vec::new(),
            query_count: 0,
            low_confidence_count: 0,
            not_found_count: 0,
        }
    }

    /// Register a query outcome. Metacognition decides if it warrants a gap.
    pub fn observe(&mut self, query: &str, confidence: Confidence, kind: GapKind) {
        self.query_count += 1;

        let should_register = match &kind {
            GapKind::NoConceptFound => { self.not_found_count += 1; true }
            GapKind::NoEdgesFound => true,
            GapKind::Contradiction { .. } => true,
            GapKind::LowConfidence if confidence <= Confidence::Weak => {
                self.low_confidence_count += 1;
                true
            }
            GapKind::ReasoningDepthExceeded => true,
            _ => false,
        };

        if !should_register { return; }

        let topic = extract_topic(query);
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        self.gaps
            .entry(topic.clone())
            .and_modify(|g| g.occurrence_count += 1)
            .or_insert(KnowledgeGap {
                topic,
                query_text: query.to_string(),
                confidence,
                detected_at_ms: ts,
                occurrence_count: 1,
                kind,
            });
    }

    /// Pick the top N gaps to propose learning for.
    pub fn propose_learning(&mut self, top_n: usize) -> Vec<LearningProposal> {
        let mut gaps: Vec<&KnowledgeGap> = self.gaps.values().collect();
        // Priority: recurring gaps first, then recency
        gaps.sort_by(|a, b| {
            b.occurrence_count
                .cmp(&a.occurrence_count)
                .then(b.detected_at_ms.cmp(&a.detected_at_ms))
        });

        let mut proposals = Vec::new();
        for (i, gap) in gaps.iter().take(top_n).enumerate() {
            let priority = ((gap.occurrence_count as u32).min(100) as u8).max(10);
            let justification = match &gap.kind {
                GapKind::NoConceptFound => {
                    format!("Concept '{}' was requested {} times but not found.",
                        gap.topic, gap.occurrence_count)
                }
                GapKind::NoEdgesFound => {
                    format!("'{}' exists but has no semantic edges — isolated node.",
                        gap.topic)
                }
                GapKind::Contradiction { sources } => {
                    format!("Conflicting info about '{}' from: {}",
                        gap.topic, sources.join(", "))
                }
                GapKind::LowConfidence => {
                    format!("Low confidence answers for '{}' ({} occurrences).",
                        gap.topic, gap.occurrence_count)
                }
                GapKind::ReasoningDepthExceeded => {
                    format!("Reasoning about '{}' hit depth limit — need more direct edges.",
                        gap.topic)
                }
            };

            proposals.push(LearningProposal {
                proposal_id: format!("prop-{}", i + 1),
                target_topic: gap.topic.clone(),
                justification,
                priority,
                suggested_sources: default_sources_for(&gap.topic),
            });
        }
        self.proposals = proposals.clone();
        proposals
    }

    pub fn stats(&self) -> MetaStats {
        MetaStats {
            total_queries: self.query_count,
            total_gaps: self.gaps.len(),
            low_confidence_rate: if self.query_count == 0 {
                0.0
            } else {
                self.low_confidence_count as f64 / self.query_count as f64
            },
            not_found_rate: if self.query_count == 0 {
                0.0
            } else {
                self.not_found_count as f64 / self.query_count as f64
            },
        }
    }

    pub fn gaps(&self) -> Vec<&KnowledgeGap> {
        let mut v: Vec<_> = self.gaps.values().collect();
        v.sort_by(|a, b| b.occurrence_count.cmp(&a.occurrence_count));
        v
    }

    pub fn clear(&mut self) {
        self.gaps.clear();
        self.proposals.clear();
        self.query_count = 0;
        self.low_confidence_count = 0;
        self.not_found_count = 0;
    }
}

impl Default for MetaCognition {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone)]
pub struct MetaStats {
    pub total_queries: u64,
    pub total_gaps: usize,
    pub low_confidence_rate: f64,
    pub not_found_rate: f64,
}

fn extract_topic(query: &str) -> String {
    // Trivial heuristic: take first few words.
    // In real ZETS, this would be a system graph route.
    let cleaned = query.trim().trim_end_matches('?').trim_end_matches('.');
    cleaned.split_whitespace().take(3).collect::<Vec<_>>().join(" ")
}

fn default_sources_for(_topic: &str) -> Vec<String> {
    vec![
        "wikipedia".to_string(),
        "wiktionary".to_string(),
        "curated_domain_bundle".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_bucketing() {
        assert_eq!(Confidence::from_score(0), Confidence::Unknown);
        assert_eq!(Confidence::from_score(30), Confidence::Weak);
        assert_eq!(Confidence::from_score(50), Confidence::Moderate);
        assert_eq!(Confidence::from_score(70), Confidence::Strong);
        assert_eq!(Confidence::from_score(95), Confidence::Certain);
    }

    #[test]
    fn observing_not_found_creates_gap() {
        let mut mc = MetaCognition::new();
        mc.observe("what is phlogiston?", Confidence::Unknown, GapKind::NoConceptFound);
        assert_eq!(mc.gaps().len(), 1);
    }

    #[test]
    fn repeated_queries_increment_counter() {
        let mut mc = MetaCognition::new();
        for _ in 0..5 {
            mc.observe("what is aether?", Confidence::Unknown, GapKind::NoConceptFound);
        }
        let gap = &mc.gaps()[0];
        assert_eq!(gap.occurrence_count, 5);
    }

    #[test]
    fn low_confidence_registered() {
        let mut mc = MetaCognition::new();
        mc.observe("what is X?", Confidence::Weak, GapKind::LowConfidence);
        assert_eq!(mc.gaps().len(), 1);
    }

    #[test]
    fn strong_confidence_not_a_gap() {
        let mut mc = MetaCognition::new();
        mc.observe("what is DNA?", Confidence::Strong, GapKind::LowConfidence);
        // Strong confidence means no gap
        assert_eq!(mc.gaps().len(), 0);
    }

    #[test]
    fn propose_learning_prioritizes_recurring_gaps() {
        let mut mc = MetaCognition::new();
        mc.observe("A?", Confidence::Unknown, GapKind::NoConceptFound);
        for _ in 0..10 {
            mc.observe("B?", Confidence::Unknown, GapKind::NoConceptFound);
        }
        mc.observe("C?", Confidence::Unknown, GapKind::NoConceptFound);
        let proposals = mc.propose_learning(3);
        // B should be first (highest occurrence count)
        assert_eq!(proposals[0].target_topic, "B");
        assert!(proposals[0].priority >= 10);
    }

    #[test]
    fn stats_track_correctly() {
        let mut mc = MetaCognition::new();
        mc.observe("A?", Confidence::Unknown, GapKind::NoConceptFound);
        mc.observe("B?", Confidence::Weak, GapKind::LowConfidence);
        mc.observe("C?", Confidence::Strong, GapKind::LowConfidence);
        let s = mc.stats();
        assert_eq!(s.total_queries, 3);
        assert_eq!(s.total_gaps, 2);  // C doesn't register
        assert!(s.not_found_rate > 0.0);
    }
}
