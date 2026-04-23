//! # Score — HumannessScore computation
//!
//! Aggregates test results into a single score (0..1) representing
//! overall ZETS capability vs the benchmark spec.

use super::spec::{BenchmarkSpec, Tier};

/// Result of running a single test.
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_id: String,
    /// Achieved score (0..1) — what the test actually produced.
    pub achieved: f32,
    /// Target from the spec (0..1).
    pub target: f32,
    /// How confident we are in this score (0..1) — for tests with small n.
    pub confidence: f32,
    /// Did this test pass? `achieved >= target`.
    pub passed: bool,
    /// When the test ran (Unix ms).
    pub run_at_ms: i64,
    /// Optional metadata — eval notes, run duration, cost.
    pub notes: Option<String>,
}

impl TestResult {
    pub fn pass(test_id: &str, achieved: f32, target: f32) -> Self {
        let clamped = achieved.clamp(0.0, 1.0);
        TestResult {
            test_id: test_id.into(),
            achieved: clamped,
            target,
            confidence: 1.0,
            passed: clamped >= target,
            run_at_ms: 0,
            notes: None,
        }
    }

    pub fn with_confidence(mut self, c: f32) -> Self {
        self.confidence = c.clamp(0.0, 1.0);
        self
    }

    pub fn with_run_time(mut self, ts: i64) -> Self {
        self.run_at_ms = ts;
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Weight contributed by this result — achieved × confidence.
    pub fn weighted_score(&self) -> f32 {
        self.achieved * self.confidence
    }
}

/// The aggregate HumannessScore.
#[derive(Debug, Clone)]
pub struct HumannessScore {
    /// Unweighted sum of (achieved × tier_weight × confidence) across all run tests.
    pub raw_score: f32,
    /// Max possible raw (if all tests perfect) — for normalization.
    pub max_possible: f32,
    /// Normalized = raw_score / max_possible, in 0..1.
    pub normalized: f32,
    /// How many of the spec's tests have results?
    pub tests_run: usize,
    /// Total tests in the spec.
    pub tests_total: usize,
    /// Per-tier breakdown.
    pub tier_s: TierScore,
    pub tier_a: TierScore,
    pub tier_b: TierScore,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TierScore {
    pub tier: &'static str,
    pub tests_run: usize,
    pub tests_total: usize,
    pub tests_passed: usize,
    pub avg_achieved: f32,
    pub raw_contribution: f32,
    pub max_contribution: f32,
}

impl HumannessScore {
    /// Compute score from a set of results against a spec.
    pub fn from_results(spec: &BenchmarkSpec, results: &[TestResult]) -> Self {
        let all_tests = spec.all_tests();

        let mut raw_score = 0.0_f32;
        let mut max_possible = 0.0_f32;
        let mut tests_run = 0;

        let mut tier_s = TierScore {
            tier: "S",
            ..Default::default()
        };
        let mut tier_a = TierScore {
            tier: "A",
            ..Default::default()
        };
        let mut tier_b = TierScore {
            tier: "B",
            ..Default::default()
        };

        for test in &all_tests {
            let weight = test.tier.weight();
            let max_contrib = 1.0 * weight;
            max_possible += max_contrib;

            let tier_slot = match test.tier {
                Tier::S => &mut tier_s,
                Tier::A => &mut tier_a,
                Tier::B => &mut tier_b,
            };
            tier_slot.tests_total += 1;
            tier_slot.max_contribution += max_contrib;

            // Find matching result
            if let Some(r) = results.iter().find(|r| r.test_id == test.id) {
                tests_run += 1;
                let contrib = r.weighted_score() * weight;
                raw_score += contrib;

                tier_slot.tests_run += 1;
                tier_slot.avg_achieved += r.achieved;
                tier_slot.raw_contribution += contrib;
                if r.passed {
                    tier_slot.tests_passed += 1;
                }
            }
        }

        // Average achieved per tier (only over run tests)
        for slot in [&mut tier_s, &mut tier_a, &mut tier_b] {
            if slot.tests_run > 0 {
                slot.avg_achieved /= slot.tests_run as f32;
            }
        }

        let normalized = if max_possible > 0.0 {
            raw_score / max_possible
        } else {
            0.0
        };

        HumannessScore {
            raw_score,
            max_possible,
            normalized,
            tests_run,
            tests_total: all_tests.len(),
            tier_s,
            tier_a,
            tier_b,
        }
    }

    pub fn is_mvp_ready(&self) -> bool {
        self.normalized >= 0.60
    }

    pub fn is_v1_ready(&self) -> bool {
        self.normalized >= 0.75
    }

    pub fn is_v2_ready(&self) -> bool {
        self.normalized >= 0.90
    }

    /// Human-readable report.
    pub fn report(&self) -> String {
        format!(
            "HumannessScore: {:.3} ({:.1}% of possible)\n\
             Tests run: {} / {}\n\
             Tier S: {}/{} ({} passed, avg {:.2})\n\
             Tier A: {}/{} ({} passed, avg {:.2})\n\
             Tier B: {}/{} ({} passed, avg {:.2})\n\
             Status: {}",
            self.normalized,
            self.normalized * 100.0,
            self.tests_run,
            self.tests_total,
            self.tier_s.tests_run,
            self.tier_s.tests_total,
            self.tier_s.tests_passed,
            self.tier_s.avg_achieved,
            self.tier_a.tests_run,
            self.tier_a.tests_total,
            self.tier_a.tests_passed,
            self.tier_a.avg_achieved,
            self.tier_b.tests_run,
            self.tier_b.tests_total,
            self.tier_b.tests_passed,
            self.tier_b.avg_achieved,
            if self.is_v2_ready() {
                "V2-ready"
            } else if self.is_v1_ready() {
                "V1-ready"
            } else if self.is_mvp_ready() {
                "MVP-ready"
            } else {
                "pre-MVP"
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_passes_if_above_target() {
        let r = TestResult::pass("1.1", 0.95, 0.90);
        assert!(r.passed);

        let r2 = TestResult::pass("1.1", 0.85, 0.90);
        assert!(!r2.passed);
    }

    #[test]
    fn test_confidence_affects_weighted_score() {
        let r = TestResult::pass("x", 1.0, 0.0).with_confidence(0.5);
        assert_eq!(r.weighted_score(), 0.5);
    }

    #[test]
    fn test_score_report_readable() {
        let spec = BenchmarkSpec::default_v1();
        let score = HumannessScore::from_results(&spec, &[]);
        let report = score.report();
        assert!(report.contains("HumannessScore"));
        assert!(report.contains("pre-MVP"));
    }

    #[test]
    fn test_mvp_detection() {
        let spec = BenchmarkSpec::default_v1();

        // Simulate MVP-grade profile
        let results: Vec<TestResult> = spec
            .all_tests()
            .iter()
            .map(|t| {
                let a = match t.tier {
                    Tier::S => 0.95,
                    Tier::A => 0.70,
                    Tier::B => 0.40,
                };
                TestResult::pass(&t.id, a, t.target)
            })
            .collect();

        let score = HumannessScore::from_results(&spec, &results);
        assert!(score.is_mvp_ready());
    }
}
