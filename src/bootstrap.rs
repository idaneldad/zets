//! Bootstrap — the "installer" for a fresh ZETS brain.
//!
//! Idan's requirement (22 Apr 2026): every new instance of ZETS must start
//! from the same FOUNDATIONAL RULES — the basic regularities of how a mind
//! works. Think of it as the 'operating system' of the brain: relations,
//! core concepts, initial skills, starter prototype chains.
//!
//! The bootstrap is deterministic: running it twice on a fresh store
//! produces the SAME atom IDs and edges, byte for byte. This is what
//! lets us distribute a "clean starter kit" for the graph.
//!
//! What bootstrap installs:
//!
//!   1. Meta-rules atoms (how reasoning works — "if X is_a Y and Y is_a Z
//!      then X is_a Z" encoded as atoms + edges)
//!   2. Core prototype chain (Thing → Entity → Living → Animal → Mammal)
//!   3. Core emotion atoms (joy, fear, sadness, anger, surprise, disgust)
//!   4. Core cognitive-mode labels (precision, divergent, gestalt, narrative)
//!   5. Appraisal dimensions (loss, threat, opportunity)
//!   6. Skill bootstrap categories (technical/procedural/conceptual)
//!   7. Relation metadata atoms (one per relation, so the graph can
//!      "reason about its own relations")
//!
//! All bootstrap atoms are tagged with prefix "zets:bootstrap:" so they
//! can be identified and regenerated if needed.

use crate::atoms::{AtomId, AtomKind, AtomStore};
use crate::relations;

/// Prefix used to mark all bootstrap atoms.
pub const BOOTSTRAP_PREFIX: &str = "zets:bootstrap:";

/// Return value: all the atoms created, for audit and chaining.
#[derive(Debug, Clone)]
pub struct BootstrapResult {
    pub meta_root: AtomId,
    pub thing_root: AtomId,
    pub emotions: Vec<(String, AtomId)>,
    pub modes: Vec<(String, AtomId)>,
    pub appraisals: Vec<(String, AtomId)>,
    pub total_atoms_created: usize,
    pub total_edges_created: usize,
}

/// Run the full bootstrap on an AtomStore. Safe to call on a store that
/// already has a partial bootstrap — content-hash dedup ensures idempotence.
pub fn bootstrap(store: &mut AtomStore) -> BootstrapResult {
    let atoms_before = store.atom_count();
    let edges_before = store.edge_count();

    // ── 1. Meta-rules root ──
    let meta_root = put_bootstrap(store, "meta_root");

    // Mark that the store has bootstrap rules (via self-link)
    let is_a = relations::by_name("is_a").unwrap().code;
    let has_attribute = relations::by_name("has_attribute").unwrap().code;
    let near = relations::by_name("near").unwrap().code;

    // ── 2. Core prototype chain ──
    // Thing → Entity → Living → Animal → Mammal → (user species)
    let thing = put_bootstrap(store, "Thing");
    let entity = put_bootstrap(store, "Entity");
    let living = put_bootstrap(store, "Living");
    let animal = put_bootstrap(store, "Animal");
    let mammal = put_bootstrap(store, "Mammal");

    let proto_of = crate::prototype::prototype_rel::PROTOTYPE_OF;
    store.link(entity, thing, proto_of, 100, 0);
    store.link(living, entity, proto_of, 100, 0);
    store.link(animal, living, proto_of, 100, 0);
    store.link(mammal, animal, proto_of, 100, 0);

    // ── 3. Core emotions (six basic emotions from appraisal theory) ──
    let emotion_labels = ["joy", "fear", "sadness", "anger", "surprise", "disgust"];
    let emotions: Vec<(String, AtomId)> = emotion_labels.iter().map(|&name| {
        let id = put_bootstrap(store, &format!("emotion:{}", name));
        store.link(id, meta_root, is_a, 80, 0);
        (name.to_string(), id)
    }).collect();

    // ── 4. Cognitive mode labels ──
    let mode_labels = ["precision", "divergent", "gestalt", "narrative"];
    let modes: Vec<(String, AtomId)> = mode_labels.iter().map(|&name| {
        let id = put_bootstrap(store, &format!("mode:{}", name));
        store.link(id, meta_root, is_a, 80, 0);
        (name.to_string(), id)
    }).collect();

    // ── 5. Appraisal dimensions ──
    let appraisal_labels = ["loss", "threat", "opportunity", "neutral"];
    let appraisals: Vec<(String, AtomId)> = appraisal_labels.iter().map(|&name| {
        let id = put_bootstrap(store, &format!("appraisal:{}", name));
        store.link(id, meta_root, is_a, 80, 0);
        (name.to_string(), id)
    }).collect();

    // ── 6. Skill categories ──
    let skill_technical = put_bootstrap(store, "skill_category:technical");
    let skill_procedural = put_bootstrap(store, "skill_category:procedural");
    let skill_conceptual = put_bootstrap(store, "skill_category:conceptual");
    store.link(skill_technical, meta_root, is_a, 80, 0);
    store.link(skill_procedural, meta_root, is_a, 80, 0);
    store.link(skill_conceptual, meta_root, is_a, 80, 0);

    // ── 7. Relation metadata atoms — one per relation, with the
    //     relation's code stored as has_attribute → (code atom) ──
    // This lets the graph reason about its own typed structure.
    for def in relations::ALL_RELATIONS.iter() {
        let rel_atom = put_bootstrap(store, &format!("relation:{}", def.name));
        store.link(rel_atom, meta_root, is_a, 70, 0);
    }

    // ── 8. Meta-reasoning rules (first-order inference principles) ──
    // Encoded as atoms linked to meta_root.
    let rules = [
        ("rule:transitivity",
         "If A is_a B and B is_a C, then A is_a C"),
        ("rule:symmetry_reject",
         "is_a is NOT symmetric: A is_a B does not imply B is_a A"),
        ("rule:parsimony",
         "Prefer fewer steps when multiple paths reach same conclusion"),
        ("rule:provenance_matters",
         "Claims are only trustworthy if their source can be verified"),
        ("rule:context_sensitivity",
         "Same surface form may map to different atoms given context"),
        ("rule:use_strengthens",
         "Repeated successful use increases edge weight"),
        ("rule:disuse_weakens",
         "Unused edges decay toward irrelevance"),
        ("rule:contradiction_flag",
         "If two paths reach opposite conclusions, mark both uncertain"),
        ("rule:emotion_shapes_attention",
         "High-valence events receive prioritized walk access"),
        ("rule:self_reflection_bound",
         "Meta-reasoning walks are capped at depth 3 to prevent loops"),
    ];
    for (name, description) in rules {
        let rule = put_bootstrap(store, name);
        store.link(rule, meta_root, is_a, 95, 0);
        // Attach description as a separate atom
        let desc = put_bootstrap(store, &format!("desc:{}", description));
        store.link(rule, desc, has_attribute, 90, 0);
    }

    // Root hierarchy connections
    store.link(thing, meta_root, near, 40, 0);
    let _ = mammal; // retained for caller awareness
    let _ = (skill_technical, skill_procedural, skill_conceptual);

    BootstrapResult {
        meta_root,
        thing_root: thing,
        emotions,
        modes,
        appraisals,
        total_atoms_created: store.atom_count() - atoms_before,
        total_edges_created: store.edge_count() - edges_before,
    }
}

/// Helper: create an atom with the bootstrap prefix.
/// Because AtomStore deduplicates by content hash, calling bootstrap
/// twice returns the SAME atom ids — idempotent.
fn put_bootstrap(store: &mut AtomStore, label: &str) -> AtomId {
    let data = format!("{}{}", BOOTSTRAP_PREFIX, label).into_bytes();
    store.put(AtomKind::Concept, data)
}

/// Check whether a store has been bootstrapped (looks for the meta_root atom).
pub fn is_bootstrapped(store: &AtomStore) -> bool {
    // Re-compute the meta_root's expected content hash
    let probe = format!("{}{}", BOOTSTRAP_PREFIX, "meta_root").into_bytes();
    // Put would dedup and find it — but we don't want to mutate.
    // Instead scan: for large stores this is O(N) but bootstrap is one-time.
    // Better: AtomStore could expose a `contains_by_hash()`. For now, scan.
    let target_hash = crate::atoms::content_hash(&probe);
    let (atoms, _) = store.snapshot();
    atoms.iter().any(|a| a.content_hash == target_hash)
}

/// Find a bootstrap atom by its suffix (e.g., "emotion:joy").
/// Returns None if bootstrap hasn't run.
pub fn find_bootstrap(store: &AtomStore, suffix: &str) -> Option<AtomId> {
    let needle = format!("{}{}", BOOTSTRAP_PREFIX, suffix);
    let needle_bytes = needle.as_bytes();
    let target_hash = crate::atoms::content_hash(needle_bytes);
    let (atoms, _) = store.snapshot();
    atoms.iter().position(|a| a.content_hash == target_hash).map(|i| i as u32)
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_store_is_not_bootstrapped() {
        let store = AtomStore::new();
        assert!(!is_bootstrapped(&store));
    }

    #[test]
    fn bootstrap_marks_store() {
        let mut store = AtomStore::new();
        bootstrap(&mut store);
        assert!(is_bootstrapped(&store));
    }

    #[test]
    fn bootstrap_is_idempotent() {
        let mut s1 = AtomStore::new();
        let mut s2 = AtomStore::new();
        let r1 = bootstrap(&mut s1);
        let r2 = bootstrap(&mut s2);
        assert_eq!(s1.atom_count(), s2.atom_count());
        assert_eq!(s1.edge_count(), s2.edge_count());
        assert_eq!(r1.meta_root, r2.meta_root);
        assert_eq!(r1.thing_root, r2.thing_root);
    }

    #[test]
    fn double_bootstrap_doesnt_duplicate() {
        let mut store = AtomStore::new();
        bootstrap(&mut store);
        let count_after_first = store.atom_count();
        bootstrap(&mut store);
        // Second call adds NO new atoms because of content-hash dedup
        assert_eq!(store.atom_count(), count_after_first);
    }

    #[test]
    fn find_bootstrap_returns_known_atoms() {
        let mut store = AtomStore::new();
        bootstrap(&mut store);
        assert!(find_bootstrap(&store, "meta_root").is_some());
        assert!(find_bootstrap(&store, "emotion:joy").is_some());
        assert!(find_bootstrap(&store, "mode:precision").is_some());
        assert!(find_bootstrap(&store, "rule:transitivity").is_some());
    }

    #[test]
    fn find_bootstrap_returns_none_for_missing() {
        let mut store = AtomStore::new();
        bootstrap(&mut store);
        assert!(find_bootstrap(&store, "emotion:nonexistent").is_none());
    }

    #[test]
    fn emotions_created() {
        let mut store = AtomStore::new();
        let result = bootstrap(&mut store);
        assert_eq!(result.emotions.len(), 6);
        for (name, _) in &result.emotions {
            let full_suffix = format!("emotion:{}", name);
            assert!(find_bootstrap(&store, &full_suffix).is_some());
        }
    }

    #[test]
    fn modes_created() {
        let mut store = AtomStore::new();
        let result = bootstrap(&mut store);
        assert_eq!(result.modes.len(), 4);
    }

    #[test]
    fn bootstrap_creates_reasoning_rules() {
        let mut store = AtomStore::new();
        bootstrap(&mut store);
        for rule in [
            "rule:transitivity",
            "rule:provenance_matters",
            "rule:use_strengthens",
            "rule:context_sensitivity",
        ] {
            assert!(find_bootstrap(&store, rule).is_some(),
                "rule '{}' missing after bootstrap", rule);
        }
    }

    #[test]
    fn bootstrap_has_relation_metadata() {
        let mut store = AtomStore::new();
        bootstrap(&mut store);
        // Every relation in the registry should have a metadata atom
        for def in relations::ALL_RELATIONS.iter() {
            let suffix = format!("relation:{}", def.name);
            assert!(find_bootstrap(&store, &suffix).is_some(),
                "missing relation metadata for '{}'", def.name);
        }
    }

    #[test]
    fn bootstrap_roundtrip_through_persistence() {
        let mut s1 = AtomStore::new();
        bootstrap(&mut s1);
        let mut buf = Vec::new();
        crate::atom_persist::serialize(&s1, &mut buf).unwrap();
        let s2 = crate::atom_persist::deserialize(&buf).unwrap();
        assert_eq!(s1.atom_count(), s2.atom_count());
        assert!(is_bootstrapped(&s2));
    }
}
