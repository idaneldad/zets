//! # TemplateRegistry — deduplicating store for templates + instances
//!
//! Templates are keyed by shape_hash — same shape → same atom.
//! Instances are keyed by dedup_key — same template+binding+language → merge.
//!
//! Storage is O(unique shapes + unique bindings), not O(sightings).

use std::collections::HashMap;

use super::instance::{Language, ProcedureInstance, SourceLocation};
use super::template::{ProcedureTemplate, TemplateId};

/// Result of registering a template.
#[derive(Debug, PartialEq, Eq)]
pub enum RegisterOutcome {
    /// Entirely new shape — registered.
    New,
    /// Same shape as existing template (dedup).
    DuplicateShape(TemplateId),
    /// Same id but different shape — rejected; ids must be stable.
    IdConflict,
}

#[derive(Debug, Default)]
pub struct TemplateRegistry {
    /// id → template (one per id).
    pub(crate) templates_by_id: HashMap<TemplateId, ProcedureTemplate>,
    /// shape_hash → canonical id (first template with this shape).
    shape_index: HashMap<u64, TemplateId>,
    /// dedup_key → instance (one per unique binding per template per language).
    instances: HashMap<String, ProcedureInstance>,
    /// template_id → set of instance dedup_keys using this template.
    instances_by_template: HashMap<TemplateId, Vec<String>>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a template. Returns outcome indicating dedup status.
    pub fn register_template(&mut self, t: ProcedureTemplate) -> RegisterOutcome {
        // Same id with different shape? Reject.
        if let Some(existing) = self.templates_by_id.get(&t.id) {
            if existing.shape_hash != t.shape_hash {
                return RegisterOutcome::IdConflict;
            }
            // Same id + same shape → already registered
            return RegisterOutcome::DuplicateShape(existing.id.clone());
        }

        // Same shape under different id?
        if let Some(canonical_id) = self.shape_index.get(&t.shape_hash).cloned() {
            return RegisterOutcome::DuplicateShape(canonical_id);
        }

        // Truly new
        self.shape_index.insert(t.shape_hash, t.id.clone());
        self.templates_by_id.insert(t.id.clone(), t);
        RegisterOutcome::New
    }

    pub fn get_template(&self, id: &TemplateId) -> Option<&ProcedureTemplate> {
        self.templates_by_id.get(id)
    }

    /// Find a template by shape hash (for dedup queries).
    pub fn find_by_shape(&self, shape_hash: u64) -> Option<&ProcedureTemplate> {
        let id = self.shape_index.get(&shape_hash)?;
        self.templates_by_id.get(id)
    }

    pub fn template_count(&self) -> usize {
        self.templates_by_id.len()
    }

    /// Register an instance. If dedup_key already exists, the existing
    /// instance gets a sighting touch. Returns the dedup key used.
    pub fn register_instance(&mut self, inst: ProcedureInstance) -> String {
        let key = inst.dedup_key();
        let template_id = inst.template_id.clone();

        if let Some(existing) = self.instances.get_mut(&key) {
            existing.touch(inst.last_seen_ms);
            return key;
        }

        self.instances.insert(key.clone(), inst);
        self.instances_by_template
            .entry(template_id)
            .or_insert_with(Vec::new)
            .push(key.clone());
        key
    }

    pub fn get_instance(&self, dedup_key: &str) -> Option<&ProcedureInstance> {
        self.instances.get(dedup_key)
    }

    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    /// Total sightings across all instances — may be much higher than
    /// instance_count when patterns repeat.
    pub fn total_sightings(&self) -> u64 {
        self.instances
            .values()
            .map(|i| i.sighting_count as u64)
            .sum()
    }

    /// All instances that use the given template.
    pub fn instances_of(&self, template_id: &TemplateId) -> Vec<&ProcedureInstance> {
        self.instances_by_template
            .get(template_id)
            .map(|keys| {
                keys.iter()
                    .filter_map(|k| self.instances.get(k))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// All instances in a specific language.
    pub fn instances_in(&self, lang: &Language) -> Vec<&ProcedureInstance> {
        self.instances
            .values()
            .filter(|i| &i.language == lang)
            .collect()
    }

    /// Most-sighted instances — the patterns that repeat most.
    pub fn top_sighted(&self, n: usize) -> Vec<&ProcedureInstance> {
        let mut all: Vec<&ProcedureInstance> = self.instances.values().collect();
        all.sort_by(|a, b| b.sighting_count.cmp(&a.sighting_count));
        all.into_iter().take(n).collect()
    }

    /// Stats.
    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            templates: self.template_count(),
            instances: self.instance_count(),
            total_sightings: self.total_sightings(),
            distinct_languages: self
                .instances
                .values()
                .map(|i| i.language.as_str())
                .collect::<std::collections::HashSet<_>>()
                .len(),
            avg_sightings_per_instance: if self.instance_count() > 0 {
                self.total_sightings() as f64 / self.instance_count() as f64
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub templates: usize,
    pub instances: usize,
    pub total_sightings: u64,
    pub distinct_languages: usize,
    pub avg_sightings_per_instance: f64,
}

#[cfg(test)]
mod tests {
    use super::super::template::{Parameter, ParamKind, NameRole, SideEffect};
    use super::*;

    fn mk_param(role: &str, kind: ParamKind, required: bool) -> Parameter {
        Parameter {
            role: role.into(),
            kind,
            required,
            name_role: NameRole::Free,
        }
    }

    fn mk_http_post() -> ProcedureTemplate {
        ProcedureTemplate::new("http.post", "HTTP POST request")
            .with_param(mk_param("url", ParamKind::Url, true))
            .with_param(mk_param("headers", ParamKind::Headers, false))
            .with_param(mk_param("body", ParamKind::Body, false))
            .with_side_effect(SideEffect::NetworkWrite)
    }

    #[test]
    fn test_register_new_template() {
        let mut reg = TemplateRegistry::new();
        let t = mk_http_post();
        assert_eq!(reg.register_template(t), RegisterOutcome::New);
        assert_eq!(reg.template_count(), 1);
    }

    #[test]
    fn test_register_duplicate_shape_different_id() {
        let mut reg = TemplateRegistry::new();
        reg.register_template(mk_http_post());

        // Same shape, different id
        let duplicate = ProcedureTemplate::new("different_name.post", "")
            .with_param(mk_param("url", ParamKind::Url, true))
            .with_param(mk_param("headers", ParamKind::Headers, false))
            .with_param(mk_param("body", ParamKind::Body, false))
            .with_side_effect(SideEffect::NetworkWrite);

        let outcome = reg.register_template(duplicate);
        assert!(matches!(outcome, RegisterOutcome::DuplicateShape(_)));
        // Only one template stored
        assert_eq!(reg.template_count(), 1);
    }

    #[test]
    fn test_register_id_conflict_rejected() {
        let mut reg = TemplateRegistry::new();
        reg.register_template(mk_http_post());

        // Same id, different shape → IdConflict
        let conflict = ProcedureTemplate::new("http.post", "different")
            .with_param(mk_param("completely", ParamKind::Text, true));

        assert_eq!(reg.register_template(conflict), RegisterOutcome::IdConflict);
        assert_eq!(reg.template_count(), 1);
    }

    #[test]
    fn test_register_instance_first_time() {
        let mut reg = TemplateRegistry::new();
        reg.register_template(mk_http_post());

        let inst = ProcedureInstance::new(
            TemplateId::new("http.post"),
            SourceLocation::new("repo1"),
            Language::Python,
            1000,
        )
        .bind("url", "url")
        .bind("body", "body");

        let key = reg.register_instance(inst);
        assert!(!key.is_empty());
        assert_eq!(reg.instance_count(), 1);
        assert_eq!(reg.total_sightings(), 1);
    }

    #[test]
    fn test_duplicate_instance_deduped() {
        let mut reg = TemplateRegistry::new();
        reg.register_template(mk_http_post());

        // Same binding observed twice
        let make_inst = |ts| {
            ProcedureInstance::new(
                TemplateId::new("http.post"),
                SourceLocation::new(format!("repo_{}", ts)),
                Language::Python,
                ts,
            )
            .bind("url", "url")
            .bind("body", "body")
        };

        reg.register_instance(make_inst(1000));
        reg.register_instance(make_inst(2000));
        reg.register_instance(make_inst(3000));

        // Still one instance, but sighting count == 3
        assert_eq!(reg.instance_count(), 1);
        assert_eq!(reg.total_sightings(), 3);
    }

    #[test]
    fn test_different_languages_different_instances() {
        let mut reg = TemplateRegistry::new();
        reg.register_template(mk_http_post());

        let template = TemplateId::new("http.post");

        reg.register_instance(
            ProcedureInstance::new(template.clone(), SourceLocation::new("a"), Language::Python, 1000)
                .bind("url", "url"),
        );
        reg.register_instance(
            ProcedureInstance::new(template.clone(), SourceLocation::new("b"), Language::JavaScript, 1000)
                .bind("url", "url"),
        );
        reg.register_instance(
            ProcedureInstance::new(template, SourceLocation::new("c"), Language::Rust, 1000)
                .bind("url", "url"),
        );

        assert_eq!(reg.instance_count(), 3);
        assert_eq!(reg.total_sightings(), 3);
    }

    #[test]
    fn test_instances_of_template() {
        let mut reg = TemplateRegistry::new();
        reg.register_template(mk_http_post());

        let template = TemplateId::new("http.post");

        for i in 0..5 {
            reg.register_instance(
                ProcedureInstance::new(
                    template.clone(),
                    SourceLocation::new(format!("repo{}", i)),
                    Language::Python,
                    1000,
                )
                .bind("url", format!("url_{}", i)),
            );
        }

        let instances = reg.instances_of(&template);
        assert_eq!(instances.len(), 5);
    }

    #[test]
    fn test_top_sighted() {
        let mut reg = TemplateRegistry::new();
        reg.register_template(mk_http_post());

        let template = TemplateId::new("http.post");

        // Pattern A: 10 sightings
        for _ in 0..10 {
            reg.register_instance(
                ProcedureInstance::new(
                    template.clone(),
                    SourceLocation::new("a"),
                    Language::Python,
                    1000,
                )
                .bind("url", "endpoint"),
            );
        }
        // Pattern B: 3 sightings
        for _ in 0..3 {
            reg.register_instance(
                ProcedureInstance::new(
                    template.clone(),
                    SourceLocation::new("b"),
                    Language::Python,
                    1000,
                )
                .bind("url", "api_url"),
            );
        }
        // Pattern C: 1 sighting
        reg.register_instance(
            ProcedureInstance::new(template, SourceLocation::new("c"), Language::Python, 1000)
                .bind("url", "weird_name"),
        );

        let top = reg.top_sighted(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].sighting_count, 10);
        assert_eq!(top[1].sighting_count, 3);
    }

    #[test]
    fn test_stats() {
        let mut reg = TemplateRegistry::new();
        reg.register_template(mk_http_post());

        let template = TemplateId::new("http.post");
        reg.register_instance(
            ProcedureInstance::new(
                template.clone(),
                SourceLocation::new("a"),
                Language::Python,
                1000,
            )
            .bind("url", "a"),
        );
        reg.register_instance(
            ProcedureInstance::new(
                template.clone(),
                SourceLocation::new("a"),
                Language::Python,
                1000,
            )
            .bind("url", "a"),
        );
        reg.register_instance(
            ProcedureInstance::new(template, SourceLocation::new("b"), Language::Rust, 1000)
                .bind("url", "b"),
        );

        let s = reg.stats();
        assert_eq!(s.templates, 1);
        assert_eq!(s.instances, 2);
        assert_eq!(s.total_sightings, 3);
        assert_eq!(s.distinct_languages, 2);
        assert!(s.avg_sightings_per_instance > 1.0);
    }

    #[test]
    fn test_storage_efficiency_scenario() {
        // Realistic scenario: 1000 GitHub repos all do http.post,
        // each with slightly different variable names.
        let mut reg = TemplateRegistry::new();
        reg.register_template(mk_http_post());
        let template = TemplateId::new("http.post");

        // 10 distinct binding patterns, 100 sightings each
        let patterns = [
            ("url", "url"),
            ("url", "endpoint"),
            ("url", "target_url"),
            ("url", "api_url"),
            ("url", "request_url"),
            ("url", "uri"),
            ("url", "address"),
            ("url", "link"),
            ("url", "webhook_url"),
            ("url", "base_url"),
        ];

        for (role, name) in patterns {
            for i in 0..100 {
                reg.register_instance(
                    ProcedureInstance::new(
                        template.clone(),
                        SourceLocation::new(format!("repo{}", i)),
                        Language::Python,
                        1000,
                    )
                    .bind(role, name),
                );
            }
        }

        // 10 distinct instances (by binding), 1000 total sightings
        assert_eq!(reg.instance_count(), 10);
        assert_eq!(reg.total_sightings(), 1000);
        // Still just 1 template
        assert_eq!(reg.template_count(), 1);
    }
}
