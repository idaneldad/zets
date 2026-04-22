//! `wiki_bridge` — link the programming taxonomy to Wikipedia atoms via alias map.
//!
//! Problem: taxonomy has `"python"` atom (hash H1). Wikipedia has `"word:python"`
//! atom (hash H2). Same concept, different hash. content_hash alone doesn't link.
//!
//! Solution: alias map built from convention — "atom X in taxonomy ↔ atom
//! word:X in wiki" — and feed into hash_registry via both atoms.
//!
//! Result: after running this, hash_registry contains cross-refs so queries
//! over ZetsEngine (not yet implemented) could pull facts from BOTH sources.
//!
//! This demonstrates the principle without modifying either snapshot.
//!
//! Usage: cargo run --release --bin wiki_bridge

use std::path::Path;
use zets::atoms::AtomStore;
use zets::atom_persist;
use zets::hash_registry::{GraphKind, GraphRef, HashRegistry};

fn main() -> std::io::Result<()> {
    println!("═══ ZETS Wiki Bridge — Taxonomy ↔ Wiki ═══");
    println!();

    // Load both snapshots
    print!("  Loading programming_taxonomy_v1 ... ");
    let tax_path = Path::new("data/baseline/programming_taxonomy_v1.atoms");
    let tax = atom_persist::load_from_file(tax_path).map_err(|e|
        std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", e)))?;
    println!("{} atoms", tax.atom_count());

    print!("  Loading wiki_all_domains_v1 ... ");
    let wiki_path = Path::new("data/baseline/wiki_all_domains_v1.atoms");
    let wiki = atom_persist::load_from_file(wiki_path).map_err(|e|
        std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", e)))?;
    println!("{} atoms", wiki.atom_count());
    println!();

    // Build lookup tables
    let tax_by_label: std::collections::HashMap<String, u32> = (0..tax.atom_count() as u32)
        .filter_map(|i| tax.get(i).map(|a|
            (String::from_utf8_lossy(&a.data).to_string(), i)))
        .collect();

    let wiki_by_label: std::collections::HashMap<String, u32> = (0..wiki.atom_count() as u32)
        .filter_map(|i| wiki.get(i).map(|a|
            (String::from_utf8_lossy(&a.data).to_string(), i)))
        .collect();

    // Alias rules for languages (taxonomy → wiki)
    let aliases: Vec<(&str, Vec<&str>)> = vec![
        ("python",     vec!["word:python"]),
        ("rust",       vec!["word:rust"]),
        ("javascript", vec!["word:javascript"]),
        ("typescript", vec!["word:typescript"]),
        ("ruby",       vec!["word:ruby"]),
        ("perl",       vec!["word:perl"]),
        ("bash",       vec!["word:bash"]),
        ("lisp",       vec!["word:lisp"]),
        ("java",       vec!["word:java"]),
        ("scala",      vec!["word:scala"]),
        ("kotlin",     vec!["word:kotlin"]),
        ("swift",      vec!["word:swift"]),
        ("haskell",    vec!["word:haskell"]),
        ("ocaml",      vec!["word:ocaml"]),
        ("prolog",     vec!["word:prolog"]),
        ("erlang",     vec!["word:erlang"]),
        ("php",        vec!["word:php"]),
        ("lua",        vec!["word:lua"]),
        ("sql",        vec!["word:sql"]),
        ("cpp",        vec!["word:c++", "word:cpp"]),
        // Alias rules for paradigms
        ("functional", vec!["word:functional"]),
        ("imperative_procedural", vec!["word:imperative", "word:procedural"]),
        ("object_oriented", vec!["word:object-oriented", "word:oop"]),
    ];

    // Build hash registry
    let mut reg = HashRegistry::new();

    let mut linked = 0;
    let mut only_taxonomy = 0;
    let mut only_wiki = 0;
    let mut failed = Vec::new();

    for (tax_label, wiki_labels) in &aliases {
        let tax_atom_id = tax_by_label.get(*tax_label);

        if let Some(&tid) = tax_atom_id {
            let tax_atom = tax.get(tid).unwrap();
            // Register taxonomy side
            reg.register(tax_atom.content_hash, GraphRef {
                graph: GraphKind::AtomStore,  // placeholder — using AtomStore for both
                local_id: tid,
                confidence: 240,
            });

            // Try to find matching wiki atoms
            let mut found_any = false;
            for wiki_label in wiki_labels {
                if let Some(&wid) = wiki_by_label.get(*wiki_label) {
                    let wiki_atom = wiki.get(wid).unwrap();
                    // Register under the TAXONOMY's hash with a GraphKind indicating
                    // this is a Wiki alias (we use DialectOverlay as a tagged slot).
                    reg.register(tax_atom.content_hash, GraphRef {
                        graph: GraphKind::DialectOverlay(1), // "wiki_alias" slot
                        local_id: wid,
                        confidence: 230,
                    });
                    // Also cross-register the wiki atom's own hash
                    reg.register(wiki_atom.content_hash, GraphRef {
                        graph: GraphKind::PieceGraph,  // placeholder for wiki source
                        local_id: wid,
                        confidence: 240,
                    });
                    found_any = true;
                    linked += 1;
                    break;
                }
            }
            if !found_any {
                only_taxonomy += 1;
                failed.push(*tax_label);
            }
        } else {
            // taxonomy label not found (shouldn't happen)
            failed.push(*tax_label);
        }
    }

    // Also: find wiki atoms we didn't cover (just for stats)
    for wiki_label in wiki_by_label.keys() {
        if wiki_label.starts_with("word:") {
            let stem = &wiki_label[5..];
            // is it any language from our aliases?
            let covered = aliases.iter().any(|(_, variants)|
                variants.iter().any(|v| *v == wiki_label.as_str()));
            if !covered && is_language_name(stem) {
                only_wiki += 1;
            }
        }
    }

    println!("━━━ Bridge results ━━━");
    println!();
    println!("  Alias rules tried:     {}", aliases.len());
    println!("  Linked (found in both): {}", linked);
    println!("  Only in taxonomy:      {}", only_taxonomy);
    println!("  Only in wiki:          {} (rough)", only_wiki);
    println!();
    println!("  Hash registry state:");
    println!("    Total hashes:        {}", reg.total_hashes());
    println!("    Shared (2+ graphs):  {}", reg.shared_hashes());
    println!();
    if !failed.is_empty() {
        println!("  Failed to bridge (not in wiki):");
        for f in &failed {
            println!("    {}", f);
        }
    }

    println!();
    println!("━━━ Gap analysis ━━━");
    let gap_tax_to_wiki = reg.gap(GraphKind::AtomStore, GraphKind::DialectOverlay(1));
    let gap_wiki_to_tax = reg.gap(GraphKind::DialectOverlay(1), GraphKind::AtomStore);
    println!("  Taxonomy knows, wiki-alias doesn't: {}", gap_tax_to_wiki.len());
    println!("  Wiki-alias knows, taxonomy doesn't: {}", gap_wiki_to_tax.len());
    println!();
    println!("  Next step: feed 'only_wiki' terms back into taxonomy for enrichment.");

    Ok(())
}

fn is_language_name(stem: &str) -> bool {
    // Rough heuristic — list of common language stems we'd care about
    const KNOWN: &[&str] = &[
        "python", "rust", "javascript", "typescript", "ruby", "perl", "bash",
        "lisp", "java", "scala", "kotlin", "swift", "haskell", "ocaml",
        "prolog", "erlang", "php", "lua", "sql", "cpp", "c++", "go", "r",
        "matlab", "fortran", "pascal", "basic", "smalltalk", "elixir",
        "julia", "scheme", "clojure", "fsharp", "csharp",
    ];
    KNOWN.contains(&stem)
}
