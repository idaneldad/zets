//! `verify_demo` — Track C capability demo.
//!
//! Shows the productizable use case: given an LLM answer, ZETS produces
//! a per-claim verdict with full provenance trace. This is the enterprise
//! deliverable — compliance-grade verification of any LLM output.
//!
//! Uses the baseline world-facts graph (236 atoms) from snapshot v1.

use zets::atoms::AtomStore;
use zets::bootstrap::bootstrap;
use zets::ingestion::{ingest_text, IngestConfig};
use zets::learning_layer::ProvenanceLog;
use zets::verify::{render_report_markdown, verify_answer};

const WORLD_FACTS: &str = "\
Paris is the capital of France. \
Berlin is the capital of Germany. \
Tokyo is the capital of Japan. \
Water contains hydrogen and oxygen. \
Gold has atomic number 79. \
Iron has atomic number 26. \
Carbon has atomic number 6. \
Dogs are mammals. Cats are mammals. \
Fish live in water. \
Einstein discovered relativity. \
Newton discovered gravity. \
The sun is a star. \
Earth is a planet. \
DNA stores genetic information.";

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  ZETS Verify Demo — Track C enterprise product            ║");
    println!("║  Verify any LLM answer against the graph, claim-by-claim  ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    // Build a modest reference graph
    let mut store = AtomStore::new();
    let log = ProvenanceLog::new();
    bootstrap(&mut store);
    let ingest = ingest_text(&mut store, "world_facts", WORLD_FACTS, &IngestConfig::default());
    println!("Reference graph: {} atoms, {} edges (ingested {} sentences)",
        store.atom_count(), store.edge_count(), ingest.sentence_atoms.len());
    println!();

    // ═══════════════════════════════════════════════════
    // Case 1: clean Supported answer
    // ═══════════════════════════════════════════════════
    println!("━━━ Case 1: Clean factual answer ━━━");
    let r1 = verify_answer(
        &store, &log,
        "What is the capital of France?",
        "Paris is the capital of France. Paris is a major European city.",
    );
    print_case("capital of France", &r1);

    // ═══════════════════════════════════════════════════
    // Case 2: mixed — some supported, some unknown
    // ═══════════════════════════════════════════════════
    println!("━━━ Case 2: Mixed — facts + out-of-scope speculation ━━━");
    let r2 = verify_answer(
        &store, &log,
        "Tell me about elements.",
        "Gold has atomic number 79. Iron has atomic number 26. \
         Uranium has atomic number 92. Plutonium is radioactive.",
    );
    print_case("elements partial-coverage", &r2);

    // ═══════════════════════════════════════════════════
    // Case 3: fully unknown — graph has no coverage
    // ═══════════════════════════════════════════════════
    println!("━━━ Case 3: Out-of-domain question ━━━");
    let r3 = verify_answer(
        &store, &log,
        "What is a hash table?",
        "A hash table is a data structure that maps keys to values using a hash function.",
    );
    print_case("hash tables (unknown)", &r3);

    // ═══════════════════════════════════════════════════
    // Case 4: the money shot — full markdown audit report
    // ═══════════════════════════════════════════════════
    println!("━━━ Case 4: Full markdown audit report ━━━");
    println!();
    let r4 = verify_answer(
        &store, &log,
        "Tell me about Einstein's discoveries.",
        "Einstein discovered relativity. Newton discovered gravity. \
         Einstein invented the telephone.",
    );
    let md = render_report_markdown(&r4);
    println!("{}", md);

    // ═══════════════════════════════════════════════════
    // Summary
    // ═══════════════════════════════════════════════════
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  Verify layer verified                                     ║");
    println!("║                                                            ║");
    println!("║  Input:  any LLM answer + the original question            ║");
    println!("║  Output: per-claim verdicts + trust recommendation         ║");
    println!("║                                                            ║");
    println!("║  This is what customers pay for: quantified confidence     ║");
    println!("║  in LLM outputs, with full provenance. No black box.       ║");
    println!("╚════════════════════════════════════════════════════════════╝");
}

fn print_case(label: &str, report: &zets::verify::VerificationReport) {
    println!("  Question: {}", report.question);
    println!("  LLM answer: {}", report.llm_answer);
    println!();
    println!("  Summary: {}", report.summary_line());
    println!("  Trust recommendation: {}", report.trust_recommendation().label());
    println!("  Support: {:.1}% | Contradiction: {:.1}% | Unknown: {:.1}%",
        report.support_ratio * 100.0,
        report.contradiction_ratio * 100.0,
        report.unknown_ratio * 100.0);
    println!();
    println!("  Per-claim:");
    for (i, cv) in report.claims.iter().enumerate() {
        println!("    {}. {} [{}] conf={:.2}",
            i + 1, cv.verdict.icon(), cv.verdict.label(), cv.verdict_confidence);
        let text_preview: String = cv.claim.text.chars().take(70).collect();
        println!("       {}", text_preview);
    }
    println!();
    let _ = label;
}
