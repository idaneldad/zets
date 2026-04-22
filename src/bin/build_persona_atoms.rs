//! `build_persona_atoms` — convert a JSONL seed file into a ZETS .atoms snapshot.
//!
//! Input format (one JSON object per line):
//!   {"subject":"idan","relation":"is_a","object":"human"}
//!   {"subject":"idan","relation":"likes","object":"zets"}
//!
//! Atoms are created on first mention. Relations use the code from
//! `relations::by_name()`. Unknown relations are skipped (with warning).
//!
//! Output: a binary .atoms file compatible with atom_persist::load_from_file.
//!
//! Usage:
//!   build-persona-atoms --seed data/personas/idan.seed.jsonl \
//!                        --out   data/clients/idan.atoms

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use zets::atom_persist;
use zets::atoms::{AtomKind, AtomStore};
use zets::http_server::json_get_str;
use zets::relations;

fn parse_args() -> (PathBuf, PathBuf) {
    let args: Vec<String> = std::env::args().collect();
    let mut seed = PathBuf::from("seed.jsonl");
    let mut out = PathBuf::from("out.atoms");
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--seed" => {
                seed = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--out" => {
                out = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            _ => i += 1,
        }
    }
    (seed, out)
}

fn main() -> std::io::Result<()> {
    let (seed_path, out_path) = parse_args();

    let text = fs::read_to_string(&seed_path)?;
    let mut store = AtomStore::new();
    let mut cache: HashMap<String, u32> = HashMap::new();
    let mut skipped = 0usize;
    let mut triples = 0usize;

    let get_or_create = |store: &mut AtomStore,
                         cache: &mut HashMap<String, u32>,
                         name: &str|
     -> u32 {
        if let Some(&id) = cache.get(name) {
            return id;
        }
        let id = store.put(AtomKind::Concept, name.as_bytes().to_vec());
        cache.insert(name.to_string(), id);
        id
    };

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let subj = match json_get_str(line, "subject") {
            Some(s) => s.to_string(),
            None => {
                skipped += 1;
                continue;
            }
        };
        let rel_name = match json_get_str(line, "relation") {
            Some(s) => s.to_string(),
            None => {
                skipped += 1;
                continue;
            }
        };
        let obj = match json_get_str(line, "object") {
            Some(s) => s.to_string(),
            None => {
                skipped += 1;
                continue;
            }
        };

        let rel_code = match relations::by_name(&rel_name) {
            Some(r) => r.code,
            None => {
                eprintln!("  ! skip triple: unknown relation '{}'", rel_name);
                skipped += 1;
                continue;
            }
        };

        let sid = get_or_create(&mut store, &mut cache, &subj);
        let oid = get_or_create(&mut store, &mut cache, &obj);
        store.link(sid, oid, rel_code, 200, 0);
        triples += 1;
    }

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    match atom_persist::save_to_file(&store, &out_path) {
        Ok(_) => {
            println!(
                "  {} atoms, {} edges, {} triples ingested, {} skipped",
                store.atom_count(),
                store.edge_count(),
                triples,
                skipped
            );
        }
        Err(e) => {
            eprintln!("  save failed: {:?}", e);
            std::process::exit(1);
        }
    }
    Ok(())
}
