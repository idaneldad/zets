//! Benchmark framework — measure ZETS on standard question-answering tasks.
//!
//! Until now, ZETS had plenty of unit tests but no way to answer
//! 'how well does this actually perform on real benchmark questions?'
//!
//! This module provides:
//!   1. Question/Expected/Answer types
//!   2. Simple multi-choice scorer (exact match and fuzzy match)
//!   3. Runner that takes a batch of questions and returns a Score
//!   4. JSONL format support (standard for MMLU/HLE/GPQA)
//!
//! Design: deterministic. Same store + same questions = same score, always.
//! This is our moat: reproducible evaluation without seed variance.

use std::collections::HashMap;

use crate::atoms::{AtomId, AtomStore};
use crate::session::SessionContext;
use crate::smart_walk::{smart_walk, WalkResult};
use crate::meta_learning::MetaLearner;

/// A single benchmark question.
#[derive(Debug, Clone)]
pub struct Question {
    pub id: String,
    /// The question text
    pub text: String,
    /// Optional multi-choice options (A, B, C, D, ...)
    pub choices: Vec<String>,
    /// The correct answer — either a letter (A/B/...) or free text
    pub expected: String,
    /// Optional context category (factual, creative, emotional...)
    pub category: String,
}

/// One question's result after running through ZETS.
#[derive(Debug, Clone)]
pub struct QuestionResult {
    pub question_id: String,
    pub predicted: String,
    pub correct: bool,
    /// Did ZETS have any atoms relevant to answer?
    pub had_relevant_atoms: bool,
    /// Which cognitive mode was chosen
    pub mode_used: String,
    /// How many candidates spreading found
    pub candidate_count: usize,
    /// Raw top-3 atom labels for audit
    pub top_candidates: Vec<String>,
}

/// Aggregate score over a batch of questions.
#[derive(Debug, Clone, Default)]
pub struct BenchScore {
    pub total: usize,
    pub correct: usize,
    pub had_relevant: usize,
    /// Breakdown by category
    pub by_category: HashMap<String, (usize, usize)>,  // (correct, total)
    pub results: Vec<QuestionResult>,
}

impl BenchScore {
    pub fn accuracy(&self) -> f32 {
        if self.total == 0 { return 0.0; }
        self.correct as f32 / self.total as f32
    }

    pub fn relevance_rate(&self) -> f32 {
        if self.total == 0 { return 0.0; }
        self.had_relevant as f32 / self.total as f32
    }

    /// Of the questions where ZETS HAD relevant atoms, what % got right?
    /// This isolates reasoning ability from knowledge gap.
    pub fn conditional_accuracy(&self) -> f32 {
        if self.had_relevant == 0 { return 0.0; }
        let correct_with_relevant = self.results.iter()
            .filter(|r| r.had_relevant_atoms && r.correct)
            .count();
        correct_with_relevant as f32 / self.had_relevant as f32
    }

    pub fn category_breakdown(&self) -> Vec<(String, f32)> {
        let mut v: Vec<(String, f32)> = self.by_category.iter()
            .map(|(cat, (c, t))| (cat.clone(), *c as f32 / *t as f32))
            .collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        v
    }
}

/// Simple answer-matching strategies.
pub fn exact_match(predicted: &str, expected: &str) -> bool {
    predicted.trim().eq_ignore_ascii_case(expected.trim())
}

pub fn contains_match(predicted: &str, expected: &str) -> bool {
    predicted.to_lowercase().contains(&expected.to_lowercase())
}

/// For multi-choice: check if predicted starts with the letter (e.g., "A" or "A.")
pub fn letter_match(predicted: &str, expected: &str) -> bool {
    let pred_trimmed = predicted.trim_start();
    let expected_trimmed = expected.trim();
    if pred_trimmed.is_empty() || expected_trimmed.is_empty() { return false; }
    pred_trimmed.chars().next().map(|c| c.to_ascii_uppercase())
        == expected_trimmed.chars().next().map(|c| c.to_ascii_uppercase())
}

/// Find atoms in the store that contain any of the tokens from the question.
/// Returns the atom_ids of matching atoms, useful as session seeds.
pub fn find_relevant_atoms(store: &AtomStore, text: &str, max: usize) -> Vec<AtomId> {
    let lower = text.to_lowercase();
    let tokens: Vec<String> = lower.split_whitespace()
        .filter(|w| w.len() >= 3)
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| !w.is_empty())
        .collect();

    let (atoms, _) = store.snapshot();
    let mut matched: Vec<AtomId> = Vec::new();

    for (atom_id, atom) in atoms.iter().enumerate() {
        let data_str = match std::str::from_utf8(&atom.data) {
            Ok(s) => s.to_lowercase(),
            Err(_) => continue,
        };
        for token in &tokens {
            if data_str.contains(token) {
                matched.push(atom_id as AtomId);
                break;
            }
        }
        if matched.len() >= max { break; }
    }
    matched
}

/// Answer one question using ZETS.
///
/// Strategy (deterministic, no LLM):
///   1. Tokenize question, find atoms mentioning those tokens
///   2. Activate them in session
///   3. smart_walk to find related atoms
///   4. If multi-choice: score each choice by how much it matches the
///      top spreading-activation results
///   5. If free-text: return the top candidate's label
pub fn answer_question(
    store: &mut AtomStore,
    meta: &mut MetaLearner,
    question: &Question,
) -> QuestionResult {
    let mut session = SessionContext::new();

    // Seed session with atoms matching question tokens
    let seeds = find_relevant_atoms(store, &question.text, 10);
    let had_relevant = !seeds.is_empty();
    for seed in &seeds {
        session.mention(*seed);
    }
    session.advance_turn();

    // Run smart_walk
    let walk: WalkResult = smart_walk(store, &session, meta,
        &question.text, &question.category, 10);

    // Harvest top candidate labels
    let top_candidates: Vec<String> = walk.candidates.iter()
        .take(3)
        .filter_map(|(aid, _)| {
            store.get(*aid)
                .and_then(|a| std::str::from_utf8(&a.data).ok().map(String::from))
        })
        .collect();

    // Prediction logic:
    //   - Multi-choice: score each choice by how many top candidates
    //     share tokens with that choice. Pick highest.
    //   - Free-text: return the top candidate's label (stripped of prefix).
    let predicted = if !question.choices.is_empty() {
        predict_multichoice(&walk, &question.choices, store)
    } else {
        predict_free_text(&walk, store)
    };

    let correct = if !question.choices.is_empty() {
        letter_match(&predicted, &question.expected)
    } else {
        exact_match(&predicted, &question.expected)
            || contains_match(&predicted, &question.expected)
    };

    QuestionResult {
        question_id: question.id.clone(),
        predicted,
        correct,
        had_relevant_atoms: had_relevant,
        mode_used: walk.mode_used.label().to_string(),
        candidate_count: walk.candidates.len(),
        top_candidates,
    }
}

/// Multi-choice prediction: score each choice by overlap with top candidates.
fn predict_multichoice(walk: &WalkResult, choices: &[String], store: &AtomStore) -> String {
    // Harvest text of top candidates, filtering out meta/hub atoms that dominate
    // spreading activation without carrying discriminative content.
    // Ignored prefixes: 'sent:' (sentence hubs), 'source:', 'utt:',
    //                   'zets:bootstrap:' (infrastructure atoms)
    // The 'word:' prefix is KEPT but its value is the word after the prefix.
    let is_meta = |text: &str| -> bool {
        text.starts_with("sent:")
            || text.starts_with("source:")
            || text.starts_with("utt:")
            || text.starts_with("zets:bootstrap:")
            || text.starts_with("category:")
    };

    // Collect candidate labels, expanding 'word:X' to just 'X' for matching.
    // Also follow sentence atoms to their content via outgoing edges if we
    // need more candidates after filtering.
    let mut candidate_labels: Vec<String> = Vec::new();
    for (aid, _score) in walk.candidates.iter() {
        if candidate_labels.len() >= 10 { break; }
        if let Some(atom) = store.get(*aid) {
            if let Ok(label) = std::str::from_utf8(&atom.data) {
                if !is_meta(label) {
                    // Strip 'word:' prefix if present
                    let clean = label.strip_prefix("word:").unwrap_or(label);
                    candidate_labels.push(clean.to_string());
                }
                // If it's a sentence, harvest its outgoing neighbors too
                if label.starts_with("sent:") {
                    for edge in store.outgoing(*aid).iter().take(8) {
                        if let Some(n) = store.get(edge.to) {
                            if let Ok(nlabel) = std::str::from_utf8(&n.data) {
                                if !is_meta(nlabel) {
                                    let clean = nlabel.strip_prefix("word:").unwrap_or(nlabel);
                                    candidate_labels.push(clean.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let combined = candidate_labels.join(" ").to_lowercase();

    // Score each choice by how many of its words appear in combined
    let mut best_idx = 0usize;
    let mut best_score = -1i32;
    for (i, choice) in choices.iter().enumerate() {
        let score: i32 = choice.split_whitespace()
            .filter(|w| w.len() >= 3)
            .map(|w| if combined.contains(&w.to_lowercase()) { 1 } else { 0 })
            .sum();
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }

    // Return the letter
    let letter = (b'A' + best_idx as u8) as char;
    format!("{}", letter)
}

/// Free-text prediction: return top candidate's label.
fn predict_free_text(walk: &WalkResult, store: &AtomStore) -> String {
    walk.candidates.first()
        .and_then(|(aid, _)| store.get(*aid))
        .and_then(|a| std::str::from_utf8(&a.data).ok().map(String::from))
        .unwrap_or_default()
}

/// Run a batch of questions, collect scores.
pub fn run_benchmark(
    store: &mut AtomStore,
    meta: &mut MetaLearner,
    questions: &[Question],
) -> BenchScore {
    let mut score = BenchScore::default();

    for q in questions {
        let result = answer_question(store, meta, q);
        if result.correct { score.correct += 1; }
        if result.had_relevant_atoms { score.had_relevant += 1; }
        score.total += 1;

        let entry = score.by_category.entry(q.category.clone()).or_insert((0, 0));
        entry.1 += 1;
        if result.correct { entry.0 += 1; }

        score.results.push(result);
    }

    score
}

// ────────────────────────────────────────────────────────────────
// JSONL loader — load questions from data/benchmarks/*.jsonl
// ────────────────────────────────────────────────────────────────

/// Load a benchmark question set from JSONL (one JSON object per line).
///
/// Expected fields per line:
///   id: string
///   text: string
///   choices: array of strings (optional, empty for free-text)
///   expected: string (letter A-D for multi-choice, else free text)
///   category: string
///
/// The parser is deliberately minimal — no serde dependency. It handles
/// the specific format we use, nothing fancier. If the file includes
/// trailing whitespace or empty lines, those are skipped.
pub fn load_jsonl(path: &std::path::Path) -> std::io::Result<Vec<Question>> {
    let content = std::fs::read_to_string(path)?;
    let mut questions = Vec::new();
    for (line_num, raw) in content.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() { continue; }
        match parse_question_line(line) {
            Ok(q) => questions.push(q),
            Err(e) => return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("line {}: {}", line_num + 1, e))),
        }
    }
    Ok(questions)
}

fn parse_question_line(line: &str) -> Result<Question, String> {
    // Minimal JSON object parser — handles string/array/flat fields.
    // Not a general-purpose JSON parser. Good enough for our JSONL schema.
    let trimmed = line.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Err("not a JSON object".to_string());
    }

    let mut q = Question {
        id: String::new(),
        text: String::new(),
        choices: Vec::new(),
        expected: String::new(),
        category: String::new(),
    };

    // Extract each field by searching for its key
    q.id = extract_string_field(trimmed, "id")?;
    q.text = extract_string_field(trimmed, "text")?;
    q.expected = extract_string_field(trimmed, "expected")?;
    q.category = extract_string_field(trimmed, "category")?;
    q.choices = extract_array_field(trimmed, "choices")?;

    Ok(q)
}

fn extract_string_field(json: &str, key: &str) -> Result<String, String> {
    let needle = format!("\"{}\":", key);
    let start = json.find(&needle)
        .ok_or_else(|| format!("missing field '{}'", key))?;
    let after_colon = &json[start + needle.len()..];
    let trimmed = after_colon.trim_start();
    if !trimmed.starts_with('"') {
        return Err(format!("field '{}' not a string", key));
    }
    // Find closing quote, respecting simple escapes
    let body = &trimmed[1..];
    let mut result = String::new();
    let mut escape = false;
    for ch in body.chars() {
        if escape {
            match ch {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                _ => result.push(ch),
            }
            escape = false;
            continue;
        }
        if ch == '\\' { escape = true; continue; }
        if ch == '"' { return Ok(result); }
        result.push(ch);
    }
    Err(format!("unterminated string for field '{}'", key))
}

fn extract_array_field(json: &str, key: &str) -> Result<Vec<String>, String> {
    let needle = format!("\"{}\":", key);
    let start = match json.find(&needle) {
        Some(p) => p,
        None => return Ok(Vec::new()),  // optional field
    };
    let after_colon = &json[start + needle.len()..];
    let trimmed = after_colon.trim_start();
    if !trimmed.starts_with('[') {
        return Err(format!("field '{}' not an array", key));
    }
    // Find matching ]
    let body_start = 1;
    let mut depth = 1;
    let mut end = 0;
    for (i, ch) in trimmed.chars().enumerate().skip(1) {
        match ch {
            '[' => depth += 1,
            ']' => { depth -= 1; if depth == 0 { end = i; break; } },
            _ => {}
        }
    }
    if end == 0 {
        return Err(format!("unterminated array for field '{}'", key));
    }
    let body = &trimmed[body_start..end];

    // Parse comma-separated strings inside
    let mut items = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escape = false;
    for ch in body.chars() {
        if escape {
            match ch {
                'n' => current.push('\n'),
                't' => current.push('\t'),
                '\\' => current.push('\\'),
                '"' => current.push('"'),
                _ => current.push(ch),
            }
            escape = false;
            continue;
        }
        if ch == '\\' { escape = true; continue; }
        if ch == '"' {
            if in_string {
                items.push(current.clone());
                current.clear();
            }
            in_string = !in_string;
            continue;
        }
        if in_string { current.push(ch); }
    }
    Ok(items)
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingestion::{ingest_text, IngestConfig};

    fn knowledgeable_store() -> AtomStore {
        let mut store = AtomStore::new();
        let config = IngestConfig::default();

        let text = "\
            Paris is the capital of France. \
            Berlin is the capital of Germany. \
            Madrid is the capital of Spain. \
            Rome is the capital of Italy. \
            Tokyo is the capital of Japan. \
            Water is composed of hydrogen and oxygen. \
            Gold has atomic number 79. \
            Dogs are mammals. Cats are mammals. Birds are animals. \
            Rust is a systems programming language. Python is interpreted.";
        ingest_text(&mut store, "facts", text, &config);
        store
    }

    #[test]
    fn free_text_simple_recall() {
        let mut store = knowledgeable_store();
        let mut meta = MetaLearner::new();
        let q = Question {
            id: "q1".to_string(),
            text: "What is the capital of France?".to_string(),
            choices: vec![],
            expected: "paris".to_string(),
            category: "geography".to_string(),
        };
        let result = answer_question(&mut store, &mut meta, &q);
        // We don't require correctness (no NLU), but we do require relevant atoms
        assert!(result.had_relevant_atoms, "should find atoms for 'France'");
    }

    #[test]
    fn multichoice_scoring() {
        let mut store = knowledgeable_store();
        let mut meta = MetaLearner::new();
        let q = Question {
            id: "q2".to_string(),
            text: "What are dogs?".to_string(),
            choices: vec![
                "Plants".to_string(),
                "Fish".to_string(),
                "Mammals".to_string(),
                "Insects".to_string(),
            ],
            expected: "C".to_string(),  // mammals
            category: "biology".to_string(),
        };
        let result = answer_question(&mut store, &mut meta, &q);
        assert!(result.had_relevant_atoms);
        // The predicted should be a letter A-D
        assert!(result.predicted.len() >= 1);
        let ch = result.predicted.chars().next().unwrap();
        assert!(ch >= 'A' && ch <= 'D', "predicted should be A-D, got {}", result.predicted);
    }

    #[test]
    fn no_relevant_atoms_returns_false_relevance() {
        let mut store = knowledgeable_store();
        let mut meta = MetaLearner::new();
        let q = Question {
            id: "q3".to_string(),
            text: "Quantum chromodynamics phenomenology regarding hadrons?".to_string(),
            choices: vec![],
            expected: "anything".to_string(),
            category: "physics".to_string(),
        };
        let result = answer_question(&mut store, &mut meta, &q);
        // Our store has nothing about quantum chromodynamics
        assert!(!result.had_relevant_atoms,
            "should NOT find relevant atoms for unknown topic");
    }

    #[test]
    fn run_benchmark_aggregates_scores() {
        let mut store = knowledgeable_store();
        let mut meta = MetaLearner::new();
        let qs = vec![
            Question {
                id: "1".to_string(),
                text: "What is Rust?".to_string(),
                choices: vec![
                    "A language".to_string(),
                    "A metal".to_string(),
                    "A color".to_string(),
                    "A car".to_string(),
                ],
                expected: "A".to_string(),
                category: "cs".to_string(),
            },
            Question {
                id: "2".to_string(),
                text: "What are dogs?".to_string(),
                choices: vec![
                    "Plants".to_string(),
                    "Fish".to_string(),
                    "Mammals".to_string(),
                    "Insects".to_string(),
                ],
                expected: "C".to_string(),
                category: "biology".to_string(),
            },
        ];
        let score = run_benchmark(&mut store, &mut meta, &qs);
        assert_eq!(score.total, 2);
        // Relevance rate should be 100% — both topics are in the store
        assert_eq!(score.had_relevant, 2);
        // We make no claims about correctness — it's a baseline
    }

    #[test]
    fn find_relevant_atoms_returns_matches() {
        let mut store = AtomStore::new();
        let config = IngestConfig::default();
        ingest_text(&mut store, "test",
            "Apples are red fruits. Bananas are yellow.", &config);
        let matches = find_relevant_atoms(&store, "What color are apples?", 10);
        assert!(!matches.is_empty(), "should find atoms mentioning 'apples'");
    }

    #[test]
    fn exact_match_ignores_case() {
        assert!(exact_match("Paris", "paris"));
        assert!(exact_match("  Paris  ", "paris"));
        assert!(!exact_match("Berlin", "paris"));
    }

    #[test]
    fn letter_match_basic() {
        assert!(letter_match("A", "A"));
        assert!(letter_match("A.", "A"));
        assert!(letter_match("A) Plants", "A"));
        assert!(!letter_match("B", "A"));
    }

    #[test]
    fn contains_match_substring() {
        assert!(contains_match("Paris is the capital", "paris"));
        assert!(contains_match("The capital is Paris.", "paris"));
        assert!(!contains_match("Berlin is nice", "paris"));
    }

    #[test]
    fn accuracy_calculations() {
        let mut score = BenchScore::default();
        score.total = 10;
        score.correct = 4;
        score.had_relevant = 7;
        // conditional_accuracy requires results — compute manually for test
        assert_eq!(score.accuracy(), 0.4);
        assert_eq!(score.relevance_rate(), 0.7);
    }

    #[test]
    fn empty_score_zero_accuracy() {
        let score = BenchScore::default();
        assert_eq!(score.accuracy(), 0.0);
        assert_eq!(score.relevance_rate(), 0.0);
    }

    #[test]
    fn load_jsonl_parses_20q_file() {
        let path = std::path::Path::new("data/benchmarks/zets_baseline_20q_v1.jsonl");
        if !path.exists() {
            eprintln!("skipping — run `snapshot bootstrap-default` to create baseline data");
            return;
        }
        let questions = load_jsonl(path).expect("load 20q JSONL");
        assert_eq!(questions.len(), 20, "expected 20 questions");
        // Spot-check first question
        assert_eq!(questions[0].id, "geo1");
        assert_eq!(questions[0].choices.len(), 4);
        assert_eq!(questions[0].expected, "B");
        assert_eq!(questions[0].category, "geography");
    }

    #[test]
    fn parse_question_line_handles_basic() {
        let line = r#"{"id":"t1","text":"What?","choices":["A","B","C","D"],"expected":"A","category":"test"}"#;
        let q = parse_question_line(line).unwrap();
        assert_eq!(q.id, "t1");
        assert_eq!(q.text, "What?");
        assert_eq!(q.choices, vec!["A", "B", "C", "D"]);
        assert_eq!(q.expected, "A");
        assert_eq!(q.category, "test");
    }

    #[test]
    fn parse_question_line_handles_empty_choices() {
        let line = r#"{"id":"t2","text":"Free text","choices":[],"expected":"Paris","category":"geo"}"#;
        let q = parse_question_line(line).unwrap();
        assert_eq!(q.choices.len(), 0);
        assert_eq!(q.expected, "Paris");
    }

    #[test]
    fn parse_question_line_rejects_malformed() {
        let malformed = r#"not json"#;
        assert!(parse_question_line(malformed).is_err());
    }

    #[test]
    fn extract_string_field_finds_value() {
        let json = r#"{"key":"value","other":"x"}"#;
        assert_eq!(extract_string_field(json, "key").unwrap(), "value");
        assert_eq!(extract_string_field(json, "other").unwrap(), "x");
    }

    #[test]
    fn extract_array_field_parses_strings() {
        let json = r#"{"items":["a","b","c"]}"#;
        assert_eq!(extract_array_field(json, "items").unwrap(),
                   vec!["a", "b", "c"]);
    }

    #[test]
    fn extract_array_field_empty() {
        let json = r#"{"items":[]}"#;
        assert_eq!(extract_array_field(json, "items").unwrap().len(), 0);
    }


    #[test]
    fn determinism_same_questions_same_score() {
        let mut s1 = knowledgeable_store();
        let mut s2 = knowledgeable_store();
        let mut m1 = MetaLearner::new();
        let mut m2 = MetaLearner::new();
        let qs = vec![Question {
            id: "1".to_string(),
            text: "What is Rust?".to_string(),
            choices: vec!["A".to_string(), "B".to_string(),
                          "C".to_string(), "D".to_string()],
            expected: "A".to_string(),
            category: "cs".to_string(),
        }];
        let s1_score = run_benchmark(&mut s1, &mut m1, &qs);
        let s2_score = run_benchmark(&mut s2, &mut m2, &qs);
        assert_eq!(s1_score.total, s2_score.total);
        assert_eq!(s1_score.correct, s2_score.correct);
        assert_eq!(s1_score.had_relevant, s2_score.had_relevant);
    }
}
