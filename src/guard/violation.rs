//! # Violation — security decisions and their justifications
//!
//! When the guard detects a problem, it produces a `GuardDecision`.
//! The decision has:
//!   - An action (Allow, Challenge, Block, Quarantine)
//!   - A violation category (what kind of problem)
//!   - An INTERNAL reason (detailed, for audit)
//!   - An EXTERNAL message (generic, for the user)
//!
//! This separation is deliberate: internal reasons help us debug and
//! improve; external messages MUST NOT teach attackers how detection
//! works.

/// What the guard decided to do with the input/output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardAction {
    /// Pass through — no issues detected.
    Allow,
    /// Allow but with extra caution — log, maybe flag for review.
    Challenge,
    /// Refuse — respond with a generic block message.
    Block,
    /// Severe — block AND quarantine the source (rate limit, session close).
    Quarantine,
}

impl GuardAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            GuardAction::Allow => "allow",
            GuardAction::Challenge => "challenge",
            GuardAction::Block => "block",
            GuardAction::Quarantine => "quarantine",
        }
    }

    pub fn is_permitted(&self) -> bool {
        matches!(self, GuardAction::Allow | GuardAction::Challenge)
    }
}

/// Category of security concern.
///
/// The category is INTERNAL — never shown to the user. It drives audit,
/// metrics, and iterative improvement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViolationKind {
    // ─── Input threats ─────────────────────────
    /// "Ignore previous instructions", role-hijack attempts, etc.
    PromptInjection,
    /// Claims of elevated privilege the source doesn't have.
    PrivilegeEscalation,
    /// Social engineering — false claims about third parties.
    SocialEngineering,
    /// Probing for system internals (architecture, prompts, code).
    SystemProbing,
    /// Request to help with clearly harmful activity.
    HarmfulIntent,
    /// Request to generate content that violates content policy.
    ContentPolicyInput,

    // ─── Output threats ────────────────────────
    /// Draft would leak secrets (API keys, passwords, business codes).
    SecretLeakage,
    /// Draft reveals system prompts, internal structure, source code.
    SystemDisclosure,
    /// Draft contains PII about a third party not authorized to discuss.
    PiiDisclosure,
    /// Draft violates content policy (illegal, harmful advice).
    ContentPolicyOutput,
    /// Draft is hallucinated — confident claims ZETS can't back up.
    Hallucination,

    // ─── Structural threats ────────────────────
    /// Source is on a deny-list / quarantined.
    Quarantined,
    /// Rate limit exceeded.
    RateLimit,
    /// Invariant violated — something impossible happened.
    InvariantBroken,
}

impl ViolationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ViolationKind::PromptInjection => "prompt_injection",
            ViolationKind::PrivilegeEscalation => "privilege_escalation",
            ViolationKind::SocialEngineering => "social_engineering",
            ViolationKind::SystemProbing => "system_probing",
            ViolationKind::HarmfulIntent => "harmful_intent",
            ViolationKind::ContentPolicyInput => "content_policy_input",
            ViolationKind::SecretLeakage => "secret_leakage",
            ViolationKind::SystemDisclosure => "system_disclosure",
            ViolationKind::PiiDisclosure => "pii_disclosure",
            ViolationKind::ContentPolicyOutput => "content_policy_output",
            ViolationKind::Hallucination => "hallucination",
            ViolationKind::Quarantined => "quarantined",
            ViolationKind::RateLimit => "rate_limit",
            ViolationKind::InvariantBroken => "invariant_broken",
        }
    }

    /// Is this violation severe enough that an attacker could learn from
    /// specific feedback? If YES, the external message must be generic.
    pub fn needs_generic_message(&self) -> bool {
        matches!(
            self,
            ViolationKind::PromptInjection
                | ViolationKind::SystemProbing
                | ViolationKind::SystemDisclosure
                | ViolationKind::SecretLeakage
                | ViolationKind::InvariantBroken
        )
    }

    /// Default action for this violation kind.
    pub fn default_action(&self) -> GuardAction {
        match self {
            ViolationKind::PromptInjection => GuardAction::Block,
            ViolationKind::PrivilegeEscalation => GuardAction::Block,
            ViolationKind::SocialEngineering => GuardAction::Challenge,
            ViolationKind::SystemProbing => GuardAction::Block,
            ViolationKind::HarmfulIntent => GuardAction::Block,
            ViolationKind::ContentPolicyInput => GuardAction::Block,
            ViolationKind::SecretLeakage => GuardAction::Block,
            ViolationKind::SystemDisclosure => GuardAction::Block,
            ViolationKind::PiiDisclosure => GuardAction::Block,
            ViolationKind::ContentPolicyOutput => GuardAction::Block,
            ViolationKind::Hallucination => GuardAction::Challenge,
            ViolationKind::Quarantined => GuardAction::Quarantine,
            ViolationKind::RateLimit => GuardAction::Block,
            ViolationKind::InvariantBroken => GuardAction::Quarantine,
        }
    }

    /// Severity from 0..10, for prioritization in audit review.
    pub fn severity(&self) -> u8 {
        match self {
            ViolationKind::PromptInjection => 8,
            ViolationKind::PrivilegeEscalation => 9,
            ViolationKind::SocialEngineering => 5,
            ViolationKind::SystemProbing => 7,
            ViolationKind::HarmfulIntent => 9,
            ViolationKind::ContentPolicyInput => 7,
            ViolationKind::SecretLeakage => 10,
            ViolationKind::SystemDisclosure => 9,
            ViolationKind::PiiDisclosure => 8,
            ViolationKind::ContentPolicyOutput => 8,
            ViolationKind::Hallucination => 4,
            ViolationKind::Quarantined => 6,
            ViolationKind::RateLimit => 3,
            ViolationKind::InvariantBroken => 10,
        }
    }
}

/// A single violation detected during guard checks.
#[derive(Debug, Clone)]
pub struct Violation {
    pub kind: ViolationKind,
    /// INTERNAL reason — free text, only for logs and audit. Contains
    /// enough detail to reproduce / investigate, including which pattern
    /// matched or which rule fired.
    pub internal_reason: String,
    /// Confidence 0..1 that this really is a violation (vs false positive).
    pub confidence: f32,
    /// Which rule/pattern id fired (for tracking hit rates).
    pub rule_id: String,
}

impl Violation {
    pub fn new(
        kind: ViolationKind,
        rule_id: impl Into<String>,
        internal_reason: impl Into<String>,
        confidence: f32,
    ) -> Self {
        Violation {
            kind,
            rule_id: rule_id.into(),
            internal_reason: internal_reason.into(),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

/// A single, final decision from the guard.
#[derive(Debug, Clone)]
pub struct GuardDecision {
    pub action: GuardAction,
    /// All violations detected (can be multiple per input/output).
    pub violations: Vec<Violation>,
    /// Message to SHOW THE USER if blocked.
    /// Generic by design. Must not leak detection logic.
    pub external_message: String,
    /// Detailed breakdown for the audit log (never shown to user).
    pub internal_summary: String,
}

impl GuardDecision {
    /// Pass-through decision — nothing detected.
    pub fn allow() -> Self {
        GuardDecision {
            action: GuardAction::Allow,
            violations: Vec::new(),
            external_message: String::new(),
            internal_summary: "no_violations".into(),
        }
    }

    /// Build a decision from a set of detected violations.
    /// The final action is the most severe action among the violations.
    pub fn from_violations(violations: Vec<Violation>) -> Self {
        if violations.is_empty() {
            return Self::allow();
        }

        // Most-severe action wins.
        let action = violations
            .iter()
            .map(|v| v.kind.default_action())
            .max_by_key(|a| match a {
                GuardAction::Quarantine => 3,
                GuardAction::Block => 2,
                GuardAction::Challenge => 1,
                GuardAction::Allow => 0,
            })
            .unwrap_or(GuardAction::Allow);

        let external_message = Self::compose_external(&violations, action);
        let internal_summary = Self::compose_internal(&violations, action);

        GuardDecision {
            action,
            violations,
            external_message,
            internal_summary,
        }
    }

    /// External message — generic, non-informative to attackers.
    fn compose_external(violations: &[Violation], action: GuardAction) -> String {
        match action {
            GuardAction::Allow => String::new(),
            GuardAction::Challenge => {
                "אני יכול לעזור אבל אני צריך לשאול עוד שאלה כדי להבין בדיוק מה אתה צריך.".into()
            }
            GuardAction::Block => {
                // Even the category is generic — we don't say "prompt injection".
                // We say "cannot help with this".
                let any_needs_generic = violations
                    .iter()
                    .any(|v| v.kind.needs_generic_message());
                if any_needs_generic {
                    "אני לא יכול לסייע בבקשה הזו.".into()
                } else {
                    // Softer phrasing for policy-style blocks that are
                    // safe to explain in general terms
                    "הבקשה הזו חורגת מהתחום שבו אני יכול לעזור.".into()
                }
            }
            GuardAction::Quarantine => {
                "הבקשה לא יכולה להימשך כרגע. נסה שוב מאוחר יותר.".into()
            }
        }
    }

    /// Internal summary — detailed, for audit.
    fn compose_internal(violations: &[Violation], action: GuardAction) -> String {
        let kinds: Vec<_> = violations.iter().map(|v| v.kind.as_str()).collect();
        let max_conf = violations
            .iter()
            .map(|v| v.confidence)
            .fold(0.0_f32, f32::max);
        let rule_ids: Vec<_> = violations.iter().map(|v| v.rule_id.as_str()).collect();
        format!(
            "action={} kinds=[{}] rules=[{}] max_conf={:.2}",
            action.as_str(),
            kinds.join(","),
            rule_ids.join(","),
            max_conf
        )
    }

    pub fn is_allowed(&self) -> bool {
        self.action.is_permitted()
    }

    pub fn is_blocked(&self) -> bool {
        !self.is_allowed()
    }

    /// Max severity across all violations.
    pub fn max_severity(&self) -> u8 {
        self.violations
            .iter()
            .map(|v| v.kind.severity())
            .max()
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allow_decision_empty() {
        let d = GuardDecision::allow();
        assert_eq!(d.action, GuardAction::Allow);
        assert!(d.violations.is_empty());
        assert!(d.is_allowed());
    }

    #[test]
    fn test_from_empty_violations_is_allow() {
        let d = GuardDecision::from_violations(vec![]);
        assert_eq!(d.action, GuardAction::Allow);
    }

    #[test]
    fn test_block_on_prompt_injection() {
        let v = Violation::new(
            ViolationKind::PromptInjection,
            "pi-001",
            "detected 'ignore previous instructions'",
            0.95,
        );
        let d = GuardDecision::from_violations(vec![v]);
        assert_eq!(d.action, GuardAction::Block);
        assert!(d.is_blocked());
    }

    #[test]
    fn test_most_severe_action_wins() {
        let low = Violation::new(ViolationKind::Hallucination, "hal-1", "low sev", 0.5);
        let high = Violation::new(ViolationKind::SecretLeakage, "sl-1", "would leak key", 0.9);
        let d = GuardDecision::from_violations(vec![low, high]);
        assert_eq!(d.action, GuardAction::Block); // SecretLeakage default = Block
    }

    #[test]
    fn test_external_message_generic_for_sensitive() {
        let v = Violation::new(
            ViolationKind::SystemProbing,
            "sp-1",
            "asked how I'm built",
            0.9,
        );
        let d = GuardDecision::from_violations(vec![v]);
        // External must not mention "system_probing" or how we detected
        assert!(!d.external_message.contains("system"));
        assert!(!d.external_message.contains("probing"));
        assert!(!d.external_message.contains("detected"));
    }

    #[test]
    fn test_internal_summary_has_details() {
        let v = Violation::new(
            ViolationKind::PromptInjection,
            "pi-42",
            "specific pattern matched",
            0.88,
        );
        let d = GuardDecision::from_violations(vec![v]);
        assert!(d.internal_summary.contains("prompt_injection"));
        assert!(d.internal_summary.contains("pi-42"));
        assert!(d.internal_summary.contains("block"));
    }

    #[test]
    fn test_confidence_clamped() {
        let v = Violation::new(ViolationKind::Hallucination, "h", "r", 1.5);
        assert_eq!(v.confidence, 1.0);
        let v2 = Violation::new(ViolationKind::Hallucination, "h", "r", -0.5);
        assert_eq!(v2.confidence, 0.0);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(ViolationKind::SecretLeakage.severity() > ViolationKind::Hallucination.severity());
        assert!(ViolationKind::InvariantBroken.severity() >= 9);
        assert!(ViolationKind::RateLimit.severity() <= 5);
    }

    #[test]
    fn test_quarantine_is_most_severe() {
        let inject = Violation::new(ViolationKind::PromptInjection, "p", "", 0.9);
        let quar = Violation::new(ViolationKind::Quarantined, "q", "", 0.9);
        let d = GuardDecision::from_violations(vec![inject, quar]);
        assert_eq!(d.action, GuardAction::Quarantine);
    }

    #[test]
    fn test_needs_generic_true_for_sensitive() {
        assert!(ViolationKind::PromptInjection.needs_generic_message());
        assert!(ViolationKind::SecretLeakage.needs_generic_message());
        assert!(ViolationKind::SystemProbing.needs_generic_message());
        assert!(!ViolationKind::SocialEngineering.needs_generic_message());
        assert!(!ViolationKind::RateLimit.needs_generic_message());
    }

    #[test]
    fn test_max_severity_aggregates() {
        let v1 = Violation::new(ViolationKind::Hallucination, "1", "", 0.5);
        let v2 = Violation::new(ViolationKind::SecretLeakage, "2", "", 0.5);
        let d = GuardDecision::from_violations(vec![v1, v2]);
        assert_eq!(d.max_severity(), ViolationKind::SecretLeakage.severity());
    }
}
