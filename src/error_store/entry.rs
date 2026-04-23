//! # ErrorEntry — one recorded failure or anomaly
//!
//! Everything that went wrong gets an ErrorEntry. The entry is append-only;
//! errors are never deleted, only marked as resolved.

use std::fmt;

/// Unique identifier for an error event.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ErrorId(pub String);

impl fmt::Display for ErrorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Severity — drives routing (logs only vs alerts vs escalation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Severity {
    /// Noise; a thing happened we want to know about but it's recoverable.
    Info,
    /// Something odd; worth a human look at some point.
    Warn,
    /// Something broke; user-visible or process-impacting.
    Error,
    /// Process in danger or data at risk.
    Critical,
    /// Security-class: credential misuse, ACL violation, injection attempt.
    Security,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Warn => "warn",
            Severity::Error => "error",
            Severity::Critical => "critical",
            Severity::Security => "security",
        }
    }

    pub fn is_alert_worthy(&self) -> bool {
        matches!(self, Severity::Critical | Severity::Security)
    }
}

/// What category of failure is this? Used for aggregation and trend detection.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// A procedure failed to execute.
    ProcedureFailure { procedure_id: String },
    /// The Reader's gate held / refused a response.
    GateHold { reason: String },
    /// An external API call failed (timeout, 5xx, etc).
    ExternalApi { service: String, status: Option<u16> },
    /// LLM call failed or produced garbage.
    LlmFailure { provider: String },
    /// ACL / permission denial.
    AccessDenied { subject: String },
    /// Secret vault access failed.
    VaultError { detail: String },
    /// Graph inconsistency detected (orphan edge, impossible state).
    GraphInconsistency { detail: String },
    /// The user/source reported dissatisfaction with an answer.
    UserFeedback { rating: i8 },
    /// Ingestion from external source failed.
    IngestionFailure { source: String },
    /// Unclassified — falls here when nothing fits.
    Other(String),
}

impl ErrorKind {
    pub fn sense_key(&self) -> String {
        let base = match self {
            ErrorKind::ProcedureFailure { .. } => "procedure_failure",
            ErrorKind::GateHold { .. } => "gate_hold",
            ErrorKind::ExternalApi { .. } => "external_api",
            ErrorKind::LlmFailure { .. } => "llm_failure",
            ErrorKind::AccessDenied { .. } => "access_denied",
            ErrorKind::VaultError { .. } => "vault_error",
            ErrorKind::GraphInconsistency { .. } => "graph_inconsistency",
            ErrorKind::UserFeedback { .. } => "user_feedback",
            ErrorKind::IngestionFailure { .. } => "ingestion_failure",
            ErrorKind::Other(_) => "other",
        };
        format!("error_kind.{}", base)
    }
}

/// Resolution status of an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Resolution {
    /// Not yet looked at.
    #[default]
    Open,
    /// Someone (or ZETS itself) is working on it.
    InProgress,
    /// Fixed.
    Resolved,
    /// Seen but not going to fix (expected behavior, duplicate).
    WontFix,
    /// Superseded by a later, broader error.
    Superseded,
}

impl Resolution {
    pub fn as_str(&self) -> &'static str {
        match self {
            Resolution::Open => "open",
            Resolution::InProgress => "in_progress",
            Resolution::Resolved => "resolved",
            Resolution::WontFix => "wont_fix",
            Resolution::Superseded => "superseded",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Resolution::Resolved | Resolution::WontFix | Resolution::Superseded
        )
    }
}

/// One recorded error.
#[derive(Debug, Clone)]
pub struct ErrorEntry {
    pub id: ErrorId,
    pub kind: ErrorKind,
    pub severity: Severity,
    /// When the error occurred (Unix ms).
    pub occurred_ms: i64,
    /// Which source was involved (identifier string — e.g. from Source::identifier()).
    pub source: Option<String>,
    /// Which session / correlation id, for grouping.
    pub context: Option<String>,
    /// Free-text description.
    pub message: String,
    /// Optional: structured detail JSON-like.
    pub detail: Option<String>,
    /// Resolution status.
    pub resolution: Resolution,
    /// When resolution last changed.
    pub resolution_changed_ms: Option<i64>,
    /// If resolved, how.
    pub resolution_note: Option<String>,
    /// How many times this kind of error has recurred (filled by store).
    pub occurrence_count: u32,
}

impl ErrorEntry {
    pub fn new(
        id: impl Into<String>,
        kind: ErrorKind,
        severity: Severity,
        message: impl Into<String>,
        occurred_ms: i64,
    ) -> Self {
        ErrorEntry {
            id: ErrorId(id.into()),
            kind,
            severity,
            occurred_ms,
            source: None,
            context: None,
            message: message.into(),
            detail: None,
            resolution: Resolution::Open,
            resolution_changed_ms: None,
            resolution_note: None,
            occurrence_count: 1,
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn resolve(&mut self, now_ms: i64, note: Option<String>) {
        self.resolution = Resolution::Resolved;
        self.resolution_changed_ms = Some(now_ms);
        self.resolution_note = note;
    }

    pub fn mark(&mut self, new_status: Resolution, now_ms: i64, note: Option<String>) {
        self.resolution = new_status;
        self.resolution_changed_ms = Some(now_ms);
        if note.is_some() {
            self.resolution_note = note;
        }
    }

    pub fn is_open(&self) -> bool {
        !self.resolution.is_terminal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kind_sense_keys() {
        assert_eq!(
            ErrorKind::ProcedureFailure {
                procedure_id: "x".into()
            }
            .sense_key(),
            "error_kind.procedure_failure"
        );
        assert_eq!(
            ErrorKind::GateHold { reason: "x".into() }.sense_key(),
            "error_kind.gate_hold"
        );
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::Error);
        assert!(Severity::Security > Severity::Critical);
        assert!(Severity::Warn < Severity::Error);
    }

    #[test]
    fn test_severity_alert() {
        assert!(Severity::Critical.is_alert_worthy());
        assert!(Severity::Security.is_alert_worthy());
        assert!(!Severity::Error.is_alert_worthy());
        assert!(!Severity::Warn.is_alert_worthy());
    }

    #[test]
    fn test_lifecycle() {
        let mut e = ErrorEntry::new(
            "err001",
            ErrorKind::LlmFailure {
                provider: "openai".into(),
            },
            Severity::Error,
            "timeout calling openai",
            1000,
        );
        assert!(e.is_open());

        e.resolve(2000, Some("retry worked".into()));
        assert_eq!(e.resolution, Resolution::Resolved);
        assert!(!e.is_open());
        assert_eq!(e.resolution_changed_ms, Some(2000));
    }

    #[test]
    fn test_terminal_states() {
        assert!(Resolution::Resolved.is_terminal());
        assert!(Resolution::WontFix.is_terminal());
        assert!(Resolution::Superseded.is_terminal());
        assert!(!Resolution::Open.is_terminal());
        assert!(!Resolution::InProgress.is_terminal());
    }

    #[test]
    fn test_with_builders() {
        let e = ErrorEntry::new(
            "err002",
            ErrorKind::ExternalApi {
                service: "github".into(),
                status: Some(503),
            },
            Severity::Warn,
            "github 503",
            5000,
        )
        .with_source("person:idan")
        .with_context("session_42")
        .with_detail(r#"{"retries": 3}"#);

        assert_eq!(e.source.as_deref(), Some("person:idan"));
        assert_eq!(e.context.as_deref(), Some("session_42"));
        assert!(e.detail.is_some());
    }
}
