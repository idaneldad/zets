//! # Result types for capability invocations
//!
//! Defines `Value` (a dependency-free JSON-like enum), `CapabilityResult`
//! (the outcome of an invocation), and `CapabilityError` (pre-invocation
//! failures like ACL denial or missing registration).

use std::fmt;

// ---------------------------------------------------------------------------
// Value — lightweight JSON-like enum (no serde_json dependency)
// TODO: Replace with serde_json::Value when serde_json is added to Cargo.toml
// ---------------------------------------------------------------------------

/// A dependency-free representation of structured data.
///
/// Used for capability arguments and outputs. Mirrors JSON semantics
/// but adds `Bytes` for binary payloads (audio, images).
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Map(Vec<(String, Value)>),
}

impl Value {
    /// Returns `true` if the value is `Null`.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Attempt to get a string reference.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Value::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Attempt to get an integer.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    /// Look up a key in a Map value.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Map(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(n) => write!(f, "{n}"),
            Value::Text(s) => write!(f, "\"{s}\""),
            Value::Bytes(b) => write!(f, "<{} bytes>", b.len()),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, v) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            Value::Map(pairs) => {
                write!(f, "{{")?;
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{k}\": {v}")?;
                }
                write!(f, "}}")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CapabilityResult — the outcome of an invocation
// ---------------------------------------------------------------------------

/// Result of a capability invocation that reached the executor.
#[derive(Debug, Clone)]
pub enum CapabilityResult {
    /// The capability succeeded.
    Success {
        output: Value,
        cost_cents: u32,
        duration_ms: u64,
    },
    /// The call exceeded the per-call timeout.
    Timeout,
    /// The caller's budget would be exceeded.
    BudgetExceeded,
    /// The capability's rate limit is exhausted.
    RateLimited {
        retry_after_ms: u64,
    },
    /// A transient (retryable) error occurred.
    TransientError {
        retry_count: u32,
    },
    /// A permanent (non-retryable) error occurred.
    PermanentError {
        reason: String,
    },
}

impl CapabilityResult {
    /// Returns `true` if the result is a `Success`.
    pub fn is_success(&self) -> bool {
        matches!(self, CapabilityResult::Success { .. })
    }

    /// Short label for audit logging (no PII).
    pub fn kind_label(&self) -> &'static str {
        match self {
            CapabilityResult::Success { .. } => "success",
            CapabilityResult::Timeout => "timeout",
            CapabilityResult::BudgetExceeded => "budget_exceeded",
            CapabilityResult::RateLimited { .. } => "rate_limited",
            CapabilityResult::TransientError { .. } => "transient_error",
            CapabilityResult::PermanentError { .. } => "permanent_error",
        }
    }
}

// ---------------------------------------------------------------------------
// CapabilityError — pre-invocation failures
// ---------------------------------------------------------------------------

/// Errors that prevent an invocation from even reaching the executor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityError {
    /// The requested capability_id is not in the registry.
    NotRegistered(String),
    /// The caller does not have permission to invoke this capability.
    AccessDenied,
    /// The invocation arguments are invalid.
    InvalidArgs(String),
    /// The capability's provider is currently unavailable.
    Unavailable,
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CapabilityError::NotRegistered(id) => {
                write!(f, "capability not registered: {id}")
            }
            CapabilityError::AccessDenied => write!(f, "access denied"),
            CapabilityError::InvalidArgs(reason) => {
                write!(f, "invalid arguments: {reason}")
            }
            CapabilityError::Unavailable => write!(f, "capability unavailable"),
        }
    }
}

impl std::error::Error for CapabilityError {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_null() {
        assert!(Value::Null.is_null());
        assert!(!Value::Bool(true).is_null());
    }

    #[test]
    fn test_value_map_get() {
        let map = Value::Map(vec![
            ("file".into(), Value::Text("audio.wav".into())),
            ("lang".into(), Value::Text("en".into())),
        ]);
        assert_eq!(map.get("file").and_then(|v| v.as_text()), Some("audio.wav"));
        assert!(map.get("missing").is_none());
    }

    #[test]
    fn test_capability_result_labels() {
        assert_eq!(
            CapabilityResult::Success {
                output: Value::Null,
                cost_cents: 0,
                duration_ms: 0
            }
            .kind_label(),
            "success"
        );
        assert_eq!(CapabilityResult::Timeout.kind_label(), "timeout");
        assert_eq!(CapabilityResult::BudgetExceeded.kind_label(), "budget_exceeded");
    }

    #[test]
    fn test_capability_error_display() {
        let e = CapabilityError::NotRegistered("whisper.transcribe".into());
        assert!(e.to_string().contains("whisper.transcribe"));
    }
}
