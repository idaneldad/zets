//! `zets-pack-read` — demonstrate lazy per-language loading.
//!
//! Usage: pack-read [lang1 lang2 ...]
//! Default: loads only he+en (Hebrew + English).
//!
//! Measures RAM for:
//!   1. Core only (no language loaded)
//!   2. Core + 1 language
//!   3. Core + 2 languages (typical user)
//!   4. Core + all languages

use std::env;
use std::path::PathBuf;
use std::time::Instant;

use zets::pack::PackReader;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let default_langs = vec!["he".to_string(), "en".to_string()];
    let langs: Vec<String> = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        default_langs
    };

    let packs_dir = PathBuf::from("data/packs");

    println!("═══ ZETS Pack Reader ═══");
    println!();

    // Phase 1: load core only
    let t0 = Instant::now();
    let core_path = packs_dir.join("zets.core");
    let mut graph = PackReader::read_core(&core_path)?;
    let core_ms = t0.elapsed();
    println!("Phase 1: Core loaded");
    println!("  Languages registered : {}", graph.langs.len());
    println!("  Concepts             : {}", graph.concept_count());
    println!("  Pieces               : {}", graph.pieces.len());
    println!("  Load time            : {:?}", core_ms);
    println!();

    // Phase 2: load each requested language lazily
    for lang in &langs {
        let lang_path = packs_dir.join(format!("zets.{}", lang));
        if !lang_path.exists() {
            eprintln!("  [!] {} pack not found at {:?}", lang, lang_path);
            continue;
        }
        let t = Instant::now();
        PackReader::read_lang(&mut graph, &lang_path)?;
        println!(
            "Phase 2: Loaded \"{}\"  in {:?}  (now {} langs in memory)",
            lang,
            t.elapsed(),
            graph.lang_indexes.len()
        );
    }

    println!();
    println!("─── Verify queries work ───");
    let samples: &[(&str, &str)] = &[
        ("en", "dog"),
        ("en", "big"),
        ("en", "house"),
        ("he", "גדול"),
        ("he", "בית"),
        ("es", "perro"),
        ("de", "Hund"),
        ("tr", "köpek"),
    ];
    for (lang, surface) in samples {
        // Only test languages we loaded
        if !langs.iter().any(|l| l == lang) {
            continue;
        }
        let concepts = graph.concepts_for_surface(lang, surface);
        println!(
            "  [{}] \"{}\"  →  {} concepts",
            lang,
            surface,
            concepts.len()
        );
        if let Some(&cid) = concepts.first() {
            if let Some(node) = graph.get_concept(cid) {
                let gloss = graph.pieces.get(node.gloss_piece);
                let anchor = graph.pieces.get(node.anchor_piece);
                println!("      c{} anchor=\"{}\"  gloss=\"{}\"", cid, anchor, gloss);
            }
        }
    }

    Ok(())
}
