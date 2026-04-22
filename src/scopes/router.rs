//! ScopeRouter — cascade query resolution across scopes.
//!
//! When a query arrives, we resolve it by walking through scopes in priority
//! order (Testing → User → Shared → Language → Data → System). The first scope
//! that has a confident answer wins. If none do, we return "not found".
//!
//! CRITICAL: every resolution step is recorded as a LogEntry in the Log scope,
//! so ZETS can answer "why did you say that?" — explainability by design.

use std::time::{SystemTime, UNIX_EPOCH};

use super::{GraphScope, ScopeId, ScopeRegistry};

/// Result of a cascade query — includes the trust chain.
#[derive(Debug, Clone)]
pub struct CascadeResult<T> {
    /// The answer, if found.
    pub value: Option<T>,
    /// Which scope provided the answer.
    pub source_scope: Option<ScopeId>,
    /// How many scopes we checked before finding (or giving up).
    pub scopes_checked: usize,
    /// Trust weight of the final answer (0-100).
    pub trust: u8,
    /// Step-by-step audit of what we tried.
    pub trail: Vec<ResolutionStep>,
}

#[derive(Debug, Clone)]
pub struct ResolutionStep {
    pub scope: ScopeId,
    pub instance: String,
    pub found: bool,
    pub trust_contribution: u8,
    pub latency_ns: u64,
}

/// A LogEntry — recorded for every query that goes through the router.
/// Written to the Log scope (append-only, signed).
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp_ms: u64,
    pub query_text: String,
    pub resolution_trail: Vec<ResolutionStep>,
    pub final_answer_summary: String,
    pub final_trust: u8,
    /// Concept IDs that contributed to the answer.
    pub contributing_concepts: Vec<u32>,
    /// Route IDs that were executed.
    pub routes_invoked: Vec<u32>,
}

impl LogEntry {
    pub fn new(query: impl Into<String>) -> Self {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Self {
            timestamp_ms,
            query_text: query.into(),
            resolution_trail: Vec::new(),
            final_answer_summary: String::new(),
            final_trust: 0,
            contributing_concepts: Vec::new(),
            routes_invoked: Vec::new(),
        }
    }
}

/// Cascade router — walks scopes in priority order, asking each to resolve.
pub struct ScopeRouter<'a> {
    registry: &'a ScopeRegistry,
}

impl<'a> ScopeRouter<'a> {
    pub fn new(registry: &'a ScopeRegistry) -> Self {
        Self { registry }
    }

    /// Execute a cascade query. `resolver` is called per scope, returns Some(T)
    /// if that scope can answer. First success wins.
    pub fn cascade<T, F>(
        &self,
        query_description: &str,
        mut resolver: F,
    ) -> (CascadeResult<T>, LogEntry)
    where
        F: FnMut(&GraphScope) -> Option<(T, u8)>,  // (value, trust 0-100)
    {
        let mut log = LogEntry::new(query_description);
        let mut trail = Vec::new();
        let mut scopes_checked = 0;

        for scope in self.registry.all() {
            // Skip Log and System from factual cascade
            if scope.id == ScopeId::Log || scope.id == ScopeId::System {
                continue;
            }
            let t0 = std::time::Instant::now();
            let outcome = resolver(scope);
            let latency = t0.elapsed().as_nanos() as u64;
            scopes_checked += 1;

            let (value, trust) = match outcome {
                Some((v, t)) => (Some(v), t),
                None => (None, 0),
            };

            let step = ResolutionStep {
                scope: scope.id,
                instance: scope.instance_name.clone(),
                found: value.is_some(),
                trust_contribution: trust,
                latency_ns: latency,
            };
            trail.push(step.clone());
            log.resolution_trail.push(step);

            if let Some(v) = value {
                log.final_trust = trust;
                log.final_answer_summary = format!("from {}", scope.qualified_name());
                return (
                    CascadeResult {
                        value: Some(v),
                        source_scope: Some(scope.id),
                        scopes_checked,
                        trust,
                        trail,
                    },
                    log,
                );
            }
        }

        // Nothing found
        log.final_answer_summary = "not found".to_string();
        (
            CascadeResult {
                value: None,
                source_scope: None,
                scopes_checked,
                trust: 0,
                trail,
            },
            log,
        )
    }

    pub fn registry(&self) -> &ScopeRegistry {
        self.registry
    }
}

/// Trust weight — separate from the cascade, this is "how much do I believe
/// this source?" Stored per-source-id, learned over time.
#[derive(Debug, Clone)]
pub struct TrustProfile {
    /// Source identifier (e.g., "wikipedia", "user_input", "extracted_from_article_123")
    pub source_id: String,
    /// 0-100. 50 = neutral, 90 = very trusted, 10 = barely trusted.
    pub weight: u8,
    /// How many times we've used this source.
    pub use_count: u64,
    /// How often has it been corroborated by other sources?
    pub corroboration_count: u64,
    /// How often has it been contradicted?
    pub contradiction_count: u64,
}

impl TrustProfile {
    pub fn new(source: impl Into<String>, initial_weight: u8) -> Self {
        Self {
            source_id: source.into(),
            weight: initial_weight,
            use_count: 0,
            corroboration_count: 0,
            contradiction_count: 0,
        }
    }

    /// Adjust weight based on accumulated evidence.
    pub fn recalibrate(&mut self) {
        if self.use_count == 0 {
            return;
        }
        let support_ratio = if self.corroboration_count + self.contradiction_count == 0 {
            0.5
        } else {
            self.corroboration_count as f64
                / (self.corroboration_count + self.contradiction_count) as f64
        };
        // Nudge weight toward support_ratio * 100, with inertia
        let target = (support_ratio * 100.0) as i32;
        let delta = (target - self.weight as i32).clamp(-5, 5);
        self.weight = ((self.weight as i32) + delta).clamp(0, 100) as u8;
    }

    pub fn record_use(&mut self) {
        self.use_count += 1;
    }

    pub fn record_corroborated(&mut self) {
        self.corroboration_count += 1;
    }

    pub fn record_contradicted(&mut self) {
        self.contradiction_count += 1;
    }
}

/// Default trust weights by source type.
pub fn default_trust(source_kind: &str) -> u8 {
    match source_kind {
        "wikipedia" => 80,
        "wiktionary" => 75,
        "curated_bundle" => 85,
        "user_correction" => 90,  // user corrections are authoritative for that user
        "user_input" => 60,
        "extracted_from_article" => 45,
        "extracted_from_web" => 35,
        "learned_from_corpus" => 50,
        "inferred" => 40,
        "unknown" => 25,
        _ => 50,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::ScopePaths;
    use std::path::PathBuf;

    fn build_registry() -> ScopeRegistry {
        let paths = ScopePaths::new("/tmp/test_router");
        let mut reg = ScopeRegistry::new(paths);
        reg.register(GraphScope::new(ScopeId::User, "idan", PathBuf::from("/a")));
        reg.register(GraphScope::new(ScopeId::Data, "universal", PathBuf::from("/b")));
        reg.register(GraphScope::new(ScopeId::Language, "he", PathBuf::from("/c")));
        reg
    }

    #[test]
    fn cascade_user_wins_over_data() {
        let reg = build_registry();
        let router = ScopeRouter::new(&reg);
        let (result, _log) = router.cascade("test_query", |scope| {
            match scope.id {
                ScopeId::User => Some(("personal answer", 90)),
                ScopeId::Data => Some(("universal answer", 60)),
                _ => None,
            }
        });
        assert_eq!(result.value, Some("personal answer"));
        assert_eq!(result.source_scope, Some(ScopeId::User));
        assert_eq!(result.trust, 90);
    }

    #[test]
    fn cascade_falls_through_to_data() {
        let reg = build_registry();
        let router = ScopeRouter::new(&reg);
        let (result, log) = router.cascade("test", |scope| {
            if scope.id == ScopeId::Data {
                Some(("universal", 60))
            } else {
                None
            }
        });
        assert_eq!(result.value, Some("universal"));
        assert!(result.scopes_checked >= 2);
        assert!(log.resolution_trail.len() >= 2);
    }

    #[test]
    fn cascade_no_match() {
        let reg = build_registry();
        let router = ScopeRouter::new(&reg);
        let (result, log) = router.cascade::<&str, _>("test", |_| None);
        assert_eq!(result.value, None);
        assert_eq!(log.final_answer_summary, "not found");
    }

    #[test]
    fn trust_recalibrates_up() {
        let mut p = TrustProfile::new("wiki", 50);
        p.use_count = 10;
        p.corroboration_count = 9;
        p.contradiction_count = 1;
        p.recalibrate();
        assert!(p.weight > 50);
    }

    #[test]
    fn trust_recalibrates_down() {
        let mut p = TrustProfile::new("bad_source", 80);
        p.use_count = 10;
        p.corroboration_count = 1;
        p.contradiction_count = 9;
        p.recalibrate();
        assert!(p.weight < 80);
    }

    #[test]
    fn default_trust_known_sources() {
        assert!(default_trust("wikipedia") > default_trust("extracted_from_web"));
        assert!(default_trust("user_correction") > default_trust("user_input"));
    }
}
