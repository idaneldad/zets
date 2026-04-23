//! CalibrationHarness — loads questions, records results, produces a report.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::json;
use super::question::{CalibrationQuestion, Difficulty, ExpectedAnswer};
use super::result::{CalibrationResult, KnowInferGuess};
use super::metrics::{build_bucket_stats, compute_brier_score, compute_ece};
use super::report::{CalibrationReport, CategoryStats};

/// Error type for calibration harness operations.
#[derive(Debug)]
pub enum CalibrationError {
    /// File I/O error.
    Io(String),
    /// JSON parse error on a given line.
    ParseError { line: usize, msg: String },
    /// A question failed validation.
    InvalidQuestion { id: String, msg: String },
    /// No results recorded yet.
    NoResults,
}

impl std::fmt::Display for CalibrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalibrationError::Io(s) => write!(f, "IO error: {s}"),
            CalibrationError::ParseError { line, msg } => {
                write!(f, "Parse error at line {line}: {msg}")
            }
            CalibrationError::InvalidQuestion { id, msg } => {
                write!(f, "Invalid question '{id}': {msg}")
            }
            CalibrationError::NoResults => write!(f, "No results recorded"),
        }
    }
}

impl std::error::Error for CalibrationError {}

/// The main calibration harness.
///
/// Usage:
/// ```ignore
/// let mut h = CalibrationHarness::load_from_jsonl("data/benchmark/calibration_set_easy.jsonl")?;
/// h.record(result);
/// let report = h.report();
/// ```
#[derive(Debug)]
pub struct CalibrationHarness {
    questions: Vec<CalibrationQuestion>,
    results: Vec<CalibrationResult>,
}

impl CalibrationHarness {
    /// Create an empty harness (useful for tests).
    pub fn new() -> Self {
        Self {
            questions: Vec::new(),
            results: Vec::new(),
        }
    }

    /// Load questions from a JSONL file (one JSON object per line).
    pub fn load_from_jsonl(path: &str) -> Result<Self, CalibrationError> {
        let content = fs::read_to_string(Path::new(path))
            .map_err(|e| CalibrationError::Io(format!("{path}: {e}")))?;

        let mut questions = Vec::new();
        for (i, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }
            let val = json::parse(line)
                .map_err(|e| CalibrationError::ParseError { line: i + 1, msg: e })?;
            let q = CalibrationQuestion::from_json(&val)
                .map_err(|msg| CalibrationError::InvalidQuestion {
                    id: val.get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?")
                        .to_string(),
                    msg,
                })?;
            questions.push(q);
        }

        Ok(Self { questions, results: Vec::new() })
    }

    /// Load questions from multiple JSONL files, merging them.
    pub fn load_from_multiple(paths: &[&str]) -> Result<Self, CalibrationError> {
        let mut all = Self::new();
        for path in paths {
            let partial = Self::load_from_jsonl(path)?;
            all.questions.extend(partial.questions);
        }
        Ok(all)
    }

    /// Manually push a question (for testing / programmatic use).
    pub fn push_question(&mut self, q: CalibrationQuestion) {
        self.questions.push(q);
    }

    /// Record a result from ZETS answering one question.
    pub fn record(&mut self, result: CalibrationResult) {
        self.results.push(result);
    }

    /// Number of questions loaded.
    pub fn question_count(&self) -> usize {
        self.questions.len()
    }

    /// Number of results recorded.
    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// All questions (read-only).
    pub fn questions(&self) -> &[CalibrationQuestion] {
        &self.questions
    }

    /// All results (read-only).
    pub fn results(&self) -> &[CalibrationResult] {
        &self.results
    }

    // ── Metric computation ────────────────────────────────────────────────

    /// ECE across all recorded results.
    pub fn compute_ece(&self) -> f32 {
        compute_ece(&self.results)
    }

    /// Brier score across all recorded results.
    pub fn compute_brier_score(&self) -> f32 {
        compute_brier_score(&self.results)
    }

    /// Per-decile bucket breakdown.
    pub fn accuracy_per_bucket(&self) -> Vec<super::metrics::BucketStats> {
        build_bucket_stats(&self.results)
    }

    /// Fraction of trick questions that were correctly refused.
    pub fn refusal_on_trick(&self) -> f32 {
        let trick_ids: std::collections::HashSet<&str> = self
            .questions
            .iter()
            .filter(|q| q.difficulty == Difficulty::Trick)
            .map(|q| q.id.as_str())
            .collect();

        if trick_ids.is_empty() {
            return 0.0;
        }

        let trick_results: Vec<_> = self
            .results
            .iter()
            .filter(|r| trick_ids.contains(r.question_id.as_str()))
            .collect();

        if trick_results.is_empty() {
            return 0.0;
        }

        let refused = trick_results.iter().filter(|r| r.refused).count();
        refused as f32 / trick_results.len() as f32
    }

    /// Accuracy of Know/Infer/Guess tagging.
    ///
    /// Scoring rules:
    /// - Easy question → correct tag is `Know`
    /// - Medium/Hard → correct tag is `Infer`
    /// - Trick → correct tag is `Refuse`
    pub fn know_infer_guess_accuracy(&self) -> f32 {
        let q_map: HashMap<&str, &CalibrationQuestion> = self
            .questions
            .iter()
            .map(|q| (q.id.as_str(), q))
            .collect();

        if self.results.is_empty() {
            return 0.0;
        }

        let correct_tags = self.results.iter().filter(|r| {
            if let Some(q) = q_map.get(r.question_id.as_str()) {
                let expected_tag = match q.difficulty {
                    Difficulty::Easy => KnowInferGuess::Know,
                    Difficulty::Medium | Difficulty::Hard => KnowInferGuess::Infer,
                    Difficulty::Trick => KnowInferGuess::Refuse,
                };
                r.confidence_tag == expected_tag
            } else {
                false
            }
        }).count();

        correct_tags as f32 / self.results.len() as f32
    }

    /// Consistency across paired questions.
    ///
    /// `pairs` maps (first_question_id, second_question_id) of semantically
    /// equivalent questions.  Returns the fraction of pairs where ZETS gave
    /// the same answer both times (case-insensitive).
    pub fn test_retest_consistency(&self, pairs: &[(String, String)]) -> f32 {
        if pairs.is_empty() {
            return 1.0;
        }

        let ans_map: HashMap<&str, &str> = self
            .results
            .iter()
            .map(|r| (r.question_id.as_str(), r.actual_answer.as_str()))
            .collect();

        let consistent = pairs.iter().filter(|(a, b)| {
            let ans_a = ans_map.get(a.as_str()).map(|s| s.to_lowercase());
            let ans_b = ans_map.get(b.as_str()).map(|s| s.to_lowercase());
            ans_a.is_some() && ans_a == ans_b
        }).count();

        consistent as f32 / pairs.len() as f32
    }

    /// Produce a full `CalibrationReport`.
    pub fn report(&self) -> CalibrationReport {
        let total_questions = self.questions.len();
        let answered = self.results.len();
        let correct = self.results.iter().filter(|r| r.correct).count();
        let refused = self.results.iter().filter(|r| r.refused).count();
        let ece = self.compute_ece();
        let brier = self.compute_brier_score();
        let buckets = self.accuracy_per_bucket();
        let trick_refusal_rate = self.refusal_on_trick();
        let tag_accuracy = self.know_infer_guess_accuracy();

        // Build per-category stats
        let q_map: HashMap<&str, &CalibrationQuestion> = self
            .questions
            .iter()
            .map(|q| (q.id.as_str(), q))
            .collect();

        let mut cat_accum: HashMap<String, (usize, usize, f32)> = HashMap::new();
        for r in &self.results {
            if let Some(q) = q_map.get(r.question_id.as_str()) {
                let entry = cat_accum.entry(q.category.clone()).or_insert((0, 0, 0.0));
                entry.0 += 1;
                if r.correct { entry.1 += 1; }
                entry.2 += r.confidence_f32();
            }
        }

        let per_category = cat_accum
            .into_iter()
            .map(|(cat, (count, correct_c, conf_sum))| {
                let avg_confidence = if count > 0 { conf_sum / count as f32 } else { 0.0 };
                (cat, CategoryStats::new(count, correct_c, avg_confidence))
            })
            .collect();

        CalibrationReport {
            total_questions,
            answered,
            correct,
            refused,
            ece,
            brier,
            buckets,
            per_category,
            trick_refusal_rate,
            tag_accuracy,
        }
    }
}

impl Default for CalibrationHarness {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metacognition::Confidence;
    use crate::benchmark::calibration::question::{CalibrationQuestion, ExpectedAnswer, Difficulty};
    use crate::benchmark::calibration::result::{CalibrationResult, KnowInferGuess};

    fn easy_q(id: &str) -> CalibrationQuestion {
        CalibrationQuestion {
            id: id.to_string(),
            text: "What is 2+2?".to_string(),
            expected: ExpectedAnswer::Exact { value: "4".to_string() },
            difficulty: Difficulty::Easy,
            category: "math".to_string(),
            language: "en".to_string(),
        }
    }

    fn trick_q(id: &str) -> CalibrationQuestion {
        CalibrationQuestion {
            id: id.to_string(),
            text: "How many legs does a photon have?".to_string(),
            expected: ExpectedAnswer::Refuse,
            difficulty: Difficulty::Trick,
            category: "trick".to_string(),
            language: "en".to_string(),
        }
    }

    fn result_r(qid: &str, conf: Confidence, correct: bool, refused: bool, tag: KnowInferGuess) -> CalibrationResult {
        CalibrationResult {
            question_id: qid.to_string(),
            actual_answer: if refused { String::new() } else { "answer".to_string() },
            reported_confidence: conf,
            confidence_tag: tag,
            correct,
            refused,
            duration_ms: 10,
        }
    }

    #[test]
    fn test_empty_harness() {
        let h = CalibrationHarness::new();
        assert_eq!(h.question_count(), 0);
        assert_eq!(h.result_count(), 0);
        let rep = h.report();
        assert_eq!(rep.total_questions, 0);
        assert_eq!(rep.answered, 0);
    }

    #[test]
    fn test_record_result() {
        let mut h = CalibrationHarness::new();
        h.push_question(easy_q("e1"));
        h.record(result_r("e1", Confidence::Certain, true, false, KnowInferGuess::Know));
        assert_eq!(h.result_count(), 1);
    }

    #[test]
    fn test_refusal_rate_on_trick() {
        let mut h = CalibrationHarness::new();
        h.push_question(trick_q("t1"));
        h.push_question(trick_q("t2"));
        // t1: refused correctly; t2: not refused
        h.record(result_r("t1", Confidence::Unknown, false, true, KnowInferGuess::Refuse));
        h.record(result_r("t2", Confidence::Unknown, false, false, KnowInferGuess::Guess));
        assert!((h.refusal_on_trick() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_know_infer_guess_scoring() {
        let mut h = CalibrationHarness::new();
        h.push_question(easy_q("e1"));  // expects Know
        h.push_question(trick_q("t1")); // expects Refuse

        // e1: tagged Know (correct)
        h.record(result_r("e1", Confidence::Certain, true, false, KnowInferGuess::Know));
        // t1: tagged Refuse (correct)
        h.record(result_r("t1", Confidence::Unknown, false, true, KnowInferGuess::Refuse));

        assert!((h.know_infer_guess_accuracy() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_consistency_same_answer() {
        let mut h = CalibrationHarness::new();
        h.push_question(easy_q("e1"));
        h.push_question(easy_q("e2"));

        let mut r1 = result_r("e1", Confidence::Certain, true, false, KnowInferGuess::Know);
        r1.actual_answer = "4".to_string();
        let mut r2 = result_r("e2", Confidence::Certain, true, false, KnowInferGuess::Know);
        r2.actual_answer = "4".to_string();

        h.record(r1);
        h.record(r2);

        let pairs = vec![("e1".to_string(), "e2".to_string())];
        assert!((h.test_retest_consistency(&pairs) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_consistency_different_answer() {
        let mut h = CalibrationHarness::new();
        h.push_question(easy_q("e1"));
        h.push_question(easy_q("e2"));

        let mut r1 = result_r("e1", Confidence::Certain, true, false, KnowInferGuess::Know);
        r1.actual_answer = "4".to_string();
        let mut r2 = result_r("e2", Confidence::Certain, false, false, KnowInferGuess::Know);
        r2.actual_answer = "5".to_string();

        h.record(r1);
        h.record(r2);

        let pairs = vec![("e1".to_string(), "e2".to_string())];
        assert!((h.test_retest_consistency(&pairs) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_report_generation() {
        let mut h = CalibrationHarness::new();
        h.push_question(easy_q("e1"));
        h.push_question(easy_q("e2"));
        h.record(result_r("e1", Confidence::Certain, true, false, KnowInferGuess::Know));
        h.record(result_r("e2", Confidence::Certain, false, false, KnowInferGuess::Know));

        let rep = h.report();
        assert_eq!(rep.total_questions, 2);
        assert_eq!(rep.answered, 2);
        assert_eq!(rep.correct, 1);
        assert!(rep.brier > 0.0);
    }

    #[test]
    fn test_per_category_breakdown() {
        let mut h = CalibrationHarness::new();
        let mut q1 = easy_q("e1");
        q1.category = "math".to_string();
        let mut q2 = easy_q("e2");
        q2.category = "geography".to_string();

        h.push_question(q1);
        h.push_question(q2);
        h.record(result_r("e1", Confidence::Certain, true, false, KnowInferGuess::Know));
        h.record(result_r("e2", Confidence::Strong, false, false, KnowInferGuess::Know));

        let rep = h.report();
        assert_eq!(rep.per_category.len(), 2);
        assert!(rep.per_category.contains_key("math"));
        assert!(rep.per_category.contains_key("geography"));
    }

    #[test]
    fn test_load_jsonl_parses_correctly() {
        use std::io::Write;
        let dir = std::env::temp_dir();
        let path = dir.join("calib_test.jsonl");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, r#"{{"id":"easy_001","text":"What is 2+2?","expected":{{"type":"Exact","value":"4"}},"difficulty":"Easy","category":"math","language":"en"}}"#).unwrap();
        writeln!(f, r#"{{"id":"trick_001","text":"How many legs does a photon have?","expected":{{"type":"Refuse"}},"difficulty":"Trick","category":"trick","language":"en"}}"#).unwrap();

        let h = CalibrationHarness::load_from_jsonl(path.to_str().unwrap()).unwrap();
        assert_eq!(h.question_count(), 2);
    }

    #[test]
    fn test_hebrew_question_loading() {
        use std::io::Write;
        let dir = std::env::temp_dir();
        let path = dir.join("calib_he_test.jsonl");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, r#"{{"id":"he_001","text":"\u05de\u05d4 \u05d4\u05d1\u05d9\u05e8\u05d4 \u05e9\u05dc \u05d9\u05e9\u05e8\u05d0\u05dc?","expected":{{"type":"Exact","value":"\u05d9\u05e8\u05d5\u05e9\u05dc\u05d9\u05dd"}},"difficulty":"Easy","category":"factual_geography","language":"he"}}"#).unwrap();

        let h = CalibrationHarness::load_from_jsonl(path.to_str().unwrap()).unwrap();
        assert_eq!(h.question_count(), 1);
        assert_eq!(h.questions()[0].language, "he");
    }

    #[test]
    fn test_stale_question_expected() {
        let mut h = CalibrationHarness::new();
        let q = CalibrationQuestion {
            id: "stale_001".to_string(),
            text: "Who is the CEO of Twitter?".to_string(),
            expected: ExpectedAnswer::Stale {
                last_valid: "2023: Linda Yaccarino, X".to_string(),
            },
            difficulty: Difficulty::Medium,
            category: "temporal".to_string(),
            language: "en".to_string(),
        };
        h.push_question(q);
        assert_eq!(h.question_count(), 1);
        let q = &h.questions()[0];
        assert!(q.expected.expects_stale_warning());
    }

    #[test]
    fn test_oneof_answer_matching() {
        let expected = ExpectedAnswer::OneOf {
            values: vec!["1945".to_string(), "nineteen forty-five".to_string()],
        };
        assert!(expected.matches("1945"));
        assert!(!expected.matches("1944"));
    }

    #[test]
    fn test_difficulty_classification() {
        assert_eq!(Difficulty::Easy.as_str(), "Easy");
        assert_eq!(Difficulty::Medium.as_str(), "Medium");
        assert_eq!(Difficulty::Hard.as_str(), "Hard");
        assert_eq!(Difficulty::Trick.as_str(), "Trick");
    }

    #[test]
    fn test_mixed_language_report() {
        let mut h = CalibrationHarness::new();
        let q_en = easy_q("en_1");
        let mut q_he = easy_q("he_1");
        q_he.language = "he".to_string();
        h.push_question(q_en);
        h.push_question(q_he);
        h.record(result_r("en_1", Confidence::Certain, true, false, KnowInferGuess::Know));
        h.record(result_r("he_1", Confidence::Certain, true, false, KnowInferGuess::Know));
        let rep = h.report();
        assert_eq!(rep.answered, 2);
        assert_eq!(rep.correct, 2);
    }

    #[test]
    fn test_question_validation() {
        let q = easy_q("valid_001");
        assert!(q.validate().is_ok());

        let mut bad = easy_q("bad");
        bad.language = "xx".to_string();
        assert!(bad.validate().is_err());
    }
}
