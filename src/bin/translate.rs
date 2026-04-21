//! `zets translate` — cross-language concept translation demo.
//!
//! This binary demonstrates the cross-language concept layer:
//! one meaning (Concept) → surfaces in many languages.
//!
//! Usage:
//!   zets-translate <from_lang> <word>              # show concept + all languages
//!   zets-translate <from_lang> <word> <to_lang>    # translate to specific language
//!   zets-translate --demo                          # curated demo

use std::env;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;

use zets::concepts::ConceptStore;
use zets::lexicon::Lexicon;

const DATA_DIR: &str = "data/multilang";
const CONCEPTS_DIR: &str = "data/multilang/_concepts";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  zets-translate <from_lang> <word>            Show all languages");
        eprintln!("  zets-translate <from_lang> <word> <to_lang>  Translate");
        eprintln!("  zets-translate --demo                        Demo");
        return ExitCode::FAILURE;
    }

    let t0 = Instant::now();

    let mut lex = Lexicon::new();
    let lex_stats = match lex.load_from_dir(DATA_DIR) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load lexicon: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut cs = ConceptStore::new();
    let cs_stats = match cs.load_from_dir(CONCEPTS_DIR) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load concepts: {e}");
            return ExitCode::FAILURE;
        }
    };

    let load_ms = t0.elapsed();

    let stdout = io::stdout();
    let mut out = stdout.lock();

    writeln!(out, "═══════════════════════════════════════════════════════").ok();
    writeln!(out, " ZETS Concept Layer — language-neutral meanings").ok();
    writeln!(out, "═══════════════════════════════════════════════════════").ok();
    writeln!(out, "  Lexicon entries : {}", lex.entry_count()).ok();
    writeln!(out, "  Concepts        : {}", cs.concept_count()).ok();
    writeln!(out, "  Surface bridges : {}", cs.link_count()).ok();
    writeln!(out, "  Languages       : {}", lex.languages().len()).ok();
    writeln!(
        out,
        "  Load time       : {:?} (lex={}, concepts={})",
        load_ms, lex_stats.definitions, cs_stats.surface_links
    )
    .ok();
    writeln!(out).ok();

    match args[1].as_str() {
        "--demo" => run_demo(&cs, &mut out),
        from_lang => {
            if args.len() < 3 {
                eprintln!("Missing word. Usage: zets-translate <from_lang> <word>");
                return ExitCode::FAILURE;
            }
            let word = &args[2];
            if args.len() >= 4 {
                let to_lang = &args[3];
                translate_pair(&cs, from_lang, word, to_lang, &mut out);
            } else {
                show_all_langs(&cs, from_lang, word, &mut out);
            }
        }
    }

    ExitCode::SUCCESS
}

fn show_all_langs(cs: &ConceptStore, from_lang: &str, word: &str, out: &mut dyn Write) {
    let t = Instant::now();
    let concepts = cs.concepts_for(from_lang, word);
    let took = t.elapsed();

    writeln!(
        out,
        "─── '{word}' in {from_lang} → all languages ───"
    )
    .ok();

    if concepts.is_empty() {
        writeln!(out, "No cross-language concept found for this surface.").ok();
        writeln!(out, "(Only present in its source language, no translation bridge).").ok();
        return;
    }

    writeln!(
        out,
        "Matched {} concept(s):",
        concepts.len()
    )
    .ok();

    for (i, cid) in concepts.iter().enumerate() {
        writeln!(out).ok();
        let concept = cs.get_concept(*cid);
        let english = concept.map(|c| c.english_anchor.as_str()).unwrap_or("?");
        let gloss = concept.map(|c| c.gloss.as_str()).unwrap_or("");
        writeln!(
            out,
            "Concept #{} ({}): anchor='{english}'  gloss='{gloss}'",
            i + 1,
            cid.0
        )
        .ok();

        let surfaces = cs.surfaces_of(*cid);
        let mut by_lang: std::collections::BTreeMap<String, Vec<String>> =
            std::collections::BTreeMap::new();
        for (lang, surface) in surfaces {
            by_lang.entry(lang).or_default().push(surface);
        }
        for (lang, surfaces) in &by_lang {
            writeln!(out, "  [{}]  {}", lang, surfaces.join(", ")).ok();
        }
    }

    writeln!(out).ok();
    writeln!(out, "Query time: {:?}", took).ok();
}

fn translate_pair(
    cs: &ConceptStore,
    from_lang: &str,
    word: &str,
    to_lang: &str,
    out: &mut dyn Write,
) {
    let t = Instant::now();
    let results = cs.cross_translate(from_lang, word, to_lang);
    let took = t.elapsed();

    writeln!(out, "─── Translate: {from_lang}:'{word}' → {to_lang} ───").ok();
    if results.is_empty() {
        writeln!(out, "No translation found.").ok();
    } else {
        writeln!(out, "Candidates in {to_lang}:").ok();
        for r in &results {
            writeln!(out, "  → {r}").ok();
        }
    }
    writeln!(out, "Query time: {:?}", took).ok();
}

fn run_demo(cs: &ConceptStore, out: &mut dyn Write) {
    writeln!(out, "═══ DEMO: cross-language concepts ═══").ok();
    writeln!(out).ok();

    let queries: &[(&str, &str)] = &[
        ("en", "dog"),
        ("he", "כלב"),
        ("de", "Hund"),
        ("fr", "chien"),
        ("en", "water"),
        ("he", "מים"),
        ("en", "book"),
        ("he", "ספר"),
        ("en", "house"),
        ("he", "בית"),
        ("en", "large"),
        ("he", "גדול"),
    ];

    for (lang, word) in queries {
        writeln!(out, "─ [{}] '{}' ─", lang, word).ok();
        let concepts = cs.concepts_for(lang, word);
        if concepts.is_empty() {
            writeln!(out, "  (no concept bridge)").ok();
        } else {
            // Pick first concept (most common sense)
            let cid = concepts[0];
            let concept = cs.get_concept(cid);
            if let Some(c) = concept {
                writeln!(
                    out,
                    "  concept#{}: {} ({})",
                    cid.0, c.english_anchor, c.gloss
                )
                .ok();
            }
            let surfaces = cs.surfaces_of(cid);
            let mut by_lang: std::collections::BTreeMap<String, Vec<String>> =
                std::collections::BTreeMap::new();
            for (l, s) in surfaces {
                by_lang.entry(l).or_default().push(s);
            }
            // Show up to 2 surfaces per language for compactness
            let snippet: Vec<String> = by_lang
                .iter()
                .map(|(l, v)| {
                    let shown: Vec<&String> = v.iter().take(2).collect();
                    format!(
                        "{}:{}",
                        l,
                        shown
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join("/")
                    )
                })
                .collect();
            writeln!(out, "  → {}", snippet.join("  ")).ok();
        }
        writeln!(out).ok();
    }

    // The "תיק גדול" demonstration — compositional phrase sharing a concept?
    // We don't have phrase-level concepts yet (Wiktionary has mostly single words).
    // But demonstrate single-word concept sharing:
    writeln!(
        out,
        "─ Compositional test: 'תיק' (Hebrew) and 'bag' (English) ─"
    )
    .ok();
    let he_concepts = cs.concepts_for("he", "תיק");
    let en_concepts = cs.concepts_for("en", "bag");
    let shared: Vec<_> = he_concepts
        .iter()
        .filter(|c| en_concepts.contains(c))
        .collect();
    if shared.is_empty() {
        writeln!(
            out,
            "  No shared concept yet (need richer translation data)."
        )
        .ok();
    } else {
        writeln!(
            out,
            "  Shared concepts: {:?}",
            shared.iter().map(|c| c.0).collect::<Vec<_>>()
        )
        .ok();
        writeln!(
            out,
            "  → 'תיק' [he] and 'bag' [en] point to the SAME meaning."
        )
        .ok();
    }
    writeln!(out).ok();

    writeln!(out, "═══ End of demo ═══").ok();
}
