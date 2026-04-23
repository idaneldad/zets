//! zets_v4_ingest — CLI: ingest Wikipedia dumps → v4 snapshot.
//!
//! Usage:
//!   zets_v4_ingest --en 5000 --he 5000 --output data/baseline/v4_pilot.zv4
//!   zets_v4_ingest --en 100 --output /tmp/test.zv4      # quick test

use std::time::Instant;
use zets::graph_v4::{build_graph, save, wiki_reader, BuildConfig};

struct Args {
    en_max: usize,
    he_max: usize,
    min_len: usize,
    max_len: usize,
    output: String,
    dumps_dir: String,
}

impl Args {
    fn parse() -> Self {
        let mut a = Args {
            en_max: 0,
            he_max: 0,
            min_len: 2000,
            max_len: 30000,
            output: String::new(),
            dumps_dir: "/home/dinio/zets/data/wikipedia_dumps".into(),
        };
        let av: Vec<String> = std::env::args().collect();
        let mut i = 1;
        while i < av.len() {
            match av[i].as_str() {
                "--en" => { a.en_max = av[i+1].parse().unwrap_or(0); i += 2; }
                "--he" => { a.he_max = av[i+1].parse().unwrap_or(0); i += 2; }
                "--min-len" => { a.min_len = av[i+1].parse().unwrap_or(2000); i += 2; }
                "--max-len" => { a.max_len = av[i+1].parse().unwrap_or(30000); i += 2; }
                "--output" | "-o" => { a.output = av[i+1].clone(); i += 2; }
                "--dumps" => { a.dumps_dir = av[i+1].clone(); i += 2; }
                _ => { i += 1; }
            }
        }
        if a.output.is_empty() {
            eprintln!("Usage: zets_v4_ingest --en N [--he M] --output <path.zv4>");
            std::process::exit(1);
        }
        a
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  ZETS v4 INGEST                                           ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!("  en_max={}  he_max={}  output={}", args.en_max, args.he_max, args.output);

    let t_total = Instant::now();

    // ─── Step 1: read articles ───
    let mut articles: Vec<(String, String)> = Vec::new();
    if args.en_max > 0 {
        let path = format!("{}/en_parsed.jsonl.gz", args.dumps_dir);
        println!("\n[1a] Reading {} articles from {}...", args.en_max, path);
        let t = Instant::now();
        let r = wiki_reader::read_articles(&path, args.en_max, args.min_len, args.max_len)?;
        println!("     got {} EN articles in {:.1}s", r.len(), t.elapsed().as_secs_f32());
        for a in r {
            articles.push((a.title, a.text));
        }
    }
    if args.he_max > 0 {
        let path = format!("{}/he_parsed.jsonl.gz", args.dumps_dir);
        println!("\n[1b] Reading {} articles from {}...", args.he_max, path);
        let t = Instant::now();
        let r = wiki_reader::read_articles(&path, args.he_max, args.min_len, args.max_len)?;
        println!("     got {} HE articles in {:.1}s", r.len(), t.elapsed().as_secs_f32());
        for a in r {
            articles.push((a.title, a.text));
        }
    }
    println!("\n  Total articles: {}", articles.len());

    // ─── Step 2: build graph ───
    println!("\n[2] Building graph...");
    let t = Instant::now();
    let mut g = build_graph(&articles, &BuildConfig::default());
    g.build_indexes();
    println!("    built in {:.1}s", t.elapsed().as_secs_f32());

    // ─── Step 3: stats ───
    println!("\n[3] Stats:");
    g.stats().print();

    // ─── Step 4: save ───
    println!("\n[4] Saving to {}...", args.output);
    let t = Instant::now();
    save(&g, &args.output)?;
    let size = std::fs::metadata(&args.output)?.len();
    println!("    saved {:.1} MB in {:.1}s",
             size as f64 / 1_048_576.0, t.elapsed().as_secs_f32());

    println!("\n  TOTAL: {:.1}s", t_total.elapsed().as_secs_f32());
    Ok(())
}
