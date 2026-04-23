//! CalibrationReport — summary of a completed calibration run.

use std::collections::HashMap;
use std::fmt;
use super::metrics::BucketStats;

/// Per-category statistics within a calibration run.
#[derive(Debug, Clone)]
pub struct CategoryStats {
    /// Number of questions in this category.
    pub count: usize,
    /// Number of correct answers.
    pub correct: usize,
    /// Fraction correct.
    pub accuracy: f32,
    /// Mean ECE contribution from questions in this category.
    pub avg_confidence: f32,
}

impl CategoryStats {
    pub fn new(count: usize, correct: usize, avg_confidence: f32) -> Self {
        let accuracy = if count > 0 { correct as f32 / count as f32 } else { 0.0 };
        Self { count, correct, accuracy, avg_confidence }
    }
}

/// Full summary of a calibration session.
#[derive(Debug, Clone)]
pub struct CalibrationReport {
    /// Total questions in the harness.
    pub total_questions: usize,
    /// Questions for which a result was recorded.
    pub answered: usize,
    /// Correct answers.
    pub correct: usize,
    /// Refused answers.
    pub refused: usize,
    /// Expected Calibration Error (0 = perfect).
    pub ece: f32,
    /// Brier score (0 = perfect).
    pub brier: f32,
    /// Per-decile bucket breakdown.
    pub buckets: Vec<BucketStats>,
    /// Per semantic-category breakdown.
    pub per_category: HashMap<String, CategoryStats>,
    /// Fraction of trick questions correctly refused (0–1).
    pub trick_refusal_rate: f32,
    /// Know/Infer/Guess tagging accuracy (0–1).
    pub tag_accuracy: f32,
}

impl CalibrationReport {
    /// Overall accuracy = correct / answered (0 if nothing answered).
    pub fn accuracy(&self) -> f32 {
        if self.answered == 0 { 0.0 } else { self.correct as f32 / self.answered as f32 }
    }

    /// Pass criterion: ECE < 0.10.
    pub fn ece_passes(&self) -> bool {
        self.ece < 0.10
    }

    /// Textual summary suitable for logs.
    pub fn summary_line(&self) -> String {
        format!(
            "Calibration: {}/{} correct ({:.1}%), ECE={:.3}, Brier={:.3}, trick_refusal={:.1}%, tag_acc={:.1}%",
            self.correct,
            self.answered,
            self.accuracy() * 100.0,
            self.ece,
            self.brier,
            self.trick_refusal_rate * 100.0,
            self.tag_accuracy * 100.0,
        )
    }
}

impl fmt::Display for CalibrationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== Calibration Report ===")?;
        writeln!(f, "Questions : {}", self.total_questions)?;
        writeln!(f, "Answered  : {}", self.answered)?;
        writeln!(f, "Correct   : {} ({:.1}%)", self.correct, self.accuracy() * 100.0)?;
        writeln!(f, "Refused   : {}", self.refused)?;
        writeln!(f, "ECE       : {:.4} {}", self.ece, if self.ece_passes() { "✓" } else { "✗" })?;
        writeln!(f, "Brier     : {:.4}", self.brier)?;
        writeln!(f, "Trick ref.: {:.1}%", self.trick_refusal_rate * 100.0)?;
        writeln!(f, "Tag acc.  : {:.1}%", self.tag_accuracy * 100.0)?;
        writeln!(f)?;
        writeln!(f, "--- Bucket breakdown ---")?;
        for b in &self.buckets {
            if b.count > 0 {
                writeln!(
                    f,
                    "  [{:.1}-{:.1}]: n={:3}  acc={:.2}  conf={:.2}  ece_contrib={:.4}",
                    b.bucket_low, b.bucket_high, b.count, b.accuracy, b.avg_confidence, b.ece_contribution
                )?;
            }
        }
        writeln!(f)?;
        writeln!(f, "--- Per category ---")?;
        let mut cats: Vec<_> = self.per_category.iter().collect();
        cats.sort_by_key(|(k, _)| k.as_str());
        for (cat, stats) in cats {
            writeln!(
                f,
                "  {:30} n={:3}  acc={:.2}  avg_conf={:.2}",
                cat, stats.count, stats.accuracy, stats.avg_confidence
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_report(correct: usize, answered: usize) -> CalibrationReport {
        CalibrationReport {
            total_questions: 100,
            answered,
            correct,
            refused: 0,
            ece: 0.05,
            brier: 0.08,
            buckets: vec![],
            per_category: HashMap::new(),
            trick_refusal_rate: 1.0,
            tag_accuracy: 0.85,
        }
    }

    #[test]
    fn test_accuracy_computation() {
        let r = make_report(80, 100);
        assert!((r.accuracy() - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_accuracy_empty() {
        let r = make_report(0, 0);
        assert_eq!(r.accuracy(), 0.0);
    }

    #[test]
    fn test_ece_passes() {
        let r = make_report(80, 100);
        assert!(r.ece_passes()); // ece = 0.05 < 0.10
    }

    #[test]
    fn test_ece_fails() {
        let mut r = make_report(80, 100);
        r.ece = 0.15;
        assert!(!r.ece_passes());
    }

    #[test]
    fn test_report_display_runs() {
        let r = make_report(70, 100);
        let s = r.to_string();
        assert!(s.contains("Calibration Report"));
        assert!(s.contains("ECE"));
    }

    #[test]
    fn test_summary_line() {
        let r = make_report(90, 100);
        let line = r.summary_line();
        assert!(line.contains("90/100"));
    }
}
