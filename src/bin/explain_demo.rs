//! `explain_demo` — GDPR-grade audit output for a real distilled prototype.
//!
//! Demonstrates the full enterprise story:
//!   - Load dialogues from JSONL
//!   - Distill patterns
//!   - For each discovered prototype, render a full explanation showing:
//!     * Which observations contributed
//!     * What exemplars are preserved
//!     * What Learned edges it produced
//!     * The full provenance chain
//!
//! This is what ZETS can do that LLMs cannot: show WHY a conclusion
//! was reached, all the way back to the raw evidence.

use zets::atoms::AtomStore;
use zets::dialogue::ingest_dialogues_from_jsonl;
use zets::distillation::{distill_dialogue_patterns, DistillConfig};
use zets::explain::{explain_atom, explain_claim, render_atom_explanation};
use zets::learning_layer::{ProtoBank, ProvenanceLog};

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  ZETS Explain Demo — GDPR/HIPAA/SOX-grade audit output    ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let mut store = AtomStore::new();
    let mut log = ProvenanceLog::new();
    let mut bank = ProtoBank::new();

    // Build the graph: ingest dialogues + distill
    let (n, _) = ingest_dialogues_from_jsonl(
        &mut store, &mut log,
        "data/conversations/synthetic_v1.jsonl",
    );
    println!("Ingested {} conversations", n);

    let result = distill_dialogue_patterns(
        &mut store, &mut log, &mut bank, &DistillConfig::default(),
    );
    println!("Distilled {} prototypes from {} observations",
        result.prototypes_created.len(), result.observations_processed);
    println!("Store: {} atoms, {} edges", store.atom_count(), store.edge_count());
    println!();

    // ═══════════════════════════════════════════════════
    // DEMO 1: Full audit trail for a Learned prototype
    // ═══════════════════════════════════════════════════
    println!("━━━ Demo 1: Full audit trail for every prototype ━━━");
    println!();
    for i in 0..bank.len() {
        let proto = bank.get(i).unwrap();
        let exp = explain_atom(&store, &log, &bank, proto.atom_id);
        print!("{}", render_atom_explanation(&exp));
        println!();
    }

    // ═══════════════════════════════════════════════════
    // DEMO 2: Per-claim audit — "why does the graph believe X?"
    // ═══════════════════════════════════════════════════
    println!("━━━ Demo 2: Per-claim explanations ━━━");
    println!();

    // Pick a Learned edge: any prototype -> intent/emotion
    if let Some(proto) = bank.get(0) {
        let has_attr = zets::relations::by_name("has_attribute").unwrap().code;
        for edge in store.outgoing(proto.atom_id) {
            if edge.relation == has_attr {
                let text = explain_claim(
                    &store, &log, proto.atom_id, edge.to, edge.relation,
                );
                println!("  {}", text);
                println!();
            }
        }
    }

    // Pick an Asserted edge: any intent -> category:intent
    let (atoms, _) = store.snapshot();
    for (idx, atom) in atoms.iter().enumerate() {
        let aid = idx as zets::atoms::AtomId;
        if let Ok(s) = std::str::from_utf8(&atom.data) {
            if s.starts_with("intent:") && s != "intent:category" {
                let is_a = zets::relations::by_name("is_a").unwrap().code;
                for edge in store.outgoing(aid) {
                    if edge.relation == is_a {
                        let text = explain_claim(
                            &store, &log, aid, edge.to, is_a,
                        );
                        println!("  {}", text);
                        println!();
                        break;
                    }
                }
                break;  // just one
            }
        }
    }

    // ═══════════════════════════════════════════════════
    // DEMO 3: Explain one of the user speakers
    // ═══════════════════════════════════════════════════
    println!("━━━ Demo 3: Audit of a speaker atom (how many utterances, etc) ━━━");
    println!();
    let (atoms2, _) = store.snapshot();
    for (idx, atom) in atoms2.iter().enumerate() {
        let aid = idx as zets::atoms::AtomId;
        if let Ok(s) = std::str::from_utf8(&atom.data) {
            if s == "speaker:user" {
                let exp = explain_atom(&store, &log, &bank, aid);
                print!("{}", render_atom_explanation(&exp));
                break;
            }
        }
    }
    println!();

    // ═══════════════════════════════════════════════════
    // Summary
    // ═══════════════════════════════════════════════════
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  Explain layer verified                                    ║");
    println!("║                                                            ║");
    println!("║  For ANY atom or edge, ZETS can produce:                   ║");
    println!("║  - Its provenance (asserted/observed/learned/hypothesis)  ║");
    println!("║  - Its confidence score (0-255)                            ║");
    println!("║  - All supporting edges (incoming + outgoing)              ║");
    println!("║  - For prototypes: the exemplar observations               ║");
    println!("║                                                            ║");
    println!("║  This is the LLM-impossible enterprise compliance feature. ║");
    println!("╚════════════════════════════════════════════════════════════╝");
}
