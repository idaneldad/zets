//! Quick: count total edges in zets.core
use std::path::PathBuf;
use zets::pack::PackReader;

fn main() -> std::io::Result<()> {
    let path = PathBuf::from("data/packs/zets.core");
    let graph = PackReader::read_core(&path)?;
    let mut total = 0u64;
    let mut by_kind = [0u64; 256];
    let mut with_edges = 0;
    let mut max_edges = 0;
    for c in &graph.concepts {
        if !c.edges.is_empty() { with_edges += 1; }
        if c.edges.len() > max_edges { max_edges = c.edges.len(); }
        total += c.edges.len() as u64;
        for e in &c.edges {
            by_kind[e.kind as usize] += 1;
        }
    }
    println!("Total concepts : {}", graph.concepts.len());
    println!("Total edges    : {}", total);
    println!("Concepts w/edges: {} ({:.2}%)", with_edges,
        100.0 * with_edges as f64 / graph.concepts.len() as f64);
    println!("Max edges on 1 : {}", max_edges);
    println!("Edges by kind:");
    for (k, count) in by_kind.iter().enumerate() {
        if *count > 0 {
            println!("  kind={} count={}", k, count);
        }
    }
    Ok(())
}
