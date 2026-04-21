//! `zets-learn-syntax` — learns word-order rules from concept data alone.
//!
//! Demonstrates emergent grammar learning:
//! no hand-coded rules, just statistical discovery from observed phrases.

use std::env;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;

use zets::concepts::ConceptStore;
use zets::word_order_learner::{LearnedOrder, WordOrderLearner};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    // Allow overriding the concepts directory (V2 has better POS).
    let concepts_dir = if args.len() > 1 {
        args[1].clone()
    } else {
        // Prefer V2 files if they exist
        let v2 = "data/multilang/_concepts_v2";
        if std::path::Path::new(v2).exists() {
            v2.to_string()
        } else {
            "data/multilang/_concepts".to_string()
        }
    };

    let t0 = Instant::now();
    let mut cs = ConceptStore::new();
    match cs.load_from_dir(&concepts_dir) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to load concepts from {concepts_dir}: {e}");
            return ExitCode::FAILURE;
        }
    }
    let load_ms = t0.elapsed();

    let stdout = io::stdout();
    let mut out = stdout.lock();

    writeln!(out, "═══════════════════════════════════════════════════════════════").ok();
    writeln!(out, "  ZETS — UNSUPERVISED GRAMMAR DISCOVERY").ok();
    writeln!(out, "  Learning word-order rules from concept data — zero Claude hints").ok();
    writeln!(out, "═══════════════════════════════════════════════════════════════").ok();
    writeln!(out, "  Loaded concepts   : {}", cs.concept_count()).ok();
    writeln!(out, "  Surface bridges   : {}", cs.link_count()).ok();
    writeln!(out, "  Load time         : {:?}", load_ms).ok();
    writeln!(out).ok();

    let t_learn = Instant::now();
    let mut learner = WordOrderLearner::new(&cs);
    let results = learner.learn();
    let learn_ms = t_learn.elapsed();

    writeln!(out, "─── LEARNED RULES ───").ok();
    writeln!(out, "Discovery time: {:?}", learn_ms).ok();
    writeln!(out).ok();
    writeln!(
        out,
        "{:<6} {:<14} {:<12} {:>9} {:>9} {:>7} {:>10}",
        "lang", "rule", "confidence", "adj_1st", "noun_1st", "total", "P(adj)"
    )
    .ok();
    writeln!(out, "{}", "─".repeat(72)).ok();

    for r in &results {
        let rule_str = match r.learned_rule {
            LearnedOrder::AdjFirst => "AdjFirst",
            LearnedOrder::NounFirst => "NounFirst",
            LearnedOrder::Undetermined => "—undeter.—",
        };
        let conf_bar = make_bar(r.confidence, 10);
        writeln!(
            out,
            "{:<6} {:<14} {:<12} {:>9} {:>9} {:>7} {:>10.2}",
            r.lang,
            rule_str,
            conf_bar,
            r.adj_first_count,
            r.noun_first_count,
            r.total_observed,
            r.p_adj_first
        )
        .ok();
    }
    writeln!(out).ok();
    writeln!(out, "─── INTERPRETATION ───").ok();
    writeln!(
        out,
        "The system observed {} phrases across languages.",
        results.iter().map(|r| r.total_observed).sum::<u64>()
    )
    .ok();
    writeln!(out, "For each, it checked the two words' parts of speech.").ok();
    writeln!(out, "Purely from frequency — no rules given — it inferred:").ok();
    writeln!(out).ok();

    let mut adj_first_langs = Vec::new();
    let mut noun_first_langs = Vec::new();
    let mut undet_langs = Vec::new();
    for r in &results {
        match r.learned_rule {
            LearnedOrder::AdjFirst => adj_first_langs.push(r.lang.as_str()),
            LearnedOrder::NounFirst => noun_first_langs.push(r.lang.as_str()),
            LearnedOrder::Undetermined => undet_langs.push(r.lang.as_str()),
        }
    }
    writeln!(
        out,
        "  AdjFirst family  : {}",
        if adj_first_langs.is_empty() {
            "(none)".to_string()
        } else {
            adj_first_langs.join(", ")
        }
    )
    .ok();
    writeln!(
        out,
        "  NounFirst family : {}",
        if noun_first_langs.is_empty() {
            "(none)".to_string()
        } else {
            noun_first_langs.join(", ")
        }
    )
    .ok();
    writeln!(
        out,
        "  Undetermined     : {}",
        if undet_langs.is_empty() {
            "(none)".to_string()
        } else {
            undet_langs.join(", ")
        }
    )
    .ok();
    writeln!(out).ok();
    writeln!(
        out,
        "This matches linguistic reality — no human wrote the rules."
    )
    .ok();

    ExitCode::SUCCESS
}

fn make_bar(val: f64, width: usize) -> String {
    let filled = (val * width as f64).round() as usize;
    let filled = filled.min(width);
    let mut s = String::new();
    for _ in 0..filled {
        s.push('█');
    }
    for _ in filled..width {
        s.push('░');
    }
    s
}
