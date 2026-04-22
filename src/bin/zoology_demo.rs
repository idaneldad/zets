//! `zoology-demo` — hierarchical decomposition of creatures via prototypes.
//!
//! Taxonomy:
//!   Mammal
//!   └─ Quadruped (4 legs, spine)
//!      ├─ Canine (Dog family)
//!      │  ├─ Poodle        (curly coat, specific head)
//!      │  │  ├─ ToyPoodle    (size:toy)
//!      │  │  │  └─ Rex (black, owner=idan)  ← individual
//!      │  │  └─ StandardPoodle (size:standard)
//!      │  └─ Labrador      (short coat, broad head)
//!      │     └─ Buddy (yellow, owner=ben)    ← individual
//!      ├─ Bovine (Cow family)
//!      │  └─ Holstein      (black+white coat, large size)
//!      └─ Elephantidae (Elephant family)
//!         └─ AfricanElephant (big ears, tusks)
//!
//! Goal: demonstrate how ONE storage structure handles all these creatures,
//! with inherited parts and variation deltas.

use zets::atoms::{AtomKind, AtomStore};
use zets::prototype::{inheritance_chain, is_a, resolve, Prototype};

fn main() {
    println!("═══ ZETS Zoology Demo — Prototype Hierarchy ═══");
    println!();

    let mut store = AtomStore::new();

    // ──────────────────────────────────────────────────────────
    // Part atoms — stored ONCE, referenced many times
    // ──────────────────────────────────────────────────────────
    let torso_generic = store.put(AtomKind::Concept, b"torso-generic".to_vec());
    let heart_mammal = store.put(AtomKind::Concept, b"heart-mammal".to_vec());
    let lungs_mammal = store.put(AtomKind::Concept, b"lungs-mammal".to_vec());
    let spine_horizontal = store.put(AtomKind::Concept, b"spine-horizontal".to_vec());
    let leg_quadruped = store.put(AtomKind::Concept, b"leg-quadruped".to_vec());
    let paw_canine = store.put(AtomKind::Concept, b"paw-canine".to_vec());
    let hoof_bovine = store.put(AtomKind::Concept, b"hoof-bovine".to_vec());
    let foot_elephant = store.put(AtomKind::Concept, b"foot-elephant-round".to_vec());
    let tail_canine_generic = store.put(AtomKind::Concept, b"tail-canine-generic".to_vec());
    let tail_bovine = store.put(AtomKind::Concept, b"tail-bovine-with-tuft".to_vec());
    let trunk_elephant = store.put(AtomKind::Concept, b"trunk-elephant".to_vec());
    let head_dog_short = store.put(AtomKind::Concept, b"head-dog-short-muzzle".to_vec());
    let head_dog_broad = store.put(AtomKind::Concept, b"head-dog-broad-lab".to_vec());
    let head_bovine = store.put(AtomKind::Concept, b"head-bovine-horned".to_vec());
    let head_elephant = store.put(AtomKind::Concept, b"head-elephant-with-ears".to_vec());
    let coat_curly = store.put(AtomKind::Concept, b"coat-curly".to_vec());
    let coat_short = store.put(AtomKind::Concept, b"coat-short".to_vec());
    let coat_holstein = store.put(AtomKind::Concept, b"coat-black-and-white".to_vec());
    let skin_elephant = store.put(AtomKind::Concept, b"skin-elephant-wrinkled".to_vec());
    let tusks_ivory = store.put(AtomKind::Concept, b"tusks-ivory".to_vec());
    let size_toy = store.put(AtomKind::Concept, b"size-toy".to_vec());
    let size_standard = store.put(AtomKind::Concept, b"size-standard".to_vec());
    let size_large = store.put(AtomKind::Concept, b"size-large".to_vec());
    let size_huge = store.put(AtomKind::Concept, b"size-huge".to_vec());
    let color_black = store.put(AtomKind::Concept, b"color-black".to_vec());
    let color_yellow = store.put(AtomKind::Concept, b"color-yellow".to_vec());

    // ──────────────────────────────────────────────────────────
    // Build the hierarchy
    // ──────────────────────────────────────────────────────────

    // Mammal
    let mammal = Prototype::create(&mut store, "Mammal", None)
        .add_part("torso", torso_generic)
        .add_part("heart", heart_mammal)
        .add_part("lungs", lungs_mammal)
        .id();

    // Quadruped
    let quadruped = Prototype::create(&mut store, "Quadruped", Some(mammal))
        .add_part("spine", spine_horizontal)
        .add_part("leg_fl", leg_quadruped)
        .add_part("leg_fr", leg_quadruped)
        .add_part("leg_rl", leg_quadruped)
        .add_part("leg_rr", leg_quadruped)
        .id();

    // Canine family
    let canine = Prototype::create(&mut store, "Canine", Some(quadruped))
        .add_part("paw_fl", paw_canine)
        .add_part("paw_fr", paw_canine)
        .add_part("paw_rl", paw_canine)
        .add_part("paw_rr", paw_canine)
        .add_part("tail", tail_canine_generic)
        .id();

    // Poodle (curly coat, specific head)
    let poodle = Prototype::create(&mut store, "Poodle", Some(canine))
        .add_part("head", head_dog_short)
        .add_part("coat", coat_curly)
        .id();

    // Toy Poodle (small size variant)
    let toy_poodle = Prototype::create(&mut store, "ToyPoodle", Some(poodle))
        .add_slot("size", size_toy)
        .id();

    // Standard Poodle (standard size)
    let standard_poodle = Prototype::create(&mut store, "StandardPoodle", Some(poodle))
        .add_slot("size", size_standard)
        .id();

    // Labrador (short coat, broad head)
    let labrador = Prototype::create(&mut store, "Labrador", Some(canine))
        .add_part("head", head_dog_broad)
        .add_part("coat", coat_short)
        .add_slot("size", size_standard)
        .id();

    // Bovine family
    let bovine = Prototype::create(&mut store, "Bovine", Some(quadruped))
        .add_part("hoof_fl", hoof_bovine)
        .add_part("hoof_fr", hoof_bovine)
        .add_part("hoof_rl", hoof_bovine)
        .add_part("hoof_rr", hoof_bovine)
        .add_part("tail", tail_bovine)
        .add_part("head", head_bovine)
        .id();

    // Holstein cow
    let holstein = Prototype::create(&mut store, "Holstein", Some(bovine))
        .add_part("coat", coat_holstein)
        .add_slot("size", size_large)
        .id();

    // Elephant family
    let elephantidae = Prototype::create(&mut store, "Elephantidae", Some(quadruped))
        .add_part("foot_fl", foot_elephant)
        .add_part("foot_fr", foot_elephant)
        .add_part("foot_rl", foot_elephant)
        .add_part("foot_rr", foot_elephant)
        .add_part("trunk", trunk_elephant)
        .add_part("head", head_elephant)
        .add_part("skin", skin_elephant)
        .id();

    // African Elephant (adds tusks, huge size)
    let african_elephant = Prototype::create(&mut store, "AfricanElephant", Some(elephantidae))
        .add_part("tusks", tusks_ivory)
        .add_slot("size", size_huge)
        .id();

    // ──────────────────────────────────────────────────────────
    // Individuals
    // ──────────────────────────────────────────────────────────
    let rex = Prototype::create(&mut store, "Rex", Some(toy_poodle))
        .add_slot("color", color_black)
        .id();

    let buddy = Prototype::create(&mut store, "Buddy", Some(labrador))
        .add_slot("color", color_yellow)
        .id();

    // ──────────────────────────────────────────────────────────
    // Query: describe Rex fully
    // ──────────────────────────────────────────────────────────
    println!("─── Rex (a toy poodle) ───");
    show_entity(&store, rex, "Rex");
    println!();

    println!("─── Buddy (a labrador) ───");
    show_entity(&store, buddy, "Buddy");
    println!();

    println!("─── Holstein (a cow) ───");
    show_entity(&store, holstein, "Holstein");
    println!();

    println!("─── AfricanElephant ───");
    show_entity(&store, african_elephant, "AfricanElephant");
    println!();

    // ──────────────────────────────────────────────────────────
    // is_a queries
    // ──────────────────────────────────────────────────────────
    println!("─── is_a queries (transitive) ───");
    let entities = [
        ("Rex", rex), ("Buddy", buddy),
        ("Holstein", holstein), ("AfricanElephant", african_elephant),
    ];
    let categories = [
        ("Canine", canine), ("Quadruped", quadruped), ("Mammal", mammal),
        ("Bovine", bovine), ("Elephantidae", elephantidae),
    ];
    print!("                 ");
    for (cname, _) in &categories { print!(" {:<12}", cname); }
    println!();
    for (ename, eid) in &entities {
        print!("  {:<15}", ename);
        for (_, cid) in &categories {
            let yes = is_a(&store, *eid, *cid);
            print!(" {:<12}", if yes { "✓" } else { "·" });
        }
        println!();
    }
    println!();

    // ──────────────────────────────────────────────────────────
    // Storage efficiency
    // ──────────────────────────────────────────────────────────
    let stats = store.stats();
    println!("─── Storage efficiency ───");
    println!("  Total atoms: {}", stats.atom_count);
    println!("  Total edges: {}", stats.edge_count);
    println!("  Raw bytes stored: {}", stats.total_bytes);
    println!("  Bytes saved by dedup: {}", stats.bytes_saved_dedup);
    println!();

    // Compare naive: if we stored each creature fully (e.g., 10 parts × 30 bytes each)
    // = 300 bytes × 9 creatures = 2,700 bytes. Plus 2 individuals ×.
    let naive_per_creature = 30 * 30; // 30 parts × 30 bytes each
    let creature_count = 11; // 9 prototypes + 2 individuals
    let naive_total = naive_per_creature * creature_count;
    println!("  Naive (store full creature each time): ~{} bytes", naive_total);
    println!("  Compositional: {} bytes ({:.1}% of naive)",
        stats.total_bytes,
        (stats.total_bytes as f64 / naive_total as f64) * 100.0);
    println!();

    // ──────────────────────────────────────────────────────────
    // Reuse demonstration: how many creatures use heart_mammal?
    // ──────────────────────────────────────────────────────────
    println!("─── Reuse counts (refcount > 1 = atom is shared) ───");
    for (label, aid) in &[
        ("heart_mammal", heart_mammal),
        ("spine_horizontal", spine_horizontal),
        ("leg_quadruped", leg_quadruped),
        ("paw_canine", paw_canine),
        ("hoof_bovine", hoof_bovine),
        ("coat_curly", coat_curly),
        ("size_toy", size_toy),
    ] {
        let atom = store.get(*aid).unwrap();
        println!("  {:<20} refcount={}", label, atom.refcount);
    }
    println!();

    println!("═══ Summary ═══");
    println!("  * {} creature types + {} individuals, all via prototype chain",
        9, 2);
    println!("  * Parts like 'heart_mammal' stored ONCE, reused by all mammals");
    println!("  * 'leg_quadruped' reused across dogs, cows, elephants");
    println!("  * Rex inherits from 6 levels of prototype (Rex → ToyPoodle → Poodle → Canine → Quadruped → Mammal)");
    println!("  * Override works: Rex adds 'color=black', everything else inherited");
    println!("  * Same mechanism will work for: cars (chassis+wheels), humans (body+face), songs (motif+variations), video frames (template+delta)");
}

fn show_entity(store: &AtomStore, id: zets::atoms::AtomId, name: &str) {
    let chain = inheritance_chain(store, id);
    let resolved = resolve(store, id);
    print!("  Inheritance chain: ");
    for (i, cid) in chain.iter().enumerate() {
        if i > 0 { print!(" → "); }
        if let Some(atom) = store.get(*cid) {
            if let Ok(n) = std::str::from_utf8(&atom.data) {
                print!("{}", n);
            }
        }
    }
    println!();
    println!("  Parts ({}):", resolved.parts.len());

    let mut part_keys: Vec<_> = resolved.parts.keys().collect();
    part_keys.sort();
    for pname in part_keys {
        let part_atom = store.get(resolved.parts[pname]).unwrap();
        let part_label = std::str::from_utf8(&part_atom.data).unwrap_or("?");
        let from = resolved.provenance.get(pname)
            .and_then(|id| store.get(*id))
            .and_then(|a| std::str::from_utf8(&a.data).ok())
            .unwrap_or("?");
        println!("    {:<12} = {:<35} (from {})", pname, part_label, from);
    }

    println!("  Slots ({}):", resolved.slots.len());
    let mut slot_keys: Vec<_> = resolved.slots.keys().collect();
    slot_keys.sort();
    for sname in slot_keys {
        let slot_atom = store.get(resolved.slots[sname]).unwrap();
        let slot_label = std::str::from_utf8(&slot_atom.data).unwrap_or("?");
        let from = resolved.provenance.get(sname)
            .and_then(|id| store.get(*id))
            .and_then(|a| std::str::from_utf8(&a.data).ok())
            .unwrap_or("?");
        println!("    {:<12} = {:<35} (from {})", sname, slot_label, from);
    }
    let _ = name;
}
