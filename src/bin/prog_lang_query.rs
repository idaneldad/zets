//! `prog_lang_query` — query the programming-languages taxonomy snapshot.
//!
//! Shows how ZETS answers questions like "what language for Linux automation?"
//! by walking the graph of paradigms/execution/type/purpose.
//!
//! Usage:
//!   cargo run --release --bin prog_lang_query -- paradigm functional
//!   cargo run --release --bin prog_lang_query -- usecase linux_server_admin
//!   cargo run --release --bin prog_lang_query -- lang python
//!   cargo run --release --bin prog_lang_query -- gap functional imperative_procedural

use std::path::Path;
use zets::atoms::AtomStore;
use zets::atom_persist;
use zets::relations;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage:");
        eprintln!("  prog_lang_query paradigm <name>");
        eprintln!("  prog_lang_query usecase <name>");
        eprintln!("  prog_lang_query lang <name>");
        eprintln!("  prog_lang_query purpose <name>");
        eprintln!("  prog_lang_query list-categories");
        std::process::exit(1);
    }

    let path = Path::new("data/baseline/programming_taxonomy_v1.atoms");
    let store = match atom_persist::load_from_file(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("load failed: {:?}", e);
            eprintln!("run `cargo run --release --bin seed_programming_taxonomy` first");
            std::process::exit(1);
        }
    };

    let is_a = relations::by_name("is_a").unwrap().code;
    let has_attr = relations::by_name("has_attribute").unwrap().code;
    let used_for = relations::by_name("used_for").unwrap().code;

    // Build label → atom lookup
    let label_to_atom: std::collections::HashMap<String, u32> = (0..store.atom_count() as u32)
        .filter_map(|i| {
            store.get(i).map(|a| {
                (String::from_utf8_lossy(&a.data).to_string(), i)
            })
        })
        .collect();

    let atom_label = |id: u32| -> String {
        store.get(id).map(|a| String::from_utf8_lossy(&a.data).to_string())
            .unwrap_or_else(|| format!("atom#{}", id))
    };

    let cmd = args[1].as_str();
    match cmd {
        "list-categories" => {
            let category = *label_to_atom.get("category").unwrap();
            let mut children: Vec<u32> = Vec::new();
            // Who is_a category?
            for i in 0..store.atom_count() as u32 {
                for edge in store.outgoing(i) {
                    if edge.relation == is_a && edge.to == category {
                        children.push(i);
                    }
                }
            }
            println!("Top-level categories:");
            for c in children {
                println!("  {}", atom_label(c));
            }
        }

        "paradigm" | "usecase" | "purpose" if args.len() >= 3 => {
            let name = &args[2];
            let atom = match label_to_atom.get(name) {
                Some(&a) => a,
                None => {
                    eprintln!("unknown: {}", name);
                    std::process::exit(1);
                }
            };
            println!("━━━ Languages where {} ━━━", atom_label(atom));
            println!();
            // Find atoms that have is_a OR used_for edge to this target
            let mut hits: Vec<(u32, &str)> = Vec::new();
            for i in 0..store.atom_count() as u32 {
                for edge in store.outgoing(i) {
                    if edge.to == atom {
                        if edge.relation == is_a {
                            hits.push((i, "is_a"));
                        } else if edge.relation == used_for {
                            hits.push((i, "used_for"));
                        }
                    }
                }
            }
            // sort by label for determinism
            hits.sort_by_key(|(id, _)| atom_label(*id));
            for (id, rel) in &hits {
                println!("  {:<30}  ({})", atom_label(*id), rel);
            }
            println!();
            println!("  {} language(s) or category total", hits.len());
        }

        "lang" if args.len() >= 3 => {
            let name = &args[2];
            let atom = match label_to_atom.get(name) {
                Some(&a) => a,
                None => {
                    eprintln!("unknown language: {}", name);
                    std::process::exit(1);
                }
            };
            println!("━━━ Profile of {} ━━━", atom_label(atom));
            println!();

            let mut paradigms = Vec::new();
            let mut attrs = Vec::new();
            let mut uses = Vec::new();

            for edge in store.outgoing(atom) {
                let target = atom_label(edge.to);
                match edge.relation {
                    r if r == is_a => paradigms.push(target),
                    r if r == has_attr => attrs.push(target),
                    r if r == used_for => uses.push(target),
                    _ => {}
                }
            }
            paradigms.sort();
            attrs.sort();
            uses.sort();

            if !paradigms.is_empty() {
                println!("  paradigms:      {}", paradigms.join(", "));
            }
            if !attrs.is_empty() {
                println!("  attributes:     {}", attrs.join(", "));
            }
            if !uses.is_empty() {
                println!("  used for:       {}", uses.join(", "));
            }
        }

        _ => {
            eprintln!("unknown command: {}", cmd);
            std::process::exit(1);
        }
    }
}
