//! `live_brain_demo` — the full 11-tree loop in one deterministic run.
//!
//! Simulates Tamar learning about Laravel over several conversations:
//!   Conversation 1: she asks about Laravel MVC → Precision mode wins
//!   Conversation 2: she asks for a creative solution → Divergent wins
//!   Conversation 3: she needs regulation after a bug → skills strengthen
//!
//! Each turn:
//!   1. Session activates atoms (tree: session)
//!   2. Scenario subgraph created (tree: scenario)
//!   3. Spreading activation finds candidates (tree: cognitive modes + context)
//!   4. Dreaming proposes new edges (tree 8: ADHD-like)
//!   5. Evaluation gates which pass (tree 9)
//!   6. Skills reinforce on success (tree 10)
//!   7. Meta-learner updates posterior (tree 11)

use zets::atoms::{AtomKind, AtomStore};
use zets::dreaming::{propose_via_two_hop, evaluate};
use zets::meta_learning::{CognitiveMode, MetaLearner, query_hash};
use zets::relations;
use zets::scenario::ScenarioBuilder;
use zets::session::SessionContext;
use zets::skills::{
    attach_skill, people_with_skill, proficiency_of, register_skill,
    reinforce_skill, skill_solves, skills_of, skill_label, Proficiency,
};
use zets::spreading_activation::{spread_from_session, SpreadConfig};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  ZETS Live Brain Demo — the full 11-tree loop               ║");
    println!("║  Deterministic, no LLM, no randomness.                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let mut store = AtomStore::new();
    let mut meta = MetaLearner::new();

    // ── Seed: Tamar + initial domain atoms ──
    let tamar = store.put(AtomKind::Concept, b"Tamar".to_vec());
    let laravel = store.put(AtomKind::Concept, b"Laravel".to_vec());
    let mvc = store.put(AtomKind::Concept, b"MVC_pattern".to_vec());
    let controller = store.put(AtomKind::Concept, b"Controller".to_vec());
    let service_container = store.put(AtomKind::Concept, b"service_container".to_vec());
    let problem_bug = store.put(AtomKind::Concept, b"undefined_method_bug".to_vec());
    let breathing = store.put(AtomKind::Concept, b"mindful_breathing".to_vec());
    let stress_event = store.put(AtomKind::Concept, b"deploy_failure_stress".to_vec());

    // Initial relationships (a Laravel knowledge prior).
    // Deeper topology gives spreading activation somewhere to go beyond the seeds.
    let is_a = relations::by_name("is_a").unwrap().code;
    let near = relations::by_name("near").unwrap().code;
    let used_for = relations::by_name("used_for").unwrap().code;
    let requires = relations::by_name("requires").unwrap().code;

    // Extra domain atoms for a richer neighborhood
    let eloquent = store.put(AtomKind::Concept, b"Eloquent_ORM".to_vec());
    let request_handling = store.put(AtomKind::Concept, b"request_handling".to_vec());
    let middleware = store.put(AtomKind::Concept, b"middleware".to_vec());
    let dependency_injection = store.put(AtomKind::Concept, b"dependency_injection".to_vec());
    let php = store.put(AtomKind::Concept, b"PHP".to_vec());

    store.link(controller, mvc, is_a, 90, 0);
    store.link(controller, request_handling, used_for, 85, 0);
    store.link(request_handling, middleware, near, 75, 0);
    store.link(service_container, laravel, near, 85, 0);
    store.link(service_container, dependency_injection, is_a, 95, 0);
    store.link(mvc, laravel, near, 95, 0);
    store.link(mvc, controller, near, 80, 0);
    store.link(laravel, eloquent, near, 88, 0);
    store.link(laravel, php, requires, 98, 0);
    store.link(eloquent, php, requires, 95, 0);
    store.link(problem_bug, laravel, near, 70, 0);
    store.link(problem_bug, php, near, 60, 0);
    store.link(stress_event, breathing, near, 70, 0);

    // Register skills Tamar has
    let laravel_skill = register_skill(&mut store, "laravel_mvc");
    let debug_skill = register_skill(&mut store, "laravel_debugging");
    let regulate_skill = register_skill(&mut store, "emotion_regulation_breathing");

    attach_skill(&mut store, tamar, laravel_skill, 45);  // Developing
    attach_skill(&mut store, tamar, debug_skill, 30);    // Novice
    attach_skill(&mut store, tamar, regulate_skill, 55); // Developing

    skill_solves(&mut store, laravel_skill, mvc);
    skill_solves(&mut store, debug_skill, problem_bug);
    skill_solves(&mut store, regulate_skill, stress_event);

    println!("─── Initial state ───");
    print_skills(&store, tamar);
    println!();

    // ═══════════════════════════════════════════════════
    // CONVERSATION 1 — factual MVC question (Precision wins)
    // ═══════════════════════════════════════════════════
    println!("━━━ Conversation 1: factual ━━━");
    println!("Tamar asks: 'איך controller עובד ב-MVC?'");

    let mut session = SessionContext::new();
    session.mention(laravel);
    session.mention(mvc);
    session.mention(controller);
    session.advance_turn();

    let activation = spread_from_session(&store, &session, &SpreadConfig::precise(), 5);
    let top: Vec<_> = activation.top_k_novel(&session.active_ids(), 5);
    println!("  Spreading activation top-5 (Precision preset, novel atoms beyond seeds):");
    if top.is_empty() {
        println!("    (no novel atoms reached — session is the whole visible subgraph)");
    } else {
        for (id, score) in &top {
            let atom = store.get(*id).unwrap();
            let label = std::str::from_utf8(&atom.data).unwrap_or("?");
            println!("    {:.3}  {}", score, label);
        }
    }

    // Create scenario for this conversation
    let sc1 = ScenarioBuilder::new(&mut store, tamar, 1_000_000, "mvc question")
        .mentioned_all(&[laravel, mvc, controller])
        .commit();
    println!("  Scenario atom: {} (committed to graph)", sc1.atom_id);

    // Outcome: user confirmed the factual answer was good
    reinforce_skill(&mut store, tamar, laravel_skill, true, 1_000_000);
    meta.record("factual", CognitiveMode::Precision, 1.0);
    println!("  ✓ Reinforced laravel_mvc skill");
    println!("  ✓ Meta-learner: Precision +1 for 'factual' context");
    println!();

    // ═══════════════════════════════════════════════════
    // CONVERSATION 2 — creative question (Divergent wins)
    // ═══════════════════════════════════════════════════
    println!("━━━ Conversation 2: creative ━━━");
    println!("Tamar asks: 'תן לי רעיון חדש לעיצוב service-container'");

    // Fresh session for new conversation
    let mut session = SessionContext::new();
    session.mention(service_container);
    session.mention(laravel);
    session.advance_turn();

    // Use Divergent preset — wider, weaker edges included
    let activation = spread_from_session(&store, &session, &SpreadConfig::divergent(), 10);
    let top: Vec<_> = activation.top_k_novel(&session.active_ids(), 5);
    println!("  Spreading activation top-5 (Divergent preset):");
    for (id, score) in &top {
        let atom = store.get(*id).unwrap();
        let label = std::str::from_utf8(&atom.data).unwrap_or("?");
        println!("    {:.3}  {}", score, label);
    }

    // Dream: propose candidate edges from service_container
    println!("  Dreaming (ADHD-like candidate generation):");
    let proposals = propose_via_two_hop(&store, &[service_container], 3, 42);
    for p in &proposals {
        let from_label = store.get(p.from)
            .and_then(|a| std::str::from_utf8(&a.data).ok().map(|s| s.to_string()))
            .unwrap_or_default();
        let to_label = store.get(p.to)
            .and_then(|a| std::str::from_utf8(&a.data).ok().map(|s| s.to_string()))
            .unwrap_or_default();
        let eval_result = evaluate(&store, p, 0.0);
        println!("    {} → {}   [{}]",
            from_label, to_label,
            if eval_result.accepted { "ACCEPTED" } else { "rejected" });
    }

    let _sc2 = ScenarioBuilder::new(&mut store, tamar, 1_000_500, "creative container")
        .mentioned_all(&[service_container, laravel])
        .commit();
    meta.record("creative", CognitiveMode::Divergent, 1.0);
    println!("  ✓ Meta-learner: Divergent +1 for 'creative' context");
    println!();

    // ═══════════════════════════════════════════════════
    // CONVERSATION 3 — bug + emotional regulation
    // ═══════════════════════════════════════════════════
    println!("━━━ Conversation 3: bug + emotional regulation ━━━");
    println!("Tamar: 'יש לי undefined method bug, אני ממש מתוסכלת'");

    let mut session = SessionContext::new();
    session.mention(problem_bug);
    session.mention(stress_event);
    session.advance_turn();

    // Which skills can address this problem?
    let available = zets::skills::skills_for_problem(&store, problem_bug);
    println!("  Skills that can solve 'undefined_method_bug':");
    for skill_id in &available {
        let label = skill_label(&store, *skill_id).unwrap_or_default();
        let prof = proficiency_of(&store, tamar, *skill_id)
            .map(|p| p.label()).unwrap_or("none");
        println!("    • {}  (Tamar's level: {})", label, prof);
    }

    // Reinforce debug skill (Tamar solved it)
    let new_weight = reinforce_skill(&mut store, tamar, debug_skill, true, 1_001_000);
    println!("  ✓ Reinforced debug_skill: weight {} → {}",
        30, new_weight);

    // Also reinforce regulation (she breathed through the frustration)
    let new_weight = reinforce_skill(&mut store, tamar, regulate_skill, true, 1_001_000);
    println!("  ✓ Reinforced regulate_skill: weight {} → {}", 55, new_weight);

    // Scenario with emotion tag
    let joy = store.put(AtomKind::Concept, b"satisfaction".to_vec());
    let _sc3 = ScenarioBuilder::new(&mut store, tamar, 1_001_500, "bug solved")
        .mentioned_all(&[problem_bug, debug_skill, regulate_skill])
        .with_emotion(joy)
        .commit();

    meta.record("problem_solving", CognitiveMode::Precision, 1.0);
    meta.record("emotional", CognitiveMode::Gestalt, 0.8);
    println!();

    // ═══════════════════════════════════════════════════
    // FINAL STATE
    // ═══════════════════════════════════════════════════
    println!("━━━ After 3 conversations ━━━");
    println!();
    println!("Tamar's skills (tree 10 — grew with use):");
    print_skills(&store, tamar);

    println!();
    println!("Meta-learner posterior (tree 11 — learned preferences):");
    for (context, weights) in &meta.per_context {
        let best = weights.best_mode();
        let confidence = weights.confidence();
        println!("  context={:<16}  best_mode={:<10}  confidence={:.2}",
            context, best.label(), confidence);
    }
    println!();

    // Query demonstrating skills_for_problem
    println!("Query: 'which skills solve laravel problems?'");
    let mvc_solvers = zets::skills::skills_for_problem(&store, mvc);
    for s in &mvc_solvers {
        println!("  • {}", skill_label(&store, *s).unwrap_or_default());
    }
    println!();

    // Query: who's proficient in emotion regulation?
    println!("Query: 'who is proficient in emotion regulation?'");
    let regulated = people_with_skill(&store, regulate_skill, Proficiency::Developing);
    for (person, weight) in &regulated {
        let label = store.get(*person)
            .and_then(|a| std::str::from_utf8(&a.data).ok().map(|s| s.to_string()))
            .unwrap_or_default();
        println!("  • {}  (weight={})", label, weight);
    }
    println!();

    // Deterministic mode choice given a new query
    println!("Given the meta-learner state, a new 'factual' query would route to:");
    let hash = query_hash("how does Eloquent work in Laravel?");
    if let Some(w) = meta.weights_for("factual") {
        let mode = w.sample_by_hash(hash);
        println!("  → {} (deterministic hash-based sample)", mode.label());
    }

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  Total atoms: {:<8}  Total edges: {:<8}              ║",
        store.stats().atom_count, store.stats().edge_count);
    println!("╚══════════════════════════════════════════════════════════════╝");
}

fn print_skills(store: &AtomStore, person: zets::atoms::AtomId) {
    let skills = skills_of(store, person);
    for (skill_id, weight) in skills {
        let label = skill_label(store, skill_id).unwrap_or("?".to_string());
        let prof = Proficiency::from_weight(weight);
        let bar = "█".repeat((weight / 5) as usize);
        println!("  {:<35} w={:>3} [{}] {}", label, weight, prof.label(), bar);
    }
}
