//! `zets-scopes-demo` — demonstrate the 6-graph architecture in action.
//!
//! Shows:
//!   1. Scope registry discovery
//!   2. Cascade query (Testing → User → Shared → Language → Data → System)
//!   3. Trust weighting per source
//!   4. Testing sandbox with staged change → test → verify → promote
//!   5. Decision log (audit trail per query)

use std::path::PathBuf;

use zets::scopes::router::{default_trust, LogEntry, ScopeRouter, TrustProfile};
use zets::scopes::{EncryptionTier, GraphScope, ScopeId, ScopePaths, ScopeRegistry};
use zets::testing_sandbox::{ChangeKind, TestResult, TestingSandbox};

fn main() {
    println!("═══ ZETS 6-Graph Architecture Demo ═══");
    println!();

    // ────────────────────────────────────────────────────────────
    // Phase 1: Build scope registry
    // ────────────────────────────────────────────────────────────
    println!("--- Phase 1: Scope Registry ---");
    let mut registry = ScopeRegistry::discover("data");

    // Add a user scope + testing scope
    let _ = registry.ensure_writable(ScopeId::User, "idan");
    let _ = registry.ensure_writable(ScopeId::Log, "");
    let _ = registry.ensure_writable(ScopeId::Testing, "session-001");

    println!("{}", registry.describe());

    // ────────────────────────────────────────────────────────────
    // Phase 2: Cascade query — "What's my favorite color?"
    // ────────────────────────────────────────────────────────────
    println!("--- Phase 2: Cascade Query — personal question ---");
    let router = ScopeRouter::new(&registry);
    let (result, log1) = router.cascade("my_favorite_color", |scope| {
        match scope.id {
            ScopeId::User => Some(("teal", 90)),   // user's personal data
            _ => None,
        }
    });
    print_cascade_result(&result, &log1);

    // ────────────────────────────────────────────────────────────
    // Phase 3: Cascade query — "What is DNA?"
    // ────────────────────────────────────────────────────────────
    println!("--- Phase 3: Cascade Query — factual question ---");
    let (result, log2) = router.cascade("what_is_DNA", |scope| {
        match scope.id {
            ScopeId::User => None,         // I don't have personal DNA info
            ScopeId::Data => Some(("molecule that carries genetic information", 80)),
            _ => None,
        }
    });
    print_cascade_result(&result, &log2);

    // ────────────────────────────────────────────────────────────
    // Phase 4: Cascade query — nothing found
    // ────────────────────────────────────────────────────────────
    println!("--- Phase 4: Cascade Query — unknown topic ---");
    let (result, log3) = router.cascade::<&str, _>("is_plutonium_tasty", |_| None);
    print_cascade_result(&result, &log3);

    // ────────────────────────────────────────────────────────────
    // Phase 5: Trust weighting
    // ────────────────────────────────────────────────────────────
    println!("--- Phase 5: Trust Weighting by source ---");
    for source in &[
        "wikipedia",
        "user_correction",
        "user_input",
        "extracted_from_article",
        "extracted_from_web",
        "unknown",
    ] {
        println!("  {:<30} default trust = {}", source, default_trust(source));
    }
    println!();

    // Simulate a source being corroborated
    let mut wiki = TrustProfile::new("wikipedia", default_trust("wikipedia"));
    for _ in 0..10 {
        wiki.record_use();
        wiki.record_corroborated();
    }
    wiki.recalibrate();
    println!("  After 10 corroborations, wikipedia trust = {}", wiki.weight);

    // Simulate a bad source
    let mut bad = TrustProfile::new("some_random_blog", 50);
    for _ in 0..10 {
        bad.record_use();
        bad.record_contradicted();
    }
    bad.recalibrate();
    println!("  After 10 contradictions, random_blog trust = {}", bad.weight);
    println!();

    // ────────────────────────────────────────────────────────────
    // Phase 6: Testing sandbox — stage → test → verify → promote
    // ────────────────────────────────────────────────────────────
    println!("--- Phase 6: Testing Sandbox workflow ---");
    let mut sandbox = TestingSandbox::new();

    // Stage a change: "DNA" learned from article
    let dna_id = sandbox.stage(
        ChangeKind::AddConcept {
            anchor: "DNA".into(),
            gloss: "molecule that carries genetic information".into(),
            pos: 1,
        },
        "Learned from article '\"Basics of Genetics\"'",
    );
    println!("  Staged: {} — status: Proposed", dna_id);

    // Run tests
    sandbox.record_test(&dna_id, TestResult {
        test_name: "conflict_check".into(),
        passed: true,
        detail: "no existing concept named 'DNA'".into(),
    });
    sandbox.record_test(&dna_id, TestResult {
        test_name: "trust_source".into(),
        passed: true,
        detail: "source confidence > 70".into(),
    });
    sandbox.record_test(&dna_id, TestResult {
        test_name: "regression_tests".into(),
        passed: true,
        detail: "existing queries still pass".into(),
    });
    println!("  Tests run: 3 passed, 0 failed");

    sandbox.finalize(&dna_id);
    println!("  Status after finalize: {:?}", sandbox.get(&dna_id).unwrap().status);

    // Stage a bad change that will fail
    let bad_id = sandbox.stage(
        ChangeKind::AddEdge { source: 1, target: 1, kind: 3 },
        "suspicious self-loop",
    );
    sandbox.record_test(&bad_id, TestResult {
        test_name: "cycle_check".into(),
        passed: false,
        detail: "would create self-loop".into(),
    });
    sandbox.finalize(&bad_id);
    println!("  Bad change {} status: {:?}", bad_id, sandbox.get(&bad_id).unwrap().status);

    // Promote the good change
    sandbox.mark_promoted(&dna_id);

    let stats = sandbox.stats();
    println!();
    println!("  Sandbox stats:");
    println!("    Total changes : {}", stats.total);
    println!("    Verified      : {}", stats.verified);
    println!("    Failed        : {}", stats.failed);
    println!("    Promoted      : {}", stats.promoted);
    println!();

    // ────────────────────────────────────────────────────────────
    // Phase 7: Log summary
    // ────────────────────────────────────────────────────────────
    println!("--- Phase 7: Decision Log Summary ---");
    for (i, log) in [&log1, &log2, &log3].iter().enumerate() {
        println!("  Query {}: '{}'", i + 1, log.query_text);
        println!("    Scopes checked: {}", log.resolution_trail.len());
        println!("    Final answer  : {}", log.final_answer_summary);
        println!("    Final trust   : {}", log.final_trust);
    }
    println!();

    println!("═══ Demo complete ═══");
    println!();
    println!("Summary:");
    println!("  * 6-graph architecture: System, User, Data, Language, Log, Testing");
    println!("  * Cascade query with priority: Testing → User → Shared → Lang → Data → System");
    println!("  * Trust weighting per source, learned over time");
    println!("  * Testing sandbox for safe self-modification");
    println!("  * Decision log for explainable AI");
}

fn print_cascade_result<T: std::fmt::Debug>(
    result: &zets::scopes::router::CascadeResult<T>,
    log: &LogEntry,
) {
    println!("  Query: '{}'", log.query_text);
    println!("  Trail:");
    for step in &result.trail {
        let marker = if step.found { "✓" } else { "·" };
        println!(
            "    {} {:<10} / {:<12} trust={:3} ({:>5}ns)",
            marker,
            step.scope.name(),
            step.instance,
            step.trust_contribution,
            step.latency_ns,
        );
    }
    if let Some(value) = &result.value {
        println!(
            "  → Answer: {:?}  (trust: {}, from: {})",
            value,
            result.trust,
            result.source_scope.map(|s| s.name()).unwrap_or("?")
        );
    } else {
        println!("  → Not found");
    }
    println!();
}
