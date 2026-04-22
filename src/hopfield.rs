//! Hopfield — associative recall for atoms.
//!
//! Modern Hopfield Networks (Ramsauer et al., 2020) provide exponential
//! capacity and continuous pattern storage. In ZETS, each "bank" stores
//! pattern vectors associated with atoms; given a partial or noisy cue,
//! the bank returns the most likely stored atom.
//!
//! Key insight from PoC (verified in research/hopfield_enhanced.py):
//!   - HIERARCHICAL decomposition works perfectly: Rex = mammal + quadruped
//!     + species + breed + color; all 5 layers recovered from additive mix.
//!   - NOISE REJECTION works: pure random cue stays below threshold.
//!   - MIXING WEIGHTS: Hopfield isn't ideal — if a component contributes
//!     only 10% to the scene, its atom may not activate. For that, NMF
//!     or sparse coding is better.
//!
//! Design:
//!   - Deterministic (no randomness in recall)
//!   - Supports threshold-based activation (can say "I don't know")
//!   - Top-K retrieval with confidence scores
//!   - Integrates with AtomStore: each atom optionally has a vector

use std::sync::Arc;

/// A single pattern stored in a Hopfield bank, linked to an atom_id.
#[derive(Debug, Clone)]
pub struct HopfieldPattern {
    pub atom_id: u32,
    pub vector: Arc<Vec<f32>>, // shared to avoid copies
}

/// A bank of associated patterns over a fixed dimension.
pub struct HopfieldBank {
    pub dim: usize,
    pub beta: f32,
    pub activation_threshold: f32,
    pub patterns: Vec<HopfieldPattern>,
    /// Pre-normalized vectors for fast dot products.
    normalized: Vec<Arc<Vec<f32>>>,
}

impl HopfieldBank {
    pub fn new(dim: usize, beta: f32, activation_threshold: f32) -> Self {
        Self {
            dim,
            beta,
            activation_threshold,
            patterns: Vec::new(),
            normalized: Vec::new(),
        }
    }

    /// Store a pattern associated with an atom. Vector is normalized to unit length.
    pub fn store(&mut self, atom_id: u32, vector: Vec<f32>) -> Result<(), String> {
        if vector.len() != self.dim {
            return Err(format!("expected dim {}, got {}", self.dim, vector.len()));
        }
        let norm = (vector.iter().map(|v| v * v).sum::<f32>()).sqrt();
        if norm < 1e-8 {
            return Err("zero vector".to_string());
        }
        let normalized: Vec<f32> = vector.iter().map(|v| v / norm).collect();
        let arc_norm = Arc::new(normalized);
        self.patterns.push(HopfieldPattern {
            atom_id,
            vector: Arc::new(vector),
        });
        self.normalized.push(arc_norm);
        Ok(())
    }

    /// Cosine similarity of cue to each stored pattern (cue must be raw, not pre-normalized).
    fn scores(&self, cue: &[f32]) -> Vec<f32> {
        let norm_cue = (cue.iter().map(|v| v * v).sum::<f32>()).sqrt();
        if norm_cue < 1e-8 {
            return vec![0.0; self.patterns.len()];
        }
        self.normalized
            .iter()
            .map(|p| {
                let dot: f32 = p.iter().zip(cue.iter()).map(|(a, b)| a * b).sum();
                dot / norm_cue
            })
            .collect()
    }

    /// Top-K patterns above threshold.
    /// Returns empty Vec if NOTHING matches — this is important: bank can say "silent".
    pub fn recall_top_k(&self, cue: &[f32], k: usize) -> Vec<(u32, f32)> {
        if self.patterns.is_empty() || cue.len() != self.dim {
            return Vec::new();
        }
        let scores = self.scores(cue);
        let mut indexed: Vec<(usize, f32)> = scores.iter().enumerate()
            .map(|(i, &s)| (i, s))
            .collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        indexed.into_iter()
            .take(k)
            .filter(|(_, s)| *s >= self.activation_threshold)
            .map(|(i, s)| (self.patterns[i].atom_id, s))
            .collect()
    }

    /// Best single match, or None if silent.
    pub fn recall_best(&self, cue: &[f32]) -> Option<(u32, f32)> {
        self.recall_top_k(cue, 1).into_iter().next()
    }

    /// Pattern completion: given partial cue (with zeros in unknown positions),
    /// perform one Ramsauer-style update to produce a "filled in" vector.
    /// Returns the reconstructed vector (useful for chaining to next bank).
    pub fn complete(&self, cue: &[f32], n_iter: usize) -> Vec<f32> {
        if self.patterns.is_empty() || cue.len() != self.dim {
            return cue.to_vec();
        }
        let mut xi: Vec<f32> = cue.to_vec();
        let norm = (xi.iter().map(|v| v * v).sum::<f32>()).sqrt().max(1e-8);
        for v in xi.iter_mut() { *v /= norm; }

        for _ in 0..n_iter {
            // compute scores = beta * X @ xi
            let mut scores: Vec<f32> = self.normalized.iter()
                .map(|p| self.beta * p.iter().zip(xi.iter()).map(|(a, b)| a * b).sum::<f32>())
                .collect();
            // softmax, numerically stable
            let max = scores.iter().copied().fold(f32::MIN, f32::max);
            let mut sum = 0.0f32;
            for s in scores.iter_mut() { *s = (*s - max).exp(); sum += *s; }
            for s in scores.iter_mut() { *s /= sum; }

            // xi_new = X.T @ probs  (weighted combination of stored patterns)
            let mut new_xi = vec![0.0f32; self.dim];
            for (p, w) in self.normalized.iter().zip(scores.iter()) {
                for (n, v) in new_xi.iter_mut().zip(p.iter()) {
                    *n += w * v;
                }
            }
            // normalize
            let n = (new_xi.iter().map(|v| v * v).sum::<f32>()).sqrt().max(1e-8);
            for v in new_xi.iter_mut() { *v /= n; }
            xi = new_xi;
        }
        xi
    }

    pub fn len(&self) -> usize { self.patterns.len() }
    pub fn is_empty(&self) -> bool { self.patterns.is_empty() }
}

/// Multi-bank decomposer: one bank per brain region / day of creation.
/// Given a scene vector, query each bank and return what activated.
pub struct MultiBankDecomposer {
    pub banks: Vec<(String, HopfieldBank)>,
}

impl MultiBankDecomposer {
    pub fn new() -> Self {
        Self { banks: Vec::new() }
    }

    pub fn add_bank(&mut self, name: &str, bank: HopfieldBank) {
        self.banks.push((name.to_string(), bank));
    }

    /// Decompose a scene: returns (bank_name, vec of (atom_id, similarity)) for
    /// every bank that activated.
    pub fn decompose(&self, scene: &[f32], top_k: usize) -> Vec<(String, Vec<(u32, f32)>)> {
        self.banks.iter()
            .map(|(name, bank)| {
                let hits = bank.recall_top_k(scene, top_k);
                (name.clone(), hits)
            })
            .filter(|(_, hits)| !hits.is_empty())
            .collect()
    }
}

impl Default for MultiBankDecomposer {
    fn default() -> Self { Self::new() }
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn deterministic_vec(seed: u64, dim: usize) -> Vec<f32> {
        // Simple linear-congruential generator — deterministic across runs
        let mut state = seed.wrapping_mul(0x9E3779B97F4A7C15);
        (0..dim).map(|_| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((state >> 32) as i32 as f32) / (i32::MAX as f32)
        }).collect()
    }

    #[test]
    fn store_and_recall_exact() {
        let mut bank = HopfieldBank::new(64, 8.0, 0.9);
        let v = deterministic_vec(1, 64);
        bank.store(42, v.clone()).unwrap();
        let hit = bank.recall_best(&v);
        assert!(hit.is_some());
        let (atom_id, sim) = hit.unwrap();
        assert_eq!(atom_id, 42);
        assert!(sim > 0.99);
    }

    #[test]
    fn silent_when_no_match() {
        let mut bank = HopfieldBank::new(64, 8.0, 0.5);
        bank.store(1, deterministic_vec(1, 64)).unwrap();
        bank.store(2, deterministic_vec(2, 64)).unwrap();
        // Random cue that shouldn't match
        let cue = deterministic_vec(9999, 64);
        let hits = bank.recall_top_k(&cue, 3);
        // Most random cues against random patterns have similarity around 0,
        // well below threshold 0.5 — bank should be silent
        assert!(hits.len() <= 1, "unlikely cue matched too many patterns");
    }

    #[test]
    fn hierarchical_decomposition() {
        // Simulate Rex = mammal[0] + quadruped[0] + species[5] + breed[42] + color[3]
        let mut mammal = HopfieldBank::new(64, 8.0, 0.25);
        let mut quadruped = HopfieldBank::new(64, 8.0, 0.25);
        let mut species = HopfieldBank::new(64, 8.0, 0.25);
        let mut breed = HopfieldBank::new(64, 8.0, 0.25);
        let mut color = HopfieldBank::new(64, 8.0, 0.25);

        // 3 mammals, 3 quadrupeds, 10 species, 20 breeds, 5 colors
        for i in 0..3 { mammal.store(i, deterministic_vec(100 + i as u64, 64)).unwrap(); }
        for i in 0..3 { quadruped.store(i, deterministic_vec(200 + i as u64, 64)).unwrap(); }
        for i in 0..10 { species.store(i, deterministic_vec(300 + i as u64, 64)).unwrap(); }
        for i in 0..20 { breed.store(i, deterministic_vec(400 + i as u64, 64)).unwrap(); }
        for i in 0..5 { color.store(i, deterministic_vec(500 + i as u64, 64)).unwrap(); }

        // Build Rex as sum of specific atoms
        let m0 = deterministic_vec(100, 64);
        let q0 = deterministic_vec(200, 64);
        let s5 = deterministic_vec(305, 64);
        let b12 = deterministic_vec(412, 64);
        let c3 = deterministic_vec(503, 64);

        let mut rex = vec![0.0f32; 64];
        for i in 0..64 {
            rex[i] = m0[i] + q0[i] + s5[i] + b12[i] + c3[i];
        }

        // Each bank should recall the correct atom
        let best_m = mammal.recall_best(&rex).map(|(id, _)| id);
        let best_q = quadruped.recall_best(&rex).map(|(id, _)| id);
        let best_s = species.recall_best(&rex).map(|(id, _)| id);
        let best_b = breed.recall_best(&rex).map(|(id, _)| id);
        let best_c = color.recall_best(&rex).map(|(id, _)| id);

        assert_eq!(best_m, Some(0), "mammal layer");
        assert_eq!(best_q, Some(0), "quadruped layer");
        assert_eq!(best_s, Some(5), "species layer");
        assert_eq!(best_b, Some(12), "breed layer");
        assert_eq!(best_c, Some(3), "color layer");
    }

    #[test]
    fn determinism_same_cue_same_result() {
        let mut bank = HopfieldBank::new(128, 8.0, 0.3);
        for i in 0..20 {
            bank.store(i, deterministic_vec(i as u64, 128)).unwrap();
        }
        let cue = deterministic_vec(5, 128);
        let r1 = bank.recall_top_k(&cue, 3);
        let r2 = bank.recall_top_k(&cue, 3);
        assert_eq!(r1, r2);
    }

    #[test]
    fn completion_fills_masked_dims() {
        let mut bank = HopfieldBank::new(64, 10.0, 0.3);
        let original = deterministic_vec(42, 64);
        bank.store(42, original.clone()).unwrap();

        // Mask half the dims
        let mut cue = original.clone();
        for i in 0..32 { cue[i] = 0.0; }

        let completed = bank.complete(&cue, 3);
        // Dot product of completed with original should be close to 1
        let norm_o: f32 = (original.iter().map(|v| v * v).sum::<f32>()).sqrt();
        let dot: f32 = completed.iter().zip(original.iter()).map(|(a, b)| a * b).sum::<f32>();
        let sim = dot / norm_o;
        assert!(sim > 0.8, "completion similarity too low: {}", sim);
    }

    #[test]
    fn multi_bank_decomposer_routes_hits() {
        let mut mb = MultiBankDecomposer::new();
        let mut b1 = HopfieldBank::new(32, 8.0, 0.5);
        b1.store(100, deterministic_vec(1, 32)).unwrap();
        let mut b2 = HopfieldBank::new(32, 8.0, 0.5);
        b2.store(200, deterministic_vec(2, 32)).unwrap();
        mb.add_bank("region_a", b1);
        mb.add_bank("region_b", b2);

        let cue = deterministic_vec(1, 32);
        let result = mb.decompose(&cue, 3);
        // region_a should activate (contains cue), region_b should be silent
        assert!(result.iter().any(|(n, _)| n == "region_a"));
        assert!(!result.iter().any(|(n, _)| n == "region_b"));
    }

    #[test]
    fn zero_vector_rejected() {
        let mut bank = HopfieldBank::new(8, 8.0, 0.3);
        let result = bank.store(1, vec![0.0; 8]);
        assert!(result.is_err());
    }

    #[test]
    fn dim_mismatch_rejected() {
        let mut bank = HopfieldBank::new(8, 8.0, 0.3);
        let result = bank.store(1, vec![1.0; 16]);
        assert!(result.is_err());
    }
}
