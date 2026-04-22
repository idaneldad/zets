//! Distillation — promote episodic observations to semantic learned patterns.
//!
//! Phase 3 of the learning layer. Takes the Observed edges that dialogue
//! ingestion produces and finds PATTERNS that repeat across conversations.
//!
//! Phase-appropriate approach: we're NOT doing neural clustering yet. We use
//! deterministic co-occurrence counting:
//!
//!   - For each (intent, emotion) pair, count how many utterances exhibit it
//!   - If count >= THRESHOLD, create a Prototype atom representing that pair
//!   - Link it to the category atoms with Provenance::Learned (confidence =
//!     f(count, stability))
//!   - Keep 2-3 exemplar utterances linked to the prototype (Observed)
//!   - The raw utterances are NEITHER deleted NOR touched — Phase A principle:
//!     "archive, don't delete"
//!
//! This is enough to show the loop closing:
//!   Observed utterances → clustering → Learned prototype → new queries can
//!   see the pattern in Precision mode (because Learned+high-confidence passes).
//!
//! When we later add real feature embeddings (Phase 4), the cluster finder
//! swaps from co-occurrence counts to k-means/DBSCAN over vectors. The rest
//! of the pipeline (threshold → promote → exemplars) stays identical.

use std::collections::HashMap;

use crate::atoms::{AtomId, AtomKind, AtomStore};
use crate::learning_layer::{
    EdgeKey, Prototype, ProtoBank, ProvenanceLog, ProvenanceRecord,
};
use crate::relations;

/// Tuning knobs for distillation.
#[derive(Debug, Clone)]
pub struct DistillConfig {
    /// Minimum co-occurrence count before a pattern qualifies as a Prototype
    pub min_cluster_size: u32,
    /// How many exemplars to keep per prototype (typically 2-3)
    pub exemplars_per_proto: usize,
    /// Base confidence for Learned edges (clamped to [50..250])
    pub base_confidence: u8,
    /// Confidence bonus per extra observation beyond min_cluster_size
    pub confidence_per_obs: u8,
}

impl Default for DistillConfig {
    fn default() -> Self {
        Self {
            min_cluster_size: 2,        // 2+ observations = tentative pattern
            exemplars_per_proto: 3,
            base_confidence: 150,       // starts below Precision threshold (200)
            confidence_per_obs: 10,     // each extra obs tightens trust
        }
    }
}

/// Result of a distillation pass.
#[derive(Debug, Clone, Default)]
pub struct DistillResult {
    /// Prototypes created in this pass (may be 0 if no clusters hit threshold)
    pub prototypes_created: Vec<AtomId>,
    /// Learned edges tagged during this pass
    pub learned_edges_tagged: usize,
    /// Observed utterances touched (not deleted — just indexed)
    pub observations_processed: usize,
    /// Patterns considered but below threshold (for monitoring)
    pub below_threshold: Vec<(String, u32)>,
}

/// Find (intent, emotion) pairs that appear frequently across utterances
/// and promote them to Prototype atoms with Learned edges.
///
/// What this does:
///   1. Walks every Text atom with the prefix "utt:" (from dialogue::ingest_dialogue)
///   2. For each, looks up its connected intent and emotion atoms
///   3. Counts co-occurrences of each (intent, emotion) pair
///   4. For pairs meeting `min_cluster_size`, creates a Prototype atom
///      linked to the intent and emotion categories via Learned edges
///   5. Links 2-3 exemplar utterances (Observed) to the prototype for audit
///
/// Idempotent: re-running on the same graph creates no new prototypes
/// if the clusters haven't grown. Re-running after new data may update
/// `observation_count` on existing prototypes (future enhancement).
pub fn distill_dialogue_patterns(
    store: &mut AtomStore,
    prov_log: &mut ProvenanceLog,
    proto_bank: &mut ProtoBank,
    config: &DistillConfig,
) -> DistillResult {
    let mut result = DistillResult::default();

    // 1. Scan for utterance atoms (data prefix "utt:")
    let utterances = collect_utterance_atoms(store);
    result.observations_processed = utterances.len();

    // 2. For each utterance, extract (intent_atom_id, emotion_atom_id) using
    //    the existing has_attribute and expresses_emotion edges
    let has_attr = relations::by_name("has_attribute").unwrap().code;
    let expresses = relations::by_name("expresses_emotion")
        .map(|r| r.code).unwrap_or(has_attr);

    let mut clusters: HashMap<(AtomId, Option<AtomId>), Vec<AtomId>> = HashMap::new();
    for &utt_atom in &utterances {
        let mut intent_atom: Option<AtomId> = None;
        let mut emotion_atom: Option<AtomId> = None;

        for edge in store.outgoing(utt_atom) {
            if let Some(target) = store.get(edge.to) {
                if let Ok(label) = std::str::from_utf8(&target.data) {
                    if edge.relation == has_attr && label.starts_with("intent:") {
                        intent_atom = Some(edge.to);
                    } else if edge.relation == expresses && label.starts_with("emotion:") {
                        emotion_atom = Some(edge.to);
                    }
                }
            }
        }

        if let Some(int) = intent_atom {
            clusters.entry((int, emotion_atom)).or_default().push(utt_atom);
        }
    }

    // 3. Promote clusters above threshold to Prototypes
    let is_a = relations::by_name("is_a").unwrap().code;
    for ((intent_atom, emotion_opt), members) in clusters {
        let count = members.len() as u32;

        // Resolve human-readable labels
        let intent_label = atom_label(store, intent_atom).unwrap_or_else(|| "unknown".into());
        let emotion_label = emotion_opt
            .and_then(|e| atom_label(store, e))
            .unwrap_or_else(|| "emotion:neutral".into());

        let pattern_name = format!("pattern:{}/{}",
            intent_label.trim_start_matches("intent:"),
            emotion_label.trim_start_matches("emotion:"));

        if count < config.min_cluster_size {
            result.below_threshold.push((pattern_name, count));
            continue;
        }

        // Skip if already exists
        if proto_bank.by_name(&pattern_name).is_some() {
            continue;
        }

        // Create Prototype atom
        let proto_atom = store.put(
            AtomKind::Concept,
            format!("proto:{}", pattern_name).into_bytes(),
        );

        // Pick exemplars — up to N from the cluster
        let exemplars: Vec<AtomId> = members.iter()
            .take(config.exemplars_per_proto)
            .copied()
            .collect();

        // Confidence = base + per-obs bonus, clamped
        let bonus = ((count.saturating_sub(config.min_cluster_size)) as u16)
            .saturating_mul(config.confidence_per_obs as u16);
        let confidence = ((config.base_confidence as u16).saturating_add(bonus))
            .min(250) as u8;

        // Link prototype to its category components with Learned edges
        // proto --has_attribute--> intent_atom (Learned, high conf)
        store.link(proto_atom, intent_atom, has_attr, 90, 0);
        prov_log.tag(
            EdgeKey::new(proto_atom, intent_atom, has_attr),
            ProvenanceRecord::learned(confidence),
        );
        result.learned_edges_tagged += 1;

        if let Some(emo) = emotion_opt {
            store.link(proto_atom, emo, expresses, 85, 0);
            prov_log.tag(
                EdgeKey::new(proto_atom, emo, expresses),
                ProvenanceRecord::learned(confidence),
            );
            result.learned_edges_tagged += 1;
        }

        // Meta: proto is_a "category:pattern"
        let proto_cat = store.put(
            AtomKind::Concept,
            b"category:pattern".to_vec(),
        );
        store.link(proto_atom, proto_cat, is_a, 95, 0);
        prov_log.tag(
            EdgeKey::new(proto_atom, proto_cat, is_a),
            ProvenanceRecord::asserted(),
        );

        // Link exemplars to prototype (Observed — they're still specific instances)
        let part_of = relations::by_name("part_of").unwrap().code;
        for &ex in &exemplars {
            store.link(ex, proto_atom, part_of, 70, 0);
            prov_log.tag(
                EdgeKey::new(ex, proto_atom, part_of),
                ProvenanceRecord::observed(),
            );
        }

        // Register in the bank
        proto_bank.register(Prototype {
            atom_id: proto_atom,
            name: pattern_name.clone(),
            domain: "dialogue_pattern".to_string(),
            observation_count: count,
            drift: 0.0,  // fresh cluster, no history yet
            exemplars,
            last_distilled: "2026-04-22".to_string(),
        });

        result.prototypes_created.push(proto_atom);
    }

    result
}

/// Collect AtomIds of all atoms whose data begins with "utt:".
fn collect_utterance_atoms(store: &AtomStore) -> Vec<AtomId> {
    let (atoms, _) = store.snapshot();
    atoms.iter().enumerate()
        .filter_map(|(i, a)| {
            if a.kind == AtomKind::Text {
                if let Ok(s) = std::str::from_utf8(&a.data) {
                    if s.starts_with("utt:") {
                        return Some(i as AtomId);
                    }
                }
            }
            None
        })
        .collect()
}

/// Extract the human-readable label from an atom's data bytes.
fn atom_label(store: &AtomStore, atom_id: AtomId) -> Option<String> {
    store.get(atom_id)
        .and_then(|a| std::str::from_utf8(&a.data).ok().map(String::from))
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialogue::{
        Conversation, ConvOutcome, DialogTurn, Emotion, Intent, ingest_dialogue,
    };

    fn build_empathy_corpus() -> (AtomStore, ProvenanceLog) {
        let mut store = AtomStore::new();
        let mut log = ProvenanceLog::new();
        let user = store.put(AtomKind::Concept, b"speaker:user".to_vec());
        let ai = store.put(AtomKind::Concept, b"speaker:ai".to_vec());

        // Two conversations where user expresses sadness via Inform intent
        let convs = [
            Conversation {
                id: "cA".into(), source: "test".into(), outcome: ConvOutcome::Resolved,
                turns: vec![
                    DialogTurn { speaker: user, text: "I lost my job".into(),
                        intent: Intent::Inform, emotion: Emotion::Sadness, turn_index: 0 },
                    DialogTurn { speaker: ai, text: "That's hard".into(),
                        intent: Intent::Empathize, emotion: Emotion::Neutral, turn_index: 1 },
                ],
            },
            Conversation {
                id: "cB".into(), source: "test".into(), outcome: ConvOutcome::Resolved,
                turns: vec![
                    DialogTurn { speaker: user, text: "My pet died".into(),
                        intent: Intent::Inform, emotion: Emotion::Sadness, turn_index: 0 },
                    DialogTurn { speaker: ai, text: "I'm sorry".into(),
                        intent: Intent::Empathize, emotion: Emotion::Neutral, turn_index: 1 },
                ],
            },
        ];

        for c in &convs {
            ingest_dialogue(&mut store, &mut log, c);
        }
        (store, log)
    }

    #[test]
    fn distill_with_empty_store_produces_no_prototypes() {
        let mut store = AtomStore::new();
        let mut log = ProvenanceLog::new();
        let mut bank = ProtoBank::new();
        let result = distill_dialogue_patterns(&mut store, &mut log, &mut bank,
                                                &DistillConfig::default());
        assert!(result.prototypes_created.is_empty());
        assert_eq!(result.observations_processed, 0);
    }

    #[test]
    fn distill_finds_recurring_inform_sadness_pattern() {
        let (mut store, mut log) = build_empathy_corpus();
        let mut bank = ProtoBank::new();

        let before_protos = bank.len();
        let result = distill_dialogue_patterns(&mut store, &mut log, &mut bank,
                                                &DistillConfig::default());

        // 4 utterances total, 2 are (Inform, Sadness), 2 are (Empathize, Neutral)
        assert_eq!(result.observations_processed, 4);
        assert!(result.prototypes_created.len() >= 1,
            "should find at least one recurring pattern");
        assert!(bank.len() > before_protos);
        assert!(result.learned_edges_tagged >= 1);
    }

    #[test]
    fn distill_creates_learned_edges_tagged() {
        let (mut store, mut log) = build_empathy_corpus();
        let mut bank = ProtoBank::new();

        let learned_before = log.counts().get(&crate::learning_layer::Provenance::Learned)
            .copied().unwrap_or(0);
        let _ = distill_dialogue_patterns(&mut store, &mut log, &mut bank,
                                           &DistillConfig::default());
        let learned_after = log.counts().get(&crate::learning_layer::Provenance::Learned)
            .copied().unwrap_or(0);
        assert!(learned_after > learned_before,
            "distillation should create Learned edges ({} -> {})",
            learned_before, learned_after);
    }

    #[test]
    fn distill_below_threshold_rejected() {
        let (mut store, mut log) = build_empathy_corpus();
        let mut bank = ProtoBank::new();

        // Threshold of 10 — nothing should make it
        let config = DistillConfig { min_cluster_size: 10, ..Default::default() };
        let result = distill_dialogue_patterns(&mut store, &mut log, &mut bank, &config);
        assert!(result.prototypes_created.is_empty());
        assert!(!result.below_threshold.is_empty(),
            "below_threshold should list the rejected candidates");
    }

    #[test]
    fn distill_idempotent() {
        let (mut store, mut log) = build_empathy_corpus();
        let mut bank = ProtoBank::new();
        let config = DistillConfig::default();

        let r1 = distill_dialogue_patterns(&mut store, &mut log, &mut bank, &config);
        let protos_after_first = bank.len();
        let r2 = distill_dialogue_patterns(&mut store, &mut log, &mut bank, &config);

        assert_eq!(bank.len(), protos_after_first,
            "second run should not add new prototypes");
        assert_eq!(r2.prototypes_created.len(), 0);
        let _ = r1;
    }

    #[test]
    fn distill_exemplars_attached_to_prototype() {
        let (mut store, mut log) = build_empathy_corpus();
        let mut bank = ProtoBank::new();
        let _ = distill_dialogue_patterns(&mut store, &mut log, &mut bank,
                                           &DistillConfig::default());

        // Every registered prototype should have at least one exemplar
        for i in 0..bank.len() {
            let proto = bank.get(i).unwrap();
            assert!(!proto.exemplars.is_empty(),
                "prototype '{}' has no exemplars", proto.name);
        }
    }

    #[test]
    fn distill_confidence_grows_with_cluster_size() {
        let (mut store, mut log) = build_empathy_corpus();
        let mut bank = ProtoBank::new();
        let config = DistillConfig {
            min_cluster_size: 2, base_confidence: 100, confidence_per_obs: 20,
            exemplars_per_proto: 3,
        };
        let _ = distill_dialogue_patterns(&mut store, &mut log, &mut bank, &config);

        // Inform/Sadness cluster has 2 members — confidence = 100 + 0 = 100
        // (2 - min(2) = 0 bonus observations)
        // This test just verifies the confidence is plausible (>= base)
        let learned = log.filter(|r| r.provenance == crate::learning_layer::Provenance::Learned);
        assert!(!learned.is_empty());
        for (_, rec) in &learned {
            assert!(rec.confidence >= config.base_confidence,
                "learned confidence {} below base {}", rec.confidence, config.base_confidence);
        }
    }

    #[test]
    fn distill_config_defaults_reasonable() {
        let cfg = DistillConfig::default();
        assert!(cfg.min_cluster_size >= 2);
        assert!(cfg.exemplars_per_proto >= 2);
        assert!(cfg.base_confidence < 200,
            "base should be below Precision threshold — needs to earn trust");
    }

    #[test]
    fn collect_utterance_atoms_finds_only_utt_prefix() {
        let mut store = AtomStore::new();
        let _a = store.put(AtomKind::Concept, b"speaker:user".to_vec());
        let _b = store.put(AtomKind::Text, b"utt:test:c1:0".to_vec());
        let _c = store.put(AtomKind::Text, b"I lost my job".to_vec());  // content, no prefix
        let _d = store.put(AtomKind::Text, b"utt:test:c1:1".to_vec());

        let utts = collect_utterance_atoms(&store);
        assert_eq!(utts.len(), 2);
    }

    #[test]
    fn distill_tags_prototype_as_category() {
        let (mut store, mut log) = build_empathy_corpus();
        let mut bank = ProtoBank::new();
        let _ = distill_dialogue_patterns(&mut store, &mut log, &mut bank,
                                           &DistillConfig::default());

        // Verify the category:pattern atom exists
        let hash = crate::atoms::content_hash(b"category:pattern");
        let (atoms, _) = store.snapshot();
        let has_cat = atoms.iter().any(|a| a.content_hash == hash);
        assert!(has_cat, "category:pattern atom should exist after distillation");
    }

    #[test]
    fn distill_no_double_register_across_passes() {
        let (mut store, mut log) = build_empathy_corpus();
        let mut bank = ProtoBank::new();
        let config = DistillConfig::default();

        // Three passes in a row — should all be no-ops after the first
        let r1 = distill_dialogue_patterns(&mut store, &mut log, &mut bank, &config);
        let r2 = distill_dialogue_patterns(&mut store, &mut log, &mut bank, &config);
        let r3 = distill_dialogue_patterns(&mut store, &mut log, &mut bank, &config);

        assert!(!r1.prototypes_created.is_empty());
        assert!(r2.prototypes_created.is_empty());
        assert!(r3.prototypes_created.is_empty());
    }
}
