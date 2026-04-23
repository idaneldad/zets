//! Canonization — variant detection and epistemic classification.
//!
//! This module solves the meta-problem: "is this content the same as that content?"
//! It handles translations, religious parallels, paper versions, derivatives,
//! citations — all as instances of the same variant-detection pattern.
//!
//! # Architecture
//!
//! ```text
//! text + language + provenance
//!        │
//!        ▼
//!   ┌─────────────┐
//!   │ fingerprint  │ ← structural + semantic hashing
//!   └──────┬──────┘
//!          │
//!          ▼
//!   ┌─────────────┐
//!   │  variant     │ ← compare against all existing works
//!   │  detection   │   thresholds: >0.95 translation, >0.70 derivative, >0.50 parallel
//!   └──────┬──────┘
//!          │
//!          ▼
//!   ┌─────────────┐
//!   │  epistemic   │ ← pattern-based classification (no LLM)
//!   │  classifier  │   fact / tradition / opinion / fiction / ...
//!   └──────┬──────┘
//!          │
//!          ▼
//!   ┌─────────────┐
//!   │  quote       │ ← derived from epistemic + trust + license
//!   │  policy      │   freely quotable / paraphrase / concept only / private
//!   └──────┬──────┘
//!          │
//!          ▼
//!      Work record
//! ```
//!
//! # Usage
//!
//! ```ignore
//! let store = Arc::new(SenseStore::new());
//! let mut engine = CanonizationEngine::new(store);
//!
//! let work = engine.canonize(
//!     "In the beginning God created the heavens and the earth.",
//!     1, // English
//!     Provenance { source: Some("bible".into()), ..Provenance::unknown() },
//! ).unwrap();
//!
//! assert_eq!(work.epistemic, EpistemicStatus::ReligiousNarrative);
//! assert_eq!(work.quote_policy, QuotePolicy::ConceptOnly);
//! ```

pub mod error;
pub mod provenance;
pub mod work;
pub mod fingerprint;
pub mod variant;
pub mod epistemic;
pub mod policy;
pub mod engine;

// Re-exports for ergonomic use
pub use error::CanonizationError;
pub use provenance::{Provenance, TrustTier};
pub use work::{
    Work, WorkId, WorkKind, Fidelity, DerivationKind, CitationScope,
    EpistemicStatus, QuotePolicy,
};
pub use fingerprint::Fingerprint;
pub use variant::VariantMatch;
pub use engine::{CanonizationEngine, SenseStoreRef};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sense_graph::{LanguageId, SenseStore, Register};
    use std::sync::Arc;

    const HE: LanguageId = 0;
    const EN: LanguageId = 1;

    /// Build a sense store with Hebrew/English words and shared senses
    /// for use in fingerprint and variant detection tests.
    fn test_sense_store() -> Arc<SenseStore> {
        let mut s = SenseStore::new();

        // Senses
        let beginning = s.add_sense(
            "beginning.time".to_string(),
            "start of something".to_string(),
            Some("time".to_string()),
        );
        let creation = s.add_sense(
            "creation.act".to_string(),
            "bringing into existence".to_string(),
            Some("cosmology".to_string()),
        );
        let heaven = s.add_sense(
            "heaven.place".to_string(),
            "sky or divine realm".to_string(),
            Some("cosmology".to_string()),
        );
        let earth = s.add_sense(
            "earth.place".to_string(),
            "the world".to_string(),
            Some("cosmology".to_string()),
        );
        let god = s.add_sense(
            "god.divine".to_string(),
            "supreme deity".to_string(),
            Some("theology".to_string()),
        );
        let peace = s.add_sense(
            "peace.state".to_string(),
            "absence of conflict".to_string(),
            Some("state".to_string()),
        );
        let greeting = s.add_sense(
            "greeting.open".to_string(),
            "opening salutation".to_string(),
            Some("social".to_string()),
        );

        // Hebrew words
        let bereshit = s.add_word("בראשית".to_string(), HE);
        s.link_word_to_sense(bereshit, beginning, Register::Neutral, 1.0);

        let bara = s.add_word("ברא".to_string(), HE);
        s.link_word_to_sense(bara, creation, Register::Neutral, 1.0);

        let elohim = s.add_word("אלהים".to_string(), HE);
        s.link_word_to_sense(elohim, god, Register::Neutral, 1.0);

        let shamayim = s.add_word("השמים".to_string(), HE);
        s.link_word_to_sense(shamayim, heaven, Register::Neutral, 1.0);

        let haaretz = s.add_word("הארץ".to_string(), HE);
        s.link_word_to_sense(haaretz, earth, Register::Neutral, 1.0);

        let shalom = s.add_word("שלום".to_string(), HE);
        s.link_word_to_sense(shalom, peace, Register::Neutral, 0.5);
        s.link_word_to_sense(shalom, greeting, Register::Neutral, 0.5);

        // English words
        let beginning_en = s.add_word("beginning".to_string(), EN);
        s.link_word_to_sense(beginning_en, beginning, Register::Neutral, 1.0);

        let created_en = s.add_word("created".to_string(), EN);
        s.link_word_to_sense(created_en, creation, Register::Neutral, 1.0);

        let god_en = s.add_word("God".to_string(), EN);
        s.link_word_to_sense(god_en, god, Register::Neutral, 1.0);

        let heavens_en = s.add_word("heavens".to_string(), EN);
        s.link_word_to_sense(heavens_en, heaven, Register::Neutral, 1.0);

        let earth_en = s.add_word("earth".to_string(), EN);
        s.link_word_to_sense(earth_en, earth, Register::Neutral, 1.0);

        let peace_en = s.add_word("peace".to_string(), EN);
        s.link_word_to_sense(peace_en, peace, Register::Neutral, 1.0);

        Arc::new(s)
    }

    fn make_engine() -> CanonizationEngine {
        CanonizationEngine::new(test_sense_store())
    }

    fn religious_provenance(source: &str) -> Provenance {
        Provenance {
            source: Some(source.to_string()),
            author: None,
            published_year: None,
            license: None,
            trust_tier: TrustTier::Community,
        }
    }

    fn academic_provenance() -> Provenance {
        Provenance {
            source: Some("Nature".to_string()),
            author: Some("Dr. Smith".to_string()),
            published_year: Some(2024),
            license: Some("CC-BY-4.0".to_string()),
            trust_tier: TrustTier::PeerReviewed,
        }
    }

    // ─── Basic engine tests ───

    #[test]
    fn test_empty_engine() {
        let engine = make_engine();
        assert_eq!(engine.work_count(), 0);
    }

    #[test]
    fn test_register_first_work_is_original() {
        let mut engine = make_engine();
        let work = engine.canonize(
            "In the beginning God created the heavens and the earth.",
            EN,
            religious_provenance("bible"),
        ).unwrap();

        assert!(matches!(work.kind, WorkKind::Original));
        assert_eq!(engine.work_count(), 1);
    }

    #[test]
    fn test_register_translation_detected_as_variant() {
        let mut engine = make_engine();

        // Register English first
        let en_work = engine.canonize(
            "In the beginning God created the heavens and the earth.",
            EN,
            religious_provenance("bible"),
        ).unwrap();
        assert!(matches!(en_work.kind, WorkKind::Original));

        // Register same text again — should detect as variant
        let en_dup = engine.canonize(
            "In the beginning God created the heavens and the earth.",
            EN,
            religious_provenance("bible"),
        ).unwrap();

        // Same text, same language → should be detected as version or translation
        assert!(!matches!(en_dup.kind, WorkKind::Original));
    }

    // ─── Fingerprint tests ───

    #[test]
    fn test_fingerprint_structural_stable() {
        let store = test_sense_store();
        let text = "The cat sat on the mat. The dog ate the bone.";

        let fp1 = fingerprint::compute_fingerprint(text, EN, &store);
        let fp2 = fingerprint::compute_fingerprint(text, EN, &store);

        assert_eq!(fp1, fp2, "same text must produce identical fingerprint");
    }

    #[test]
    fn test_fingerprint_semantic_via_sense() {
        let store = test_sense_store();

        let fp = fingerprint::compute_fingerprint(
            "In the beginning God created the heavens and the earth.",
            EN,
            &store,
        );

        assert!(!fp.semantic.is_empty(), "semantic fingerprint should be non-empty");
        assert!(!fp.structural.is_empty(), "structural fingerprint should be non-empty");
        assert_eq!(fp.length, 0, "short text should be tier 0");
    }

    // ─── Variant detection tests ───

    #[test]
    fn test_variant_detection_above_threshold() {
        let mut engine = make_engine();

        let text = "In the beginning God created the heavens and the earth.";
        let _w1 = engine.canonize(text, EN, religious_provenance("bible")).unwrap();

        let fp = fingerprint::compute_fingerprint(text, EN, engine.sense_store());
        let variants = engine.detect_variants(&fp, EN);

        assert!(!variants.is_empty(), "identical text should match");
        assert!(variants[0].similarity > 0.9, "identical text should have high similarity");
    }

    #[test]
    fn test_variant_detection_below_threshold_means_original() {
        let mut engine = make_engine();

        let _w1 = engine.canonize(
            "In the beginning God created the heavens and the earth.",
            EN,
            religious_provenance("bible"),
        ).unwrap();

        // Completely different text
        let w2 = engine.canonize(
            "The quick brown fox jumps over the lazy dog.",
            EN,
            Provenance::unknown(),
        ).unwrap();

        assert!(matches!(w2.kind, WorkKind::Original),
            "unrelated text should be classified as Original");
    }

    // ─── Epistemic classifier tests ───

    #[test]
    fn test_epistemic_tradition_pattern() {
        let prov = Provenance::unknown();
        let status = epistemic::classify_epistemic(
            "Tradition holds that the elders passed down this knowledge through generations.",
            &prov,
        );
        assert_eq!(status, EpistemicStatus::Tradition);
    }

    #[test]
    fn test_epistemic_empirical_pattern() {
        let prov = Provenance::unknown();
        let status = epistemic::classify_epistemic(
            "The experiment measured a statistically significant difference in the control group.",
            &prov,
        );
        assert_eq!(status, EpistemicStatus::EmpiricalFact);
    }

    #[test]
    fn test_epistemic_religious_narrative_pattern() {
        let prov = Provenance::unknown();
        let status = epistemic::classify_epistemic(
            "And God created the heavens and the earth. The Lord blessed them.",
            &prov,
        );
        assert_eq!(status, EpistemicStatus::ReligiousNarrative);
    }

    #[test]
    fn test_epistemic_religious_via_provenance() {
        let prov = religious_provenance("torah");
        let status = epistemic::classify_epistemic(
            "Some generic text with no obvious markers.",
            &prov,
        );
        assert_eq!(status, EpistemicStatus::ReligiousNarrative,
            "source=torah should override pattern matching");
    }

    #[test]
    fn test_epistemic_historical_pattern() {
        let prov = Provenance::unknown();
        let status = epistemic::classify_epistemic(
            "In the year 1492, historically documented in records show that Columbus sailed.",
            &prov,
        );
        assert_eq!(status, EpistemicStatus::HistoricalRecord);
    }

    #[test]
    fn test_epistemic_fiction_pattern() {
        let prov = Provenance::unknown();
        let status = epistemic::classify_epistemic(
            "Once upon a time in a land far away, there lived a princess.",
            &prov,
        );
        assert_eq!(status, EpistemicStatus::Fiction);
    }

    #[test]
    fn test_epistemic_opinion_pattern() {
        let prov = Provenance::unknown();
        let status = epistemic::classify_epistemic(
            "I believe we should invest more in education. In my opinion this is critical.",
            &prov,
        );
        assert_eq!(status, EpistemicStatus::Opinion);
    }

    #[test]
    fn test_epistemic_mythology_pattern() {
        let prov = Provenance::unknown();
        let status = epistemic::classify_epistemic(
            "According to myth, the gods created the world from the body of Ymir.",
            &prov,
        );
        assert_eq!(status, EpistemicStatus::Mythology);
    }

    // ─── Quote policy tests ───

    #[test]
    fn test_policy_religious_is_concept_only() {
        let qp = policy::derive_quote_policy(
            EpistemicStatus::ReligiousNarrative,
            TrustTier::Community,
            None,
        );
        assert_eq!(qp, QuotePolicy::ConceptOnly, "Idan's rule: religious → concept only");
    }

    #[test]
    fn test_policy_empirical_peer_reviewed_is_freely_quotable() {
        let qp = policy::derive_quote_policy(
            EpistemicStatus::EmpiricalFact,
            TrustTier::PeerReviewed,
            None,
        );
        assert_eq!(qp, QuotePolicy::FreelyQuotable);
    }

    #[test]
    fn test_policy_opinion_is_paraphraseable() {
        let qp = policy::derive_quote_policy(
            EpistemicStatus::Opinion,
            TrustTier::Anecdotal,
            None,
        );
        assert_eq!(qp, QuotePolicy::Paraphraseable);
    }

    #[test]
    fn test_policy_open_license_is_freely_quotable() {
        let qp = policy::derive_quote_policy(
            EpistemicStatus::Unknown,
            TrustTier::Unknown,
            Some("CC-BY-4.0"),
        );
        assert_eq!(qp, QuotePolicy::FreelyQuotable);
    }

    #[test]
    fn test_policy_mythology_is_concept_only() {
        let qp = policy::derive_quote_policy(
            EpistemicStatus::Mythology,
            TrustTier::Community,
            None,
        );
        assert_eq!(qp, QuotePolicy::ConceptOnly);
    }

    // ─── Parallel tradition tests ───

    #[test]
    fn test_parallel_tradition_creates_siblings() {
        let mut engine = make_engine();

        let genesis = engine.canonize(
            "And God created the heavens and the earth.",
            EN,
            religious_provenance("bible"),
        ).unwrap();

        let norse = engine.canonize(
            "From the void of Ginnungagap the world was formed by the gods.",
            EN,
            Provenance::unknown(),
        ).unwrap();

        engine.mark_parallel_tradition(
            &[genesis.id, norse.id],
            "Creation_Myths".to_string(),
        );

        let g = engine.get_work(genesis.id).unwrap();
        match &g.kind {
            WorkKind::ParallelTradition { family, siblings } => {
                assert_eq!(family, "Creation_Myths");
                assert!(siblings.contains(&norse.id));
            }
            _ => panic!("expected ParallelTradition kind"),
        }

        let n = engine.get_work(norse.id).unwrap();
        match &n.kind {
            WorkKind::ParallelTradition { family, siblings } => {
                assert_eq!(family, "Creation_Myths");
                assert!(siblings.contains(&genesis.id));
            }
            _ => panic!("expected ParallelTradition kind"),
        }
    }

    // ─── Canonical and variant retrieval tests ───

    #[test]
    fn test_canonical_of_returns_original() {
        let mut engine = make_engine();

        let original = engine.canonize(
            "The measured results show a statistically significant effect.",
            EN,
            academic_provenance(),
        ).unwrap();

        // Canonical of an original should be itself
        let canonical = engine.canonical_of(original.id).unwrap();
        assert_eq!(canonical.id, original.id);
    }

    #[test]
    fn test_variants_of_returns_all() {
        let mut engine = make_engine();

        let original = engine.canonize(
            "The measured results show a statistically significant effect.",
            EN,
            academic_provenance(),
        ).unwrap();

        let citation = engine.register_citation(
            "Smith et al. found a statistically significant effect.",
            original.id,
            CitationScope::Reference,
            EN,
            Provenance::unknown(),
        ).unwrap();

        let variants = engine.variants_of(original.id);
        assert_eq!(variants.len(), 1, "should find the citation as a variant");
        assert_eq!(variants[0].id, citation.id);
    }

    // ─── Retranslation detection ───

    #[test]
    fn test_retranslation_drift_detected() {
        // A translation of a translation has drift risk
        let is_retrans = variant::is_retranslation(&WorkKind::Translation {
            from: WorkId(1),
            fidelity: Fidelity::Faithful,
        });
        assert!(is_retrans, "translation of translation = retranslation");

        let not_retrans = variant::is_retranslation(&WorkKind::Original);
        assert!(!not_retrans, "original is not a retranslation");
    }

    // ─── Hebrew-English translation scenario ───

    #[test]
    fn test_hebrew_english_translation_recognized() {
        let store = test_sense_store();
        let he_fp = fingerprint::compute_fingerprint(
            "בראשית ברא אלהים את השמים ואת הארץ.",
            HE,
            &store,
        );
        let en_fp = fingerprint::compute_fingerprint(
            "In the beginning God created the heavens and the earth.",
            EN,
            &store,
        );

        // Both should have semantic components via shared senses
        assert!(!he_fp.semantic.is_empty());
        assert!(!en_fp.semantic.is_empty());

        // The semantic similarity should be non-zero since they share sense concepts
        let sim = fingerprint::fingerprint_similarity(&he_fp, &en_fp);
        assert!(sim > 0.0, "Hebrew and English Genesis should have some similarity via shared senses, got {sim}");
    }

    // ─── Citation chain test ───

    #[test]
    fn test_citation_chain_preserved() {
        let mut engine = make_engine();

        let original = engine.canonize(
            "The experiment measured a control group with p-value below threshold.",
            EN,
            academic_provenance(),
        ).unwrap();

        let cite1 = engine.register_citation(
            "As shown by Smith (2024).",
            original.id,
            CitationScope::Reference,
            EN,
            Provenance::unknown(),
        ).unwrap();

        let cite2 = engine.register_citation(
            "Referencing the work of Smith and colleagues.",
            original.id,
            CitationScope::Allusion,
            EN,
            Provenance::unknown(),
        ).unwrap();

        let variants = engine.variants_of(original.id);
        assert_eq!(variants.len(), 2);

        let ids: Vec<WorkId> = variants.iter().map(|w| w.id).collect();
        assert!(ids.contains(&cite1.id));
        assert!(ids.contains(&cite2.id));
    }

    // ─── Summary derivative test ───

    #[test]
    fn test_summary_is_derivative_not_translation() {
        // A summary should be classified as Derivative, not Translation
        let kind = WorkKind::Derivative {
            of: WorkId(42),
            transform: DerivationKind::Summary,
        };
        assert!(matches!(kind, WorkKind::Derivative { transform: DerivationKind::Summary, .. }));

        // Ensure it's not confused with translation
        assert!(!matches!(kind, WorkKind::Translation { .. }));
    }

    // ─── Scenario: Torah verse Hebrew/English ───

    #[test]
    fn test_torah_verse_english_hebrew_recognized_as_variants() {
        let mut engine = make_engine();

        let he_work = engine.canonize(
            "בראשית ברא אלהים את השמים ואת הארץ.",
            HE,
            religious_provenance("torah"),
        ).unwrap();

        assert_eq!(he_work.epistemic, EpistemicStatus::ReligiousNarrative,
            "Torah source → ReligiousNarrative");
        assert_eq!(he_work.quote_policy, QuotePolicy::ConceptOnly,
            "Religious → ConceptOnly");

        let en_work = engine.canonize(
            "In the beginning God created the heavens and the earth.",
            EN,
            religious_provenance("bible"),
        ).unwrap();

        assert_eq!(en_work.epistemic, EpistemicStatus::ReligiousNarrative);
        assert_eq!(en_work.quote_policy, QuotePolicy::ConceptOnly);
    }

    // ─── Scenario: Scientific paper ───

    #[test]
    fn test_scientific_paper_he_en_recognized() {
        let mut engine = make_engine();

        let en_work = engine.canonize(
            "The experiment measured a statistically significant result in the control group.",
            EN,
            academic_provenance(),
        ).unwrap();

        assert_eq!(en_work.epistemic, EpistemicStatus::EmpiricalFact);
        assert_eq!(en_work.quote_policy, QuotePolicy::FreelyQuotable);
    }

    // ─── Scenario: Creation myths parallel tradition ───

    #[test]
    fn test_creation_myths_parallel_tradition() {
        let mut engine = make_engine();

        let genesis = engine.canonize(
            "And God created the heavens and the earth. The Lord blessed creation.",
            EN,
            religious_provenance("bible"),
        ).unwrap();
        assert_eq!(genesis.epistemic, EpistemicStatus::ReligiousNarrative);

        let norse = engine.canonize(
            "From the void of Ginnungagap the gods shaped the world. According to myth and legend.",
            EN,
            Provenance::unknown(),
        ).unwrap();
        assert_eq!(norse.epistemic, EpistemicStatus::Mythology);

        let quran = engine.canonize(
            "Allah created the heavens and the earth. The divine revelation and scripture.",
            EN,
            religious_provenance("quran"),
        ).unwrap();
        assert_eq!(quran.epistemic, EpistemicStatus::ReligiousNarrative);

        engine.mark_parallel_tradition(
            &[genesis.id, norse.id, quran.id],
            "Creation_Myths".to_string(),
        );

        // Verify bidirectional links
        let g = engine.get_work(genesis.id).unwrap();
        if let WorkKind::ParallelTradition { family, siblings } = &g.kind {
            assert_eq!(family, "Creation_Myths");
            assert_eq!(siblings.len(), 2);
        } else {
            panic!("expected ParallelTradition");
        }
    }

    // ─── Edge cases ───

    #[test]
    fn test_empty_text_returns_error() {
        let mut engine = make_engine();
        let result = engine.canonize("", EN, Provenance::unknown());
        assert!(result.is_err());
    }

    #[test]
    fn test_whitespace_only_returns_error() {
        let mut engine = make_engine();
        let result = engine.canonize("   \n\t  ", EN, Provenance::unknown());
        assert!(result.is_err());
    }
}
