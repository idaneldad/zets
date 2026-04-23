//! # Benchmark — ZETS capability measurement framework
//!
//! This module defines the STRUCTURE of the benchmark — the types,
//! categories, tiers, scoring. The actual tests live in:
//!   - `tests/benchmark/` directory (integration tests, heavy)
//!   - External evaluation runs (e.g. human raters via a web UI)
//!
//! The spec document: docs/10_architecture/20260423_benchmark_spec_V1.md
//!
//! ## Why this skeleton exists
//!
//! Without a type-safe representation of the benchmark, test results
//! can't be aggregated, regression-tracked, or compared across runs.
//! This module gives us:
//!   - `Test`, `Category`, `Tier` typed representations
//!   - `TestResult` with confidence intervals and metadata
//!   - `HumannessScore` computation
//!   - Serialization for storage in graph atoms
//!
//! Implementation of actual tests happens incrementally — each
//! capability gets implemented + benchmark entry + passes threshold.

pub mod calibration;
pub mod score;
pub mod spec;

pub use score::{HumannessScore, TestResult};
pub use spec::{BenchmarkSpec, Category, Test, Tier};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_spec_has_expected_structure() {
        let spec = BenchmarkSpec::default_v1();
        // 14 categories
        assert_eq!(spec.categories.len(), 14);
        // 87 tests total
        assert_eq!(spec.total_tests(), 100);
        // Tier distribution: 27 S, 30 A, 30 B
        assert_eq!(spec.tests_in_tier(Tier::S), 40);
        assert_eq!(spec.tests_in_tier(Tier::A), 30);
        assert_eq!(spec.tests_in_tier(Tier::B), 30);
    }

    #[test]
    fn test_score_empty_run_is_zero() {
        let spec = BenchmarkSpec::default_v1();
        let score = HumannessScore::from_results(&spec, &[]);
        assert_eq!(score.raw_score, 0.0);
    }

    #[test]
    fn test_score_perfect_run_is_one() {
        let spec = BenchmarkSpec::default_v1();
        let results: Vec<TestResult> = spec
            .all_tests()
            .iter()
            .map(|t| TestResult::pass(&t.id, 1.0, t.target))
            .collect();
        let score = HumannessScore::from_results(&spec, &results);
        // Perfect + all weights applied
        assert!(score.normalized >= 0.99);
    }

    #[test]
    fn test_tier_weighting() {
        let spec = BenchmarkSpec::default_v1();

        // Only Tier S perfect → should get Tier S proportion
        let s_only: Vec<TestResult> = spec
            .all_tests()
            .iter()
            .filter(|t| t.tier == Tier::S)
            .map(|t| TestResult::pass(&t.id, 1.0, t.target))
            .collect();
        let score_s = HumannessScore::from_results(&spec, &s_only);
        // Tier S max = 40 × 1.0 = 40 / 70 = 0.571
        assert!(score_s.normalized > 0.55 && score_s.normalized < 0.60);
    }

    #[test]
    fn test_partial_score() {
        let spec = BenchmarkSpec::default_v1();
        let results: Vec<TestResult> = spec
            .all_tests()
            .iter()
            .take(10)
            .map(|t| TestResult::pass(&t.id, 0.5, t.target))
            .collect();
        let score = HumannessScore::from_results(&spec, &results);
        assert!(score.normalized > 0.0 && score.normalized < 0.2);
    }

    #[test]
    fn test_mvp_target_achievable() {
        // Check MVP target 0.60 — achievable with strong S + half A + marginal B?
        let spec = BenchmarkSpec::default_v1();

        let mut results = Vec::new();
        for t in spec.all_tests() {
            let achieved = match t.tier {
                Tier::S => 0.95, // strong
                Tier::A => 0.70, // partial
                Tier::B => 0.40, // marginal
            };
            results.push(TestResult::pass(&t.id, achieved, t.target));
        }

        let score = HumannessScore::from_results(&spec, &results);
        assert!(
            score.normalized >= 0.60,
            "MVP profile should hit 0.60, got {}",
            score.normalized
        );
    }
}
