//! `zets-mmap-read` — OS-level lazy loading via mmap.
//!
//! Usage:
//!   mmap-read                 — core only
//!   mmap-read he en           — core + Hebrew + English (typical user)
//!   mmap-read all             — core + all 16 languages
//!   mmap-read --all-concepts  — force-read every concept (load test)

use std::env;
use std::path::PathBuf;
use std::time::Instant;

use zets::mmap_core::MmapCore;
use zets::mmap_lang::MmapLangPack;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let force_all = args.iter().any(|a| a == "--all-concepts");
    let langs: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with("--"))
        .flat_map(|a| {
            if a == "all" {
                vec![
                    "en", "he", "de", "fr", "es", "it", "pt", "nl", "ru", "ar", "ja", "tr", "pl",
                    "sv", "ro", "ca",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
            } else {
                vec![a.clone()]
            }
        })
        .collect();

    let packs_dir = PathBuf::from("data/packs");

    println!("═══ ZETS mmap Reader ═══");
    println!();

    // Phase 1: open core
    let t0 = Instant::now();
    let core_path = packs_dir.join("zets.core");
    let core = MmapCore::open(&core_path)?;
    let core_ms = t0.elapsed();
    println!("Phase 1: Core mmap'd");
    println!("  File size          : {:.1} MB", core.mmap_len() as f64 / 1_048_576.0);
    println!("  Open time          : {:?}", core_ms);
    println!("  Languages in registry: {}", core.lang_codes.len());
    println!("  Concepts           : {}", core.concept_count);
    println!("  Pieces             : {}", core.piece_count);
    println!();

    // Phase 2: open each requested language pack
    let mut lang_packs = Vec::new();
    for lang in &langs {
        let path = packs_dir.join(format!("zets.{}", lang));
        if !path.exists() {
            eprintln!("  [!] pack not found: zets.{}", lang);
            continue;
        }
        let t = Instant::now();
        let pack = MmapLangPack::open(&path)?;
        println!(
            "Phase 2: Loaded \"{}\"  file={:.2} MB  index_entries={}  in {:?}",
            lang,
            pack.mmap_len() as f64 / 1_048_576.0,
            pack.surface_count(),
            t.elapsed()
        );
        lang_packs.push(pack);
    }
    println!();

    // Phase 3: sample queries across loaded languages
    println!("Phase 3: Sample queries");
    let samples: &[(&str, &str)] = &[
        ("en", "dog"),
        ("en", "big"),
        ("en", "tree"),
        ("he", "גדול"),
        ("he", "ספר"),
        ("he", "בית"),
        ("es", "perro"),
        ("de", "Hund"),
        ("tr", "köpek"),
        ("ja", "犬"),
    ];
    for (lang, surface) in samples {
        // Is this language loaded?
        let Some(pack) = lang_packs.iter().find(|p| p.lang_code == *lang) else {
            continue;
        };
        // Find the piece_id via scanning (we don't have a reverse surface → id map yet)
        // For demo we iterate through core.piece_count and find matching surface
        let mut found_piece = None;
        for pid in 0..core.piece_count {
            if core.get_piece(pid) == *surface {
                found_piece = Some(pid);
                break;
            }
        }
        match found_piece {
            None => println!("  [{}] \"{}\"  (not in piece pool)", lang, surface),
            Some(pid) => {
                let concepts = pack.concepts_for_surface_piece(pid);
                println!(
                    "  [{}] \"{}\" piece={}  →  {} concepts",
                    lang,
                    surface,
                    pid,
                    concepts.len()
                );
                if let Some(&cid) = concepts.first() {
                    if let Some(c) = core.get_concept(cid) {
                        let anchor = core.get_piece(c.anchor_piece);
                        let gloss = core.get_piece(c.gloss_piece);
                        let gloss_short: String = gloss.chars().take(50).collect();
                        println!(
                            "      c{} anchor=\"{}\" gloss=\"{}\" pos={}",
                            cid, anchor, gloss_short, c.pos
                        );
                    }
                }
            }
        }
    }
    println!();

    if force_all {
        println!("Phase 4: force-read all concepts");
        let mut total_edges = 0usize;
        let mut present = 0usize;
        for cid in 0..core.concept_count {
            if let Some(c) = core.get_concept(cid) {
                total_edges += c.edge_count;
                present += 1;
            }
        }
        println!("  Present: {}", present);
        println!("  Total edges: {}", total_edges);
    }

    println!();
    println!("DONE — check RAM with /usr/bin/time -v");

    Ok(())
}
