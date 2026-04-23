//! zets_v4_query — לtesting — load snapshot + ask one question.
use zets::graph_v4::{answer, compute_idf, load, phrases_from_graph};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: zets_v4_query <snapshot.zv4> \"question\"");
        std::process::exit(1);
    }
    let path = &args[1];
    let q = &args[2];

    let t = Instant::now();
    let mut g = load(path)?;
    g.build_indexes();
    let idf = compute_idf(&g);
    let phrases = phrases_from_graph(&g);
    eprintln!("loaded in {:.2}s, idf={}, phrases={}", t.elapsed().as_secs_f32(), idf.len(), phrases.len());

    let t = Instant::now();
    let ans = answer(q, &g, &idf, &phrases, 5, 5);
    let dt = t.elapsed().as_millis();

    println!("\n❓ {}", q);
    println!("   tokens: {:?}", ans.tokens);
    println!("   seeds:  {:?}", ans.seeds.iter().map(|(k,key)| format!("{:?}:{}", k, key)).collect::<Vec<_>>());
    println!("   📚 top articles:");
    for (name, score) in &ans.top_articles {
        println!("      {:40} {:.2}", name, score);
    }
    println!("   📝 top sentences:");
    for (text, score, _) in &ans.top_sentences {
        let preview: String = text.chars().take(150).collect();
        println!("      [{:.2}] \"{}\"", score, preview);
    }
    println!("   ⏱ {} ms", dt);
    Ok(())
}
