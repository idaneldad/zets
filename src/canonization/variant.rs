//! Variant detection — decides whether new content is a variant of existing works.
//!
//! Thresholds (from mission spec):
//!   >0.95 → Translation / same content
//!   0.70-0.95 → Derivative or close paraphrase
//!   0.50-0.70 → ParallelTradition or loose translation
//!   <0.50 → Unrelated

use super::fingerprint::{Fingerprint, fingerprint_similarity};
use super::work::{WorkId, WorkKind, Fidelity, DerivationKind};

/// Similarity thresholds for variant classification.
pub const THRESHOLD_TRANSLATION: f32 = 0.95;
pub const THRESHOLD_DERIVATIVE: f32 = 0.70;
pub const THRESHOLD_PARALLEL: f32 = 0.50;

/// Result of comparing a fingerprint against an existing work.
#[derive(Debug, Clone)]
pub struct VariantMatch {
    pub similarity: f32,
    pub likely_kind: WorkKind,
    pub evidence: Vec<String>,
}

/// Classify the relationship between a new fingerprint and an existing one.
/// Returns None if unrelated (below parallel threshold).
pub fn classify_variant(
    new_fp: &Fingerprint,
    existing_fp: &Fingerprint,
    existing_id: WorkId,
    same_language: bool,
) -> Option<VariantMatch> {
    let similarity = fingerprint_similarity(new_fp, existing_fp);

    if similarity < THRESHOLD_PARALLEL {
        return None;
    }

    let mut evidence = Vec::new();
    evidence.push(format!("similarity: {similarity:.3}"));

    if new_fp.length == existing_fp.length {
        evidence.push("same length tier".to_string());
    } else {
        evidence.push(format!(
            "length tier mismatch: {} vs {}",
            new_fp.length, existing_fp.length
        ));
    }

    let likely_kind = if similarity >= THRESHOLD_TRANSLATION {
        if same_language {
            evidence.push("same language, near-identical → version".to_string());
            WorkKind::Version {
                of: existing_id,
                version_num: 0, // caller will assign
                change_summary: None,
            }
        } else {
            evidence.push("cross-language, near-identical → faithful translation".to_string());
            WorkKind::Translation {
                from: existing_id,
                fidelity: Fidelity::Faithful,
            }
        }
    } else if similarity >= THRESHOLD_DERIVATIVE {
        // Length difference can distinguish summary from paraphrase
        if new_fp.length < existing_fp.length && existing_fp.length > 0 {
            evidence.push("shorter than original → likely summary/derivative".to_string());
            WorkKind::Derivative {
                of: existing_id,
                transform: DerivationKind::Summary,
            }
        } else if same_language {
            evidence.push("same language, moderate similarity → paraphrase".to_string());
            WorkKind::Derivative {
                of: existing_id,
                transform: DerivationKind::Paraphrase,
            }
        } else {
            evidence.push("cross-language, moderate similarity → loose translation".to_string());
            WorkKind::Translation {
                from: existing_id,
                fidelity: Fidelity::Loose,
            }
        }
    } else {
        // 0.50 - 0.70 range
        if same_language {
            evidence.push("same language, low similarity → adaptation".to_string());
            WorkKind::Derivative {
                of: existing_id,
                transform: DerivationKind::Adaptation,
            }
        } else {
            evidence.push("cross-language, low similarity → parallel tradition candidate".to_string());
            WorkKind::Translation {
                from: existing_id,
                fidelity: Fidelity::Adaptation,
            }
        }
    };

    Some(VariantMatch {
        similarity,
        likely_kind,
        evidence,
    })
}

/// Detect if a translation is a retranslation (translation of a translation).
/// Returns true if the source work is itself a translation.
pub fn is_retranslation(source_kind: &WorkKind) -> bool {
    matches!(source_kind, WorkKind::Translation { .. })
}
