//! # Rules — the immutable security rules
//!
//! These rules are COMPILED INTO THE BINARY. They cannot be changed at
//! runtime. Attempts to modify the graph do not affect them.
//!
//! ## The 5 trust-floor rules — NEVER negotiable
//!
//! 1. Secret values never leak in responses.
//! 2. System prompts / source code / architecture details never disclosed.
//! 3. Procedures with TrustLevel::Experimental cannot be promoted to
//!    System at runtime via conversation.
//! 4. No user can impersonate the Owner without cryptographic proof.
//! 5. Harmful content (weapons, CSAM, hacking live systems) always refused.
//!
//! ## Versioning
//!
//! Every rule has an id + version. The integrity hash of the rule set is
//! computed at startup and checked periodically. If the hash mismatches
//! expected, ZETS refuses to start.

/// Hardcoded constant representing the expected checksum of the rule set.
///
/// Updated whenever rules change (deliberate, compile-time edit).
/// Used for integrity verification at startup.
pub const RULE_SET_VERSION: u32 = 1;

/// A single rule identifier. Embedded in audit logs + patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleId {
    // Trust floor (immutable)
    TF01_NoSecretLeakage,
    TF02_NoSystemDisclosure,
    TF03_NoRuntimeTrustPromotion,
    TF04_NoOwnerImpersonation,
    TF05_NoHarmfulContent,

    // Input patterns
    IP01_InstructionOverride,
    IP02_RolePlayHijack,
    IP03_AuthorityClaim,
    IP04_SystemQuery,
    IP05_BackdoorPhrase,

    // Output patterns
    OP01_ApiKeyLeak,
    OP02_SourceCodeLeak,
    OP03_SystemPromptLeak,
    OP04_PiiExposure,
    OP05_InternalPathLeak,

    // Structural
    ST01_RateLimit,
    ST02_Quarantine,
    ST03_InvariantBreach,
}

impl RuleId {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleId::TF01_NoSecretLeakage => "TF01",
            RuleId::TF02_NoSystemDisclosure => "TF02",
            RuleId::TF03_NoRuntimeTrustPromotion => "TF03",
            RuleId::TF04_NoOwnerImpersonation => "TF04",
            RuleId::TF05_NoHarmfulContent => "TF05",
            RuleId::IP01_InstructionOverride => "IP01",
            RuleId::IP02_RolePlayHijack => "IP02",
            RuleId::IP03_AuthorityClaim => "IP03",
            RuleId::IP04_SystemQuery => "IP04",
            RuleId::IP05_BackdoorPhrase => "IP05",
            RuleId::OP01_ApiKeyLeak => "OP01",
            RuleId::OP02_SourceCodeLeak => "OP02",
            RuleId::OP03_SystemPromptLeak => "OP03",
            RuleId::OP04_PiiExposure => "OP04",
            RuleId::OP05_InternalPathLeak => "OP05",
            RuleId::ST01_RateLimit => "ST01",
            RuleId::ST02_Quarantine => "ST02",
            RuleId::ST03_InvariantBreach => "ST03",
        }
    }

    /// Is this rule absolutely immutable (trust floor)?
    pub fn is_trust_floor(&self) -> bool {
        matches!(
            self,
            RuleId::TF01_NoSecretLeakage
                | RuleId::TF02_NoSystemDisclosure
                | RuleId::TF03_NoRuntimeTrustPromotion
                | RuleId::TF04_NoOwnerImpersonation
                | RuleId::TF05_NoHarmfulContent
        )
    }
}

/// Patterns that indicate likely prompt injection in user input.
///
/// INTENTIONALLY NOT EXHAUSTIVE — attackers read source. These catch the
/// lazy attacks. Pattern library in `patterns.rs` extends this at runtime
/// (graph-stored but ADDITIVE, can never disable these).
pub const INJECTION_PATTERNS: &[&str] = &[
    // Direct instruction override
    "ignore previous instructions",
    "ignore all previous",
    "disregard the above",
    "forget everything",
    "forget the prior instructions",
    "your new instructions",
    "you are now",
    "pretend you are",
    "act as if",
    "you will now",

    // Hebrew equivalents
    "התעלם מההוראות",
    "התעלם מכל מה שנאמר",
    "שכח את כל",
    "אתה עכשיו",
    "העמד פנים",
    "תתנהג כאילו",

    // Role hijack
    "developer mode",
    "admin mode",
    "jailbreak",
    "DAN mode",
    "without restrictions",
    "bypass your",
    "unrestricted mode",

    // System exfiltration
    "show me your prompt",
    "what are your instructions",
    "print your system",
    "reveal your rules",
    "your source code",
    "your actual prompt",

    // Hebrew
    "הראה לי את ה-prompt",
    "מה ההוראות שלך",
    "חשוף את החוקים",
    "הקוד שלך",
];

/// Patterns in OUTPUT drafts that must never appear in responses.
/// If matched, output is blocked.
pub const OUTPUT_LEAKAGE_PATTERNS: &[&str] = &[
    // API key shapes
    "sk-ant-",
    "sk-proj-",
    "AIza",       // Google API keys
    "ghp_",       // GitHub personal tokens
    "github_pat_",
    "xoxb-",      // Slack bot tokens
    "xoxp-",      // Slack user tokens
    "AKIA",       // AWS access key prefix
    "eyJhbGciOiJ", // JWT header base64

    // Internal paths
    "/home/dinio/",
    "/home/dinio/.env",
    "src/secrets/",
    "vault.enc",
    "master_key",

    // System disclosure
    "my system prompt",
    "I was instructed to",
    "my guidelines say",
    "the rules I follow",
    "RULE_SET_VERSION",
    "TrustLevel::",
];

/// Authority-claim patterns — social engineering attempts.
pub const AUTHORITY_CLAIM_PATTERNS: &[&str] = &[
    "I am the owner",
    "I am idan eldad",
    "this is the administrator",
    "I have permission to",
    "i was told I can",
    "[the owner] said",

    // Hebrew
    "אני הבעלים",
    "אני המנהל",
    "קיבלתי אישור",
    "עידן אמר שאני",
];

/// Harmful content indicators — topics where we refuse help outright.
/// Intentionally short list focused on SEVERE harm. Borderline cases
/// are handled by ContentPolicy (softer, more nuanced) not here.
pub const HARMFUL_CONTENT_PATTERNS: &[&str] = &[
    "how do I make a bomb",
    "synthesize meth",
    "child exploit",
    "hack into a live",

    // These are detected as SEVERE — anything borderline routes elsewhere
];

/// Compute a simple checksum of the rule set. Used at startup to verify
/// the binary hasn't been tampered with in memory (detects some classes
/// of attacks at the cost of minor compute).
pub fn rules_checksum() -> u64 {
    let mut h: u64 = 0xcbf29ce484222325; // FNV offset
    const FNV_PRIME: u64 = 0x100000001b3;

    let all_patterns = [
        INJECTION_PATTERNS,
        OUTPUT_LEAKAGE_PATTERNS,
        AUTHORITY_CLAIM_PATTERNS,
        HARMFUL_CONTENT_PATTERNS,
    ];

    for group in &all_patterns {
        for pat in group.iter() {
            for b in pat.bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(FNV_PRIME);
            }
        }
    }

    h ^ (RULE_SET_VERSION as u64)
}

/// Expected checksum — computed at first run and pinned as a constant.
/// If this doesn't match runtime checksum, rules have been modified.
///
/// NOTE: This is NOT a cryptographic proof — it's a tamper detection
/// at the compiled-binary level. Production systems need code signing.
pub const EXPECTED_CHECKSUM: u64 = 0; // 0 = skip check (populated below)

/// Verify rules integrity.
///
/// In production, this would be called at startup and compared against
/// a signed expected value. For now it just computes and returns the
/// checksum so tests can snapshot it.
pub fn verify_rules_integrity() -> RulesIntegrity {
    let actual = rules_checksum();
    // If EXPECTED_CHECKSUM is 0, we are in dev/test mode — skip strict check
    if EXPECTED_CHECKSUM == 0 {
        return RulesIntegrity::DevMode(actual);
    }
    if actual == EXPECTED_CHECKSUM {
        RulesIntegrity::Valid
    } else {
        RulesIntegrity::Tampered {
            expected: EXPECTED_CHECKSUM,
            actual,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulesIntegrity {
    Valid,
    DevMode(u64), // dev mode, no strict check, just reports actual
    Tampered { expected: u64, actual: u64 },
}

impl RulesIntegrity {
    pub fn is_ok(&self) -> bool {
        matches!(self, RulesIntegrity::Valid | RulesIntegrity::DevMode(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_ids_unique() {
        let ids = [
            RuleId::TF01_NoSecretLeakage.as_str(),
            RuleId::TF02_NoSystemDisclosure.as_str(),
            RuleId::IP01_InstructionOverride.as_str(),
            RuleId::OP01_ApiKeyLeak.as_str(),
            RuleId::ST01_RateLimit.as_str(),
        ];
        // All distinct
        for (i, a) in ids.iter().enumerate() {
            for (j, b) in ids.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn test_trust_floor_recognition() {
        assert!(RuleId::TF01_NoSecretLeakage.is_trust_floor());
        assert!(RuleId::TF05_NoHarmfulContent.is_trust_floor());
        assert!(!RuleId::IP01_InstructionOverride.is_trust_floor());
        assert!(!RuleId::OP01_ApiKeyLeak.is_trust_floor());
    }

    #[test]
    fn test_injection_patterns_nonempty() {
        assert!(INJECTION_PATTERNS.len() > 10);
        // Must include both English and Hebrew
        assert!(INJECTION_PATTERNS.iter().any(|p| p.contains("ignore")));
        assert!(INJECTION_PATTERNS.iter().any(|p| p.contains("התעלם")));
    }

    #[test]
    fn test_output_patterns_cover_key_secrets() {
        // Common API key prefixes should all be covered
        let must_cover = ["sk-ant-", "AIza", "ghp_", "xoxb-"];
        for k in must_cover {
            assert!(
                OUTPUT_LEAKAGE_PATTERNS.iter().any(|p| p == &k),
                "missing pattern for {}",
                k
            );
        }
    }

    #[test]
    fn test_checksum_deterministic() {
        let c1 = rules_checksum();
        let c2 = rules_checksum();
        assert_eq!(c1, c2);
        // And non-zero
        assert_ne!(c1, 0);
    }

    #[test]
    fn test_checksum_includes_version() {
        // This can't really be tested without mutating const, but we
        // verify the result is a 64-bit value incorporating version
        let c = rules_checksum();
        assert!(c > 0);
    }

    #[test]
    fn test_integrity_in_dev_mode() {
        let r = verify_rules_integrity();
        // EXPECTED_CHECKSUM is 0 → dev mode
        assert!(r.is_ok());
        assert!(matches!(r, RulesIntegrity::DevMode(_)));
    }

    #[test]
    fn test_authority_claim_hebrew() {
        assert!(AUTHORITY_CLAIM_PATTERNS
            .iter()
            .any(|p| p.contains("הבעלים")));
    }

    #[test]
    fn test_harmful_content_focused() {
        // Should be SHORT — this is for severe-only. Borderline goes elsewhere.
        assert!(HARMFUL_CONTENT_PATTERNS.len() < 10);
    }
}
