//! `benchmark_with_adapter` — runs the same 20-question benchmark as
//! run_benchmark, but BEFORE answering each question we pass it through
//! the LLM adapter to extract structured (intent, key_terms, domain).
//! These are then used to seed the session more precisely.
//!
//! This is Phase 1 of the bottleneck master plan — adding an NLU layer
//! in front of ZETS's reasoning.
//!
//! With offline_only=true (no API key), this uses the rule-based local
//! parser. Even without a real LLM, domain classification and key-term
//! extraction should improve accuracy over the naive token-matching
//! baseline of run_benchmark.

use zets::atoms::AtomStore;
use zets::benchmarks::{run_benchmark, Question, answer_question};
use zets::bootstrap::bootstrap;
use zets::ingestion::{ingest_text, IngestConfig};
use zets::llm_adapter::{LlmAdapter, local_parse};
use zets::meta_learning::MetaLearner;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  ZETS Benchmark WITH Adapter — Phase 1 measurement        ║");
    println!("║  Compares baseline vs LLM-adapter-assisted question answering║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let questions = build_questions();
    let mut store = build_store();
    let mut meta = MetaLearner::new();

    // ═══════════════════════════════════════════════════
    // Run 1: BASELINE — no adapter (same as run_benchmark)
    // ═══════════════════════════════════════════════════
    println!("━━━ Run A: Baseline (no adapter) ━━━");
    let baseline = run_benchmark(&mut store, &mut meta, &questions);
    println!("  Accuracy: {:.1}%  Correct: {}/{}",
        baseline.accuracy() * 100.0, baseline.correct, baseline.total);
    println!("  Relevance rate: {:.1}%", baseline.relevance_rate() * 100.0);
    println!("  By category:");
    for (cat, acc) in baseline.category_breakdown() {
        println!("    {:<12} {:.1}%", cat, acc * 100.0);
    }
    println!();

    // ═══════════════════════════════════════════════════
    // Run 2: ADAPTER-ASSISTED — parse first, then answer
    // ═══════════════════════════════════════════════════
    println!("━━━ Run B: With LLM adapter (offline/local parser) ━━━");
    let mut adapter = LlmAdapter::offline();
    let mut correct = 0usize;
    let mut total = 0usize;
    let mut by_cat: std::collections::HashMap<String, (usize, usize)> =
        std::collections::HashMap::new();

    for q in &questions {
        // Parse the question first
        let parse = adapter.parse(&q.text).unwrap_or_else(|_| local_parse(&q.text));

        // Build an augmented question — use parse.key_terms as the text ZETS
        // will token-match against. This is the KEY improvement: instead of
        // matching every word in the question (including stopwords), we
        // match only the content words.
        let augmented = Question {
            id: q.id.clone(),
            text: parse.key_terms.join(" "),
            choices: q.choices.clone(),
            expected: q.expected.clone(),
            category: if parse.domain != "general" {
                parse.domain.clone()
            } else {
                q.category.clone()
            },
        };

        let result = answer_question(&mut store, &mut meta, &augmented);
        if result.correct { correct += 1; }
        total += 1;
        let entry = by_cat.entry(q.category.clone()).or_insert((0, 0));
        entry.1 += 1;
        if result.correct { entry.0 += 1; }
    }

    let adapter_acc = correct as f32 / total as f32;
    println!("  Accuracy: {:.1}%  Correct: {}/{}", adapter_acc * 100.0, correct, total);
    println!("  Parse count: {}  (fallback count: {})",
        adapter.parse_count(), adapter.fallback_count());
    println!("  By category:");
    let mut breakdown: Vec<(String, f32)> = by_cat.iter()
        .map(|(k, (c, t))| (k.clone(), *c as f32 / *t as f32))
        .collect();
    breakdown.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    for (cat, acc) in breakdown {
        println!("    {:<12} {:.1}%", cat, acc * 100.0);
    }
    println!();

    // ═══════════════════════════════════════════════════
    // Run 3 (optional): WITH REAL GEMINI API if key is set
    // ═══════════════════════════════════════════════════
    let gemini_acc = if std::env::var("ZETS_GEMINI_KEY").is_ok() {
        println!("━━━ Run C: With REAL Gemini 2.5 Flash API ━━━");
        let mut adapter_live = LlmAdapter::new();  // reads ZETS_GEMINI_KEY

        let mut store3 = AtomStore::new();
        bootstrap(&mut store3);
        let _ = ingest_text(&mut store3, "world_facts_v1", KNOWLEDGE, &IngestConfig::default());
        let mut meta3 = MetaLearner::new();

        let mut c_correct = 0usize;
        let mut c_total = 0usize;
        let mut c_by_cat: std::collections::HashMap<String, (usize, usize)> =
            std::collections::HashMap::new();

        for q in &questions {
            let parse = adapter_live.parse(&q.text).unwrap_or_else(|_| local_parse(&q.text));
            let augmented = Question {
                id: q.id.clone(),
                text: parse.key_terms.join(" "),
                choices: q.choices.clone(),
                expected: q.expected.clone(),
                category: if parse.domain != "general" { parse.domain.clone() } else { q.category.clone() },
            };
            let result = answer_question(&mut store3, &mut meta3, &augmented);
            if result.correct { c_correct += 1; }
            c_total += 1;
            let entry = c_by_cat.entry(q.category.clone()).or_insert((0, 0));
            entry.1 += 1;
            if result.correct { entry.0 += 1; }
        }

        let acc = c_correct as f32 / c_total as f32;
        println!("  Accuracy: {:.1}%  Correct: {}/{}", acc * 100.0, c_correct, c_total);
        println!("  Parse count: {}  (fallback count: {})",
            adapter_live.parse_count(), adapter_live.fallback_count());
        let real_calls = adapter_live.parse_count() - adapter_live.fallback_count();
        println!("  Real Gemini API calls: {}", real_calls);
        println!("  By category:");
        let mut breakdown: Vec<(String, f32)> = c_by_cat.iter()
            .map(|(k, (c, t))| (k.clone(), *c as f32 / *t as f32))
            .collect();
        breakdown.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        for (cat, acc) in breakdown {
            println!("    {:<12} {:.1}%", cat, acc * 100.0);
        }
        println!();
        Some(acc)
    } else {
        println!("━━━ Run C: SKIPPED (set ZETS_GEMINI_KEY to run with real API) ━━━");
        println!();
        None
    };

        // ═══════════════════════════════════════════════════
    // Comparison
    // ═══════════════════════════════════════════════════
    println!("━━━ Comparison ━━━");
    println!("  Baseline (A):         {:.1}%", baseline.accuracy() * 100.0);
    println!("  Adapter (B):          {:.1}%", adapter_acc * 100.0);
    let delta = (adapter_acc - baseline.accuracy()) * 100.0;
    if delta.abs() < 0.5 {
        println!("  Δ: no meaningful change ({:+.1} pp)", delta);
    } else if delta > 0.0 {
        println!("  Δ: {:+.1} percentage points (adapter helped)", delta);
    } else {
        println!("  Δ: {:+.1} pp (adapter hurt — needs tuning)", delta);
    }
    println!();

    println!("━━━ Interpretation ━━━");
    println!();
    println!("  Run A (baseline) is pure token matching.");
    println!();
    println!("  Run B uses a local rule-based parser — strips stopwords,");
    println!("  classifies domain via keyword rules. No network required.");
    println!();
    if gemini_acc.is_some() {
        println!("  Run C uses the real Gemini 2.5 Flash API.");
        println!();
        println!("  Observed phenomenon (22.04.2026): local parser (B) often");
        println!("  OUTPERFORMS real LLM (C) on this benchmark. Reason:");
        println!("    - Our graph has only ~200 atoms (small vocabulary)");
        println!("    - Gemini returns rich, abstract entity names that");
        println!("      don't token-match the small graph");
        println!("    - Local parser's aggressive stopword stripping yields");
        println!("      cleaner, simpler seeds better suited to sparse graphs");
        println!();
        println!("  LESSON: LLM adapters help when graph is rich. For small");
        println!("  graphs, local parsing wins. The architecture must test");
        println!("  both per query and route to whichever scored higher.");
    } else {
        println!("  Set ZETS_GEMINI_KEY and re-run to see Run C comparison.");
    }
}

// ═══════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════

const KNOWLEDGE: &str = "\
Paris is the capital of France. \
Berlin is the capital of Germany. \
Madrid is the capital of Spain. \
Rome is the capital of Italy. \
Tokyo is the capital of Japan. \
London is the capital of England. \
Cairo is the capital of Egypt. \
Water contains hydrogen and oxygen. \
Gold has atomic number 79. \
Iron has atomic number 26. \
Carbon has atomic number 6. \
Dogs are mammals. Cats are mammals. Birds are animals. \
Fish live in water. Birds have feathers. Snakes are reptiles. \
Rust is a systems programming language. \
Python is an interpreted language. \
JavaScript runs in browsers. \
The sun is a star. The moon orbits Earth. \
Earth is a planet. Mars is a planet. \
Shakespeare wrote plays. \
Einstein discovered relativity. \
Newton discovered gravity. \
Photosynthesis converts light to energy. \
Gravity attracts objects. \
Atoms compose all matter. \
DNA stores genetic information. \
The heart pumps blood. \
Lungs process oxygen. \
Eyes perceive light. \
Ears detect sound.";

fn build_store() -> AtomStore {
    let mut store = AtomStore::new();
    bootstrap(&mut store);
    ingest_text(&mut store, "world-facts-v1", KNOWLEDGE, &IngestConfig::default());
    store
}

fn build_questions() -> Vec<Question> {
    vec![
        Question { id: "geo1".to_string(),
            text: "Which city is the capital of France?".to_string(),
            choices: vec!["Berlin".to_string(), "Paris".to_string(),
                          "Rome".to_string(), "Madrid".to_string()],
            expected: "B".to_string(), category: "geography".to_string() },
        Question { id: "geo2".to_string(),
            text: "What is the capital of Germany?".to_string(),
            choices: vec!["Paris".to_string(), "Berlin".to_string(),
                          "Rome".to_string(), "Madrid".to_string()],
            expected: "B".to_string(), category: "geography".to_string() },
        Question { id: "geo3".to_string(),
            text: "Tokyo is the capital of which country?".to_string(),
            choices: vec!["China".to_string(), "Korea".to_string(),
                          "Japan".to_string(), "Thailand".to_string()],
            expected: "C".to_string(), category: "geography".to_string() },
        Question { id: "geo4".to_string(),
            text: "What is capital of Italy?".to_string(),
            choices: vec!["Madrid".to_string(), "Athens".to_string(),
                          "Rome".to_string(), "Paris".to_string()],
            expected: "C".to_string(), category: "geography".to_string() },
        Question { id: "geo5".to_string(),
            text: "Cairo is the capital of which country?".to_string(),
            choices: vec!["Egypt".to_string(), "Libya".to_string(),
                          "Sudan".to_string(), "Israel".to_string()],
            expected: "A".to_string(), category: "geography".to_string() },
        Question { id: "bio1".to_string(),
            text: "What kind of animal is a dog?".to_string(),
            choices: vec!["Plants".to_string(), "Reptiles".to_string(),
                          "Mammals".to_string(), "Fish".to_string()],
            expected: "C".to_string(), category: "biology".to_string() },
        Question { id: "bio2".to_string(),
            text: "Where do fish live?".to_string(),
            choices: vec!["Trees".to_string(), "Water".to_string(),
                          "Clouds".to_string(), "Underground".to_string()],
            expected: "B".to_string(), category: "biology".to_string() },
        Question { id: "bio3".to_string(),
            text: "What do birds have?".to_string(),
            choices: vec!["Scales".to_string(), "Fur".to_string(),
                          "Feathers".to_string(), "Gills".to_string()],
            expected: "C".to_string(), category: "biology".to_string() },
        Question { id: "bio4".to_string(),
            text: "Snakes are what kind of animals?".to_string(),
            choices: vec!["Mammals".to_string(), "Birds".to_string(),
                          "Fish".to_string(), "Reptiles".to_string()],
            expected: "D".to_string(), category: "biology".to_string() },
        Question { id: "bio5".to_string(),
            text: "What stores genetic information?".to_string(),
            choices: vec!["RNA".to_string(), "Proteins".to_string(),
                          "DNA".to_string(), "Lipids".to_string()],
            expected: "C".to_string(), category: "biology".to_string() },
        Question { id: "chem1".to_string(),
            text: "What is gold's atomic number?".to_string(),
            choices: vec!["26".to_string(), "79".to_string(),
                          "6".to_string(), "92".to_string()],
            expected: "B".to_string(), category: "chemistry".to_string() },
        Question { id: "chem2".to_string(),
            text: "Water contains which elements?".to_string(),
            choices: vec!["Nitrogen carbon".to_string(),
                          "Hydrogen oxygen".to_string(),
                          "Helium neon".to_string(),
                          "Iron sulfur".to_string()],
            expected: "B".to_string(), category: "chemistry".to_string() },
        Question { id: "chem3".to_string(),
            text: "What compose all matter?".to_string(),
            choices: vec!["Cells".to_string(), "Organs".to_string(),
                          "Atoms".to_string(), "Waves".to_string()],
            expected: "C".to_string(), category: "chemistry".to_string() },
        Question { id: "cs1".to_string(),
            text: "What is Rust?".to_string(),
            choices: vec!["A metal".to_string(),
                          "A systems programming language".to_string(),
                          "A color".to_string(), "A plant".to_string()],
            expected: "B".to_string(), category: "cs".to_string() },
        Question { id: "cs2".to_string(),
            text: "What kind of language is Python?".to_string(),
            choices: vec!["Compiled".to_string(), "Interpreted".to_string(),
                          "Assembly".to_string(), "Machine".to_string()],
            expected: "B".to_string(), category: "cs".to_string() },
        Question { id: "cs3".to_string(),
            text: "Where does JavaScript run?".to_string(),
            choices: vec!["Browsers".to_string(), "Kernels".to_string(),
                          "Phones only".to_string(), "Mainframes".to_string()],
            expected: "A".to_string(), category: "cs".to_string() },
        Question { id: "cs4".to_string(),
            text: "What is carbon's atomic number?".to_string(),
            choices: vec!["79".to_string(), "26".to_string(),
                          "6".to_string(), "1".to_string()],
            expected: "C".to_string(), category: "chemistry".to_string() },
        Question { id: "astro1".to_string(),
            text: "What is the sun?".to_string(),
            choices: vec!["A planet".to_string(), "A moon".to_string(),
                          "A star".to_string(), "A comet".to_string()],
            expected: "C".to_string(), category: "astronomy".to_string() },
        Question { id: "astro2".to_string(),
            text: "What does gravity do?".to_string(),
            choices: vec!["Repels".to_string(), "Attracts".to_string(),
                          "Rotates".to_string(), "Glows".to_string()],
            expected: "B".to_string(), category: "physics".to_string() },
        Question { id: "hist1".to_string(),
            text: "Who discovered relativity?".to_string(),
            choices: vec!["Newton".to_string(), "Einstein".to_string(),
                          "Darwin".to_string(), "Galileo".to_string()],
            expected: "B".to_string(), category: "history".to_string() },
    ]
}
