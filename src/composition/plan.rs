//! # Plan — a composition plan, the skeleton before realization
//!
//! A CompositionPlan breaks a generation request into ORDERED STEPS.
//! Each step either fills a motif (native) or calls an external tool.
//!
//! The plan is graph-native: it's a DAG of steps, each pointing to a
//! motif and slot bindings.

use std::collections::HashMap;

/// One step in a composition plan.
#[derive(Debug, Clone)]
pub struct Step {
    pub id: String,
    pub kind: StepKind,
    /// Id of the motif (from MotifBank) that fills this step's content.
    pub motif_id: Option<String>,
    /// Values to fill the motif's slots.
    pub bindings: HashMap<String, String>,
    /// If this step depends on prior steps (their output used as slot values).
    pub depends_on: Vec<String>,
    /// Notes — human-readable context.
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepKind {
    /// Fill a motif template — fully native.
    MotifFill,
    /// Orchestrate an external capability (diffusion, LLM, STT).
    CapabilityCall,
    /// Merge outputs of prior steps.
    Merge,
    /// Control — skip / branch / repeat.
    Control,
}

impl StepKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            StepKind::MotifFill => "motif_fill",
            StepKind::CapabilityCall => "capability_call",
            StepKind::Merge => "merge",
            StepKind::Control => "control",
        }
    }
}

impl Step {
    pub fn motif_fill(id: impl Into<String>, motif_id: impl Into<String>) -> Self {
        Step {
            id: id.into(),
            kind: StepKind::MotifFill,
            motif_id: Some(motif_id.into()),
            bindings: HashMap::new(),
            depends_on: Vec::new(),
            note: None,
        }
    }

    pub fn capability(id: impl Into<String>, capability_name: impl Into<String>) -> Self {
        Step {
            id: id.into(),
            kind: StepKind::CapabilityCall,
            motif_id: Some(capability_name.into()),
            bindings: HashMap::new(),
            depends_on: Vec::new(),
            note: None,
        }
    }

    pub fn bind(mut self, slot: impl Into<String>, value: impl Into<String>) -> Self {
        self.bindings.insert(slot.into(), value.into());
        self
    }

    pub fn after(mut self, step_id: impl Into<String>) -> Self {
        self.depends_on.push(step_id.into());
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}

/// A complete composition plan.
#[derive(Debug, Clone)]
pub struct CompositionPlan {
    pub id: String,
    /// The ordered sequence of steps (topologically sorted, or execution order).
    pub steps: Vec<Step>,
    /// Expected style/domain tags (for motif selection).
    pub style_hints: Vec<String>,
    /// Hard constraints (must include / must exclude).
    pub must_include: Vec<String>,
    pub must_exclude: Vec<String>,
    /// Budget for external calls (cost in $$ or API credits).
    pub external_budget: Option<f32>,
}

impl CompositionPlan {
    pub fn new(id: impl Into<String>) -> Self {
        CompositionPlan {
            id: id.into(),
            steps: Vec::new(),
            style_hints: Vec::new(),
            must_include: Vec::new(),
            must_exclude: Vec::new(),
            external_budget: None,
        }
    }

    pub fn add_step(mut self, step: Step) -> Self {
        self.steps.push(step);
        self
    }

    pub fn with_style(mut self, s: impl Into<String>) -> Self {
        self.style_hints.push(s.into());
        self
    }

    pub fn require(mut self, s: impl Into<String>) -> Self {
        self.must_include.push(s.into());
        self
    }

    pub fn forbid(mut self, s: impl Into<String>) -> Self {
        self.must_exclude.push(s.into());
        self
    }

    pub fn with_budget(mut self, b: f32) -> Self {
        self.external_budget = Some(b);
        self
    }

    /// Is this plan entirely native (no external calls)?
    pub fn is_fully_native(&self) -> bool {
        self.steps.iter().all(|s| s.kind != StepKind::CapabilityCall)
    }

    /// What external capabilities does this plan require?
    pub fn required_capabilities(&self) -> Vec<&str> {
        self.steps
            .iter()
            .filter(|s| s.kind == StepKind::CapabilityCall)
            .filter_map(|s| s.motif_id.as_deref())
            .collect()
    }

    /// Validate topological order — no step depends on something after it.
    pub fn is_topologically_sorted(&self) -> bool {
        let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for step in &self.steps {
            for dep in &step.depends_on {
                if !seen.contains(dep.as_str()) {
                    return false;
                }
            }
            seen.insert(&step.id);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_motif_fill_creation() {
        let s = Step::motif_fill("s1", "opener")
            .bind("hero", "dragon")
            .bind("place", "cave");
        assert_eq!(s.kind, StepKind::MotifFill);
        assert_eq!(s.bindings.len(), 2);
    }

    #[test]
    fn test_capability_step() {
        let s = Step::capability("img1", "image_gen").bind("prompt", "sunset over mountains");
        assert_eq!(s.kind, StepKind::CapabilityCall);
    }

    #[test]
    fn test_plan_with_deps() {
        let p = CompositionPlan::new("short_story_1")
            .with_style("fantasy")
            .with_style("uplifting")
            .require("protagonist")
            .forbid("violence")
            .add_step(Step::motif_fill("opener", "opener1").bind("hero", "brave knight"))
            .add_step(
                Step::motif_fill("conflict", "conflict1")
                    .bind("character", "brave knight")
                    .bind("obstacle", "dragon")
                    .after("opener"),
            )
            .add_step(
                Step::motif_fill("resolution", "resolution1")
                    .bind("character", "brave knight")
                    .bind("outcome", "peace")
                    .after("conflict"),
            );

        assert_eq!(p.steps.len(), 3);
        assert!(p.is_topologically_sorted());
        assert!(p.is_fully_native());
    }

    #[test]
    fn test_detects_bad_order() {
        let p = CompositionPlan::new("bad_order")
            .add_step(Step::motif_fill("first", "m1").after("second"))
            .add_step(Step::motif_fill("second", "m2"));

        assert!(!p.is_topologically_sorted());
    }

    #[test]
    fn test_mixed_plan_native_and_external() {
        let p = CompositionPlan::new("mixed")
            .add_step(Step::motif_fill("plan", "story_skeleton").bind("theme", "adventure"))
            .add_step(Step::capability("generate", "llm.text").after("plan"))
            .add_step(Step::capability("illustrate", "image_gen").after("generate"));

        assert!(!p.is_fully_native());
        let caps = p.required_capabilities();
        assert_eq!(caps.len(), 2);
        assert!(caps.contains(&"llm.text"));
        assert!(caps.contains(&"image_gen"));
    }

    #[test]
    fn test_budget_attached() {
        let p = CompositionPlan::new("with_budget").with_budget(0.50);
        assert_eq!(p.external_budget, Some(0.50));
    }

    #[test]
    fn test_image_composition_plan() {
        // Realistic: "create a brown bulldog with white face" → plan
        let p = CompositionPlan::new("bulldog_image")
            .with_style("photorealistic")
            .require("brown body")
            .require("white face")
            .add_step(
                Step::motif_fill("prompt", "portrait_spec")
                    .bind("subject", "brown bulldog with white face")
                    .bind("style", "photorealistic")
                    .bind("lighting", "soft natural")
                    .bind("composition", "centered, eye-level"),
            )
            .add_step(Step::capability("render", "image_gen").after("prompt"))
            .with_budget(0.10);

        assert_eq!(p.steps.len(), 2);
        assert_eq!(p.required_capabilities(), vec!["image_gen"]);
        assert!(p.is_topologically_sorted());
    }
}
