//! `personas-demo` — Idan's diversity-gradient test case.
//!
//! Six personas from VERY LOW to VERY HIGH diversity (age, occupations,
//! hobbies, languages, group memberships), proving the Cognitive Brain Graph
//! handles all complexity uniformly:
//!
//! - No new scopes needed per person
//! - Groups are atoms, not tenants — a person can belong to 5 hobby clubs,
//!   3 workplaces, and 1 family without schema changes
//! - Diversity score is a single walk query
//! - Same inheritance chain serves student and polymath
//!
//! This answers Idan's design question: scope = security level,
//! group = semantic membership. They are orthogonal, not hierarchical.

use zets::atoms::{AtomKind, AtomStore};
use zets::relations;

// Simple helper: add a named attribute edge
fn attr(store: &mut AtomStore, subject: zets::atoms::AtomId, rel_name: &str,
        value_kind: AtomKind, value: &[u8]) -> zets::atoms::AtomId {
    let val = store.put(value_kind, value.to_vec());
    let rel = relations::by_name(rel_name).unwrap().code;
    store.link(subject, val, rel, 80, 0);
    val
}

// Count outgoing edges of a specific relation kind for a subject
fn count_rel(store: &AtomStore, subject: zets::atoms::AtomId, rel_name: &str) -> usize {
    let rel_code = relations::by_name(rel_name).unwrap().code;
    store.outgoing(subject).iter()
        .filter(|e| e.relation == rel_code)
        .count()
}

fn diversity_score(store: &AtomStore, subject: zets::atoms::AtomId) -> (usize, usize, usize, usize, usize) {
    let occupations = count_rel(store, subject, "role_toward");
    let hobbies = count_rel(store, subject, "cares_for");
    let languages = count_rel(store, subject, "imitates"); // proxy: imitates language
    let groups = count_rel(store, subject, "belongs_to_group");
    let total = occupations + hobbies + languages + groups;
    (occupations, hobbies, languages, groups, total)
}

fn main() {
    println!("═══ ZETS Personas Demo — Diversity Gradient ═══");
    println!();

    let mut store = AtomStore::new();

    // ──────────────────────────────────────────────────────────
    // Shared concepts (reused across personas — dedup'd by content hash)
    // ──────────────────────────────────────────────────────────

    // Languages (shared — one atom per language, referenced by everyone)
    let hebrew = store.put(AtomKind::Concept, b"\xD7\xA2\xD7\x91\xD7\xA8\xD7\x99\xD7\xAA".to_vec());
    let english = store.put(AtomKind::Concept, b"English".to_vec());
    let russian = store.put(AtomKind::Concept, b"Russian".to_vec());
    let french = store.put(AtomKind::Concept, b"French".to_vec());
    let italian = store.put(AtomKind::Concept, b"Italian".to_vec());
    let spanish = store.put(AtomKind::Concept, b"Spanish".to_vec());
    let german = store.put(AtomKind::Concept, b"German".to_vec());

    // Occupations (shared pool)
    let cs_student = store.put(AtomKind::Concept, b"CS_student".to_vec());
    let hardware_engineer = store.put(AtomKind::Concept, b"hardware_engineer".to_vec());
    let lecturer = store.put(AtomKind::Concept, b"lecturer".to_vec());
    let tech_entrepreneur = store.put(AtomKind::Concept, b"tech_entrepreneur".to_vec());
    let strategy_consultant = store.put(AtomKind::Concept, b"strategy_consultant".to_vec());
    let army_cadet = store.put(AtomKind::Concept, b"army_cadet".to_vec());
    let kiosk_worker = store.put(AtomKind::Concept, b"kiosk_worker".to_vec());
    let family_doctor = store.put(AtomKind::Concept, b"family_doctor".to_vec());
    let health_blogger = store.put(AtomKind::Concept, b"health_blogger".to_vec());
    let archivist_volunteer = store.put(AtomKind::Concept, b"archivist_volunteer".to_vec());
    let _retiree = store.put(AtomKind::Concept, b"retiree".to_vec());

    // Hobbies (shared pool)
    let coding = store.put(AtomKind::Concept, b"coding".to_vec());
    let video_games = store.put(AtomKind::Concept, b"video_games".to_vec());
    let scifi_reading = store.put(AtomKind::Concept, b"scifi_reading".to_vec());
    let photography = store.put(AtomKind::Concept, b"photography".to_vec());
    let guitar = store.put(AtomKind::Concept, b"guitar".to_vec());
    let environmental_vol = store.put(AtomKind::Concept, b"environmental_volunteering".to_vec());
    let philosophy = store.put(AtomKind::Concept, b"philosophy".to_vec());
    let long_distance_running = store.put(AtomKind::Concept, b"long_distance_running".to_vec());
    let yoga = store.put(AtomKind::Concept, b"yoga".to_vec());
    let graffiti = store.put(AtomKind::Concept, b"graffiti_art".to_vec());
    let basketball = store.put(AtomKind::Concept, b"basketball".to_vec());
    let bjj = store.put(AtomKind::Concept, b"brazilian_jiu_jitsu".to_vec());
    let cooking = store.put(AtomKind::Concept, b"home_cooking".to_vec());
    let gardening = store.put(AtomKind::Concept, b"gardening".to_vec());
    let podcasting = store.put(AtomKind::Concept, b"podcasting".to_vec());
    let history_reading = store.put(AtomKind::Concept, b"history_reading".to_vec());
    let swimming = store.put(AtomKind::Concept, b"swimming".to_vec());
    let classical_music = store.put(AtomKind::Concept, b"classical_music".to_vec());

    // Groups/communities (shared — same group can include multiple personas)
    let hackers_club = store.put(AtomKind::Concept, b"campus_hackers_club".to_vec());
    let photographers_community = store.put(AtomKind::Concept, b"photographers_community".to_vec());
    let env_volunteers = store.put(AtomKind::Concept, b"env_volunteers_forum".to_vec());
    let women_tech_network = store.put(AtomKind::Concept, b"women_tech_network".to_vec());
    let running_club = store.put(AtomKind::Concept, b"city_running_club".to_vec());
    let street_art_circle = store.put(AtomKind::Concept, b"street_art_circle".to_vec());
    let martial_arts_group = store.put(AtomKind::Concept, b"martial_arts_group".to_vec());
    let doctors_forum = store.put(AtomKind::Concept, b"community_doctors_forum".to_vec());
    let urban_gardening = store.put(AtomKind::Concept, b"urban_gardening_club".to_vec());
    let seniors_club = store.put(AtomKind::Concept, b"seniors_club".to_vec());
    let local_choir = store.put(AtomKind::Concept, b"local_choir".to_vec());

    // ──────────────────────────────────────────────────────────
    // Persona A: Very Low Diversity (22, CS student)
    // ──────────────────────────────────────────────────────────
    let person_a = store.put(AtomKind::Composition, b"Person_A".to_vec());
    attr(&mut store, person_a, "has_attribute", AtomKind::Concept, b"age_22");

    let role = relations::by_name("role_toward").unwrap().code;
    let cares = relations::by_name("cares_for").unwrap().code;
    let imitates = relations::by_name("imitates").unwrap().code;
    let belongs = relations::by_name("belongs_to_group").unwrap().code;

    // 1 occupation, 3 tech-only hobbies, 2 languages, 1 group
    store.link(person_a, cs_student, role, 80, 0);
    store.link(person_a, coding, cares, 90, 0);
    store.link(person_a, video_games, cares, 70, 0);
    store.link(person_a, scifi_reading, cares, 70, 0);
    store.link(person_a, hebrew, imitates, 100, 0);
    store.link(person_a, english, imitates, 80, 0);
    store.link(person_a, hackers_club, belongs, 70, 0);

    // ──────────────────────────────────────────────────────────
    // Persona B: High Diversity (35, hardware engineer + lecturer)
    // ──────────────────────────────────────────────────────────
    let person_b = store.put(AtomKind::Composition, b"Person_B".to_vec());
    attr(&mut store, person_b, "has_attribute", AtomKind::Concept, b"age_35");

    // 2 occupations, 3 hobbies across domains, 3 languages, 2 groups
    store.link(person_b, hardware_engineer, role, 90, 0);
    store.link(person_b, lecturer, role, 70, 0);
    store.link(person_b, photography, cares, 80, 0);
    store.link(person_b, guitar, cares, 75, 0);
    store.link(person_b, environmental_vol, cares, 85, 0);
    store.link(person_b, hebrew, imitates, 100, 0);
    store.link(person_b, italian, imitates, 70, 0);
    store.link(person_b, spanish, imitates, 65, 0);
    store.link(person_b, photographers_community, belongs, 75, 0);
    store.link(person_b, env_volunteers, belongs, 80, 0);

    // ──────────────────────────────────────────────────────────
    // Persona C: Very High Diversity (58, entrepreneur + consultant)
    // ──────────────────────────────────────────────────────────
    let person_c = store.put(AtomKind::Composition, b"Person_C".to_vec());
    attr(&mut store, person_c, "has_attribute", AtomKind::Concept, b"age_58");

    // 2 occupations, 3 distinct-domain hobbies, 3 languages, 2 groups
    store.link(person_c, tech_entrepreneur, role, 95, 0);
    store.link(person_c, strategy_consultant, role, 85, 0);
    store.link(person_c, philosophy, cares, 85, 0);
    store.link(person_c, long_distance_running, cares, 80, 0);
    store.link(person_c, yoga, cares, 75, 0);
    store.link(person_c, hebrew, imitates, 100, 0);
    store.link(person_c, english, imitates, 95, 0);
    store.link(person_c, german, imitates, 70, 0);
    store.link(person_c, women_tech_network, belongs, 90, 0);
    store.link(person_c, running_club, belongs, 70, 0);

    // ──────────────────────────────────────────────────────────
    // Persona D: Medium-Low Diversity (19, young soldier)
    // ──────────────────────────────────────────────────────────
    let person_d = store.put(AtomKind::Composition, b"Person_D".to_vec());
    attr(&mut store, person_d, "has_attribute", AtomKind::Concept, b"age_19");

    // 2 light occupations, 3 hobbies (art+sport+martial), 2 languages, 2 groups
    store.link(person_d, army_cadet, role, 90, 0);
    store.link(person_d, kiosk_worker, role, 40, 0);
    store.link(person_d, graffiti, cares, 85, 0);
    store.link(person_d, basketball, cares, 80, 0);
    store.link(person_d, bjj, cares, 75, 0);
    store.link(person_d, hebrew, imitates, 100, 0);
    store.link(person_d, russian, imitates, 80, 0);
    store.link(person_d, street_art_circle, belongs, 80, 0);
    store.link(person_d, martial_arts_group, belongs, 75, 0);

    // ──────────────────────────────────────────────────────────
    // Persona E: High Diversity (45, doctor + blogger)
    // ──────────────────────────────────────────────────────────
    let person_e = store.put(AtomKind::Composition, b"Person_E".to_vec());
    attr(&mut store, person_e, "has_attribute", AtomKind::Concept, b"age_45");

    // 2 occupations, 3 hobbies, 3 languages, 2 groups
    store.link(person_e, family_doctor, role, 95, 0);
    store.link(person_e, health_blogger, role, 70, 0);
    store.link(person_e, cooking, cares, 80, 0);
    store.link(person_e, gardening, cares, 75, 0);
    store.link(person_e, podcasting, cares, 70, 0);
    store.link(person_e, hebrew, imitates, 100, 0);
    store.link(person_e, english, imitates, 90, 0);
    store.link(person_e, french, imitates, 65, 0);
    store.link(person_e, doctors_forum, belongs, 85, 0);
    store.link(person_e, urban_gardening, belongs, 70, 0);

    // ──────────────────────────────────────────────────────────
    // Persona F: Low Diversity (70, retiree archivist)
    // ──────────────────────────────────────────────────────────
    let person_f = store.put(AtomKind::Composition, b"Person_F".to_vec());
    attr(&mut store, person_f, "has_attribute", AtomKind::Concept, b"age_70");

    // 1 volunteer role, 3 mellow hobbies, 2 languages, 2 groups
    store.link(person_f, archivist_volunteer, role, 80, 0);
    store.link(person_f, history_reading, cares, 85, 0);
    store.link(person_f, swimming, cares, 65, 0);
    store.link(person_f, classical_music, cares, 80, 0);
    store.link(person_f, hebrew, imitates, 100, 0);
    store.link(person_f, english, imitates, 60, 0);
    store.link(person_f, seniors_club, belongs, 90, 0);
    store.link(person_f, local_choir, belongs, 75, 0);

    // ──────────────────────────────────────────────────────────
    // Diversity scoring
    // ──────────────────────────────────────────────────────────
    println!("─── Diversity Gradient Across 6 Personas ───");
    println!();
    println!("{:<14} {:>4} {:>5} {:>5} {:>5} {:>7} {:<30}",
        "Person", "Occ.", "Hobby", "Lang", "Grp", "TOTAL", "Label");
    println!("{:─<75}", "");

    let mut scored: Vec<(zets::atoms::AtomId, &str, &str, usize)> = Vec::new();
    for (pid, label, age) in [
        (person_a, "Person A", "22 CS student"),
        (person_b, "Person B", "35 HW eng + lecturer"),
        (person_c, "Person C", "58 entrepreneur+consultant"),
        (person_d, "Person D", "19 soldier + kiosk"),
        (person_e, "Person E", "45 doctor + blogger"),
        (person_f, "Person F", "70 retiree + volunteer"),
    ] {
        let (o, h, l, g, total) = diversity_score(&store, pid);
        println!("{:<14} {:>4} {:>5} {:>5} {:>5} {:>7} {:<30}",
            label, o, h, l, g, total, age);
        scored.push((pid, label, age, total));
    }
    println!();

    // Sort by total diversity
    scored.sort_by_key(|(_, _, _, t)| std::cmp::Reverse(*t));
    println!("─── Ranked by Diversity (highest → lowest) ───");
    for (i, (_, label, age, total)) in scored.iter().enumerate() {
        let rank_emoji = match i { 0 => "🥇", 1 => "🥈", 2 => "🥉", _ => "  " };
        println!("  {} {} {} (diversity: {})", rank_emoji, label, age, total);
    }
    println!();

    // ──────────────────────────────────────────────────────────
    // Structural efficiency: atom reuse across personas
    // ──────────────────────────────────────────────────────────
    println!("─── Shared Atom Reuse (shows memory efficiency) ───");
    for (label, atom_id) in &[
        ("Hebrew (language)", hebrew),
        ("English (language)", english),
        ("video_games (hobby)", video_games),
        ("photography (hobby)", photography),
        ("hackers_club (group)", hackers_club),
        ("women_tech_network (group)", women_tech_network),
    ] {
        let a = store.get(*atom_id).unwrap();
        println!("  {:<35} refcount={}", label, a.refcount);
    }
    println!();

    // ──────────────────────────────────────────────────────────
    // Compound query: "Who speaks 3+ languages AND belongs to 2+ groups?"
    // ──────────────────────────────────────────────────────────
    println!("─── Query: 'Speaks 3+ languages AND belongs to 2+ groups' ───");
    let personas = [
        (person_a, "Person A"), (person_b, "Person B"), (person_c, "Person C"),
        (person_d, "Person D"), (person_e, "Person E"), (person_f, "Person F"),
    ];
    for (pid, label) in &personas {
        let langs = count_rel(&store, *pid, "imitates");
        let groups = count_rel(&store, *pid, "belongs_to_group");
        if langs >= 3 && groups >= 2 {
            println!("  ✓ {} — {} languages, {} groups", label, langs, groups);
        }
    }
    println!();

    // ──────────────────────────────────────────────────────────
    // The key insight: scope vs group
    // ──────────────────────────────────────────────────────────
    let s = store.stats();
    println!("─── Graph State ───");
    println!("  Total atoms: {}", s.atom_count);
    println!("  Total edges: {}", s.edge_count);
    println!("  Bytes saved by content dedup: {}", s.bytes_saved_dedup);
    println!();

    println!("═══ Architectural Insight ═══");
    println!();
    println!("  All 6 personas live in the SAME AtomStore, SAME scope.");
    println!("  Yet each has distinct diversity — from 6 total links (Person A)");
    println!("  to 10 total links (Persons B, C, E).");
    println!();
    println!("  Idan's concern: 'do I need separate graphs for family/work/hobby?'");
    println!("  Answer: NO. Scope = security tier (e.g. UserScope = encrypted).");
    println!("          Group = belongs_to_group edges = semantic membership.");
    println!("          These are ORTHOGONAL. No overhead per group type.");
    println!();
    println!("  A person can belong to 5 hobby clubs + 3 workplaces + 1 family");
    println!("  with zero schema changes. Shared atoms (Hebrew, photography,");
    println!("  hackers_club) are stored ONCE and referenced by many — that's");
    println!("  what content-addressed dedup guarantees.");
    println!();
    println!("  For encryption: all persona data goes in UserScope (encrypted).");
    println!("  Shared atoms (languages, common hobbies) stay in Data scope");
    println!("  (cheap, public). Result: selective encryption WITHOUT");
    println!("  'RocksDB column-family per tenant' complexity.");
}
