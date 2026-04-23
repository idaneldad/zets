//! Quote policy derivation — rules for how content may be referenced.
//!
//! Derives QuotePolicy from epistemic status + trust tier + license.
//! Idan's specific rule: ReligiousNarrative → ConceptOnly.

use super::provenance::TrustTier;
use super::work::{EpistemicStatus, QuotePolicy};

/// Derive the quote policy for content based on its epistemic status,
/// trust level, and licensing.
///
/// Rules (from mission spec):
///   - EmpiricalFact + PeerReviewed → FreelyQuotable
///   - ReligiousNarrative → ConceptOnly (Idan's rule)
///   - Fiction + copyright → Paraphraseable
///   - Opinion + Anecdotal → Paraphraseable
///   - Open license (CC, public domain) → FreelyQuotable
///   - default → Paraphraseable
pub fn derive_quote_policy(
    epistemic: EpistemicStatus,
    trust: TrustTier,
    license: Option<&str>,
) -> QuotePolicy {
    // Idan's rule: religious narratives are concept-only
    if epistemic == EpistemicStatus::ReligiousNarrative {
        return QuotePolicy::ConceptOnly;
    }

    // Private content stays private
    if trust == TrustTier::Unknown && license.is_none() {
        // Could be private, but default to paraphraseable rather than private
        // (Private should be explicitly set by the user)
    }

    // Check for open license
    let is_open = license.map(|l| {
        let lower = l.to_lowercase();
        lower.contains("cc-by") || lower.contains("cc0")
            || lower.contains("public domain") || lower.contains("mit")
            || lower.contains("apache") || lower.contains("bsd")
    }).unwrap_or(false);

    // Empirical + peer reviewed → freely quotable
    if epistemic == EpistemicStatus::EmpiricalFact && trust == TrustTier::PeerReviewed {
        return QuotePolicy::FreelyQuotable;
    }

    // Theoretical + peer reviewed → freely quotable
    if epistemic == EpistemicStatus::Theoretical && trust == TrustTier::PeerReviewed {
        return QuotePolicy::FreelyQuotable;
    }

    // Historical record + editorial review → freely quotable
    if epistemic == EpistemicStatus::HistoricalRecord
        && (trust == TrustTier::PeerReviewed || trust == TrustTier::EditorialReview)
    {
        return QuotePolicy::FreelyQuotable;
    }

    // Open license → freely quotable
    if is_open {
        return QuotePolicy::FreelyQuotable;
    }

    // Mythology → concept only (similar reasoning to religious narrative)
    if epistemic == EpistemicStatus::Mythology {
        return QuotePolicy::ConceptOnly;
    }

    // Everything else defaults to paraphraseable
    QuotePolicy::Paraphraseable
}
