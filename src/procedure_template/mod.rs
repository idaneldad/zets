//! # ProcedureTemplate — two-layer model for dedup-friendly procedure storage
//!
//! ## The problem Idan raised
//!
//! When we ingest procedures from code (GitHub repos, MCP specs, user
//! files), we see the SAME procedure written with different variable
//! names:
//!
//! ```text
//! Python:     requests.post(target_url, headers=my_headers, data=payload)
//! JavaScript: fetch(endpoint, { method: 'POST', headers: h, body: b })
//! Rust:       reqwest::post(uri).headers(hdrs).body(bytes).send()
//! ```
//!
//! These are all the SAME procedure (HTTP POST), just with different
//! surface names. We shouldn't store the name-variations as distinct
//! atoms — but we also can't throw away the naming, because:
//!
//!   - Some parameter names ARE the meaning (`customer_id`, not `arg1`)
//!   - Math names ARE meanings (`E = mc²`, not `a = b * c²`)
//!   - Different languages have different conventions
//!
//! ## The solution: Template + Instance
//!
//! **Template** (`template.rs`): the canonical shape. Params are ROLES
//! like `url`, `body`. One template per shape, deduped by shape_hash.
//! Describes what the procedure IS.
//!
//! **Instance** (`instance.rs`): a sighting of a template in specific
//! code. Binds template roles to local names, records language and
//! source. Duplicates within the same binding+language collapse via
//! sighting counter.
//!
//! **Registry** (`registry.rs`): the store. Dedups automatically.
//!
//! ## The name_role decision
//!
//! Each parameter carries a `NameRole`:
//!   - `Free`: name is cosmetic. `url` ≡ `endpoint` ≡ `target`.
//!     For most IO and network params.
//!   - `Domain`: name anchors to a business concept. `customer_id`
//!     refers to the customer_id atom in the graph.
//!   - `Convention(LoopCounter|Temporary|...)`: known convention.
//!   - `MathSymbol`: physics/math variables where the letter IS the
//!     meaning.
//!
//! When NameRole is `Free`, the instance layer stores binding purely
//! for source-traceability. When Domain or MathSymbol, the binding
//! connects to graph atoms.
//!
//! ## Storage efficiency (validated by tests)
//!
//! 1000 repos × 1 shape × 10 distinct binding patterns = 1 template
//! + 10 instances + 1000 sighting_count updates. NOT 1000 duplicate atoms.
//!
//! ## What this module does NOT do
//!
//! - **Execution**: templates describe, they don't run. That's the VM
//!   layer (future). ZETS stores KNOWLEDGE of procedures; execution
//!   happens via procedure_atom.rs steps + a future runtime.
//! - **Optimization** (loop vs recursion): that's a compiler's job.
//!   ZETS stores patterns as observed; optimization happens at the
//!   call site, not in the graph.
//! - **Code generation**: given a template and binding, producing
//!   target-language code is a separate resynthesis step (Phase C).

pub mod instance;
pub mod registry;
pub mod template;

pub use instance::{Language, ProcedureInstance, SourceLocation};
pub use registry::{RegisterOutcome, RegistryStats, TemplateRegistry};
pub use template::{
    Convention, NameRole, Parameter, ParamKind, ProcedureTemplate, SideEffect, TemplateId,
};

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn mk_param(role: &str, kind: ParamKind, required: bool, name_role: NameRole) -> Parameter {
        Parameter {
            role: role.into(),
            kind,
            required,
            name_role,
        }
    }

    #[test]
    fn test_scenario_cross_language_http() {
        // Full scenario: register http.post template, ingest sightings
        // from 4 different languages and 3 different source repos each.
        let mut reg = TemplateRegistry::new();

        let http_post = ProcedureTemplate::new("http.post", "HTTP POST request")
            .with_param(mk_param("url", ParamKind::Url, true, NameRole::Free))
            .with_param(mk_param("headers", ParamKind::Headers, false, NameRole::Free))
            .with_param(mk_param("body", ParamKind::Body, false, NameRole::Free))
            .with_output(mk_param("status", ParamKind::Number, true, NameRole::Free))
            .with_side_effect(SideEffect::NetworkWrite)
            .with_side_effect(SideEffect::NetworkRead);

        reg.register_template(http_post);

        // Ingest sightings
        let template_id = TemplateId::new("http.post");
        for (lang, bindings) in [
            (
                Language::Python,
                vec![
                    ("target_url", "my_headers", "payload"),
                    ("target_url", "my_headers", "payload"), // dupe
                    ("endpoint", "h", "data"),
                ],
            ),
            (
                Language::JavaScript,
                vec![
                    ("endpoint", "h", "b"),
                    ("url", "headers", "body"),
                ],
            ),
            (
                Language::Rust,
                vec![("uri", "hdrs", "bytes")],
            ),
        ] {
            for (url, headers, body) in bindings {
                reg.register_instance(
                    ProcedureInstance::new(
                        template_id.clone(),
                        SourceLocation::new("github.com/x/y"),
                        lang.clone(),
                        1000,
                    )
                    .bind("url", url)
                    .bind("headers", headers)
                    .bind("body", body),
                );
            }
        }

        let stats = reg.stats();

        // 1 template
        assert_eq!(stats.templates, 1);
        // 5 distinct instances (2 Python bindings dedup to 1 → 2 Py, 2 JS, 1 Rust = 5)
        assert_eq!(stats.instances, 5);
        // 6 total sightings (one Py binding seen twice)
        assert_eq!(stats.total_sightings, 6);
        // 3 languages
        assert_eq!(stats.distinct_languages, 3);
    }

    #[test]
    fn test_scenario_math_template_preserved() {
        // E = mc² — parameter names ARE the meaning.
        // We still factor common math templates: "binary operation on numbers"
        // is generic, but mass_energy_equivalence has domain-anchored names.
        let mut reg = TemplateRegistry::new();

        let emc2 = ProcedureTemplate::new(
            "physics.mass_energy_equivalence",
            "E = m · c²",
        )
        .with_param(mk_param(
            "mass",
            ParamKind::Number,
            true,
            NameRole::MathSymbol,
        ))
        .with_param(mk_param(
            "speed_of_light",
            ParamKind::Number,
            true,
            NameRole::MathSymbol,
        ))
        .with_output(mk_param(
            "energy",
            ParamKind::Number,
            true,
            NameRole::MathSymbol,
        ));

        assert!(emc2.is_pure());

        // Sightings across physics textbooks, each with slightly different
        // notation. In physics, "m", "mass", "M" all map to same role.
        let template_id = emc2.id.clone();
        reg.register_template(emc2);

        reg.register_instance(
            ProcedureInstance::new(
                template_id.clone(),
                SourceLocation::new("wikipedia"),
                Language::Math,
                1000,
            )
            .bind("mass", "m")
            .bind("speed_of_light", "c"),
        );
        reg.register_instance(
            ProcedureInstance::new(
                template_id.clone(),
                SourceLocation::new("textbook_A"),
                Language::Math,
                1000,
            )
            .bind("mass", "M")
            .bind("speed_of_light", "c"),
        );

        assert_eq!(reg.instance_count(), 2);
        assert_eq!(reg.template_count(), 1);
    }

    #[test]
    fn test_domain_param_does_not_collapse() {
        // billing.lookup_invoice and customer.lookup_by_email are different
        // Domain-anchored procedures. Even with similar shape they shouldn't
        // collapse — the role name is part of the semantic identity.
        let mut reg = TemplateRegistry::new();

        let lookup_invoice = ProcedureTemplate::new("billing.lookup_invoice", "")
            .with_param(mk_param(
                "customer_id",
                ParamKind::AtomRef,
                true,
                NameRole::Domain,
            ))
            .with_side_effect(SideEffect::GraphRead);

        let lookup_by_email = ProcedureTemplate::new("customer.lookup_by_email", "")
            .with_param(mk_param(
                "email",
                ParamKind::Text,
                true,
                NameRole::Domain,
            ))
            .with_side_effect(SideEffect::GraphRead);

        assert_eq!(reg.register_template(lookup_invoice), RegisterOutcome::New);
        assert_eq!(reg.register_template(lookup_by_email), RegisterOutcome::New);

        assert_eq!(reg.template_count(), 2);
    }

    #[test]
    fn test_realistic_scale() {
        let mut reg = TemplateRegistry::new();

        // 20 templates across common operations
        let templates = [
            ("http.get", vec!["url", "headers"], vec![SideEffect::NetworkRead]),
            ("http.post", vec!["url", "headers", "body", "method_post"], vec![SideEffect::NetworkWrite, SideEffect::NetworkRead]),
            ("http.put", vec!["url", "headers", "body", "method_put"], vec![SideEffect::NetworkWrite, SideEffect::NetworkRead]),
            ("http.delete", vec!["url", "headers"], vec![SideEffect::NetworkWrite]),
            ("fs.read", vec!["path"], vec![SideEffect::FileRead]),
            ("fs.write", vec!["path", "content"], vec![SideEffect::FileWrite]),
            ("json.parse", vec!["input"], vec![SideEffect::Pure]),
            ("json.serialize", vec!["value"], vec![SideEffect::Pure]),
        ];

        for (id, params, effects) in templates {
            let mut t = ProcedureTemplate::new(id, "");
            for p in params {
                t = t.with_param(mk_param(p, ParamKind::Opaque, true, NameRole::Free));
            }
            for e in effects {
                t = t.with_side_effect(e);
            }
            reg.register_template(t);
        }

        assert_eq!(reg.template_count(), 8);

        // Simulate ingest of 10,000 sightings across these templates
        // with 50 different binding patterns per template
        let ids: Vec<TemplateId> = reg.templates_by_id_keys();

        for id in &ids {
            for binding_idx in 0..50 {
                for _sighting in 0..25 {
                    // Different sources & times, same binding → dedups via touch
                    reg.register_instance(
                        ProcedureInstance::new(
                            id.clone(),
                            SourceLocation::new(format!("sighting_{}", binding_idx)),
                            Language::Python,
                            1000,
                        )
                        .bind("url", format!("binding_{}", binding_idx)),
                    );
                }
            }
        }

        let s = reg.stats();
        // 8 templates × 50 bindings = 400 instances
        assert_eq!(s.instances, 400);
        // 400 × 25 sightings = 10,000
        assert_eq!(s.total_sightings, 10_000);
        // Avg 25 sightings/instance
        assert!((s.avg_sightings_per_instance - 25.0).abs() < 0.01);
    }
}

#[cfg(test)]
impl TemplateRegistry {
    fn templates_by_id_keys(&self) -> Vec<TemplateId> {
        self.templates_by_id.keys().cloned().collect()
    }
}
