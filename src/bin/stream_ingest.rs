//! `stream_ingest` — batch corpus ingestion from JSONL stream.
//!
//! Each line on stdin is a JSON object: {"title": "...", "text": "..."}
//! Reads, ingests in chunks, writes snapshot + periodic checkpoints.
//!
//! Designed for:
//!   - Wikipedia dumps (piped from xml_to_jsonl.py or api fetch)
//!   - PubMed abstracts (piped from efetch)
//!   - arXiv metadata (piped from OAI-PMH harvest)
//!   - Any custom corpus serialized as JSONL
//!
//! Usage:
//!   cat articles.jsonl | stream-ingest --name wikipedia_v1 --base v1_bootstrap
//!   curl api | stream-ingest --name pubmed_v1 --checkpoint-every 1000
//!
//! Key design: CHECKPOINTING + RESUMPTION.
//! If interrupted at article N, the snapshot has N-checkpoint_interval
//! articles persisted and can resume from the next batch.

use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::time::Instant;

use zets::atom_persist;
use zets::atoms::AtomStore;
use zets::bootstrap::bootstrap;
use zets::ingestion::{ingest_text, IngestConfig};

const BASELINE_DIR: &str = "data/baseline";
const LOG_DIR: &str = "data/log";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut name: Option<String> = None;
    let mut base: Option<String> = None;
    let mut checkpoint_every: usize = 1000;
    let mut max_articles: Option<usize> = None;
    let mut source: String = "stream".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--name"              => { name = Some(args[i+1].clone()); i += 2; }
            "--base"              => { base = Some(args[i+1].clone()); i += 2; }
            "--source"            => { source = args[i+1].clone(); i += 2; }
            "--checkpoint-every"  => { checkpoint_every = args[i+1].parse().unwrap_or(1000); i += 2; }
            "--max-articles"      => { max_articles = args[i+1].parse().ok(); i += 2; }
            "--help" | "-h"       => { print_usage(); std::process::exit(0); }
            other => { eprintln!("unknown: {}", other); std::process::exit(1); }
        }
    }

    let name = name.unwrap_or_else(|| { print_usage(); std::process::exit(1); });

    eprintln!("╔════════════════════════════════════════════════════════════╗");
    eprintln!("║  ZETS Stream Ingest                                       ║");
    eprintln!("╚════════════════════════════════════════════════════════════╝");
    eprintln!("  Name:             {}", name);
    eprintln!("  Source:           {}", source);
    eprintln!("  Checkpoint every: {} articles", checkpoint_every);
    if let Some(m) = max_articles { eprintln!("  Max articles:     {}", m); }
    eprintln!();

    // Initialize store (load base or fresh bootstrap)
    let mut store = if let Some(ref b) = base {
        let p = PathBuf::from(BASELINE_DIR).join(format!("{}.atoms", b));
        match atom_persist::load_from_file(&p) {
            Ok(s) => { eprintln!("  Loaded base: {} atoms", s.atom_count()); s }
            Err(e) => { eprintln!("  ✗ Load base failed: {}", e); std::process::exit(1); }
        }
    } else {
        let mut s = AtomStore::new();
        let r = bootstrap(&mut s);
        eprintln!("  Fresh bootstrap: {} atoms", r.total_atoms_created);
        s
    };

    std::fs::create_dir_all(LOG_DIR).ok();
    let log_path = PathBuf::from(LOG_DIR).join(format!("{}.log", name));
    let mut log_file = std::fs::File::create(&log_path).ok();

    // Main loop: parse JSONL lines, ingest in batches
    let stdin = std::io::stdin();
    let handle = stdin.lock();
    let cfg = IngestConfig::default();
    let start = Instant::now();
    let mut articles_processed = 0usize;
    let mut articles_skipped = 0usize;
    let mut total_sentences = 0usize;

    for line in handle.lines() {
        let raw = match line { Ok(l) => l, Err(_) => continue };
        if raw.trim().is_empty() { continue; }

        // Parse: expect {"title": "...", "text": "..."}
        let (title, text) = match parse_article_jsonl(&raw) {
            Some(t) => t,
            None => { articles_skipped += 1; continue; }
        };
        if text.len() < 50 { articles_skipped += 1; continue; }

        let label = format!("{}:{}", source, title);
        let _ = ingest_text(&mut store, &label, &text, &cfg);
        articles_processed += 1;
        total_sentences += count_sentences(&text);

        // Periodic status
        if articles_processed % 100 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let rate = articles_processed as f64 / elapsed.max(0.001);
            eprintln!("  [{:>6}] '{}' | {} atoms | {} edges | {:.1} art/sec",
                articles_processed,
                title.chars().take(40).collect::<String>(),
                store.atom_count(), store.edge_count(), rate);
            if let Some(f) = log_file.as_mut() {
                let _ = writeln!(f, "{},{},{},{:.1}",
                    articles_processed, store.atom_count(), store.edge_count(), rate);
            }
        }

        // Checkpoint
        if articles_processed % checkpoint_every == 0 {
            let checkpoint_name = format!("{}_ckpt_{}", name, articles_processed);
            save_snapshot(&store, &checkpoint_name);
            eprintln!("  ✓ Checkpoint: {}", checkpoint_name);
        }

        if let Some(m) = max_articles {
            if articles_processed >= m { break; }
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    eprintln!();
    eprintln!("━━━ Final ━━━");
    eprintln!("  Articles processed: {}", articles_processed);
    eprintln!("  Articles skipped:   {}", articles_skipped);
    eprintln!("  Total sentences:    {}", total_sentences);
    eprintln!("  Total atoms:        {}", store.atom_count());
    eprintln!("  Total edges:        {}", store.edge_count());
    eprintln!("  Elapsed:            {:.1}s", elapsed);
    eprintln!("  Throughput:         {:.1} articles/sec", articles_processed as f64 / elapsed.max(0.001));
    eprintln!();

    // Save final snapshot
    save_snapshot(&store, &name);
    eprintln!("  ✓ Saved final: data/baseline/{}.atoms", name);
}

fn save_snapshot(store: &AtomStore, name: &str) {
    let path = PathBuf::from(BASELINE_DIR).join(format!("{}.atoms", name));
    if let Err(e) = atom_persist::save_to_file(store, &path) {
        eprintln!("  ✗ Save failed: {}", e);
    }
    // Tiny manifest
    let man_path = PathBuf::from(BASELINE_DIR).join(format!("{}.manifest.json", name));
    let man = format!(
        "{{\n  \"name\": \"{}\",\n  \"atoms\": {},\n  \"edges\": {},\n  \"format_version\": 1\n}}\n",
        name, store.atom_count(), store.edge_count()
    );
    let _ = std::fs::write(&man_path, man);
}

/// Parse one line of JSONL: {"title": "...", "text": "..."}
/// Returns (title, text) or None if unparseable.
fn parse_article_jsonl(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if !line.starts_with('{') { return None; }
    let title = extract_str(line, "title")?;
    let text  = extract_str(line, "text")?;
    Some((title, text))
}

fn extract_str(s: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\":", key);
    let start = s.find(&needle)? + needle.len();
    let rest = s[start..].trim_start();
    let rest = rest.strip_prefix('"')?;

    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        match c {
            '"'  => return Some(out),
            '\\' => {
                match chars.next() {
                    Some('"')  => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some('n')  => out.push('\n'),
                    Some('t')  => out.push('\t'),
                    Some('r')  => out.push('\r'),
                    Some('/')  => out.push('/'),
                    Some('u')  => {
                        // \uXXXX — grab 4 hex chars, convert
                        let hex: String = chars.by_ref().take(4).collect();
                        if let Ok(n) = u32::from_str_radix(&hex, 16) {
                            if let Some(ch) = char::from_u32(n) { out.push(ch); }
                        }
                    }
                    Some(other) => { out.push('\\'); out.push(other); }
                    None => return None,
                }
            }
            _ => out.push(c),
        }
    }
    None
}

fn count_sentences(text: &str) -> usize {
    text.chars().filter(|c| matches!(c, '.' | '!' | '?')).count()
}

fn print_usage() {
    eprintln!("ZETS Stream Ingest — batch corpus ingester from JSONL stdin");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  stream-ingest --name <snapshot_name> [options] < articles.jsonl");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --name <n>               Output snapshot name (required)");
    eprintln!("  --base <snapshot>        Start from existing snapshot");
    eprintln!("  --source <label>         Source label for atoms (default: stream)");
    eprintln!("  --checkpoint-every <N>   Save snapshot every N articles (default: 1000)");
    eprintln!("  --max-articles <N>       Stop after N articles");
    eprintln!();
    eprintln!("Input format (JSONL, one article per line):");
    eprintln!("  {{\"title\": \"Paris\", \"text\": \"Paris is the capital of France...\"}}");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  # From Wikipedia API fetch script");
    eprintln!("  scripts/fetch_wikipedia.sh topics.txt | stream-ingest --name wiki_v1 --base v1_bootstrap");
    eprintln!();
    eprintln!("  # From file");
    eprintln!("  cat pubmed_abstracts.jsonl | stream-ingest --name pubmed_v1 --checkpoint-every 500");
}
