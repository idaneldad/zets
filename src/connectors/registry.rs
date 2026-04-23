//! # ConnectorRegistry — indexed access to all connector procedures
//!
//! The registry holds all bundles and provides fast lookup by:
//!   - platform (e.g. "gmail" → bundle)
//!   - procedure path (e.g. "gmail.send" → procedure)
//!   - sense key (e.g. "communication.send.email" → list of matching procedures)
//!
//! This is the Reader's lookup target when a user asks "send an email".

use std::collections::HashMap;

use super::bundle::{ConnectorBundle, ConnectorProcedure};

/// The registry holds bundles and indexes for fast lookup.
pub struct ConnectorRegistry {
    bundles: Vec<ConnectorBundle>,
    /// platform name → bundle index
    by_platform: HashMap<String, usize>,
    /// sense_key → list of (bundle_idx, procedure_idx)
    by_sense: HashMap<String, Vec<(usize, usize)>>,
}

impl ConnectorRegistry {
    pub fn new() -> Self {
        ConnectorRegistry {
            bundles: Vec::new(),
            by_platform: HashMap::new(),
            by_sense: HashMap::new(),
        }
    }

    /// Load all seed bundles. Call at startup.
    pub fn with_seeds() -> Self {
        let mut reg = Self::new();
        for bundle in super::seed::all_seed_bundles() {
            reg.register(bundle);
        }
        reg
    }

    /// Add a bundle to the registry.
    pub fn register(&mut self, bundle: ConnectorBundle) {
        let bundle_idx = self.bundles.len();
        self.by_platform.insert(bundle.platform.clone(), bundle_idx);

        for (proc_idx, proc) in bundle.procedures.iter().enumerate() {
            for key in &proc.sense_keys {
                self.by_sense
                    .entry(key.clone())
                    .or_default()
                    .push((bundle_idx, proc_idx));
            }
        }
        self.bundles.push(bundle);
    }

    /// Find all bundles.
    pub fn bundles(&self) -> &[ConnectorBundle] {
        &self.bundles
    }

    /// Lookup bundle by platform name.
    pub fn get_bundle(&self, platform: &str) -> Option<&ConnectorBundle> {
        self.by_platform.get(platform).map(|&i| &self.bundles[i])
    }

    /// Lookup a specific procedure by "platform.procedure_id".
    pub fn get_procedure(&self, path: &str) -> Option<&ConnectorProcedure> {
        let (platform, id) = path.split_once('.')?;
        let bundle = self.get_bundle(platform)?;
        bundle.find(id)
    }

    /// Find procedures matching a sense key (exact or prefix).
    ///
    /// Returns `(platform, procedure)` pairs, sorted by specificity
    /// (exact match first, then prefix matches).
    pub fn find_by_sense(&self, key: &str) -> Vec<(&str, &ConnectorProcedure)> {
        let mut exact: Vec<(&str, &ConnectorProcedure)> = Vec::new();
        let mut prefix: Vec<(&str, &ConnectorProcedure)> = Vec::new();

        // Exact match
        if let Some(hits) = self.by_sense.get(key) {
            for &(b, p) in hits {
                let bundle = &self.bundles[b];
                if let Some(proc) = bundle.procedures.get(p) {
                    exact.push((bundle.platform.as_str(), proc));
                }
            }
        }

        // Prefix matches (excluding the exact match)
        for (sense_key, hits) in &self.by_sense {
            if sense_key != key && sense_key.starts_with(key) {
                for &(b, p) in hits {
                    let bundle = &self.bundles[b];
                    if let Some(proc) = bundle.procedures.get(p) {
                        prefix.push((bundle.platform.as_str(), proc));
                    }
                }
            }
        }

        exact.extend(prefix);
        exact
    }

    /// Stats for diagnostics.
    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            bundle_count: self.bundles.len(),
            procedure_count: self.bundles.iter().map(|b| b.procedure_count()).sum(),
            sense_key_count: self.by_sense.len(),
            total_bytes: self.bundles.iter().map(|b| b.total_size_bytes()).sum(),
        }
    }
}

impl Default for ConnectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RegistryStats {
    pub bundle_count: usize,
    pub procedure_count: usize,
    pub sense_key_count: usize,
    pub total_bytes: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_registry() {
        let r = ConnectorRegistry::new();
        assert_eq!(r.bundles().len(), 0);
        assert_eq!(r.stats().bundle_count, 0);
    }

    #[test]
    fn test_seed_registry_loads_9_bundles() {
        let r = ConnectorRegistry::with_seeds();
        assert_eq!(r.stats().bundle_count, 9);
        assert!(r.stats().procedure_count >= 20);
    }

    #[test]
    fn test_lookup_bundle_by_platform() {
        let r = ConnectorRegistry::with_seeds();
        assert!(r.get_bundle("gmail").is_some());
        assert!(r.get_bundle("slack").is_some());
        assert!(r.get_bundle("nonexistent").is_none());
    }

    #[test]
    fn test_lookup_procedure_by_path() {
        let r = ConnectorRegistry::with_seeds();
        let proc = r.get_procedure("gmail.send");
        assert!(proc.is_some());
        assert_eq!(proc.unwrap().id, "send");

        assert!(r.get_procedure("slack.message_send").is_some());
        assert!(r.get_procedure("gmail.nonexistent").is_none());
        assert!(r.get_procedure("nonexistent.anything").is_none());
    }

    #[test]
    fn test_find_by_sense_exact_hits() {
        let r = ConnectorRegistry::with_seeds();
        let hits = r.find_by_sense("communication.send.email");
        // Gmail.send + SMTP.send both answer this sense
        assert!(hits.len() >= 2);

        let platforms: Vec<&str> = hits.iter().map(|(p, _)| *p).collect();
        assert!(platforms.contains(&"gmail"));
        assert!(platforms.contains(&"smtp"));
    }

    #[test]
    fn test_find_by_sense_send_chat_multiple_platforms() {
        let r = ConnectorRegistry::with_seeds();
        let hits = r.find_by_sense("communication.send.chat");
        // Slack + Telegram + WhatsApp all answer this
        assert!(hits.len() >= 3);
    }

    #[test]
    fn test_stats_consistent() {
        let r = ConnectorRegistry::with_seeds();
        let s = r.stats();
        assert_eq!(s.bundle_count, r.bundles().len());
        assert!(s.sense_key_count > 0);
        assert!(s.total_bytes > 0);
    }

    #[test]
    fn test_total_storage_under_15kb() {
        let r = ConnectorRegistry::with_seeds();
        assert!(
            r.stats().total_bytes < 15_000,
            "all seeds take {} bytes",
            r.stats().total_bytes
        );
    }

    #[test]
    fn test_register_custom_bundle() {
        let mut r = ConnectorRegistry::new();
        let custom = super::super::bundle::ConnectorBundle::new(
            "custom_api",
            super::super::bundle::AuthKind::Bearer,
            "https://custom.test",
            "https://docs",
            60,
        );
        r.register(custom);
        assert!(r.get_bundle("custom_api").is_some());
    }
}
