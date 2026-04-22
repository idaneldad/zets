//! Dialogue — typed ingestion of multi-turn conversations.
//!
//! This is the cognitive training path Idan identified: we want ZETS to
//! LEARN FROM dialogues (how people express frustration, how empathetic
//! responses reduce escalation) WITHOUT treating each specific utterance
//! as a fact.
//!
//! The key move: every edge created during dialogue ingestion gets tagged
//! `Provenance::Observed` in the ProvenanceLog. This means:
//!   - Precision mode (strict reasoning) IGNORES these edges
//!   - Narrative/Gestalt modes USE them for context
//!   - Divergent mode freely traverses them
//!   - Distillation can later cluster them into Learned patterns
//!
//! Structure:
//!   Conversation {
//!     participants: [user_atom, assistant_atom, ...]
//!     turns: [DialogTurn, ...]
//!     outcome: resolved | escalated | abandoned | converted
//!   }
//!   DialogTurn { speaker, utterance_text, intent, emotion, replies_to }
//!
//! When ingested into AtomStore + ProvenanceLog:
//!   - Each utterance becomes a Text atom
//!   - said_by edge (Observed) connects speaker -> utterance
//!   - expresses_emotion edge (Observed) connects utterance -> emotion
//!   - has_intent edge (Observed) connects utterance -> intent
//!   - replies_to edge (Observed) connects utterance -> previous utterance
//!   - part_of edge connects all turns to a conversation atom

use crate::atoms::{AtomId, AtomKind, AtomStore};
use crate::learning_layer::{EdgeKey, ProvenanceLog, ProvenanceRecord};
use crate::relations;

/// What was the speaker trying to do with this utterance?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Intent {
    Request,       // "can you help me with X"
    Complain,      // "this is broken"
    Agree,         // "that makes sense"
    Decline,       // "no, I don't want that"
    Clarify,       // "what do you mean by X?"
    Empathize,     // "that sounds rough"
    Inform,        // "the sky is blue"
    Question,      // "why is X?"
    Greet,         // "hello"
    Farewell,      // "bye"
    Other,
}

impl Intent {
    pub fn label(self) -> &'static str {
        match self {
            Self::Request   => "intent:request",
            Self::Complain  => "intent:complain",
            Self::Agree     => "intent:agree",
            Self::Decline   => "intent:decline",
            Self::Clarify   => "intent:clarify",
            Self::Empathize => "intent:empathize",
            Self::Inform    => "intent:inform",
            Self::Question  => "intent:question",
            Self::Greet     => "intent:greet",
            Self::Farewell  => "intent:farewell",
            Self::Other     => "intent:other",
        }
    }
}

/// Emotional coloring of an utterance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Emotion {
    Joy,
    Sadness,
    Anger,
    Fear,
    Surprise,
    Disgust,
    Trust,
    Anticipation,
    Neutral,
}

impl Emotion {
    pub fn label(self) -> &'static str {
        match self {
            Self::Joy          => "emotion:joy",
            Self::Sadness      => "emotion:sadness",
            Self::Anger        => "emotion:anger",
            Self::Fear         => "emotion:fear",
            Self::Surprise     => "emotion:surprise",
            Self::Disgust      => "emotion:disgust",
            Self::Trust        => "emotion:trust",
            Self::Anticipation => "emotion:anticipation",
            Self::Neutral      => "emotion:neutral",
        }
    }
}

/// One turn in a conversation.
#[derive(Debug, Clone)]
pub struct DialogTurn {
    /// Who spoke — an atom (usually Concept: "user" or "assistant", or specific user)
    pub speaker: AtomId,
    /// The actual utterance text
    pub text: String,
    /// Inferred speaker intent
    pub intent: Intent,
    /// Inferred emotional state
    pub emotion: Emotion,
    /// 0-indexed position within the conversation
    pub turn_index: u32,
}

/// How did a conversation end?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvOutcome {
    Resolved,   // user got what they needed
    Escalated,  // frustration increased, possibly handed off
    Abandoned,  // user dropped without resolution
    Converted,  // sale / signup / goal completion
    Ongoing,    // still in progress (or we don't know)
}

impl ConvOutcome {
    pub fn label(self) -> &'static str {
        match self {
            Self::Resolved  => "outcome:resolved",
            Self::Escalated => "outcome:escalated",
            Self::Abandoned => "outcome:abandoned",
            Self::Converted => "outcome:converted",
            Self::Ongoing   => "outcome:ongoing",
        }
    }
}

/// A full conversation — ingested as an Observed subgraph.
#[derive(Debug, Clone)]
pub struct Conversation {
    pub id: String,
    pub turns: Vec<DialogTurn>,
    pub outcome: ConvOutcome,
    /// Optional metadata: source dataset, timestamp, etc.
    pub source: String,
}

/// Stats returned by `ingest_dialogue`.
#[derive(Debug, Clone, Default)]
pub struct DialogueIngestStats {
    pub conversation_atom: AtomId,
    pub turn_atoms: Vec<AtomId>,
    pub new_atoms_total: usize,
    pub new_edges_total: usize,
    pub observed_edges_tagged: usize,
}

// ────────────────────────────────────────────────────────────────
// Ingestion
// ────────────────────────────────────────────────────────────────

/// Ingest a conversation into the graph.
///
/// CRITICAL: every edge created here is tagged `Provenance::Observed`
/// in the ProvenanceLog. The utterances are NOT asserted as truth.
/// Precision mode will ignore them. Only distillation (later) will
/// convert patterns to Learned edges.
pub fn ingest_dialogue(
    store: &mut AtomStore,
    prov_log: &mut ProvenanceLog,
    conv: &Conversation,
) -> DialogueIngestStats {
    let atoms_before = store.atom_count();
    let edges_before = store.edge_count();
    let tagged_before = prov_log.len();

    // 1. Create conversation atom
    let conv_data = format!("conversation:{}:{}", conv.source, conv.id).into_bytes();
    let conv_atom = store.put(AtomKind::Concept, conv_data);

    // 2. Tag outcome
    let outcome_atom = ensure_concept(store, conv.outcome.label());
    let is_a = relations::by_name("is_a").unwrap().code;
    let has_attribute = relations::by_name("has_attribute").unwrap().code;
    let part_of = relations::by_name("part_of").unwrap().code;
    let expresses = relations::by_name("expresses_emotion")
        .map(|r| r.code).unwrap_or(has_attribute);
    let replies_to_rel = relations::by_name("replies_to")
        .map(|r| r.code).unwrap_or(part_of);
    let said_by_rel = relations::by_name("said_by")
        .map(|r| r.code).unwrap_or(has_attribute);

    store.link(conv_atom, outcome_atom, has_attribute, 80, 0);
    prov_log.tag(EdgeKey::new(conv_atom, outcome_atom, has_attribute),
                 ProvenanceRecord::observed());

    // 3. For each turn
    let mut turn_atoms = Vec::with_capacity(conv.turns.len());
    let mut prev_turn_atom: Option<AtomId> = None;

    for (i, turn) in conv.turns.iter().enumerate() {
        // 3a. Utterance atom
        let utt_data = format!("utt:{}:{}:{}", conv.source, conv.id, i).into_bytes();
        let utt_atom = store.put(AtomKind::Text, utt_data);

        // Also create a content atom holding the text itself (could be large;
        // in Phase B we'd hash or embed it)
        let content_data = turn.text.as_bytes().to_vec();
        let content_atom = store.put(AtomKind::Text, content_data);
        store.link(utt_atom, content_atom, has_attribute, 90, 0);
        prov_log.tag(EdgeKey::new(utt_atom, content_atom, has_attribute),
                     ProvenanceRecord::observed());

        // 3b. part_of conversation
        store.link(utt_atom, conv_atom, part_of, 90, 0);
        prov_log.tag(EdgeKey::new(utt_atom, conv_atom, part_of),
                     ProvenanceRecord::observed());

        // 3c. said_by speaker
        store.link(utt_atom, turn.speaker, said_by_rel, 90, 0);
        prov_log.tag(EdgeKey::new(utt_atom, turn.speaker, said_by_rel),
                     ProvenanceRecord::observed());

        // 3d. intent
        let intent_atom = ensure_concept(store, turn.intent.label());
        store.link(utt_atom, intent_atom, has_attribute, 75, 0);
        prov_log.tag(EdgeKey::new(utt_atom, intent_atom, has_attribute),
                     ProvenanceRecord::observed());
        // Also: intent is_a category:intent (Asserted — this is structure)
        let intent_cat = ensure_concept(store, "category:intent");
        store.link(intent_atom, intent_cat, is_a, 95, 0);
        prov_log.tag(EdgeKey::new(intent_atom, intent_cat, is_a),
                     ProvenanceRecord::asserted());

        // 3e. emotion (skip Neutral — no edge adds noise)
        if turn.emotion != Emotion::Neutral {
            let emo_atom = ensure_concept(store, turn.emotion.label());
            store.link(utt_atom, emo_atom, expresses, 70, 0);
            prov_log.tag(EdgeKey::new(utt_atom, emo_atom, expresses),
                         ProvenanceRecord::observed());
            let emo_cat = ensure_concept(store, "category:emotion");
            store.link(emo_atom, emo_cat, is_a, 95, 0);
            prov_log.tag(EdgeKey::new(emo_atom, emo_cat, is_a),
                         ProvenanceRecord::asserted());
        }

        // 3f. replies_to previous turn
        if let Some(prev) = prev_turn_atom {
            store.link(utt_atom, prev, replies_to_rel, 85, 0);
            prov_log.tag(EdgeKey::new(utt_atom, prev, replies_to_rel),
                         ProvenanceRecord::observed());
        }

        turn_atoms.push(utt_atom);
        prev_turn_atom = Some(utt_atom);
    }

    DialogueIngestStats {
        conversation_atom: conv_atom,
        turn_atoms,
        new_atoms_total: store.atom_count() - atoms_before,
        new_edges_total: store.edge_count() - edges_before,
        observed_edges_tagged: prov_log.len() - tagged_before,
    }
}

/// Idempotent: look up or create a concept atom by label.
fn ensure_concept(store: &mut AtomStore, label: &str) -> AtomId {
    // AtomStore::put is already content-hash-dedup'd, so this is naturally idempotent.
    store.put(AtomKind::Concept, label.as_bytes().to_vec())
}

// ────────────────────────────────────────────────────────────────
// Pattern search — find conversations matching a shape
// ────────────────────────────────────────────────────────────────

/// Find all conversations whose outcome matches the given outcome.
pub fn conversations_with_outcome(
    store: &AtomStore,
    outcome: ConvOutcome,
) -> Vec<AtomId> {
    let outcome_hash = crate::atoms::content_hash(outcome.label().as_bytes());
    let (atoms, _) = store.snapshot();
    let outcome_atom = match atoms.iter().position(|a| a.content_hash == outcome_hash) {
        Some(i) => i as AtomId,
        None => return Vec::new(),
    };

    // Find all atoms that link to outcome_atom via has_attribute
    let has_attr = relations::by_name("has_attribute").unwrap().code;
    let mut results = Vec::new();
    for (idx, _) in atoms.iter().enumerate() {
        let aid = idx as AtomId;
        for edge in store.outgoing(aid) {
            if edge.to == outcome_atom && edge.relation == has_attr {
                // Check this is a conversation atom (content starts with "conversation:")
                if let Some(atom) = store.get(aid) {
                    if let Ok(s) = std::str::from_utf8(&atom.data) {
                        if s.starts_with("conversation:") {
                            results.push(aid);
                        }
                    }
                }
                break;
            }
        }
    }
    results
}

/// Count turns in a conversation by walking part_of edges.
pub fn conversation_turn_count(store: &AtomStore, conv_atom: AtomId) -> usize {
    // Find all atoms that have a part_of edge pointing to conv_atom
    let part_of = relations::by_name("part_of").unwrap().code;
    let (atoms, _) = store.snapshot();
    let mut count = 0;
    for (idx, _) in atoms.iter().enumerate() {
        let aid = idx as AtomId;
        if aid == conv_atom { continue; }
        for edge in store.outgoing(aid) {
            if edge.to == conv_atom && edge.relation == part_of {
                count += 1;
                break;
            }
        }
    }
    count
}

// ────────────────────────────────────────────────────────────────
// JSONL loader — hand-written to avoid serde dependency
// ────────────────────────────────────────────────────────────────

/// Parse a single JSONL line into a Conversation.
/// Speaker atoms are resolved via the provided resolver function.
pub fn parse_jsonl_line<F>(
    line: &str,
    mut speaker_resolver: F,
) -> Result<Conversation, String>
where F: FnMut(&str) -> AtomId,
{
    let line = line.trim();
    if line.is_empty() { return Err("empty line".into()); }

    // Very small JSON parser — scans the expected structure directly.
    // Expected keys: id, source, outcome, turns[]
    let id = extract_string_field(line, "id")?;
    let source = extract_string_field(line, "source")?;
    let outcome_str = extract_string_field(line, "outcome")?;
    let outcome = match outcome_str.as_str() {
        "Resolved"  => ConvOutcome::Resolved,
        "Escalated" => ConvOutcome::Escalated,
        "Abandoned" => ConvOutcome::Abandoned,
        "Converted" => ConvOutcome::Converted,
        "Ongoing"   => ConvOutcome::Ongoing,
        other => return Err(format!("unknown outcome: {}", other)),
    };

    // Extract turns array: "turns":[{...},{...}]
    let turns_start = line.find("\"turns\"")
        .ok_or_else(|| "missing turns field".to_string())?;
    let after_key = &line[turns_start..];
    let bracket_start = after_key.find('[')
        .ok_or_else(|| "turns array not found".to_string())?;
    let mut depth = 0;
    let mut end_idx = None;
    for (i, c) in after_key[bracket_start..].char_indices() {
        match c {
            '[' => depth += 1,
            ']' => { depth -= 1; if depth == 0 { end_idx = Some(bracket_start + i + 1); break; } }
            _ => {}
        }
    }
    let end_idx = end_idx.ok_or_else(|| "unterminated turns array".to_string())?;
    let turns_json = &after_key[bracket_start + 1..end_idx - 1];

    let mut turns = Vec::new();
    let mut turn_index: u32 = 0;
    let mut depth = 0;
    let mut obj_start = None;
    for (i, c) in turns_json.char_indices() {
        match c {
            '{' => { if depth == 0 { obj_start = Some(i); } depth += 1; }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(start) = obj_start {
                        let obj = &turns_json[start..=i];
                        let turn = parse_turn_object(obj, &mut speaker_resolver, turn_index)?;
                        turns.push(turn);
                        turn_index += 1;
                        obj_start = None;
                    }
                }
            }
            _ => {}
        }
    }

    Ok(Conversation { id, source, outcome, turns })
}

fn parse_turn_object<F>(
    obj: &str,
    resolver: &mut F,
    fallback_index: u32,
) -> Result<DialogTurn, String>
where F: FnMut(&str) -> AtomId,
{
    let speaker_name = extract_string_field(obj, "speaker")?;
    let text = extract_string_field(obj, "text")?;
    let intent_str = extract_string_field(obj, "intent")?;
    let emotion_str = extract_string_field(obj, "emotion")?;

    let intent = match intent_str.as_str() {
        "Request"   => Intent::Request,
        "Complain"  => Intent::Complain,
        "Agree"     => Intent::Agree,
        "Decline"   => Intent::Decline,
        "Clarify"   => Intent::Clarify,
        "Empathize" => Intent::Empathize,
        "Inform"    => Intent::Inform,
        "Question"  => Intent::Question,
        "Greet"     => Intent::Greet,
        "Farewell"  => Intent::Farewell,
        _           => Intent::Other,
    };
    let emotion = match emotion_str.as_str() {
        "Joy"          => Emotion::Joy,
        "Sadness"      => Emotion::Sadness,
        "Anger"        => Emotion::Anger,
        "Fear"         => Emotion::Fear,
        "Surprise"     => Emotion::Surprise,
        "Disgust"      => Emotion::Disgust,
        "Trust"        => Emotion::Trust,
        "Anticipation" => Emotion::Anticipation,
        _              => Emotion::Neutral,
    };

    // turn_index is optional — try to extract, default to fallback
    let turn_index = extract_u32_field(obj, "turn_index").unwrap_or(fallback_index);

    Ok(DialogTurn {
        speaker: resolver(&speaker_name),
        text,
        intent,
        emotion,
        turn_index,
    })
}

/// Extract a JSON string field value. Handles basic \" escapes.
fn extract_string_field(json: &str, field: &str) -> Result<String, String> {
    let pattern = format!("\"{}\"", field);
    let pos = json.find(&pattern)
        .ok_or_else(|| format!("field '{}' not found", field))?;
    let after = &json[pos + pattern.len()..];

    // Skip whitespace + colon
    let after = after.trim_start();
    let after = after.strip_prefix(':')
        .ok_or_else(|| format!("missing colon after field '{}'", field))?;
    let after = after.trim_start();
    let after = after.strip_prefix('"')
        .ok_or_else(|| format!("field '{}' is not a string", field))?;

    // Read until unescaped "
    let mut result = String::new();
    let mut chars = after.chars();
    while let Some(c) = chars.next() {
        match c {
            '"'  => return Ok(result),
            '\\' => {
                match chars.next() {
                    Some('"')  => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some('n')  => result.push('\n'),
                    Some('t')  => result.push('\t'),
                    Some('r')  => result.push('\r'),
                    Some(other) => { result.push('\\'); result.push(other); }
                    None => return Err(format!("unterminated escape in field '{}'", field)),
                }
            }
            _ => result.push(c),
        }
    }
    Err(format!("unterminated string in field '{}'", field))
}

fn extract_u32_field(json: &str, field: &str) -> Option<u32> {
    let pattern = format!("\"{}\"", field);
    let pos = json.find(&pattern)?;
    let after = &json[pos + pattern.len()..];
    let after = after.trim_start().strip_prefix(':')?.trim_start();
    let end = after.find(|c: char| !c.is_ascii_digit())?;
    if end == 0 { return None; }
    after[..end].parse::<u32>().ok()
}

/// Load a JSONL file of conversations + ingest them all.
/// Returns count of successfully ingested conversations + list of parse errors.
pub fn ingest_dialogues_from_jsonl<P: AsRef<std::path::Path>>(
    store: &mut AtomStore,
    prov_log: &mut ProvenanceLog,
    path: P,
) -> (usize, Vec<String>) {
    use std::collections::HashMap;
    let mut speaker_cache: HashMap<String, AtomId> = HashMap::new();

    let contents = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => return (0, vec![format!("read file: {}", e)]),
    };

    let mut success = 0;
    let mut errors = Vec::new();

    for (line_no, line) in contents.lines().enumerate() {
        if line.trim().is_empty() { continue; }
        match parse_jsonl_with_cache(line, &mut speaker_cache, store) {
            Ok(conv) => {
                ingest_dialogue(store, prov_log, &conv);
                success += 1;
            }
            Err(e) => errors.push(format!("line {}: {}", line_no + 1, e)),
        }
    }

    (success, errors)
}

/// Helper: parse a line using a speaker cache to avoid duplicate atoms.
fn parse_jsonl_with_cache(
    line: &str,
    cache: &mut std::collections::HashMap<String, AtomId>,
    store: &mut AtomStore,
) -> Result<Conversation, String> {
    // We can't use a closure that borrows both cache and store — so pre-scan
    // speakers first, then parse.
    let mut pre_speakers: Vec<String> = Vec::new();
    let mut rest = line;
    while let Some(pos) = rest.find("\"speaker\"") {
        let after = &rest[pos..];
        if let Ok(name) = extract_string_field(after, "speaker") {
            pre_speakers.push(name);
            rest = &after[9..];  // skip past "speaker"
        } else {
            break;
        }
    }
    for name in &pre_speakers {
        if !cache.contains_key(name) {
            let aid = store.put(AtomKind::Concept, format!("speaker:{}", name).into_bytes());
            cache.insert(name.clone(), aid);
        }
    }
    // Now parse with a simple resolver from the now-populated cache
    parse_jsonl_line(line, |name| {
        *cache.get(name).expect("speaker should be pre-cached")
    })
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning_layer::Provenance;

    fn sample_conversation() -> (AtomStore, AtomId, AtomId, Conversation) {
        let mut store = AtomStore::new();
        let user = store.put(AtomKind::Concept, b"speaker:user".to_vec());
        let assistant = store.put(AtomKind::Concept, b"speaker:assistant".to_vec());

        let conv = Conversation {
            id: "c001".to_string(),
            source: "test".to_string(),
            outcome: ConvOutcome::Resolved,
            turns: vec![
                DialogTurn {
                    speaker: user,
                    text: "I lost my job yesterday".to_string(),
                    intent: Intent::Inform,
                    emotion: Emotion::Sadness,
                    turn_index: 0,
                },
                DialogTurn {
                    speaker: assistant,
                    text: "That sounds devastating".to_string(),
                    intent: Intent::Empathize,
                    emotion: Emotion::Neutral,
                    turn_index: 1,
                },
                DialogTurn {
                    speaker: user,
                    text: "Thanks, it means a lot".to_string(),
                    intent: Intent::Agree,
                    emotion: Emotion::Trust,
                    turn_index: 2,
                },
            ],
        };
        (store, user, assistant, conv)
    }

    #[test]
    fn intent_labels() {
        assert_eq!(Intent::Request.label(), "intent:request");
        assert_eq!(Intent::Empathize.label(), "intent:empathize");
    }

    #[test]
    fn emotion_labels() {
        assert_eq!(Emotion::Sadness.label(), "emotion:sadness");
        assert_eq!(Emotion::Joy.label(), "emotion:joy");
    }

    #[test]
    fn outcome_labels() {
        assert_eq!(ConvOutcome::Resolved.label(), "outcome:resolved");
        assert_eq!(ConvOutcome::Escalated.label(), "outcome:escalated");
    }

    #[test]
    fn ingest_creates_conversation_atom() {
        let (mut store, _, _, conv) = sample_conversation();
        let mut log = ProvenanceLog::new();
        let stats = ingest_dialogue(&mut store, &mut log, &conv);

        let atom = store.get(stats.conversation_atom).unwrap();
        let label = std::str::from_utf8(&atom.data).unwrap();
        assert!(label.starts_with("conversation:test:c001"));
    }

    #[test]
    fn ingest_creates_turn_atoms() {
        let (mut store, _, _, conv) = sample_conversation();
        let mut log = ProvenanceLog::new();
        let stats = ingest_dialogue(&mut store, &mut log, &conv);
        assert_eq!(stats.turn_atoms.len(), 3);
    }

    #[test]
    fn ingest_tags_most_edges_as_observed() {
        let (mut store, _, _, conv) = sample_conversation();
        let mut log = ProvenanceLog::new();
        let _ = ingest_dialogue(&mut store, &mut log, &conv);

        let counts = log.counts();
        let observed = counts.get(&Provenance::Observed).copied().unwrap_or(0);
        let asserted = counts.get(&Provenance::Asserted).copied().unwrap_or(0);

        // Most should be Observed (dialogue content)
        assert!(observed > asserted,
            "expected Observed > Asserted, got observed={} asserted={}",
            observed, asserted);
    }

    #[test]
    fn ingest_stats_accurate() {
        let (mut store, _, _, conv) = sample_conversation();
        let mut log = ProvenanceLog::new();
        let stats = ingest_dialogue(&mut store, &mut log, &conv);

        assert!(stats.new_atoms_total > 0);
        assert!(stats.new_edges_total > 0);
        assert!(stats.observed_edges_tagged > 0);
        // Total tagged edges is what ended up in the log
        let total_tagged = log.counts().values().sum::<usize>();
        assert!(total_tagged > 0);
    }

    #[test]
    fn ingest_is_idempotent() {
        let (mut store, _, _, conv) = sample_conversation();
        let mut log1 = ProvenanceLog::new();
        let mut log2 = ProvenanceLog::new();

        let s1 = ingest_dialogue(&mut store, &mut log1, &conv);
        let atoms_after_first = store.atom_count();

        // Re-ingest same conversation — atoms should NOT duplicate
        let s2 = ingest_dialogue(&mut store, &mut log2, &conv);
        let atoms_after_second = store.atom_count();

        assert_eq!(atoms_after_first, atoms_after_second,
            "re-ingesting same conversation should be idempotent");
        assert_eq!(s1.conversation_atom, s2.conversation_atom);
    }

    #[test]
    fn conversations_with_outcome_query() {
        let (mut store, _, _, conv) = sample_conversation();
        let mut log = ProvenanceLog::new();
        let stats = ingest_dialogue(&mut store, &mut log, &conv);

        let resolved = conversations_with_outcome(&store, ConvOutcome::Resolved);
        assert!(resolved.contains(&stats.conversation_atom));

        let escalated = conversations_with_outcome(&store, ConvOutcome::Escalated);
        assert!(!escalated.contains(&stats.conversation_atom));
    }

    #[test]
    fn turn_count_query() {
        let (mut store, _, _, conv) = sample_conversation();
        let mut log = ProvenanceLog::new();
        let stats = ingest_dialogue(&mut store, &mut log, &conv);
        assert_eq!(conversation_turn_count(&store, stats.conversation_atom), 3);
    }

    #[test]
    fn multiple_conversations_accumulate() {
        let (mut store, user, assistant, conv1) = sample_conversation();
        let mut conv2 = conv1.clone();
        conv2.id = "c002".to_string();
        conv2.outcome = ConvOutcome::Escalated;
        conv2.turns[0].emotion = Emotion::Anger;
        // Change a turn text so atoms differ
        conv2.turns[0].text = "I'm really frustrated with this service".to_string();

        let mut log = ProvenanceLog::new();
        let s1 = ingest_dialogue(&mut store, &mut log, &conv1);
        let s2 = ingest_dialogue(&mut store, &mut log, &conv2);

        assert_ne!(s1.conversation_atom, s2.conversation_atom);
        assert_eq!(conversations_with_outcome(&store, ConvOutcome::Resolved).len(), 1);
        assert_eq!(conversations_with_outcome(&store, ConvOutcome::Escalated).len(), 1);

        let _ = (user, assistant);
    }

    #[test]
    fn empty_conversation_safe() {
        let mut store = AtomStore::new();
        let mut log = ProvenanceLog::new();
        let conv = Conversation {
            id: "empty".to_string(),
            source: "test".to_string(),
            outcome: ConvOutcome::Ongoing,
            turns: vec![],
        };
        let stats = ingest_dialogue(&mut store, &mut log, &conv);
        assert!(stats.turn_atoms.is_empty());
        // Should still have the conversation atom + outcome atom
        assert!(stats.new_atoms_total >= 2);
    }

    #[test]
    fn parse_jsonl_line_basic() {
        let line = r#"{"id":"t1","source":"test","outcome":"Resolved","turns":[{"speaker":"user","text":"hello","intent":"Greet","emotion":"Neutral","turn_index":0}]}"#;
        let mut count = 0u32;
        let conv = parse_jsonl_line(line, |_| { count += 1; count as AtomId }).unwrap();
        assert_eq!(conv.id, "t1");
        assert_eq!(conv.outcome, ConvOutcome::Resolved);
        assert_eq!(conv.turns.len(), 1);
        assert_eq!(conv.turns[0].text, "hello");
        assert_eq!(conv.turns[0].intent, Intent::Greet);
    }

    #[test]
    fn parse_jsonl_line_multi_turn() {
        let line = r#"{"id":"t2","source":"test","outcome":"Escalated","turns":[{"speaker":"u","text":"a","intent":"Complain","emotion":"Anger","turn_index":0},{"speaker":"a","text":"b","intent":"Empathize","emotion":"Neutral","turn_index":1}]}"#;
        let conv = parse_jsonl_line(line, |_| 42).unwrap();
        assert_eq!(conv.turns.len(), 2);
        assert_eq!(conv.turns[0].intent, Intent::Complain);
        assert_eq!(conv.turns[1].intent, Intent::Empathize);
        assert_eq!(conv.outcome, ConvOutcome::Escalated);
    }

    #[test]
    fn parse_jsonl_handles_all_outcomes() {
        for outcome_str in ["Resolved", "Escalated", "Abandoned", "Converted", "Ongoing"] {
            let line = format!(
                r#"{{"id":"x","source":"s","outcome":"{}","turns":[]}}"#,
                outcome_str
            );
            let result = parse_jsonl_line(&line, |_| 0);
            assert!(result.is_ok(), "outcome {} should parse", outcome_str);
        }
    }

    #[test]
    fn parse_jsonl_empty_line_errors() {
        let result = parse_jsonl_line("", |_| 0);
        assert!(result.is_err());
    }

    #[test]
    fn parse_jsonl_unknown_outcome_errors() {
        let line = r#"{"id":"x","source":"s","outcome":"FooBar","turns":[]}"#;
        let result = parse_jsonl_line(line, |_| 0);
        assert!(result.is_err());
    }

    #[test]
    fn ingest_jsonl_file_workflow() {
        use std::io::Write;
        let tmp = std::env::temp_dir().join("zets_dialogue_test.jsonl");
        {
            let mut f = std::fs::File::create(&tmp).unwrap();
            writeln!(f, r#"{{"id":"f1","source":"test","outcome":"Resolved","turns":[{{"speaker":"user","text":"hi","intent":"Greet","emotion":"Neutral","turn_index":0}}]}}"#).unwrap();
            writeln!(f, r#"{{"id":"f2","source":"test","outcome":"Escalated","turns":[{{"speaker":"user","text":"angry","intent":"Complain","emotion":"Anger","turn_index":0}}]}}"#).unwrap();
        }

        let mut store = AtomStore::new();
        let mut log = ProvenanceLog::new();
        let (success, errors) = ingest_dialogues_from_jsonl(&mut store, &mut log, &tmp);
        assert_eq!(success, 2);
        assert!(errors.is_empty());

        assert_eq!(conversations_with_outcome(&store, ConvOutcome::Resolved).len(), 1);
        assert_eq!(conversations_with_outcome(&store, ConvOutcome::Escalated).len(), 1);

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn ingest_jsonl_reuses_speaker_atoms() {
        use std::io::Write;
        let tmp = std::env::temp_dir().join("zets_dialogue_reuse_test.jsonl");
        {
            let mut f = std::fs::File::create(&tmp).unwrap();
            // Two conversations both with speaker "user"
            writeln!(f, r#"{{"id":"r1","source":"t","outcome":"Resolved","turns":[{{"speaker":"user","text":"a","intent":"Greet","emotion":"Neutral","turn_index":0}}]}}"#).unwrap();
            writeln!(f, r#"{{"id":"r2","source":"t","outcome":"Resolved","turns":[{{"speaker":"user","text":"b","intent":"Greet","emotion":"Neutral","turn_index":0}}]}}"#).unwrap();
        }

        let mut store = AtomStore::new();
        let mut log = ProvenanceLog::new();
        let (success, _) = ingest_dialogues_from_jsonl(&mut store, &mut log, &tmp);
        assert_eq!(success, 2);

        // Count atoms matching "speaker:user" — should be exactly 1 due to content dedup
        let hash = crate::atoms::content_hash(b"speaker:user");
        let (atoms, _) = store.snapshot();
        let user_count = atoms.iter().filter(|a| a.content_hash == hash).count();
        assert_eq!(user_count, 1, "speaker atom should be deduplicated");

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn neutral_emotion_skipped() {
        let mut store = AtomStore::new();
        let user = store.put(AtomKind::Concept, b"speaker:user".to_vec());
        let conv = Conversation {
            id: "neutral_test".to_string(),
            source: "test".to_string(),
            outcome: ConvOutcome::Resolved,
            turns: vec![DialogTurn {
                speaker: user,
                text: "Hello there".to_string(),
                intent: Intent::Greet,
                emotion: Emotion::Neutral,
                turn_index: 0,
            }],
        };

        let mut log = ProvenanceLog::new();
        let _ = ingest_dialogue(&mut store, &mut log, &conv);

        // No emotion atom should be created for Neutral
        let emo_neutral_hash = crate::atoms::content_hash("emotion:neutral".as_bytes());
        let (atoms, _) = store.snapshot();
        let has_neutral = atoms.iter().any(|a| a.content_hash == emo_neutral_hash);
        assert!(!has_neutral, "Neutral emotion should not create atom");
    }
}
