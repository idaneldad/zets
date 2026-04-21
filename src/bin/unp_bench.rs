//! Measure UNP normalize throughput — the hottest path.

use std::time::Instant;
use zets::{unp, LangCode};

fn main() {
    let n: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1_000_000);

    // Realistic mixed inputs: Hebrew and English sentences
    let samples: Vec<&str> = vec![
        "הבית הגדול",
        "כלב קטן ושחור",
        "Hello World",
        "The quick brown fox",
        "שָׁלוֹם לכם",
        "מה שלומך היום?",
        "Programming is fun",
        "אני אוהב קוד",
        "This is a test sentence",
        "בית ספר יסודי",
    ];

    // Serial Hebrew
    let t = Instant::now();
    let mut total_bytes = 0usize;
    for i in 0..n {
        let s = samples[i % samples.len()];
        let c = unp::normalize(s, LangCode::HEBREW);
        total_bytes += c.as_bytes().len();
    }
    let elapsed = t.elapsed();
    let rate = n as f64 / elapsed.as_secs_f64();
    println!("UNP normalize (serial, {n} iterations):");
    println!("  Total time:    {elapsed:?}");
    println!("  Rate:          {rate:>14.0} ops/sec");
    println!("  Per normalize: {:?}", elapsed / u32::try_from(n).unwrap_or(u32::MAX));
    println!("  Bytes output:  {total_bytes} (sanity check)");
}
