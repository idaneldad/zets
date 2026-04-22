//! Session context — working memory that anchors search and resolves ambiguity.
//!
//! Idea (Idan's insight, 22 Apr 2026): every conversation generates its own
//! subgraph. Atoms mentioned during the conversation receive ACTIVATION.
//! Later queries bias toward currently-active atoms, resolving ambiguity
//! ("crown" in dental context vs royal context) and priming retrieval
//! ("dogs are topic" narrows search space).
//!
//! This is classic SPREADING ACTIVATION (Collins & Loftus 1975), adapted
//! for persistent graph storage with three decay layers:
//!
//!   - Working memory: decays over seconds-to-minutes (current turn)
//!   - Session memory: decays over hours (this conversation)
//!   - Long-term: no decay, but requires re-activation via similar context
//!
//! The design stays DETERMINISTIC. No randomness. Same sequence of
//! mentions + same time deltas → same activation map.

use std::collections::HashMap;

use crate::atoms::AtomId;

/// Activation level 0.0 to 1.0. 1.0 = just mentioned; 0.0 = fully decayed.
pub type Activation = f32;

/// One entry in the session's active atom map.
#[derive(Debug, Clone, Copy)]
pub struct ActiveAtom {
    pub atom_id: AtomId,
    /// Current activation, 0.0 to 1.0 (may drift slightly above due to repeated mentions)
    pub activation: f32,
    /// Logical timestamp when last mentioned (monotonic counter — turn number)
    pub last_mentioned_turn: u64,
}

/// A session = a conversation = a short-term memory buffer over atoms.
///
/// Activation model:
///   - When an atom is mentioned: activation = min(1.0, current + BOOST)
///   - On each new turn: activation *= DECAY_PER_TURN
///   - Atoms below PRUNE_THRESHOLD are removed to keep map small
///
/// Typical values (calibrated to feel like human working memory):
///   BOOST = 1.0          (full re-activation on mention)
///   DECAY_PER_TURN = 0.85 (atom from 5 turns ago: 0.85^5 = 0.44 remaining)
///   PRUNE_THRESHOLD = 0.05 (atoms drop out after ~20 turns of silence)
#[derive(Debug, Clone)]
pub struct SessionContext {
    /// Active atoms with their activation levels
    pub active: HashMap<AtomId, ActiveAtom>,
    /// Turn counter (increments on each new utterance)
    pub current_turn: u64,
    /// Activation given when an atom is freshly mentioned
    pub boost: f32,
    /// Multiplicative decay per turn (0.85 = 15% loss per turn)
    pub decay_per_turn: f32,
    /// Activation below this → atom pruned from session
    pub prune_threshold: f32,
    /// Maximum atoms kept in session (LRU-style)
    pub max_size: usize,
}

impl SessionContext {
    /// Create a fresh session with default calibration.
    pub fn new() -> Self {
        Self {
            active: HashMap::new(),
            current_turn: 0,
            boost: 1.0,
            decay_per_turn: 0.85,
            prune_threshold: 0.05,
            max_size: 256,
        }
    }

    /// Create a session with custom calibration.
    pub fn with_params(boost: f32, decay: f32, prune: f32, max_size: usize) -> Self {
        Self {
            active: HashMap::new(),
            current_turn: 0,
            boost,
            decay_per_turn: decay,
            prune_threshold: prune,
            max_size,
        }
    }

    /// Advance to next turn — applies decay to all active atoms and prunes weak ones.
    pub fn advance_turn(&mut self) {
        self.current_turn += 1;
        let cutoff = self.prune_threshold;
        // Apply decay, collect atoms to remove
        let mut to_prune: Vec<AtomId> = Vec::new();
        for (aid, entry) in self.active.iter_mut() {
            entry.activation *= self.decay_per_turn;
            if entry.activation < cutoff {
                to_prune.push(*aid);
            }
        }
        for aid in to_prune {
            self.active.remove(&aid);
        }
    }

    /// Record that an atom was mentioned — boost its activation.
    /// Does NOT advance the turn; caller controls turn pacing.
    pub fn mention(&mut self, atom_id: AtomId) {
        let entry = self.active.entry(atom_id).or_insert(ActiveAtom {
            atom_id,
            activation: 0.0,
            last_mentioned_turn: self.current_turn,
        });
        entry.activation = (entry.activation + self.boost).min(1.0);
        entry.last_mentioned_turn = self.current_turn;

        // Enforce max_size via LRU-style eviction if needed
        if self.active.len() > self.max_size {
            self.evict_weakest();
        }
    }

    /// Mention multiple atoms at once (one turn = whole utterance's concepts).
    pub fn mention_all(&mut self, atoms: &[AtomId]) {
        for &a in atoms {
            self.mention(a);
        }
    }

    /// Evict the atom with lowest activation (LRU-ish).
    fn evict_weakest(&mut self) {
        if let Some((&weakest_id, _)) = self.active.iter()
            .min_by(|(_, a), (_, b)| {
                a.activation.partial_cmp(&b.activation)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            self.active.remove(&weakest_id);
        }
    }

    /// Current activation of an atom (0.0 if not in session).
    pub fn activation_of(&self, atom_id: AtomId) -> f32 {
        self.active.get(&atom_id).map(|e| e.activation).unwrap_or(0.0)
    }

    /// Is this atom currently in the session?
    pub fn is_active(&self, atom_id: AtomId) -> bool {
        self.active.contains_key(&atom_id)
    }

    /// Top-K most active atoms (sorted by activation descending).
    pub fn top_k(&self, k: usize) -> Vec<(AtomId, f32)> {
        let mut v: Vec<(AtomId, f32)> = self.active.iter()
            .map(|(&aid, e)| (aid, e.activation))
            .collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        v.truncate(k);
        v
    }

    /// All currently-active atom_ids (useful as seeds for spreading activation).
    pub fn active_ids(&self) -> Vec<AtomId> {
        self.active.keys().copied().collect()
    }

    pub fn size(&self) -> usize { self.active.len() }
    pub fn is_empty(&self) -> bool { self.active.is_empty() }
}

impl Default for SessionContext {
    fn default() -> Self { Self::new() }
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_session_empty() {
        let s = SessionContext::new();
        assert!(s.is_empty());
        assert_eq!(s.current_turn, 0);
    }

    #[test]
    fn mention_activates() {
        let mut s = SessionContext::new();
        s.mention(42);
        assert!(s.is_active(42));
        assert_eq!(s.activation_of(42), 1.0);
    }

    #[test]
    fn decay_reduces_activation_per_turn() {
        let mut s = SessionContext::new();
        s.mention(100);
        assert_eq!(s.activation_of(100), 1.0);
        s.advance_turn();
        // After 1 turn: 1.0 * 0.85 = 0.85
        assert!((s.activation_of(100) - 0.85).abs() < 0.01);
        s.advance_turn();
        // After 2 turns: 0.85 * 0.85 = 0.7225
        assert!((s.activation_of(100) - 0.7225).abs() < 0.01);
    }

    #[test]
    fn old_atoms_pruned_below_threshold() {
        let mut s = SessionContext::new();
        s.mention(1);
        // Advance enough turns: after 20 turns, 0.85^20 ≈ 0.039 (below 0.05)
        for _ in 0..20 {
            s.advance_turn();
        }
        assert!(!s.is_active(1), "atom should be pruned after 20 turns of silence");
    }

    #[test]
    fn re_mention_refreshes_activation() {
        let mut s = SessionContext::new();
        s.mention(5);
        s.advance_turn();
        s.advance_turn();
        assert!(s.activation_of(5) < 0.75); // decayed

        s.mention(5); // re-mention
        assert_eq!(s.activation_of(5), 1.0); // back to max (capped at 1.0)
    }

    #[test]
    fn top_k_returns_most_active_first() {
        let mut s = SessionContext::new();
        s.mention(10);
        s.advance_turn();
        s.advance_turn();
        // atom 10 has decayed twice
        s.mention(20); // fresh
        s.mention(30); // fresh
        s.advance_turn();
        // Now: 10 decayed 3 times, 20 and 30 decayed once

        let top = s.top_k(2);
        assert_eq!(top.len(), 2);
        // 20 and 30 should be more active than 10
        assert!(!top.iter().any(|(id, _)| *id == 10));
    }

    #[test]
    fn max_size_evicts_weakest() {
        let mut s = SessionContext::with_params(1.0, 0.85, 0.05, 3);
        s.mention(1);
        s.advance_turn(); // 1 → 0.85
        s.advance_turn(); // 1 → 0.7225
        s.mention(2);
        s.advance_turn(); // 1 → 0.61, 2 → 0.85
        s.mention(3);
        s.mention(4); // Over capacity, should evict weakest (atom 1)

        assert!(!s.is_active(1), "weakest should be evicted");
        assert!(s.is_active(2));
        assert!(s.is_active(3));
        assert!(s.is_active(4));
    }

    #[test]
    fn determinism_identical_sequence_identical_state() {
        let mut s1 = SessionContext::new();
        let mut s2 = SessionContext::new();

        let seq = [(10, 0), (20, 1), (10, 2), (30, 2), (20, 5)];
        for (atom, turns) in seq {
            for _ in 0..turns { s1.advance_turn(); s2.advance_turn(); }
            s1.mention(atom);
            s2.mention(atom);
        }

        // Both sessions must have identical state
        assert_eq!(s1.size(), s2.size());
        for aid in s1.active_ids() {
            assert_eq!(s1.activation_of(aid), s2.activation_of(aid));
        }
    }

    #[test]
    fn mention_all_sets_multiple() {
        let mut s = SessionContext::new();
        s.mention_all(&[1, 2, 3]);
        assert_eq!(s.size(), 3);
        for id in 1..=3 {
            assert_eq!(s.activation_of(id), 1.0);
        }
    }

    #[test]
    fn activation_caps_at_one() {
        let mut s = SessionContext::new();
        s.mention(1);
        s.mention(1);
        s.mention(1); // mention 3 times without decay
        assert_eq!(s.activation_of(1), 1.0); // capped
    }
}
