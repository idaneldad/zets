//! # ConnectorPrimitive — the atomic building blocks of all connectors
//!
//! A primitive is a low-level operation that most connectors need:
//! HTTP requests, auth flows, JSON parsing, etc. Primitives are shared
//! across procedures — a `send_gmail` procedure and a `send_slack`
//! procedure BOTH use `http_post` + `build_auth_header`.
//!
//! ## Design
//!
//! Primitives are identified by a stable `PrimitiveId`. Each primitive has
//! a signature (input/output types) and a `sense_key` for discovery in
//! the graph. The actual implementation is bytecode executed by the
//! system_graph VM.
//!
//! Procedures compose primitives — that's where the real logic lives.
//! Storage: 15 primitives × ~500 bytes bytecode = ~7.5KB. Then thousands
//! of procedures × ~200 bytes of composition edges. Linear in procedures,
//! O(unique) in primitives.

use std::fmt;

/// Identifier for a connector primitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveId {
    // ─── Network ──────────────────────────
    HttpGet,
    HttpPost,
    HttpPut,
    HttpPatch,
    HttpDelete,
    WebsocketOpen,
    WebsocketSend,

    // ─── Auth ─────────────────────────────
    /// Build a bearer-token auth header from a SecretRef.
    BuildBearerAuth,
    /// Build a basic-auth header.
    BuildBasicAuth,
    /// Build an API-key header (e.g. X-API-Key: ...).
    BuildApiKeyHeader,
    /// OAuth 2.0 refresh-token flow.
    OAuthRefresh,
    /// HMAC-SHA256 signature for request signing.
    HmacSign,

    // ─── Serialization ────────────────────
    ParseJson,
    BuildJson,
    ParseXml,
    BuildXml,
    /// Extract value by JSONPath-like expression.
    JsonPath,
    /// Extract value by XPath.
    XPath,
    UrlEncode,
    UrlDecode,
    Base64Encode,
    Base64Decode,

    // ─── Retry / reliability ──────────────
    /// Retry a step with exponential backoff.
    RetryBackoff,
    /// Apply a rate limit (token bucket).
    RateLimit,

    // ─── Files / blobs ────────────────────
    MultipartForm,
    ReadFileBytes,
    WriteFileBytes,

    // ─── Time ─────────────────────────────
    Iso8601Now,
    UnixTimestamp,
    ParseIso8601,
    FormatIso8601,
}

impl PrimitiveId {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrimitiveId::HttpGet => "http_get",
            PrimitiveId::HttpPost => "http_post",
            PrimitiveId::HttpPut => "http_put",
            PrimitiveId::HttpPatch => "http_patch",
            PrimitiveId::HttpDelete => "http_delete",
            PrimitiveId::WebsocketOpen => "ws_open",
            PrimitiveId::WebsocketSend => "ws_send",
            PrimitiveId::BuildBearerAuth => "build_bearer_auth",
            PrimitiveId::BuildBasicAuth => "build_basic_auth",
            PrimitiveId::BuildApiKeyHeader => "build_api_key_header",
            PrimitiveId::OAuthRefresh => "oauth_refresh",
            PrimitiveId::HmacSign => "hmac_sign",
            PrimitiveId::ParseJson => "parse_json",
            PrimitiveId::BuildJson => "build_json",
            PrimitiveId::ParseXml => "parse_xml",
            PrimitiveId::BuildXml => "build_xml",
            PrimitiveId::JsonPath => "json_path",
            PrimitiveId::XPath => "xpath",
            PrimitiveId::UrlEncode => "url_encode",
            PrimitiveId::UrlDecode => "url_decode",
            PrimitiveId::Base64Encode => "base64_encode",
            PrimitiveId::Base64Decode => "base64_decode",
            PrimitiveId::RetryBackoff => "retry_backoff",
            PrimitiveId::RateLimit => "rate_limit",
            PrimitiveId::MultipartForm => "multipart_form",
            PrimitiveId::ReadFileBytes => "read_file_bytes",
            PrimitiveId::WriteFileBytes => "write_file_bytes",
            PrimitiveId::Iso8601Now => "iso8601_now",
            PrimitiveId::UnixTimestamp => "unix_timestamp",
            PrimitiveId::ParseIso8601 => "parse_iso8601",
            PrimitiveId::FormatIso8601 => "format_iso8601",
        }
    }

    pub fn sense_key(&self) -> String {
        format!("connector_primitive.{}", self.as_str())
    }

    pub fn category(&self) -> PrimitiveCategory {
        use PrimitiveId::*;
        match self {
            HttpGet | HttpPost | HttpPut | HttpPatch | HttpDelete | WebsocketOpen
            | WebsocketSend => PrimitiveCategory::Network,
            BuildBearerAuth | BuildBasicAuth | BuildApiKeyHeader | OAuthRefresh | HmacSign => {
                PrimitiveCategory::Auth
            }
            ParseJson | BuildJson | ParseXml | BuildXml | JsonPath | XPath | UrlEncode
            | UrlDecode | Base64Encode | Base64Decode => PrimitiveCategory::Serialization,
            RetryBackoff | RateLimit => PrimitiveCategory::Reliability,
            MultipartForm | ReadFileBytes | WriteFileBytes => PrimitiveCategory::Io,
            Iso8601Now | UnixTimestamp | ParseIso8601 | FormatIso8601 => PrimitiveCategory::Time,
        }
    }

    /// Approximate bytecode size in bytes (for storage accounting).
    pub fn estimated_bytecode_size(&self) -> u32 {
        match self.category() {
            PrimitiveCategory::Network => 800,
            PrimitiveCategory::Auth => 400,
            PrimitiveCategory::Serialization => 600,
            PrimitiveCategory::Reliability => 500,
            PrimitiveCategory::Io => 400,
            PrimitiveCategory::Time => 200,
        }
    }

    /// All 31 primitives — for seeding the registry.
    pub fn all() -> Vec<PrimitiveId> {
        use PrimitiveId::*;
        vec![
            HttpGet, HttpPost, HttpPut, HttpPatch, HttpDelete, WebsocketOpen, WebsocketSend,
            BuildBearerAuth, BuildBasicAuth, BuildApiKeyHeader, OAuthRefresh, HmacSign,
            ParseJson, BuildJson, ParseXml, BuildXml, JsonPath, XPath,
            UrlEncode, UrlDecode, Base64Encode, Base64Decode,
            RetryBackoff, RateLimit,
            MultipartForm, ReadFileBytes, WriteFileBytes,
            Iso8601Now, UnixTimestamp, ParseIso8601, FormatIso8601,
        ]
    }
}

impl fmt::Display for PrimitiveId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveCategory {
    Network,
    Auth,
    Serialization,
    Reliability,
    Io,
    Time,
}

impl PrimitiveCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrimitiveCategory::Network => "network",
            PrimitiveCategory::Auth => "auth",
            PrimitiveCategory::Serialization => "serialization",
            PrimitiveCategory::Reliability => "reliability",
            PrimitiveCategory::Io => "io",
            PrimitiveCategory::Time => "time",
        }
    }
}

/// Value types passed between primitives.
///
/// Simple, tagged. The VM knows how to marshal these.
#[derive(Debug, Clone, PartialEq)]
pub enum PValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Json(String),
    List(Vec<PValue>),
    Map(Vec<(String, PValue)>),
    /// Handle to a secret — the vault resolves it when executing.
    SecretRef(String),
}

impl PValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            PValue::Null => "null",
            PValue::Bool(_) => "bool",
            PValue::Int(_) => "int",
            PValue::Float(_) => "float",
            PValue::String(_) => "string",
            PValue::Bytes(_) => "bytes",
            PValue::Json(_) => "json",
            PValue::List(_) => "list",
            PValue::Map(_) => "map",
            PValue::SecretRef(_) => "secret_ref",
        }
    }
}

/// Total storage impact of all primitives in the binary.
///
/// Used for reporting — how much binary space the primitives cost.
pub fn total_primitives_bytecode() -> u32 {
    PrimitiveId::all()
        .iter()
        .map(|p| p.estimated_bytecode_size())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_31_primitives() {
        assert_eq!(PrimitiveId::all().len(), 31);
    }

    #[test]
    fn test_as_str_uniqueness() {
        let all = PrimitiveId::all();
        let mut seen = std::collections::HashSet::new();
        for p in all {
            let s = p.as_str();
            assert!(seen.insert(s), "duplicate as_str: {}", s);
        }
    }

    #[test]
    fn test_sense_key_prefix() {
        for p in PrimitiveId::all() {
            assert!(p.sense_key().starts_with("connector_primitive."));
        }
    }

    #[test]
    fn test_category_network_includes_http() {
        assert_eq!(PrimitiveId::HttpGet.category(), PrimitiveCategory::Network);
        assert_eq!(PrimitiveId::HttpPost.category(), PrimitiveCategory::Network);
    }

    #[test]
    fn test_category_auth_includes_bearer() {
        assert_eq!(
            PrimitiveId::BuildBearerAuth.category(),
            PrimitiveCategory::Auth
        );
    }

    #[test]
    fn test_total_primitives_under_20kb() {
        let total = total_primitives_bytecode();
        // 31 primitives — must stay under 20KB of bytecode total
        assert!(total < 20_000, "primitives too large: {}", total);
        // But not trivially small either
        assert!(total > 5_000);
    }

    #[test]
    fn test_pvalue_type_names() {
        assert_eq!(PValue::Null.type_name(), "null");
        assert_eq!(PValue::Bool(true).type_name(), "bool");
        assert_eq!(PValue::String("x".into()).type_name(), "string");
        assert_eq!(PValue::SecretRef("k".into()).type_name(), "secret_ref");
    }

    #[test]
    fn test_primitive_id_roundtrip_display() {
        assert_eq!(format!("{}", PrimitiveId::HttpPost), "http_post");
    }

    #[test]
    fn test_categories_have_different_sizes() {
        // Network primitives are more expensive than Time primitives
        let http = PrimitiveId::HttpPost.estimated_bytecode_size();
        let time = PrimitiveId::UnixTimestamp.estimated_bytecode_size();
        assert!(http > time);
    }

    #[test]
    fn test_no_duplicates_in_all() {
        let all = PrimitiveId::all();
        let mut set = std::collections::HashSet::new();
        for p in all {
            assert!(set.insert(p), "duplicate in all(): {:?}", p);
        }
    }
}
