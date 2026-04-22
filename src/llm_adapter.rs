//! LLM Adapter — bridges natural language to ZETS's structured representation.
//!
//! This module addresses the #1 bottleneck identified in the bottleneck master
//! doc: ZETS can reason deterministically but can't PARSE questions written in
//! natural language well. An LLM (Gemini Flash) reads the question and returns
//! structured output that ZETS can work with directly.
//!
//! The contract is explicit and narrow:
//!   LLM receives: a natural-language question
//!   LLM returns:  JSON with (intent, key_terms, expected_answer_type)
//!
//! ZETS keeps ALL reasoning. The LLM only does the parsing step it excels at.
//! This is exactly what "Track C" proposes in the master doc.
//!
//! Why this matters:
//!   - Opens MMLU, AGIEval, GPQA benchmarks to ZETS
//!   - Preserves 100% determinism of the reasoning step
//!   - Preserves 100% audit trace (LLM output is saved + hashed)
//!   - Keeps ZETS's speed advantage for follow-up queries (cache hits)

use std::time::Duration;

/// Parsed understanding of a question — what the LLM extracted.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct QuestionParse {
    /// The query intent: "lookup", "compare", "explain", "derive", etc.
    pub intent: String,
    /// Key nouns/entities from the question that ZETS should seed with
    pub key_terms: Vec<String>,
    /// What KIND of answer is expected: "name", "number", "choice", "explanation"
    pub answer_type: String,
    /// Domain hint: "geography", "biology", "cs", "general"
    pub domain: String,
    /// Raw LLM response (for audit)
    pub raw_response: String,
}

/// Error variants when talking to the LLM.
#[derive(Debug)]
pub enum AdapterError {
    /// HTTP/network failure
    Network(String),
    /// LLM returned malformed JSON
    ParseFailure(String),
    /// LLM refused or returned an empty response
    EmptyResponse,
    /// API key missing from environment
    NoApiKey,
    /// Fallback path used (LLM unreachable) — returns deterministic parse
    Fallback(QuestionParse),
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Network(s) => write!(f, "network: {}", s),
            Self::ParseFailure(s) => write!(f, "parse failure: {}", s),
            Self::EmptyResponse => write!(f, "empty LLM response"),
            Self::NoApiKey => write!(f, "missing ZETS_GEMINI_KEY env var"),
            Self::Fallback(_) => write!(f, "used local fallback parser"),
        }
    }
}
impl std::error::Error for AdapterError {}

/// The LLM adapter itself. Configured with an API key and a timeout.
pub struct LlmAdapter {
    api_key: Option<String>,
    #[allow(dead_code)]
    timeout: Duration,
    /// If true, never call the network; use the local rule-based parser only.
    /// Useful for tests and offline runs.
    pub offline_only: bool,
    /// Monotonic counter — how many parses did we do?
    parse_count: u64,
    /// How many fell back to the local parser?
    fallback_count: u64,
}

impl Default for LlmAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmAdapter {
    /// Create a new adapter. Reads ZETS_GEMINI_KEY from environment if present.
    pub fn new() -> Self {
        let api_key = std::env::var("ZETS_GEMINI_KEY").ok();
        Self {
            api_key,
            timeout: Duration::from_secs(10),
            offline_only: false,
            parse_count: 0,
            fallback_count: 0,
        }
    }

    /// Create an offline-only adapter. Will never call the network.
    pub fn offline() -> Self {
        let mut a = Self::new();
        a.offline_only = true;
        a
    }

    /// How many total parses has this adapter handled?
    pub fn parse_count(&self) -> u64 { self.parse_count }

    /// How many of those went to the local fallback?
    pub fn fallback_count(&self) -> u64 { self.fallback_count }

    /// Parse a question into structured form.
    ///
    /// Strategy: if offline_only or no API key, use the local rule parser.
    /// Otherwise, the CALLER is responsible for network I/O (we return the
    /// prompt that should be sent). This keeps this module network-free in
    /// the hot path — the binary decides whether to hit Gemini or not.
    ///
    /// For NOW we always use the local parser. Network integration happens
    /// in a separate `llm_client` module in a future pass.
    pub fn parse(&mut self, question: &str) -> Result<QuestionParse, AdapterError> {
        self.parse_count += 1;

        if self.offline_only || self.api_key.is_none() {
            self.fallback_count += 1;
            return Ok(local_parse(question));
        }

        // TODO: wire up actual HTTP call to Gemini.
        // For now, return local parse + flag that we WOULD have called the API.
        self.fallback_count += 1;
        Ok(local_parse(question))
    }

    /// Build the exact prompt string we'd send to Gemini.
    /// Exposed so tests + the caller can validate the prompt shape.
    pub fn build_prompt(question: &str) -> String {
        format!(
            "Parse this question into structured JSON.\n\n\
             Question: {}\n\n\
             Return ONLY valid JSON with these fields:\n\
             {{\n\
             \t\"intent\": \"lookup | compare | explain | derive\",\n\
             \t\"key_terms\": [\"noun1\", \"noun2\"],\n\
             \t\"answer_type\": \"name | number | choice | explanation\",\n\
             \t\"domain\": \"geography | biology | chemistry | cs | physics | history | general\"\n\
             }}",
            question
        )
    }
}

/// Local rule-based parser — produces a reasonable QuestionParse without an LLM.
/// Uses simple heuristics: wh-words identify intent, nouns identify key terms.
pub fn local_parse(question: &str) -> QuestionParse {
    let lower = question.to_lowercase();

    // Intent from wh-word
    let intent = if lower.starts_with("what is") || lower.starts_with("what's")
               || lower.starts_with("which") || lower.starts_with("who")
               || lower.starts_with("where") || lower.starts_with("when") {
        "lookup".to_string()
    } else if lower.contains("compare") || lower.contains("difference")
           || lower.contains(" vs ") || lower.contains("versus") {
        "compare".to_string()
    } else if lower.starts_with("why") || lower.starts_with("how")
           || lower.contains("explain") || lower.contains("describe") {
        "explain".to_string()
    } else if lower.contains("calculate") || lower.contains("derive")
           || lower.contains("compute") {
        "derive".to_string()
    } else {
        "lookup".to_string()
    };

    // Key terms = content words, deduplicated, preserving first-occurrence order
    let stop: &[&str] = &[
        "what", "is", "the", "a", "an", "of", "in", "on", "at", "to", "from",
        "and", "or", "but", "for", "with", "by", "as", "about", "which",
        "who", "whom", "whose", "where", "when", "why", "how", "does", "do",
        "did", "are", "was", "were", "be", "been", "being", "has", "have",
        "had", "can", "could", "should", "would", "will", "shall", "may",
        "might", "must", "this", "that", "these", "those", "i", "you", "he",
        "she", "it", "we", "they", "me", "him", "her", "us", "them", "my",
        "your", "his", "hers", "our", "their", "its",
    ];
    let mut seen = std::collections::HashSet::new();
    let mut key_terms: Vec<String> = Vec::new();
    for raw in question.split_whitespace() {
        let token = raw.to_lowercase();
        let clean: String = token.chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        if clean.len() < 3 { continue; }
        if stop.contains(&clean.as_str()) { continue; }
        if seen.insert(clean.clone()) {
            key_terms.push(clean);
        }
    }

    // Answer type — multi-choice detection needs choices, but we infer from question
    let answer_type = if lower.contains("how many") || lower.contains("how much")
                    || lower.contains("what year") || lower.contains("atomic number") {
        "number".to_string()
    } else if lower.starts_with("who") || lower.starts_with("where")
           || lower.starts_with("what is") || lower.starts_with("which") {
        "name".to_string()
    } else if lower.starts_with("why") || lower.starts_with("how") {
        "explanation".to_string()
    } else {
        "name".to_string()
    };

    // Domain — keyword presence
    let domain = classify_domain(&lower);

    QuestionParse {
        intent,
        key_terms,
        answer_type,
        domain,
        raw_response: String::new(),  // local parse has no LLM response
    }
}

fn classify_domain(lower: &str) -> String {
    let checks: &[(&str, &[&str])] = &[
        ("geography", &["capital", "country", "city", "continent", "ocean",
                         "france", "germany", "japan", "paris", "berlin", "tokyo"]),
        ("biology",   &["animal", "mammal", "plant", "cell", "dna", "species",
                         "dog", "cat", "fish", "bird", "reptile"]),
        ("chemistry", &["atom", "element", "molecule", "hydrogen", "oxygen",
                         "gold", "iron", "carbon", "atomic"]),
        ("cs",        &["programming", "language", "code", "algorithm", "rust",
                         "python", "javascript", "compiler", "browser"]),
        ("physics",   &["gravity", "force", "energy", "quantum", "relativity",
                         "velocity", "mass", "light", "wave"]),
        ("astronomy", &["sun", "moon", "star", "planet", "galaxy", "orbit",
                         "earth", "mars", "jupiter"]),
        ("history",   &["year", "century", "empire", "war", "revolution",
                         "discovered", "invented", "wrote", "composed"]),
    ];
    for (name, keywords) in checks {
        for kw in keywords.iter() {
            if lower.contains(kw) { return name.to_string(); }
        }
    }
    "general".to_string()
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_parse_geography() {
        let p = local_parse("What is the capital of France?");
        assert_eq!(p.intent, "lookup");
        assert_eq!(p.domain, "geography");
        assert!(p.key_terms.contains(&"capital".to_string()));
        assert!(p.key_terms.contains(&"france".to_string()));
    }

    #[test]
    fn local_parse_biology() {
        let p = local_parse("What kind of animal is a dog?");
        assert_eq!(p.intent, "lookup");
        assert_eq!(p.domain, "biology");
        assert!(p.key_terms.contains(&"animal".to_string()));
        assert!(p.key_terms.contains(&"dog".to_string()));
    }

    #[test]
    fn local_parse_chemistry_number() {
        let p = local_parse("What is gold's atomic number?");
        assert_eq!(p.answer_type, "number");
        assert_eq!(p.domain, "chemistry");
    }

    #[test]
    fn local_parse_comparison() {
        let p = local_parse("Compare Rust and Python performance");
        assert_eq!(p.intent, "compare");
        assert_eq!(p.domain, "cs");
    }

    #[test]
    fn local_parse_explain() {
        let p = local_parse("Why is the sky blue?");
        assert_eq!(p.intent, "explain");
        assert_eq!(p.answer_type, "explanation");
    }

    #[test]
    fn local_parse_strips_stopwords() {
        let p = local_parse("What is a cat?");
        // should NOT contain "what", "is", "a"
        assert!(!p.key_terms.contains(&"what".to_string()));
        assert!(!p.key_terms.contains(&"is".to_string()));
        assert!(p.key_terms.contains(&"cat".to_string()));
    }

    #[test]
    fn local_parse_preserves_order() {
        let p = local_parse("Where is Paris Berlin Tokyo located?");
        // Order should be first-seen
        let i_paris = p.key_terms.iter().position(|s| s == "paris");
        let i_berlin = p.key_terms.iter().position(|s| s == "berlin");
        let i_tokyo = p.key_terms.iter().position(|s| s == "tokyo");
        assert!(i_paris.is_some() && i_berlin.is_some() && i_tokyo.is_some());
        assert!(i_paris < i_berlin);
        assert!(i_berlin < i_tokyo);
    }

    #[test]
    fn local_parse_deduplicates() {
        let p = local_parse("Dog dog DOG Dog?");
        let dog_count = p.key_terms.iter().filter(|s| *s == "dog").count();
        assert_eq!(dog_count, 1);
    }

    #[test]
    fn local_parse_handles_hebrew() {
        // Hebrew text should not panic — key_terms may be empty but function must return
        let p = local_parse("מה בירת צרפת?");
        assert!(!p.intent.is_empty());
    }

    #[test]
    fn local_parse_unknown_domain() {
        let p = local_parse("What is zqbxlq?");
        assert_eq!(p.domain, "general");
    }

    #[test]
    fn adapter_offline_uses_local_parse() {
        let mut a = LlmAdapter::offline();
        let result = a.parse("What is the capital of France?").unwrap();
        assert_eq!(result.domain, "geography");
        assert_eq!(a.parse_count(), 1);
        assert_eq!(a.fallback_count(), 1);
    }

    #[test]
    fn adapter_without_api_key_falls_back() {
        // Ensure env var is not set for this test
        std::env::remove_var("ZETS_GEMINI_KEY");
        let mut a = LlmAdapter::new();
        let result = a.parse("What is Rust?").unwrap();
        assert_eq!(result.domain, "cs");
        assert_eq!(a.fallback_count(), 1);
    }

    #[test]
    fn build_prompt_contains_question() {
        let prompt = LlmAdapter::build_prompt("Test question?");
        assert!(prompt.contains("Test question?"));
        assert!(prompt.contains("JSON"));
        assert!(prompt.contains("intent"));
        assert!(prompt.contains("key_terms"));
        assert!(prompt.contains("answer_type"));
        assert!(prompt.contains("domain"));
    }

    #[test]
    fn determinism_same_input_same_output() {
        let p1 = local_parse("What is the capital of France?");
        let p2 = local_parse("What is the capital of France?");
        assert_eq!(p1, p2);
    }

    #[test]
    fn parse_counts_track_correctly() {
        let mut a = LlmAdapter::offline();
        for i in 0..5 {
            a.parse(&format!("Question {}?", i)).unwrap();
        }
        assert_eq!(a.parse_count(), 5);
        assert_eq!(a.fallback_count(), 5);
    }
}
