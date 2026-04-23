//! # Connector registry
//!
//! In-memory storage and lookup for `CapabilityDefinition`s.
//! Thread-safe: uses no interior mutability; mutation requires `&mut self`.

use std::collections::HashMap;

use super::definition::CapabilityDefinition;

/// In-memory registry of capability definitions.
///
/// Capabilities are registered once at startup (or dynamically via admin
/// commands) and looked up on every invocation.
#[derive(Debug, Default)]
pub struct ConnectorRegistry {
    definitions: HashMap<String, CapabilityDefinition>,
}

impl ConnectorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a capability definition. Replaces any existing definition
    /// with the same `id`.
    pub fn register(&mut self, definition: CapabilityDefinition) {
        self.definitions.insert(definition.id.clone(), definition);
    }

    /// Look up a capability by ID.
    pub fn lookup(&self, capability_id: &str) -> Option<&CapabilityDefinition> {
        self.definitions.get(capability_id)
    }

    /// Remove a capability from the registry.
    pub fn unregister(&mut self, capability_id: &str) -> bool {
        self.definitions.remove(capability_id).is_some()
    }

    /// List all registered capability IDs.
    pub fn list_ids(&self) -> Vec<&str> {
        self.definitions.keys().map(|s| s.as_str()).collect()
    }

    /// Number of registered capabilities.
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Returns `true` if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability_runtime::definition::Provider;

    fn whisper_def() -> CapabilityDefinition {
        CapabilityDefinition::new(
            "whisper.transcribe",
            "Transcribe audio",
            Provider::HttpPost,
        )
        .with_cost(3)
        .with_rate_limit(60)
    }

    fn gemini_def() -> CapabilityDefinition {
        CapabilityDefinition::new(
            "gemini.vision",
            "Analyze images",
            Provider::HttpPost,
        )
        .with_cost(5)
    }

    #[test]
    fn test_register_and_lookup() {
        let mut reg = ConnectorRegistry::new();
        reg.register(whisper_def());

        let found = reg.lookup("whisper.transcribe");
        assert!(found.is_some());
        assert_eq!(found.unwrap().cost_per_call_cents, 3);
    }

    #[test]
    fn test_lookup_missing() {
        let reg = ConnectorRegistry::new();
        assert!(reg.lookup("nonexistent").is_none());
    }

    #[test]
    fn test_register_replaces() {
        let mut reg = ConnectorRegistry::new();
        reg.register(whisper_def());
        // Re-register with different cost
        reg.register(
            CapabilityDefinition::new("whisper.transcribe", "Updated", Provider::HttpPost)
                .with_cost(10),
        );
        assert_eq!(
            reg.lookup("whisper.transcribe").unwrap().cost_per_call_cents,
            10
        );
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn test_unregister() {
        let mut reg = ConnectorRegistry::new();
        reg.register(whisper_def());
        assert!(reg.unregister("whisper.transcribe"));
        assert!(!reg.unregister("whisper.transcribe")); // already gone
        assert!(reg.is_empty());
    }

    #[test]
    fn test_list_ids() {
        let mut reg = ConnectorRegistry::new();
        reg.register(whisper_def());
        reg.register(gemini_def());
        let mut ids = reg.list_ids();
        ids.sort();
        assert_eq!(ids, vec!["gemini.vision", "whisper.transcribe"]);
    }
}
