//! `benchmark_runner` — run any JSONL benchmark against any snapshot.
//!
//! Usage:
//!   benchmark-runner --snapshot <n> --questions <jsonl> [--use-gemini]
//!
//! Reports per-category accuracy + overall score + trust recommendation.
//! This is the "measure training progress" tool.

use std::path::PathBuf;
use std::time::Instant;

use zets::atom_persist;
// AtomStore used transitively via atom_persist::load_from_file
use zets::benchmarks::{answer_question, load_jsonl, Question};
use zets::llm_adapter::{local_parse, LlmAdapter};
use zets::meta_learning::MetaLearner;

const BASELINE_DIR: &str = "data/baseline";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut snapshot: Option<String> = None;
    let mut questions_path: Option<String> = None;
    let mut use_gemini = false;
    let mut use_local_parser = true;  // default on

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--snapshot"          => { snapshot = Some(args[i+1].clone()); i += 2; }
            "--questions"         => { questions_path = Some(args[i+1].clone()); i += 2; }
            "--use-gemini"        => { use_gemini = true; i += 1; }
            "--no-parser"         => { use_local_parser = false; i += 1; }
            "--help" | "-h"       => { print_usage(); std::process::exit(0); }
            other => { eprintln!("unknown: {}", other); std::process::exit(1); }
        }
    }

    let snapshot = snapshot.unwrap_or_else(|| { print_usage(); std::process::exit(1); });
    let questions_path = questions_path.unwrap_or_else(|| { print_usage(); std::process::exit(1); });

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  ZETS Benchmark Runner                                    ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    // ─── Load snapshot ───
    let atom_path = PathBuf::from(BASELINE_DIR).join(format!("{}.atoms", snapshot));
    println!("  Loading snapshot: {}", atom_path.display());
    let mut store = match atom_persist::load_from_file(&atom_path) {
        Ok(s) => s,
        Err(e) => { eprintln!("  ✗ Load failed: {}", e); std::process::exit(1); }
    };
    println!("  ✓ {} atoms, {} edges", store.atom_count(), store.edge_count());

    // ─── Load questions ───
    let questions = match load_jsonl(std::path::Path::new(&questions_path)) {
        Ok(q) => q,
        Err(e) => { eprintln!("  ✗ Questions load failed: {}", e); std::process::exit(1); }
    };
    println!("  ✓ {} questions loaded from {}", questions.len(), questions_path);
    println!();

    // ─── Setup adapter if requested ───
    let mut adapter = if use_gemini {
        println!("  Adapter: REAL Gemini 2.5 Flash API (requires ZETS_GEMINI_KEY)");
        LlmAdapter::new()
    } else if use_local_parser {
        println!("  Adapter: local rule-based parser (offline, no API calls)");
        LlmAdapter::offline()
    } else {
        println!("  Adapter: none (raw tokens)");
        LlmAdapter::offline()
    };
    println!();

    // ─── Run ───
    println!("━━━ Running {} questions ━━━", questions.len());
    let start = Instant::now();
    let mut correct = 0;
    let mut total = 0;
    let mut by_cat: std::collections::HashMap<String, (usize, usize)> =
        std::collections::HashMap::new();
    let mut meta = MetaLearner::new();

    for q in &questions {
        total += 1;
        let augmented = if use_local_parser || use_gemini {
            let parse = adapter.parse(&q.text).unwrap_or_else(|_| local_parse(&q.text));
            Question {
                id: q.id.clone(),
                text: if parse.key_terms.is_empty() { q.text.clone() } else { parse.key_terms.join(" ") },
                choices: q.choices.clone(),
                expected: q.expected.clone(),
                category: if parse.domain != "general" { parse.domain.clone() } else { q.category.clone() },
            }
        } else {
            q.clone()
        };

        let result = answer_question(&mut store, &mut meta, &augmented);
        if result.correct { correct += 1; }
        let entry = by_cat.entry(q.category.clone()).or_insert((0, 0));
        entry.1 += 1;
        if result.correct { entry.0 += 1; }
    }
    let elapsed = start.elapsed();

    // ─── Report ───
    println!();
    println!("━━━ Results ━━━");
    let accuracy = correct as f32 / total as f32;
    println!("  Overall:     {:.1}%  ({}/{})", accuracy * 100.0, correct, total);
    println!("  Elapsed:     {:.2}s", elapsed.as_secs_f64());
    println!("  Throughput:  {:.0} q/s", total as f64 / elapsed.as_secs_f64().max(0.001));
    if use_gemini {
        println!("  Gemini calls: {}  (fallbacks: {})",
            adapter.parse_count() - adapter.fallback_count(),
            adapter.fallback_count());
    }
    println!();

    // Per-category
    println!("━━━ Per-category ━━━");
    let mut rows: Vec<(String, f32, usize, usize)> = by_cat.iter()
        .map(|(k, (c, t))| (k.clone(), *c as f32 / *t as f32, *c, *t))
        .collect();
    rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    for (cat, acc, c, t) in rows {
        let bar: String = (0..((acc * 20.0) as usize)).map(|_| '█').collect();
        println!("  {:<14} {:.1}%  ({}/{})  {}", cat, acc * 100.0, c, t, bar);
    }
    println!();

    // Judgment
    let judgment = if accuracy >= 0.85 { "excellent" }
        else if accuracy >= 0.7 { "good" }
        else if accuracy >= 0.5 { "acceptable" }
        else if accuracy >= 0.3 { "weak" }
        else { "poor" };
    println!("  Judgment: {}", judgment);

    println!();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  Benchmark complete                                        ║");
    println!("╚════════════════════════════════════════════════════════════╝");
}

fn print_usage() {
    println!("ZETS Benchmark Runner");
    println!();
    println!("Usage:");
    println!("  benchmark-runner --snapshot <name> --questions <jsonl> [options]");
    println!();
    println!("Options:");
    println!("  --snapshot <name>   Snapshot from data/baseline/<name>.atoms");
    println!("  --questions <path>  JSONL file of questions");
    println!("  --use-gemini        Use real Gemini 2.5 Flash (needs ZETS_GEMINI_KEY)");
    println!("  --no-parser         Skip LLM adapter (raw token matching)");
    println!();
    println!("Examples:");
    println!("  benchmark-runner --snapshot v1_world_facts \\");
    println!("                   --questions data/benchmarks/zets_expanded_32q_v1.jsonl");
}
