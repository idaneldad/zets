//! CalibrationQuestion — the input side of a calibration test.

use super::json::JsonValue;

/// How well-established the answer to a question is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    /// Well-known facts — any educated person should know.
    Easy,
    /// Requires specific knowledge or reasoning.
    Medium,
    /// Specialized domain knowledge; low confidence expected.
    Hard,
    /// Deliberately misleading or unanswerable — should trigger refusal.
    Trick,
}

impl Difficulty {
    /// Return the canonical string used in JSONL files.
    pub fn as_str(self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
            Difficulty::Trick => "Trick",
        }
    }

    /// Parse from a string.
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "Easy" => Ok(Difficulty::Easy),
            "Medium" => Ok(Difficulty::Medium),
            "Hard" => Ok(Difficulty::Hard),
            "Trick" => Ok(Difficulty::Trick),
            other => Err(format!("unknown difficulty: '{other}'")),
        }
    }
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The form of the correct answer for a calibration question.
#[derive(Debug, Clone, PartialEq)]
pub enum ExpectedAnswer {
    /// There is exactly one correct string (case-insensitive comparison).
    Exact { value: String },
    /// Any one of these strings is acceptable.
    OneOf { values: Vec<String> },
    /// The system should decline to answer (trick/nonsense question).
    Refuse,
    /// The answer was once valid but may be outdated; system should flag staleness.
    Stale {
        /// A human-readable note about what was last known to be valid.
        last_valid: String,
    },
}

impl ExpectedAnswer {
    /// Parse from a `JsonValue` object.
    pub fn from_json(v: &JsonValue) -> Result<Self, String> {
        let type_str = v.get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| "expected.type must be a string".to_string())?;
        match type_str {
            "Exact" => {
                let value = v.get("value")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| "Exact expected.value must be a string".to_string())?
                    .to_string();
                Ok(ExpectedAnswer::Exact { value })
            }
            "OneOf" => {
                let arr = v.get("values")
                    .and_then(|a| a.as_array())
                    .ok_or_else(|| "OneOf expected.values must be an array".to_string())?;
                let values = arr.iter()
                    .map(|item| item.as_str().map(|s| s.to_string())
                        .ok_or_else(|| "OneOf value must be a string".to_string()))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ExpectedAnswer::OneOf { values })
            }
            "Refuse" => Ok(ExpectedAnswer::Refuse),
            "Stale" => {
                let last_valid = v.get("last_valid")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| "Stale expected.last_valid must be a string".to_string())?
                    .to_string();
                Ok(ExpectedAnswer::Stale { last_valid })
            }
            other => Err(format!("unknown expected.type: '{other}'")),
        }
    }

    /// Check whether a given answer string satisfies this expectation.
    /// Returns `true` if `answer` is correct (case-insensitive).
    /// Always returns `false` for `Refuse` and `Stale` (those need special logic).
    pub fn matches(&self, answer: &str) -> bool {
        match self {
            ExpectedAnswer::Exact { value } => {
                answer.trim().to_lowercase() == value.trim().to_lowercase()
            }
            ExpectedAnswer::OneOf { values } => {
                let norm = answer.trim().to_lowercase();
                values.iter().any(|v| v.trim().to_lowercase() == norm)
            }
            ExpectedAnswer::Refuse => false,
            ExpectedAnswer::Stale { .. } => false,
        }
    }

    /// True when the expected outcome is a refusal.
    pub fn expects_refusal(&self) -> bool {
        matches!(self, ExpectedAnswer::Refuse)
    }

    /// True when the expected outcome is a staleness warning.
    pub fn expects_stale_warning(&self) -> bool {
        matches!(self, ExpectedAnswer::Stale { .. })
    }
}

/// A single calibration question loaded from JSONL.
#[derive(Debug, Clone)]
pub struct CalibrationQuestion {
    /// Unique identifier (e.g. `"easy_001"`).
    pub id: String,
    /// The question text — may be in Hebrew or English.
    pub text: String,
    /// How to judge a correct answer.
    pub expected: ExpectedAnswer,
    /// Difficulty classification.
    pub difficulty: Difficulty,
    /// Semantic category (`"factual_geography"`, `"math"`, `"temporal"`, …).
    pub category: String,
    /// BCP-47 language tag (`"he"` or `"en"`).
    pub language: String,
}

impl CalibrationQuestion {
    /// Parse from a `JsonValue` object (one line of JSONL).
    pub fn from_json(v: &JsonValue) -> Result<Self, String> {
        let id = v.get("id").and_then(|s| s.as_str())
            .ok_or_else(|| "missing 'id' field".to_string())?
            .to_string();
        let text = v.get("text").and_then(|s| s.as_str())
            .ok_or_else(|| format!("[{id}] missing 'text' field"))?
            .to_string();
        let expected_json = v.get("expected")
            .ok_or_else(|| format!("[{id}] missing 'expected' field"))?;
        let expected = ExpectedAnswer::from_json(expected_json)?;
        let difficulty_str = v.get("difficulty").and_then(|s| s.as_str())
            .ok_or_else(|| format!("[{id}] missing 'difficulty' field"))?;
        let difficulty = Difficulty::from_str(difficulty_str)?;
        let category = v.get("category").and_then(|s| s.as_str())
            .ok_or_else(|| format!("[{id}] missing 'category' field"))?
            .to_string();
        let language = v.get("language").and_then(|s| s.as_str())
            .ok_or_else(|| format!("[{id}] missing 'language' field"))?
            .to_string();

        let q = CalibrationQuestion { id, text, expected, difficulty, category, language };
        q.validate()?;
        Ok(q)
    }

    /// Validate that the question is well-formed.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("id must not be empty".to_string());
        }
        if self.text.is_empty() {
            return Err(format!("[{}] text must not be empty", self.id));
        }
        if self.category.is_empty() {
            return Err(format!("[{}] category must not be empty", self.id));
        }
        if self.language != "he" && self.language != "en" {
            return Err(format!(
                "[{}] language must be 'he' or 'en', got '{}'",
                self.id, self.language
            ));
        }
        if let ExpectedAnswer::OneOf { values } = &self.expected {
            if values.is_empty() {
                return Err(format!("[{}] OneOf must have at least one value", self.id));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::json::parse;

    fn make_exact(id: &str, answer: &str) -> CalibrationQuestion {
        CalibrationQuestion {
            id: id.to_string(),
            text: "Test question?".to_string(),
            expected: ExpectedAnswer::Exact { value: answer.to_string() },
            difficulty: Difficulty::Easy,
            category: "factual".to_string(),
            language: "en".to_string(),
        }
    }

    #[test]
    fn test_exact_match_case_insensitive() {
        let q = make_exact("t1", "Paris");
        assert!(q.expected.matches("Paris"));
        assert!(q.expected.matches("paris"));
        assert!(q.expected.matches("PARIS"));
        assert!(!q.expected.matches("London"));
    }

    #[test]
    fn test_oneof_match() {
        let expected = ExpectedAnswer::OneOf {
            values: vec!["1945".to_string(), "nineteen forty-five".to_string()],
        };
        assert!(expected.matches("1945"));
        assert!(expected.matches("Nineteen Forty-Five"));
        assert!(!expected.matches("1944"));
    }

    #[test]
    fn test_refuse_never_matches() {
        let expected = ExpectedAnswer::Refuse;
        assert!(!expected.matches("anything"));
        assert!(expected.expects_refusal());
    }

    #[test]
    fn test_stale_never_matches() {
        let expected = ExpectedAnswer::Stale {
            last_valid: "Elon Musk (2022)".to_string(),
        };
        assert!(!expected.matches("Elon Musk"));
        assert!(expected.expects_stale_warning());
    }

    #[test]
    fn test_question_validation_valid() {
        let q = make_exact("easy_001", "Paris");
        assert!(q.validate().is_ok());
    }

    #[test]
    fn test_question_validation_empty_id() {
        let mut q = make_exact("t1", "Paris");
        q.id = "".to_string();
        assert!(q.validate().is_err());
    }

    #[test]
    fn test_question_validation_bad_language() {
        let mut q = make_exact("t1", "Paris");
        q.language = "fr".to_string();
        assert!(q.validate().is_err());
    }

    #[test]
    fn test_difficulty_display() {
        assert_eq!(Difficulty::Easy.as_str(), "Easy");
        assert_eq!(Difficulty::Trick.as_str(), "Trick");
    }

    #[test]
    fn test_from_json_exact() {
        let json_str = r#"{"id":"easy_001","text":"What is 2+2?","expected":{"type":"Exact","value":"4"},"difficulty":"Easy","category":"math","language":"en"}"#;
        let val = parse(json_str).unwrap();
        let q = CalibrationQuestion::from_json(&val).unwrap();
        assert_eq!(q.id, "easy_001");
        assert_eq!(q.difficulty, Difficulty::Easy);
        assert!(q.expected.matches("4"));
    }

    #[test]
    fn test_from_json_refuse() {
        let json_str = r#"{"id":"trick_001","text":"How many legs on a photon?","expected":{"type":"Refuse"},"difficulty":"Trick","category":"trick","language":"en"}"#;
        let val = parse(json_str).unwrap();
        let q = CalibrationQuestion::from_json(&val).unwrap();
        assert!(q.expected.expects_refusal());
    }
}
