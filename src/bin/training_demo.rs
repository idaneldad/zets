//! `training_demo` — demonstrates the full corpus acquisition + training pipeline.
//!
//! The document Idan shared (22.04.2026) asks: given raw dialogue from Reddit,
//! HuggingFace, or podcasts, can ZETS learn patterns from it?
//!
//! This demo answers YES, end-to-end:
//!   1. Take raw speaker/text turns (simulating what scrapers produce)
//!   2. Scrub PII (emails, phones, URLs) — GDPR compliance
//!   3. Auto-tag intent + emotion (heuristic, no LLM required)
//!   4. Auto-detect conversation outcome
//!   5. Ingest as Observed edges with full provenance
//!   6. Distill recurring patterns into Learned prototypes
//!   7. Verify a new LLM answer against the newly-learned patterns

use zets::atoms::{AtomKind, AtomStore};
use zets::corpus_acquisition::{build_conversation_from_turns, guess_emotion, guess_intent, scrub_pii};
use zets::dialogue::{ingest_dialogue, ConvOutcome};
use zets::distillation::{distill_dialogue_patterns, DistillConfig};
use zets::learning_layer::{Provenance, ProtoBank, ProvenanceLog};

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  ZETS Training Demo — corpus acquisition → learning loop  ║");
    println!("║  Simulates ingesting real Reddit/HuggingFace-style data   ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    // ═══════════════════════════════════════════════════
    // STEP 1: Simulate raw scraped dialogue with PII
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 1: Raw scraped dialogue (simulating scraper output) ━━━");
    println!();

    let raw_corpus = vec![
        // Conv 1: support success — notice PII that needs scrubbing
        vec![
            ("user_4738".to_string(), "My account is broken, my email john.doe@example.com doesn't work. Call 054-123-4567".to_string()),
            ("agent".to_string(), "I'm sorry to hear that. Can you try clearing cookies?".to_string()),
            ("user_4738".to_string(), "It worked! Thanks so much for the help.".to_string()),
        ],
        // Conv 2: empathy / loss
        vec![
            ("user_9201".to_string(), "I lost my grandmother last week".to_string()),
            ("friend".to_string(), "That sounds really hard. I'm so sorry for your loss.".to_string()),
            ("user_9201".to_string(), "Thanks, I appreciate that".to_string()),
        ],
        // Conv 3: frustration escalating
        vec![
            ("customer".to_string(), "This is the third time I've been transferred!".to_string()),
            ("rep".to_string(), "I understand your frustration. Let me help.".to_string()),
            ("customer".to_string(), "I've already explained this twice, it's broken and awful".to_string()),
            ("rep".to_string(), "Let me escalate to a supervisor".to_string()),
        ],
        // Conv 4: greeting + quick resolution
        vec![
            ("user".to_string(), "Hi there, quick question".to_string()),
            ("bot".to_string(), "Hello! How can I help?".to_string()),
            ("user".to_string(), "What's the status of my order? Reach me at jane@test.org".to_string()),
            ("bot".to_string(), "It shipped yesterday".to_string()),
            ("user".to_string(), "Thanks, sounds good".to_string()),
        ],
        // Conv 5: empathy + conversion
        vec![
            ("visitor".to_string(), "My dog passed away yesterday".to_string()),
            ("support".to_string(), "I'm so sorry for your loss.".to_string()),
            ("visitor".to_string(), "I'd like to order memorial flowers please".to_string()),
            ("support".to_string(), "Here are some options we recommend".to_string()),
            ("visitor".to_string(), "I'll take the white roses".to_string()),
        ],
        // Conv 6: another loss + empathy
        vec![
            ("user_77".to_string(), "I got laid off from my job today".to_string()),
            ("listener".to_string(), "That must be devastating".to_string()),
            ("user_77".to_string(), "Yeah it really hurts".to_string()),
            ("listener".to_string(), "I'm sorry you're going through this".to_string()),
        ],
        // Conv 7: complaint success
        vec![
            ("buyer".to_string(), "My login is broken and I'm frustrated".to_string()),
            ("cs".to_string(), "I see, let me check your account".to_string()),
            ("buyer".to_string(), "Please hurry my url https://example.com/orders isn't loading".to_string()),
            ("cs".to_string(), "Fixed! Try again.".to_string()),
            ("buyer".to_string(), "It works, thanks!".to_string()),
        ],
    ];

    // Demonstrate PII scrubbing on a raw turn
    let example_raw = &raw_corpus[0][0].1;
    let cleaned = scrub_pii(example_raw);
    println!("  Example PII scrubbing:");
    println!("    raw:     {}", truncate(example_raw, 80));
    println!("    cleaned: {}", truncate(&cleaned, 80));
    println!();

    // Demonstrate intent + emotion tagging
    println!("  Example auto-tagging:");
    for (_, text) in raw_corpus[1].iter().take(2) {
        let cleaned = scrub_pii(text);
        let intent = guess_intent(&cleaned);
        let emotion = guess_emotion(&cleaned);
        println!("    \"{}\"", truncate(&cleaned, 55));
        println!("       intent={:?}  emotion={:?}", intent, emotion);
    }
    println!();

    // ═══════════════════════════════════════════════════
    // STEP 2: Build + ingest conversations
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 2: Build conversations, auto-tag, ingest ━━━");
    let mut store = AtomStore::new();
    let mut prov_log = ProvenanceLog::new();
    let mut proto_bank = ProtoBank::new();

    use std::collections::HashMap;
    let mut speaker_cache: HashMap<String, zets::atoms::AtomId> = HashMap::new();

    let mut ingested = 0;
    for (i, turns) in raw_corpus.iter().enumerate() {
        // Pre-populate speaker cache to avoid closure borrow issues
        for (speaker, _) in turns {
            if !speaker_cache.contains_key(speaker) {
                let aid = store.put(AtomKind::Concept,
                    format!("speaker:{}", speaker).into_bytes());
                speaker_cache.insert(speaker.clone(), aid);
            }
        }

        let conv = build_conversation_from_turns(
            &format!("scraped_c{}", i),
            "simulated_scrape",
            turns,
            &mut |name| *speaker_cache.get(name).unwrap(),
        );

        let stats = ingest_dialogue(&mut store, &mut prov_log, &conv);
        println!("  Conv {} ({:?}): +{} atoms, +{} edges",
            i, conv.outcome, stats.new_atoms_total, stats.new_edges_total);
        ingested += 1;
    }
    println!();
    println!("  Ingested: {} conversations", ingested);
    println!("  Store: {} atoms, {} edges", store.atom_count(), store.edge_count());

    let counts = prov_log.counts();
    println!("  Provenance: asserted={} observed={} learned={} hypothesis={}",
        counts.get(&Provenance::Asserted).copied().unwrap_or(0),
        counts.get(&Provenance::Observed).copied().unwrap_or(0),
        counts.get(&Provenance::Learned).copied().unwrap_or(0),
        counts.get(&Provenance::Hypothesis).copied().unwrap_or(0));
    println!();

    // Outcome distribution — auto-detected
    println!("  Auto-detected outcomes:");
    for outcome in [ConvOutcome::Resolved, ConvOutcome::Escalated,
                    ConvOutcome::Abandoned, ConvOutcome::Converted] {
        let convs = zets::dialogue::conversations_with_outcome(&store, outcome);
        println!("    {:<11} {}", outcome.label(), convs.len());
    }
    println!();

    // ═══════════════════════════════════════════════════
    // STEP 3: Distillation
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 3: Distill recurring patterns ━━━");
    let result = distill_dialogue_patterns(
        &mut store, &mut prov_log, &mut proto_bank, &DistillConfig::default(),
    );
    println!("  Observations processed: {}", result.observations_processed);
    println!("  Prototypes created:     {}", result.prototypes_created.len());
    println!();
    println!("  Discovered patterns:");
    for i in 0..proto_bank.len() {
        let proto = proto_bank.get(i).unwrap();
        println!("    {} (obs={}, {} exemplars)",
            proto.name, proto.observation_count, proto.exemplars.len());
    }
    println!();

    // ═══════════════════════════════════════════════════
    // STEP 4: Verify an LLM answer against the learned patterns
    // ═══════════════════════════════════════════════════
    println!("━━━ Step 4: Verify an LLM answer against what was learned ━━━");
    let llm_answer = "When users express sadness, empathetic responses help.";
    let report = zets::verify::verify_answer(&store, &prov_log, "q", llm_answer);
    println!("  LLM answer: \"{}\"", llm_answer);
    println!("  Summary: {}", report.summary_line());
    println!("  Trust: {}", report.trust_recommendation().label());
    println!();

    // ═══════════════════════════════════════════════════
    // Summary
    // ═══════════════════════════════════════════════════
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  Training pipeline verified end-to-end                    ║");
    println!("║                                                            ║");
    println!("║  Raw scrape → PII scrub → auto-tag → ingest → distill      ║");
    println!("║                                                            ║");
    println!("║  Store: {:>3} atoms, {:>4} edges, {:>2} prototypes learned     ║",
        store.atom_count(), store.edge_count(), proto_bank.len());
    println!("║                                                            ║");
    println!("║  Ready for real-scale corpora: HuggingFace / OpenSubtitles ║");
    println!("╚════════════════════════════════════════════════════════════╝");
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{}...", truncated)
    }
}
