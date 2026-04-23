//! # Birur Tuning — Semantic Learning Store (בירור כוונון)
//!
//! The brain's learned knowledge about search quality.
//! NOT hardcoded. Learns from 3 teachers:
//!   1. Users (גולשים) — reward 👍/👎 → most authoritative
//!   2. AI (Gemini) — teaches synonyms, meanings, negatives → high volume
//!   3. Self (Cortex) — coherence_reject, NightMode patterns → always-on
//!
//! Principle: "Nothing hardcoded except fundamental laws.
//!  The test: how would a brilliant human learn this?"
//!
//! Stores: Negatives, Synonyms, Disambiguation, Boosts, Context Rules.
//! Queried at retrieval time to adjust candidate scores.
//! Decays unused tunings over time.
//! Persists to data/tuning.bin.
//!
//! Kabbalistic: 22 paths of Etz Chaim = 22 relation types.
//! birur = בירור = clarification/refinement. The 288 sparks.

use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════
// I. TYPES
// ═══════════════════════════════════════════════════════════════

/// A single piece of learned search knowledge.
#[derive(Clone)]
pub struct TuningEntry {
    pub entry_type: TuningType,
    pub source: TuningSource,
    pub strength: f32,       // 0.0-1.0, decays over time
    pub created_at: u32,     // unix timestamp
    pub last_used: u32,      // unix timestamp
    pub uses: u32,           // how many times this was applied
}

/// What kind of tuning this is.
#[derive(Clone)]
pub enum TuningType {
    /// "חתול" ≠ "טיגריס" — these are NOT the same
    Negative {
        word: String,
        not_word: String,
        reason: String,     // "different species", "phonetic confusion"
    },

    /// "חתול" = "cat" — these ARE the same
    Synonym {
        word_a: String,
        word_b: String,
        bidirectional: bool,
    },

    /// "אריה" → multiple meanings with context clues
    Disambiguation {
        word: String,
        meanings: Vec<DisambigMeaning>,
    },

    /// query "X" → concept Y was good (+) or bad (-)
    RetrievalBoost {
        query_hash: u64,
        concept_id: u32,
        direction: f32,     // +1.0 boost, -1.0 penalize
    },

    /// Context-dependent hint
    ContextHint {
        trigger_words: Vec<String>,   // if these words are present...
        boost_domain: String,          // ...prefer this domain
        penalize_domain: String,       // ...avoid this domain
    },
}

/// One possible meaning of an ambiguous word.
#[derive(Clone)]
pub struct DisambigMeaning {
    pub meaning: String,           // "שם פרטי", "חיה", "מזל"
    pub concept_id: Option<u32>,   // link to concept if known
    pub context_words: Vec<String>, // words that hint at this meaning
    pub probability: f32,           // base probability (0.0-1.0)
    pub reward_count: i32,         // net positive rewards for this meaning
}

/// Who taught this tuning.
#[derive(Clone, Copy, PartialEq)]
pub enum TuningSource {
    User,           // reward 👍/👎 — strength 1.0
    Gemini,         // AI taught — strength 0.7
    SelfCoherence,  // coherence_reject — strength 0.5
    SelfNightMode,  // NightMode pattern — strength 0.6
    SelfPain,       // chronic pain detection — strength 0.5
    Manual,         // /teach endpoint — strength 0.9
}

impl TuningSource {
    fn base_strength(&self) -> f32 {
        match self {
            TuningSource::User => 1.0,
            TuningSource::Manual => 0.9,
            TuningSource::Gemini => 0.7,
            TuningSource::SelfNightMode => 0.6,
            TuningSource::SelfCoherence => 0.5,
            TuningSource::SelfPain => 0.5,
        }
    }

    fn to_byte(&self) -> u8 {
        match self {
            TuningSource::User => 1,
            TuningSource::Gemini => 2,
            TuningSource::SelfCoherence => 3,
            TuningSource::SelfNightMode => 4,
            TuningSource::SelfPain => 5,
            TuningSource::Manual => 6,
        }
    }

    fn from_byte(b: u8) -> Self {
        match b {
            1 => TuningSource::User,
            2 => TuningSource::Gemini,
            3 => TuningSource::SelfCoherence,
            4 => TuningSource::SelfNightMode,
            5 => TuningSource::SelfPain,
            6 => TuningSource::Manual,
            _ => TuningSource::SelfCoherence,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// II. TUNING STORE
// ═══════════════════════════════════════════════════════════════

/// The store of all learned search tunings.
pub struct TuningStore {
    /// Negatives: word_hash → list of negative entries
    negatives: HashMap<u64, Vec<NegativeEntry>>,
    /// Synonyms: word_hash → list of synonym words
    synonyms: HashMap<u64, Vec<SynonymEntry>>,
    /// Disambiguation: word_hash → meanings
    disambiguations: HashMap<u64, Vec<DisambigMeaning>>,
    /// Retrieval boosts: query_hash → (concept_id, direction, strength)
    boosts: HashMap<u64, Vec<BoostEntry>>,
    /// Context hints: trigger_hash → hint
    context_hints: Vec<ContextHintEntry>,
    /// Stats
    pub total_entries: usize,
    total_applied: u64,
}

#[derive(Clone)]
struct NegativeEntry {
    word: String,
    not_word: String,
    reason: String,
    source: TuningSource,
    strength: f32,
    created_at: u32,
    last_used: u32,
    uses: u32,
}

#[derive(Clone)]
struct SynonymEntry {
    word: String,
    synonym: String,
    bidirectional: bool,
    source: TuningSource,
    strength: f32,
}

#[derive(Clone)]
struct BoostEntry {
    concept_id: u32,
    direction: f32,
    source: TuningSource,
    strength: f32,
    uses: u32,
}

#[derive(Clone)]
struct ContextHintEntry {
    trigger_words: Vec<String>,
    boost_domain: String,
    penalize_domain: String,
    source: TuningSource,
    strength: f32,
}

impl TuningStore {
    pub fn new() -> Self {
        TuningStore {
            negatives: HashMap::new(),
            synonyms: HashMap::new(),
            disambiguations: HashMap::new(),
            boosts: HashMap::new(),
            context_hints: Vec::new(),
            total_entries: 0,
            total_applied: 0,
        }
    }

    // ═══════════════════════════════════════════════════════════
    // III. LEARNING — 3 teachers feed the store
    // ═══════════════════════════════════════════════════════════

    /// Teacher 3 (Self): Learn from coherence_reject — "X was matched to Y but they're unrelated"
    pub fn learn_negative(&mut self, query: &str, wrong_concept: &str, reason: &str) {
        let query_lower = query.to_lowercase();
        let concept_lower = wrong_concept.to_lowercase();

        // Extract meaningful words from both
        let q_words = extract_content_words(&query_lower);
        let c_words = extract_content_words(&concept_lower);

        // Create negative entries for each query_word ↔ concept_word pair
        for qw in &q_words {
            let h = hash_word(qw);
            for cw in &c_words {
                // Don't create negative between identical words
                if qw == cw { continue; }

                let entry = NegativeEntry {
                    word: qw.clone(),
                    not_word: cw.clone(),
                    reason: reason.into(),
                    source: TuningSource::SelfCoherence,
                    strength: TuningSource::SelfCoherence.base_strength(),
                    created_at: now_unix(),
                    last_used: now_unix(),
                    uses: 0,
                };

                let list = self.negatives.entry(h).or_insert_with(Vec::new);
                // Don't duplicate
                if !list.iter().any(|e| e.not_word == *cw) {
                    list.push(entry);
                    self.total_entries += 1;
                }
            }
        }

        // Also create a boost penalty: this specific query → this concept = BAD
        let qh = hash_word(&query_lower);
        let c_id_guess = hash_word(&concept_lower) as u32; // approximate
        let boosts = self.boosts.entry(qh).or_insert_with(Vec::new);
        if !boosts.iter().any(|b| b.concept_id == c_id_guess && b.direction < 0.0) {
            boosts.push(BoostEntry {
                concept_id: c_id_guess,
                direction: -1.0,
                source: TuningSource::SelfCoherence,
                strength: 0.5,
                uses: 0,
            });
        }
    }

    /// Teacher 1 (User): Learn from reward — user said 👍 or 👎
    pub fn learn_from_reward(&mut self, query: &str, concept_text: &str, concept_id: u32, positive: bool) {
        let qh = hash_word(&query.to_lowercase());
        let boosts = self.boosts.entry(qh).or_insert_with(Vec::new);

        // Find existing boost for this concept
        let existing = boosts.iter_mut().find(|b| b.concept_id == concept_id);
        if let Some(b) = existing {
            // Reinforce or weaken
            if positive {
                b.direction = (b.direction + 0.3).min(1.0);
                b.strength = (b.strength + 0.1).min(1.0);
            } else {
                b.direction = (b.direction - 0.3).max(-1.0);
                b.strength = (b.strength + 0.1).min(1.0); // strength INCREASES (we're more sure)
            }
            b.source = TuningSource::User; // User source overrides
            b.uses += 1;
        } else {
            boosts.push(BoostEntry {
                concept_id,
                direction: if positive { 0.5 } else { -0.5 },
                source: TuningSource::User,
                strength: TuningSource::User.base_strength(),
                uses: 1,
            });
            self.total_entries += 1;
        }

        // If negative reward: also create negative pair
        if !positive {
            self.learn_negative(query, concept_text, "user_thumbs_down");
        }
    }

    /// Teacher 2 (AI): Learn synonyms from Gemini
    pub fn learn_synonym(&mut self, word: &str, synonym: &str, bidirectional: bool) {
        let w = word.to_lowercase();
        let s = synonym.to_lowercase();
        let h = hash_word(&w);

        let list = self.synonyms.entry(h).or_insert_with(Vec::new);
        if !list.iter().any(|e| e.synonym == s) {
            list.push(SynonymEntry {
                word: w.clone(),
                synonym: s.clone(),
                bidirectional,
                source: TuningSource::Gemini,
                strength: TuningSource::Gemini.base_strength(),
            });
            self.total_entries += 1;
        }

        // If bidirectional, also store reverse
        if bidirectional {
            let rh = hash_word(&s);
            let rlist = self.synonyms.entry(rh).or_insert_with(Vec::new);
            if !rlist.iter().any(|e| e.synonym == w) {
                rlist.push(SynonymEntry {
                    word: s,
                    synonym: w,
                    bidirectional: true,
                    source: TuningSource::Gemini,
                    strength: TuningSource::Gemini.base_strength(),
                });
            }
        }
    }

    /// Teacher 2 (AI): Learn disambiguation from Gemini
    pub fn learn_disambiguation(&mut self, word: &str, meanings: Vec<DisambigMeaning>) {
        let h = hash_word(&word.to_lowercase());
        self.disambiguations.insert(h, meanings);
        self.total_entries += 1;
    }

    /// Teacher 2 (AI): Learn negative from Gemini ("X is NOT Y")
    pub fn learn_negative_from_gemini(&mut self, word: &str, not_word: &str, reason: &str) {
        let h = hash_word(&word.to_lowercase());
        let list = self.negatives.entry(h).or_insert_with(Vec::new);
        if !list.iter().any(|e| e.not_word == not_word.to_lowercase()) {
            list.push(NegativeEntry {
                word: word.to_lowercase(),
                not_word: not_word.to_lowercase(),
                reason: reason.into(),
                source: TuningSource::Gemini,
                strength: TuningSource::Gemini.base_strength(),
                created_at: now_unix(),
                last_used: now_unix(),
                uses: 0,
            });
            self.total_entries += 1;
        }
    }

    /// Teacher 0 (Manual): /teach endpoint for direct tuning
    pub fn teach_negative(&mut self, word: &str, not_word: &str, reason: &str) {
        let h = hash_word(&word.to_lowercase());
        let list = self.negatives.entry(h).or_insert_with(Vec::new);
        list.push(NegativeEntry {
            word: word.to_lowercase(),
            not_word: not_word.to_lowercase(),
            reason: reason.into(),
            source: TuningSource::Manual,
            strength: TuningSource::Manual.base_strength(),
            created_at: now_unix(),
            last_used: now_unix(),
            uses: 0,
        });
        self.total_entries += 1;
    }

    // ═══════════════════════════════════════════════════════════
    // IV. QUERY — adjust retrieval at search time
    // ═══════════════════════════════════════════════════════════

    /// Adjust retrieval candidates based on learned tunings.
    /// Called in pipeline after Gevurah, before confidence.
    /// Modifies scores in-place: boosts good matches, penalizes bad ones.
    pub fn adjust_retrieval(
        &mut self,
        query: &str,
        candidates: &mut Vec<(u32, f64, String)>, // (concept_id, score, concept_text)
        _session_context: &[String],
    ) -> usize {
        let query_lower = query.to_lowercase();
        let q_words = extract_content_words(&query_lower);
        let qh = hash_word(&query_lower);
        let mut adjustments = 0usize;

        for (cid, score, concept_text) in candidates.iter_mut() {
            let c_lower = concept_text.to_lowercase();
            let c_words = extract_content_words(&c_lower);
            let mut penalty = 0.0f64;
            let mut boost = 0.0f64;

            // ═══ Check negatives: any query word has a negative against any concept word? ═══
            for qw in &q_words {
                let h = hash_word(qw);
                if let Some(negs) = self.negatives.get_mut(&h) {
                    for neg in negs.iter_mut() {
                        if c_words.iter().any(|cw| *cw == neg.not_word || c_lower.contains(&neg.not_word)) {
                            penalty += neg.strength as f64 * 0.3;
                            neg.uses += 1;
                            neg.last_used = now_unix();
                            adjustments += 1;
                        }
                    }
                }
            }

            // ═══ Check boosts: specific query→concept reinforcement ═══
            if let Some(boosts) = self.boosts.get_mut(&qh) {
                for b in boosts.iter_mut() {
                    if b.concept_id == *cid {
                        if b.direction > 0.0 {
                            boost += b.direction as f64 * b.strength as f64 * 0.2;
                        } else {
                            penalty += (-b.direction) as f64 * b.strength as f64 * 0.3;
                        }
                        b.uses += 1;
                        adjustments += 1;
                    }
                }
            }

            // ═══ Check synonyms: does query contain a synonym that maps to this concept? ═══
            for qw in &q_words {
                let h = hash_word(qw);
                if let Some(syns) = self.synonyms.get(&h) {
                    for syn in syns {
                        if c_words.iter().any(|cw| *cw == syn.synonym) || c_lower.contains(&syn.synonym) {
                            boost += syn.strength as f64 * 0.15;
                            adjustments += 1;
                        }
                    }
                }
            }

            // Apply adjustments
            *score = (*score + boost - penalty).max(0.0).min(1.0);
        }

        self.total_applied += adjustments as u64;
        adjustments
    }

    /// Disambiguate a word given context.
    /// Returns the most likely concept_id for this word in this context.
    pub fn disambiguate(&self, word: &str, context: &[String]) -> Option<(u32, String, f32)> {
        let h = hash_word(&word.to_lowercase());
        let meanings = self.disambiguations.get(&h)?;
        if meanings.is_empty() { return None; }

        let context_lower: Vec<String> = context.iter().map(|w| w.to_lowercase()).collect();

        // Score each meaning by context match
        let mut best_score = 0.0f32;
        let mut best: Option<&DisambigMeaning> = None;

        for m in meanings {
            let mut score = m.probability; // base probability

            // Context word match: each matching context word adds 0.15
            for ctx in &m.context_words {
                if context_lower.iter().any(|c| c.contains(ctx) || ctx.contains(c.as_str())) {
                    score += 0.15;
                }
            }

            // Reward history: each net positive reward adds 0.05
            score += (m.reward_count as f32 * 0.05).max(-0.3).min(0.3);

            if score > best_score {
                best_score = score;
                best = Some(m);
            }
        }

        best.and_then(|m| {
            m.concept_id.map(|cid| (cid, m.meaning.clone(), best_score))
        })
    }

    /// Check if query word has a known negative against concept word.
    /// Fast check — used in coherence gate to replace hardcoded lists.
    pub fn has_negative(&self, query_word: &str, concept_word: &str) -> bool {
        let h = hash_word(&query_word.to_lowercase());
        if let Some(negs) = self.negatives.get(&h) {
            let cw = concept_word.to_lowercase();
            negs.iter().any(|n| n.not_word == cw || cw.contains(&n.not_word))
        } else {
            false
        }
    }

    /// Get synonyms for a word.
    pub fn get_synonyms(&self, word: &str) -> Vec<String> {
        let h = hash_word(&word.to_lowercase());
        self.synonyms.get(&h)
            .map(|syns| syns.iter().map(|s| s.synonym.clone()).collect())
            .unwrap_or_default()
    }

    // ═══════════════════════════════════════════════════════════
    // V. NIGHTMODE — pattern analysis + decay
    // ═══════════════════════════════════════════════════════════

    /// NightMode: decay unused tunings, garbage collect dead ones.
    /// Called once per NightMode cycle (daily 3AM).
    pub fn nightmode_decay(&mut self) -> (usize, usize) {
        let mut decayed = 0usize;
        let mut removed = 0usize;
        let decay_rate = 0.02; // 2% per day

        // Decay negatives
        for (_, entries) in self.negatives.iter_mut() {
            entries.retain_mut(|e| {
                if e.uses == 0 && (now_unix() - e.last_used) > 86400 {
                    e.strength -= decay_rate;
                    decayed += 1;
                }
                if e.strength <= 0.05 {
                    removed += 1;
                    self.total_entries = self.total_entries.saturating_sub(1);
                    false // remove
                } else {
                    true // keep
                }
            });
        }

        // Decay boosts
        for (_, entries) in self.boosts.iter_mut() {
            entries.retain_mut(|e| {
                if e.uses == 0 {
                    e.strength -= decay_rate;
                    decayed += 1;
                }
                if e.strength <= 0.05 {
                    removed += 1;
                    false
                } else {
                    true
                }
            });
        }

        // Clean empty buckets
        self.negatives.retain(|_, v| !v.is_empty());
        self.boosts.retain(|_, v| !v.is_empty());

        (decayed, removed)
    }

    /// NightMode: analyze query patterns and auto-generate tunings.
    /// Takes list of (query, concept_id, was_correct) from recent history.
    pub fn nightmode_analyze(&mut self, recent_queries: &[(String, u32, bool)]) -> usize {
        let mut generated = 0;

        // Find confusion clusters: same query → different concepts → mixed success
        let mut query_results: HashMap<String, Vec<(u32, bool)>> = HashMap::new();
        for (q, cid, ok) in recent_queries {
            let key = q.to_lowercase();
            query_results.entry(key).or_default().push((*cid, *ok));
        }

        for (query, results) in &query_results {
            if results.len() < 2 { continue; }

            // Find concepts that were bad for this query
            let bad_concepts: Vec<u32> = results.iter()
                .filter(|(_, ok)| !ok)
                .map(|(cid, _)| *cid)
                .collect();

            // Find concepts that were good
            let good_concepts: Vec<u32> = results.iter()
                .filter(|(_, ok)| *ok)
                .map(|(cid, _)| *cid)
                .collect();

            // Create boosts
            let qh = hash_word(query);
            let boosts = self.boosts.entry(qh).or_insert_with(Vec::new);
            for cid in &bad_concepts {
                if !boosts.iter().any(|b| b.concept_id == *cid) {
                    boosts.push(BoostEntry {
                        concept_id: *cid,
                        direction: -0.5,
                        source: TuningSource::SelfNightMode,
                        strength: 0.6,
                        uses: 0,
                    });
                    generated += 1;
                }
            }
            for cid in &good_concepts {
                if !boosts.iter().any(|b| b.concept_id == *cid) {
                    boosts.push(BoostEntry {
                        concept_id: *cid,
                        direction: 0.5,
                        source: TuningSource::SelfNightMode,
                        strength: 0.6,
                        uses: 0,
                    });
                    generated += 1;
                }
            }
        }

        generated
    }

    // ═══════════════════════════════════════════════════════════
    // VI. PERSISTENCE — save/load binary
    // ═══════════════════════════════════════════════════════════

    pub fn save(&self, path: &str) -> Result<(), String> {
        let mut data: Vec<u8> = Vec::new();

        // Magic + version
        data.extend_from_slice(b"BTUN"); // Birur TUNing
        data.push(1); // version

        // ═══ Negatives ═══
        let neg_count: u32 = self.negatives.values().map(|v| v.len() as u32).sum();
        data.extend_from_slice(&neg_count.to_le_bytes());
        for entries in self.negatives.values() {
            for e in entries {
                data.push(1); // type = negative
                data.push(e.source.to_byte());
                data.extend_from_slice(&e.strength.to_le_bytes());
                data.extend_from_slice(&e.created_at.to_le_bytes());
                data.extend_from_slice(&e.uses.to_le_bytes());
                write_string(&mut data, &e.word);
                write_string(&mut data, &e.not_word);
                write_string(&mut data, &e.reason);
            }
        }

        // ═══ Synonyms ═══
        let syn_count: u32 = self.synonyms.values().map(|v| v.len() as u32).sum();
        data.extend_from_slice(&syn_count.to_le_bytes());
        for entries in self.synonyms.values() {
            for e in entries {
                data.push(2); // type = synonym
                data.push(e.source.to_byte());
                data.extend_from_slice(&e.strength.to_le_bytes());
                data.push(if e.bidirectional { 1 } else { 0 });
                write_string(&mut data, &e.word);
                write_string(&mut data, &e.synonym);
            }
        }

        // ═══ Boosts ═══
        let boost_count: u32 = self.boosts.values().map(|v| v.len() as u32).sum();
        data.extend_from_slice(&boost_count.to_le_bytes());
        for (qh, entries) in &self.boosts {
            for e in entries {
                data.push(3); // type = boost
                data.push(e.source.to_byte());
                data.extend_from_slice(&e.strength.to_le_bytes());
                data.extend_from_slice(&qh.to_le_bytes());
                data.extend_from_slice(&e.concept_id.to_le_bytes());
                data.extend_from_slice(&e.direction.to_le_bytes());
                data.extend_from_slice(&e.uses.to_le_bytes());
            }
        }

        // Write to file
        std::fs::write(path, &data).map_err(|e| format!("save failed: {}", e))?;
        Ok(())
    }

    pub fn load(path: &str) -> Option<Self> {
        let data = std::fs::read(path).ok()?;
        if data.len() < 5 { return None; }
        if &data[0..4] != b"BTUN" { return None; }
        if data[4] != 1 { return None; } // version check

        let mut store = TuningStore::new();
        let mut pos = 5;

        // ═══ Negatives ═══
        if pos + 4 > data.len() { return Some(store); }
        let neg_count = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
        pos += 4;
        for _ in 0..neg_count {
            if pos + 2 > data.len() { break; }
            let _type_byte = data[pos]; pos += 1;
            let source = TuningSource::from_byte(data[pos]); pos += 1;
            if pos + 12 > data.len() { break; }
            let strength = f32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]); pos += 4;
            let created_at = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]); pos += 4;
            let uses = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]); pos += 4;
            let (word, new_pos) = read_string(&data, pos)?; pos = new_pos;
            let (not_word, new_pos) = read_string(&data, pos)?; pos = new_pos;
            let (reason, new_pos) = read_string(&data, pos)?; pos = new_pos;

            let h = hash_word(&word);
            store.negatives.entry(h).or_default().push(NegativeEntry {
                word, not_word, reason, source, strength, created_at, last_used: created_at, uses,
            });
            store.total_entries += 1;
        }

        // ═══ Synonyms ═══
        if pos + 4 > data.len() { return Some(store); }
        let syn_count = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
        pos += 4;
        for _ in 0..syn_count {
            if pos + 2 > data.len() { break; }
            let _type_byte = data[pos]; pos += 1;
            let source = TuningSource::from_byte(data[pos]); pos += 1;
            if pos + 5 > data.len() { break; }
            let strength = f32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]); pos += 4;
            let bidi = data[pos] == 1; pos += 1;
            let (word, new_pos) = read_string(&data, pos)?; pos = new_pos;
            let (synonym, new_pos) = read_string(&data, pos)?; pos = new_pos;

            let h = hash_word(&word);
            store.synonyms.entry(h).or_default().push(SynonymEntry {
                word, synonym, bidirectional: bidi, source, strength,
            });
            store.total_entries += 1;
        }

        // ═══ Boosts ═══
        if pos + 4 > data.len() { return Some(store); }
        let boost_count = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
        pos += 4;
        for _ in 0..boost_count {
            if pos + 2 > data.len() { break; }
            let _type_byte = data[pos]; pos += 1;
            let source = TuningSource::from_byte(data[pos]); pos += 1;
            if pos + 24 > data.len() { break; }
            let strength = f32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]); pos += 4;
            let qh = u64::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3], data[pos+4], data[pos+5], data[pos+6], data[pos+7]]); pos += 8;
            let concept_id = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]); pos += 4;
            let direction = f32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]); pos += 4;
            let uses = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]); pos += 4;

            store.boosts.entry(qh).or_default().push(BoostEntry {
                concept_id, direction, source, strength, uses,
            });
            store.total_entries += 1;
        }

        Some(store)
    }

    // ═══════════════════════════════════════════════════════════
    // VII. STATS
    // ═══════════════════════════════════════════════════════════

    pub fn stats(&self) -> TuningStats {
        TuningStats {
            total_entries: self.total_entries,
            negatives: self.negatives.values().map(|v| v.len()).sum(),
            synonyms: self.synonyms.values().map(|v| v.len()).sum(),
            disambiguations: self.disambiguations.len(),
            boosts: self.boosts.values().map(|v| v.len()).sum(),
            context_hints: self.context_hints.len(),
            total_applied: self.total_applied,
        }
    }
}

pub struct TuningStats {
    pub total_entries: usize,
    pub negatives: usize,
    pub synonyms: usize,
    pub disambiguations: usize,
    pub boosts: usize,
    pub context_hints: usize,
    pub total_applied: u64,
}

// ═══════════════════════════════════════════════════════════════
// VIII. HELPERS
// ═══════════════════════════════════════════════════════════════

fn hash_word(word: &str) -> u64 {
    // FNV-1a hash
    let mut h: u64 = 0xcbf29ce484222325;
    for b in word.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn now_unix() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32
}

fn extract_content_words(text: &str) -> Vec<String> {
    let stopwords = [
        "מה", "זה", "של", "את", "על", "עם", "בין", "או", "גם", "לא",
        "כל", "הוא", "היא", "ב", "ל", "מ", "ה", "ו", "כ", "ש",
        "the", "a", "an", "is", "are", "was", "in", "on", "at", "to",
        "for", "with", "from", "and", "or", "but", "not",
    ];
    text.split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| w.len() > 1 && !stopwords.contains(w))
        .map(|w| w.to_string())
        .collect()
}

fn write_string(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    let len = bytes.len().min(u16::MAX as usize) as u16;
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(&bytes[..len as usize]);
}

fn read_string(data: &[u8], pos: usize) -> Option<(String, usize)> {
    if pos + 2 > data.len() { return None; }
    let len = u16::from_le_bytes([data[pos], data[pos+1]]) as usize;
    let start = pos + 2;
    if start + len > data.len() { return None; }
    let s = String::from_utf8_lossy(&data[start..start+len]).to_string();
    Some((s, start + len))
}

// ═══════════════════════════════════════════════════════════════
// IX. TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learn_negative() {
        let mut store = TuningStore::new();
        store.learn_negative("חתול", "טיגריס לבן", "wrong animal");
        assert!(store.has_negative("חתול", "טיגריס"));
        assert!(!store.has_negative("חתול", "חתלתול"));
    }

    #[test]
    fn test_learn_synonym() {
        let mut store = TuningStore::new();
        store.learn_synonym("חתול", "cat", true);
        let syns = store.get_synonyms("חתול");
        assert!(syns.contains(&"cat".to_string()));
        let reverse = store.get_synonyms("cat");
        assert!(reverse.contains(&"חתול".to_string()));
    }

    #[test]
    fn test_learn_from_reward_positive() {
        let mut store = TuningStore::new();
        store.learn_from_reward("מה זה חשמל", "חשמל", 216, true);
        let stats = store.stats();
        assert!(stats.boosts >= 1);
    }

    #[test]
    fn test_learn_from_reward_negative() {
        let mut store = TuningStore::new();
        store.learn_from_reward("כמה זה 2+2", "כיפה סאטן", 100632, false);
        let stats = store.stats();
        assert!(stats.negatives >= 1); // negative pair created
        assert!(stats.boosts >= 1);    // boost penalty created
    }

    #[test]
    fn test_adjust_retrieval() {
        let mut store = TuningStore::new();
        store.learn_negative("חתול", "טיגריס", "wrong");

        let mut candidates = vec![
            (1, 0.8, "חתול ביתי".to_string()),
            (2, 0.7, "טיגריס לבן".to_string()),
        ];
        let adj = store.adjust_retrieval("חתול", &mut candidates, &[]);
        assert!(adj > 0);
        // Tigris should be penalized
        assert!(candidates[1].1 < 0.7);
    }

    #[test]
    fn test_disambiguation() {
        let mut store = TuningStore::new();
        store.learn_disambiguation("אריה", vec![
            DisambigMeaning {
                meaning: "שם פרטי".into(),
                concept_id: Some(1),
                context_words: vec!["הלכת".into(), "דיברת".into(), "של".into()],
                probability: 0.45,
                reward_count: 0,
            },
            DisambigMeaning {
                meaning: "חיה".into(),
                concept_id: Some(2),
                context_words: vec!["חיות".into(), "ספארי".into(), "טורף".into()],
                probability: 0.35,
                reward_count: 0,
            },
        ]);

        // Context: person — "הלכת" matches person context (+0.15) → 0.60 > 0.35
        let result = store.disambiguate("אריה", &["הלכת".into(), "ל".into()]);
        assert!(result.is_some());
        let (cid, meaning, _) = result.unwrap();
        assert_eq!(cid, 1);
        assert_eq!(meaning, "שם פרטי");

        // Context: animal — "חיות"+"ספארי" match animal context (+0.30) → 0.65 > 0.45
        let result = store.disambiguate("אריה", &["חיות".into(), "ספארי".into()]);
        assert!(result.is_some());
        let (cid, _, _) = result.unwrap();
        assert_eq!(cid, 2);
    }

    #[test]
    fn test_save_load() {
        let mut store = TuningStore::new();
        store.learn_negative("מחברת", "מחבת", "phonetic confusion");
        store.learn_synonym("notebook", "מחברת", true);
        store.learn_from_reward("test", "concept", 42, false);

        let path = "/tmp/test_tuning.bin";
        store.save(path).unwrap();

        let loaded = TuningStore::load(path).unwrap();
        assert!(loaded.has_negative("מחברת", "מחבת"));
        assert!(loaded.get_synonyms("notebook").contains(&"מחברת".to_string()));
        let stats = loaded.stats();
        assert!(stats.negatives > 0);
        assert!(stats.synonyms > 0);
    }

    #[test]
    fn test_nightmode_decay() {
        let mut store = TuningStore::new();
        // Add entry with 0 uses and very old timestamp
        let h = hash_word("test");
        store.negatives.entry(h).or_default().push(NegativeEntry {
            word: "test".into(),
            not_word: "wrong".into(),
            reason: "test".into(),
            source: TuningSource::SelfCoherence,
            strength: 0.1, // almost dead
            created_at: 0,
            last_used: 0, // very old
            uses: 0,
        });
        store.total_entries = 1;

        let (decayed, removed) = store.nightmode_decay();
        assert!(decayed > 0 || removed > 0);
    }
}
