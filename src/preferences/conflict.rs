//! # Conflict resolution — how competing preferences are resolved
//!
//! Rules (in priority order):
//!   1. Explicit > Inferred > Default
//!   2. Among Explicit: newer beats older
//!   3. Among Inferred: higher confidence beats lower
//!   4. If two Explicits have the same timestamp: keep the newer by set_at_ms

use crate::preferences::value::{PreferenceOrigin, PreferenceValue};

/// The outcome of resolving a conflict between two preference values.
#[derive(Debug, Clone)]
pub enum PreferenceConflict {
    /// New value overrides old value.
    Override {
        old: PreferenceValue,
        new: PreferenceValue,
    },
    /// Multiple values merged (used when collecting history).
    Merge { values: Vec<PreferenceValue> },
    /// Keep the existing value — new value doesn't win.
    Keep { reason: String },
}

/// Determine which of two values should win.
///
/// Returns `true` if `new_val` should replace `existing_val`.
pub fn new_wins(existing: &PreferenceValue, new_val: &PreferenceValue) -> bool {
    let existing_rank = existing.priority_rank();
    let new_rank = new_val.priority_rank();

    if new_rank > existing_rank {
        // Higher origin priority always wins
        return true;
    }

    if new_rank < existing_rank {
        // Lower origin priority never wins
        return false;
    }

    // Same rank — break ties within origin type
    match (&existing.origin, &new_val.origin) {
        (PreferenceOrigin::Explicit { .. }, PreferenceOrigin::Explicit { .. }) => {
            // Newer explicit beats older
            new_val.set_at_ms > existing.set_at_ms
        }
        (PreferenceOrigin::Inferred { .. }, PreferenceOrigin::Inferred { .. }) => {
            // Higher confidence beats lower; if equal, newer beats older
            if (new_val.confidence - existing.confidence).abs() > 0.05 {
                new_val.confidence > existing.confidence
            } else {
                new_val.set_at_ms > existing.set_at_ms
            }
        }
        _ => false,
    }
}

/// Resolve a conflict between an existing and a new preference value.
///
/// Returns the value that should become the active preference,
/// along with a `PreferenceConflict` describing what happened.
pub fn resolve(
    existing: PreferenceValue,
    new_val: PreferenceValue,
) -> (PreferenceValue, PreferenceConflict) {
    if new_wins(&existing, &new_val) {
        let conflict = PreferenceConflict::Override {
            old: existing,
            new: new_val.clone(),
        };
        (new_val, conflict)
    } else {
        let reason = format!(
            "existing ({:?}, conf={:.2}, ts={}) beats new ({:?}, conf={:.2}, ts={})",
            origin_label(&existing.origin),
            existing.confidence,
            existing.set_at_ms,
            origin_label(&new_val.origin),
            new_val.confidence,
            new_val.set_at_ms,
        );
        let conflict = PreferenceConflict::Keep { reason };
        (existing, conflict)
    }
}

fn origin_label(origin: &PreferenceOrigin) -> &'static str {
    match origin {
        PreferenceOrigin::Explicit { .. } => "explicit",
        PreferenceOrigin::Inferred { .. } => "inferred",
        PreferenceOrigin::Default => "default",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::personal_graph::{IdentityId, IdentityKind};
    use crate::preferences::value::PreferenceValue;

    fn owner() -> IdentityId {
        IdentityId::new(IdentityKind::Person, "idan")
    }

    #[test]
    fn test_explicit_beats_inferred() {
        let inferred = PreferenceValue::inferred("short", vec![], 0.9, 1000);
        let explicit = PreferenceValue::explicit("long", owner(), 500); // even older!
        assert!(new_wins(&inferred, &explicit));
    }

    #[test]
    fn test_inferred_does_not_beat_explicit() {
        let explicit = PreferenceValue::explicit("formal", owner(), 1000);
        let inferred = PreferenceValue::inferred("casual", vec![], 0.99, 2000);
        assert!(!new_wins(&explicit, &inferred));
    }

    #[test]
    fn test_newer_explicit_beats_older_explicit() {
        let older = PreferenceValue::explicit("formal", owner(), 1000);
        let newer = PreferenceValue::explicit("casual", owner(), 2000);
        assert!(new_wins(&older, &newer));
        assert!(!new_wins(&newer, &older));
    }

    #[test]
    fn test_higher_confidence_inferred_beats_lower() {
        let low = PreferenceValue::inferred("short", vec![], 0.4, 1000);
        let high = PreferenceValue::inferred("long", vec![], 0.9, 500);
        assert!(new_wins(&low, &high));
    }

    #[test]
    fn test_resolve_explicit_over_inferred() {
        let inferred = PreferenceValue::inferred("short", vec![], 0.8, 1000);
        let explicit = PreferenceValue::explicit("long", owner(), 500);
        let (winner, conflict) = resolve(inferred, explicit);
        assert_eq!(winner.value, "long");
        assert!(matches!(conflict, PreferenceConflict::Override { .. }));
    }

    #[test]
    fn test_resolve_keeps_explicit_over_inferred() {
        let explicit = PreferenceValue::explicit("formal", owner(), 1000);
        let inferred = PreferenceValue::inferred("casual", vec![], 0.99, 2000);
        let (winner, conflict) = resolve(explicit, inferred);
        assert_eq!(winner.value, "formal");
        assert!(matches!(conflict, PreferenceConflict::Keep { .. }));
    }

    #[test]
    fn test_default_loses_to_everything() {
        let default = PreferenceValue::default_value("medium");
        let inferred = PreferenceValue::inferred("short", vec![], 0.3, 100);
        assert!(new_wins(&default, &inferred));
    }
}
