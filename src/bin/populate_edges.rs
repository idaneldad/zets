//! `populate-edges` — extract semantic edges from Wiktionary glosses in zets.core.
//!
//! Transforms (144K concepts, 0 edges) → (144K concepts, ~10-20K edges)
//! by mining "a kind of X", "a type of X", "any of X" patterns.

use std::path::PathBuf;
use std::time::Instant;

use zets::edge_extraction::{extract_edges, apply_edges, ExtractionStats};
use zets::pack::PackReader;

fn main() -> std::io::Result<()> {
    println!("═══ ZETS Edge Population ═══");
    println!();

    let core_path = PathBuf::from("data/packs/zets.core");
    println!("Loading core pack from {:?} ...", core_path);
    let t0 = Instant::now();
    let mut graph = PackReader::read_core(&core_path)?;
    let load_ms = t0.elapsed();
    println!("  Loaded {} concepts in {:?}", graph.concepts.len(), load_ms);

    let edges_before: usize = graph.concepts.iter().map(|c| c.edges.len()).sum();
    println!("  Edges before extraction: {}", edges_before);
    println!();

    println!("Running pattern extractor...");
    let t1 = Instant::now();
    let result = extract_edges(&graph);
    let extract_ms = t1.elapsed();
    println!("  Extraction completed in {:?}", extract_ms);
    println!();

    print_stats(&result.stats);
    println!();

    println!("Applying {} proposed edges to graph...", result.proposed.len());
    let t2 = Instant::now();
    let applied = apply_edges(&mut graph, &result.proposed);
    let apply_ms = t2.elapsed();
    println!("  {} edges actually inserted (after dedup) in {:?}", applied, apply_ms);
    println!();

    let edges_after: usize = graph.concepts.iter().map(|c| c.edges.len()).sum();
    println!("Edges: {} → {}  (+{})", edges_before, edges_after,
             edges_after - edges_before);
    println!();

    // Sample some enriched concepts to verify
    println!("─── Sample enriched concepts ───");
    let mut shown = 0;
    for c in &graph.concepts {
        if !c.edges.is_empty() && shown < 15 {
            let anchor = graph.pieces.get(c.anchor_piece);
            let gloss = graph.pieces.get(c.gloss_piece);
            print!("  {:20} [{:3} edges] gloss=\"{}\"",
                anchor, c.edges.len(),
                gloss.chars().take(50).collect::<String>());
            print!(" → ");
            for (i, e) in c.edges.iter().take(3).enumerate() {
                if i > 0 { print!(", "); }
                if let Some(target) = graph.concepts.get(e.target as usize) {
                    let t_anchor = graph.pieces.get(target.anchor_piece);
                    print!("{:?}→{}", e.kind_enum(), t_anchor);
                }
            }
            println!();
            shown += 1;
        }
    }

    println!();
    println!("═══ Summary ═══");
    println!("  Concepts scanned     : {}", result.stats.concepts_scanned);
    println!("  Glosses with pattern : {}", result.stats.glosses_with_pattern);
    println!("  Edges extracted      : {}", result.proposed.len());
    println!("  Edges applied        : {}", applied);
    println!("  Head nouns resolved  : {}", result.stats.head_nouns_resolved);
    println!("  Head nouns missed    : {}", result.stats.head_nouns_missed);
    println!("  Self-links skipped   : {}", result.stats.self_links_skipped);
    println!();
    println!("  Total time: {:?} (load) + {:?} (extract) + {:?} (apply) = {:?}",
             load_ms, extract_ms, apply_ms, load_ms + extract_ms + apply_ms);

    Ok(())
}

fn print_stats(s: &ExtractionStats) {
    println!("─── Pattern match counts ───");
    let mut items: Vec<_> = s.patterns_matched.iter().collect();
    items.sort_by_key(|(_, c)| std::cmp::Reverse(**c));
    for (prefix, count) in items {
        println!("  {:20} : {} matches", format!("\"{}\"", prefix), count);
    }
}
