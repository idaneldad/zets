//! `zets-phrase` — compose/translate adjective+noun phrases across 10 languages.
//!
//! Demonstrates Idan's core insight: "big house" [en] and "בית גדול" [he]
//! are the SAME MEANING — only word order differs by language syntax rule.

use std::env;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;

use zets::concepts::ConceptStore;
use zets::phrase::{Phrase, PhraseComposer};

const CONCEPTS_DIR: &str = "data/multilang/_concepts";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  zets-phrase --demo                              Curated demo");
        eprintln!("  zets-phrase <lang> <noun> <adj>                 Compose in one language");
        eprintln!("  zets-phrase --all <lang> <noun> <adj>           Compose in all 10 langs");
        eprintln!("  zets-phrase --translate <from> <phrase> <to>    Translate phrase");
        return ExitCode::FAILURE;
    }

    let t0 = Instant::now();
    let mut cs = ConceptStore::new();
    match cs.load_from_dir(CONCEPTS_DIR) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to load concepts: {e}");
            return ExitCode::FAILURE;
        }
    }
    let load_ms = t0.elapsed();

    let stdout = io::stdout();
    let mut out = stdout.lock();

    writeln!(out, "═══════════════════════════════════════════════════════").ok();
    writeln!(out, " ZETS Phrase Composer — word order by language rule").ok();
    writeln!(out, "═══════════════════════════════════════════════════════").ok();
    writeln!(out, "  Concepts        : {}", cs.concept_count()).ok();
    writeln!(out, "  Surface bridges : {}", cs.link_count()).ok();
    writeln!(out, "  Load time       : {:?}", load_ms).ok();
    writeln!(out).ok();

    let composer = PhraseComposer::new(&cs);

    match args[1].as_str() {
        "--demo" => run_demo(&composer, &mut out),
        "--all" => {
            if args.len() < 5 {
                eprintln!("Usage: zets-phrase --all <lang> <noun> <adj>");
                return ExitCode::FAILURE;
            }
            compose_in_all(&composer, &args[2], &args[3], &args[4], &mut out);
        }
        "--translate" => {
            if args.len() < 5 {
                eprintln!("Usage: zets-phrase --translate <from> <phrase> <to>");
                return ExitCode::FAILURE;
            }
            translate(&composer, &args[2], &args[3], &args[4], &mut out);
        }
        lang => {
            if args.len() < 4 {
                eprintln!("Usage: zets-phrase <lang> <noun> <adj>");
                return ExitCode::FAILURE;
            }
            compose_one(&composer, lang, &args[2], &args[3], &mut out);
        }
    }

    ExitCode::SUCCESS
}

fn compose_one(c: &PhraseComposer, lang: &str, noun: &str, adj: &str, out: &mut dyn Write) {
    let t = Instant::now();
    if let Some(phrase) = c.compose_from_words(lang, noun, adj) {
        if let Some(r) = c.realize_in(&phrase, lang) {
            writeln!(out, "[{}]  noun='{}'  adj='{}'  →  {}", r.lang, r.noun_surface, r.adj_surface, r.text).ok();
        } else {
            writeln!(out, "No realization possible in {lang}.").ok();
        }
    } else {
        writeln!(out, "Could not find concepts for noun='{noun}' or adj='{adj}' in {lang}.").ok();
    }
    writeln!(out, "Time: {:?}", t.elapsed()).ok();
}

fn compose_in_all(
    c: &PhraseComposer,
    lang: &str,
    noun: &str,
    adj: &str,
    out: &mut dyn Write,
) {
    let t = Instant::now();
    let Some(phrase) = c.compose_from_words(lang, noun, adj) else {
        writeln!(
            out,
            "Could not find concepts for noun='{noun}' or adj='{adj}' in {lang}."
        )
        .ok();
        return;
    };

    writeln!(out, "─── Compose [{lang}] noun='{noun}' + adj='{adj}' → all languages ───").ok();
    writeln!(out, "  noun concept: #{}   adj concept: #{}", phrase.noun_concept.0, phrase.adj_concept.0).ok();
    writeln!(out).ok();

    let realizations = c.realize_all(&phrase);
    for r in &realizations {
        writeln!(
            out,
            "  [{}]  {}  (noun='{}' adj='{}')",
            r.lang, r.text, r.noun_surface, r.adj_surface
        )
        .ok();
    }
    writeln!(out).ok();
    writeln!(out, "Generated in: {:?}", t.elapsed()).ok();
}

fn translate(c: &PhraseComposer, from: &str, phrase: &str, to: &str, out: &mut dyn Write) {
    let t = Instant::now();
    writeln!(out, "─── Translate [{from}]  '{phrase}'  →  [{to}] ───").ok();
    if let Some(r) = c.translate_phrase(from, phrase, to) {
        writeln!(out, "  → {}", r.text).ok();
    } else {
        writeln!(out, "  (translation not possible with current concept data)").ok();
    }
    writeln!(out, "Time: {:?}", t.elapsed()).ok();
}

fn run_demo(c: &PhraseComposer, out: &mut dyn Write) {
    writeln!(out, "═══ DEMO: same meaning, ten languages, correct word order ═══").ok();
    writeln!(out).ok();

    // Core compositional tests — noun + adjective pairs
    let tests: &[(&str, &str, &str, &str)] = &[
        ("en", "house", "big",     "BIG HOUSE"),
        ("en", "house", "small",   "SMALL HOUSE"),
        ("en", "dog",   "big",     "BIG DOG"),
        ("en", "book",  "new",     "NEW BOOK"),
        ("en", "car",   "red",     "RED CAR"),
        ("he", "בית",    "גדול",    "BIG HOUSE (from Hebrew start)"),
        ("he", "כלב",    "קטן",     "SMALL DOG (from Hebrew start)"),
    ];

    for (src_lang, noun, adj, label) in tests {
        writeln!(out, "─ {label} — starting from [{src_lang}] '{noun}' + '{adj}' ─").ok();
        let Some(phrase) = c.compose_from_words(src_lang, noun, adj) else {
            writeln!(out, "  (concepts not found, skipping)").ok();
            writeln!(out).ok();
            continue;
        };
        for r in c.realize_all(&phrase) {
            let mark = if r.lang == *src_lang { " ←" } else { "  " };
            writeln!(out, "  [{}]{}  {}", r.lang, mark, r.text).ok();
        }
        writeln!(out).ok();
    }

    // Bidirectional phrase translation
    writeln!(out, "═══ Section 2: bidirectional phrase translation ═══").ok();
    writeln!(out).ok();

    let trans_tests: &[(&str, &str, &str)] = &[
        ("en", "big house", "he"),
        ("he", "בית גדול", "en"),
        ("en", "small dog", "fr"),
        ("he", "כלב קטן", "de"),
        ("en", "new book", "ru"),
        ("it", "casa grande", "he"),
    ];

    for (from, phrase, to) in trans_tests {
        writeln!(out, "  [{from}] '{phrase}'  →  [{to}]").ok();
        if let Some(r) = c.translate_phrase(from, phrase, to) {
            writeln!(out, "    = '{}'", r.text).ok();
        } else {
            writeln!(out, "    (not enough concept data)").ok();
        }
    }
    writeln!(out).ok();

    // Determinism proof
    writeln!(out, "═══ Section 3: determinism proof ═══").ok();
    let p1 = c.compose_from_words("en", "house", "big");
    let p2 = c.compose_from_words("en", "house", "big");
    let identical = match (&p1, &p2) {
        (Some(a), Some(b)) => a.noun_concept == b.noun_concept && a.adj_concept == b.adj_concept,
        _ => false,
    };
    writeln!(
        out,
        "Same phrase composed twice → identical concept pair: {identical}"
    )
    .ok();
    writeln!(out).ok();
    writeln!(out, "═══════════════════════════════════════════════════════").ok();
}
