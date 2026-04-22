//! `planner_demo` — Multi-step goal pursuit with auto-commit of sessions.
//!
//! Shows the Tree-10 / Gemini-priority-3 capability: agentic behavior.
//! Given a goal atom, ZETS plans a sequence of steps, executes them,
//! and tracks whether the goal was reached.
//!
//! At the end of the conversation, the session is auto-committed as a
//! Scenario atom in the persona graph — creating a persistent memory
//! trace of what was discussed.

use zets::atoms::{AtomKind, AtomStore};
use zets::bootstrap::bootstrap;
use zets::ingestion::{ingest_text, IngestConfig};
use zets::meta_learning::MetaLearner;
use zets::planner::{pursue_goal, Goal};
use zets::scenario::{auto_commit_session, scenarios_of, scenarios_mentioning};
use zets::session::SessionContext;

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║  ZETS Planner Demo — goal pursuit + auto-scenario     ║");
    println!("╚════════════════════════════════════════════════════════╝");
    println!();

    // Fresh brain + bootstrap
    let mut store = AtomStore::new();
    bootstrap(&mut store);

    // Add a person (the "user" of this brain)
    let user = store.put(AtomKind::Concept, b"user:Idan".to_vec());

    // Ingest a small knowledge base so the planner has something to work with
    let text = "\
        Rust is a systems programming language. Rust has ownership rules. \
        Ownership prevents memory bugs. Memory bugs cause crashes. \
        Crashes are unpleasant experiences. Unpleasant experiences teach us. \
        Programming teaches problem solving. Problem solving is valuable. \
        Valuable skills deserve investment. Investment creates mastery.";
    let _ = ingest_text(&mut store, "programming-wisdom", text, &IngestConfig::default());
    println!("Knowledge loaded: {} atoms, {} edges",
        store.atom_count(), store.edge_count());
    println!();

    // Find the atom IDs we'll use as start/target
    let rust_hash = zets::atoms::content_hash("word:rust".as_bytes());
    let mastery_hash = zets::atoms::content_hash("word:mastery".as_bytes());
    let (all_atoms, _) = store.snapshot();
    let rust_atom = all_atoms.iter().position(|a| a.content_hash == rust_hash).map(|i| i as u32);
    let mastery_atom = all_atoms.iter().position(|a| a.content_hash == mastery_hash).map(|i| i as u32);

    let (Some(rust), Some(mastery)) = (rust_atom, mastery_atom) else {
        println!("Could not find expected atoms — skipping demo");
        return;
    };
    println!("Start atom: 'rust' (id={})", rust);
    println!("Target atom: 'mastery' (id={})", mastery);
    println!();

    // ═══════════════════════════════════════════════════
    // GOAL PURSUIT
    // ═══════════════════════════════════════════════════
    let mut session = SessionContext::new();
    let mut meta = MetaLearner::new();

    let goal = Goal {
        start: rust,
        target: Some(mastery),
        relation_filter: None,  // any relation is OK
        label: "How does Rust lead to mastery?".to_string(),
        query_context: "reasoning".to_string(),
    };

    println!("━━━ Goal: '{}' ━━━", goal.label);
    println!();

    let plan = pursue_goal(&mut store, &mut session, &mut meta, goal);

    println!("Plan executed in {} steps. Target reached: {}",
        plan.history.len(), plan.target_reached);
    println!();
    for (i, (step, result)) in plan.history.iter().enumerate() {
        let step_name = match step {
            zets::planner::Step::Focus { atoms } =>
                format!("Focus on {} atom(s)", atoms.len()),
            zets::planner::Step::Explore { query_text, top_k } =>
                format!("Explore top-{} for '{}'", top_k, query_text),
            zets::planner::Step::Dream { max_edges, depth, .. } =>
                format!("Dream (max {} edges, depth {})", max_edges, depth),
            zets::planner::Step::ProposeEdges { max_per_seed, .. } =>
                format!("Propose up to {} edges per seed", max_per_seed),
            zets::planner::Step::AdvanceTurn =>
                "Advance session turn".to_string(),
        };

        let result_summary = match result {
            zets::planner::StepResult::Focused => "session updated".to_string(),
            zets::planner::StepResult::TurnAdvanced => "decay applied".to_string(),
            zets::planner::StepResult::Explored { candidates, mode } =>
                format!("{} candidates found (mode={})", candidates.len(), mode),
            zets::planner::StepResult::Dreamed { committed, proposed } =>
                format!("{} / {} edges committed", committed, proposed),
            zets::planner::StepResult::Proposed { count } =>
                format!("{} proposals evaluated", count),
            zets::planner::StepResult::Failed { reason } =>
                format!("FAILED: {}", reason),
        };

        println!("  Step {}: {}", i + 1, step_name);
        println!("          → {}", result_summary);
    }
    println!();

    // ═══════════════════════════════════════════════════
    // AUTO-COMMIT SESSION → SCENARIO
    // ═══════════════════════════════════════════════════
    println!("━━━ Auto-commit session as scenario ━━━");
    let scenario = auto_commit_session(
        &mut store,
        &session,
        user,
        1_000_000,
        "reasoning about rust and mastery",
        None,
    );

    if let Some(sc) = scenario {
        println!("  ✓ Scenario {} committed to graph", sc.atom_id);
        println!("    label: {}", sc.label);
        println!("    mentioned {} atoms", sc.mentioned_atoms.len());
        println!("    linked to user: {}", sc.person_id);
    } else {
        println!("  (session was empty — nothing to commit)");
    }
    println!();

    // Verify the scenario is now queryable
    println!("━━━ Query scenarios of the user ━━━");
    let user_scenarios = scenarios_of(&store, user);
    println!("  {} scenarios total for user", user_scenarios.len());

    let about_rust = scenarios_mentioning(&store, user, rust);
    println!("  {} scenarios mention 'rust'", about_rust.len());
    println!();

    // Final state
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║  Final store: {:>4} atoms, {:>4} edges               ║",
        store.atom_count(), store.edge_count());
    println!("║  Plan reached target: {:<30}  ║",
        if plan.target_reached { "YES" } else { "no" });
    println!("║  All deterministic. All auditable.                     ║");
    println!("╚════════════════════════════════════════════════════════╝");
}
