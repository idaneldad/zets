//! CanonizationEngine — the main orchestrator for content canonization.
//!
//! Pipeline: text → fingerprint → variant detection → epistemic classification
//! → quote policy → Work record.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::sense_graph::{LanguageId, SenseStore};

use super::error::CanonizationError;
use super::fingerprint::{Fingerprint, compute_fingerprint, fingerprint_similarity};
use super::variant::{VariantMatch, classify_variant, is_retranslation, THRESHOLD_TRANSLATION};
use super::epistemic::classify_epistemic;
use super::policy::derive_quote_policy;
use super::provenance::Provenance;
use super::work::{Work, WorkId, WorkKind, Fidelity, EpistemicStatus, QuotePolicy};

/// Shared reference to a SenseStore (read-only).
pub type SenseStoreRef = Arc<SenseStore>;

/// The canonization engine — manages works, detects variants, classifies content.
#[derive(Debug)]
pub struct CanonizationEngine {
    works: HashMap<WorkId, Work>,
    fingerprint_index: HashMap<Fingerprint, Vec<WorkId>>,
    sense_store: SenseStoreRef,
    next_id_seed: u64,
}

impl CanonizationEngine {
    pub fn new(sense_store: SenseStoreRef) -> Self {
        Self {
            works: HashMap::new(),
            fingerprint_index: HashMap::new(),
            sense_store,
            next_id_seed: 0,
        }
    }

    /// How many works are registered.
    pub fn work_count(&self) -> usize {
        self.works.len()
    }

    /// Access the sense store.
    pub fn sense_store(&self) -> &SenseStore {
        &self.sense_store
    }

    /// Generate a deterministic WorkId from content + provenance.
    fn generate_id(&mut self, text: &str, language: LanguageId, provenance: &Provenance) -> WorkId {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        language.hash(&mut hasher);
        if let Some(ref title) = provenance.source {
            title.hash(&mut hasher);
        }
        if let Some(ref author) = provenance.author {
            author.hash(&mut hasher);
        }
        self.next_id_seed.hash(&mut hasher);
        self.next_id_seed += 1;
        WorkId(hasher.finish())
    }

    // ─── Core operations ───

    /// Register new content. Returns Work with variant detection + epistemic classification.
    pub fn canonize(
        &mut self,
        text: &str,
        language: LanguageId,
        provenance: Provenance,
    ) -> Result<Work, CanonizationError> {
        if text.trim().is_empty() {
            return Err(CanonizationError::InvalidInput(
                "text is empty".to_string(),
            ));
        }

        let fp = compute_fingerprint(text, language, &self.sense_store);
        let variants = self.detect_variants(&fp, language);

        // Determine kind based on best variant match
        let (kind, canonical) = if let Some(best) = variants.first() {
            if best.similarity >= THRESHOLD_TRANSLATION {
                // Check if source is itself a translation (retranslation)
                let source_work = self.works.values()
                    .find(|w| fingerprint_similarity(&w.fingerprint, &fp) >= THRESHOLD_TRANSLATION);

                if let Some(sw) = source_work {
                    if is_retranslation(&sw.kind) {
                        // Mark as retranslation with drift risk
                        (
                            WorkKind::Translation {
                                from: sw.id,
                                fidelity: Fidelity::Retranslation,
                            },
                            sw.canonical.or(Some(sw.id)),
                        )
                    } else {
                        (best.likely_kind.clone(), Some(sw.id))
                    }
                } else {
                    (best.likely_kind.clone(), None)
                }
            } else {
                (best.likely_kind.clone(), None)
            }
        } else {
            (WorkKind::Original, None)
        };

        let epistemic = classify_epistemic(text, &provenance);
        let quote_policy = derive_quote_policy(epistemic, provenance.trust_tier, provenance.license.as_deref());

        let id = self.generate_id(text, language, &provenance);

        let work = Work {
            id,
            title: None,
            language,
            kind,
            canonical,
            provenance,
            epistemic,
            quote_policy,
            fingerprint: fp.clone(),
            created_at_ms: 0, // caller can set real timestamps
        };

        self.works.insert(id, work.clone());
        self.fingerprint_index.entry(fp).or_default().push(id);

        Ok(work)
    }

    /// Detect if this fingerprint is a variant of existing works.
    /// Returns matches sorted by similarity (highest first).
    pub fn detect_variants(
        &self,
        fingerprint: &Fingerprint,
        language: LanguageId,
    ) -> Vec<VariantMatch> {
        let mut matches: Vec<VariantMatch> = Vec::new();

        for work in self.works.values() {
            let same_lang = work.language == language;
            if let Some(m) = classify_variant(fingerprint, &work.fingerprint, work.id, same_lang) {
                matches.push(m);
            }
        }

        matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }

    /// Classify epistemic status from text patterns.
    pub fn classify_epistemic(
        &self,
        text: &str,
        provenance: &Provenance,
    ) -> EpistemicStatus {
        classify_epistemic(text, provenance)
    }

    /// Derive quote policy from epistemic + trust + license.
    pub fn derive_quote_policy(
        &self,
        epistemic: EpistemicStatus,
        trust: super::provenance::TrustTier,
        license: Option<&str>,
    ) -> QuotePolicy {
        derive_quote_policy(epistemic, trust, license)
    }

    /// Retrieve a work by ID.
    pub fn get_work(&self, id: WorkId) -> Option<&Work> {
        self.works.get(&id)
    }

    /// Retrieve the canonical version of a work.
    /// If the work has a canonical pointer, follow it. Otherwise, return the work itself.
    pub fn canonical_of(&self, id: WorkId) -> Option<&Work> {
        let work = self.works.get(&id)?;
        match work.canonical {
            Some(canonical_id) => self.works.get(&canonical_id).or(Some(work)),
            None => Some(work),
        }
    }

    /// Get all variants (translations, versions, derivatives) of a work.
    pub fn variants_of(&self, id: WorkId) -> Vec<&Work> {
        self.works.values()
            .filter(|w| {
                if w.id == id {
                    return false;
                }
                match &w.kind {
                    WorkKind::Translation { from, .. } => *from == id,
                    WorkKind::Citation { of, .. } => *of == id,
                    WorkKind::Derivative { of, .. } => *of == id,
                    WorkKind::Version { of, .. } => *of == id,
                    WorkKind::ParallelTradition { siblings, .. } => siblings.contains(&id),
                    WorkKind::Original => w.canonical == Some(id),
                }
            })
            .collect()
    }

    /// Mark works as parallel tradition siblings.
    /// Updates each work's kind to ParallelTradition with bidirectional links.
    pub fn mark_parallel_tradition(
        &mut self,
        work_ids: &[WorkId],
        family: String,
    ) {
        // First pass: collect siblings for each work
        for &wid in work_ids {
            let siblings: Vec<WorkId> = work_ids.iter()
                .copied()
                .filter(|&id| id != wid)
                .collect();

            if let Some(work) = self.works.get_mut(&wid) {
                work.kind = WorkKind::ParallelTradition {
                    family: family.clone(),
                    siblings,
                };
            }
        }
    }

    /// Register a citation of an existing work.
    pub fn register_citation(
        &mut self,
        text: &str,
        of: WorkId,
        scope: super::work::CitationScope,
        language: LanguageId,
        provenance: Provenance,
    ) -> Result<Work, CanonizationError> {
        if text.trim().is_empty() {
            return Err(CanonizationError::InvalidInput("text is empty".to_string()));
        }

        let fp = compute_fingerprint(text, language, &self.sense_store);
        let epistemic = classify_epistemic(text, &provenance);
        let quote_policy = derive_quote_policy(epistemic, provenance.trust_tier, provenance.license.as_deref());
        let id = self.generate_id(text, language, &provenance);

        let work = Work {
            id,
            title: None,
            language,
            kind: WorkKind::Citation { of, scope },
            canonical: Some(of),
            provenance,
            epistemic,
            quote_policy,
            fingerprint: fp.clone(),
            created_at_ms: 0,
        };

        self.works.insert(id, work.clone());
        self.fingerprint_index.entry(fp).or_default().push(id);

        Ok(work)
    }
}
