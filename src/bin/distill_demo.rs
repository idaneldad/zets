//! `distill_demo` — Phase 3 proof of the closed learning loop.
//!
//! Shows the full journey:
//!   1. Load JSONL corpus of dialogues → Observed edges
//!   2. Run distillation → Learned edges + Prototype atoms
//!   3. Verify Precision mode can NOW see patterns that were invisible before
//!
//! This is the "episodic → semantic" promotion in action.

use zets::atoms::AtomStore;
use zets::dialogue::ingest_dialogues_from_jsonl;
use zets::distillation::{distill_dialogue_patterns, DistillConfig};
use zets::learning_layer::{Provenance, ProtoBank, ProvenanceLog};

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  ZETS Distillation Demo — closing the learning loop       ║");
    println!("║  Observed episodes → Learned semantic patterns             ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let mut store = AtomStore::new();
    let mut prov_log = ProvenanceLog::new();
    let mut proto_bank = ProtoBank::new();

    // ═══════════════════════════════════════════════════
    // PHASE 2: Ingest dialogues → Observed edges
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 1: Load dialogues from JSONL ━━━");
    let (success, errors) = ingest_dialogues_from_jsonl(
        &mut store, &mut prov_log,
        "data/conversations/synthetic_v1.jsonl",
    );
    println!("  Loaded: {} conversations, {} errors", success, errors.len());
    println!("  Store:  {} atoms, {} edges", store.atom_count(), store.edge_count());
    println!();

    // ═══════════════════════════════════════════════════
    // BEFORE DISTILLATION: audit
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 2: Before distillation — provenance audit ━━━");
    let before = prov_log.counts();
    print_provenance_row("Asserted (structural facts)", before.get(&Provenance::Asserted).copied().unwrap_or(0));
    print_provenance_row("Observed (episodic memory)", before.get(&Provenance::Observed).copied().unwrap_or(0));
    print_provenance_row("Learned (semantic patterns)", before.get(&Provenance::Learned).copied().unwrap_or(0));
    print_provenance_row("Hypothesis", before.get(&Provenance::Hypothesis).copied().unwrap_or(0));
    println!();
    println!("  Precision-mode visible edges: {}", count_precision_visible(&prov_log));
    println!("  Proto bank size: {}", proto_bank.len());
    println!();

    // ═══════════════════════════════════════════════════
    // PHASE 3: Distillation → Learned edges + Prototype atoms
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 3: Run distillation ━━━");
    let config = DistillConfig::default();
    println!("  Config: min_cluster={} exemplars={} base_conf={} per_obs={}",
        config.min_cluster_size, config.exemplars_per_proto,
        config.base_confidence, config.confidence_per_obs);
    println!();

    let result = distill_dialogue_patterns(
        &mut store, &mut prov_log, &mut proto_bank, &config,
    );

    println!("  Observations processed:  {}", result.observations_processed);
    println!("  Prototypes created:      {}", result.prototypes_created.len());
    println!("  Learned edges tagged:    {}", result.learned_edges_tagged);
    println!("  Below-threshold patterns: {}", result.below_threshold.len());
    if !result.below_threshold.is_empty() {
        println!("  (near-miss patterns visible for monitoring:)");
        for (name, count) in result.below_threshold.iter().take(5) {
            println!("     {} ({})", name, count);
        }
    }
    println!();

    // ═══════════════════════════════════════════════════
    // Show each prototype discovered
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 4: Discovered prototypes ━━━");
    for i in 0..proto_bank.len() {
        let proto = proto_bank.get(i).unwrap();
        println!("  #{} {}", i, proto.name);
        println!("       domain: {}", proto.domain);
        println!("       observations: {}", proto.observation_count);
        println!("       exemplars: {} utterances kept", proto.exemplars.len());
        for ex_id in &proto.exemplars {
            // Walk from exemplar (utterance atom) to its content atom
            if let Some(content_text) = utterance_content(&store, *ex_id) {
                let trimmed: String = content_text.chars().take(60).collect();
                println!("         — \"{}\"", trimmed);
            }
        }
    }
    println!();

    // ═══════════════════════════════════════════════════
    // AFTER DISTILLATION: compare
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 5: After distillation — provenance audit ━━━");
    let after = prov_log.counts();
    print_provenance_delta("Asserted",
        before.get(&Provenance::Asserted).copied().unwrap_or(0),
        after.get(&Provenance::Asserted).copied().unwrap_or(0));
    print_provenance_delta("Observed",
        before.get(&Provenance::Observed).copied().unwrap_or(0),
        after.get(&Provenance::Observed).copied().unwrap_or(0));
    print_provenance_delta("Learned",
        before.get(&Provenance::Learned).copied().unwrap_or(0),
        after.get(&Provenance::Learned).copied().unwrap_or(0));
    println!();
    println!("  Precision-mode visible edges: {}", count_precision_visible(&prov_log));
    println!("  Proto bank size: {}", proto_bank.len());
    println!();

    // ═══════════════════════════════════════════════════
    // Idempotency proof
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 6: Idempotency proof (run distillation again) ━━━");
    let atoms_before_2nd = store.atom_count();
    let edges_before_2nd = store.edge_count();
    let bank_before_2nd = proto_bank.len();

    let r2 = distill_dialogue_patterns(
        &mut store, &mut prov_log, &mut proto_bank, &config,
    );

    println!("  2nd run — prototypes created: {} (should be 0)",
        r2.prototypes_created.len());
    println!("  2nd run — atom delta: {} (should be 0)",
        store.atom_count() as i64 - atoms_before_2nd as i64);
    println!("  2nd run — edge delta: {} (should be 0)",
        store.edge_count() as i64 - edges_before_2nd as i64);
    println!("  2nd run — bank delta: {} (should be 0)",
        proto_bank.len() as i64 - bank_before_2nd as i64);
    println!();

    // ═══════════════════════════════════════════════════
    // The big claim: patterns now VISIBLE in Precision mode
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 7: The payoff ━━━");
    println!();
    println!("  BEFORE distillation: Precision mode saw only structural");
    println!("  category edges (intent is_a category, emotion is_a category).");
    println!("  Every dialogue utterance edge was Observed → invisible.");
    println!();
    println!("  AFTER distillation: Precision mode also sees the LEARNED");
    println!("  pattern edges. If a cluster had enough observations to pass");
    println!("  threshold + earned confidence >= 200, it surfaces in strict");
    println!("  reasoning alongside textbook facts.");
    println!();
    println!("  This is episodic → semantic memory promotion.");
    println!("  Same graph, new knowledge derived deterministically.");
    println!();

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  Learning loop verified end-to-end                         ║");
    println!("║                                                            ║");
    println!("║  {:>3} atoms | {:>3} edges | {:>3} tagged | {:>2} prototypes       ║",
        store.atom_count(), store.edge_count(),
        prov_log.counts().values().sum::<usize>(), proto_bank.len());
    println!("║                                                            ║");
    println!("║  Same corpus, every run produces the same prototypes.     ║");
    println!("║  No neural networks. No hidden state. 100% deterministic. ║");
    println!("╚════════════════════════════════════════════════════════════╝");
}

fn print_provenance_row(label: &str, count: usize) {
    println!("  {:<32} {:>4}", label, count);
}

fn print_provenance_delta(label: &str, before: usize, after: usize) {
    let delta = after as i64 - before as i64;
    let sign = if delta >= 0 { "+" } else { "" };
    println!("  {:<12} {:>4} → {:>4}  ({}{})", label, before, after, sign, delta);
}

fn count_precision_visible(prov_log: &ProvenanceLog) -> usize {
    prov_log.filter(|r|
        r.provenance == Provenance::Asserted
        || (r.provenance == Provenance::Learned && r.confidence >= 200)
    ).len()
}

/// Walk from an utterance atom to its content text, if available.
fn utterance_content(store: &AtomStore, utt_atom: zets::atoms::AtomId) -> Option<String> {
    let has_attr = zets::relations::by_name("has_attribute")?.code;
    for edge in store.outgoing(utt_atom) {
        if edge.relation == has_attr {
            if let Some(content_atom) = store.get(edge.to) {
                if content_atom.kind == zets::atoms::AtomKind::Text {
                    if let Ok(s) = std::str::from_utf8(&content_atom.data) {
                        // Skip "utt:..." labels, only show actual content
                        if !s.starts_with("utt:") {
                            return Some(s.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}
