//! Provenance — source tracking and trust tiers.

/// Trust level of the source material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrustTier {
    PeerReviewed,
    EditorialReview,
    Community,
    Anecdotal,
    Unknown,
}

/// Where content came from and how trustworthy it is.
#[derive(Debug, Clone)]
pub struct Provenance {
    pub source: Option<String>,
    pub author: Option<String>,
    pub published_year: Option<i32>,
    pub license: Option<String>,
    pub trust_tier: TrustTier,
}

impl Provenance {
    pub fn unknown() -> Self {
        Self {
            source: None,
            author: None,
            published_year: None,
            license: None,
            trust_tier: TrustTier::Unknown,
        }
    }

    /// Check if source string matches any of the given keywords (case-insensitive).
    pub fn source_contains_any(&self, keywords: &[&str]) -> bool {
        match &self.source {
            Some(src) => {
                let lower = src.to_lowercase();
                keywords.iter().any(|kw| lower.contains(kw))
            }
            None => false,
        }
    }
}
