//! Persona graph — map from persona identity to search strategy.
//!
//! Personas are NOT hardcoded config — they live in the graph as atoms.
//! Users can edit them. The system can learn preferences over time.
//!
//! Stored as: `atom[persona:<id>] --routes_to--> atom[strategy:<label>]`

use std::collections::HashMap;
use super::strategy::{StrategyLabel, SearchStrategy, default_strategies};

/// PersonaGraph — lookup from persona_id to search strategy.
///
/// In the final Rust integration, this will be backed by atom edges.
/// For now it's an in-memory HashMap that can be initialized from graph
/// or from defaults.
#[derive(Debug, Clone)]
pub struct PersonaGraph {
    persona_to_label: HashMap<String, StrategyLabel>,
    strategies: HashMap<StrategyLabel, SearchStrategy>,
    default_label: StrategyLabel,
}

impl PersonaGraph {
    /// Create with the default 6 personas. All behavioral labels, no diagnostic terms.
    pub fn new_default() -> Self {
        let mut m = HashMap::new();
        m.insert("dry_precise_user".to_string(), StrategyLabel::Precise);
        m.insert("medical_patient".to_string(), StrategyLabel::Exhaustive);
        m.insert("creative_scientist".to_string(), StrategyLabel::Exploratory);
        m.insert("brainstormer".to_string(), StrategyLabel::RapidIteration);
        m.insert("deep_researcher".to_string(), StrategyLabel::DeepDive);
        m.insert("anonymous".to_string(), StrategyLabel::Standard7x7);

        Self {
            persona_to_label: m,
            strategies: default_strategies(),
            default_label: StrategyLabel::Standard7x7,
        }
    }

    /// Override or add a persona→strategy mapping.
    pub fn set_persona(&mut self, persona_id: &str, label: StrategyLabel) {
        self.persona_to_label.insert(persona_id.to_string(), label);
    }

    /// Add or replace a strategy definition.
    pub fn set_strategy(&mut self, strategy: SearchStrategy) {
        self.strategies.insert(strategy.label, strategy);
    }

    /// Look up strategy for a given persona. Falls back to default if unknown.
    pub fn strategy_for(&self, persona_id: &str) -> SearchStrategy {
        let label = self.persona_to_label.get(persona_id)
            .copied()
            .unwrap_or(self.default_label);
        self.strategies.get(&label)
            .cloned()
            .unwrap_or_else(|| self.strategies[&StrategyLabel::Standard7x7].clone())
    }

    /// All known persona IDs.
    pub fn known_personas(&self) -> Vec<&str> {
        self.persona_to_label.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for PersonaGraph {
    fn default() -> Self {
        Self::new_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_personas_exist() {
        let pg = PersonaGraph::new_default();
        let personas = pg.known_personas();
        assert!(personas.contains(&"dry_precise_user"));
        assert!(personas.contains(&"medical_patient"));
        assert!(personas.contains(&"creative_scientist"));
        assert!(personas.contains(&"brainstormer"));
        assert!(personas.contains(&"deep_researcher"));
    }

    #[test]
    fn precise_user_gets_precise_strategy() {
        let pg = PersonaGraph::new_default();
        let s = pg.strategy_for("dry_precise_user");
        assert_eq!(s.label, StrategyLabel::Precise);
        assert!(s.beam_width < 5);
    }

    #[test]
    fn scientist_gets_exploratory() {
        let pg = PersonaGraph::new_default();
        let s = pg.strategy_for("creative_scientist");
        assert_eq!(s.label, StrategyLabel::Exploratory);
        assert!(s.beam_width >= 8);
    }

    #[test]
    fn medical_patient_gets_exhaustive() {
        let pg = PersonaGraph::new_default();
        let s = pg.strategy_for("medical_patient");
        assert_eq!(s.label, StrategyLabel::Exhaustive);
        assert!(s.retry_waves >= 3);
    }

    #[test]
    fn unknown_persona_falls_back_to_default() {
        let pg = PersonaGraph::new_default();
        let s = pg.strategy_for("completely_made_up_persona_xyz");
        assert_eq!(s.label, StrategyLabel::Standard7x7);
    }

    #[test]
    fn set_persona_overrides() {
        let mut pg = PersonaGraph::new_default();
        pg.set_persona("dry_precise_user", StrategyLabel::Exhaustive);
        let s = pg.strategy_for("dry_precise_user");
        assert_eq!(s.label, StrategyLabel::Exhaustive);
    }

    #[test]
    fn set_custom_strategy_works() {
        let mut pg = PersonaGraph::new_default();
        let custom = SearchStrategy {
            label: StrategyLabel::Custom,
            beam_width: 9,
            max_depth: 6,
            retry_waves: 2,
            confidence_threshold: 0.65,
            description: "user-defined blend",
        };
        pg.set_strategy(custom.clone());
        pg.set_persona("power_user", StrategyLabel::Custom);
        let s = pg.strategy_for("power_user");
        assert_eq!(s.beam_width, 9);
        assert_eq!(s.max_depth, 6);
    }
}
