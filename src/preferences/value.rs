//! # PreferenceValue — a preference value with provenance
//!
//! Each preference has a value (string), an origin (who/what set it),
//! a timestamp, and a confidence score. History is stored as a Vec of
//! past values — the active value is the last entry.

use crate::personal_graph::IdentityId;

/// A single preference value with full provenance.
#[derive(Debug, Clone)]
pub struct PreferenceValue {
    /// The actual value string (e.g. "short", "en", "formal").
    pub value: String,
    /// How this preference was established.
    pub origin: PreferenceOrigin,
    /// When this value was recorded (Unix milliseconds).
    pub set_at_ms: i64,
    /// Confidence that this is a genuine, stable preference (0..1).
    /// Explicit values have confidence 1.0; inferred start lower.
    pub confidence: f32,
}

/// How a preference was established.
#[derive(Debug, Clone)]
pub enum PreferenceOrigin {
    /// User or owner explicitly set it (e.g. "prefer English").
    Explicit {
        /// Who set it — the owner themselves, or an admin.
        by: IdentityId,
    },
    /// Detected from conversation patterns.
    Inferred {
        /// IDs / hashes of messages that contributed evidence.
        from_messages: Vec<String>,
    },
    /// System default — used when nothing else is set.
    Default,
}

impl PreferenceValue {
    /// Create an explicit preference (confidence = 1.0).
    pub fn explicit(value: impl Into<String>, by: IdentityId, now_ms: i64) -> Self {
        PreferenceValue {
            value: value.into(),
            origin: PreferenceOrigin::Explicit { by },
            set_at_ms: now_ms,
            confidence: 1.0,
        }
    }

    /// Create an inferred preference.
    pub fn inferred(
        value: impl Into<String>,
        from_messages: Vec<String>,
        confidence: f32,
        now_ms: i64,
    ) -> Self {
        PreferenceValue {
            value: value.into(),
            origin: PreferenceOrigin::Inferred { from_messages },
            set_at_ms: now_ms,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }

    /// Create a system default.
    pub fn default_value(value: impl Into<String>) -> Self {
        PreferenceValue {
            value: value.into(),
            origin: PreferenceOrigin::Default,
            set_at_ms: 0,
            confidence: 1.0,
        }
    }

    /// Whether this value came from an explicit user/admin action.
    pub fn is_explicit(&self) -> bool {
        matches!(self.origin, PreferenceOrigin::Explicit { .. })
    }

    /// Whether this value was inferred from messages.
    pub fn is_inferred(&self) -> bool {
        matches!(self.origin, PreferenceOrigin::Inferred { .. })
    }

    /// Whether this is a system default.
    pub fn is_default(&self) -> bool {
        matches!(self.origin, PreferenceOrigin::Default)
    }

    /// Priority rank for conflict resolution (higher = wins).
    /// Explicit > Inferred > Default.
    pub fn priority_rank(&self) -> u8 {
        match &self.origin {
            PreferenceOrigin::Explicit { .. } => 2,
            PreferenceOrigin::Inferred { .. } => 1,
            PreferenceOrigin::Default => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::personal_graph::{IdentityId, IdentityKind};

    fn owner() -> IdentityId {
        IdentityId::new(IdentityKind::Person, "idan")
    }

    #[test]
    fn test_explicit_has_full_confidence() {
        let v = PreferenceValue::explicit("formal", owner(), 1000);
        assert_eq!(v.confidence, 1.0);
        assert!(v.is_explicit());
        assert!(!v.is_inferred());
    }

    #[test]
    fn test_inferred_value() {
        let v = PreferenceValue::inferred("short", vec!["msg1".into()], 0.7, 2000);
        assert!(v.is_inferred());
        assert_eq!(v.confidence, 0.7);
        assert!(!v.is_explicit());
    }

    #[test]
    fn test_default_value() {
        let v = PreferenceValue::default_value("auto");
        assert!(v.is_default());
        assert_eq!(v.set_at_ms, 0);
    }

    #[test]
    fn test_priority_rank_order() {
        let explicit = PreferenceValue::explicit("a", owner(), 1000);
        let inferred = PreferenceValue::inferred("b", vec![], 0.8, 1000);
        let default = PreferenceValue::default_value("c");
        assert!(explicit.priority_rank() > inferred.priority_rank());
        assert!(inferred.priority_rank() > default.priority_rank());
    }

    #[test]
    fn test_confidence_clamped() {
        let v = PreferenceValue::inferred("x", vec![], 2.5, 1000);
        assert_eq!(v.confidence, 1.0);
        let v2 = PreferenceValue::inferred("x", vec![], -0.5, 1000);
        assert_eq!(v2.confidence, 0.0);
    }
}
