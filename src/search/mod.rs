//! Search — parallelized beam search with persona-adaptive parameters.
//!
//! Architecture per docs/working/20260423_neural_tree_7x7_design_V2.md:
//! - `strategy`: SearchStrategy — beam_width, max_depth, retry_waves
//! - `persona`: PersonaGraph — persona_id → strategy lookup
//! - `beam`: parallel walk worker (one per beam slot)
//! - `cancel`: cooperative cancellation via atomic flag
//!
//! **Why parallel?** Gemini + gpt-4o consultation + Python prototype
//! (py_testers/test_7x7_parallel_search_v1.py) measured 30-550× speedup
//! over sequential depth-8 DFS for graph sizes 50-3000 nodes.
//!
//! **Why 7×7 default?** Miller's 7±2 working memory + 6 degrees of
//! separation. Configurable per persona.
//!
//! **Cognitive-ergonomic labels, NOT diagnostic:** Per Gemini's explicit
//! critique, we use behavioral descriptors (precise, exploratory,
//! exhaustive, rapid_iteration, deep_dive, standard_7x7) instead of
//! diagnostic terms like ADHD/autistic.

pub mod strategy;
pub mod persona;
pub mod beam;
pub mod cancel;

pub use strategy::{SearchStrategy, StrategyLabel, default_strategies};
pub use persona::PersonaGraph;
pub use beam::{beam_search, BeamResult};
pub use cancel::CancelToken;

/// The maximum beam width we allow, regardless of strategy.
/// Prevents resource exhaustion from mis-configured personas.
pub const MAX_BEAM_WIDTH: usize = 64;

/// The maximum depth we allow, regardless of strategy.
/// Prevents infinite walks and matches cognitive-ergonomic upper bound.
pub const MAX_SEARCH_DEPTH: usize = 16;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_beam_width_reasonable() {
        assert!(MAX_BEAM_WIDTH >= 16);
        assert!(MAX_BEAM_WIDTH <= 256);
    }

    #[test]
    fn max_search_depth_reasonable() {
        assert!(MAX_SEARCH_DEPTH >= 8);
        assert!(MAX_SEARCH_DEPTH <= 32);
    }
}
