//! `pack-inventory` — report on all ZETS pack files.
//!
//! Scans data/packs/ and reports: file sizes, mmap open time,
//! concept/piece counts per pack. Used to validate the lazy-loading
//! infrastructure without loading everything into RAM.
//!
//! Usage:  cargo run --release --bin pack-inventory
//!
//! Never modifies anything. Read-only.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use zets::mmap_core::MmapCore;
use zets::mmap_lang::MmapLangPack;

fn human_size(bytes: u64) -> String {
    const K: u64 = 1024;
    const M: u64 = K * 1024;
    const G: u64 = M * 1024;
    if bytes >= G {
        format!("{:.1} GB", bytes as f64 / G as f64)
    } else if bytes >= M {
        format!("{:.1} MB", bytes as f64 / M as f64)
    } else if bytes >= K {
        format!("{:.1} KB", bytes as f64 / K as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn main() -> std::io::Result<()> {
    let packs_dir = PathBuf::from("data/packs");
    if !packs_dir.exists() {
        eprintln!("  ✗ {} not found", packs_dir.display());
        std::process::exit(1);
    }

    println!("═══ ZETS Pack Inventory ═══");
    println!();
    println!("  Location: {}", packs_dir.display());
    println!();

    // ── Core ──────────────────────────────────────────────────
    let core_path = packs_dir.join("zets.core");
    if core_path.exists() {
        let size = fs::metadata(&core_path)?.len();
        let t0 = Instant::now();
        let core = MmapCore::open(&core_path)?;
        let open_ms = t0.elapsed().as_secs_f64() * 1000.0;
        println!("  CORE                              {:>10}", human_size(size));
        println!("    mmap open time               : {:>7.2} ms", open_ms);
        println!("    languages registered         : {:>7}", core.lang_codes.len());
        println!("    concepts                     : {:>7}", core.concept_count);
        println!("    pieces                       : {:>7}", core.piece_count);
        println!();
    } else {
        println!("  CORE                              NOT FOUND");
        println!();
    }

    // ── Language packs ────────────────────────────────────────
    println!("  LANGUAGE PACKS");
    println!("  {:<6} {:>10} {:>10} {:>14} {:>14}",
             "lang", "size", "open_ms", "has_pos", "has_synonyms");
    println!("  {}", "─".repeat(60));

    let mut lang_entries: Vec<(String, PathBuf, u64)> = Vec::new();
    for entry in fs::read_dir(&packs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if let Some(lang) = name.strip_prefix("zets.") {
                if lang != "core" {
                    let size = entry.metadata()?.len();
                    lang_entries.push((lang.to_string(), path, size));
                }
            }
        }
    }
    lang_entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut total_lang_size: u64 = 0;
    for (lang, path, size) in &lang_entries {
        total_lang_size += size;
        let t0 = Instant::now();
        match MmapLangPack::open(path) {
            Ok(pack) => {
                let open_ms = t0.elapsed().as_secs_f64() * 1000.0;
                println!("  {:<6} {:>10} {:>8.2} ms {:>14} {:>14}",
                         lang,
                         human_size(*size),
                         open_ms,
                         pack.surface_pos.len(),
                         pack.synonyms_index.len());
            }
            Err(e) => {
                println!("  {:<6} {:>10}     ERROR: {}", lang, human_size(*size), e);
            }
        }
    }
    println!("  {}", "─".repeat(60));
    println!("  {:<6} {:>10}      total", "", human_size(total_lang_size));
    println!();

    // ── Total footprint ──────────────────────────────────────
    let core_size = fs::metadata(&core_path).map(|m| m.len()).unwrap_or(0);
    let total = core_size + total_lang_size;
    println!("  TOTAL PACK FOOTPRINT           : {}", human_size(total));
    println!();
    println!("  Notes:");
    println!("    - Sizes are ON DISK (not in RAM).");
    println!("    - mmap means the OS loads 4KB pages on first access.");
    println!("    - Typical RAM usage during a query: <100 KB per pack touched.");
    println!();

    Ok(())
}
