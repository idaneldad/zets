//! Explain — provenance-aware trace of HOW an atom or edge got into the graph.
//!
//! The killer enterprise feature Idan identified: every answer ZETS gives
//! must be explainable. LLMs can't do this — attention weights are opaque.
//! ZETS can, because every edge has a source + provenance.
//!
//! Given an atom or edge, this module returns:
//!   - Its direct provenance (Asserted/Observed/Learned/Hypothesis)
//!   - The edges that SUPPORT it (what links into it and why)
//!   - For Learned edges: the exemplar observations that contributed
//!   - A human-readable explanation string
//!
//! This is the GDPR Article 22 / HIPAA / SOX compliance layer:
//! "why did the system conclude X about this person?"
//!
//! Example:
//!   explain_atom(store, prov_log, bank, proto_atom_id)
//!     -> "Pattern 'inform/sadness' (Learned, confidence=150)
//!         derived from 2 observations: 'I lost my job', 'My dog passed away'.
//!         Category: category:pattern (Asserted)."

use crate::atoms::{AtomEdge, AtomId, AtomStore};
use crate::learning_layer::{
    EdgeKey, Provenance, ProvenanceLog, ProvenanceRecord, ProtoBank,
};

/// An explanation of one edge: where it came from and what supports it.
#[derive(Debug, Clone)]
pub struct EdgeExplanation {
    pub from: AtomId,
    pub to: AtomId,
    pub relation: u8,
    pub weight: u8,
    pub provenance: Provenance,
    pub confidence: u8,
    /// Human-readable label for the source atom, if recoverable
    pub from_label: Option<String>,
    /// Human-readable label for the target atom, if recoverable
    pub to_label: Option<String>,
    /// Relation name (e.g., "is_a", "has_attribute")
    pub relation_name: Option<String>,
}

impl EdgeExplanation {
    /// Render as a one-line human-readable string.
    pub fn one_liner(&self) -> String {
        let from = self.from_label.as_deref().unwrap_or("?");
        let to = self.to_label.as_deref().unwrap_or("?");
        let rel = self.relation_name.as_deref().unwrap_or("rel");
        format!(
            "{} --{}--> {}  [{}, conf={}, weight={}]",
            from, rel, to, self.provenance.label(), self.confidence, self.weight
        )
    }
}

/// Full explanation of an atom: its direct edges + provenance summary.
#[derive(Debug, Clone)]
pub struct AtomExplanation {
    pub atom_id: AtomId,
    pub label: Option<String>,
    pub outgoing_edges: Vec<EdgeExplanation>,
    pub incoming_edges_sampled: Vec<EdgeExplanation>,  // up to 20
    pub provenance_summary: ProvenanceSummary,
    /// If this atom is a Prototype (registered in ProtoBank), its details
    pub prototype_info: Option<PrototypeInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct ProvenanceSummary {
    pub asserted: usize,
    pub observed: usize,
    pub learned: usize,
    pub hypothesis: usize,
}

#[derive(Debug, Clone)]
pub struct PrototypeInfo {
    pub name: String,
    pub domain: String,
    pub observation_count: u32,
    pub exemplar_texts: Vec<String>,
}

// ────────────────────────────────────────────────────────────────
// Main API
// ────────────────────────────────────────────────────────────────

/// Explain an atom — why it's in the graph and what's connected to it.
pub fn explain_atom(
    store: &AtomStore,
    prov_log: &ProvenanceLog,
    proto_bank: &ProtoBank,
    atom_id: AtomId,
) -> AtomExplanation {
    let label = atom_label(store, atom_id);

    let outgoing: Vec<EdgeExplanation> = store.outgoing(atom_id).iter()
        .map(|edge| explain_edge(store, prov_log, atom_id, edge))
        .collect();

    // Incoming: scan all atoms for edges pointing HERE.
    // O(N*E) — fine for explanation (human latency is ms-scale).
    // Cap at 20 to avoid huge outputs.
    let mut incoming_sampled = Vec::new();
    let (atoms, _) = store.snapshot();
    'outer: for (idx, _) in atoms.iter().enumerate() {
        let from_id = idx as AtomId;
        if from_id == atom_id { continue; }
        for edge in store.outgoing(from_id) {
            if edge.to == atom_id {
                incoming_sampled.push(explain_edge(store, prov_log, from_id, &edge));
                if incoming_sampled.len() >= 20 { break 'outer; }
            }
        }
    }

    let mut summary = ProvenanceSummary::default();
    for e in outgoing.iter().chain(incoming_sampled.iter()) {
        match e.provenance {
            Provenance::Asserted   => summary.asserted   += 1,
            Provenance::Observed   => summary.observed   += 1,
            Provenance::Learned    => summary.learned    += 1,
            Provenance::Hypothesis => summary.hypothesis += 1,
        }
    }

    // Prototype info if this atom is registered
    let prototype_info = (0..proto_bank.len())
        .filter_map(|i| proto_bank.get(i))
        .find(|p| p.atom_id == atom_id)
        .map(|p| PrototypeInfo {
            name: p.name.clone(),
            domain: p.domain.clone(),
            observation_count: p.observation_count,
            exemplar_texts: p.exemplars.iter()
                .filter_map(|e_id| exemplar_content_text(store, *e_id))
                .collect(),
        });

    AtomExplanation {
        atom_id,
        label,
        outgoing_edges: outgoing,
        incoming_edges_sampled: incoming_sampled,
        provenance_summary: summary,
        prototype_info,
    }
}

/// Explain a specific edge.
pub fn explain_edge(
    store: &AtomStore,
    prov_log: &ProvenanceLog,
    from: AtomId,
    edge: &AtomEdge,
) -> EdgeExplanation {
    let key = EdgeKey::new(from, edge.to, edge.relation);
    let record = prov_log.get(&key).copied()
        .unwrap_or_else(ProvenanceRecord::asserted);

    EdgeExplanation {
        from,
        to: edge.to,
        relation: edge.relation,
        weight: edge.weight,
        provenance: record.provenance,
        confidence: record.confidence,
        from_label: atom_label(store, from),
        to_label: atom_label(store, edge.to),
        relation_name: crate::relations::get(edge.relation).map(|r| r.name.to_string()),
    }
}

/// Render a full multi-line explanation suitable for console / audit log.
pub fn render_atom_explanation(exp: &AtomExplanation) -> String {
    let mut s = String::new();
    s.push_str(&format!("╔ ATOM #{} ", exp.atom_id));
    if let Some(ref label) = exp.label {
        s.push_str(&format!("'{}'", label));
    }
    s.push_str("\n");

    s.push_str(&format!("║ Provenance summary:\n"));
    s.push_str(&format!("║   asserted:   {}\n", exp.provenance_summary.asserted));
    s.push_str(&format!("║   observed:   {}\n", exp.provenance_summary.observed));
    s.push_str(&format!("║   learned:    {}\n", exp.provenance_summary.learned));
    s.push_str(&format!("║   hypothesis: {}\n", exp.provenance_summary.hypothesis));

    if let Some(ref info) = exp.prototype_info {
        s.push_str(&format!("║\n"));
        s.push_str(&format!("║ Prototype: {}\n", info.name));
        s.push_str(&format!("║   domain:       {}\n", info.domain));
        s.push_str(&format!("║   observations: {}\n", info.observation_count));
        if !info.exemplar_texts.is_empty() {
            s.push_str(&format!("║   exemplars:\n"));
            for text in &info.exemplar_texts {
                let truncated: String = text.chars().take(60).collect();
                s.push_str(&format!("║     — \"{}\"\n", truncated));
            }
        }
    }

    if !exp.outgoing_edges.is_empty() {
        s.push_str(&format!("║\n║ Outgoing ({} edges):\n", exp.outgoing_edges.len()));
        for edge in exp.outgoing_edges.iter().take(10) {
            s.push_str(&format!("║   → {}\n", edge.one_liner()));
        }
        if exp.outgoing_edges.len() > 10 {
            s.push_str(&format!("║   ... and {} more\n", exp.outgoing_edges.len() - 10));
        }
    }

    if !exp.incoming_edges_sampled.is_empty() {
        s.push_str(&format!("║\n║ Incoming (up to 20):\n"));
        for edge in exp.incoming_edges_sampled.iter().take(10) {
            s.push_str(&format!("║   ← {}\n", edge.one_liner()));
        }
    }

    s.push_str("╚\n");
    s
}

/// Why does the graph believe (from --rel--> to)? Returns a one-paragraph
/// audit-grade explanation.
pub fn explain_claim(
    store: &AtomStore,
    prov_log: &ProvenanceLog,
    from: AtomId,
    to: AtomId,
    relation: u8,
) -> String {
    let key = EdgeKey::new(from, to, relation);
    let record = prov_log.get(&key).copied()
        .unwrap_or_else(ProvenanceRecord::asserted);

    let from_label = atom_label(store, from).unwrap_or_else(|| format!("#{}", from));
    let to_label = atom_label(store, to).unwrap_or_else(|| format!("#{}", to));
    let rel_name = crate::relations::get(relation)
        .map(|r| r.name.to_string())
        .unwrap_or_else(|| format!("rel#{}", relation));

    let core = format!(
        "The graph holds that '{}' --{}--> '{}' [{}, confidence {}/255]",
        from_label, rel_name, to_label, record.provenance.label(), record.confidence
    );

    let explanation = match record.provenance {
        Provenance::Asserted =>
            " — this is a stated truth from a trusted source (textbook / user assertion / structural fact).",
        Provenance::Observed =>
            " — this is episodic evidence from a specific observation (dialogue turn, event). Not a claim about universal truth.",
        Provenance::Learned =>
            " — this is a generalization derived from multiple observations. It earned trust through distillation.",
        Provenance::Hypothesis =>
            " — this was proposed by dreaming/exploration and has NOT been verified. Use with caution.",
    };

    format!("{}{}", core, explanation)
}

// ────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────

fn atom_label(store: &AtomStore, atom_id: AtomId) -> Option<String> {
    store.get(atom_id)
        .and_then(|a| std::str::from_utf8(&a.data).ok().map(String::from))
}

/// For a prototype exemplar (which is a "utt:..." atom), walk to the content Text.
fn exemplar_content_text(store: &AtomStore, utt_atom: AtomId) -> Option<String> {
    let has_attr = crate::relations::by_name("has_attribute")?.code;
    for edge in store.outgoing(utt_atom) {
        if edge.relation == has_attr {
            if let Some(content) = store.get(edge.to) {
                if content.kind == crate::atoms::AtomKind::Text {
                    if let Ok(s) = std::str::from_utf8(&content.data) {
                        if !s.starts_with("utt:") {
                            return Some(s.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::AtomKind;
    use crate::relations;

    fn sample_graph() -> (AtomStore, ProvenanceLog, ProtoBank, AtomId, AtomId) {
        let mut store = AtomStore::new();
        let mut log = ProvenanceLog::new();
        let bank = ProtoBank::new();

        let dog = store.put(AtomKind::Concept, b"dog".to_vec());
        let animal = store.put(AtomKind::Concept, b"animal".to_vec());
        let is_a = relations::by_name("is_a").unwrap().code;
        store.link(dog, animal, is_a, 95, 0);
        log.tag(EdgeKey::new(dog, animal, is_a), ProvenanceRecord::asserted());

        (store, log, bank, dog, animal)
    }

    #[test]
    fn explain_atom_returns_label() {
        let (store, log, bank, dog, _) = sample_graph();
        let exp = explain_atom(&store, &log, &bank, dog);
        assert_eq!(exp.atom_id, dog);
        assert_eq!(exp.label.as_deref(), Some("dog"));
    }

    #[test]
    fn explain_atom_finds_outgoing_edges() {
        let (store, log, bank, dog, _) = sample_graph();
        let exp = explain_atom(&store, &log, &bank, dog);
        assert_eq!(exp.outgoing_edges.len(), 1);
        assert_eq!(exp.outgoing_edges[0].provenance, Provenance::Asserted);
        assert_eq!(exp.outgoing_edges[0].from_label.as_deref(), Some("dog"));
        assert_eq!(exp.outgoing_edges[0].to_label.as_deref(), Some("animal"));
    }

    #[test]
    fn explain_atom_finds_incoming_edges() {
        let (store, log, bank, _, animal) = sample_graph();
        let exp = explain_atom(&store, &log, &bank, animal);
        assert_eq!(exp.incoming_edges_sampled.len(), 1);
        assert_eq!(exp.incoming_edges_sampled[0].from_label.as_deref(), Some("dog"));
    }

    #[test]
    fn explain_atom_provenance_summary() {
        let (store, log, bank, dog, _) = sample_graph();
        let exp = explain_atom(&store, &log, &bank, dog);
        assert_eq!(exp.provenance_summary.asserted, 1);
        assert_eq!(exp.provenance_summary.observed, 0);
    }

    #[test]
    fn explain_edge_one_liner() {
        let (store, log, _, dog, animal) = sample_graph();
        let is_a = relations::by_name("is_a").unwrap().code;
        let edge = store.outgoing(dog).into_iter().find(|e| e.to == animal).unwrap();
        let exp = explain_edge(&store, &log, dog, &edge);
        let line = exp.one_liner();
        assert!(line.contains("dog"));
        assert!(line.contains("animal"));
        assert!(line.contains("is_a"));
        assert!(line.contains("asserted"));
        let _ = is_a;
    }

    #[test]
    fn explain_claim_describes_asserted() {
        let (store, log, _, dog, animal) = sample_graph();
        let is_a = relations::by_name("is_a").unwrap().code;
        let explanation = explain_claim(&store, &log, dog, animal, is_a);
        assert!(explanation.contains("dog"));
        assert!(explanation.contains("animal"));
        assert!(explanation.contains("asserted"));
        assert!(explanation.contains("trusted source"));
    }

    #[test]
    fn explain_claim_describes_hypothesis() {
        let mut store = AtomStore::new();
        let mut log = ProvenanceLog::new();
        let comfort = store.put(AtomKind::Concept, b"comfort".to_vec());
        let sadness = store.put(AtomKind::Concept, b"sadness".to_vec());
        let near = relations::by_name("near").unwrap().code;
        store.link(comfort, sadness, near, 60, 0);
        log.tag(EdgeKey::new(comfort, sadness, near), ProvenanceRecord::hypothesis());

        let text = explain_claim(&store, &log, comfort, sadness, near);
        assert!(text.contains("hypothesis"));
        assert!(text.contains("NOT been verified"));
    }

    #[test]
    fn explain_claim_untagged_treated_as_asserted() {
        let (store, log, _, dog, animal) = sample_graph();
        let near = relations::by_name("near").unwrap().code;
        // near edge doesn't exist but we ask about it — the log has no entry
        let text = explain_claim(&store, &log, dog, animal, near);
        assert!(text.contains("asserted"));  // default provenance
    }

    #[test]
    fn render_atom_explanation_contains_label() {
        let (store, log, bank, dog, _) = sample_graph();
        let exp = explain_atom(&store, &log, &bank, dog);
        let rendered = render_atom_explanation(&exp);
        assert!(rendered.contains("dog"));
        assert!(rendered.contains("asserted"));
        assert!(rendered.contains("Provenance summary"));
    }

    #[test]
    fn explain_atom_with_prototype_includes_info() {
        let mut store = AtomStore::new();
        let log = ProvenanceLog::new();
        let mut bank = ProtoBank::new();

        let proto_atom = store.put(AtomKind::Concept, b"proto:test".to_vec());
        bank.register(crate::learning_layer::Prototype {
            atom_id: proto_atom,
            name: "pattern:test".to_string(),
            domain: "test_domain".to_string(),
            observation_count: 42,
            drift: 0.05,
            exemplars: vec![],
            last_distilled: "2026-04-22".to_string(),
        });

        let exp = explain_atom(&store, &log, &bank, proto_atom);
        assert!(exp.prototype_info.is_some());
        let info = exp.prototype_info.unwrap();
        assert_eq!(info.name, "pattern:test");
        assert_eq!(info.observation_count, 42);
    }

    #[test]
    fn explain_atom_without_prototype_has_none() {
        let (store, log, bank, dog, _) = sample_graph();
        let exp = explain_atom(&store, &log, &bank, dog);
        assert!(exp.prototype_info.is_none());
    }

    #[test]
    fn explain_edge_learned_provenance() {
        let mut store = AtomStore::new();
        let mut log = ProvenanceLog::new();
        let from = store.put(AtomKind::Concept, b"A".to_vec());
        let to = store.put(AtomKind::Concept, b"B".to_vec());
        let is_a = relations::by_name("is_a").unwrap().code;
        store.link(from, to, is_a, 90, 0);
        log.tag(EdgeKey::new(from, to, is_a), ProvenanceRecord::learned(215));

        let edge = store.outgoing(from).into_iter().find(|e| e.to == to).unwrap();
        let exp = explain_edge(&store, &log, from, &edge);
        assert_eq!(exp.provenance, Provenance::Learned);
        assert_eq!(exp.confidence, 215);
    }
}
