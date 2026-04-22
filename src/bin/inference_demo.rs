//! `inference-demo` — show inference_walk producing Hypothesis attributes.
//!
//! Builds a small mammal hierarchy, asks the system to infer attributes of
//! an atom it has never been directly told about.
//!
//! Usage:  cargo run --release --bin inference_demo

use zets::atoms::{AtomKind, AtomStore};
use zets::inference::{infer_attributes, DEFAULT_MAX_HOPS};
use zets::relations;

fn main() {
    println!("═══ ZETS Inference Demo ═══");
    println!();
    println!("Building a mammal hierarchy:");
    println!("  capybara → rodent → mammal → animal");
    println!("  mammal has_attribute: four_legs, tail, produces_milk");
    println!("  rodent has_attribute: gnawing_teeth, continuous_tooth_growth");
    println!("  animal has_attribute: alive");
    println!();

    let mut store = AtomStore::new();

    // Atoms
    let capybara = store.put(AtomKind::Concept, b"capybara".to_vec());
    let rodent = store.put(AtomKind::Concept, b"rodent".to_vec());
    let mammal = store.put(AtomKind::Concept, b"mammal".to_vec());
    let animal = store.put(AtomKind::Concept, b"animal".to_vec());

    let four_legs = store.put(AtomKind::Concept, b"four_legs".to_vec());
    let tail = store.put(AtomKind::Concept, b"tail".to_vec());
    let produces_milk = store.put(AtomKind::Concept, b"produces_milk".to_vec());
    let gnawing_teeth = store.put(AtomKind::Concept, b"gnawing_teeth".to_vec());
    let continuous_growth = store.put(AtomKind::Concept, b"continuous_tooth_growth".to_vec());
    let alive = store.put(AtomKind::Concept, b"alive".to_vec());

    let is_a = relations::by_name("is_a").unwrap().code;
    let has_attr = relations::by_name("has_attribute").unwrap().code;

    // is_a chain
    store.link(capybara, rodent, is_a, 240, 0);
    store.link(rodent, mammal, is_a, 240, 0);
    store.link(mammal, animal, is_a, 240, 0);

    // has_attribute edges at each level
    store.link(rodent, gnawing_teeth, has_attr, 240, 0);
    store.link(rodent, continuous_growth, has_attr, 240, 0);
    store.link(mammal, four_legs, has_attr, 240, 0);
    store.link(mammal, tail, has_attr, 240, 0);
    store.link(mammal, produces_milk, has_attr, 240, 0);
    store.link(animal, alive, has_attr, 240, 0);

    // Stats
    println!(
        "Graph: {} atoms, {} edges",
        store.atom_count(),
        store.edge_count()
    );
    println!();

    // Run inference
    println!("Query: infer attributes of 'capybara'");
    println!("  (system was told: capybara is_a rodent. That's all.)");
    println!();

    let results = infer_attributes(&store, capybara, DEFAULT_MAX_HOPS, None);

    println!("Inferred {} attribute(s) as Hypothesis:", results.len());
    println!();
    println!(
        "  {:<4} {:<30} {:>5} {:<30}",
        "hops", "attribute", "conf", "source_ancestor"
    );
    println!("  {}", "─".repeat(76));

    for inf in &results {
        let attr = String::from_utf8_lossy(&store.get(inf.attribute).unwrap().data);
        let src = String::from_utf8_lossy(&store.get(inf.source_ancestor).unwrap().data);
        println!(
            "  {:<4} {:<30} {:>5} {:<30}",
            inf.hops(),
            attr,
            inf.confidence,
            src
        );
    }
    println!();

    // Show a derivation trace for the highest-hop one
    if let Some(deepest) = results.iter().max_by_key(|r| r.hops()) {
        println!("Trace for deepest inference:");
        println!("  {}", deepest.trace(&store));
        println!();
        println!("  Provenance tag that would be stored:");
        println!("  {}", deepest.provenance());
    }

    println!();
    println!("Note: these are Hypothesis edges, not Asserted. A caller using");
    println!("Precision mode would filter them out; Divergent mode would");
    println!("include them (with a dampened confidence).");
}
