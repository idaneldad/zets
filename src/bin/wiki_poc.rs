//! POC: Ingest one wiki batch (10K Hebrew Wikipedia entries) into ZETS.
//!
//! Measures: ingestion time, edges generated, memory used,
//! adjacency-index build time, query latency on the result.

use std::time::Instant;

use zets::edge_store::EdgeStore;
use zets::learning::{self, AuxSynsetAllocator, Tier};
use zets::{LangCode, SynsetId};

fn main() {
    let path = "data/hebrew/wiki_batch_0030.tsv";
    println!("=== ZETS Wiki POC — single batch ingestion ===\n");

    // Phase 1: read + parse
    let t0 = Instant::now();
    let content = std::fs::read_to_string(path).expect("read tsv");
    let read_ms = t0.elapsed();
    println!("[1] Read {} bytes in {read_ms:?}", content.len());

    let t1 = Instant::now();
    let entries = match learning::parse_tsv(&content) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Parse error: {e}");
            std::process::exit(1);
        }
    };
    let parse_ms = t1.elapsed();
    println!("[2] Parsed {} entries in {parse_ms:?}", entries.len());

    // Phase 2: ingest
    let mut store = EdgeStore::new_with_meta();
    let meta_count = store.len();
    let mut aux = AuxSynsetAllocator::new();

    let t2 = Instant::now();
    let stats = learning::ingest(&mut store, &mut aux, &entries, LangCode::HEBREW, Tier::Tier3Corpus);
    let ingest_ms = t2.elapsed();

    println!("[3] Ingested in {ingest_ms:?}:");
    println!("    Edges added:       {}", stats.edges_added);
    println!("      POS:             {}", stats.pos_edges);
    println!("      Translations:    {}", stats.translation_edges);
    println!("      Definitions:     {}", stats.definition_edges);
    println!("      Synonyms:        {}", stats.synonym_edges);
    println!("      Unresolved syns: {}", stats.unresolved_synonyms);
    println!("    Aux synsets:       {}", stats.synsets_allocated);

    // Phase 3: adjacency index
    use zets::edge_store::AdjacencyIndex;
    let t3 = Instant::now();
    let idx = AdjacencyIndex::build(&store);
    let idx_ms = t3.elapsed();
    println!("[4] AdjacencyIndex built in {idx_ms:?}, {} bytes RAM", idx.bytes_used());

    println!("\nGraph state:");
    println!("  Meta-graph edges:    {meta_count}");
    println!("  Learned edges:       {}", store.len() - meta_count);
    println!("  Total edges:         {}", store.len());
    println!("  EdgeStore RAM:       {} bytes ({:.2} per edge)",
        store.bytes_used(),
        store.bytes_used() as f64 / store.len() as f64);
    println!("  Index RAM:           {} bytes", idx.bytes_used());
    println!("  Total RAM:           {} bytes", store.bytes_used() + idx.bytes_used());

    // Phase 4: persist to disk
    let pack_path = "/tmp/zets_wiki_batch_0030.pack";
    let t4 = Instant::now();
    {
        use std::fs::File;
        use std::io::BufWriter;
        let f = File::create(pack_path).expect("create");
        let mut w = BufWriter::new(f);
        store.write_to(&mut w).expect("write");
    }
    let write_ms = t4.elapsed();
    let file_size = std::fs::metadata(pack_path).map(|m| m.len()).unwrap_or(0);
    println!("\n[5] Persisted to {pack_path}: {file_size} bytes in {write_ms:?}");

    // Phase 5: dedup analysis — how many UNIQUE definition strings?
    println!("\n=== Dedup analysis (string-level) ===");
    use std::collections::HashSet;
    let mut unique_defs: HashSet<&String> = HashSet::new();
    let mut total_def_bytes = 0usize;
    let mut unique_def_bytes = 0usize;
    for e in &entries {
        total_def_bytes += e.definition.len();
        if unique_defs.insert(&e.definition) {
            unique_def_bytes += e.definition.len();
        }
    }
    let dedup_savings = total_def_bytes - unique_def_bytes;
    let dedup_pct = 100.0 * dedup_savings as f64 / total_def_bytes as f64;
    println!("  Total definition bytes:    {total_def_bytes}");
    println!("  Unique definition bytes:   {unique_def_bytes} ({} unique strings)", unique_defs.len());
    println!("  Dedup savings:             {dedup_savings} bytes ({dedup_pct:.2}%)");

    // Phase 6: query demo
    println!("\n=== Query demo ===");
    let sample_synset = entries[100].synset_id;
    let sample_surface = &entries[100].surface;
    println!("Querying outgoing edges from synset {} ('{}')...", sample_synset.0, sample_surface);
    let t5 = Instant::now();
    let outgoing = idx.outgoing(&store, sample_synset);
    let query_ns = t5.elapsed().as_nanos();
    println!("  Found {} edges in {query_ns}ns ({:.1}us)", outgoing.len(), query_ns as f64 / 1000.0);
    for e in outgoing.iter().take(3) {
        println!("    -> synset {}, relation {:?}", e.target.0, e.relation);
    }
}
