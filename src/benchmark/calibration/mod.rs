//! # Calibration Harness
//!
//! Measures how well ZETS's confidence aligns with its actual accuracy.
//!
//! ## Key metrics
//!
//! | Metric | Target | Meaning |
//! |--------|--------|---------|
//! | ECE    | < 0.10 | Expected Calibration Error |
//! | Brier  | < 0.20 | Mean squared probability error |
//! | Trick refusal | > 0.80 | Fraction of trick Qs correctly refused |
//! | Tag accuracy  | > 0.70 | Know/Infer/Guess self-label accuracy |
//!
//! ## Usage
//!
//! ```ignore
//! use zets::benchmark::calibration::{CalibrationHarness, CalibrationResult, KnowInferGuess};
//! use zets::metacognition::Confidence;
//!
//! let mut harness = CalibrationHarness::load_from_jsonl(
//!     "data/benchmark/calibration_set_easy.jsonl"
//! ).unwrap();
//!
//! harness.record(CalibrationResult {
//!     question_id: "easy_001".to_string(),
//!     actual_answer: "Paris".to_string(),
//!     reported_confidence: Confidence::Certain,
//!     confidence_tag: KnowInferGuess::Know,
//!     correct: true,
//!     refused: false,
//!     duration_ms: 42,
//! });
//!
//! let report = harness.report();
//! println!("{}", report);
//! ```

pub mod harness;
pub mod json;
pub mod metrics;
pub mod question;
pub mod report;
pub mod result;

pub use harness::{CalibrationError, CalibrationHarness};
pub use metrics::{BucketStats, build_bucket_stats, compute_brier_score, compute_ece};
pub use question::{CalibrationQuestion, Difficulty, ExpectedAnswer};
pub use report::{CalibrationReport, CategoryStats};
pub use result::{CalibrationResult, KnowInferGuess};
