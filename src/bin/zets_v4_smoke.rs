//! zets_v4_smoke — smoke test ל-graph_v4 עם mock corpus.
//!
//! בודק: build → stats → retrieve על שאלות בסיסיות.
//! לא נוגע ב-disk. זה דרך לאמת end-to-end מהר.

use zets::graph_v4::{
    build_graph, compute_idf, phrases_from_graph, answer, BuildConfig, AtomKind,
};

fn mock_corpus() -> Vec<(String, String)> {
    vec![
        ("Gravity".into(),
         "Gravity is a fundamental force of nature. Gravity attracts all objects with mass. \
          The gravity of the Earth keeps us grounded. Isaac Newton described gravity mathematically. \
          Albert Einstein redefined gravity with general relativity. Gravity causes objects to fall.".into()),
        ("Heart".into(),
         "The heart is a muscular organ. The heart pumps blood through the body. \
          The heart beats about 60 to 100 times per minute. The heart has four chambers. \
          The heart is part of the circulatory system. Heart disease is a leading cause of death.".into()),
        ("Photosynthesis".into(),
         "Photosynthesis is the process plants use to make food. Photosynthesis requires sunlight. \
          Photosynthesis converts carbon dioxide into oxygen. Photosynthesis happens in chloroplasts. \
          Photosynthesis is essential for life on Earth. Green plants perform photosynthesis.".into()),
        ("Moon".into(),
         "The Moon is Earth's only natural satellite. The Moon orbits the Earth. \
          The Moon causes tides on Earth. The Moon's gravity affects our oceans. \
          The Moon reflects light from the Sun. Apollo missions landed on the Moon.".into()),
        ("Newton".into(),
         "Isaac Newton formulated the laws of motion. Newton discovered the law of gravity. \
          Newton's laws describe how objects move. Newton was an English mathematician. \
          Newton invented calculus independently. Newton's principia is a famous work.".into()),
    ]
}

fn main() {
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║  ZETS v4 — Smoke Test (Rust implementation)           ║");
    println!("╚═══════════════════════════════════════════════════════╝");
    let articles = mock_corpus();
    println!("\n[1] Corpus: {} articles", articles.len());

    let t0 = std::time::Instant::now();
    let mut g = build_graph(&articles, &BuildConfig::default());
    g.build_indexes();
    let dt = t0.elapsed();
    println!("[2] Graph built in {:.2}s", dt.as_secs_f32());

    let s = g.stats();
    println!("[3] Stats:");
    s.print();

    let idf = compute_idf(&g);
    let phrases = phrases_from_graph(&g);
    println!("[4] IDF: {} atoms, phrases: {}", idf.len(), phrases.len());

    let questions = [
        "What is gravity?",
        "What is the heart?",
        "What is photosynthesis?",
        "What is the Moon?",
        "Who was Isaac Newton?",
    ];
    println!("\n[5] Retrieval test:");
    let mut correct = 0;
    for q in &questions {
        let ans = answer(q, &g, &idf, &phrases, 3, 3);
        let top = ans.top_articles.get(0).map(|(n, s)| (n.as_str(), *s)).unwrap_or(("(none)", 0.0));
        // heuristic: מצופה שנוכל article ישויך לשאלה
        let expected = if q.contains("gravity") { "Gravity" }
                       else if q.contains("heart") { "Heart" }
                       else if q.contains("photosynthesis") { "Photosynthesis" }
                       else if q.contains("Moon") { "Moon" }
                       else if q.contains("Newton") { "Newton" }
                       else { "?" };
        let ok = top.0 == expected;
        let mark = if ok { "✓" } else { "✗" };
        println!("  {} {:35} → {:20} ({:.2})  expected={}", mark, q, top.0, top.1, expected);
        if ok { correct += 1; }
        if let Some((text, score, _)) = ans.top_sentences.get(0) {
            let preview: String = text.chars().take(120).collect();
            println!("      best: \"{}\" [{:.2}]", preview, score);
        }
    }
    println!("\n[6] Accuracy: {}/{}", correct, questions.len());

    // Determinism check
    let mut g2 = build_graph(&articles, &BuildConfig::default());
    g2.build_indexes();
    let det = g.atom_count() == g2.atom_count() && g.edge_count() == g2.edge_count();
    println!("[7] Determinism: {}", if det { "✓ (same input → same graph)" } else { "✗" });

    println!("\n{}", if correct == questions.len() && det {
        "🟢 ALL SMOKE TESTS PASSED"
    } else {
        "🔴 SOME FAILURES"
    });
}
