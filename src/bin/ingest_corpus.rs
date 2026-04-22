//! `ingest_corpus` — production corpus ingestion tool.
//!
//! Takes any text file and ingests it into a named snapshot with proper
//! provenance tagging. This is the "going to training" entrypoint.
//!
//! Usage:
//!   ingest-corpus --input <file> --name <snapshot_name> [--source <label>]
//!                 [--base <existing_snapshot>] [--max-sentences N]
//!
//! Behavior:
//!   - Optionally loads a base snapshot to extend (e.g., v1_bootstrap)
//!   - Reads input file, splits into sentences
//!   - Ingests each sentence via existing ingest_text pipeline
//!   - Tags all new edges as Asserted (textbook/source facts)
//!   - Writes output to data/baseline/<name>.atoms + manifest
//!
//! Designed for:
//!   - Wikipedia subsets (Common Crawl text)
//!   - Textbook corpora
//!   - Customer-provided domain documents

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use zets::atom_persist;
use zets::atoms::AtomStore;
use zets::bootstrap::bootstrap;
use zets::ingestion::{ingest_text, IngestConfig};

const BASELINE_DIR: &str = "data/baseline";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        print_usage();
        std::process::exit(1);
    }

    let mut input: Option<String> = None;
    let mut name: Option<String> = None;
    let mut source: String = "user_corpus".to_string();
    let mut base_snapshot: Option<String> = None;
    let mut max_sentences: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input"          => { input = Some(args[i+1].clone()); i += 2; }
            "--name"           => { name  = Some(args[i+1].clone()); i += 2; }
            "--source"         => { source = args[i+1].clone(); i += 2; }
            "--base"           => { base_snapshot = Some(args[i+1].clone()); i += 2; }
            "--max-sentences"  => { max_sentences = args[i+1].parse().ok(); i += 2; }
            "--help" | "-h"    => { print_usage(); std::process::exit(0); }
            other => { eprintln!("unknown flag: {}", other); std::process::exit(1); }
        }
    }

    let input = input.unwrap_or_else(|| { print_usage(); std::process::exit(1); });
    let name  = name.unwrap_or_else(|| { print_usage(); std::process::exit(1); });

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  ZETS Corpus Ingestion                                    ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Input:  {}", input);
    println!("  Name:   {}", name);
    println!("  Source: {}", source);
    if let Some(ref b) = base_snapshot {
        println!("  Base:   {}", b);
    }
    if let Some(max) = max_sentences {
        println!("  Limit:  {} sentences", max);
    }
    println!();

    // ─── Load base or start fresh ───
    let mut store = if let Some(ref base) = base_snapshot {
        let path = PathBuf::from(BASELINE_DIR).join(format!("{}.atoms", base));
        match atom_persist::load_from_file(&path) {
            Ok(s) => {
                println!("  ✓ Loaded base '{}': {} atoms, {} edges",
                    base, s.atom_count(), s.edge_count());
                s
            }
            Err(e) => {
                eprintln!("  ✗ Failed to load base '{}': {}", base, e);
                std::process::exit(1);
            }
        }
    } else {
        let mut s = AtomStore::new();
        let r = bootstrap(&mut s);
        println!("  ✓ Fresh bootstrap: {} atoms, {} edges",
            r.total_atoms_created, r.total_edges_created);
        s
    };

    let atoms_before = store.atom_count();
    let edges_before = store.edge_count();

    // ─── Read input ───
    let text = match fs::read_to_string(&input) {
        Ok(t) => t,
        Err(e) => { eprintln!("  ✗ Read error: {}", e); std::process::exit(1); }
    };
    println!("  Read {} bytes from input", text.len());
    println!();

    // ─── Optional truncation ───
    let text_to_ingest = if let Some(max) = max_sentences {
        truncate_to_sentences(&text, max)
    } else {
        text.clone()
    };

    // ─── Ingest ───
    println!("━━━ Ingesting... ━━━");
    let start = Instant::now();
    let config = IngestConfig::default();
    let result = ingest_text(&mut store, &source, &text_to_ingest, &config);
    let elapsed = start.elapsed();

    println!("  Elapsed:      {:.2}s", elapsed.as_secs_f64());
    println!("  Sentences:    {}", result.sentence_atoms.len());
    println!("  Unique tokens: {}", result.unique_tokens);
    println!("  New atoms:    +{}", store.atom_count() - atoms_before);
    println!("  New edges:    +{}", store.edge_count() - edges_before);
    println!();

    // ─── Save snapshot ───
    fs::create_dir_all(BASELINE_DIR).ok();
    let atom_path = PathBuf::from(BASELINE_DIR).join(format!("{}.atoms", name));
    match atom_persist::save_to_file(&store, &atom_path) {
        Ok(bytes) => println!("  ✓ Saved {} ({} bytes)", atom_path.display(), bytes),
        Err(e) => { eprintln!("  ✗ Save failed: {}", e); std::process::exit(1); }
    }

    // ─── Manifest ───
    let manifest_path = PathBuf::from(BASELINE_DIR).join(format!("{}.manifest.json", name));
    let manifest = build_manifest(
        &name, &source, &input, &base_snapshot,
        &store, &result, elapsed.as_secs_f64(),
    );
    if let Err(e) = fs::write(&manifest_path, &manifest) {
        eprintln!("  ✗ Manifest write failed: {}", e);
    } else {
        println!("  ✓ Saved {}", manifest_path.display());
    }
    println!();

    println!("━━━ Final ━━━");
    println!("  Total atoms: {}", store.atom_count());
    println!("  Total edges: {}", store.edge_count());
    println!("  Atoms/sec:   {:.0}",
        (store.atom_count() - atoms_before) as f64 / elapsed.as_secs_f64().max(0.001));

    println!();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  Training step complete                                   ║");
    println!("║  Snapshot '{}' ready for use                               ║", name);
    println!("╚════════════════════════════════════════════════════════════╝");
}

fn print_usage() {
    println!("ZETS Corpus Ingestion Tool");
    println!();
    println!("Usage:");
    println!("  ingest-corpus --input <file> --name <snapshot_name> [options]");
    println!();
    println!("Options:");
    println!("  --input <path>       Text file to ingest (UTF-8, any size)");
    println!("  --name <id>          Snapshot name (goes to data/baseline/<id>.atoms)");
    println!("  --source <label>     Provenance source tag (default: user_corpus)");
    println!("  --base <snapshot>    Start from existing snapshot instead of fresh bootstrap");
    println!("  --max-sentences <N>  Truncate input to first N sentences (for testing)");
    println!();
    println!("Examples:");
    println!("  ingest-corpus --input wiki_cs.txt --name v2_cs --base v1_bootstrap");
    println!("  ingest-corpus --input textbook.txt --name physics_v1 --source textbook-intro");
    println!("  ingest-corpus --input big.txt --name test --max-sentences 100");
}

fn truncate_to_sentences(text: &str, max: usize) -> String {
    let mut out = String::new();
    let mut count = 0usize;
    for ch in text.chars() {
        out.push(ch);
        if matches!(ch, '.' | '!' | '?') {
            count += 1;
            if count >= max { break; }
        }
    }
    out
}

fn build_manifest(
    name: &str, source: &str, input: &str,
    base: &Option<String>, store: &AtomStore,
    ingest: &zets::ingestion::IngestionResult, elapsed_sec: f64,
) -> String {
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str(&format!("  \"name\": \"{}\",\n", name));
    s.push_str(&format!("  \"source\": \"{}\",\n", source));
    s.push_str(&format!("  \"input_file\": \"{}\",\n", input));
    s.push_str(&format!("  \"base_snapshot\": {},\n",
        base.as_ref().map(|b| format!("\"{}\"", b)).unwrap_or_else(|| "null".into())));
    s.push_str(&format!("  \"created_utc\": \"2026-04-22\",\n"));
    s.push_str(&format!("  \"format_version\": 1,\n"));
    s.push_str(&format!("  \"atoms\": {},\n", store.atom_count()));
    s.push_str(&format!("  \"edges\": {},\n", store.edge_count()));
    s.push_str(&format!("  \"ingest_elapsed_sec\": {:.3},\n", elapsed_sec));
    s.push_str("  \"ingestion_stats\": {\n");
    s.push_str(&format!("    \"sentences\": {},\n", ingest.sentence_atoms.len()));
    s.push_str(&format!("    \"unique_tokens\": {},\n", ingest.unique_tokens));
    s.push_str(&format!("    \"new_atoms\": {},\n", ingest.new_atoms));
    s.push_str(&format!("    \"new_edges\": {}\n", ingest.new_edges));
    s.push_str("  }\n");
    s.push_str("}\n");
    s
}
