//! `zets-piece-graph` — load PieceGraph and measure memory vs old architecture.

use std::time::Instant;
use zets::piece_graph_loader::PieceGraphLoader;

fn main() -> std::io::Result<()> {
    let t0 = Instant::now();
    let loader = PieceGraphLoader::new("data/multilang");
    let graph = loader.load()?;
    let load_ms = t0.elapsed();

    println!("═══════════════════════════════════════════════════════");
    println!(" PieceGraph loaded in {:?}", load_ms);
    println!("═══════════════════════════════════════════════════════");
    println!();
    println!("CORE GRAPH:");
    println!("  Languages           : {}", graph.langs.len());
    println!("  Concepts            : {}", graph.concept_count());
    println!("  Pieces (strings)    : {}", graph.pieces.len());
    println!("  Bytes saved (dedup) : {}", graph.pieces.bytes_saved);
    println!("  Bytes stored        : {}", graph.pieces.bytes_stored);
    let saving_ratio = if graph.pieces.bytes_stored > 0 {
        (graph.pieces.bytes_saved as f64 / graph.pieces.bytes_stored as f64) * 100.0
    } else {
        0.0
    };
    println!("  Dedup rate          : {:.1}%", saving_ratio);
    println!();
    println!("STATS:");
    println!("  Edges total         : {}", graph.stats.edges_total);
    println!("  Edges deduped       : {}", graph.stats.edges_deduped);
    println!("  Definitions         : {}", graph.stats.definitions);
    println!("  Synonyms            : {}", graph.stats.synonyms);
    println!("  Antonyms            : {}", graph.stats.antonyms);
    println!("  POS tags            : {}", graph.stats.pos_tags);
    println!();
    println!("PER-LANGUAGE INDEX SIZES:");
    let mut langs: Vec<_> = graph.lang_indexes.iter().collect();
    langs.sort_by_key(|(lid, _)| *lid);
    for (lang_id, idx) in langs {
        println!(
            "  {:<6} entries={} surfaces_with_concepts={} defs={} syn={} ant={}",
            graph.langs.name(*lang_id),
            idx.entry_count(),
            idx.surface_to_concepts.len(),
            idx.definitions.len(),
            idx.synonyms.len(),
            idx.antonyms.len(),
        );
    }
    println!();
    println!("SAMPLE QUERIES:");
    let samples: &[(&str, &str)] = &[
        ("en", "dog"),
        ("he", "כלב"),
        ("es", "perro"),
        ("de", "Hund"),
        ("tr", "köpek"),
    ];
    for (lang, surface) in samples {
        let concepts = graph.concepts_for_surface(lang, surface);
        println!(
            "  [{}] \"{}\"  →  {} concepts",
            lang,
            surface,
            concepts.len()
        );
        // Show first concept gloss
        if let Some(&cid) = concepts.first() {
            if let Some(node) = graph.get_concept(cid) {
                let gloss = graph.pieces.get(node.gloss_piece);
                let anchor = graph.pieces.get(node.anchor_piece);
                println!("      → c{} anchor=\"{}\"  gloss=\"{}\"", cid, anchor, gloss);
            }
        }
    }

    Ok(())
}
