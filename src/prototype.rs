//! Prototype hierarchy — inheritance + variation on top of AtomStore.
//!
//! Concept (per consultation with Gemini + Groq):
//!   - Every creature/object is a PrototypeNode (an atom of kind Composition)
//!   - It has a PARENT via PROTOTYPE_OF edge (single inheritance for now)
//!   - It owns PARTS via HAS_PART edges (with slot names for disambiguation)
//!   - It owns PROPERTIES via HAS_SLOT edges (slot name → value atom)
//!
//! Resolution rule (Gemini's insight):
//!   To fully describe Rex: walk up the prototype chain
//!   (Rex → Poodle → Dog → Quadruped → Mammal)
//!   Collect all parts/slots. CHILD LEVEL WINS on conflicts.
//!   (Poodle's curly-coat overrides Mammal's generic-coat.)
//!
//! This gives us:
//!   - Storage: Each part/slot stored once, at the level where it's introduced
//!   - Inheritance: Rex automatically has 4 legs without re-declaring
//!   - Variation: Rex adds color=black, keeps everything else
//!   - Deterministic: traversal is deterministic (sorted), output is stable

use std::collections::HashMap;

use crate::atoms::{rel, AtomEdge, AtomId, AtomKind, AtomStore};

/// Extended relation kinds for inheritance.
pub mod prototype_rel {
    /// X PROTOTYPE_OF Y means "X is a specialization of Y".
    /// Traversal goes from child up to parent.
    pub const PROTOTYPE_OF: u8 = 20;
    /// X HAS_SLOT Y with slot_name tag means "X has property named slot_name = Y".
    pub const HAS_SLOT: u8 = 21;
    /// X OVERRIDES Y with slot_name tag means "X replaces parent's slot_name".
    /// This is implicit via resolve() — HAS_SLOT at lower level wins.
    pub const OVERRIDES: u8 = 22;
}

/// A resolved view — what an entity looks like after walking its full chain.
#[derive(Debug, Clone, Default)]
pub struct ResolvedEntity {
    /// All parts (named slot → atom_id), with child-level winning on conflicts.
    pub parts: HashMap<String, AtomId>,
    /// Properties (named slot → atom_id).
    pub slots: HashMap<String, AtomId>,
    /// The inheritance chain we walked (leaf to root).
    pub chain: Vec<AtomId>,
    /// Which node contributed which slot (for explainability).
    pub provenance: HashMap<String, AtomId>,
}

/// Builder for creating prototype hierarchies ergonomically on top of AtomStore.
pub struct Prototype<'a> {
    store: &'a mut AtomStore,
    id: AtomId,
}

impl<'a> Prototype<'a> {
    /// Create a new prototype node. If parent is Some, links PROTOTYPE_OF.
    pub fn create(store: &'a mut AtomStore, name: &str, parent: Option<AtomId>) -> Self {
        let id = store.put(AtomKind::Composition, name.as_bytes().to_vec());
        if let Some(p) = parent {
            store.link(id, p, prototype_rel::PROTOTYPE_OF, 100, 0);
        }
        Self { store, id }
    }

    pub fn id(&self) -> AtomId { self.id }

    /// Add a named part. The slot_name is stored in the AtomStore itself as
    /// a small string atom (dedup'd); the edge carries the link.
    /// We encode slot_name in a "slot_name_atom" approach: a name atom plus
    /// a slot-index hash for disambiguation when slot_name appears multiple
    /// times (e.g., 4 legs each with different slot tags).
    pub fn add_part(self, slot_name: &str, part_atom: AtomId) -> Self {
        // Encode slot_name as a small u16 hash — deterministic.
        let slot_tag = slot_hash(slot_name);
        // We also put the name itself as a small atom so explanations can
        // recover the string (it dedups with identical names elsewhere).
        let name_atom = self.store.put(AtomKind::Text, slot_name.as_bytes().to_vec());
        // Two edges: one for the part, one binding the slot-tag to the name.
        self.store.link(self.id, part_atom, rel::HAS_PART, 100, slot_tag);
        self.store.link(self.id, name_atom, prototype_rel::HAS_SLOT, 100, slot_tag);
        self
    }

    /// Add a simple slot (property) — slot_name → value.
    pub fn add_slot(self, slot_name: &str, value_atom: AtomId) -> Self {
        let slot_tag = slot_hash(slot_name);
        let name_atom = self.store.put(AtomKind::Text, slot_name.as_bytes().to_vec());
        self.store.link(self.id, value_atom, prototype_rel::HAS_SLOT, 100, slot_tag);
        // Also store the name for retrieval
        self.store.link(self.id, name_atom, prototype_rel::HAS_SLOT, 100,
            slot_tag.wrapping_add(0x8000)); // reserved high bit = name-binding
        self
    }
}

/// Deterministic hash of slot_name → u16.
fn slot_hash(name: &str) -> u16 {
    let mut h: u32 = 5381;
    for b in name.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u32);
    }
    (h & 0x7FFF) as u16 // keep low 15 bits
}

/// Walk the prototype chain from `id` up to root, collecting all slots.
/// CHILD-LEVEL WINS on conflicts (Gemini's rule).
pub fn resolve(store: &AtomStore, id: AtomId) -> ResolvedEntity {
    let mut entity = ResolvedEntity::default();
    let mut current = Some(id);
    let mut safety = 32; // bounded: no runaway

    while let Some(n) = current {
        if safety == 0 { break; }
        safety -= 1;
        entity.chain.push(n);

        let edges = store.outgoing(n);
        // Collect slot-name mappings first (slot_tag → name_string)
        let mut tag_to_name: HashMap<u16, String> = HashMap::new();
        for e in &edges {
            if e.relation == prototype_rel::HAS_SLOT && (e.slot & 0x8000) != 0 {
                // name-binding edge
                let tag = e.slot & 0x7FFF;
                if let Some(atom) = store.get(e.to) {
                    if let Ok(s) = std::str::from_utf8(&atom.data) {
                        tag_to_name.insert(tag, s.to_string());
                    }
                }
            }
        }
        // Now also collect name atoms from HAS_PART semantics — name atoms are
        // connected via HAS_SLOT without the high-bit reserved (see add_part)
        for e in &edges {
            if e.relation == prototype_rel::HAS_SLOT && (e.slot & 0x8000) == 0 {
                if let Some(atom) = store.get(e.to) {
                    if atom.kind == AtomKind::Text {
                        if let Ok(s) = std::str::from_utf8(&atom.data) {
                            tag_to_name.entry(e.slot).or_insert_with(|| s.to_string());
                        }
                    }
                }
            }
        }

        // Walk parts — only insert if not already present (child wins)
        for e in &edges {
            if e.relation == rel::HAS_PART {
                let name = tag_to_name.get(&e.slot)
                    .cloned()
                    .unwrap_or_else(|| format!("part_{}", e.slot));
                entity.parts.entry(name.clone()).or_insert_with(|| {
                    entity.provenance.insert(name.clone(), n);
                    e.to
                });
            }
        }

        // Walk slots (values, not parts) — same child-wins rule
        for e in &edges {
            if e.relation == prototype_rel::HAS_SLOT && (e.slot & 0x8000) == 0 {
                if let Some(target) = store.get(e.to) {
                    // Skip the name binding itself
                    if target.kind == AtomKind::Text { continue; }
                }
                let name = tag_to_name.get(&e.slot)
                    .cloned()
                    .unwrap_or_else(|| format!("slot_{}", e.slot));
                entity.slots.entry(name.clone()).or_insert_with(|| {
                    entity.provenance.insert(name.clone(), n);
                    e.to
                });
            }
        }

        // Follow PROTOTYPE_OF to parent
        current = edges.iter()
            .find(|e| e.relation == prototype_rel::PROTOTYPE_OF)
            .map(|e| e.to);
    }

    entity
}

/// Return the chain of prototypes from leaf to root.
pub fn inheritance_chain(store: &AtomStore, id: AtomId) -> Vec<AtomId> {
    let mut chain = vec![id];
    let mut current = id;
    let mut safety = 32;
    while safety > 0 {
        safety -= 1;
        let edges = store.outgoing(current);
        let parent = edges.iter()
            .find(|e| e.relation == prototype_rel::PROTOTYPE_OF)
            .map(|e| e.to);
        match parent {
            Some(p) => { chain.push(p); current = p; }
            None => break,
        }
    }
    chain
}

/// Test if `a` inherits from `b` (transitively).
pub fn is_a(store: &AtomStore, a: AtomId, b: AtomId) -> bool {
    inheritance_chain(store, a).contains(&b)
}

// ────────────────────────────────────────────────────────────────
// Unused edge type silenced
// ────────────────────────────────────────────────────────────────
#[allow(dead_code)]
fn _unused(_: AtomEdge) {}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Classic example: dog hierarchy.
    /// Mammal -> Quadruped -> Dog -> Poodle -> MiniaturePoodle -> Rex
    fn build_dog_hierarchy() -> (AtomStore, HashMap<String, AtomId>) {
        let mut store = AtomStore::new();
        let mut ids = HashMap::new();

        // Part atoms (stored once, reused everywhere)
        let torso = store.put(AtomKind::Concept, b"torso-generic".to_vec());
        let heart = store.put(AtomKind::Concept, b"heart-mammal".to_vec());
        let leg_quadruped = store.put(AtomKind::Concept, b"leg-quadruped".to_vec());
        let tail_dog = store.put(AtomKind::Concept, b"tail-dog-generic".to_vec());
        let head_poodle = store.put(AtomKind::Concept, b"head-poodle".to_vec());
        let coat_curly = store.put(AtomKind::Concept, b"coat-curly".to_vec());
        let size_mini = store.put(AtomKind::Concept, b"size-miniature".to_vec());
        let color_black = store.put(AtomKind::Concept, b"color-black".to_vec());

        ids.insert("torso".to_string(), torso);
        ids.insert("heart".to_string(), heart);
        ids.insert("leg".to_string(), leg_quadruped);

        // Mammal prototype
        let mammal = Prototype::create(&mut store, "Mammal", None)
            .add_part("torso", torso)
            .add_part("heart", heart)
            .id();
        ids.insert("Mammal".to_string(), mammal);

        // Quadruped (inherits Mammal, adds 4 legs)
        let quadruped = Prototype::create(&mut store, "Quadruped", Some(mammal))
            .add_part("leg_fl", leg_quadruped)
            .add_part("leg_fr", leg_quadruped)
            .add_part("leg_rl", leg_quadruped)
            .add_part("leg_rr", leg_quadruped)
            .id();
        ids.insert("Quadruped".to_string(), quadruped);

        // Dog (inherits Quadruped, adds tail)
        let dog = Prototype::create(&mut store, "Dog", Some(quadruped))
            .add_part("tail", tail_dog)
            .id();
        ids.insert("Dog".to_string(), dog);

        // Poodle (inherits Dog, adds specific head + curly coat)
        let poodle = Prototype::create(&mut store, "Poodle", Some(dog))
            .add_part("head", head_poodle)
            .add_part("coat", coat_curly)
            .id();
        ids.insert("Poodle".to_string(), poodle);

        // MiniaturePoodle (inherits Poodle, adds size)
        let mini_poodle = Prototype::create(&mut store, "MiniaturePoodle", Some(poodle))
            .add_slot("size", size_mini)
            .id();
        ids.insert("MiniaturePoodle".to_string(), mini_poodle);

        // Rex — an individual (inherits MiniaturePoodle, adds color)
        let rex = Prototype::create(&mut store, "Rex", Some(mini_poodle))
            .add_slot("color", color_black)
            .id();
        ids.insert("Rex".to_string(), rex);

        (store, ids)
    }

    #[test]
    fn inheritance_chain_walks_correctly() {
        let (store, ids) = build_dog_hierarchy();
        let chain = inheritance_chain(&store, ids["Rex"]);
        // Rex, MiniaturePoodle, Poodle, Dog, Quadruped, Mammal
        assert_eq!(chain.len(), 6);
        assert_eq!(chain[0], ids["Rex"]);
        assert_eq!(chain[5], ids["Mammal"]);
    }

    #[test]
    fn is_a_works_transitively() {
        let (store, ids) = build_dog_hierarchy();
        assert!(is_a(&store, ids["Rex"], ids["Mammal"]));
        assert!(is_a(&store, ids["Rex"], ids["Dog"]));
        assert!(is_a(&store, ids["Poodle"], ids["Quadruped"]));
        // Sanity: not inverse
        assert!(!is_a(&store, ids["Mammal"], ids["Rex"]));
        assert!(!is_a(&store, ids["Dog"], ids["Poodle"]));
    }

    #[test]
    fn resolve_rex_has_all_inherited_parts() {
        let (store, ids) = build_dog_hierarchy();
        let rex = resolve(&store, ids["Rex"]);

        // Rex must have: torso+heart (from Mammal), 4 legs (from Quadruped),
        // tail (from Dog), head+coat (from Poodle)
        assert!(rex.parts.contains_key("torso"));
        assert!(rex.parts.contains_key("heart"));
        assert!(rex.parts.contains_key("leg_fl"));
        assert!(rex.parts.contains_key("leg_fr"));
        assert!(rex.parts.contains_key("leg_rl"));
        assert!(rex.parts.contains_key("leg_rr"));
        assert!(rex.parts.contains_key("tail"));
        assert!(rex.parts.contains_key("head"));
        assert!(rex.parts.contains_key("coat"));
    }

    #[test]
    fn resolve_rex_has_all_slots_with_overrides() {
        let (store, ids) = build_dog_hierarchy();
        let rex = resolve(&store, ids["Rex"]);

        // Rex-specific: color
        // Inherited from MiniaturePoodle: size
        assert!(rex.slots.contains_key("color"));
        assert!(rex.slots.contains_key("size"));
    }

    #[test]
    fn provenance_shows_where_each_slot_came_from() {
        let (store, ids) = build_dog_hierarchy();
        let rex = resolve(&store, ids["Rex"]);

        // torso came from Mammal
        assert_eq!(rex.provenance["torso"], ids["Mammal"]);
        // legs came from Quadruped
        assert_eq!(rex.provenance["leg_fl"], ids["Quadruped"]);
        // tail came from Dog
        assert_eq!(rex.provenance["tail"], ids["Dog"]);
        // head from Poodle
        assert_eq!(rex.provenance["head"], ids["Poodle"]);
        // size from MiniaturePoodle
        assert_eq!(rex.provenance["size"], ids["MiniaturePoodle"]);
        // color only at Rex level
        assert_eq!(rex.provenance["color"], ids["Rex"]);
    }

    #[test]
    fn child_overrides_parent_on_same_slot() {
        let mut store = AtomStore::new();
        let red = store.put(AtomKind::Concept, b"red".to_vec());
        let blue = store.put(AtomKind::Concept, b"blue".to_vec());

        let car_base = Prototype::create(&mut store, "CarBase", None)
            .add_slot("color", red)
            .id();
        let special = Prototype::create(&mut store, "SpecialEdition", Some(car_base))
            .add_slot("color", blue)
            .id();

        let resolved = resolve(&store, special);
        assert_eq!(resolved.slots["color"], blue, "child color must win");
        assert_eq!(resolved.provenance["color"], special);
    }

    #[test]
    fn single_wheel_serves_many_cars_via_parts() {
        // This is Idan's original reuse example formalized via prototype.
        let mut store = AtomStore::new();
        let wheel = store.put(AtomKind::Concept, b"wheel-standard".to_vec());
        let chassis = store.put(AtomKind::Concept, b"chassis-sedan".to_vec());

        let car_template = Prototype::create(&mut store, "Car", None)
            .add_part("chassis", chassis)
            .add_part("wheel_fl", wheel)
            .add_part("wheel_fr", wheel)
            .add_part("wheel_rl", wheel)
            .add_part("wheel_rr", wheel)
            .id();

        let c1 = Prototype::create(&mut store, "Car-A", Some(car_template)).id();
        let c2 = Prototype::create(&mut store, "Car-B", Some(car_template)).id();
        let c3 = Prototype::create(&mut store, "Car-C", Some(car_template)).id();

        // All three cars see 4 wheels — but wheel atom exists ONCE.
        for car in [c1, c2, c3] {
            let r = resolve(&store, car);
            assert_eq!(r.parts["wheel_fl"], wheel);
            assert_eq!(r.parts["wheel_fr"], wheel);
        }

        // The wheel atom has high refcount (1 put + 4 links from template =  5)
        let wheel_atom = store.get(wheel).unwrap();
        assert!(wheel_atom.refcount >= 5, "wheel should be reused, got {}", wheel_atom.refcount);
    }

    #[test]
    fn chain_is_bounded_safely() {
        // Even if somehow there's a cycle (shouldn't happen, but safety net)
        // the walker should terminate via the safety counter.
        let mut store = AtomStore::new();
        let a = store.put(AtomKind::Composition, b"A".to_vec());
        // Create a self-loop via PROTOTYPE_OF
        store.link(a, a, prototype_rel::PROTOTYPE_OF, 100, 0);
        let r = resolve(&store, a);
        // Should not hang; should produce some bounded output.
        assert!(r.chain.len() <= 32);
    }

    #[test]
    fn slot_hash_is_deterministic() {
        let h1 = slot_hash("head");
        let h2 = slot_hash("head");
        assert_eq!(h1, h2);
        // Different names should (usually) differ
        assert_ne!(slot_hash("head"), slot_hash("tail"));
    }
}
