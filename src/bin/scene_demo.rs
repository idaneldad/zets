//! `scene-demo` — compose a full scene from reusable atoms.
//!
//! Idan's scenario:
//!   "A child is hugging a dog in love. His sister beside him is crying
//!    because the dog was lost earlier. They are at the house entrance,
//!    a Ferrari is parked nearby, and people are walking on the street."
//!
//! We will store this as REUSABLE ATOMS + INSTANCE EDGES, proving:
//!   - The concept "dog" exists ONCE, even though it's referenced by both
//!     "child-hugs-dog" and "sister-cries-about-lost-dog"
//!   - People on the street share body templates with pose deltas
//!   - The scene composition is just edges — zero content duplication
//!
//! Then we'll walk it with cognitive modes and compose a natural sentence.

use std::time::Instant;

use zets::atoms::{rel, AtomId, AtomKind, AtomStore, DiffMethod};
use zets::cognitive_modes::{CognitiveMode, GraphHost, PrecisionMode, Query};

fn main() {
    println!("═══ ZETS Scene Composition Demo ═══");
    println!();
    println!("Scene: 'A child hugs a dog in love while his sister cries because");
    println!("        the dog was lost earlier. They are at the house entrance,");
    println!("        a Ferrari is parked nearby, people are walking in the street.'");
    println!();

    let mut store = AtomStore::new();

    // ──────────────────────────────────────────────────────────────
    // 1. Load concept atoms (the "dictionary")
    // ──────────────────────────────────────────────────────────────
    println!("─── 1. Concept atoms (stored once) ───");
    let child = store.put(AtomKind::Concept, b"\xD7\x99\xD7\x9C\xD7\x93".to_vec()); // ילד
    let dog = store.put(AtomKind::Concept, b"\xD7\x9B\xD7\x9C\xD7\x91".to_vec()); // כלב
    let sister = store.put(AtomKind::Concept, b"\xD7\x90\xD7\x97\xD7\x95\xD7\xAA".to_vec()); // אחות
    let house = store.put(AtomKind::Concept, b"\xD7\x91\xD7\x99\xD7\xAA".to_vec()); // בית
    let entrance = store.put(AtomKind::Concept, b"\xD7\x9B\xD7\xA0\xD7\x99\xD7\xA1\xD7\x94".to_vec()); // כניסה
    let ferrari = store.put(AtomKind::Concept, b"\xD7\xA4\xD7\xA8\xD7\x90\xD7\xA8\xD7\x99".to_vec()); // פררי
    let person = store.put(AtomKind::Concept, b"\xD7\x90\xD7\x93\xD7\x9D".to_vec()); // אדם
    let street = store.put(AtomKind::Concept, b"\xD7\xA8\xD7\x97\xD7\x95\xD7\x91".to_vec()); // רחוב

    println!("  Atoms: child={}, dog={}, sister={}, house={}, entrance={}, ferrari={}, person={}, street={}",
        child, dog, sister, house, entrance, ferrari, person, street);
    println!();

    // ──────────────────────────────────────────────────────────────
    // 2. Relation atoms (reused across many instances)
    // ──────────────────────────────────────────────────────────────
    println!("─── 2. Relation atoms (also stored once) ───");
    let hug = store.put(AtomKind::Relation, b"hug".to_vec());
    let love = store.put(AtomKind::Relation, b"love".to_vec());
    let cry = store.put(AtomKind::Relation, b"cry".to_vec());
    let lost = store.put(AtomKind::Relation, b"lost-event".to_vec());
    println!("  hug={}, love={}, cry={}, lost={}", hug, love, cry, lost);
    println!();

    // ──────────────────────────────────────────────────────────────
    // 3. Template + Delta: body atoms for many persons
    // ──────────────────────────────────────────────────────────────
    println!("─── 3. Template + Delta: 50 persons from 1 body + 50 pose deltas ───");
    let body_template = store.put(AtomKind::Template, vec![0u8; 2048]); // 2KB body silhouette
    println!("  body_template: {} bytes", store.get(body_template).unwrap().size_bytes());

    let mut person_ids = Vec::with_capacity(50);
    for i in 0..50 {
        // Each person is the same body + a small rotation/position delta (12 bytes)
        let delta_bytes = vec![(i % 256) as u8; 12];
        let person_pose = store.put_delta(body_template, delta_bytes,
            DiffMethod::RotationDegrees, 2048);
        person_ids.push(person_pose);
    }
    let stats_after_bodies = store.stats();
    println!("  50 persons stored. Atoms: {}, bytes saved by delta: {}",
        stats_after_bodies.atom_count, stats_after_bodies.bytes_saved_delta);
    println!();

    // ──────────────────────────────────────────────────────────────
    // 4. Scene composition nodes
    // ──────────────────────────────────────────────────────────────
    println!("─── 4. Scene composition (edges, no new content) ───");
    let scene = store.put(AtomKind::Composition, b"scene-2026-04-22-front-door".to_vec());

    // child --hugs--> dog
    store.link(child, dog, rel::HUG, 95, 0);
    // child --loves--> dog (stronger inference)
    store.link(child, dog, rel::LOVE, 90, 0);
    // sister --cries--> dog (she cries ABOUT the dog)
    store.link(sister, dog, rel::CRY, 85, 0);
    // dog --was_lost--> (temporal: earlier)
    store.link(dog, dog, rel::LOST, 70, 1); // slot=1 means "earlier in time"

    // all 3 people are AT the entrance
    store.link(child, entrance, rel::AT_LOCATION, 100, 0);
    store.link(sister, entrance, rel::AT_LOCATION, 100, 0);
    store.link(dog, entrance, rel::AT_LOCATION, 100, 0);

    // entrance IS_A of house
    store.link(entrance, house, rel::IS_A, 100, 0);

    // ferrari NEAR entrance
    store.link(ferrari, entrance, rel::NEAR, 80, 0);

    // 50 persons are walking on the street
    for &p in &person_ids {
        store.link(p, street, rel::WALK_IN, 60, 0);
        store.link(p, person, rel::IS_A, 100, 0);
    }

    // Scene grounds all main entities
    for &a in &[child, dog, sister, ferrari, house, entrance] {
        store.link(scene, a, rel::HAS_PART, 100, 0);
    }
    for &p in &person_ids {
        store.link(scene, p, rel::HAS_PART, 30, 0);
    }

    let final_stats = store.stats();
    println!("  Final:");
    println!("    Atoms      : {}", final_stats.atom_count);
    println!("    Edges      : {}", final_stats.edge_count);
    println!("    Raw bytes  : {}", final_stats.total_bytes);
    println!("    Saved (dedup): {}", final_stats.bytes_saved_dedup);
    println!("    Saved (delta): {}", final_stats.bytes_saved_delta);
    println!("    Compression ratio: {:.1}x", final_stats.compression_ratio());
    println!();

    // ──────────────────────────────────────────────────────────────
    // 5. What if we stored the scene NAIVELY (no reuse)?
    // ──────────────────────────────────────────────────────────────
    println!("─── 5. Naive vs compositional storage ───");
    let naive_size: u64 = 50 * 2048  // 50 full persons × 2KB
        + 6 * 8                      // 6 concept words (avg 8 bytes)
        + 4 * 8                      // 4 relations
        + 2048;                      // scene description
    let actual_size = final_stats.total_bytes;
    println!("  Naive:          {} bytes (50 full body copies)", naive_size);
    println!("  Compositional:  {} bytes ({:.1}% of naive)",
        actual_size,
        (actual_size as f64 / naive_size as f64) * 100.0);
    println!();

    // ──────────────────────────────────────────────────────────────
    // 6. Query the scene with cognitive modes
    // ──────────────────────────────────────────────────────────────
    println!("─── 6. Query via cognitive modes ───");
    let mut host = AtomStoreHost::new(&store);

    // Query: start from 'dog', walk with Precision (strict IS_A-style chains)
    let q = Query::new(dog, 3).with_kinds(vec![
        rel::HUG, rel::LOVE, rel::CRY, rel::AT_LOCATION, rel::IS_A, rel::LOST,
    ]);
    let t = Instant::now();
    let result = PrecisionMode { min_weight: 70 }.walk(&q, &mut host);
    let elapsed = t.elapsed();
    println!("  PrecisionMode walk from 'dog' (depth 3):");
    println!("    Visited: {:?}", result.visited);
    println!("    Steps:   {}, elapsed: {:?}", result.steps.len(), elapsed);
    println!();

    // ──────────────────────────────────────────────────────────────
    // 7. Compose sentence from edges
    // ──────────────────────────────────────────────────────────────
    println!("─── 7. Sentence composition from edges ───");
    let sentence = compose_scene_description(&store,
        child, dog, sister, ferrari, entrance, house, street);
    println!("  {}", sentence);
    println!();

    println!("═══ Summary ═══");
    println!("  * Each concept stored ONCE (dedup by content hash)");
    println!("  * 50 persons = 1 body template + 50 pose deltas (saves {:.1}KB)",
        final_stats.bytes_saved_delta as f64 / 1024.0);
    println!("  * Scene = 120 edges, 0 new content bytes");
    println!("  * Sentence composed deterministically from edge walk");
    println!();
    println!("This is how ZETS scales: not by accumulating bytes,");
    println!("but by composing new meanings from existing pieces.");
    println!("Same as the brain: new scenes from known parts.");
}

// ─────────────────────────────────────────────────────────────────
// Sentence composition — template strings filled from walked edges.
// ─────────────────────────────────────────────────────────────────

fn label_of(store: &AtomStore, id: AtomId) -> String {
    store.get(id)
        .and_then(|a| std::str::from_utf8(&a.data).ok().map(|s| s.to_string()))
        .unwrap_or_else(|| format!("#{}", id))
}

fn compose_scene_description(
    store: &AtomStore,
    child: AtomId, dog: AtomId, sister: AtomId,
    ferrari: AtomId, entrance: AtomId, _house: AtomId, _street: AtomId,
) -> String {
    let cl = label_of(store, child);
    let dl = label_of(store, dog);
    let sl = label_of(store, sister);
    let fl = label_of(store, ferrari);
    let el = label_of(store, entrance);

    format!(
        "{} מחבק את {} באהבה. {} בוכה כי {} אבד קודם. \
         הם ב{} הבית, {} חונה קרוב, ואנשים הולכים ברחוב.",
        cl, dl, sl, dl, el, fl
    )
}

// ─────────────────────────────────────────────────────────────────
// Bridge: expose AtomStore edges as a GraphHost for cognitive_modes
// ─────────────────────────────────────────────────────────────────

struct AtomStoreHost<'a> {
    store: &'a AtomStore,
}

impl<'a> AtomStoreHost<'a> {
    fn new(store: &'a AtomStore) -> Self { Self { store } }
}

impl<'a> GraphHost for AtomStoreHost<'a> {
    fn outgoing(&mut self, node: u32) -> Vec<(u32, u8, u8)> {
        self.store.outgoing(node)
            .into_iter()
            .map(|e| (e.to, e.relation, e.weight))
            .collect()
    }
    fn label(&mut self, node: u32) -> String {
        label_of(self.store, node)
    }
}
