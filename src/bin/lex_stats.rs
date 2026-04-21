//! Memory diagnostics for Lexicon.

use std::time::Instant;
use zets::lexicon::Lexicon;

fn main() {
    let t = Instant::now();
    let mut lex = Lexicon::new();
    lex.load_from_dir("data/multilang").expect("load failed");
    let load_ms = t.elapsed();

    let (gloss_count, surface_count, entry_count) = lex.pool_stats();

    println!("Lexicon loaded in {:?}", load_ms);
    println!();
    println!("  Languages         : {}", lex.languages().len());
    println!("  Total entries     : {}", entry_count);
    println!("  Distinct glosses  : {} (in pool)", gloss_count);
    println!("  Distinct surfaces : {} (in pool)", surface_count);
    println!();
    println!("  Avg key chars per entry: ~12 (lang+surface in HashMap key)");
    println!("  Each entry holds compact ids only (~24 bytes payload)");
}
