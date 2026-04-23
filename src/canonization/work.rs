//! Work — the central unit of content in the canonization system.
//!
//! A Work represents any piece of content: an original text, a translation,
//! a citation, a derivative, or a parallel tradition variant. Every work
//! has a fingerprint for identity detection, an epistemic status, and a
//! quote policy.

use crate::sense_graph::LanguageId;
use super::fingerprint::Fingerprint;
use super::provenance::Provenance;

/// Unique identifier for a work, derived from content hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkId(pub u64);

/// How faithfully a translation tracks the original.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fidelity {
    Faithful,
    Loose,
    Adaptation,
    Retranslation,
}

/// The kind of derivation applied to create new content from a source.
#[derive(Debug, Clone)]
pub enum DerivationKind {
    Summary,
    Paraphrase,
    Commentary,
    Critique,
    Adaptation,
}

/// How much of the original is cited.
#[derive(Debug, Clone, Copy)]
pub enum CitationScope {
    Quote { word_count: u32 },
    Reference,
    Allusion,
}

/// The relationship a work has to other works.
#[derive(Debug, Clone)]
pub enum WorkKind {
    Original,
    Translation {
        from: WorkId,
        fidelity: Fidelity,
    },
    Citation {
        of: WorkId,
        scope: CitationScope,
    },
    Derivative {
        of: WorkId,
        transform: DerivationKind,
    },
    ParallelTradition {
        family: String,
        siblings: Vec<WorkId>,
    },
    Version {
        of: WorkId,
        version_num: u32,
        change_summary: Option<String>,
    },
}

/// Epistemic status — what kind of truth claim does this content make?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EpistemicStatus {
    EmpiricalFact,
    HistoricalRecord,
    Tradition,
    ReligiousNarrative,
    Opinion,
    Theoretical,
    Fiction,
    Mythology,
    Speculation,
    Unknown,
}

/// Quote policy — how may this content be referenced/reproduced?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuotePolicy {
    FreelyQuotable,
    Paraphraseable,
    ConceptOnly,
    Private,
}

/// A work — the canonical unit of content.
#[derive(Debug, Clone)]
pub struct Work {
    pub id: WorkId,
    pub title: Option<String>,
    pub language: LanguageId,
    pub kind: WorkKind,
    pub canonical: Option<WorkId>,
    pub provenance: Provenance,
    pub epistemic: EpistemicStatus,
    pub quote_policy: QuotePolicy,
    pub fingerprint: Fingerprint,
    pub created_at_ms: i64,
}
