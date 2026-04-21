//! Scale test: linear scan vs. AdjacencyIndex binary search.

use std::time::Instant;
use zets::edge_store::{AdjacencyIndex, Edge, EdgeStore, Relation};
use zets::SynsetId;

fn main() {
    let n: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1_000_000);
    let queries: usize = 100;

    println!("Building graph with {n} edges...");
    let t0 = Instant::now();
    let mut store = EdgeStore::with_capacity(n);
    for i in 0..n {
        let src = if i % 100 == 0 { 1000 } else { (i as u32) % 500_000 };
        store.push(Edge {
            source: SynsetId(src),
            target: SynsetId((i as u32 * 31) % 500_000),
            relation: Relation::IsA,
            weight: 0,
            provenance: 0,
        });
    }
    println!("Store built in {:?}", t0.elapsed());
    println!("Store RAM:  {} bytes ({:.2} per edge)\n",
        store.bytes_used(),
        store.bytes_used() as f64 / n as f64);

    // Build adjacency index
    let t_idx = Instant::now();
    let idx = AdjacencyIndex::build(&store);
    let build_time = t_idx.elapsed();
    println!("AdjacencyIndex built in {build_time:?}");
    println!("Index RAM:  {} bytes ({:.2} per edge)\n",
        idx.bytes_used(),
        idx.bytes_used() as f64 / n as f64);

    // ---- Linear scan ----
    let t_linear_sparse = Instant::now();
    let mut total = 0usize;
    for _ in 0..queries {
        total += store.outgoing(SynsetId(999_999)).len();
    }
    let linear_sparse = t_linear_sparse.elapsed();
    println!("Linear outgoing (sparse, unknown): {linear_sparse:?} / {queries} = {:?} avg  [{total} matches]",
        linear_sparse / queries as u32);

    let t_linear_dense = Instant::now();
    let mut total = 0usize;
    for _ in 0..queries {
        total += store.outgoing(SynsetId(1000)).len();
    }
    let linear_dense = t_linear_dense.elapsed();
    println!("Linear outgoing (dense, hub):      {linear_dense:?} / {queries} = {:?} avg  [{total} matches]",
        linear_dense / queries as u32);

    // ---- Indexed ----
    let t_idx_sparse = Instant::now();
    let mut total = 0usize;
    for _ in 0..queries {
        total += idx.outgoing(&store, SynsetId(999_999)).len();
    }
    let idx_sparse = t_idx_sparse.elapsed();
    println!("Indexed outgoing (sparse):         {idx_sparse:?} / {queries} = {:?} avg  [{total} matches]",
        idx_sparse / queries as u32);

    let t_idx_dense = Instant::now();
    let mut total = 0usize;
    for _ in 0..queries {
        total += idx.outgoing(&store, SynsetId(1000)).len();
    }
    let idx_dense = t_idx_dense.elapsed();
    println!("Indexed outgoing (dense):          {idx_dense:?} / {queries} = {:?} avg  [{total} matches]",
        idx_dense / queries as u32);

    // Speedups
    let sparse_speedup = linear_sparse.as_secs_f64() / idx_sparse.as_secs_f64();
    let dense_speedup = linear_dense.as_secs_f64() / idx_dense.as_secs_f64();
    println!();
    println!("Speedups:");
    println!("  Sparse (empty result): {sparse_speedup:>8.1}x");
    println!("  Dense  (hub result):   {dense_speedup:>8.1}x");

    // Count-only — should be fastest (no materialization)
    let t_count = Instant::now();
    let mut total = 0usize;
    for _ in 0..queries {
        total += idx.outgoing_count(&store, SynsetId(1000));
    }
    let count_time = t_count.elapsed();
    println!();
    println!("Indexed outgoing_count (dense):    {count_time:?} / {queries} = {:?} avg  [{total} total]",
        count_time / queries as u32);
    println!("  (no Vec allocation, just binary search range)");
}
