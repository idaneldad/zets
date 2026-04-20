//! ZETS CLI — thin wrapper over the `zets` library.
//!
//! This binary dispatches user commands to functions in `zets::lib`.
//! For test running, evaluation, and benchmarks, see the `tester` binary.

use std::env;
use std::process;

use zets::{hebrew, unp, LangCode, SynsetId};
use zets::{LANGUAGES, LANGUAGE_COUNT, RELATION_NAMES, RELATION_COUNT, RELATION_INVERSES,
           SYSTEM_SYNSETS, SYSTEM_SYNSET_COUNT, SystemSynset};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }
    let cmd = args[1].as_str();
    let rest = &args[2..];

    match cmd {
        "version" => println!("zets {}", env!("CARGO_PKG_VERSION")),
        "normalize" => cmd_normalize(rest),
        "stem-he" => cmd_stem_he(rest),
        "languages" => cmd_languages(),
        "relations" => cmd_relations(),
        "meta-graph" => cmd_meta_graph(),
        _ => {
            eprintln!("Unknown command: {cmd}");
            print_usage(&args[0]);
            process::exit(1);
        }
    }
}

fn print_usage(program: &str) {
    eprintln!("ZETS — Deterministic knowledge graph engine");
    eprintln!();
    eprintln!("Usage: {program} <command> [args...]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  normalize <lang> <text>   Normalize text via UNP (lang = ISO 639-1)");
    eprintln!("  stem-he <word>            Show Hebrew stemming variants");
    eprintln!("  languages                 List supported languages");
    eprintln!("  relations                 List relation types");
    eprintln!("  meta-graph                Show system synsets");
    eprintln!("  version                   Show version");
    eprintln!();
    eprintln!("For tests + benchmarks: run the `tester` binary.");
}

fn cmd_normalize(rest: &[String]) {
    if rest.len() < 2 {
        eprintln!("Usage: zets normalize <lang> <text>");
        process::exit(1);
    }
    let lang = match LangCode::parse(&rest[0]) {
        Some(l) => l,
        None => {
            eprintln!("Invalid language code: {}", rest[0]);
            process::exit(1);
        }
    };
    let text = rest[1..].join(" ");
    let canon = unp::normalize(&text, lang);
    println!("Input:     {text}");
    println!("Language:  {lang}");
    println!("Canonical: {}", canon.as_str());
    println!("Bytes:     {}", canon.len());
}

fn cmd_stem_he(rest: &[String]) {
    if rest.is_empty() {
        eprintln!("Usage: zets stem-he <word>");
        process::exit(1);
    }
    let word = &rest[0];
    println!("Input:              {word}");
    println!("Canonical (safe):   {}", hebrew::canonicalize(word));
    println!("Aggressive (Wk3+):  {}", hebrew::aggressive_stem_fallback(word));
}

fn cmd_languages() {
    println!("Supported languages ({LANGUAGE_COUNT}):");
    for l in LANGUAGES {
        let status = if l.default_active { "active" } else { "pack-on-demand" };
        let dir = if l.rtl { "RTL" } else { "LTR" };
        println!("  {:2} {:8} {:18} ({}) [{}] {}",
            l.code, l.script, l.name_english, l.name_native, dir, status);
    }
}

fn cmd_relations() {
    println!("Relation types ({RELATION_COUNT}):");
    for (i, name) in RELATION_NAMES.iter().enumerate() {
        let inv = RELATION_INVERSES[i];
        let inv_name = if inv == 255 { "(none)" } else { RELATION_NAMES[inv as usize] };
        println!("  {:2} {:15} inverse={}", i, name, inv_name);
    }
}

fn cmd_meta_graph() {
    println!("System synsets ({SYSTEM_SYNSET_COUNT} in reserved range 0..1000):");
    let mut by_cat: std::collections::BTreeMap<&str, Vec<&SystemSynset>> =
        std::collections::BTreeMap::new();
    for s in SYSTEM_SYNSETS {
        by_cat.entry(s.category).or_default().push(s);
    }
    for (cat, entries) in &by_cat {
        println!("\n  [{cat}]");
        for s in entries {
            println!("    {:4}  {}", s.id, s.key);
        }
    }
    println!("\nUser content starts at synset ID {}.", SynsetId::USER_CONTENT_START);
    println!("Documents start at synset ID {}.", zets::document::DOCUMENT_ID_START);
}
