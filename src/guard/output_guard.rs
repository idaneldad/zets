//! # OutputGuard — checks draft responses BEFORE sending
//!
//! Even with a clean input, ZETS's draft response could accidentally leak:
//!   - API keys it has access to
//!   - Source code fragments
//!   - System prompts / internal structure
//!   - PII about third parties
//!   - File paths revealing the server layout
//!
//! The OutputGuard is the LAST line of defense. If it blocks, ZETS
//! regenerates or falls back to a generic apology.

use super::rules::{OUTPUT_LEAKAGE_PATTERNS, RuleId};
use super::violation::{GuardDecision, Violation, ViolationKind};

pub struct OutputGuard;

impl OutputGuard {
    /// Check a draft response. Returns Allow if safe to send.
    pub fn check(draft: &str) -> GuardDecision {
        let mut violations = Vec::new();

        // 1. Literal leakage patterns — API keys, internal paths, etc.
        for pattern in OUTPUT_LEAKAGE_PATTERNS {
            if draft.contains(pattern) {
                let kind = classify_leakage(pattern);
                let rule_id = match kind {
                    ViolationKind::SecretLeakage => RuleId::OP01_ApiKeyLeak,
                    ViolationKind::SystemDisclosure => RuleId::OP03_SystemPromptLeak,
                    _ => RuleId::OP05_InternalPathLeak,
                };
                violations.push(Violation::new(
                    kind,
                    rule_id.as_str(),
                    format!("output leakage pattern matched"),
                    0.95,
                ));
            }
        }

        // 2. Secret-value shape detection — long base64-like strings
        if contains_secret_shape(draft) {
            violations.push(Violation::new(
                ViolationKind::SecretLeakage,
                RuleId::OP01_ApiKeyLeak.as_str(),
                format!("output contains secret-shaped string"),
                0.70,
            ));
        }

        // 3. Self-disclosure patterns — "I was instructed to..."
        if contains_self_disclosure(&draft.to_lowercase()) {
            violations.push(Violation::new(
                ViolationKind::SystemDisclosure,
                RuleId::OP03_SystemPromptLeak.as_str(),
                format!("output reveals system instructions"),
                0.85,
            ));
        }

        // 4. Internal path leakage — ANY /home/, /etc/, C:\Users\
        if contains_internal_path(draft) {
            violations.push(Violation::new(
                ViolationKind::SystemDisclosure,
                RuleId::OP05_InternalPathLeak.as_str(),
                format!("output contains internal file path"),
                0.80,
            ));
        }

        GuardDecision::from_violations(violations)
    }
}

fn classify_leakage(pattern: &str) -> ViolationKind {
    // API keys and tokens → SecretLeakage
    if pattern.starts_with("sk-")
        || pattern.starts_with("AIza")
        || pattern.starts_with("ghp_")
        || pattern.starts_with("github_pat_")
        || pattern.starts_with("xox")
        || pattern.starts_with("AKIA")
        || pattern.starts_with("eyJ")
    {
        return ViolationKind::SecretLeakage;
    }
    // System/code → SystemDisclosure
    if pattern.contains("system prompt")
        || pattern.contains("instructed to")
        || pattern.contains("RULE_SET")
        || pattern.contains("TrustLevel::")
    {
        return ViolationKind::SystemDisclosure;
    }
    // Paths and file names
    ViolationKind::SystemDisclosure
}

/// Heuristic for strings that LOOK like secrets — long, high-entropy,
/// mixed-case alphanumeric with dashes/underscores.
fn contains_secret_shape(draft: &str) -> bool {
    // Look for tokens of 30+ chars that look like secrets
    for token in draft.split_whitespace() {
        // Strip punctuation from ends
        let t = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
        if t.len() < 30 {
            continue;
        }
        if !t.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            continue;
        }
        // Requires mixed case OR digits and alphas mixed (not a single-case word)
        let has_upper = t.chars().any(|c| c.is_ascii_uppercase());
        let has_lower = t.chars().any(|c| c.is_ascii_lowercase());
        let has_digit = t.chars().any(|c| c.is_ascii_digit());
        let mixed_count = [has_upper, has_lower, has_digit].iter().filter(|x| **x).count();
        if mixed_count >= 2 && t.len() >= 30 {
            return true;
        }
    }
    false
}

fn contains_self_disclosure(draft_lower: &str) -> bool {
    let disclosures = [
        "my system prompt",
        "i was instructed to",
        "my guidelines say",
        "the rules i follow",
        "my training data",
        "i am an llm",
        "i was trained",
        "anthropic told me",
        "openai told me",
        "google told me",
        // Hebrew
        "הוראות המערכת שלי",
        "אני מונחה על ידי",
        "חוקים פנימיים",
    ];
    disclosures.iter().any(|d| draft_lower.contains(d))
}

fn contains_internal_path(draft: &str) -> bool {
    let patterns = [
        "/home/",
        "/root/",
        "/etc/passwd",
        "/var/",
        "/usr/local/",
        "C:\\Users\\",
        "C:\\Program Files",
        ".env",
        "vault.enc",
        "master_key",
        ".ssh/id_",
        "id_rsa",
    ];
    patterns.iter().any(|p| draft.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_output_allowed() {
        let d = OutputGuard::check("The weather today is sunny with a high of 25°C.");
        assert!(d.is_allowed());
    }

    #[test]
    fn test_api_key_blocked() {
        let d = OutputGuard::check("Your key is sk-ant-api03-abc123-def456");
        assert!(d.is_blocked());
        assert!(d
            .violations
            .iter()
            .any(|v| v.kind == ViolationKind::SecretLeakage));
    }

    #[test]
    fn test_google_api_key_blocked() {
        let d = OutputGuard::check("Use this: AIzaSyABC-DEF_GHIJK-LMNOP");
        assert!(d.is_blocked());
    }

    #[test]
    fn test_github_token_blocked() {
        let d = OutputGuard::check("Token: ghp_1234567890abcdefABCDEFghijk");
        assert!(d.is_blocked());
    }

    #[test]
    fn test_self_disclosure_blocked() {
        let d = OutputGuard::check("I was instructed to help with your business questions.");
        assert!(d.is_blocked());
        assert!(d
            .violations
            .iter()
            .any(|v| v.kind == ViolationKind::SystemDisclosure));
    }

    #[test]
    fn test_path_leak_blocked() {
        let d = OutputGuard::check("The config is at /home/dinio/.env for reference.");
        assert!(d.is_blocked());
    }

    #[test]
    fn test_secret_shape_blocked() {
        let d = OutputGuard::check(
            "Here's the value: ABCdef1234567890XYZwvu098765zyxw99 for you.",
        );
        assert!(d.is_blocked());
    }

    #[test]
    fn test_normal_mixed_words_not_blocked() {
        // Normal sentences shouldn't trigger secret-shape detection
        let d =
            OutputGuard::check("Thanks for your message about the Q3 financial report review.");
        assert!(d.is_allowed());
    }

    #[test]
    fn test_external_message_on_block_is_generic() {
        let d = OutputGuard::check("Your API key is sk-ant-real-leaking-key-123456789");
        // Must not contain specifics
        assert!(!d.external_message.contains("API"));
        assert!(!d.external_message.contains("sk-"));
        assert!(!d.external_message.contains("key"));
    }

    #[test]
    fn test_hebrew_disclosure_blocked() {
        let d = OutputGuard::check("חוקים פנימיים שלי מכתיבים לענות בצורה מסוימת");
        assert!(d.is_blocked());
    }

    #[test]
    fn test_multiple_leaks_all_recorded() {
        let d = OutputGuard::check(
            "My system prompt tells me to keep sk-ant-secret-123 at /home/dinio/",
        );
        // Should flag multiple things
        assert!(d.is_blocked());
        assert!(d.violations.len() >= 2);
    }

    #[test]
    fn test_short_random_strings_allowed() {
        // Short hex like "abc123" should not trigger secret detection
        let d = OutputGuard::check("The commit hash is abc123def456.");
        assert!(d.is_allowed());
    }
}
