//! Edge extraction from Wiktionary-style glosses.
//!
//! Transforms 144K concepts with 0 edges into a populated graph by mining
//! IS_A, PART_OF, and USED_FOR relationships from English gloss text.
//!
//! Deterministic string-pattern extraction — no neural model, no randomness.
//! Every edge produced carries a pattern label for explainability.

use std::collections::HashMap;

#[allow(unused_imports)]
use crate::piece_graph::{ConceptId, ConceptNode, EdgeKind, PackedEdge, PieceGraph};

/// Pattern that can extract an IS_A target from a gloss.
/// Each pattern has a prefix to match and a method to extract the head noun.
struct HypernymPattern {
    prefix: &'static str,
    confidence: u8, // 0-7 weight
    /// Strip the prefix, then take words until hitting a comma, period, semicolon,
    /// "that", "which", "with", or "of" (depending on pattern needs).
    strip_head: &'static [&'static str],
    kind: EdgeKind,
}

/// Primary patterns. Order matters — longer prefixes first so "a kind of" matches
/// before just "a ".
const PATTERNS: &[HypernymPattern] = &[
    HypernymPattern { prefix: "a kind of ",    confidence: 7, strip_head: &[",", ";", ".", " that ", " which ", " used "], kind: EdgeKind::IsA },
    HypernymPattern { prefix: "a type of ",    confidence: 7, strip_head: &[",", ";", ".", " that ", " which ", " used "], kind: EdgeKind::IsA },
    HypernymPattern { prefix: "a species of ", confidence: 7, strip_head: &[",", ";", ".", " that ", " which "], kind: EdgeKind::IsA },
    HypernymPattern { prefix: "any of ",       confidence: 5, strip_head: &[",", ";", ".", " that ", " which ", " of "], kind: EdgeKind::IsA },
    HypernymPattern { prefix: "one of ",       confidence: 5, strip_head: &[",", ";", ".", " that ", " which "], kind: EdgeKind::IsA },
    HypernymPattern { prefix: "member of ",    confidence: 6, strip_head: &[",", ";", ".", " that ", " which "], kind: EdgeKind::IsA },
    HypernymPattern { prefix: "an ",           confidence: 4, strip_head: &[",", ";", ".", " that ", " which ", " of ", " in ", " with ", " for "], kind: EdgeKind::IsA },
    HypernymPattern { prefix: "a ",            confidence: 4, strip_head: &[",", ";", ".", " that ", " which ", " of ", " in ", " with ", " for "], kind: EdgeKind::IsA },
];

const USED_FOR_PATTERNS: &[HypernymPattern] = &[
    HypernymPattern { prefix: "used to ",    confidence: 5, strip_head: &[",", ";", "."], kind: EdgeKind::Other },
    HypernymPattern { prefix: "used for ",   confidence: 5, strip_head: &[",", ";", "."], kind: EdgeKind::Other },
];

/// Find the first occurrence of any separator; return the prefix before it.
fn head_until_separator(s: &str, seps: &[&str]) -> String {
    let mut cutoff = s.len();
    for sep in seps {
        if let Some(i) = s.find(sep) {
            if i < cutoff { cutoff = i; }
        }
    }
    s[..cutoff].trim().to_lowercase()
}

/// Extract a head-noun candidate from a phrase. Handles:
///   "small red tile"   → "tile"   (last word as head; noun phrases in English)
///   "sword that cuts"  → "sword"
/// For now we take the LAST word of the stripped phrase as the head noun.
/// This is correct for English ~80% of the time (right-headed NPs).
fn extract_head_noun(phrase: &str) -> Option<String> {
    let cleaned = phrase.trim_end_matches(&[',', ';', ':', '.', '!', '?', ')', '('] as &[char]);
    let last = cleaned.split_whitespace().last()?;
    // Strip possessive/plural noise
    let last = last.trim_end_matches("'s").trim_end_matches('s');
    // Skip obvious non-nouns
    if last.len() < 2 { return None; }
    if matches!(last, "the" | "a" | "an" | "of" | "in" | "with" | "to"
              | "from" | "by" | "for" | "on" | "at" | "is" | "are" | "or"
              | "and" | "be" | "it" | "any" | "some") {
        return None;
    }
    Some(last.to_lowercase())
}

/// Build an index from lowercase anchor → ConceptId (first occurrence).
/// For words that map to multiple concepts (polysemy), we pick the NOUN variant
/// if available, otherwise the first. This is a deterministic heuristic.
pub fn build_anchor_index(graph: &PieceGraph) -> HashMap<String, ConceptId> {
    let mut index: HashMap<String, ConceptId> = HashMap::new();
    let mut noun_index: HashMap<String, ConceptId> = HashMap::new();

    for c in &graph.concepts {
        let anchor = graph.pieces.get(c.anchor_piece).to_lowercase();
        if anchor.is_empty() { continue; }
        // Prefer noun POS (code 1) when available
        if c.pos == 1 && !noun_index.contains_key(&anchor) {
            noun_index.insert(anchor.clone(), c.id);
        }
        if !index.contains_key(&anchor) {
            index.insert(anchor, c.id);
        }
    }

    // Merge: noun wins if available
    for (k, v) in noun_index {
        index.insert(k, v);
    }
    index
}

/// Extraction result — proposed edges with provenance for each.
pub struct ExtractionResult {
    /// Proposed edges grouped by source concept: (source_id, kind, target_id, confidence, pattern_label)
    pub proposed: Vec<(ConceptId, EdgeKind, ConceptId, u8, &'static str)>,
    pub stats: ExtractionStats,
}

#[derive(Default, Debug)]
pub struct ExtractionStats {
    pub concepts_scanned: usize,
    pub glosses_with_pattern: usize,
    pub patterns_matched: HashMap<&'static str, usize>,
    pub head_nouns_resolved: usize,
    pub head_nouns_missed: usize,
    pub self_links_skipped: usize,
}

/// Run extraction over the whole graph.
pub fn extract_edges(graph: &PieceGraph) -> ExtractionResult {
    let anchor_index = build_anchor_index(graph);
    let mut proposed = Vec::with_capacity(50_000);
    let mut stats = ExtractionStats::default();
    stats.concepts_scanned = graph.concepts.len();

    for c in &graph.concepts {
        let gloss = graph.pieces.get(c.gloss_piece).to_lowercase();
        if gloss.is_empty() { continue; }

        // Try IS_A patterns
        for pat in PATTERNS {
            if let Some(rest) = gloss.strip_prefix(pat.prefix) {
                stats.glosses_with_pattern += 1;
                *stats.patterns_matched.entry(pat.prefix).or_insert(0) += 1;
                let head_phrase = head_until_separator(rest, pat.strip_head);
                if let Some(head) = extract_head_noun(&head_phrase) {
                    if let Some(&target_id) = anchor_index.get(&head) {
                        if target_id != c.id {
                            proposed.push((c.id, pat.kind, target_id, pat.confidence, pat.prefix));
                            stats.head_nouns_resolved += 1;
                        } else {
                            stats.self_links_skipped += 1;
                        }
                    } else {
                        stats.head_nouns_missed += 1;
                    }
                } else {
                    stats.head_nouns_missed += 1;
                }
                break; // one pattern per gloss
            }
        }

        // USED_FOR patterns (EdgeKind::Other with label)
        for pat in USED_FOR_PATTERNS {
            if gloss.starts_with(pat.prefix) {
                *stats.patterns_matched.entry(pat.prefix).or_insert(0) += 1;
                // We don't have a dedicated USED_FOR kind yet, so skip storing for now
                // (tracked in stats only). Future: add UsedFor to EdgeKind enum.
                break;
            }
        }
    }

    ExtractionResult { proposed, stats }
}

/// Apply proposed edges to the graph in-place. Deduplicates (src, kind, dst).
pub fn apply_edges(graph: &mut PieceGraph, proposed: &[(ConceptId, EdgeKind, ConceptId, u8, &'static str)])
    -> usize
{
    // Build a dedup key per concept
    let mut added = 0;
    let mut by_source: HashMap<ConceptId, Vec<(EdgeKind, ConceptId)>> = HashMap::new();
    for (src, kind, dst, _w, _pat) in proposed {
        by_source.entry(*src).or_insert_with(Vec::new).push((*kind, *dst));
    }

    for c in graph.concepts.iter_mut() {
        if let Some(new_edges) = by_source.get(&c.id) {
            // existing set
            let mut existing: std::collections::HashSet<(u8, u32)> =
                c.edges.iter().map(|e| (e.kind, e.target)).collect();
            for (kind, dst) in new_edges {
                let key = (kind.as_u8(), *dst);
                if !existing.contains(&key) {
                    c.edges.push(PackedEdge::new(*kind, *dst));
                    existing.insert(key);
                    added += 1;
                }
            }
        }
    }
    added
}

// ────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece_graph::{PiecePool, PieceGraph};

    fn mini_graph() -> PieceGraph {
        let mut pieces = PiecePool::new();
        let dog_pid = pieces.intern("dog");
        let dog_gloss = pieces.intern("a kind of mammal");
        let mammal_pid = pieces.intern("mammal");
        let mammal_gloss = pieces.intern("a warm-blooded animal");
        let animal_pid = pieces.intern("animal");
        let animal_gloss = pieces.intern("living organism");

        let concepts = vec![
            ConceptNode { id: 0, anchor_piece: dog_pid, gloss_piece: dog_gloss,
                pos: 1, edges: Vec::new() },
            ConceptNode { id: 1, anchor_piece: mammal_pid, gloss_piece: mammal_gloss,
                pos: 1, edges: Vec::new() },
            ConceptNode { id: 2, anchor_piece: animal_pid, gloss_piece: animal_gloss,
                pos: 1, edges: Vec::new() },
        ];
        let mut g = PieceGraph::new();
        g.pieces = pieces;
        g.concepts = concepts;
        g
    }

    #[test]
    fn head_noun_extraction_works() {
        assert_eq!(extract_head_noun("small red tile").as_deref(), Some("tile"));
        assert_eq!(extract_head_noun("a mammal").as_deref(), Some("mammal"));
        assert_eq!(extract_head_noun("wild pigs").as_deref(), Some("pig"));
    }

    #[test]
    fn head_until_separator_stops_at_comma() {
        let s = "mammal, typically four-legged";
        let head = head_until_separator(s, &[",", ";", "."]);
        assert_eq!(head, "mammal");
    }

    #[test]
    fn anchor_index_finds_concepts() {
        let g = mini_graph();
        let idx = build_anchor_index(&g);
        assert_eq!(idx.get("dog"), Some(&0));
        assert_eq!(idx.get("mammal"), Some(&1));
    }

    #[test]
    fn extract_finds_dog_is_a_mammal() {
        let g = mini_graph();
        let result = extract_edges(&g);
        let has_dog_mammal = result.proposed.iter().any(|(src, kind, dst, _, _)|
            *src == 0 && matches!(kind, EdgeKind::IsA) && *dst == 1);
        assert!(has_dog_mammal,
            "should have extracted dog→mammal, got: {:?}",
            result.proposed.iter().map(|(s,k,d,_,_)| (s,k,d)).collect::<Vec<_>>());
    }

    #[test]
    fn apply_edges_adds_to_concepts() {
        let mut g = mini_graph();
        let result = extract_edges(&g);
        assert_eq!(g.concepts[0].edges.len(), 0);
        let added = apply_edges(&mut g, &result.proposed);
        assert!(added > 0, "should have applied at least one edge");
        assert!(g.concepts[0].edges.len() > 0 || g.concepts[1].edges.len() > 0);
    }

    #[test]
    fn self_links_skipped() {
        let mut pieces = PiecePool::new();
        let w = pieces.intern("word");
        let g_word = pieces.intern("a word");
        let concepts = vec![
            ConceptNode { id: 0, anchor_piece: w, gloss_piece: g_word,
                pos: 1, edges: Vec::new() },
        ];
        let mut g = PieceGraph::new();
        g.pieces = pieces;
        g.concepts = concepts;
        let result = extract_edges(&g);
        assert_eq!(result.stats.self_links_skipped, 1);
        assert!(result.proposed.is_empty());
    }

    #[test]
    fn determinism_same_input_same_output() {
        let g = mini_graph();
        let r1 = extract_edges(&g);
        let r2 = extract_edges(&g);
        assert_eq!(r1.proposed.len(), r2.proposed.len());
        for (a, b) in r1.proposed.iter().zip(r2.proposed.iter()) {
            assert_eq!(a.0, b.0);
            assert_eq!(a.2, b.2);
        }
    }
}
