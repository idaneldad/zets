//! # Weaver — executes a CompositionPlan by filling motifs
//!
//! The Weaver takes a plan + a motif bank and produces the output,
//! step by step. Currently: pure motif_fill (native). Future: mark
//! capability_call steps as requiring the capability registry layer.

use std::collections::HashMap;
use std::fmt;

use super::motif_bank::MotifBank;
use super::plan::{CompositionPlan, Step, StepKind};

#[derive(Debug)]
pub enum ComposerError {
    MotifNotFound(String),
    MissingSlot { step: String, slot: String },
    DependencyNotSatisfied(String),
    CapabilityNotYetSupported(String),
}

impl fmt::Display for ComposerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComposerError::MotifNotFound(id) => write!(f, "motif not found: {}", id),
            ComposerError::MissingSlot { step, slot } => {
                write!(f, "step '{}' missing slot '{}'", step, slot)
            }
            ComposerError::DependencyNotSatisfied(d) => {
                write!(f, "depends on unprocessed step: {}", d)
            }
            ComposerError::CapabilityNotYetSupported(c) => {
                write!(f, "capability call not yet supported: {}", c)
            }
        }
    }
}

impl std::error::Error for ComposerError {}

#[derive(Debug, Clone)]
pub struct WovenOutput {
    pub plan_id: String,
    /// Output per step id.
    pub step_outputs: HashMap<String, String>,
    /// Steps executed successfully.
    pub completed_steps: Vec<String>,
    /// Steps skipped (e.g. capability calls not yet available).
    pub skipped_steps: Vec<String>,
    /// Final concatenated output (in step order).
    pub final_text: String,
}

pub struct Weaver;

impl Weaver {
    /// Execute a plan against a motif bank.
    ///
    /// Native MotifFill steps are executed directly. CapabilityCall
    /// steps are recorded as skipped — to be picked up by the capability
    /// orchestrator later (when that layer exists).
    pub fn weave(
        plan: &CompositionPlan,
        bank: &mut MotifBank,
    ) -> Result<WovenOutput, ComposerError> {
        let mut step_outputs: HashMap<String, String> = HashMap::new();
        let mut completed: Vec<String> = Vec::new();
        let mut skipped: Vec<String> = Vec::new();
        let mut final_parts: Vec<String> = Vec::new();

        for step in &plan.steps {
            // Check dependencies
            for dep in &step.depends_on {
                if !completed.contains(dep) && !skipped.contains(dep) {
                    return Err(ComposerError::DependencyNotSatisfied(dep.clone()));
                }
            }

            match step.kind {
                StepKind::MotifFill => {
                    let motif_id = step
                        .motif_id
                        .clone()
                        .ok_or_else(|| ComposerError::MotifNotFound("<none>".into()))?;

                    let output = {
                        let motif = bank
                            .get(&motif_id)
                            .ok_or_else(|| ComposerError::MotifNotFound(motif_id.clone()))?;

                        // Build binding values — some may reference prior step outputs
                        let mut bindings = step.bindings.clone();
                        for dep in &step.depends_on {
                            if let Some(prev) = step_outputs.get(dep) {
                                // Make prior output available under the dep id
                                bindings.insert(dep.clone(), prev.clone());
                            }
                        }

                        // Validate all slots will be filled
                        for slot in &motif.slots {
                            if !bindings.contains_key(slot) {
                                return Err(ComposerError::MissingSlot {
                                    step: step.id.clone(),
                                    slot: slot.clone(),
                                });
                            }
                        }

                        motif.fill(&bindings).ok_or_else(|| ComposerError::MissingSlot {
                            step: step.id.clone(),
                            slot: "(unknown)".into(),
                        })?
                    };

                    // Now update use counter
                    if let Some(m) = bank.get_mut(&motif_id) {
                        m.record_use();
                    }

                    step_outputs.insert(step.id.clone(), output.clone());
                    completed.push(step.id.clone());
                    final_parts.push(output);
                }

                StepKind::CapabilityCall => {
                    // Not yet supported — mark skipped, collect for future orchestrator
                    let cap = step
                        .motif_id
                        .clone()
                        .unwrap_or_else(|| "<unspecified>".into());
                    skipped.push(step.id.clone());
                    step_outputs.insert(
                        step.id.clone(),
                        format!("<<capability_placeholder: {}>>", cap),
                    );
                }

                StepKind::Merge => {
                    // Merge outputs of depends_on steps
                    let merged: Vec<String> = step
                        .depends_on
                        .iter()
                        .filter_map(|d| step_outputs.get(d).cloned())
                        .collect();
                    let out = merged.join("\n\n");
                    step_outputs.insert(step.id.clone(), out.clone());
                    completed.push(step.id.clone());
                    final_parts.push(out);
                }

                StepKind::Control => {
                    // Stub: control flow not yet implemented
                    completed.push(step.id.clone());
                }
            }
        }

        let final_text = final_parts.join("\n");

        Ok(WovenOutput {
            plan_id: plan.id.clone(),
            step_outputs,
            completed_steps: completed,
            skipped_steps: skipped,
            final_text,
        })
    }
}

// So step can be constructed for tests
impl Step {
    /// Convenience for tests — create without going through builder
    #[cfg(test)]
    pub fn new_for_test(id: impl Into<String>, kind: StepKind, motif_id: Option<String>) -> Self {
        Step {
            id: id.into(),
            kind,
            motif_id,
            bindings: HashMap::new(),
            depends_on: Vec::new(),
            note: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::motif_bank::{Motif, MotifKind};
    use crate::composition::plan::Step;

    fn seed_bank() -> MotifBank {
        let mut bank = MotifBank::new();
        bank.insert(Motif::new(
            "opener1",
            MotifKind::NarrativeBeat,
            "Once upon a time, {hero} lived in {place}.",
        ));
        bank.insert(Motif::new(
            "conflict1",
            MotifKind::NarrativeBeat,
            "One day, {character} faced {obstacle}.",
        ));
        bank.insert(Motif::new(
            "resolution1",
            MotifKind::NarrativeBeat,
            "And so {character} found {outcome}.",
        ));
        bank
    }

    #[test]
    fn test_weave_simple_plan() {
        let mut bank = seed_bank();
        let plan = CompositionPlan::new("tiny_story")
            .add_step(
                Step::motif_fill("opener", "opener1")
                    .bind("hero", "a brave knight")
                    .bind("place", "a dark forest"),
            )
            .add_step(
                Step::motif_fill("conflict", "conflict1")
                    .bind("character", "the knight")
                    .bind("obstacle", "a fierce dragon")
                    .after("opener"),
            );

        let result = Weaver::weave(&plan, &mut bank).unwrap();
        assert_eq!(result.completed_steps.len(), 2);
        assert_eq!(result.skipped_steps.len(), 0);
        assert!(result.final_text.contains("brave knight"));
        assert!(result.final_text.contains("fierce dragon"));
    }

    #[test]
    fn test_weave_three_step_story() {
        let mut bank = seed_bank();
        let plan = CompositionPlan::new("full_story")
            .add_step(
                Step::motif_fill("opener", "opener1")
                    .bind("hero", "a young witch")
                    .bind("place", "a seaside town"),
            )
            .add_step(
                Step::motif_fill("conflict", "conflict1")
                    .bind("character", "the witch")
                    .bind("obstacle", "a mystery")
                    .after("opener"),
            )
            .add_step(
                Step::motif_fill("resolution", "resolution1")
                    .bind("character", "the witch")
                    .bind("outcome", "wisdom")
                    .after("conflict"),
            );

        let result = Weaver::weave(&plan, &mut bank).unwrap();
        assert_eq!(result.completed_steps.len(), 3);
        // All motifs got used
        assert_eq!(bank.get("opener1").unwrap().used_count, 1);
        assert_eq!(bank.get("resolution1").unwrap().used_count, 1);
    }

    #[test]
    fn test_weave_missing_motif_fails() {
        let mut bank = seed_bank();
        let plan = CompositionPlan::new("bad").add_step(
            Step::motif_fill("s1", "nonexistent")
                .bind("x", "y"),
        );
        let err = Weaver::weave(&plan, &mut bank).unwrap_err();
        assert!(matches!(err, ComposerError::MotifNotFound(_)));
    }

    #[test]
    fn test_weave_missing_slot_fails() {
        let mut bank = seed_bank();
        let plan = CompositionPlan::new("bad")
            .add_step(Step::motif_fill("s1", "opener1").bind("hero", "dragon"));
        // missing "place" slot
        let err = Weaver::weave(&plan, &mut bank).unwrap_err();
        assert!(matches!(err, ComposerError::MissingSlot { .. }));
    }

    #[test]
    fn test_capability_call_is_skipped_not_failed() {
        let mut bank = seed_bank();
        let plan = CompositionPlan::new("mixed")
            .add_step(
                Step::motif_fill("plan", "opener1")
                    .bind("hero", "a bulldog")
                    .bind("place", "a sunny yard"),
            )
            .add_step(Step::capability("render", "image_gen").after("plan"));

        let result = Weaver::weave(&plan, &mut bank).unwrap();
        assert_eq!(result.completed_steps.len(), 1);
        assert_eq!(result.skipped_steps.len(), 1);
        assert!(result.final_text.contains("bulldog"));
    }

    #[test]
    fn test_generation_is_really_generative() {
        // The key test: we produce NEW text that didn't exist before
        // the call, just from motifs + slot values.
        let mut bank = seed_bank();

        let plan = CompositionPlan::new("novel_story")
            .add_step(
                Step::motif_fill("o", "opener1")
                    .bind("hero", "a scientist named Miriam")
                    .bind("place", "a quiet laboratory in 1947 Jerusalem"),
            );

        let result = Weaver::weave(&plan, &mut bank).unwrap();
        // This exact sentence has never been seen — it's genuinely new
        assert!(result.final_text.contains("Miriam"));
        assert!(result.final_text.contains("1947 Jerusalem"));
        assert_eq!(
            result.final_text,
            "Once upon a time, a scientist named Miriam lived in a quiet laboratory in 1947 Jerusalem."
        );
    }
}
