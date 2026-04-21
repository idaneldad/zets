//! `zets-debug-disagreements` — show where Graph_B and Graph_A disagree.
//! Useful for diagnosing learner errors.

use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use zets::concepts::ConceptStore;
use zets::self_learner::SelfLearner;

const SEEDS_DIR: &str = "data/seeds";
const V2_CONCEPTS: &str = "data/multilang/_concepts_v2";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let target_lang = args.get(1).map(|s| s.as_str()).unwrap_or("en");
    let limit: usize = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(40);

    let mut graph_a = ConceptStore::new();
    if let Err(e) = graph_a.load_from_dir(V2_CONCEPTS) {
        eprintln!("Failed to load Graph_A: {e}");
        return ExitCode::FAILURE;
    }

    let mut learner = SelfLearner::new();
    let langs = ["en", "he", "de", "fr", "es", "it", "pt", "nl", "ru", "ar"];
    for lang in langs {
        let path = format!("{}/{}.tsv", SEEDS_DIR, lang);
        if Path::new(&path).exists() {
            let _ = learner.load_seed(lang, &path);
        }
    }
    let _ = learner.load_corpus(format!("{}/concept_surfaces.tsv", V2_CONCEPTS));
    learner.learn();

    let stdout = io::stdout();
    let mut out = stdout.lock();

    writeln!(out, "Disagreements for [{target_lang}] (Graph_B vs Graph_A):").ok();
    writeln!(out, "{:<25} {:<10} {:<10}", "word", "graph_b", "graph_a").ok();
    writeln!(out, "{}", "─".repeat(50)).ok();

    let mut shown = 0;
    let mut counts: std::collections::HashMap<(String, String), usize> = std::collections::HashMap::new();

    for cid_u32 in graph_a.all_concept_ids() {
        let Some(concept) = graph_a.get_concept(zets::concepts::ConceptId(cid_u32)) else {
            continue;
        };
        let pos_a = &concept.pos;
        if pos_a.is_empty() {
            continue;
        }
        let surfaces = graph_a.surfaces_of(zets::concepts::ConceptId(cid_u32));
        for (lang, surface) in surfaces {
            if lang != target_lang {
                continue;
            }
            if surface.contains(' ') {
                continue;
            }
            if let Some(pos_b) = learner.graph.pos_for(&lang, &surface) {
                if pos_b != pos_a {
                    *counts
                        .entry((pos_b.to_string(), pos_a.to_string()))
                        .or_insert(0) += 1;
                    if shown < limit {
                        writeln!(out, "{:<25} {:<10} {:<10}", surface, pos_b, pos_a).ok();
                        shown += 1;
                    }
                }
            }
        }
    }

    writeln!(out).ok();
    writeln!(out, "Disagreement summary (B → A, count):").ok();
    let mut sorted: Vec<_> = counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    for ((b, a), n) in sorted.iter().take(10) {
        writeln!(out, "  {} → {}  : {} times", b, a, n).ok();
    }

    ExitCode::SUCCESS
}
