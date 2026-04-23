//! # SecretRef — a pointer to a secret, NOT the secret itself
//!
//! Critical design: the graph NEVER holds secret values. The graph holds
//! REFERENCES — atoms that say "secret X belongs to Owner Y, has ACL Z,
//! was last used at time T". The actual value (API key, password) lives
//! in a separate encrypted vault, accessed only by the vault module.
//!
//! This separation means:
//!   - Graph dumps, debug traces, and logs can't leak secrets
//!   - Secrets can be rotated without touching the graph
//!   - Audit trail (who accessed what when) is a first-class graph query
//!   - If the graph is compromised, secrets remain safe

use std::fmt;

use crate::personal_graph::{IdentityId, Visibility};

/// Unique identifier for a secret — human-readable handle.
/// Format: `{scope_owner}/{kind}/{local_name}` — e.g.
/// `person:idan/api_key/openai`, `org:chooz/oauth/gmail`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SecretId(pub String);

impl SecretId {
    pub fn new(owner: &IdentityId, kind: SecretKind, local_name: impl Into<String>) -> Self {
        SecretId(format!("{}/{}/{}", owner.0, kind.as_str(), local_name.into()))
    }
}

impl fmt::Display for SecretId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Kind of secret — controls rotation policy, access frequency expectations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretKind {
    /// An API key for a third-party service (OpenAI, Gemini, GitHub).
    ApiKey,
    /// An OAuth refresh or access token.
    OAuth,
    /// A password (user credential).
    Password,
    /// A webhook signing secret (for verifying incoming requests).
    WebhookSecret,
    /// A TLS/SSH private key.
    PrivateKey,
    /// A database connection string (contains password).
    ConnectionString,
    /// A business code / coupon / promo code (not a cryptographic secret
    /// but treated as protected data).
    BusinessCode,
    /// Generic — use when none of the above applies.
    Generic,
}

impl SecretKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecretKind::ApiKey => "api_key",
            SecretKind::OAuth => "oauth",
            SecretKind::Password => "password",
            SecretKind::WebhookSecret => "webhook_secret",
            SecretKind::PrivateKey => "private_key",
            SecretKind::ConnectionString => "connection_string",
            SecretKind::BusinessCode => "business_code",
            SecretKind::Generic => "generic",
        }
    }

    /// Recommended rotation period in days (0 = no auto-rotation).
    pub fn rotation_days(&self) -> u32 {
        match self {
            SecretKind::ApiKey => 90,
            SecretKind::OAuth => 60,
            SecretKind::Password => 180,
            SecretKind::WebhookSecret => 180,
            SecretKind::PrivateKey => 365,
            SecretKind::ConnectionString => 180,
            SecretKind::BusinessCode => 0,
            SecretKind::Generic => 0,
        }
    }

    /// Is this secret potentially subject to sensitivity regulations
    /// (GDPR, HIPAA, etc.)? Higher sensitivity → stricter audit.
    pub fn is_high_sensitivity(&self) -> bool {
        matches!(
            self,
            SecretKind::Password | SecretKind::PrivateKey | SecretKind::ConnectionString
        )
    }
}

/// SecretRef — the GRAPH ATOM. Contains no secret material.
///
/// This struct is what gets stored in the graph. It tells us WHO owns the
/// secret, WHO can access it, WHEN it was used — but never WHAT the value is.
#[derive(Debug, Clone)]
pub struct SecretRef {
    pub id: SecretId,
    pub kind: SecretKind,
    /// The identity that owns this secret (Person or Org).
    pub owner: IdentityId,
    /// Visibility — who can see that this secret EXISTS.
    pub visibility: Visibility,
    /// Access control list — identities allowed to access the VALUE.
    /// The owner is implicitly included.
    pub acl: Vec<IdentityId>,
    /// When the secret was first recorded.
    pub created_ms: i64,
    /// Last time the value was accessed (not the ref — the actual value).
    pub last_accessed_ms: Option<i64>,
    /// Last time the value was rotated.
    pub last_rotated_ms: Option<i64>,
    /// Optional human-readable description (NEVER contains the secret itself).
    pub description: Option<String>,
    /// Lifecycle — active, revoked, expired.
    pub status: SecretStatus,
}

/// Lifecycle status of a secret.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SecretStatus {
    /// In use.
    #[default]
    Active,
    /// Temporarily disabled — value exists but refuses access.
    Disabled,
    /// Revoked — value should no longer work at the provider.
    /// The vault entry may be zeroed out, but the ref stays for audit.
    Revoked,
    /// Expired — provider-side expiry passed. May be renewable.
    Expired,
}

impl SecretStatus {
    pub fn allows_access(&self) -> bool {
        matches!(self, SecretStatus::Active)
    }
}

impl SecretRef {
    pub fn new(
        owner: IdentityId,
        kind: SecretKind,
        local_name: impl Into<String>,
        now_ms: i64,
    ) -> Self {
        let id = SecretId::new(&owner, kind, local_name);
        SecretRef {
            id,
            kind,
            owner,
            visibility: Visibility::Private,
            acl: Vec::new(),
            created_ms: now_ms,
            last_accessed_ms: None,
            last_rotated_ms: None,
            description: None,
            status: SecretStatus::Active,
        }
    }

    /// Can this identity access the secret's value?
    pub fn can_access(&self, who: &IdentityId) -> bool {
        if !self.status.allows_access() {
            return false;
        }
        // Owner always has access.
        if &self.owner == who {
            return true;
        }
        self.acl.iter().any(|a| a == who)
    }

    /// Grant access to another identity.
    pub fn grant(&mut self, who: IdentityId) {
        if !self.acl.iter().any(|a| a == &who) && who != self.owner {
            self.acl.push(who);
        }
    }

    /// Revoke access.
    pub fn revoke_access(&mut self, who: &IdentityId) {
        self.acl.retain(|a| a != who);
    }

    /// Mark the value as accessed (for audit).
    pub fn touch(&mut self, now_ms: i64) {
        self.last_accessed_ms = Some(now_ms);
    }

    /// Mark the secret as rotated.
    pub fn rotated(&mut self, now_ms: i64) {
        self.last_rotated_ms = Some(now_ms);
    }

    /// Should this secret be rotated based on its age?
    pub fn needs_rotation(&self, now_ms: i64) -> bool {
        let rot_days = self.kind.rotation_days();
        if rot_days == 0 {
            return false;
        }
        let reference_ms = self.last_rotated_ms.unwrap_or(self.created_ms);
        let age_ms = now_ms.saturating_sub(reference_ms);
        let age_days = age_ms / (24 * 60 * 60 * 1000);
        age_days >= rot_days as i64
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_visibility(mut self, v: Visibility) -> Self {
        self.visibility = v;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::personal_graph::IdentityKind;

    fn idan() -> IdentityId {
        IdentityId::new(IdentityKind::Person, "idan")
    }
    fn chooz() -> IdentityId {
        IdentityId::new(IdentityKind::Org, "chooz")
    }
    fn employee() -> IdentityId {
        IdentityId::new(IdentityKind::Person, "employee1")
    }

    #[test]
    fn test_secret_id_format() {
        let sid = SecretId::new(&idan(), SecretKind::ApiKey, "openai");
        assert_eq!(sid.0, "person:idan/api_key/openai");
    }

    #[test]
    fn test_org_secret_id() {
        let sid = SecretId::new(&chooz(), SecretKind::OAuth, "gmail");
        assert_eq!(sid.0, "org:chooz/oauth/gmail");
    }

    #[test]
    fn test_owner_always_has_access() {
        let r = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
        assert!(r.can_access(&idan()));
    }

    #[test]
    fn test_stranger_no_access() {
        let r = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
        assert!(!r.can_access(&employee()));
    }

    #[test]
    fn test_grant_revoke() {
        let mut r = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
        r.grant(employee());
        assert!(r.can_access(&employee()));
        r.revoke_access(&employee());
        assert!(!r.can_access(&employee()));
    }

    #[test]
    fn test_disabled_blocks_owner() {
        let mut r = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
        r.status = SecretStatus::Disabled;
        assert!(!r.can_access(&idan())); // even owner can't access disabled secret
    }

    #[test]
    fn test_revoked_blocks_access() {
        let mut r = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
        r.status = SecretStatus::Revoked;
        assert!(!r.can_access(&idan()));
    }

    #[test]
    fn test_rotation_needed() {
        let mut r = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
        // API keys rotate every 90 days = ~7.78 * 10^9 ms
        let day_ms: i64 = 24 * 60 * 60 * 1000;
        let sixty_days_later = 1000 + 60 * day_ms;
        assert!(!r.needs_rotation(sixty_days_later));
        let hundred_days_later = 1000 + 100 * day_ms;
        assert!(r.needs_rotation(hundred_days_later));

        // After rotation, clock resets
        r.rotated(hundred_days_later);
        assert!(!r.needs_rotation(hundred_days_later + 60 * day_ms));
    }

    #[test]
    fn test_business_code_no_rotation() {
        let r = SecretRef::new(chooz(), SecretKind::BusinessCode, "promo2026", 1000);
        let far_future = 1000 + 365 * 24 * 60 * 60 * 1000;
        assert!(!r.needs_rotation(far_future));
    }

    #[test]
    fn test_touch_updates_access_time() {
        let mut r = SecretRef::new(idan(), SecretKind::ApiKey, "openai", 1000);
        assert!(r.last_accessed_ms.is_none());
        r.touch(2000);
        assert_eq!(r.last_accessed_ms, Some(2000));
    }

    #[test]
    fn test_high_sensitivity() {
        assert!(SecretKind::Password.is_high_sensitivity());
        assert!(SecretKind::PrivateKey.is_high_sensitivity());
        assert!(!SecretKind::ApiKey.is_high_sensitivity());
    }

    #[test]
    fn test_company_code_scenario() {
        // Per Idan's ask: company has a code; employee is a user with
        // relationship to company; the company's secret is separate
        // from the employee's secrets.
        let company_code = SecretRef::new(
            chooz(),
            SecretKind::BusinessCode,
            "master_code_2026",
            1000,
        );
        let employee_api = SecretRef::new(
            employee(),
            SecretKind::ApiKey,
            "their_own_api",
            1000,
        );

        // Employee can't access company code (no grant)
        assert!(!company_code.can_access(&employee()));
        // Company can't access employee's personal key
        assert!(!employee_api.can_access(&chooz()));
        // Different IDs → different graph atoms
        assert_ne!(company_code.id, employee_api.id);
    }
}
