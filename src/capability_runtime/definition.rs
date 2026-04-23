//! # Capability definitions
//!
//! A `CapabilityDefinition` describes an external capability that the
//! orchestrator can invoke — its provider type, endpoint, cost model,
//! rate limits, and the secret key needed for authentication.

use std::fmt;

/// How a capability is provided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    /// An HTTP POST endpoint (REST API, webhook).
    HttpPost,
    /// A local process or function call.
    Local,
    /// A custom provider (plugin, gRPC, etc.).
    Custom(String),
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provider::HttpPost => write!(f, "http_post"),
            Provider::Local => write!(f, "local"),
            Provider::Custom(name) => write!(f, "custom:{name}"),
        }
    }
}

/// Describes a registered capability — what it does, how to call it,
/// what it costs, and how fast it can be called.
#[derive(Debug, Clone)]
pub struct CapabilityDefinition {
    /// Unique identifier, e.g. `"whisper.transcribe"`, `"gemini.vision"`.
    pub id: String,
    /// Human-readable description.
    pub description: String,
    /// How this capability is provided.
    pub provider: Provider,
    /// HTTP endpoint (for `HttpPost` provider).
    pub endpoint: Option<String>,
    /// Key into the secrets vault for auth (e.g. `"person:idan/api_key/openai"`).
    pub auth_secret_id: Option<String>,
    /// Estimated cost per invocation in cents.
    pub cost_per_call_cents: u32,
    /// Maximum calls allowed per minute (0 = unlimited).
    pub rate_limit_per_minute: u32,
    /// Typical latency in milliseconds (informational).
    pub typical_latency_ms: u64,
}

impl CapabilityDefinition {
    /// Create a new capability definition with required fields.
    pub fn new(
        id: impl Into<String>,
        description: impl Into<String>,
        provider: Provider,
    ) -> Self {
        CapabilityDefinition {
            id: id.into(),
            description: description.into(),
            provider,
            endpoint: None,
            auth_secret_id: None,
            cost_per_call_cents: 0,
            rate_limit_per_minute: 0,
            typical_latency_ms: 0,
        }
    }

    /// Builder: set HTTP endpoint.
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Builder: set auth secret ID (key into vault).
    pub fn with_auth_secret(mut self, secret_id: impl Into<String>) -> Self {
        self.auth_secret_id = Some(secret_id.into());
        self
    }

    /// Builder: set cost per call in cents.
    pub fn with_cost(mut self, cents: u32) -> Self {
        self.cost_per_call_cents = cents;
        self
    }

    /// Builder: set rate limit (calls per minute).
    pub fn with_rate_limit(mut self, per_minute: u32) -> Self {
        self.rate_limit_per_minute = per_minute;
        self
    }

    /// Builder: set typical latency.
    pub fn with_latency(mut self, ms: u64) -> Self {
        self.typical_latency_ms = ms;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definition_builder() {
        let def = CapabilityDefinition::new(
            "whisper.transcribe",
            "Transcribe audio to text via OpenAI Whisper",
            Provider::HttpPost,
        )
        .with_endpoint("https://api.openai.com/v1/audio/transcriptions")
        .with_auth_secret("person:idan/api_key/openai")
        .with_cost(3)
        .with_rate_limit(60)
        .with_latency(2000);

        assert_eq!(def.id, "whisper.transcribe");
        assert_eq!(def.cost_per_call_cents, 3);
        assert_eq!(def.rate_limit_per_minute, 60);
        assert!(def.endpoint.is_some());
        assert!(def.auth_secret_id.is_some());
    }

    #[test]
    fn test_provider_display() {
        assert_eq!(Provider::HttpPost.to_string(), "http_post");
        assert_eq!(Provider::Local.to_string(), "local");
        assert_eq!(Provider::Custom("grpc".into()).to_string(), "custom:grpc");
    }
}
