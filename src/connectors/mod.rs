//! # Connectors — platform-specific capability bundles
//!
//! This module answers Idan's core insight (23.04.2026):
//!
//! > "In graph architecture, these weigh very little, and the ZETS brain
//! > will learn excellently how to program."
//!
//! That's exactly right. Each connector procedure is ~200 bytes of graph
//! edges composing pre-existing primitives. 50 platforms × ~6 procedures
//! each = ~60KB total storage. And every procedure is TRAINING DATA for
//! ZETS to learn API composition patterns.
//!
//! ## Architecture
//!
//! ```text
//! ConnectorRegistry
//!     ├─ Bundle: gmail      [OAuth2, 4 procedures]
//!     │    ├─ send          → uses http_post + build_bearer_auth + build_json
//!     │    ├─ list          → uses http_get + build_bearer_auth + parse_json
//!     │    ├─ read          → uses http_get + build_bearer_auth + parse_json
//!     │    └─ label_add     → uses http_post + build_bearer_auth + build_json
//!     ├─ Bundle: slack      [Bearer, 4 procedures]
//!     ├─ Bundle: telegram   [ApiKey, 3 procedures]
//!     ├─ Bundle: whatsapp   [ApiKey, 2 procedures]
//!     ├─ Bundle: smtp       [Basic, 1 procedure]
//!     ├─ Bundle: drive      [OAuth2, 3 procedures]
//!     ├─ Bundle: sheets     [OAuth2, 3 procedures]
//!     ├─ Bundle: calendar   [OAuth2, 3 procedures]
//!     └─ Bundle: openai     [Bearer, 3 procedures]
//!
//! Shared primitives (compiled, ~15KB):
//!     http_get, http_post, http_put, http_patch, http_delete
//!     build_bearer_auth, build_api_key_header, oauth_refresh
//!     parse_json, build_json, json_path, xml parsing
//!     url_encode, base64, retry, rate_limit, multipart
//!     iso8601_now, unix_timestamp, ...
//! ```
//!
//! ## Why so many
//!
//! Three reasons:
//!
//! 1. **Dogfooding** — ZETS itself needs to send emails, write to Drive,
//!    post to Slack, reply on WhatsApp. These connectors are FIRST for
//!    ZETS's own agentic behavior; second for end-users.
//!
//! 2. **Learning signal** — every procedure is a worked example of API
//!    composition. When ZETS meets a new API (Phase C ingest), it
//!    generalizes from these patterns: OAuth2 flow, JSON request shape,
//!    response parsing, error handling. The 51st platform is "free"
//!    because ZETS has seen 50.
//!
//! 3. **Discovery surface** — when Reader detects intent like "send an
//!    email" (sense: communication.send.email), there must be procedures
//!    to find. No procedures → nothing to execute.
//!
//! ## Discovery model
//!
//! Every procedure carries sense_keys describing what it does:
//!   - `communication.send.email` — matches gmail.send AND smtp.send
//!   - `communication.send.chat` — matches slack + telegram + whatsapp
//!   - `schedule.create.event` — matches calendar.event_create
//!   - `ai.chat.complete` — matches openai.chat_complete
//!
//! Reader finds by sense; user's Owner context selects the right
//! platform (the one they have auth for).

pub mod bundle;
pub mod primitive;
pub mod registry;
pub mod seed;

pub use bundle::{AuthKind, ConnectorBundle, ConnectorProcedure, ProcedureStep};
pub use primitive::{PValue, PrimitiveCategory, PrimitiveId};
pub use registry::{ConnectorRegistry, RegistryStats};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_registry_loads() {
        let r = ConnectorRegistry::with_seeds();
        let stats = r.stats();
        assert_eq!(stats.bundle_count, 9);
        // Total storage budget for all seeded bundles: 15KB
        assert!(stats.total_bytes < 15_000);
    }

    #[test]
    fn test_send_email_discovery_paths() {
        let r = ConnectorRegistry::with_seeds();
        let hits = r.find_by_sense("communication.send.email");
        // Gmail + SMTP both handle this
        assert!(hits.len() >= 2);
    }

    #[test]
    fn test_send_chat_covers_three_platforms() {
        let r = ConnectorRegistry::with_seeds();
        let hits = r.find_by_sense("communication.send.chat");
        let platforms: std::collections::HashSet<&str> =
            hits.iter().map(|(p, _)| *p).collect();
        assert!(platforms.contains("slack"));
        assert!(platforms.contains("telegram"));
        assert!(platforms.contains("whatsapp_greenapi"));
    }

    #[test]
    fn test_ai_senses_route_to_openai() {
        let r = ConnectorRegistry::with_seeds();
        let chat = r.find_by_sense("ai.chat.complete");
        assert!(!chat.is_empty());
        assert_eq!(chat[0].0, "openai");

        let tts = r.find_by_sense("ai.tts");
        assert!(!tts.is_empty());
    }

    #[test]
    fn test_storage_vs_productivity_ratio() {
        // The whole point of the graph architecture: many capabilities,
        // little storage. Verify the ratio is good.
        let r = ConnectorRegistry::with_seeds();
        let stats = r.stats();

        // Capability density: procedures per KB
        let per_kb = stats.procedure_count as f32 / (stats.total_bytes as f32 / 1024.0);
        // Should be at least 2 procedures per KB of graph storage
        assert!(per_kb > 2.0, "density too low: {} proc/KB", per_kb);
    }

    #[test]
    fn test_every_platform_exports_at_least_one_procedure() {
        let r = ConnectorRegistry::with_seeds();
        for b in r.bundles() {
            assert!(
                b.procedure_count() > 0,
                "empty bundle: {}",
                b.platform
            );
        }
    }

    #[test]
    fn test_no_duplicate_platform_names() {
        let r = ConnectorRegistry::with_seeds();
        let mut seen = std::collections::HashSet::new();
        for b in r.bundles() {
            assert!(seen.insert(&b.platform), "duplicate platform: {}", b.platform);
        }
    }
}
