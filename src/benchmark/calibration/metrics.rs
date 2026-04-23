//! Calibration metrics: ECE, Brier score, bucket statistics.

use crate::metacognition::Confidence;
use super::result::CalibrationResult;

/// Statistics for a single confidence bucket (e.g. [0.7, 0.8)).
#[derive(Debug, Clone)]
pub struct BucketStats {
    /// Lower bound (inclusive), e.g. 0.7.
    pub bucket_low: f32,
    /// Upper bound (exclusive), e.g. 0.8.
    pub bucket_high: f32,
    /// Number of results in this bucket.
    pub count: usize,
    /// Fraction of results in this bucket that were correct.
    pub accuracy: f32,
    /// Mean reported confidence of results in this bucket.
    pub avg_confidence: f32,
    /// This bucket's contribution to ECE = (count/N) * |accuracy - avg_confidence|.
    pub ece_contribution: f32,
}

impl BucketStats {
    /// Midpoint of the bucket range.
    pub fn midpoint(&self) -> f32 {
        (self.bucket_low + self.bucket_high) / 2.0
    }
}

/// Number of confidence buckets (deciles: 0-10%, 10-20%, …, 90-100%).
pub const NUM_BUCKETS: usize = 10;

/// Build bucket stats from a slice of results.
pub fn build_bucket_stats(results: &[CalibrationResult]) -> Vec<BucketStats> {
    let n = results.len();
    let mut buckets: Vec<Vec<&CalibrationResult>> = vec![Vec::new(); NUM_BUCKETS];

    for r in results {
        let c = r.confidence_f32().clamp(0.0, 1.0);
        // Map [0,1] → [0, NUM_BUCKETS-1]; clamp to last bucket for c == 1.0
        let idx = ((c * NUM_BUCKETS as f32) as usize).min(NUM_BUCKETS - 1);
        buckets[idx].push(r);
    }

    buckets
        .into_iter()
        .enumerate()
        .map(|(i, items)| {
            let bucket_low = i as f32 / NUM_BUCKETS as f32;
            let bucket_high = (i + 1) as f32 / NUM_BUCKETS as f32;
            let count = items.len();

            if count == 0 {
                return BucketStats {
                    bucket_low,
                    bucket_high,
                    count: 0,
                    accuracy: 0.0,
                    avg_confidence: 0.0,
                    ece_contribution: 0.0,
                };
            }

            let accuracy = items.iter().filter(|r| r.correct).count() as f32 / count as f32;
            let avg_confidence = items.iter().map(|r| r.confidence_f32()).sum::<f32>() / count as f32;
            let ece_contribution = if n > 0 {
                (count as f32 / n as f32) * (accuracy - avg_confidence).abs()
            } else {
                0.0
            };

            BucketStats {
                bucket_low,
                bucket_high,
                count,
                accuracy,
                avg_confidence,
                ece_contribution,
            }
        })
        .collect()
}

/// Expected Calibration Error — weighted mean of |accuracy - confidence| across buckets.
///
/// ECE = Σ_b (|B_b| / N) × |acc_b − conf_b|
///
/// Perfect calibration → ECE = 0.0.  Target: ECE < 0.10.
pub fn compute_ece(results: &[CalibrationResult]) -> f32 {
    if results.is_empty() {
        return 0.0;
    }
    build_bucket_stats(results)
        .iter()
        .map(|b| b.ece_contribution)
        .sum()
}

/// Brier score — mean squared error between confidence and binary outcome.
///
/// Brier = (1/N) Σ (confidence_i − outcome_i)²
///
/// Lower is better.  Perfect calibration on a hard task: ~0.0.
pub fn compute_brier_score(results: &[CalibrationResult]) -> f32 {
    if results.is_empty() {
        return 0.0;
    }
    results.iter().map(|r| r.brier_term()).sum::<f32>() / results.len() as f32
}

/// Map a `Confidence` level to a continuous value in [0.0, 1.0].
pub fn confidence_to_f32(c: Confidence) -> f32 {
    c.as_score() as f32 / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metacognition::Confidence;
    use crate::benchmark::calibration::result::{CalibrationResult, KnowInferGuess};

    fn make_result(confidence: Confidence, correct: bool) -> CalibrationResult {
        CalibrationResult {
            question_id: "q".to_string(),
            actual_answer: "a".to_string(),
            reported_confidence: confidence,
            confidence_tag: KnowInferGuess::Know,
            correct,
            refused: false,
            duration_ms: 0,
        }
    }

    /// 10 results at Moderate (0.5 confidence), 5 correct → ECE should be ~0.0
    #[test]
    fn test_ece_on_perfect_calibration() {
        let results: Vec<_> = (0..10)
            .map(|i| make_result(Confidence::Moderate, i < 5))
            .collect();
        let ece = compute_ece(&results);
        // All in bucket [0.4, 0.5]: accuracy=0.5, avg_conf=0.5 → contribution=0
        assert!(ece < 0.05, "ECE was {ece}, expected < 0.05");
    }

    /// 10 results at Certain (0.9 confidence), only 5 correct → ECE ≈ 0.4
    #[test]
    fn test_ece_on_overconfident() {
        let results: Vec<_> = (0..10)
            .map(|i| make_result(Confidence::Certain, i < 5))
            .collect();
        let ece = compute_ece(&results);
        // bucket [0.9,1.0]: accuracy=0.5, avg_conf=0.9 → contribution = 1.0 * 0.4 = 0.4
        assert!(
            (ece - 0.4).abs() < 0.05,
            "ECE was {ece}, expected ~0.4"
        );
    }

    /// 10 results at Unknown (0.1 confidence), all correct → under-confident
    #[test]
    fn test_ece_on_underconfident() {
        let results: Vec<_> = (0..10)
            .map(|_| make_result(Confidence::Unknown, true))
            .collect();
        let ece = compute_ece(&results);
        // bucket [0.0,0.1]: accuracy=1.0, avg_conf=0.1 → contribution = 0.9
        assert!(ece > 0.5, "ECE was {ece}, expected > 0.5");
    }

    /// Brier score with perfect predictions (Certain + correct).
    #[test]
    fn test_brier_on_perfect() {
        let results: Vec<_> = (0..10)
            .map(|_| make_result(Confidence::Certain, true))
            .collect();
        // Each term: (0.9 - 1.0)^2 = 0.01
        let brier = compute_brier_score(&results);
        assert!((brier - 0.01).abs() < 1e-5, "Brier was {brier}");
    }

    #[test]
    fn test_brier_on_always_wrong() {
        let results: Vec<_> = (0..10)
            .map(|_| make_result(Confidence::Certain, false))
            .collect();
        // Each term: (0.9 - 0.0)^2 = 0.81
        let brier = compute_brier_score(&results);
        assert!((brier - 0.81).abs() < 1e-5, "Brier was {brier}");
    }

    #[test]
    fn test_empty_ece_is_zero() {
        assert_eq!(compute_ece(&[]), 0.0);
    }

    #[test]
    fn test_empty_brier_is_zero() {
        assert_eq!(compute_brier_score(&[]), 0.0);
    }

    #[test]
    fn test_buckets_distribution() {
        // 5 Certain, 5 Unknown → two non-empty buckets
        let mut results: Vec<_> = (0..5)
            .map(|_| make_result(Confidence::Certain, true))
            .collect();
        results.extend((0..5).map(|_| make_result(Confidence::Unknown, false)));
        let buckets = build_bucket_stats(&results);
        let non_empty: Vec<_> = buckets.iter().filter(|b| b.count > 0).collect();
        assert_eq!(non_empty.len(), 2);
    }
}
