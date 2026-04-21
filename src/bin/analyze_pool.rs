use zets::piece_graph_loader::PieceGraphLoader;

fn main() {
    let loader = PieceGraphLoader::new("data/multilang");
    let graph = loader.load().unwrap();

    // Count how many pieces are used by: concepts (anchors/glosses) vs. lang data
    let mut concept_pieces = std::collections::HashSet::new();
    for c in &graph.concepts {
        concept_pieces.insert(c.anchor_piece);
        if c.gloss_piece != 0 {
            concept_pieces.insert(c.gloss_piece);
        }
    }

    let mut lang_surface_pieces = std::collections::HashSet::new();
    let mut lang_def_pieces = std::collections::HashSet::new();
    let mut lang_syn_pieces = std::collections::HashSet::new();
    for idx in graph.lang_indexes.values() {
        for k in idx.surface_to_concepts.keys() {
            lang_surface_pieces.insert(*k);
        }
        for (k, defs) in &idx.definitions {
            lang_surface_pieces.insert(*k);
            for d in defs {
                lang_def_pieces.insert(*d);
            }
        }
        for (k, syns) in &idx.synonyms {
            lang_surface_pieces.insert(*k);
            for s in syns {
                lang_syn_pieces.insert(*s);
            }
        }
    }

    println!("Total pieces in pool      : {}", graph.pieces.len());
    println!("Used by concepts          : {}", concept_pieces.len());
    println!("Used as lang surfaces     : {}", lang_surface_pieces.len());
    println!("Used as lang definitions  : {}", lang_def_pieces.len());
    println!("Used as lang synonyms     : {}", lang_syn_pieces.len());

    // How much bytes do definitions take?
    let mut def_bytes = 0u64;
    for pid in &lang_def_pieces {
        def_bytes += graph.pieces.get(*pid).len() as u64;
    }
    println!();
    println!("Bytes in definitions      : {} ({:.1} MB)", def_bytes, def_bytes as f64 / 1_048_576.0);

    let mut surface_bytes = 0u64;
    for pid in &lang_surface_pieces {
        surface_bytes += graph.pieces.get(*pid).len() as u64;
    }
    println!("Bytes in surfaces         : {} ({:.1} MB)", surface_bytes, surface_bytes as f64 / 1_048_576.0);

    let mut concept_bytes = 0u64;
    for pid in &concept_pieces {
        concept_bytes += graph.pieces.get(*pid).len() as u64;
    }
    println!("Bytes in concepts anchor/gloss: {} ({:.1} MB)", concept_bytes, concept_bytes as f64 / 1_048_576.0);
}
