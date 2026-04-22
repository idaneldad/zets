//! agi-demo — demonstrate that ZETS can do multi-hop reasoning.
//!
//! Scenario: classic inheritance chain.
//!   dog --IS_A→ canine --IS_A→ mammal --IS_A→ animal
//!
//! Questions ZETS should now answer:
//!   Q1. Is dog an animal?  (3 hops away — yes)
//!   Q2. Is animal a dog?   (unreachable — no)
//!   Q3. Is dog a dog?      (self — yes, trivially)
//!
//! All answered by ONE route (is_ancestor) executed on the VM.
//! The Rust side only hosts the graph. The reasoning is in bytecode.

use std::collections::HashMap;
use std::time::Instant;

use zets::system_graph::{Host, Tier, Value, Vm};
use zets::system_graph::reasoning::{all_reasoning_routes, KIND_IS_A, R_IS_ANCESTOR};

fn main() {
    println!("═══ ZETS AGI Capability Demo ═══");
    println!("Scenario: multi-hop reasoning over an IS_A hierarchy");
    println!();

    // ── Build the world ──
    // dog (1) → canine (2) → mammal (3) → animal (4)
    // cat (5) → feline (6) → mammal (3)   (shared ancestor)
    // car (10) → vehicle (11)             (separate hierarchy)
    let mut graph = WorldGraph::new();
    graph.add_isa(1, 2);  // dog → canine
    graph.add_isa(2, 3);  // canine → mammal
    graph.add_isa(3, 4);  // mammal → animal
    graph.add_isa(5, 6);  // cat → feline
    graph.add_isa(6, 3);  // feline → mammal
    graph.add_isa(10, 11); // car → vehicle

    graph.name(1, "dog");
    graph.name(2, "canine");
    graph.name(3, "mammal");
    graph.name(4, "animal");
    graph.name(5, "cat");
    graph.name(6, "feline");
    graph.name(10, "car");
    graph.name(11, "vehicle");

    // ── Load reasoning routes ──
    let mut routes = HashMap::new();
    for r in all_reasoning_routes() {
        println!("Loaded route: {} (id={}, tier={:?}, bytecode={}B)",
            r.name, r.id,
            match r.tier { Tier::Hot => "Hot", Tier::Warm => "Warm",
                           Tier::Cold => "Cold", Tier::Archive => "Archive" },
            r.bytecode.len());
        routes.insert(r.id, r);
    }
    println!();

    // ── Run reasoning queries ──
    let questions: Vec<(u32, u32, i64, bool)> = vec![
        (1, 4, 5, true),   // dog IS_A animal (3 hops)
        (1, 3, 5, true),   // dog IS_A mammal (2 hops)
        (1, 2, 5, true),   // dog IS_A canine (1 hop)
        (1, 1, 5, true),   // dog IS_A dog (self)
        (5, 4, 5, true),   // cat IS_A animal (3 hops, shared mammal)
        (5, 3, 5, true),   // cat IS_A mammal (2 hops)
        (4, 1, 5, false),  // animal IS_A dog (reverse, unreachable)
        (1, 10, 5, false), // dog IS_A car (different hierarchy)
        (1, 4, 2, false),  // dog IS_A animal with depth=2 only (would need 3)
    ];

    println!("{:<8} {:<10} {:<10} {:<7} {:<8} {:<12} {:<6}",
        "Query", "Source", "Target", "Depth", "Expected", "Result", "Time");
    println!("{}", "─".repeat(70));

    let mut all_correct = true;
    for (i, (src, tgt, depth, expected)) in questions.iter().enumerate() {
        let mut vm = Vm::new(&routes);
        let t = Instant::now();
        let r = vm.run(
            R_IS_ANCESTOR,
            vec![Value::ConceptId(*src), Value::ConceptId(*tgt), Value::Int(*depth)],
            &mut graph,
        );
        let elapsed = t.elapsed();
        let actual = matches!(r, Ok(Value::Bool(true)));
        let mark = if actual == *expected { "✓" } else { "✗ MISMATCH!" };
        if actual != *expected { all_correct = false; }

        println!("Q{}     {:<10} {:<10} {:<7} {:<8} {} {:<10} {:<6}",
            i + 1,
            graph.name_of(*src),
            graph.name_of(*tgt),
            depth,
            expected,
            mark,
            actual,
            format!("{:.1}µs", elapsed.as_nanos() as f64 / 1000.0),
        );
    }
    println!();

    // ── Performance benchmark ──
    println!("--- Performance: 10,000 × 3-hop is_ancestor(dog, animal) ---");
    let t = Instant::now();
    for _ in 0..10_000 {
        let mut vm = Vm::new(&routes);
        let _ = vm.run(
            R_IS_ANCESTOR,
            vec![Value::ConceptId(1), Value::ConceptId(4), Value::Int(5)],
            &mut graph,
        );
    }
    let elapsed = t.elapsed();
    println!("  Total: {:?}, avg: {:.1}µs per 3-hop query",
        elapsed, elapsed.as_micros() as f64 / 10_000.0);
    println!();

    println!("═══ Summary ═══");
    if all_correct {
        println!("  ✓ All {} reasoning queries correct.", questions.len());
        println!("  ZETS now performs multi-hop reasoning entirely via bytecode.");
        println!("  No Rust if/else chains — only route recursion in the VM.");
    } else {
        println!("  ✗ Some queries failed!");
    }
    println!();
    println!("What this demonstrates:");
    println!("  * Recursion bounded by depth param (no runaway)");
    println!("  * Shared hierarchies work (cat + dog both reach animal via mammal)");
    println!("  * Unreachable targets correctly return false");
    println!("  * Depth limits enforced (dog→animal needs 3, fails at depth=2)");
    println!("  * Self-reference (X IS_A X) trivially true");
    println!();
    println!("Next AGI capabilities to add (as routes):");
    println!("  - is_descendant (inverse traversal)");
    println!("  - common_ancestor (find LCA of two concepts)");
    println!("  - causal_chain (CAUSES edges, explaining 'why?')");
    println!("  - part_of_path (PART_OF edges, 'what makes up X?')");
    println!("  - confidence_calc (aggregate trust across path)");
    println!("  - contradiction_detect (X IS_A Y and X IS_NOT_A Y)");
}

// ──────────────────────────────────────────────────────────────
// WorldGraph — the knowledge graph for this demo
// ──────────────────────────────────────────────────────────────
struct WorldGraph {
    isa: HashMap<u32, Vec<u32>>,
    names: HashMap<u32, String>,
}

impl WorldGraph {
    fn new() -> Self {
        Self { isa: HashMap::new(), names: HashMap::new() }
    }
    fn add_isa(&mut self, from: u32, to: u32) {
        self.isa.entry(from).or_default().push(to);
    }
    fn name(&mut self, id: u32, n: &str) {
        self.names.insert(id, n.to_string());
    }
    fn name_of(&self, id: u32) -> String {
        self.names.get(&id).cloned().unwrap_or_else(|| format!("#{}", id))
    }
}

impl Host for WorldGraph {
    fn concept_lookup(&mut self, _: &str, _: &str) -> Option<u32> { None }
    fn concept_create(&mut self, _: &str, _: &str, _: u8) -> u32 { 0 }
    fn edge_add(&mut self, _: u32, _: u32, _: u8) {}
    fn edge_traverse(&mut self, c: u32, kind: u8) -> Vec<u32> {
        if kind == KIND_IS_A as u8 {
            self.isa.get(&c).cloned().unwrap_or_default()
        } else {
            vec![]
        }
    }
    fn morph_analyze(&mut self, _: &str, s: &str) -> String { s.to_string() }
    fn string_match(&mut self, t: &str, p: &str) -> bool { t.contains(p) }
    fn wal_write(&mut self, _: u8, _: &[u8]) {}
}
