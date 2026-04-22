//! `seed_programming_taxonomy` — build a taxonomy snapshot for programming languages.
//!
//! Based on Idan's research (22.04.2026) of 9 paradigms × 4 execution modes
//! × 5 type systems × 6 purposes, cross-referenced with concrete languages.
//!
//! Outputs: data/baseline/programming_taxonomy_v1.atoms
//!
//! This snapshot is INDEPENDENT of the wiki snapshot. Later phases can bridge
//! them via content_hash registry — but this one stands alone so it's easy to
//! test and extend.
//!
//! Usage: cargo run --release --bin seed_programming_taxonomy

use std::path::Path;
use zets::atoms::{AtomKind, AtomStore};
use zets::atom_persist;
use zets::relations;

fn main() -> std::io::Result<()> {
    let mut store = AtomStore::new();

    let is_a = relations::by_name("is_a").unwrap().code;
    let has_attr = relations::by_name("has_attribute").unwrap().code;
    let used_for = relations::by_name("used_for").unwrap().code;

    println!("═══ Seed Programming Languages Taxonomy ═══");
    println!();

    // ─── Top-level category atoms ──────────────────────────────────────
    let category = store.put(AtomKind::Concept, b"category".to_vec());

    let paradigm = store.put(AtomKind::Concept, b"paradigm".to_vec());
    let execution = store.put(AtomKind::Concept, b"execution_mode".to_vec());
    let type_system = store.put(AtomKind::Concept, b"type_system".to_vec());
    let purpose = store.put(AtomKind::Concept, b"purpose".to_vec());

    store.link(paradigm, category, is_a, 240, 0);
    store.link(execution, category, is_a, 240, 0);
    store.link(type_system, category, is_a, 240, 0);
    store.link(purpose, category, is_a, 240, 0);

    // ─── Paradigms (9) ─────────────────────────────────────────────────
    let paradigms: Vec<(&str, u32)> = vec![
        ("imperative_procedural",   store.put(AtomKind::Concept, b"imperative_procedural".to_vec())),
        ("object_oriented",          store.put(AtomKind::Concept, b"object_oriented".to_vec())),
        ("functional",               store.put(AtomKind::Concept, b"functional".to_vec())),
        ("declarative_logic",        store.put(AtomKind::Concept, b"declarative_logic".to_vec())),
        ("constraint_based",         store.put(AtomKind::Concept, b"constraint_based".to_vec())),
        ("scripting",                store.put(AtomKind::Concept, b"scripting".to_vec())),
        ("vm_bytecode",              store.put(AtomKind::Concept, b"vm_bytecode".to_vec())),
        ("assembly_low_level",       store.put(AtomKind::Concept, b"assembly_low_level".to_vec())),
        ("very_high_level",          store.put(AtomKind::Concept, b"very_high_level".to_vec())),
    ];
    for (_name, atom) in &paradigms {
        store.link(*atom, paradigm, is_a, 240, 0);
    }

    // ─── Execution modes (4) ───────────────────────────────────────────
    let executions: Vec<(&str, u32)> = vec![
        ("interpreted",          store.put(AtomKind::Concept, b"interpreted".to_vec())),
        ("compiled",             store.put(AtomKind::Concept, b"compiled".to_vec())),
        ("bytecode_jit",         store.put(AtomKind::Concept, b"bytecode_jit".to_vec())),
        ("jit_or_aot_optional",  store.put(AtomKind::Concept, b"jit_or_aot_optional".to_vec())),
    ];
    for (_, atom) in &executions {
        store.link(*atom, execution, is_a, 240, 0);
    }

    // ─── Type systems (5) ──────────────────────────────────────────────
    let type_systems: Vec<(&str, u32)> = vec![
        ("static_explicit", store.put(AtomKind::Concept, b"static_explicit".to_vec())),
        ("dynamic",         store.put(AtomKind::Concept, b"dynamic".to_vec())),
        ("type_inferred",   store.put(AtomKind::Concept, b"type_inferred".to_vec())),
        ("trait_based",     store.put(AtomKind::Concept, b"trait_based".to_vec())),
        ("untyped",         store.put(AtomKind::Concept, b"untyped".to_vec())),
    ];
    for (_, atom) in &type_systems {
        store.link(*atom, type_system, is_a, 240, 0);
    }

    // ─── Purposes (6) ──────────────────────────────────────────────────
    let purposes: Vec<(&str, u32)> = vec![
        ("general_purpose",    store.put(AtomKind::Concept, b"general_purpose".to_vec())),
        ("markup",             store.put(AtomKind::Concept, b"markup".to_vec())),
        ("query_language",     store.put(AtomKind::Concept, b"query_language".to_vec())),
        ("hdl",                store.put(AtomKind::Concept, b"hdl".to_vec())),
        ("game_graphics",      store.put(AtomKind::Concept, b"game_graphics".to_vec())),
        ("automation_sysadmin", store.put(AtomKind::Concept, b"automation_sysadmin".to_vec())),
    ];
    for (_, atom) in &purposes {
        store.link(*atom, purpose, is_a, 240, 0);
    }

    // ─── Helper: find atom by name from our vecs ──────────────────────
    let find = |name: &str, v: &[(&str, u32)]| -> u32 {
        v.iter().find(|(n, _)| *n == name).map(|(_, a)| *a)
            .unwrap_or_else(|| panic!("not found: {}", name))
    };

    // ─── Language atoms + edges ────────────────────────────────────────
    // Each language: (name, paradigms, execution, type_system, main_purposes)
    type LangDef<'a> = (&'a str, Vec<&'a str>, &'a str, &'a str, Vec<&'a str>);
    let langs: Vec<LangDef> = vec![
        // General-purpose workhorses
        ("python", vec!["scripting","object_oriented","imperative_procedural","functional"],
         "interpreted", "dynamic", vec!["general_purpose","automation_sysadmin"]),
        ("javascript", vec!["scripting","object_oriented","functional","imperative_procedural"],
         "interpreted", "dynamic", vec!["general_purpose","automation_sysadmin"]),
        ("typescript", vec!["object_oriented","functional","imperative_procedural"],
         "bytecode_jit", "static_explicit", vec!["general_purpose"]),
        ("ruby", vec!["scripting","object_oriented","imperative_procedural"],
         "interpreted", "dynamic", vec!["general_purpose","automation_sysadmin"]),
        ("perl", vec!["scripting","imperative_procedural"],
         "interpreted", "dynamic", vec!["automation_sysadmin","general_purpose"]),
        ("php", vec!["scripting","object_oriented","imperative_procedural"],
         "interpreted", "dynamic", vec!["general_purpose"]),
        ("lua", vec!["scripting","imperative_procedural","functional"],
         "interpreted", "dynamic", vec!["game_graphics","general_purpose"]),
        ("bash", vec!["scripting","imperative_procedural"],
         "interpreted", "untyped", vec!["automation_sysadmin"]),
        ("powershell", vec!["scripting","object_oriented","imperative_procedural"],
         "interpreted", "dynamic", vec!["automation_sysadmin"]),

        // JVM / bytecode family
        ("java", vec!["object_oriented","imperative_procedural","vm_bytecode"],
         "bytecode_jit", "static_explicit", vec!["general_purpose"]),
        ("csharp", vec!["object_oriented","imperative_procedural","functional","vm_bytecode"],
         "bytecode_jit", "static_explicit", vec!["general_purpose"]),
        ("kotlin", vec!["object_oriented","functional","imperative_procedural","vm_bytecode"],
         "bytecode_jit", "type_inferred", vec!["general_purpose"]),
        ("scala", vec!["object_oriented","functional","vm_bytecode"],
         "bytecode_jit", "type_inferred", vec!["general_purpose"]),

        // Systems / compiled native
        ("rust", vec!["imperative_procedural","functional"],
         "compiled", "trait_based", vec!["general_purpose"]),
        ("go", vec!["imperative_procedural","object_oriented"],
         "compiled", "static_explicit", vec!["general_purpose","automation_sysadmin"]),
        ("c", vec!["imperative_procedural"],
         "compiled", "static_explicit", vec!["general_purpose"]),
        ("cpp", vec!["imperative_procedural","object_oriented","functional"],
         "compiled", "static_explicit", vec!["general_purpose","game_graphics"]),
        ("swift", vec!["object_oriented","imperative_procedural","functional"],
         "compiled", "type_inferred", vec!["general_purpose"]),

        // Functional pure
        ("haskell", vec!["functional"],
         "compiled", "trait_based", vec!["general_purpose"]),
        ("ocaml", vec!["functional","object_oriented"],
         "compiled", "type_inferred", vec!["general_purpose"]),
        ("fsharp", vec!["functional","object_oriented","vm_bytecode"],
         "bytecode_jit", "type_inferred", vec!["general_purpose"]),
        ("lisp", vec!["functional","scripting"],
         "interpreted", "dynamic", vec!["general_purpose"]),
        ("clojure", vec!["functional","vm_bytecode"],
         "bytecode_jit", "dynamic", vec!["general_purpose"]),
        ("elixir", vec!["functional"],
         "bytecode_jit", "dynamic", vec!["general_purpose"]),
        ("erlang", vec!["functional"],
         "bytecode_jit", "dynamic", vec!["general_purpose"]),

        // Logic / constraint
        ("prolog", vec!["declarative_logic"],
         "interpreted", "dynamic", vec!["general_purpose"]),
        ("minizinc", vec!["constraint_based"],
         "interpreted", "static_explicit", vec!["general_purpose"]),

        // Specialized
        ("sql", vec!["declarative_logic"],
         "interpreted", "static_explicit", vec!["query_language"]),
        ("html", vec![],
         "interpreted", "untyped", vec!["markup"]),
        ("css", vec![],
         "interpreted", "untyped", vec!["markup"]),
        ("yaml", vec![],
         "interpreted", "untyped", vec!["markup"]),
        ("json", vec![],
         "interpreted", "untyped", vec!["markup"]),
        ("toml", vec![],
         "interpreted", "untyped", vec!["markup"]),
        ("verilog", vec!["imperative_procedural"],
         "compiled", "static_explicit", vec!["hdl"]),
        ("vhdl", vec!["imperative_procedural"],
         "compiled", "static_explicit", vec!["hdl"]),
        ("glsl", vec!["imperative_procedural"],
         "compiled", "static_explicit", vec!["game_graphics"]),
        ("r", vec!["functional","imperative_procedural","very_high_level"],
         "interpreted", "dynamic", vec!["general_purpose"]),
        ("matlab", vec!["imperative_procedural","functional","very_high_level"],
         "interpreted", "dynamic", vec!["general_purpose"]),

        // Assembly
        ("x86_64_assembly", vec!["assembly_low_level"],
         "compiled", "untyped", vec!["general_purpose"]),
        ("arm_assembly", vec!["assembly_low_level"],
         "compiled", "untyped", vec!["general_purpose"]),
    ];

    let mut lang_atoms: Vec<(String, u32)> = Vec::new();
    for (name, paradigm_list, exec_mode, tsys, purposes_list) in &langs {
        let atom = store.put(AtomKind::Concept, name.as_bytes().to_vec());
        lang_atoms.push((name.to_string(), atom));

        // is_a each paradigm
        for pname in paradigm_list {
            let patom = find(pname, &paradigms);
            store.link(atom, patom, is_a, 240, 0);
        }
        // has_attribute execution_mode
        let eatom = find(exec_mode, &executions);
        store.link(atom, eatom, has_attr, 240, 0);
        // has_attribute type_system
        let tatom = find(tsys, &type_systems);
        store.link(atom, tatom, has_attr, 240, 0);
        // used_for purposes
        for pname in purposes_list {
            let patom = find(pname, &purposes);
            store.link(atom, patom, used_for, 240, 0);
        }
    }

    // ─── Use-case atoms (what Idan's second doc was about) ─────────────
    let usecases: Vec<(&str, Vec<&str>)> = vec![
        // (use_case_name, best_languages_for_it)
        ("automation_general",          vec!["python", "ruby"]),
        ("linux_server_admin",          vec!["bash", "python"]),
        ("windows_server_admin",        vec!["powershell", "python"]),
        ("browser_automation",          vec!["python", "javascript"]),
        ("web_scraping",                vec!["python", "javascript"]),
        ("test_automation",             vec!["python", "javascript", "typescript"]),
        ("api_client",                  vec!["python", "go", "typescript"]),
        ("rpa_desktop",                 vec!["python"]),
        ("data_science",                vec!["python", "r", "matlab"]),
        ("systems_programming",         vec!["rust", "c", "cpp", "go"]),
        ("game_development",            vec!["cpp", "csharp", "lua", "rust"]),
        ("mobile_ios",                  vec!["swift"]),
        ("mobile_android",              vec!["kotlin", "java"]),
        ("hardware_description",        vec!["verilog", "vhdl"]),
        ("shader_graphics",             vec!["glsl"]),
        ("database_query",              vec!["sql"]),
    ];

    let use_case_root = store.put(AtomKind::Concept, b"use_case".to_vec());
    for (uc_name, langs_for_uc) in &usecases {
        let uc_atom = store.put(AtomKind::Concept, uc_name.as_bytes().to_vec());
        store.link(uc_atom, use_case_root, is_a, 240, 0);
        for lang in langs_for_uc {
            let la = lang_atoms.iter().find(|(n, _)| n == lang)
                .map(|(_, a)| *a)
                .unwrap_or_else(|| panic!("unknown lang ref: {}", lang));
            // lang used_for use_case
            store.link(la, uc_atom, used_for, 230, 0);
        }
    }

    // ─── Save + stats ──────────────────────────────────────────────────
    let out = Path::new("data/baseline/programming_taxonomy_v1.atoms");
    atom_persist::save_to_file(&store, out).map_err(|e|
        std::io::Error::new(std::io::ErrorKind::Other, format!("save: {:?}", e)))?;

    let stats = store.stats();
    println!("  saved: {}", out.display());
    println!("  atoms: {}", stats.atom_count);
    println!("  edges: {}", stats.edge_count);
    println!();
    println!("  categories:");
    println!("    paradigms       : {}", paradigms.len());
    println!("    execution_modes : {}", executions.len());
    println!("    type_systems    : {}", type_systems.len());
    println!("    purposes        : {}", purposes.len());
    println!("    languages       : {}", lang_atoms.len());
    println!("    use_cases       : {}", usecases.len());
    println!();
    println!("  size on disk: {} bytes",
             std::fs::metadata(out).map(|m| m.len()).unwrap_or(0));

    // ─── Write a manifest ──────────────────────────────────────────────
    let manifest_path = Path::new("data/baseline/programming_taxonomy_v1.manifest.json");
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let simple_manifest = format!(
        r#"{{
  "name": "programming_taxonomy_v1",
  "atoms": {},
  "edges": {},
  "source": "Idan research 22.04.2026",
  "created_ts": {}
}}
"#,
        stats.atom_count, stats.edge_count, ts,
    );
    std::fs::write(manifest_path, simple_manifest)?;
    println!("  manifest: {}", manifest_path.display());

    Ok(())
}
