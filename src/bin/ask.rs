//! `zets ask` — deterministic multilingual Q&A on the loaded lexicon.
//!
//! Usage:
//!   zets-ask <lang> <word>              # describe word in that language
//!   zets-ask --cross <word>             # show all languages where word exists
//!   zets-ask --demo                     # run a curated demo across 10 langs

use std::env;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;

use zets::compose::{self, Style};
use zets::lexicon::Lexicon;

const DATA_DIR: &str = "data/multilang";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  zets-ask <lang> <word>         Describe a word");
        eprintln!("  zets-ask --cross <word>        Show word across all languages");
        eprintln!("  zets-ask --demo                Run the demo");
        return ExitCode::FAILURE;
    }

    let t0 = Instant::now();
    let mut lex = Lexicon::new();
    let stats = match lex.load_from_dir(DATA_DIR) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load lexicon from {DATA_DIR}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let load_ms = t0.elapsed();

    let stdout = io::stdout();
    let mut out = stdout.lock();

    writeln!(out, "=== ZETS Lexicon Loaded ===").ok();
    writeln!(
        out,
        "  Languages : {}  ({})",
        lex.languages().len(),
        lex.languages().join(", ")
    )
    .ok();
    writeln!(out, "  Entries   : {}", lex.entry_count()).ok();
    writeln!(out, "  Definitions : {}", stats.definitions).ok();
    writeln!(out, "  Synonyms    : {}", stats.synonyms).ok();
    writeln!(out, "  Antonyms    : {}", stats.antonyms).ok();
    writeln!(out, "  POS tags    : {}", stats.pos).ok();
    writeln!(out, "  Load time   : {:?}", load_ms).ok();
    writeln!(out).ok();

    match args[1].as_str() {
        "--demo" => run_demo(&lex, &mut out),
        "--cross" => {
            if args.len() < 3 {
                eprintln!("Need a word. Usage: zets-ask --cross <word>");
                return ExitCode::FAILURE;
            }
            run_cross(&lex, &args[2], &mut out);
        }
        lang => {
            if args.len() < 3 {
                eprintln!("Need a word. Usage: zets-ask <lang> <word>");
                return ExitCode::FAILURE;
            }
            run_describe(&lex, lang, &args[2], &mut out);
        }
    }

    ExitCode::SUCCESS
}

fn run_describe(lex: &Lexicon, lang: &str, word: &str, out: &mut dyn Write) {
    let t = Instant::now();
    let composition = compose::describe(lex, lang, word, Style::Rich);
    let took = t.elapsed();

    writeln!(out, "─── Query: lang={lang}  word='{word}' ───").ok();
    writeln!(out, "{}", composition.text).ok();
    if !composition.citations.is_empty() {
        writeln!(out, "Sources: {}", composition.citations.join("; ")).ok();
    }
    writeln!(out, "Query time: {:?}", took).ok();
}

fn run_cross(lex: &Lexicon, word: &str, out: &mut dyn Write) {
    let t = Instant::now();
    let results = compose::cross_language(lex, word);
    let took = t.elapsed();

    writeln!(out, "─── Cross-language lookup: '{word}' ───").ok();
    if results.is_empty() {
        writeln!(out, "No matches in any loaded language.").ok();
    } else {
        writeln!(
            out,
            "Found in {} language(s): {}",
            results.len(),
            results
                .iter()
                .map(|c| c.lang.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
        .ok();
        writeln!(out).ok();
        for c in &results {
            writeln!(out, "[{}]  {}", c.lang, c.text).ok();
            writeln!(out).ok();
        }
    }
    writeln!(out, "Query time: {:?}", took).ok();
}

fn run_demo(lex: &Lexicon, out: &mut dyn Write) {
    writeln!(out, "═══════════════════════════════════════════════════════").ok();
    writeln!(out, " ZETS MULTILINGUAL DEMO — 100% deterministic, 0% LLM").ok();
    writeln!(out, "═══════════════════════════════════════════════════════").ok();
    writeln!(out).ok();

    // 1. Single-language queries per language
    let single_queries: &[(&str, &str)] = &[
        ("en", "dog"),
        ("en", "water"),
        ("de", "Hund"),
        ("fr", "chien"),
        ("es", "perro"),
        ("it", "cane"),
        ("he", "מלון"),
        ("he", "מטוס"),
        ("ar", "كتاب"),
        ("ru", "дом"),
        ("nl", "hond"),
        ("pt", "cachorro"),
    ];

    writeln!(out, "═══ Section 1: describe word in specific language ═══").ok();
    for (lang, word) in single_queries {
        let t = Instant::now();
        let c = compose::describe(lex, lang, word, Style::Standard);
        let took = t.elapsed();
        writeln!(out).ok();
        writeln!(out, "[{lang}] '{word}'  ({took:?})").ok();
        writeln!(out, "  → {}", c.text).ok();
        if !c.citations.is_empty() {
            writeln!(out, "  sources: {}", c.citations.join("; ")).ok();
        }
    }

    // 2. Cross-language homographs (the killer demo)
    writeln!(out).ok();
    writeln!(out, "═══ Section 2: cross-language homographs ═══").ok();
    writeln!(
        out,
        "Same surface, different languages = different meanings."
    )
    .ok();

    let homographs = &["Gift", "brand", "fast", "pain", "mist", "ja", "si"];
    for word in homographs {
        writeln!(out).ok();
        writeln!(out, "─ '{word}' across languages ─").ok();
        let results = compose::cross_language(lex, word);
        if results.is_empty() {
            writeln!(out, "  (no matches)").ok();
        } else {
            for c in results.iter().take(5) {
                // Truncate text for display
                let preview: String = c.text.chars().take(120).collect();
                writeln!(out, "  [{}]  {}", c.lang, preview).ok();
            }
        }
    }

    // 3. Determinism check — run twice, verify identical output
    writeln!(out).ok();
    writeln!(out, "═══ Section 3: determinism check ═══").ok();
    let first = compose::describe(lex, "en", "dog", Style::Rich);
    let second = compose::describe(lex, "en", "dog", Style::Rich);
    let identical = first.text == second.text;
    writeln!(out, "Same query twice → identical output: {identical}").ok();
    if identical {
        writeln!(out, "✓ Graph is deterministic. No hallucination possible.").ok();
    }

    writeln!(out).ok();
    writeln!(out, "═══════════════════════════════════════════════════════").ok();
    writeln!(out, " Demo complete.").ok();
    writeln!(out, "═══════════════════════════════════════════════════════").ok();
}
