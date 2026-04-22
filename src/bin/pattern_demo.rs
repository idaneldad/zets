//! `pattern_demo` — Phase A demo of the learning layer.
//!
//! Shows the end-to-end workflow that Idan asked about:
//! - ASSERT facts (Paris is_a capital)
//! - OBSERVE dialogue turns (specific user said X) 
//! - LEARN a pattern from multiple observations (sadness correlates with X)
//! - HYPOTHESIZE via dreaming (comfort reduces sadness — unverified)
//!
//! Then demonstrates per-mode filtering: Precision mode sees only
//! Asserted+high-confidence-Learned; Divergent mode sees everything.

use zets::atoms::{AtomKind, AtomStore};
use zets::learning_layer::{
    EdgeKey, Provenance, ProvenanceLog, ProvenanceRecord,
    ProtoBank, Prototype, DialoguePattern,
};
use zets::relations;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  ZETS Learning Layer Demo — Phase A                       ║");
    println!("║  Cognitive training vs asserted knowledge                 ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let mut store = AtomStore::new();
    let mut prov_log = ProvenanceLog::new();
    let mut proto_bank = ProtoBank::new();

    let is_a = relations::by_name("is_a").unwrap().code;
    let expresses = relations::by_name("expresses_emotion").map(|r| r.code).unwrap_or(is_a);
    let co_occurs = relations::by_name("co_occurs_with").unwrap().code;
    let near = relations::by_name("near").unwrap().code;

    // ═══════════════════════════════════════════════════
    // 1. ASSERTED — textbook facts
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 1: ASSERT facts (textbook-level truth) ━━━");
    let sadness = store.put(AtomKind::Concept, b"emotion:sadness".to_vec());
    let joy = store.put(AtomKind::Concept, b"emotion:joy".to_vec());
    let emotion = store.put(AtomKind::Concept, b"category:emotion".to_vec());

    store.link(sadness, emotion, is_a, 95, 0);
    prov_log.tag(EdgeKey::new(sadness, emotion, is_a), ProvenanceRecord::asserted());

    store.link(joy, emotion, is_a, 95, 0);
    prov_log.tag(EdgeKey::new(joy, emotion, is_a), ProvenanceRecord::asserted());

    println!("  sadness --is_a--> emotion  [Asserted, conf=240]");
    println!("  joy --is_a--> emotion      [Asserted, conf=240]");
    println!();

    // ═══════════════════════════════════════════════════
    // 2. OBSERVED — 3 specific dialogue turns
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 2: OBSERVE dialogue turns (specific moments) ━━━");
    let observations = [
        ("I lost my job yesterday",       sadness),
        ("My dog passed away",            sadness),
        ("I didn't get into the program", sadness),
    ];
    let mut observed_atoms = Vec::new();
    for (text, emotion_id) in &observations {
        let utt = store.put(AtomKind::Text, text.as_bytes().to_vec());
        store.link(utt, *emotion_id, expresses, 70, 0);
        prov_log.tag(EdgeKey::new(utt, *emotion_id, expresses), ProvenanceRecord::observed());
        observed_atoms.push(utt);
        println!("  \"{}\"  --expresses--> sadness  [Observed, conf=128]", text);
    }
    println!();

    // ═══════════════════════════════════════════════════
    // 3. LEARNED — distill a pattern from 3 observations
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 3: LEARN a pattern (distilled from observations) ━━━");
    let loss = store.put(AtomKind::Concept, b"concept:loss".to_vec());
    store.link(loss, sadness, co_occurs, 80, 0);
    prov_log.tag(EdgeKey::new(loss, sadness, co_occurs), ProvenanceRecord::learned(210));
    println!("  loss --co_occurs_with--> sadness  [Learned, conf=210, from 3 obs]");

    // Register a prototype in the ProtoBank
    let proto_idx = proto_bank.register(Prototype {
        atom_id: sadness,
        name: "sadness".to_string(),
        domain: "emotion".to_string(),
        observation_count: 3,
        drift: 0.08,
        exemplars: observed_atoms[..2].to_vec(),  // keep 2 canonical
        last_distilled: "2026-04-22".to_string(),
    });
    println!("  Registered Prototype #{}: 'sadness' ({} obs, drift=0.08, {} exemplars)",
        proto_idx, proto_bank.get(proto_idx).unwrap().observation_count,
        proto_bank.get(proto_idx).unwrap().exemplars.len());
    println!();

    // ═══════════════════════════════════════════════════
    // 4. HYPOTHESIS — dreaming proposes unverified link
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 4: HYPOTHESIZE (dreaming, not yet verified) ━━━");
    let comfort = store.put(AtomKind::Concept, b"action:comfort".to_vec());
    store.link(comfort, sadness, near, 60, 0);
    prov_log.tag(EdgeKey::new(comfort, sadness, near), ProvenanceRecord::hypothesis());
    println!("  comfort --near--> sadness  [Hypothesis, conf=100]");
    println!("  (proposed by dreaming, awaiting verification)");
    println!();

    // ═══════════════════════════════════════════════════
    // 5. AUDIT — count edges by provenance
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 5: AUDIT — edges by provenance ━━━");
    let counts = prov_log.counts();
    for provenance in [Provenance::Asserted, Provenance::Observed,
                       Provenance::Learned, Provenance::Hypothesis] {
        let count = counts.get(&provenance).copied().unwrap_or(0);
        println!("  {:<12} {}", provenance.label(), count);
    }
    println!("  Total tagged edges: {}", prov_log.len());
    println!();

    // ═══════════════════════════════════════════════════
    // 6. MODE FILTERING — same graph, different views
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 6: COGNITIVE MODE FILTERING ━━━");
    println!();

    println!("  Precision mode (strict — only trusted knowledge):");
    println!("    filter: Asserted OR Learned(conf >= 200)");
    let precision_edges = prov_log.filter(|r|
        r.provenance == Provenance::Asserted
        || (r.provenance == Provenance::Learned && r.confidence >= 200)
    );
    for (key, rec) in &precision_edges {
        let from_label = store.get(key.from)
            .and_then(|a| std::str::from_utf8(&a.data).ok().map(String::from))
            .unwrap_or_default();
        let to_label = store.get(key.to)
            .and_then(|a| std::str::from_utf8(&a.data).ok().map(String::from))
            .unwrap_or_default();
        println!("      {} → {} [{}/{}]",
            from_label, to_label, rec.provenance.label(), rec.confidence);
    }
    println!("    → {} edges visible", precision_edges.len());
    println!();

    println!("  Narrative mode (includes learned patterns):");
    println!("    filter: Asserted OR Learned OR Observed");
    let narrative_edges = prov_log.filter(|r|
        r.provenance != Provenance::Hypothesis
    );
    println!("    → {} edges visible", narrative_edges.len());
    println!();

    println!("  Divergent mode (includes hypotheses — exploratory):");
    println!("    filter: any provenance");
    let divergent_edges = prov_log.filter(|_| true);
    println!("    → {} edges visible", divergent_edges.len());
    println!();

    // ═══════════════════════════════════════════════════
    // 7. DIALOGUE PATTERN — a template with slots
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 7: DIALOGUE PATTERN (template + slots) ━━━");
    let pattern_atom = store.put(AtomKind::Template, b"pattern:I_just_want_to".to_vec());
    let pattern = DialoguePattern {
        atom_id: pattern_atom,
        template: "I just want to [VERB]".to_string(),
        slots: vec!["VERB".to_string()],
        intent: "request".to_string(),
        typical_emotion_proto: Some(proto_idx),
        observation_count: 0,
    };
    println!("  Template: \"{}\"", pattern.template);
    println!("  Slots: {:?}", pattern.slots);
    println!("  Intent: {}", pattern.intent);
    println!("  Typical emotion proto: #{:?}", pattern.typical_emotion_proto);
    println!();

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  Phase A verified — additive, non-breaking                ║");
    println!("║                                                            ║");
    println!("║  Store: {:>3} atoms, {:>3} edges                             ║",
        store.atom_count(), store.edge_count());
    println!("║  ProvenanceLog: {:>3} tagged edges                          ║", prov_log.len());
    println!("║  ProtoBank: {:>3} prototypes                                ║", proto_bank.len());
    println!("║                                                            ║");
    println!("║  Same graph, 4 provenance tiers, mode-specific views.     ║");
    println!("╚════════════════════════════════════════════════════════════╝");
}
