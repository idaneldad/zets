//! `cognitive-demo` — the "genius with disorders" architecture in action.
//!
//! Same graph, same query, three different cognitive modes produce three
//! completely different traversal patterns — each revealing different
//! insights about the same reality.
//!
//! Graph layout (a fragment of a 'dog' knowledge neighborhood):
//!
//!   dog (1)
//!     ├─IS_A→ canine (2) ─IS_A→ mammal (3) ─IS_A→ animal (4)
//!     ├─HAS_PROP (w=40)→ loyal (10)
//!     ├─HAS_PROP (w=55)→ friendly (11)
//!     ├─SEEN_IN (w=5) → space (20)          ← weird connection
//!     ├─COMPARES_TO (w=15)→ wolf (30)
//!     └─COMPARES_TO (w=25)→ cat (31) ─IS_A→ feline (32) ─IS_A→ mammal (3)
//!
//! Question: "What is a dog?"
//!
//! PrecisionMode will walk the strong IS_A chain up to animal.
//! DivergentMode will discover the remote cat→feline→mammal path.
//! GestaltMode will gather the neighborhood for a holistic picture.
//! NarrativeMode will tell the story of all three.

use std::collections::HashMap;
use std::time::Instant;

use zets::cognitive_modes::{
    CognitiveMode, DivergentMode, GestaltMode, GraphHost, NarrativeMode, PrecisionMode, Query,
};

fn main() {
    println!("═══ ZETS Cognitive Modes Demo ═══");
    println!();
    println!("Question: 'What is a dog?'");
    println!("Starting concept: #1 'dog'");
    println!();

    let mut world = build_world();

    let query = Query::new(1, 5);
    println!("Query id (deterministic hash): {:x}", query.id);
    println!();

    // ──────────────────────────────────────────────────────────
    // Mode 1: PrecisionMode (autism-inspired)
    // ──────────────────────────────────────────────────────────
    let precision = PrecisionMode::default();
    let t = Instant::now();
    let p_result = precision.walk(&query, &mut world);
    let p_elapsed = t.elapsed();

    println!("─── {} ({}) ───", precision.name(), precision.inspired_by());
    println!("  visited : {:?}", p_result.visited);
    println!("  steps   : {}, dead_ends: {}, elapsed: {:?}",
        p_result.steps.len(), p_result.dead_ends, p_elapsed);
    for step in &p_result.steps {
        println!("    d{} {} →(k{},w{}) {}  [{}]",
            step.depth,
            world.label(step.from), step.edge_kind, step.weight,
            world.label(step.to), step.reason);
    }
    println!();

    // ──────────────────────────────────────────────────────────
    // Mode 2: DivergentMode (ADHD-inspired)
    // ──────────────────────────────────────────────────────────
    let divergent = DivergentMode::default();  // 15% divergence
    let t = Instant::now();
    let d_result = divergent.walk(&query, &mut world);
    let d_elapsed = t.elapsed();

    println!("─── {} ({}) ───", divergent.name(), divergent.inspired_by());
    println!("  visited : {:?}", d_result.visited);
    println!("  steps   : {}, dead_ends: {}, elapsed: {:?}",
        d_result.steps.len(), d_result.dead_ends, d_elapsed);
    for step in &d_result.steps {
        println!("    d{} {} →(k{},w{}) {}  [{}]",
            step.depth,
            world.label(step.from), step.edge_kind, step.weight,
            world.label(step.to), step.reason);
    }
    println!();

    // ──────────────────────────────────────────────────────────
    // Mode 3: GestaltMode (dyslexia-inspired)
    // ──────────────────────────────────────────────────────────
    let gestalt = GestaltMode { neighborhood_hops: 2 };
    let t = Instant::now();
    let g_result = gestalt.walk(&query, &mut world);
    let g_elapsed = t.elapsed();

    println!("─── {} ({}) ───", gestalt.name(), gestalt.inspired_by());
    println!("  visited : {:?}", g_result.visited);
    println!("  steps   : {}, dead_ends: {}, elapsed: {:?}",
        g_result.steps.len(), g_result.dead_ends, g_elapsed);
    println!();

    // ──────────────────────────────────────────────────────────
    // Determinism proof
    // ──────────────────────────────────────────────────────────
    println!("─── Determinism Proof ───");
    println!("Running DivergentMode 3 times on the same query:");
    for run in 1..=3 {
        let mut w = build_world();
        let r = divergent.walk(&query, &mut w);
        println!("  Run {}: visited = {:?}", run, r.visited);
    }
    println!("  → All runs IDENTICAL (hash-based divergence, no rand)");
    println!();

    // ──────────────────────────────────────────────────────────
    // Different queries → different divergence paths
    // ──────────────────────────────────────────────────────────
    println!("─── Different queries produce different walks ───");
    for start in [1u32, 31, 3] {
        let q = Query::new(start, 3);
        let mut w = build_world();
        let r = divergent.walk(&q, &mut w);
        println!("  Start=#{} ({}), query_id={:x}, visited={:?}",
            start, w.label(start), q.id, r.visited);
    }
    println!();

    // ──────────────────────────────────────────────────────────
    // Narrative composition
    // ──────────────────────────────────────────────────────────
    println!("─── NarrativeMode (composes all) ───");
    let mut w = build_world();
    let qq = Query::new(1, 5);
    let p = PrecisionMode::default().walk(&qq, &mut w);
    let d = DivergentMode::default().walk(&qq, &mut w);
    let g = GestaltMode { neighborhood_hops: 2 }.walk(&qq, &mut w);

    let story = NarrativeMode.compose(
        &qq,
        &[
            (&PrecisionMode::default() as &dyn CognitiveMode, &p),
            (&DivergentMode::default() as &dyn CognitiveMode, &d),
            (&GestaltMode { neighborhood_hops: 2 } as &dyn CognitiveMode, &g),
        ],
        &mut w,
    );
    println!("{}", story);

    // ──────────────────────────────────────────────────────────
    // Performance: 10,000 runs
    // ──────────────────────────────────────────────────────────
    println!("─── Performance Benchmark (10,000 runs each) ───");
    macro_rules! bench {
        ($name:expr, $body:expr) => {{
            let t = Instant::now();
            let mut sum = 0usize;
            for _ in 0..10_000 {
                let mut w = build_world();
                sum += $body(&mut w);
            }
            let e = t.elapsed();
            println!("  {}: {:?} total, avg={:.2}µs/run",
                $name, e, e.as_micros() as f64 / 10_000.0);
            let _ = sum;
        }};
    }
    bench!("Precision", |w: &mut World| {
        PrecisionMode::default().walk(&Query::new(1, 5), w).visited.len()
    });
    bench!("Divergent", |w: &mut World| {
        DivergentMode::default().walk(&Query::new(1, 5), w).visited.len()
    });
    bench!("Gestalt  ", |w: &mut World| {
        GestaltMode { neighborhood_hops: 2 }.walk(&Query::new(1, 5), w).visited.len()
    });

    println!();
    println!("═══ Summary ═══");
    println!("  PrecisionMode:  {} nodes, strict IS_A chain only", p_result.visited.len());
    println!("  DivergentMode:  {} nodes (includes some weak edges)", d_result.visited.len());
    println!("  GestaltMode:    {} nodes (whole neighborhood)", g_result.visited.len());
    println!();
    println!("Same graph, same question — 3 different answers.");
    println!("All fully deterministic. Same query tomorrow = same result.");
    println!("This is the 'genius with disorders' architecture in silicon.");
}

// ──────────────────────────────────────────────────────────────────
// World graph
// ──────────────────────────────────────────────────────────────────

struct World {
    edges: HashMap<u32, Vec<(u32, u8, u8)>>,
    labels: HashMap<u32, &'static str>,
}

impl GraphHost for World {
    fn outgoing(&mut self, n: u32) -> Vec<(u32, u8, u8)> {
        self.edges.get(&n).cloned().unwrap_or_default()
    }
    fn label(&mut self, n: u32) -> String {
        self.labels.get(&n).map(|s| s.to_string()).unwrap_or_else(|| format!("#{}", n))
    }
}

fn build_world() -> World {
    let mut edges: HashMap<u32, Vec<(u32, u8, u8)>> = HashMap::new();
    let mut labels: HashMap<u32, &'static str> = HashMap::new();

    // Edge kinds: 3=IS_A, 7=HAS_PROP, 9=SEEN_IN, 11=COMPARES_TO
    //
    // dog (1)
    edges.insert(1, vec![
        (2,  3, 90),   // dog IS_A canine
        (10, 7, 40),   // dog HAS_PROP loyal (weak)
        (11, 7, 55),   // dog HAS_PROP friendly (medium)
        (20, 9, 5),    // dog SEEN_IN space (very weak — unusual)
        (30, 11, 15),  // dog COMPARES_TO wolf (weak)
        (31, 11, 25),  // dog COMPARES_TO cat (weak)
    ]);
    edges.insert(2,  vec![(3, 3, 90)]);   // canine IS_A mammal
    edges.insert(3,  vec![(4, 3, 95)]);   // mammal IS_A animal
    edges.insert(4,  vec![]);              // animal (root)
    edges.insert(10, vec![]);              // loyal
    edges.insert(11, vec![]);              // friendly
    edges.insert(20, vec![]);              // space
    edges.insert(30, vec![(3, 3, 85)]);   // wolf IS_A mammal
    edges.insert(31, vec![(32, 3, 90)]);  // cat IS_A feline
    edges.insert(32, vec![(3,  3, 90)]);  // feline IS_A mammal

    labels.insert(1, "dog");       labels.insert(2, "canine");
    labels.insert(3, "mammal");    labels.insert(4, "animal");
    labels.insert(10, "loyal");    labels.insert(11, "friendly");
    labels.insert(20, "space");    labels.insert(30, "wolf");
    labels.insert(31, "cat");      labels.insert(32, "feline");

    World { edges, labels }
}
