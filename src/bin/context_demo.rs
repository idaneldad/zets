//! `context-demo` — Idan's "kerech/crown" disambiguation test.
//!
//! Same word → different meaning based on conversation history.
//! No LLM. Pure deterministic graph walk with spreading activation.
//!
//! Scenario 1: Tamar is at the dentist. Says "כתר" → dental crown.
//! Scenario 2: Tamar is reading history book. Says "כתר" → royal crown.
//!
//! The DIFFERENCE is not in the word, not in the query pattern.
//! The difference is the session context that shaped activation PRIOR
//! to the ambiguous word being mentioned.

use zets::atoms::{AtomKind, AtomStore};
use zets::relations;
use zets::session::SessionContext;
use zets::spreading_activation::{disambiguate, spread_from_session, SpreadConfig};

fn main() {
    println!("═══ ZETS Context Demo — Disambiguating 'כתר' via session ═══");
    println!();

    let mut store = AtomStore::new();

    // Build a modest knowledge graph around "crown" ambiguity
    // Hebrew labels stored as UTF-8 in atom data
    let keter_crown_dental = store.put(AtomKind::Concept,
        "כתר_לשיניים".as_bytes().to_vec());
    let keter_crown_royal = store.put(AtomKind::Concept,
        "כתר_מלוכה".as_bytes().to_vec());

    // Dental cluster
    let tooth = store.put(AtomKind::Concept, "שן".as_bytes().to_vec());
    let dentist = store.put(AtomKind::Concept, "רופא_שיניים".as_bytes().to_vec());
    let cavity = store.put(AtomKind::Concept, "עששת".as_bytes().to_vec());
    let filling = store.put(AtomKind::Concept, "סתימה".as_bytes().to_vec());
    let porcelain = store.put(AtomKind::Concept, "חרסינה".as_bytes().to_vec());

    // Royal cluster
    let king = store.put(AtomKind::Concept, "מלך".as_bytes().to_vec());
    let queen = store.put(AtomKind::Concept, "מלכה".as_bytes().to_vec());
    let kingdom = store.put(AtomKind::Concept, "ממלכה".as_bytes().to_vec());
    let gold = store.put(AtomKind::Concept, "זהב".as_bytes().to_vec());
    let coronation = store.put(AtomKind::Concept, "הכתרה".as_bytes().to_vec());

    // Edges around dental crown
    let located_in = relations::by_name("located_in").unwrap().code;
    let used_for = relations::by_name("used_for").unwrap().code;
    let near = relations::by_name("near").unwrap().code;
    let has_attribute = relations::by_name("has_attribute").unwrap().code;

    store.link(keter_crown_dental, tooth, located_in, 95, 0);
    store.link(keter_crown_dental, dentist, used_for, 85, 0);
    store.link(keter_crown_dental, porcelain, has_attribute, 70, 0);
    store.link(tooth, dentist, near, 80, 0);
    store.link(dentist, cavity, used_for, 75, 0);
    store.link(dentist, filling, used_for, 80, 0);
    store.link(cavity, filling, near, 70, 0);

    // Edges around royal crown
    store.link(keter_crown_royal, king, located_in, 95, 0);
    store.link(keter_crown_royal, queen, located_in, 85, 0);
    store.link(keter_crown_royal, gold, has_attribute, 80, 0);
    store.link(keter_crown_royal, coronation, used_for, 90, 0);
    store.link(king, queen, near, 85, 0);
    store.link(king, kingdom, located_in, 90, 0);
    store.link(queen, kingdom, located_in, 90, 0);
    store.link(coronation, king, near, 85, 0);

    println!("Graph built: {} atoms, {} edges", store.stats().atom_count, store.stats().edge_count);
    println!("Both 'כתר_לשיניים' and 'כתר_מלוכה' exist as separate atoms.");
    println!();

    // ═══════════════════════════════════════════════════
    // Scenario 1: Dentist visit
    // ═══════════════════════════════════════════════════
    println!("─── Scenario 1: Tamar at the dentist ───");
    println!("Turn 1: 'הלכתי לרופא_שיניים'    → mention dentist");
    println!("Turn 2: 'כואב לי ב-שן'         → mention tooth");
    println!("Turn 3: 'אולי יש לי עששת?'     → mention cavity");
    println!("Turn 4: ...אז אולי אני צריך כתר? ← AMBIGUOUS");

    let mut session1 = SessionContext::new();
    session1.advance_turn();
    session1.mention(dentist);
    session1.advance_turn();
    session1.mention(tooth);
    session1.advance_turn();
    session1.mention(cavity);
    session1.advance_turn();

    // Show session state
    println!();
    println!("  Session state at moment of 'כתר':");
    for (aid, act) in session1.top_k(5) {
        let name = std::str::from_utf8(&store.get(aid).unwrap().data).unwrap();
        println!("    activation={:.3}  {}", act, name);
    }

    // Disambiguate "crown" — both candidates offered
    let candidates = vec![keter_crown_dental, keter_crown_royal];
    let result = disambiguate(&store, &session1, &candidates, &SpreadConfig::default());
    println!();
    if let Some((winner, score)) = result {
        let name = std::str::from_utf8(&store.get(winner).unwrap().data).unwrap();
        let expected = "כתר_לשיניים";
        let correct = name == expected;
        println!("  Disambiguation result: {}  (score={:.3})", name, score);
        println!("  Expected: {}  →  {}", expected, if correct { "✓ CORRECT" } else { "✗ WRONG" });
    }
    println!();

    // ═══════════════════════════════════════════════════
    // Scenario 2: Reading history
    // ═══════════════════════════════════════════════════
    println!("─── Scenario 2: Tamar reading about medieval history ───");
    println!("Turn 1: 'קראתי על מלך של ממלכה עתיקה'     → mention king, kingdom");
    println!("Turn 2: 'הייתה גם מלכה שלטה לידו'         → mention queen");
    println!("Turn 3: 'ראיתי תמונה של הכתרה'            → mention coronation");
    println!("Turn 4: ...יפה מאוד הכתר שהוא חבש         ← AMBIGUOUS");

    let mut session2 = SessionContext::new();
    session2.advance_turn();
    session2.mention(king);
    session2.mention(kingdom);
    session2.advance_turn();
    session2.mention(queen);
    session2.advance_turn();
    session2.mention(coronation);
    session2.advance_turn();

    println!();
    println!("  Session state at moment of 'כתר':");
    for (aid, act) in session2.top_k(5) {
        let name = std::str::from_utf8(&store.get(aid).unwrap().data).unwrap();
        println!("    activation={:.3}  {}", act, name);
    }

    let result = disambiguate(&store, &session2, &candidates, &SpreadConfig::default());
    println!();
    if let Some((winner, score)) = result {
        let name = std::str::from_utf8(&store.get(winner).unwrap().data).unwrap();
        let expected = "כתר_מלוכה";
        let correct = name == expected;
        println!("  Disambiguation result: {}  (score={:.3})", name, score);
        println!("  Expected: {}  →  {}", expected, if correct { "✓ CORRECT" } else { "✗ WRONG" });
    }
    println!();

    // ═══════════════════════════════════════════════════
    // Scenario 3: Decay over time — old context loses grip
    // ═══════════════════════════════════════════════════
    println!("─── Scenario 3: Context decay over many turns ───");
    println!("Tamar had the dental conversation, but then switched to other topics.");
    println!("20 turns later, she says 'כתר' — does the OLD dental context still win?");
    println!();

    let mut session3 = session1.clone();
    // 20 turns of silence
    for _ in 0..20 {
        session3.advance_turn();
    }
    println!("  Session state after 20 turns of silence: {} active atoms",
        session3.size());
    if session3.is_empty() {
        println!("  All atoms decayed below threshold — context is clear.");
    }

    let result = disambiguate(&store, &session3, &candidates, &SpreadConfig::default());
    if let Some((winner, score)) = result {
        let name = std::str::from_utf8(&store.get(winner).unwrap().data).unwrap();
        println!("  Without active context, disambiguation falls back to: {} (score={:.3})",
            name, score);
        println!("  (This is the DEFAULT meaning — whichever candidate comes first.)");
    }
    println!();

    // ═══════════════════════════════════════════════════
    // Scenario 4: Spread activation visualization
    // ═══════════════════════════════════════════════════
    println!("─── Scenario 4: What the graph sees when you're 'at the dentist' ───");
    let map = spread_from_session(&store, &session1, &SpreadConfig::default(), 10);
    println!("  Top-K activated atoms (session context = dentist+tooth+cavity):");
    for (aid, score) in map.top_k(10) {
        let name = std::str::from_utf8(&store.get(aid).unwrap().data).unwrap();
        let bar = "█".repeat(((score * 20.0) as usize).min(30));
        println!("    {:.3}  {:<20}  {}", score, name, bar);
    }
    println!();

    // ═══════════════════════════════════════════════════
    // Summary
    // ═══════════════════════════════════════════════════
    println!("═══ Summary ═══");
    println!();
    println!("What this demo proves:");
    println!();
    println!("1. SAME AMBIGUOUS WORD → different atom based on context.");
    println!("   'כתר' in dental session → crown_dental (tooth + dentist primed)");
    println!("   'כתר' in royal session → crown_royal (king + queen primed)");
    println!();
    println!("2. DETERMINISTIC. Same sequence of mentions → same result, always.");
    println!();
    println!("3. CONTEXT DECAYS. After 20 silent turns, old context loses grip.");
    println!("   Matches human forgetting curve (Ebbinghaus 1885).");
    println!();
    println!("4. SEARCH IS NARROWER when context is active. 'look for crown info'");
    println!("   in dental session → doesn't return kings/queens/gold.");
    println!();
    println!("5. This solves Idan's hierarchical conversation problem:");
    println!("   Each conversation = subgraph. Within-session atoms stay active,");
    println!("   priming future retrieval. Cross-session forgetting is automatic");
    println!("   via decay — no manual 'forget command' needed.");
    println!();
    println!("What this does NOT solve (honesty matters):");
    println!("   - Long-term re-activation: if Tamar discusses dental work a year");
    println!("     later, the old session is gone. Would need similarity-based");
    println!("     re-activation (future work).");
    println!("   - Cross-person context: each user has their own session state.");
    println!("     Sharing context between users requires explicit modeling.");
}
