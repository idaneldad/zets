//! # LLM abstraction — one interface for OpenAI / Anthropic / Gemini / local
//!
//! ZETS uses LLMs as external helpers (translation, code resynthesis,
//! answer composition). Without an abstraction, calls to each provider
//! are scattered in the codebase and hard to swap / test / rate-limit.
//!
//! The `LlmClient` trait is the contract. Concrete impls live in
//! provider-specific modules. For Phase 1 we ship a `MockClient` that
//! returns canned responses, and trait definitions. Real HTTP clients
//! will be added when a concrete call site emerges.
//!
//! ## Design
//!
//! - Async-less for now (will add `async` trait when callers need it).
//! - Each provider's auth is a `SecretRef` — no hardcoded keys.
//! - Response shape is normalized: text + token counts + finish_reason.
//! - Failures are `LlmError` — caller can log to ErrorStore.

use std::fmt;

use crate::secrets::SecretId;

/// A chat-style LLM request.
#[derive(Debug, Clone)]
pub struct LlmRequest {
    /// Model identifier in provider-native format.
    pub model: String,
    /// Ordered messages. System message first if present.
    pub messages: Vec<Message>,
    /// Hard cap on output tokens. 0 = use provider default.
    pub max_tokens: u32,
    /// 0.0 = deterministic, 1.0 = max creative.
    pub temperature: f32,
    /// Optional: stop sequences.
    pub stop: Vec<String>,
}

/// A single message in a chat request.
#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// Role of a message — standard across providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
        }
    }
}

/// Normalized response across providers.
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub text: String,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub finish_reason: FinishReason,
    pub provider: String,
    pub model: String,
    pub latency_ms: u64,
}

/// Why the LLM stopped generating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinishReason {
    /// Reached a natural stop token.
    Stop,
    /// Hit max_tokens.
    Length,
    /// Provider-side safety filter triggered.
    SafetyFilter,
    /// Error partway through.
    Error,
    /// Unknown / not reported.
    Unknown,
}

impl FinishReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            FinishReason::Stop => "stop",
            FinishReason::Length => "length",
            FinishReason::SafetyFilter => "safety_filter",
            FinishReason::Error => "error",
            FinishReason::Unknown => "unknown",
        }
    }
}

/// Errors from LLM calls.
#[derive(Debug)]
pub enum LlmError {
    /// Network/transport failure.
    Transport(String),
    /// Provider returned 4xx/5xx.
    Http { status: u16, body: String },
    /// Malformed response we can't parse.
    Parse(String),
    /// Rate limit exceeded.
    RateLimit { retry_after_ms: Option<u64> },
    /// Authentication failure (bad API key).
    Auth,
    /// Requested model doesn't exist or isn't accessible.
    ModelNotFound(String),
    /// Provider-side safety block.
    SafetyBlock(String),
    /// Timeout waiting for response.
    Timeout,
    /// The SecretRef says the caller can't use this secret.
    AccessDenied,
    /// Other.
    Other(String),
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmError::Transport(s) => write!(f, "transport: {}", s),
            LlmError::Http { status, body } => write!(f, "http {}: {}", status, body),
            LlmError::Parse(s) => write!(f, "parse: {}", s),
            LlmError::RateLimit { retry_after_ms } => {
                write!(f, "rate limit (retry after {:?}ms)", retry_after_ms)
            }
            LlmError::Auth => write!(f, "auth failed"),
            LlmError::ModelNotFound(m) => write!(f, "model not found: {}", m),
            LlmError::SafetyBlock(s) => write!(f, "safety block: {}", s),
            LlmError::Timeout => write!(f, "timeout"),
            LlmError::AccessDenied => write!(f, "caller cannot access the secret"),
            LlmError::Other(s) => write!(f, "other: {}", s),
        }
    }
}

impl std::error::Error for LlmError {}

/// Which LLM provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provider {
    OpenAi,
    Anthropic,
    Google,
    Groq,
    Local,
    Mock,
}

impl Provider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::OpenAi => "openai",
            Provider::Anthropic => "anthropic",
            Provider::Google => "google",
            Provider::Groq => "groq",
            Provider::Local => "local",
            Provider::Mock => "mock",
        }
    }
}

/// Which secret this client needs to authenticate.
#[derive(Debug, Clone)]
pub struct AuthSpec {
    pub provider: Provider,
    pub secret_id: SecretId,
}

/// The core LLM trait.
///
/// Intentionally simple: one method, chat completion. Multi-modal and
/// streaming can be added later via extension traits.
pub trait LlmClient {
    fn provider(&self) -> Provider;

    /// Make a chat completion call.
    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError>;
}

/// A mock client — returns canned responses. Useful for tests.
pub struct MockClient {
    canned_response: String,
    provider: Provider,
}

impl MockClient {
    pub fn new(canned: impl Into<String>) -> Self {
        MockClient {
            canned_response: canned.into(),
            provider: Provider::Mock,
        }
    }

    pub fn with_provider(mut self, p: Provider) -> Self {
        self.provider = p;
        self
    }
}

impl LlmClient for MockClient {
    fn provider(&self) -> Provider {
        self.provider
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let tokens_in = request
            .messages
            .iter()
            .map(|m| (m.content.len() / 4) as u32)
            .sum::<u32>()
            .max(1);
        let tokens_out = (self.canned_response.len() / 4) as u32;

        Ok(LlmResponse {
            text: self.canned_response.clone(),
            tokens_in,
            tokens_out,
            finish_reason: FinishReason::Stop,
            provider: self.provider.as_str().into(),
            model: request.model.clone(),
            latency_ms: 0,
        })
    }
}

/// A registry of LLM clients — lets code pick by provider without
/// knowing the concrete type.
///
/// Usage:
/// ```ignore
/// let mut reg = LlmRegistry::new();
/// reg.register(Box::new(MockClient::new("hi")));
/// let client = reg.get(Provider::Mock)?;
/// ```
pub struct LlmRegistry {
    clients: Vec<Box<dyn LlmClient>>,
}

impl LlmRegistry {
    pub fn new() -> Self {
        LlmRegistry { clients: Vec::new() }
    }

    pub fn register(&mut self, client: Box<dyn LlmClient>) {
        self.clients.push(client);
    }

    pub fn get(&self, provider: Provider) -> Option<&dyn LlmClient> {
        self.clients
            .iter()
            .find(|c| c.provider() == provider)
            .map(|b| b.as_ref())
    }

    pub fn len(&self) -> usize {
        self.clients.len()
    }

    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }
}

impl Default for LlmRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience constructor.
pub fn user_message(content: impl Into<String>) -> Message {
    Message {
        role: Role::User,
        content: content.into(),
    }
}

pub fn system_message(content: impl Into<String>) -> Message {
    Message {
        role: Role::System,
        content: content.into(),
    }
}

pub fn assistant_message(content: impl Into<String>) -> Message {
    Message {
        role: Role::Assistant,
        content: content.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_client_basic() {
        let client = MockClient::new("hello world");
        let req = LlmRequest {
            model: "test-model".into(),
            messages: vec![user_message("hi")],
            max_tokens: 100,
            temperature: 0.5,
            stop: vec![],
        };
        let resp = client.complete(&req).unwrap();
        assert_eq!(resp.text, "hello world");
        assert_eq!(resp.finish_reason, FinishReason::Stop);
        assert_eq!(resp.provider, "mock");
    }

    #[test]
    fn test_mock_client_custom_provider() {
        let client = MockClient::new("canned").with_provider(Provider::OpenAi);
        assert_eq!(client.provider(), Provider::OpenAi);
        let req = LlmRequest {
            model: "gpt-4".into(),
            messages: vec![user_message("hi")],
            max_tokens: 100,
            temperature: 0.5,
            stop: vec![],
        };
        let resp = client.complete(&req).unwrap();
        assert_eq!(resp.provider, "openai");
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut reg = LlmRegistry::new();
        reg.register(Box::new(MockClient::new("r1").with_provider(Provider::OpenAi)));
        reg.register(Box::new(MockClient::new("r2").with_provider(Provider::Anthropic)));

        assert_eq!(reg.len(), 2);
        assert!(reg.get(Provider::OpenAi).is_some());
        assert!(reg.get(Provider::Anthropic).is_some());
        assert!(reg.get(Provider::Google).is_none());
    }

    #[test]
    fn test_registry_uses_correct_client() {
        let mut reg = LlmRegistry::new();
        reg.register(Box::new(MockClient::new("openai_text").with_provider(Provider::OpenAi)));
        reg.register(
            Box::new(MockClient::new("anthropic_text").with_provider(Provider::Anthropic)),
        );

        let req = LlmRequest {
            model: "any".into(),
            messages: vec![user_message("x")],
            max_tokens: 10,
            temperature: 0.0,
            stop: vec![],
        };

        let oai = reg.get(Provider::OpenAi).unwrap();
        let ant = reg.get(Provider::Anthropic).unwrap();

        assert_eq!(oai.complete(&req).unwrap().text, "openai_text");
        assert_eq!(ant.complete(&req).unwrap().text, "anthropic_text");
    }

    #[test]
    fn test_role_strings() {
        assert_eq!(Role::System.as_str(), "system");
        assert_eq!(Role::User.as_str(), "user");
        assert_eq!(Role::Assistant.as_str(), "assistant");
    }

    #[test]
    fn test_message_builders() {
        let m = user_message("hi");
        assert_eq!(m.role, Role::User);
        let s = system_message("sys");
        assert_eq!(s.role, Role::System);
        let a = assistant_message("reply");
        assert_eq!(a.role, Role::Assistant);
    }

    #[test]
    fn test_token_estimate() {
        let client = MockClient::new("reply of about twelve tokens maybe more");
        let req = LlmRequest {
            model: "m".into(),
            messages: vec![
                system_message("you are helpful"),
                user_message("a question"),
            ],
            max_tokens: 50,
            temperature: 0.5,
            stop: vec![],
        };
        let resp = client.complete(&req).unwrap();
        assert!(resp.tokens_in > 0);
        assert!(resp.tokens_out > 0);
    }

    #[test]
    fn test_error_display() {
        let e = LlmError::Http {
            status: 503,
            body: "overloaded".into(),
        };
        let s = format!("{}", e);
        assert!(s.contains("503"));
        assert!(s.contains("overloaded"));
    }

    #[test]
    fn test_finish_reason_strings() {
        assert_eq!(FinishReason::Stop.as_str(), "stop");
        assert_eq!(FinishReason::SafetyFilter.as_str(), "safety_filter");
    }

    #[test]
    fn test_provider_strings() {
        assert_eq!(Provider::OpenAi.as_str(), "openai");
        assert_eq!(Provider::Anthropic.as_str(), "anthropic");
        assert_eq!(Provider::Google.as_str(), "google");
        assert_eq!(Provider::Local.as_str(), "local");
    }

    #[test]
    fn test_empty_registry() {
        let reg = LlmRegistry::new();
        assert!(reg.is_empty());
        assert!(reg.get(Provider::OpenAi).is_none());
    }
}
