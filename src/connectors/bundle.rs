//! # ConnectorBundle — a package of procedures for one platform
//!
//! A bundle groups all the procedures for a single platform (Gmail, Slack,
//! Telegram, etc). Each procedure is just a composition of primitives +
//! platform-specific configuration. Storage per procedure: ~200 bytes of
//! graph edges.
//!
//! This is intentionally REPETITIVE — 50 platforms × 6 procedures each =
//! 300 procedures. That sounds like a lot, but at 200 bytes each = 60KB
//! total. Thin code, many capabilities.

use crate::secrets::SecretKind;
use super::primitive::PrimitiveId;

/// A named procedure within a bundle.
#[derive(Debug, Clone)]
pub struct ConnectorProcedure {
    /// Unique id within the bundle (e.g. "send", "list", "delete").
    pub id: String,
    /// Human-readable description, used for when_to_use discovery.
    pub description: String,
    /// Steps: ordered calls to primitives + local values.
    pub steps: Vec<ProcedureStep>,
    /// Sense keys — how a Reader can discover this procedure.
    /// E.g. ["communication.send.email", "gmail.send"]
    pub sense_keys: Vec<String>,
    /// Required secret kinds.
    pub required_secrets: Vec<SecretKind>,
    /// Approximate compiled size.
    pub estimated_size_bytes: u32,
}

/// One step in a procedure — a primitive call with argument bindings.
#[derive(Debug, Clone)]
pub struct ProcedureStep {
    pub primitive: PrimitiveId,
    /// Argument bindings — placeholder strings that reference prior
    /// step outputs or caller inputs.
    ///
    /// Example: ["{caller.url}", "{step_0.headers}", "{caller.body_json}"]
    pub args: Vec<String>,
    /// Optional label for this step, so later steps can reference it.
    pub output_label: Option<String>,
}

/// A bundle of connectors for one platform.
#[derive(Debug, Clone)]
pub struct ConnectorBundle {
    /// Platform name (e.g. "gmail", "slack", "telegram").
    pub platform: String,
    /// Which auth method the platform uses.
    pub auth_kind: AuthKind,
    /// Base URL(s) of the platform's API.
    pub api_base: String,
    /// All procedures in this bundle.
    pub procedures: Vec<ConnectorProcedure>,
    /// Rate limit (requests per minute) — platform-dependent.
    pub rate_limit_rpm: u32,
    /// Documentation URL for the platform's API.
    pub docs_url: String,
}

/// How a platform authenticates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthKind {
    /// Bearer token in Authorization header.
    Bearer,
    /// API key header (varies — platform-specific header name).
    ApiKey,
    /// OAuth 2.0 flow.
    OAuth2,
    /// Basic auth (username + password).
    Basic,
    /// Signed request (HMAC).
    Signed,
    /// Webhook — no outbound auth, signature verification inbound.
    Webhook,
    /// Custom — platform-specific, needs special handling.
    Custom,
}

impl AuthKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthKind::Bearer => "bearer",
            AuthKind::ApiKey => "api_key",
            AuthKind::OAuth2 => "oauth2",
            AuthKind::Basic => "basic",
            AuthKind::Signed => "signed",
            AuthKind::Webhook => "webhook",
            AuthKind::Custom => "custom",
        }
    }
}

impl ConnectorProcedure {
    /// Compute estimated size from steps (heuristic).
    pub fn compute_size(&self) -> u32 {
        // Each step: ~40 bytes (primitive id + arg refs). Plus 50 bytes base.
        let step_size = self.steps.len() as u32 * 40;
        let meta_size = 50 + self.sense_keys.iter().map(|k| k.len() as u32).sum::<u32>();
        step_size + meta_size
    }
}

impl ConnectorBundle {
    pub fn new(
        platform: impl Into<String>,
        auth_kind: AuthKind,
        api_base: impl Into<String>,
        docs_url: impl Into<String>,
        rate_limit_rpm: u32,
    ) -> Self {
        ConnectorBundle {
            platform: platform.into(),
            auth_kind,
            api_base: api_base.into(),
            procedures: Vec::new(),
            rate_limit_rpm,
            docs_url: docs_url.into(),
        }
    }

    pub fn add_procedure(&mut self, proc: ConnectorProcedure) {
        self.procedures.push(proc);
    }

    pub fn find(&self, proc_id: &str) -> Option<&ConnectorProcedure> {
        self.procedures.iter().find(|p| p.id == proc_id)
    }

    pub fn procedure_count(&self) -> usize {
        self.procedures.len()
    }

    pub fn total_size_bytes(&self) -> u32 {
        self.procedures.iter().map(|p| p.compute_size()).sum()
    }

    /// Discover procedures by sense_key prefix. Useful for the Reader
    /// finding "communication.send.email" etc.
    pub fn find_by_sense(&self, sense_prefix: &str) -> Vec<&ConnectorProcedure> {
        self.procedures
            .iter()
            .filter(|p| p.sense_keys.iter().any(|k| k.starts_with(sense_prefix)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_step(p: PrimitiveId, args: &[&str]) -> ProcedureStep {
        ProcedureStep {
            primitive: p,
            args: args.iter().map(|s| s.to_string()).collect(),
            output_label: None,
        }
    }

    #[test]
    fn test_empty_bundle() {
        let b = ConnectorBundle::new("test", AuthKind::Bearer, "https://api.test", "https://docs", 60);
        assert_eq!(b.procedure_count(), 0);
        assert_eq!(b.total_size_bytes(), 0);
    }

    #[test]
    fn test_add_procedure() {
        let mut b = ConnectorBundle::new("gmail", AuthKind::OAuth2, "https://gmail", "https://docs", 250);
        b.add_procedure(ConnectorProcedure {
            id: "send".into(),
            description: "Send email".into(),
            steps: vec![
                mk_step(PrimitiveId::BuildJson, &["{caller.body}"]),
                mk_step(PrimitiveId::BuildBearerAuth, &["{secret.oauth_token}"]),
                mk_step(PrimitiveId::HttpPost, &["{base_url}/send", "{step_1}", "{step_0}"]),
            ],
            sense_keys: vec!["communication.send.email".into(), "gmail.send".into()],
            required_secrets: vec![SecretKind::OAuth],
            estimated_size_bytes: 0,
        });
        assert_eq!(b.procedure_count(), 1);
        assert!(b.total_size_bytes() > 0);
        assert!(b.find("send").is_some());
        assert!(b.find("unknown").is_none());
    }

    #[test]
    fn test_find_by_sense() {
        let mut b = ConnectorBundle::new("gmail", AuthKind::OAuth2, "x", "x", 100);
        b.add_procedure(ConnectorProcedure {
            id: "send".into(),
            description: "send".into(),
            steps: vec![mk_step(PrimitiveId::HttpPost, &[])],
            sense_keys: vec!["communication.send.email".into()],
            required_secrets: vec![],
            estimated_size_bytes: 0,
        });
        b.add_procedure(ConnectorProcedure {
            id: "list".into(),
            description: "list".into(),
            steps: vec![mk_step(PrimitiveId::HttpGet, &[])],
            sense_keys: vec!["communication.list.email".into()],
            required_secrets: vec![],
            estimated_size_bytes: 0,
        });

        let sends = b.find_by_sense("communication.send");
        assert_eq!(sends.len(), 1);
        assert_eq!(sends[0].id, "send");
    }

    #[test]
    fn test_size_under_budget() {
        // Each procedure should be well under 500 bytes
        let proc = ConnectorProcedure {
            id: "send".into(),
            description: "desc".into(),
            steps: vec![
                mk_step(PrimitiveId::BuildJson, &["{caller.body}"]),
                mk_step(PrimitiveId::BuildBearerAuth, &["{secret.token}"]),
                mk_step(PrimitiveId::HttpPost, &["{base}/endpoint"]),
                mk_step(PrimitiveId::ParseJson, &["{step_2}"]),
            ],
            sense_keys: vec!["communication.send.email".into()],
            required_secrets: vec![SecretKind::OAuth],
            estimated_size_bytes: 0,
        };
        let size = proc.compute_size();
        assert!(size < 500, "procedure too large: {} bytes", size);
        assert!(size > 100);
    }

    #[test]
    fn test_auth_kinds_distinct() {
        let all = [
            AuthKind::Bearer,
            AuthKind::ApiKey,
            AuthKind::OAuth2,
            AuthKind::Basic,
            AuthKind::Signed,
            AuthKind::Webhook,
            AuthKind::Custom,
        ];
        for (i, a) in all.iter().enumerate() {
            for (j, b) in all.iter().enumerate() {
                if i != j {
                    assert_ne!(a.as_str(), b.as_str());
                }
            }
        }
    }
}
