//! `zets-engine` — unified CLI for the ZETS knowledge graph.
//!
//! Commands:
//!   zets-engine stats
//!     — show engine stats after opening core + all languages
//!   zets-engine query <lang> <surface>
//!     — look up a word and print matching concepts
//!   zets-engine learn-pos <lang> <surface> <pos>
//!     — record a learned POS (persists via WAL)
//!   zets-engine learn-order <lang> <adj_first|noun_first|undetermined> <confidence>
//!     — record a learned word-order rule
//!   zets-engine show-wal
//!     — dump all WAL records

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use zets::engine::{WordOrder, ZetsEngine};
use zets::wal::{wal_path_for_core, WalReader};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        print_usage();
        return ExitCode::FAILURE;
    }

    let packs_dir = PathBuf::from("data/packs");
    let cmd = args[0].as_str();

    match cmd {
        "stats" => cmd_stats(&packs_dir, &args[1..]),
        "query" => cmd_query(&packs_dir, &args[1..]),
        "learn-pos" => cmd_learn_pos(&packs_dir, &args[1..]),
        "learn-order" => cmd_learn_order(&packs_dir, &args[1..]),
        "show-wal" => cmd_show_wal(&packs_dir),
        "show-order" => cmd_show_order(&packs_dir, &args[1..]),
        _ => {
            eprintln!("Unknown command: {}", cmd);
            print_usage();
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!("zets-engine — unified ZETS CLI");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  zets-engine stats [langs...]");
    eprintln!("  zets-engine query <lang> <surface>");
    eprintln!("  zets-engine learn-pos <lang> <surface> <noun|verb|adj|...>");
    eprintln!("  zets-engine learn-order <lang> <adj_first|noun_first|undetermined> <confidence>");
    eprintln!("  zets-engine show-order [lang]");
    eprintln!("  zets-engine show-wal");
}

fn cmd_stats(packs_dir: &PathBuf, args: &[String]) -> ExitCode {
    let t = Instant::now();
    let langs: Vec<&str> = if args.is_empty() {
        vec!["he", "en"]
    } else {
        args.iter().map(String::as_str).collect()
    };

    let engine = match ZetsEngine::open(packs_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Open failed: {}", e);
            return ExitCode::FAILURE;
        }
    };
    let mut engine = match engine.with_langs(&langs) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Language load failed: {}", e);
            return ExitCode::FAILURE;
        }
    };
    let elapsed = t.elapsed();

    let s = engine.stats();
    println!("═══ ZETS Engine Stats ═══");
    println!("Loaded in {:?}", elapsed);
    println!();
    println!("  Core file          : {:.1} MB", s.core_file_bytes as f64 / 1_048_576.0);
    println!("  Concepts           : {}", s.concept_count);
    println!("  Pieces             : {}", s.piece_count);
    println!("  Languages loaded   : {} / {} registered", s.languages_loaded, s.languages_registered);
    println!("  POS overrides (WAL): {}", s.pos_overrides);
    println!("  Word-order rules   : {}", s.word_order_rules);

    // Do a test query to warm up lookup
    println!();
    println!("Quick query test:");
    let test_cases: &[(&str, &str)] = &[
        ("en", "dog"),
        ("en", "big"),
        ("he", "גדול"),
        ("he", "בית"),
    ];
    for (lang, surface) in test_cases {
        if !langs.contains(lang) {
            continue;
        }
        let t = Instant::now();
        let results = engine.query(lang, surface);
        println!(
            "  [{}] \"{}\"  →  {} concepts  ({:?})",
            lang,
            surface,
            results.len(),
            t.elapsed()
        );
    }

    ExitCode::SUCCESS
}

fn cmd_query(packs_dir: &PathBuf, args: &[String]) -> ExitCode {
    if args.len() < 2 {
        eprintln!("Usage: zets-engine query <lang> <surface>");
        return ExitCode::FAILURE;
    }
    let lang = &args[0];
    let surface = &args[1];

    let mut engine = match ZetsEngine::open(packs_dir).and_then(|e| e.with_langs(&[lang.as_str()])) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Open failed: {}", e);
            return ExitCode::FAILURE;
        }
    };

    let results = engine.query(lang, surface);
    if results.is_empty() {
        println!("No concepts found for [{}] \"{}\"", lang, surface);
        return ExitCode::SUCCESS;
    }
    println!("[{}] \"{}\"  →  {} concept(s):", lang, surface, results.len());
    for r in results {
        let gloss_short: String = r.gloss.chars().take(80).collect();
        println!(
            "  c{} anchor=\"{}\" pos={} edges={}",
            r.concept_id, r.anchor, r.pos, r.edge_count
        );
        if !gloss_short.is_empty() {
            println!("       gloss: {}", gloss_short);
        }
    }

    // Show overridden POS (from WAL) if any
    if let Some(pos) = engine.pos_for_surface(lang, surface) {
        println!();
        println!("Current POS for \"{}\" in [{}]: {}", surface, lang, pos);
    }

    ExitCode::SUCCESS
}

fn cmd_learn_pos(packs_dir: &PathBuf, args: &[String]) -> ExitCode {
    if args.len() < 3 {
        eprintln!("Usage: zets-engine learn-pos <lang> <surface> <pos>");
        return ExitCode::FAILURE;
    }
    let (lang, surface, pos) = (&args[0], &args[1], &args[2]);
    let mut engine = match ZetsEngine::open(packs_dir).and_then(|e| e.with_langs(&[lang.as_str()])) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Open failed: {}", e);
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = engine.learn_pos(lang, surface, pos) {
        eprintln!("learn-pos failed: {}", e);
        return ExitCode::FAILURE;
    }
    if let Err(e) = engine.sync() {
        eprintln!("sync failed: {}", e);
        return ExitCode::FAILURE;
    }
    println!("Recorded: [{}] \"{}\" → pos={}", lang, surface, pos);
    ExitCode::SUCCESS
}

fn cmd_learn_order(packs_dir: &PathBuf, args: &[String]) -> ExitCode {
    if args.len() < 3 {
        eprintln!("Usage: zets-engine learn-order <lang> <rule> <confidence>");
        return ExitCode::FAILURE;
    }
    let (lang, rule_str, conf_str) = (&args[0], &args[1], &args[2]);
    let rule = match rule_str.as_str() {
        "adj_first" => WordOrder::AdjFirst,
        "noun_first" => WordOrder::NounFirst,
        "undetermined" => WordOrder::Undetermined,
        _ => {
            eprintln!("rule must be adj_first|noun_first|undetermined");
            return ExitCode::FAILURE;
        }
    };
    let Ok(conf) = conf_str.parse::<u16>() else {
        eprintln!("confidence must be u16 (0-65535)");
        return ExitCode::FAILURE;
    };

    let mut engine = match ZetsEngine::open(packs_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Open failed: {}", e);
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = engine.learn_order(lang, rule, conf) {
        eprintln!("learn-order failed: {}", e);
        return ExitCode::FAILURE;
    }
    if let Err(e) = engine.sync() {
        eprintln!("sync failed: {}", e);
        return ExitCode::FAILURE;
    }
    println!(
        "Recorded: [{}] word-order = {} (confidence {})",
        lang,
        rule.as_str(),
        conf
    );
    ExitCode::SUCCESS
}

fn cmd_show_order(packs_dir: &PathBuf, args: &[String]) -> ExitCode {
    let engine = match ZetsEngine::open(packs_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Open failed: {}", e);
            return ExitCode::FAILURE;
        }
    };
    if let Some(lang) = args.first() {
        let (rule, conf) = engine.word_order(lang);
        println!(
            "[{}] word-order = {} (confidence {})",
            lang,
            rule.as_str(),
            conf
        );
    } else {
        println!("Learned word-order rules:");
        for code in &engine.core.lang_codes {
            let (rule, conf) = engine.word_order(code);
            if rule != WordOrder::Undetermined || conf > 0 {
                println!("  [{}]  {}  confidence={}", code, rule.as_str(), conf);
            }
        }
    }
    ExitCode::SUCCESS
}

fn cmd_show_wal(packs_dir: &PathBuf) -> ExitCode {
    let path = wal_path_for_core(packs_dir);
    if !path.exists() {
        println!("No WAL at {:?}", path);
        return ExitCode::SUCCESS;
    }
    let mut r = match WalReader::open(&path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("open failed: {}", e);
            return ExitCode::FAILURE;
        }
    };
    let records = match r.read_all() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("read failed: {}", e);
            return ExitCode::FAILURE;
        }
    };
    println!("WAL contains {} records:", records.len());
    for rec in records {
        println!(
            "  ts={} kind={:?} payload_len={}",
            rec.timestamp_ms, rec.kind, rec.payload.len()
        );
    }
    ExitCode::SUCCESS
}
