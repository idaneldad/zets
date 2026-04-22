//! Meta-Learning — tree 11: recursive feedback adapts cognitive mode weights.
//!
//! Idan's insight: after every interaction, the system should know WHICH
//! cognitive mode was most useful and adjust its priors accordingly.
//! A Bayesian Dirichlet update is the classic deterministic way to do this.
//!
//! The Dirichlet distribution is the conjugate prior of the Multinomial:
//!   - Parameters α = (α₁, α₂, α₃, α₄) for 4 cognitive modes
//!   - Each αᵢ starts at 1.0 (uniform prior)
//!   - On each observation, increment the α of the mode that succeeded
//!   - Posterior mean: αᵢ / Σα = the probability we'd choose mode i
//!
//! This is DETERMINISTIC: same sequence of observations → same α vector.
//! The "sampling" we do is also deterministic — hash-based pseudo-random.
//!
//! Four modes (matching cognitive_modes.rs):
//!   0 = Precision   (strong edges, factual)
//!   1 = Divergent   (weak edges, creative)
//!   2 = Gestalt     (pattern synthesis)
//!   3 = Narrative   (causal-temporal)


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CognitiveMode {
    Precision = 0,
    Divergent = 1,
    Gestalt = 2,
    Narrative = 3,
}

impl CognitiveMode {
    pub fn all() -> [CognitiveMode; 4] {
        [Self::Precision, Self::Divergent, Self::Gestalt, Self::Narrative]
    }
    pub fn as_index(self) -> usize { self as usize }
    pub fn from_index(i: usize) -> Option<Self> {
        match i {
            0 => Some(Self::Precision),
            1 => Some(Self::Divergent),
            2 => Some(Self::Gestalt),
            3 => Some(Self::Narrative),
            _ => None,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Precision => "precision",
            Self::Divergent => "divergent",
            Self::Gestalt => "gestalt",
            Self::Narrative => "narrative",
        }
    }
}

/// Dirichlet posterior over 4 cognitive modes.
/// Starts with uniform prior α = (1, 1, 1, 1).
/// Each successful use of a mode increments its α.
#[derive(Debug, Clone)]
pub struct ModeWeights {
    pub alpha: [f32; 4],
    pub total_observations: u64,
}

impl ModeWeights {
    /// Create with a uniform prior — all modes equally likely at start.
    pub fn uniform() -> Self {
        Self {
            alpha: [1.0, 1.0, 1.0, 1.0],
            total_observations: 0,
        }
    }

    /// Create with a custom prior (useful if you have domain knowledge).
    pub fn with_prior(alpha: [f32; 4]) -> Self {
        Self { alpha, total_observations: 0 }
    }

    /// Record an observation: mode i was used and produced this usefulness
    /// score in [0, 1]. The alpha for mode i is incremented by that score.
    ///
    /// usefulness = 0: no update (indifferent)
    /// usefulness = 1: full increment (+1 to this mode's count)
    /// Intermediate values give proportional credit.
    pub fn record(&mut self, mode: CognitiveMode, usefulness: f32) {
        let u = usefulness.clamp(0.0, 1.0);
        self.alpha[mode.as_index()] += u;
        self.total_observations += 1;
    }

    /// Posterior mean — the "smoothed" probability of each mode.
    /// This is what you'd use for a deterministic argmax choice.
    pub fn posterior_mean(&self) -> [f32; 4] {
        let sum: f32 = self.alpha.iter().sum();
        [
            self.alpha[0] / sum,
            self.alpha[1] / sum,
            self.alpha[2] / sum,
            self.alpha[3] / sum,
        ]
    }

    /// Mode with the highest posterior mean (argmax of the mean).
    pub fn best_mode(&self) -> CognitiveMode {
        let mean = self.posterior_mean();
        let mut best_idx = 0;
        let mut best_val = mean[0];
        for i in 1..4 {
            if mean[i] > best_val {
                best_val = mean[i];
                best_idx = i;
            }
        }
        CognitiveMode::from_index(best_idx).unwrap()
    }

    /// Deterministic pseudo-sample — given a query hash, produces a mode
    /// choice proportional to the posterior mean. Same hash + same weights
    /// → same mode, always.
    ///
    /// The "sampling" is not truly random; it's a hash-bucketed decision
    /// that approximates proportional selection when used across many
    /// distinct hashes.
    pub fn sample_by_hash(&self, query_hash: u64) -> CognitiveMode {
        let mean = self.posterior_mean();
        // Bucket the hash into a number 0..1 deterministically
        let bucket = ((query_hash >> 32) as u32 as f32) / (u32::MAX as f32);

        // Cumulative distribution function lookup
        let mut cum = 0.0f32;
        for i in 0..4 {
            cum += mean[i];
            if bucket < cum {
                return CognitiveMode::from_index(i).unwrap();
            }
        }
        // Fallback (rounding errors)
        CognitiveMode::Precision
    }

    /// How confident is the system in its best mode?
    /// Entropy of the posterior mean — low entropy = high confidence.
    /// Returns a value in [0, log2(4)] = [0, 2].
    pub fn entropy(&self) -> f32 {
        let mean = self.posterior_mean();
        -mean.iter()
            .filter(|&&p| p > 1e-8)
            .map(|&p| p * p.log2())
            .sum::<f32>()
    }

    /// Normalized confidence in [0, 1]: 1 = one mode dominates,
    /// 0 = perfectly uniform.
    pub fn confidence(&self) -> f32 {
        let max_entropy = 4f32.log2(); // log2(4) = 2
        1.0 - (self.entropy() / max_entropy).clamp(0.0, 1.0)
    }
}

impl Default for ModeWeights {
    fn default() -> Self { Self::uniform() }
}

// ────────────────────────────────────────────────────────────────
// Context-specific weights — meta-learning per query type
// ────────────────────────────────────────────────────────────────

/// A registry mapping QUERY TYPES to their own ModeWeights posterior.
/// Example query types: "factual", "creative", "explanatory", "emotional".
/// The system learns: factual queries → Precision wins; creative → Divergent wins.
#[derive(Debug, Clone)]
pub struct MetaLearner {
    pub per_context: std::collections::HashMap<String, ModeWeights>,
    pub global: ModeWeights,
}

impl MetaLearner {
    pub fn new() -> Self {
        Self {
            per_context: std::collections::HashMap::new(),
            global: ModeWeights::uniform(),
        }
    }

    /// Record success in a context.
    pub fn record(&mut self, context: &str, mode: CognitiveMode, usefulness: f32) {
        self.global.record(mode, usefulness);
        self.per_context.entry(context.to_string())
            .or_insert_with(ModeWeights::uniform)
            .record(mode, usefulness);
    }

    /// Get the best mode for a given context; falls back to global if unknown.
    pub fn best_for(&self, context: &str) -> CognitiveMode {
        self.per_context.get(context)
            .map(|w| w.best_mode())
            .unwrap_or_else(|| self.global.best_mode())
    }

    /// Get the full ModeWeights for a context (or None if unseen).
    pub fn weights_for(&self, context: &str) -> Option<&ModeWeights> {
        self.per_context.get(context)
    }
}

impl Default for MetaLearner {
    fn default() -> Self { Self::new() }
}

/// Hash a query string deterministically — used for sample_by_hash.
pub fn query_hash(text: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in text.as_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_prior_no_preference() {
        let w = ModeWeights::uniform();
        let mean = w.posterior_mean();
        for v in mean.iter() {
            assert!((v - 0.25).abs() < 0.001, "uniform should give 0.25, got {}", v);
        }
    }

    #[test]
    fn record_updates_alpha() {
        let mut w = ModeWeights::uniform();
        w.record(CognitiveMode::Divergent, 1.0);
        assert_eq!(w.alpha[CognitiveMode::Divergent.as_index()], 2.0);
        assert_eq!(w.alpha[CognitiveMode::Precision.as_index()], 1.0);
        assert_eq!(w.total_observations, 1);
    }

    #[test]
    fn many_successes_dominate() {
        let mut w = ModeWeights::uniform();
        for _ in 0..50 {
            w.record(CognitiveMode::Precision, 1.0);
        }
        assert_eq!(w.best_mode(), CognitiveMode::Precision);
        let mean = w.posterior_mean();
        // Precision should dominate by far
        assert!(mean[0] > 0.9, "precision should dominate, got {:?}", mean);
    }

    #[test]
    fn partial_credit_affects_alpha() {
        let mut w = ModeWeights::uniform();
        w.record(CognitiveMode::Gestalt, 0.5);
        assert!((w.alpha[CognitiveMode::Gestalt.as_index()] - 1.5).abs() < 0.001);
    }

    #[test]
    fn determinism_same_sequence_same_weights() {
        let mut w1 = ModeWeights::uniform();
        let mut w2 = ModeWeights::uniform();
        let seq = [
            (CognitiveMode::Precision, 1.0),
            (CognitiveMode::Divergent, 0.7),
            (CognitiveMode::Precision, 0.3),
            (CognitiveMode::Narrative, 1.0),
        ];
        for (m, u) in seq {
            w1.record(m, u);
            w2.record(m, u);
        }
        assert_eq!(w1.alpha, w2.alpha);
    }

    #[test]
    fn sample_by_hash_deterministic() {
        let mut w = ModeWeights::uniform();
        for _ in 0..20 {
            w.record(CognitiveMode::Divergent, 1.0);
        }
        // Same hash → same mode, always
        let hash = query_hash("how do I come up with a new idea?");
        let m1 = w.sample_by_hash(hash);
        let m2 = w.sample_by_hash(hash);
        assert_eq!(m1, m2);
    }

    #[test]
    fn entropy_reflects_uncertainty() {
        let uniform = ModeWeights::uniform();
        let e_uniform = uniform.entropy();
        // Uniform = max entropy (log2(4) = 2)
        assert!((e_uniform - 2.0).abs() < 0.01, "uniform entropy should be 2, got {}", e_uniform);

        let mut confident = ModeWeights::uniform();
        for _ in 0..100 {
            confident.record(CognitiveMode::Precision, 1.0);
        }
        let e_confident = confident.entropy();
        assert!(e_confident < 0.5, "confident entropy should be low, got {}", e_confident);
    }

    #[test]
    fn confidence_inversely_to_entropy() {
        let uniform = ModeWeights::uniform();
        assert!(uniform.confidence() < 0.01, "uniform should be low confidence");

        let mut sharp = ModeWeights::uniform();
        for _ in 0..100 {
            sharp.record(CognitiveMode::Narrative, 1.0);
        }
        assert!(sharp.confidence() > 0.7, "sharp should be high confidence, got {}",
                sharp.confidence());
    }

    #[test]
    fn context_specific_learning() {
        let mut ml = MetaLearner::new();
        // Factual queries: Precision wins
        for _ in 0..10 {
            ml.record("factual", CognitiveMode::Precision, 1.0);
        }
        // Creative queries: Divergent wins
        for _ in 0..10 {
            ml.record("creative", CognitiveMode::Divergent, 1.0);
        }

        assert_eq!(ml.best_for("factual"), CognitiveMode::Precision);
        assert_eq!(ml.best_for("creative"), CognitiveMode::Divergent);
    }

    #[test]
    fn unknown_context_falls_back_to_global() {
        let mut ml = MetaLearner::new();
        for _ in 0..10 {
            ml.record("any", CognitiveMode::Narrative, 1.0);
        }
        assert_eq!(ml.best_for("never_seen"), CognitiveMode::Narrative);
    }

    #[test]
    fn query_hash_deterministic() {
        assert_eq!(query_hash("foo"), query_hash("foo"));
        assert_ne!(query_hash("foo"), query_hash("bar"));
    }

    #[test]
    fn custom_prior_preserved() {
        let w = ModeWeights::with_prior([10.0, 1.0, 1.0, 1.0]);
        assert_eq!(w.best_mode(), CognitiveMode::Precision);
    }

    #[test]
    fn usefulness_clamped_to_01() {
        let mut w = ModeWeights::uniform();
        w.record(CognitiveMode::Gestalt, 5.0); // out of range
        // Should be clamped to 1.0
        assert_eq!(w.alpha[CognitiveMode::Gestalt.as_index()], 2.0);

        w.record(CognitiveMode::Gestalt, -3.0); // negative
        // Should be clamped to 0 — no change
        assert_eq!(w.alpha[CognitiveMode::Gestalt.as_index()], 2.0);
    }

    #[test]
    fn sample_by_hash_proportional_across_many_queries() {
        let mut w = ModeWeights::uniform();
        // Bias heavily toward Precision (90%)
        for _ in 0..90 {
            w.record(CognitiveMode::Precision, 1.0);
        }
        for _ in 0..10 {
            w.record(CognitiveMode::Divergent, 1.0);
        }
        // posterior: Precision ≈ 0.89, Divergent ≈ 0.10, others ≈ 0.01 each

        // Across 10k queries, ~89% should go to Precision
        let mut counts = [0; 4];
        for i in 0..10_000u64 {
            let m = w.sample_by_hash(query_hash(&format!("q{}", i)));
            counts[m.as_index()] += 1;
        }
        let precision_pct = counts[0] as f32 / 10_000.0;
        assert!(precision_pct > 0.80, "Precision should dominate samples, got {}", precision_pct);
    }
}
