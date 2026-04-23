//! CalibrationResult — the output side of a calibration test.

use crate::metacognition::Confidence;

/// ZETS's self-categorization of how it produced an answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowInferGuess {
    /// Directly recalled from stored knowledge.
    Know,
    /// Derived by reasoning from known facts.
    Infer,
    /// Uncertain — best guess with low confidence.
    Guess,
    /// Refused to answer (trick question or ethical concern).
    Refuse,
}

impl KnowInferGuess {
    pub fn as_str(self) -> &'static str {
        match self {
            KnowInferGuess::Know => "Know",
            KnowInferGuess::Infer => "Infer",
            KnowInferGuess::Guess => "Guess",
            KnowInferGuess::Refuse => "Refuse",
        }
    }
}

impl std::fmt::Display for KnowInferGuess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The recorded outcome of ZETS answering a single calibration question.
#[derive(Debug, Clone)]
pub struct CalibrationResult {
    /// ID of the `CalibrationQuestion` this corresponds to.
    pub question_id: String,
    /// The answer ZETS gave (empty string if it refused).
    pub actual_answer: String,
    /// Discrete confidence level reported by ZETS.
    pub reported_confidence: Confidence,
    /// How ZETS self-categorized its answer source.
    pub confidence_tag: KnowInferGuess,
    /// Whether the answer was judged correct against `ExpectedAnswer`.
    pub correct: bool,
    /// Whether ZETS chose to refuse the question.
    pub refused: bool,
    /// Wall-clock time to produce the answer.
    pub duration_ms: u64,
}

impl CalibrationResult {
    /// Numeric confidence in [0.0, 1.0] derived from the `Confidence` level.
    pub fn confidence_f32(&self) -> f32 {
        self.reported_confidence.as_score() as f32 / 100.0
    }

    /// Outcome as {0.0, 1.0} for Brier / ECE computation.
    pub fn outcome_f32(&self) -> f32 {
        if self.correct { 1.0 } else { 0.0 }
    }

    /// Squared error term for Brier score: (confidence - outcome)².
    pub fn brier_term(&self) -> f32 {
        let c = self.confidence_f32();
        let o = self.outcome_f32();
        (c - o).powi(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(confidence: Confidence, correct: bool) -> CalibrationResult {
        CalibrationResult {
            question_id: "q1".to_string(),
            actual_answer: "42".to_string(),
            reported_confidence: confidence,
            confidence_tag: KnowInferGuess::Know,
            correct,
            refused: false,
            duration_ms: 10,
        }
    }

    #[test]
    fn test_confidence_f32_certain() {
        let r = make_result(Confidence::Certain, true);
        assert!((r.confidence_f32() - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_confidence_f32_unknown() {
        let r = make_result(Confidence::Unknown, false);
        assert!((r.confidence_f32() - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_brier_term_perfect() {
        // confidence=0.9, correct=true → (0.9-1.0)^2 = 0.01
        let r = make_result(Confidence::Certain, true);
        assert!((r.brier_term() - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_brier_term_wrong() {
        // confidence=0.9, correct=false → (0.9-0.0)^2 = 0.81
        let r = make_result(Confidence::Certain, false);
        assert!((r.brier_term() - 0.81).abs() < 1e-6);
    }

    #[test]
    fn test_know_infer_guess_display() {
        assert_eq!(KnowInferGuess::Know.as_str(), "Know");
        assert_eq!(KnowInferGuess::Refuse.as_str(), "Refuse");
    }
}
