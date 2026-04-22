use zets::atoms::AtomStore;
use zets::bootstrap::bootstrap;
use zets::ingestion::{ingest_text, IngestConfig};
use std::time::Instant;

fn main() {
    let mut store = AtomStore::new();
    bootstrap(&mut store);
    let config = IngestConfig::default();

    // Generate synthetic text — 1000 'sentences'
    let subjects = ["dog","cat","bird","fish","horse","cow","sheep","pig","goat","rabbit"];
    let verbs = ["eats","sees","likes","hates","finds","brings","loses","keeps"];
    let objects = ["food","water","grass","bone","feather","nest","meadow","forest"];

    let mut text = String::with_capacity(100_000);
    let mut count = 0;
    'outer: for s in &subjects {
        for v in &verbs {
            for o in &objects {
                text.push_str(&format!("{} {} {}. ", s, v, o));
                count += 1;
                if count >= 1000 { break 'outer; }
            }
        }
    }
    // Pad if needed
    while count < 1000 {
        text.push_str("dog eats food. ");
        count += 1;
    }

    println!("Stress test: {} sentences, {} bytes of text", count, text.len());

    let t = Instant::now();
    let result = ingest_text(&mut store, "stress", &text, &config);
    let elapsed = t.elapsed();

    println!("Ingestion complete in {:?}", elapsed);
    println!("  Sentences: {}", result.sentence_atoms.len());
    println!("  Unique tokens: {}", result.unique_tokens);
    println!("  New atoms: {}", result.new_atoms);
    println!("  New edges: {}", result.new_edges);
    println!("  Total store: {} atoms, {} edges", store.atom_count(), store.edge_count());
    println!("  Rate: {:.0} sentences/sec, {:.0} edges/sec",
        result.sentence_atoms.len() as f64 / elapsed.as_secs_f64(),
        result.new_edges as f64 / elapsed.as_secs_f64());
}
