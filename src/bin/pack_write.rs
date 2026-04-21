//! `zets-pack-write` — serialize the PieceGraph (built from TSV) to binary packs.
//!
//! Output:
//!   data/packs/zets.core    — universal (always loaded)
//!   data/packs/zets.<lang>  — one per language (lazy loaded later)
//!
//! Proves Stage 2 works end-to-end: TSV → PieceGraph → binary packs on disk.

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use zets::pack::PackWriter;
use zets::piece_graph_loader::PieceGraphLoader;

fn main() -> std::io::Result<()> {
    let out_dir = PathBuf::from("data/packs");
    fs::create_dir_all(&out_dir)?;

    println!("═══ ZETS Pack Writer ═══");
    println!();

    let t0 = Instant::now();
    println!("Building PieceGraph from TSV sources...");
    let loader = PieceGraphLoader::new("data/multilang");
    let graph = loader.load()?;
    let build_ms = t0.elapsed();
    println!("  Loaded in {:?}", build_ms);
    println!("  Languages          : {}", graph.langs.len());
    println!("  Concepts           : {}", graph.concept_count());
    println!("  Pieces (strings)   : {}", graph.pieces.len());
    println!("  Edges deduped      : {}", graph.stats.edges_deduped);
    println!();

    // Write core file
    let t1 = Instant::now();
    let core_path = out_dir.join("zets.core");
    let core_bytes = PackWriter::write_core(&graph, &core_path)?;
    let core_ms = t1.elapsed();
    println!(
        "Wrote zets.core   : {} bytes ({:.1} MB) in {:?}",
        core_bytes,
        core_bytes as f64 / 1_048_576.0,
        core_ms
    );

    // Write per-language packs
    let mut total_lang_bytes = 0u64;
    let mut sorted_langs: Vec<_> = graph.lang_indexes.keys().copied().collect();
    sorted_langs.sort();
    for lang_id in sorted_langs {
        let code = graph.langs.name(lang_id).to_string();
        if code.is_empty() {
            continue;
        }
        let idx = graph.lang_indexes.get(&lang_id).unwrap();
        let t2 = Instant::now();
        let path = out_dir.join(format!("zets.{}", code));
        let bytes = PackWriter::write_lang(&code, idx, &path)?;
        total_lang_bytes += bytes;
        println!(
            "  zets.{:<5} : {:>9} bytes ({:>6.2} MB)  in {:?}",
            code,
            bytes,
            bytes as f64 / 1_048_576.0,
            t2.elapsed()
        );
    }

    println!();
    let total = core_bytes + total_lang_bytes;
    println!(
        "TOTAL             : {} bytes ({:.2} MB)",
        total,
        total as f64 / 1_048_576.0
    );
    println!();

    // Compare to TSV baseline
    let tsv_size = du_bytes("data/multilang")?;
    println!(
        "TSV baseline     : {} bytes ({:.2} MB)",
        tsv_size,
        tsv_size as f64 / 1_048_576.0
    );
    if total > 0 {
        println!("Compression ratio: {:.2}x", tsv_size as f64 / total as f64);
    }

    Ok(())
}

fn du_bytes(path: &str) -> std::io::Result<u64> {
    let mut total = 0u64;
    let p = PathBuf::from(path);
    if !p.exists() {
        return Ok(0);
    }
    let mut stack = vec![p];
    while let Some(cur) = stack.pop() {
        if cur.is_dir() {
            for entry in fs::read_dir(&cur)? {
                let entry = entry?;
                let ep = entry.path();
                if ep.is_dir() {
                    stack.push(ep);
                } else if let Ok(meta) = entry.metadata() {
                    total += meta.len();
                }
            }
        }
    }
    Ok(total)
}
