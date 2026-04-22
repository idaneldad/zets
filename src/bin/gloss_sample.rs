//! Sample glosses to understand what patterns we can mine.
use std::path::PathBuf;
use zets::pack::PackReader;
use zets::piece_graph::pos_code_to_str;

fn main() -> std::io::Result<()> {
    let graph = PackReader::read_core(&PathBuf::from("data/packs/zets.core"))?;
    // Sample 30 concepts with non-empty glosses
    let mut shown = 0;
    for c in &graph.concepts {
        let anchor = graph.pieces.get(c.anchor_piece);
        let gloss = graph.pieces.get(c.gloss_piece);
        if gloss.is_empty() || gloss.len() < 10 { continue; }
        if shown >= 30 { break; }
        println!("{:20} ({:4}) : {}", anchor, pos_code_to_str(c.pos),
            gloss.chars().take(100).collect::<String>());
        shown += 1;
    }
    // Now look for common patterns
    println!("\n=== Pattern hunt: glosses starting with common hypernym markers ===");
    let patterns = [
        "a kind of ", "a type of ", "a species of ", "any of ", "a ", "an ", "the ",
        "one of ", "member of ", "relating to ", "used to ", "used for ",
    ];
    let mut counts = [0u64; 20];
    let mut samples: Vec<Vec<String>> = vec![Vec::new(); 20];
    for c in &graph.concepts {
        let gloss = graph.pieces.get(c.gloss_piece).to_lowercase();
        for (i, p) in patterns.iter().enumerate() {
            if gloss.starts_with(p) {
                counts[i] += 1;
                if samples[i].len() < 3 {
                    let anchor = graph.pieces.get(c.anchor_piece).to_string();
                    samples[i].push(format!("{} → {}", anchor,
                        gloss.chars().take(80).collect::<String>()));
                }
            }
        }
    }
    for (i, p) in patterns.iter().enumerate() {
        println!("\n  '{}' : {} matches", p, counts[i]);
        for s in &samples[i] { println!("      {}", s); }
    }
    Ok(())
}
