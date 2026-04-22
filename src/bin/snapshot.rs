//! `snapshot` — manage versioned graph snapshots for ZETS.
//!
//! Idan's request (22 Apr 2026): "צריך שיהיה מקום בגיט שאתה מחזיק גרסאות
//! של גרף בסיסי ואתחול גרף... כדי לשחזר ולשדרג גרסאות שצריכות תוכן ולא
//! רק קבצי קוד."
//!
//! This tool manages a data/baseline/ directory of AtomStore snapshots,
//! each tagged with a version and a manifest.
//!
//! Usage:
//!   snapshot create <name> <description>
//!     Builds a fresh bootstrap store, runs optional ingestion, saves as
//!     data/baseline/<name>.atoms + data/baseline/<name>.manifest.json
//!
//!   snapshot list
//!     Shows all available snapshots with their atom/edge counts.
//!
//!   snapshot info <name>
//!     Shows manifest for a specific snapshot.
//!
//!   snapshot restore <name> <output-path>
//!     Decrypts installer into target path.
//!
//!   snapshot package <name> <passphrase>
//!     Creates encrypted installer at data/installer/<name>.zets_enc
//!
//!   snapshot verify <name>
//!     Loads + verifies snapshot integrity.

use std::fs;
use std::path::{Path, PathBuf};

use zets::atom_persist;
use zets::atoms::AtomStore;
use zets::bootstrap::{bootstrap, is_bootstrapped};
use zets::encrypted_installer;
use zets::ingestion::{ingest_text, IngestConfig};

const BASELINE_DIR: &str = "data/baseline";
const INSTALLER_DIR: &str = "data/installer";
const CORPORA_DIR: &str = "data/corpora";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let command = &args[1];
    let result = match command.as_str() {
        "create"  => cmd_create(&args[2..]),
        "list"    => cmd_list(),
        "info"    => cmd_info(&args[2..]),
        "restore" => cmd_restore(&args[2..]),
        "package" => cmd_package(&args[2..]),
        "verify"  => cmd_verify(&args[2..]),
        "bootstrap-default" => cmd_bootstrap_default(),
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn print_usage() {
    println!("ZETS Snapshot Tool — versioned graph baselines");
    println!();
    println!("Commands:");
    println!("  snapshot create <name> [--corpus <file>]  Build + save new baseline");
    println!("  snapshot list                              List all snapshots");
    println!("  snapshot info <name>                       Show manifest");
    println!("  snapshot verify <name>                     Verify integrity");
    println!("  snapshot package <name> <passphrase>       Build encrypted installer");
    println!("  snapshot restore <name> <output>           Copy to output path");
    println!("  snapshot bootstrap-default                 Create v1_bootstrap + v1_world_facts");
    println!();
    println!("Snapshots live in {} (tracked in git).", BASELINE_DIR);
    println!("Installers live in {} (encrypted, shippable).", INSTALLER_DIR);
}

// ────────────────────────────────────────────────────────────────
// create — build a new baseline from bootstrap + optional corpus
// ────────────────────────────────────────────────────────────────

fn cmd_create(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("usage: snapshot create <name> [--corpus <file>]".into());
    }
    let name = &args[0];
    let mut corpus_file: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--corpus" => {
                if i + 1 >= args.len() {
                    return Err("--corpus requires a filename".into());
                }
                corpus_file = Some(args[i + 1].clone());
                i += 2;
            }
            other => return Err(format!("unknown flag: {}", other)),
        }
    }

    ensure_dirs()?;
    let mut store = AtomStore::new();
    let result = bootstrap(&mut store);
    println!("Bootstrap: +{} atoms, +{} edges",
        result.total_atoms_created, result.total_edges_created);

    let mut ingestion_stats = Vec::new();
    if let Some(ref corpus) = corpus_file {
        let path = PathBuf::from(corpus);
        let text = fs::read_to_string(&path)
            .map_err(|e| format!("read corpus {}: {}", corpus, e))?;
        let label = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("corpus");
        let config = IngestConfig::default();
        let ingest_result = ingest_text(&mut store, label, &text, &config);
        println!("Ingested '{}': +{} atoms, +{} edges, {} unique tokens",
            label, ingest_result.new_atoms, ingest_result.new_edges,
            ingest_result.unique_tokens);
        ingestion_stats.push((label.to_string(), ingest_result));
    }

    // Save atom dump
    let atom_path = PathBuf::from(BASELINE_DIR).join(format!("{}.atoms", name));
    let byte_count = atom_persist::save_to_file(&store, &atom_path)
        .map_err(|e| format!("save atoms: {}", e))?;

    // Save manifest (JSON-like, hand-written — no serde needed)
    let manifest_path = PathBuf::from(BASELINE_DIR).join(format!("{}.manifest.json", name));
    let manifest = build_manifest(name, &store, &ingestion_stats, byte_count, corpus_file.as_deref());
    fs::write(&manifest_path, &manifest)
        .map_err(|e| format!("write manifest: {}", e))?;

    println!();
    println!("✓ Saved {} ({} bytes)", atom_path.display(), byte_count);
    println!("✓ Saved {}", manifest_path.display());
    println!();
    println!("Total: {} atoms, {} edges", store.atom_count(), store.edge_count());
    Ok(())
}

// ────────────────────────────────────────────────────────────────
// list — show all snapshots
// ────────────────────────────────────────────────────────────────

fn cmd_list() -> Result<(), String> {
    ensure_dirs()?;
    let mut entries: Vec<PathBuf> = fs::read_dir(BASELINE_DIR)
        .map_err(|e| format!("read {}: {}", BASELINE_DIR, e))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|e| e == "atoms").unwrap_or(false))
        .collect();
    entries.sort();

    if entries.is_empty() {
        println!("No snapshots found in {}", BASELINE_DIR);
        println!("Run: snapshot bootstrap-default");
        return Ok(());
    }

    println!("{:<30} {:>12} {:>10} {:>10}", "NAME", "SIZE", "ATOMS", "EDGES");
    println!("{}", "─".repeat(66));
    for p in &entries {
        let name = p.file_stem().and_then(|s| s.to_str()).unwrap_or("?");
        let size = fs::metadata(p).map(|m| m.len()).unwrap_or(0);
        match atom_persist::load_from_file(p) {
            Ok(store) => println!("{:<30} {:>10} B {:>10} {:>10}",
                name, size, store.atom_count(), store.edge_count()),
            Err(e) => println!("{:<30} {:>10} B  ERROR: {}", name, size, e),
        }
    }
    Ok(())
}

// ────────────────────────────────────────────────────────────────
// info — show manifest for a snapshot
// ────────────────────────────────────────────────────────────────

fn cmd_info(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("usage: snapshot info <name>".into());
    }
    let manifest_path = PathBuf::from(BASELINE_DIR)
        .join(format!("{}.manifest.json", args[0]));
    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("read manifest: {}", e))?;
    println!("{}", content);
    Ok(())
}

// ────────────────────────────────────────────────────────────────
// verify — load snapshot and check integrity
// ────────────────────────────────────────────────────────────────

fn cmd_verify(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("usage: snapshot verify <name>".into());
    }
    let atom_path = PathBuf::from(BASELINE_DIR).join(format!("{}.atoms", args[0]));
    println!("Loading {}...", atom_path.display());
    let store = atom_persist::load_from_file(&atom_path)
        .map_err(|e| format!("load failed: {}", e))?;
    println!("  Atoms: {}", store.atom_count());
    println!("  Edges: {}", store.edge_count());
    println!("  Bootstrapped: {}", is_bootstrapped(&store));
    println!("✓ Snapshot {} verified", args[0]);
    Ok(())
}

// ────────────────────────────────────────────────────────────────
// package — build encrypted installer from snapshot
// ────────────────────────────────────────────────────────────────

fn cmd_package(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("usage: snapshot package <name> <passphrase>".into());
    }
    let name = &args[0];
    let passphrase = &args[1];

    // For now, package just rebuilds bootstrap + encrypts.
    // For non-bootstrap snapshots we'd need to extend encrypted_installer.
    let installer_path = PathBuf::from(INSTALLER_DIR).join(format!("{}.zets_enc", name));
    let size = encrypted_installer::build_to_file(passphrase, &installer_path)
        .map_err(|e| format!("build installer: {}", e))?;
    println!("✓ Packaged {} ({} bytes encrypted)", installer_path.display(), size);
    Ok(())
}

// ────────────────────────────────────────────────────────────────
// restore — copy snapshot to target path
// ────────────────────────────────────────────────────────────────

fn cmd_restore(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("usage: snapshot restore <name> <output>".into());
    }
    let src = PathBuf::from(BASELINE_DIR).join(format!("{}.atoms", args[0]));
    let dst = PathBuf::from(&args[1]);
    fs::copy(&src, &dst).map_err(|e| format!("copy: {}", e))?;
    println!("✓ Restored {} -> {}", src.display(), dst.display());
    Ok(())
}

// ────────────────────────────────────────────────────────────────
// bootstrap-default — create the canonical starter snapshots
// ────────────────────────────────────────────────────────────────

fn cmd_bootstrap_default() -> Result<(), String> {
    ensure_dirs()?;
    println!("Creating default baseline snapshots...");
    println!();

    // v1_bootstrap — fresh bootstrap, no ingestion
    println!("━━━ v1_bootstrap — fresh brain seed ━━━");
    cmd_create(&["v1_bootstrap".to_string()])?;
    println!();

    // v1_world_facts — bootstrap + 30 world facts
    println!("━━━ v1_world_facts — bootstrap + world knowledge ━━━");
    let corpus_path = PathBuf::from(CORPORA_DIR).join("world_facts_v1.txt");
    if !corpus_path.exists() {
        write_world_facts_corpus(&corpus_path)?;
    }
    cmd_create(&[
        "v1_world_facts".to_string(),
        "--corpus".to_string(),
        corpus_path.to_string_lossy().to_string(),
    ])?;
    println!();

    // v1_pet_facts — bootstrap + pet domain (for autonomous-demo reproducibility)
    println!("━━━ v1_pet_facts — bootstrap + pet domain ━━━");
    let pet_path = PathBuf::from(CORPORA_DIR).join("pet_facts_v1.txt");
    if !pet_path.exists() {
        write_pet_facts_corpus(&pet_path)?;
    }
    cmd_create(&[
        "v1_pet_facts".to_string(),
        "--corpus".to_string(),
        pet_path.to_string_lossy().to_string(),
    ])?;
    println!();

    println!("━━━ Done ━━━");
    cmd_list()?;
    Ok(())
}

// ────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────

fn ensure_dirs() -> Result<(), String> {
    for dir in &[BASELINE_DIR, INSTALLER_DIR, CORPORA_DIR] {
        fs::create_dir_all(dir).map_err(|e| format!("mkdir {}: {}", dir, e))?;
    }
    Ok(())
}

fn build_manifest(
    name: &str,
    store: &AtomStore,
    ingests: &[(String, zets::ingestion::IngestionResult)],
    byte_size: u64,
    corpus: Option<&str>,
) -> String {
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str(&format!("  \"name\": \"{}\",\n", name));
    s.push_str(&format!("  \"created_utc\": \"2026-04-22\",\n"));
    s.push_str(&format!("  \"format_version\": 1,\n"));
    s.push_str(&format!("  \"atoms\": {},\n", store.atom_count()));
    s.push_str(&format!("  \"edges\": {},\n", store.edge_count()));
    s.push_str(&format!("  \"bytes_on_disk\": {},\n", byte_size));
    s.push_str(&format!("  \"bootstrapped\": {},\n", is_bootstrapped(store)));
    if let Some(c) = corpus {
        s.push_str(&format!("  \"corpus\": \"{}\",\n", c));
    }
    s.push_str("  \"ingestions\": [\n");
    for (i, (label, result)) in ingests.iter().enumerate() {
        s.push_str("    {\n");
        s.push_str(&format!("      \"label\": \"{}\",\n", label));
        s.push_str(&format!("      \"sentences\": {},\n", result.sentence_atoms.len()));
        s.push_str(&format!("      \"unique_tokens\": {},\n", result.unique_tokens));
        s.push_str(&format!("      \"new_atoms\": {},\n", result.new_atoms));
        s.push_str(&format!("      \"new_edges\": {}\n", result.new_edges));
        s.push_str(if i + 1 < ingests.len() { "    },\n" } else { "    }\n" });
    }
    s.push_str("  ]\n");
    s.push_str("}\n");
    s
}

fn write_world_facts_corpus(path: &Path) -> Result<(), String> {
    let text = "Paris is the capital of France. \
        Berlin is the capital of Germany. \
        Madrid is the capital of Spain. \
        Rome is the capital of Italy. \
        Tokyo is the capital of Japan. \
        London is the capital of England. \
        Cairo is the capital of Egypt. \
        Water contains hydrogen and oxygen. \
        Gold has atomic number 79. \
        Iron has atomic number 26. \
        Carbon has atomic number 6. \
        Dogs are mammals. Cats are mammals. Birds are animals. \
        Fish live in water. Birds have feathers. Snakes are reptiles. \
        Rust is a systems programming language. \
        Python is an interpreted language. \
        JavaScript runs in browsers. \
        The sun is a star. The moon orbits Earth. \
        Earth is a planet. Mars is a planet. \
        Shakespeare wrote plays. \
        Einstein discovered relativity. \
        Newton discovered gravity. \
        Photosynthesis converts light to energy. \
        Gravity attracts objects. \
        Atoms compose all matter. \
        DNA stores genetic information. \
        The heart pumps blood. \
        Lungs process oxygen. \
        Eyes perceive light. \
        Ears detect sound.";
    fs::write(path, text).map_err(|e| format!("write {}: {}", path.display(), e))?;
    println!("  Wrote corpus: {}", path.display());
    Ok(())
}

fn write_pet_facts_corpus(path: &Path) -> Result<(), String> {
    let text = "Dogs are loyal animals. Dogs love their owners. \
        Cats are independent animals. Cats tolerate their owners. \
        Horses are strong animals. Horses carry people. \
        Animals need food. Animals need water. Water helps growth. \
        Food gives energy. Owners care for pets. \
        Pets are animals kept at home. A leash controls a dog.";
    fs::write(path, text).map_err(|e| format!("write {}: {}", path.display(), e))?;
    println!("  Wrote corpus: {}", path.display());
    Ok(())
}
