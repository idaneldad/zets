# Calibration Harness

Measures how well ZETS's confidence aligns with its actual accuracy
across **130 questions** spanning Easy, Hard, and Trick categories
in English and Hebrew.

## Key Metrics

| Metric | Formula | Target |
|--------|---------|--------|
| **ECE** (Expected Calibration Error) | Σ_b (n_b/N) · \|acc_b − conf_b\| | < 0.10 |
| **Brier Score** | mean (confidence − outcome)² | < 0.20 |
| **Trick Refusal Rate** | refused_trick / total_trick | > 0.80 |
| **Know/Infer/Guess Accuracy** | correct_tag / total | > 0.70 |

## Files

```
src/benchmark/calibration/
├── mod.rs        — Module root, re-exports
├── json.rs       — Zero-dependency JSON parser
├── question.rs   — CalibrationQuestion, ExpectedAnswer, Difficulty
├── result.rs     — CalibrationResult, KnowInferGuess
├── metrics.rs    — ECE, Brier, BucketStats
├── harness.rs    — CalibrationHarness (load, record, report)
└── report.rs     — CalibrationReport (Display + summary)

data/benchmark/
├── calibration_set_easy.jsonl   — 50 well-known-fact questions
├── calibration_set_hard.jsonl   — 50 specialized-knowledge questions
└── calibration_set_trick.jsonl  — 30 deliberately unanswerable questions
```

## JSONL Format

```json
{"id":"easy_001","text":"What is the capital of France?","expected":{"type":"Exact","value":"Paris"},"difficulty":"Easy","category":"factual_geography","language":"en"}
{"id":"he_001","text":"מה הבירה של ישראל?","expected":{"type":"Exact","value":"ירושלים"},"difficulty":"Easy","category":"factual_geography","language":"he"}
{"id":"trick_001","text":"How tall is the average unicorn?","expected":{"type":"Refuse"},"difficulty":"Trick","category":"mythology_trap","language":"en"}
{"id":"stale_001","text":"Who is the CEO of Twitter?","expected":{"type":"Stale","last_valid":"2023: Linda Yaccarino, X"},"difficulty":"Medium","category":"temporal","language":"en"}
```

### ExpectedAnswer types

| Type | Description |
|------|-------------|
| `Exact` | Single correct string (case-insensitive) |
| `OneOf` | Any element of `values` is acceptable |
| `Refuse` | System must decline to answer |
| `Stale` | Answer once valid; system should warn of possible staleness |

## Usage

```rust
use zets::benchmark::calibration::{
    CalibrationHarness, CalibrationResult, KnowInferGuess,
};
use zets::metacognition::Confidence;

let mut h = CalibrationHarness::load_from_jsonl(
    "data/benchmark/calibration_set_easy.jsonl"
).unwrap();

h.record(CalibrationResult {
    question_id: "easy_001".into(),
    actual_answer: "Paris".into(),
    reported_confidence: Confidence::Certain,
    confidence_tag: KnowInferGuess::Know,
    correct: true,
    refused: false,
    duration_ms: 12,
});

let report = h.report();
println!("{}", report);
println!("ECE: {:.3}", report.ece);
assert!(report.ece_passes()); // ECE < 0.10
```

## Running Tests

```bash
cargo test --lib benchmark::calibration
# Expected: 51 passed, 0 failed
```

## Design Decisions

- **Zero external dependencies**: JSON parsing is hand-rolled (like the rest of ZETS).
- **Confidence mapping**: `Confidence::Certain` → 0.9, `Unknown` → 0.1 (from `as_score()`).
- **ECE bucketing**: 10 equal-width buckets over [0.0, 1.0]; confidence=1.0 lands in the last bucket.
- **Know/Infer/Guess scoring**: Easy → Know, Medium/Hard → Infer, Trick → Refuse.
