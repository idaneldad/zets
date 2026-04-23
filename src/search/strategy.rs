//! Search strategies — behavioral parameter sets for persona-adaptive search.
//!
//! Labels are BEHAVIORAL, not diagnostic. Per Gemini's explicit critique
//! (docs/working/20260423_neural_tree_7x7_consultation_V1.md):
//! "Do NOT use diagnostic labels (ADHD, Autistic) for engineering abstractions.
//! Abstract to the desired search behavior or information processing style."

use std::collections::HashMap;

/// Behavioral label for a search strategy.
/// NEVER contains diagnostic terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrategyLabel {
    /// Narrow beam, high-confidence only. For factual queries.
    Precise,
    /// Wide beam, divergent. For creative/research queries.
    Exploratory,
    /// Cautious multi-path. For medical/safety-critical queries.
    Exhaustive,
    /// Broad shallow, many retries. For brainstorming.
    RapidIteration,
    /// Narrow but deep. For detailed analysis on known path.
    DeepDive,
    /// Default balanced. Miller's 7±2 × 7 depth.
    Standard7x7,
    /// Custom, user-defined.
    Custom,
}

impl StrategyLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            StrategyLabel::Precise => "precise",
            StrategyLabel::Exploratory => "exploratory",
            StrategyLabel::Exhaustive => "exhaustive",
            StrategyLabel::RapidIteration => "rapid_iteration",
            StrategyLabel::DeepDive => "deep_dive",
            StrategyLabel::Standard7x7 => "standard_7x7",
            StrategyLabel::Custom => "custom",
        }
    }
}

/// Parameters for a parallelized beam search.
#[derive(Debug, Clone)]
pub struct SearchStrategy {
    pub label: StrategyLabel,
    /// Parallel walks per step (capped at super::MAX_BEAM_WIDTH).
    pub beam_width: usize,
    /// Hop limit (capped at super::MAX_SEARCH_DEPTH).
    pub max_depth: usize,
    /// If no answer found, restart with new seeds this many times.
    pub retry_waves: usize,
    /// 0.0-1.0. Higher = stop only on very confident match.
    pub confidence_threshold: f32,
    pub description: &'static str,
}

impl SearchStrategy {
    /// Clamp all fields to safe bounds.
    pub fn clamped(mut self) -> Self {
        self.beam_width = self.beam_width.clamp(1, super::MAX_BEAM_WIDTH);
        self.max_depth = self.max_depth.clamp(1, super::MAX_SEARCH_DEPTH);
        self.retry_waves = self.retry_waves.clamp(1, 10);
        self.confidence_threshold = self.confidence_threshold.clamp(0.0, 1.0);
        self
    }
}

/// Built-in strategies, keyed by label.
pub fn default_strategies() -> HashMap<StrategyLabel, SearchStrategy> {
    let mut m = HashMap::new();
    m.insert(StrategyLabel::Precise, SearchStrategy {
        label: StrategyLabel::Precise,
        beam_width: 3, max_depth: 5, retry_waves: 1,
        confidence_threshold: 0.9,
        description: "Narrow beam, high-confidence only. For factual queries.",
    });
    m.insert(StrategyLabel::Exploratory, SearchStrategy {
        label: StrategyLabel::Exploratory,
        beam_width: 12, max_depth: 9, retry_waves: 3,
        confidence_threshold: 0.6,
        description: "Wide beam, divergent. For creative/research queries.",
    });
    m.insert(StrategyLabel::Exhaustive, SearchStrategy {
        label: StrategyLabel::Exhaustive,
        beam_width: 7, max_depth: 12, retry_waves: 5,
        confidence_threshold: 0.5,
        description: "Cautious multi-path. For medical/safety-critical queries.",
    });
    m.insert(StrategyLabel::RapidIteration, SearchStrategy {
        label: StrategyLabel::RapidIteration,
        beam_width: 15, max_depth: 3, retry_waves: 5,
        confidence_threshold: 0.5,
        description: "Broad shallow, many retries. For brainstorming.",
    });
    m.insert(StrategyLabel::DeepDive, SearchStrategy {
        label: StrategyLabel::DeepDive,
        beam_width: 3, max_depth: 10, retry_waves: 1,
        confidence_threshold: 0.8,
        description: "Narrow but deep. For detailed analysis on known path.",
    });
    m.insert(StrategyLabel::Standard7x7, SearchStrategy {
        label: StrategyLabel::Standard7x7,
        beam_width: 7, max_depth: 7, retry_waves: 3,
        confidence_threshold: 0.7,
        description: "Default balanced. Miller's 7±2 × 7 depth.",
    });
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_strategies_includes_all_labels() {
        let m = default_strategies();
        assert!(m.contains_key(&StrategyLabel::Precise));
        assert!(m.contains_key(&StrategyLabel::Exploratory));
        assert!(m.contains_key(&StrategyLabel::Exhaustive));
        assert!(m.contains_key(&StrategyLabel::RapidIteration));
        assert!(m.contains_key(&StrategyLabel::DeepDive));
        assert!(m.contains_key(&StrategyLabel::Standard7x7));
    }

    #[test]
    fn labels_are_behavioral_not_diagnostic() {
        // Per Gemini's critique: no diagnostic terms
        let forbidden = [
            "adhd", "autistic", "autism", "neurotypical",
            "deficit", "disorder", "syndrome",
        ];
        for (label, strategy) in default_strategies().iter() {
            let lbl_str = label.as_str().to_lowercase();
            let desc_str = strategy.description.to_lowercase();
            for f in &forbidden {
                assert!(!lbl_str.contains(f),
                    "label '{}' contains diagnostic term '{}'", lbl_str, f);
                assert!(!desc_str.contains(f),
                    "description contains diagnostic term '{}': {}", f, desc_str);
            }
        }
    }

    #[test]
    fn precise_is_narrow_and_shallow() {
        let m = default_strategies();
        let p = &m[&StrategyLabel::Precise];
        assert!(p.beam_width < 5);
        assert!(p.max_depth < 8);
    }

    #[test]
    fn exploratory_is_wide_and_fairly_deep() {
        let m = default_strategies();
        let e = &m[&StrategyLabel::Exploratory];
        assert!(e.beam_width >= 8);
        assert!(e.max_depth >= 7);
    }

    #[test]
    fn clamping_enforces_limits() {
        let s = SearchStrategy {
            label: StrategyLabel::Custom,
            beam_width: 10000, // over-max
            max_depth: 100,     // over-max
            retry_waves: 50,
            confidence_threshold: 2.5,
            description: "test",
        };
        let clamped = s.clamped();
        assert!(clamped.beam_width <= super::super::MAX_BEAM_WIDTH);
        assert!(clamped.max_depth <= super::super::MAX_SEARCH_DEPTH);
        assert!(clamped.confidence_threshold <= 1.0);
    }

    #[test]
    fn standard_is_7_by_7() {
        let m = default_strategies();
        let s = &m[&StrategyLabel::Standard7x7];
        assert_eq!(s.beam_width, 7);
        assert_eq!(s.max_depth, 7);
    }
}
