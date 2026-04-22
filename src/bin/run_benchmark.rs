//! `run_benchmark` — first real baseline score for ZETS.
//!
//! 20 simple questions across 5 categories. We INGEST a knowledge base
//! first (so ZETS has atoms to reason over), then ask questions. No
//! NLU layer — so this is the HONEST baseline of what symbolic
//! graph-only reasoning can achieve.
//!
//! Expected result (rough prediction):
//!   - Relevance rate: ~80%+ (token matching works)
//!   - Accuracy: 20-40% (no NLU = can't parse question intent well)
//!   - Conditional accuracy (where relevant found): 25-50%

use zets::atoms::AtomStore;
use zets::benchmarks::{run_benchmark, Question};
use zets::bootstrap::bootstrap;
use zets::ingestion::{ingest_text, IngestConfig};
use zets::meta_learning::MetaLearner;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  ZETS Benchmark Run — 20 questions, baseline measurement  ║");
    println!("║  No NLU, no LLM. Pure symbolic graph reasoning.           ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let mut store = AtomStore::new();
    bootstrap(&mut store);

    // Seed with a reasonable knowledge base (~40 facts)
    let knowledge = "\
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
    let _ = ingest_text(&mut store, "world-facts-v1", knowledge, &IngestConfig::default());

    println!("Knowledge base: {} atoms, {} edges",
        store.atom_count(), store.edge_count());
    println!();

    // ═══════════════════════════════════════════════════
    // 20 questions — mixed multi-choice + free-text
    // ═══════════════════════════════════════════════════
    let questions: Vec<Question> = vec![
        // Geography (5)
        Question {
            id: "geo1".to_string(),
            text: "Which city is the capital of France?".to_string(),
            choices: vec!["Berlin".to_string(), "Paris".to_string(),
                          "Rome".to_string(), "Madrid".to_string()],
            expected: "B".to_string(),
            category: "geography".to_string(),
        },
        Question {
            id: "geo2".to_string(),
            text: "What is the capital of Germany?".to_string(),
            choices: vec!["Paris".to_string(), "Berlin".to_string(),
                          "Rome".to_string(), "Madrid".to_string()],
            expected: "B".to_string(),
            category: "geography".to_string(),
        },
        Question {
            id: "geo3".to_string(),
            text: "Which country's capital is Tokyo?".to_string(),
            choices: vec!["China".to_string(), "Korea".to_string(),
                          "Japan".to_string(), "Thailand".to_string()],
            expected: "C".to_string(),
            category: "geography".to_string(),
        },
        Question {
            id: "geo4".to_string(),
            text: "What is capital of Italy?".to_string(),
            choices: vec!["Madrid".to_string(), "Athens".to_string(),
                          "Rome".to_string(), "Paris".to_string()],
            expected: "C".to_string(),
            category: "geography".to_string(),
        },
        Question {
            id: "geo5".to_string(),
            text: "Cairo is the capital of which country?".to_string(),
            choices: vec!["Egypt".to_string(), "Libya".to_string(),
                          "Sudan".to_string(), "Israel".to_string()],
            expected: "A".to_string(),
            category: "geography".to_string(),
        },

        // Biology (5)
        Question {
            id: "bio1".to_string(),
            text: "What kind of animal is a dog?".to_string(),
            choices: vec!["Plants".to_string(), "Reptiles".to_string(),
                          "Mammals".to_string(), "Fish".to_string()],
            expected: "C".to_string(),
            category: "biology".to_string(),
        },
        Question {
            id: "bio2".to_string(),
            text: "Where do fish live?".to_string(),
            choices: vec!["Trees".to_string(), "Water".to_string(),
                          "Clouds".to_string(), "Underground".to_string()],
            expected: "B".to_string(),
            category: "biology".to_string(),
        },
        Question {
            id: "bio3".to_string(),
            text: "What do birds have?".to_string(),
            choices: vec!["Scales".to_string(), "Fur".to_string(),
                          "Feathers".to_string(), "Gills".to_string()],
            expected: "C".to_string(),
            category: "biology".to_string(),
        },
        Question {
            id: "bio4".to_string(),
            text: "Snakes are what kind of animals?".to_string(),
            choices: vec!["Mammals".to_string(), "Birds".to_string(),
                          "Fish".to_string(), "Reptiles".to_string()],
            expected: "D".to_string(),
            category: "biology".to_string(),
        },
        Question {
            id: "bio5".to_string(),
            text: "What stores genetic information?".to_string(),
            choices: vec!["RNA".to_string(), "Proteins".to_string(),
                          "DNA".to_string(), "Lipids".to_string()],
            expected: "C".to_string(),
            category: "biology".to_string(),
        },

        // Chemistry (3)
        Question {
            id: "chem1".to_string(),
            text: "What is gold's atomic number?".to_string(),
            choices: vec!["26".to_string(), "79".to_string(),
                          "6".to_string(), "92".to_string()],
            expected: "B".to_string(),
            category: "chemistry".to_string(),
        },
        Question {
            id: "chem2".to_string(),
            text: "Water contains which elements?".to_string(),
            choices: vec!["Nitrogen carbon".to_string(),
                          "Hydrogen oxygen".to_string(),
                          "Helium neon".to_string(),
                          "Iron sulfur".to_string()],
            expected: "B".to_string(),
            category: "chemistry".to_string(),
        },
        Question {
            id: "chem3".to_string(),
            text: "What compose all matter?".to_string(),
            choices: vec!["Cells".to_string(), "Organs".to_string(),
                          "Atoms".to_string(), "Waves".to_string()],
            expected: "C".to_string(),
            category: "chemistry".to_string(),
        },

        // CS (4)
        Question {
            id: "cs1".to_string(),
            text: "What is Rust?".to_string(),
            choices: vec!["A metal".to_string(),
                          "A systems programming language".to_string(),
                          "A color".to_string(), "A plant".to_string()],
            expected: "B".to_string(),
            category: "cs".to_string(),
        },
        Question {
            id: "cs2".to_string(),
            text: "What kind of language is Python?".to_string(),
            choices: vec!["Compiled".to_string(), "Interpreted".to_string(),
                          "Assembly".to_string(), "Machine".to_string()],
            expected: "B".to_string(),
            category: "cs".to_string(),
        },
        Question {
            id: "cs3".to_string(),
            text: "Where does JavaScript run?".to_string(),
            choices: vec!["Browsers".to_string(), "Kernels".to_string(),
                          "Phones only".to_string(), "Mainframes".to_string()],
            expected: "A".to_string(),
            category: "cs".to_string(),
        },
        Question {
            id: "cs4".to_string(),
            text: "What is carbon's atomic number?".to_string(),
            choices: vec!["79".to_string(), "26".to_string(),
                          "6".to_string(), "1".to_string()],
            expected: "C".to_string(),
            category: "chemistry".to_string(),
        },

        // Physics / Astronomy (3)
        Question {
            id: "astro1".to_string(),
            text: "What is the sun?".to_string(),
            choices: vec!["A planet".to_string(), "A moon".to_string(),
                          "A star".to_string(), "A comet".to_string()],
            expected: "C".to_string(),
            category: "astronomy".to_string(),
        },
        Question {
            id: "astro2".to_string(),
            text: "What does gravity do?".to_string(),
            choices: vec!["Repels".to_string(), "Attracts".to_string(),
                          "Rotates".to_string(), "Glows".to_string()],
            expected: "B".to_string(),
            category: "physics".to_string(),
        },
        Question {
            id: "hist1".to_string(),
            text: "Who discovered relativity?".to_string(),
            choices: vec!["Newton".to_string(), "Einstein".to_string(),
                          "Darwin".to_string(), "Galileo".to_string()],
            expected: "B".to_string(),
            category: "history".to_string(),
        },
    ];

    println!("Running {} questions...", questions.len());
    println!();
    let mut meta = MetaLearner::new();
    let start = std::time::Instant::now();
    let score = run_benchmark(&mut store, &mut meta, &questions);
    let elapsed = start.elapsed();

    println!("━━━ Results ━━━");
    println!("Time: {:?}", elapsed);
    println!("  Total:              {}", score.total);
    println!("  Correct:            {}", score.correct);
    println!("  Had relevant atoms: {}", score.had_relevant);
    println!();
    println!("  Accuracy:              {:.1}%", score.accuracy() * 100.0);
    println!("  Relevance rate:        {:.1}%", score.relevance_rate() * 100.0);
    println!("  Conditional accuracy:  {:.1}%  (among questions with relevant atoms)",
        score.conditional_accuracy() * 100.0);
    println!();

    println!("━━━ By category ━━━");
    for (cat, acc) in score.category_breakdown() {
        println!("  {:<12} {:.1}%", cat, acc * 100.0);
    }
    println!();

    println!("━━━ Sample results ━━━");
    for r in score.results.iter().take(5) {
        let mark = if r.correct { "✓" } else { "✗" };
        println!("  {} {:<8} predicted={}  mode={}  candidates={}  relevant={}",
            mark, r.question_id, r.predicted, r.mode_used,
            r.candidate_count, r.had_relevant_atoms);
        if !r.top_candidates.is_empty() {
            println!("             top: {}", r.top_candidates[0]);
        }
    }
    println!();

    println!("━━━ Baseline interpretation ━━━");
    println!();
    println!("This is ZETS's HONEST baseline with no NLU layer.");
    println!();
    println!("What to expect given the architecture:");
    println!("  - Relevance rate should be HIGH — token matching just works.");
    println!("  - Accuracy is capped by how well spreading-activation from");
    println!("    question-token seeds reaches the correct answer's atoms.");
    println!("  - Conditional accuracy (when relevant atoms found) tells us:");
    println!("    how good is the reasoning step ALONE.");
    println!();
    println!("If conditional_accuracy > 50%, the reasoning core is solid;");
    println!("the gap to top LLMs is purely in NLU + knowledge scale.");
    println!();
    println!("If conditional_accuracy < 30%, we need deeper reasoning, not");
    println!("just more knowledge.");
}
