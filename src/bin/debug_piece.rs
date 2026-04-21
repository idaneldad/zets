use zets::piece_graph_loader::PieceGraphLoader;

fn main() {
    let loader = PieceGraphLoader::new("data/multilang");
    let graph = loader.load().unwrap();

    // The graph is frozen — pieces.lookup might be broken after freeze
    let dog_piece = graph.pieces.lookup("dog");
    println!("'dog' piece_id: {:?}", dog_piece);

    let en_id = graph.langs.get("en");
    println!("'en' lang_id: {:?}", en_id);

    if let (Some(pid), Some(lid)) = (dog_piece, en_id) {
        let idx = graph.lang_indexes.get(&lid);
        println!("lang_index present: {}", idx.is_some());
        if let Some(idx) = idx {
            let concepts = idx.surface_to_concepts.get(&pid);
            println!("concepts for 'dog': {:?}", concepts.map(|v| v.len()));
        }
    }
}
