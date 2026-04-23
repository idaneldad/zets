//! # Composition — graph-native generation from motif paths
//!
//! ## Idan's correction to an earlier mistake
//!
//! I initially claimed ZETS can't be generative — "Midjourney/Suno/Sora
//! are required for generation." Idan correctly pushed back:
//!
//! > "We already saved small parts and create a matrix of elements with
//! >  or without orchestration — this lets us generate something that
//! >  LOOKS LIKE the source, so we CAN be Generation, even at a simpler
//! >  level. Later we'll improve the model as it learns more."
//!
//! He's right. The cluster-tree + motifs + path-composition approach we
//! already have IS generative, at its own level. Humans with creative
//! minds generate wonderful things from internal memory — and that is
//! exactly what graph-walk + motif-assembly is doing.
//!
//! ## What this module does (and doesn't)
//!
//! **Does:**
//!   - Assemble motifs (short recurring patterns) into new sequences
//!   - Compose text/melodies/stories from learned graph atoms
//!   - Respect style constraints via motif selection
//!   - Cache successful compositions as new motifs (meta-learning)
//!
//! **Doesn't:**
//!   - Produce photorealistic images (needs diffusion)
//!   - Produce studio-quality music (needs Suno-class models)
//!   - Novel-length coherent fiction (needs long-range LLM)
//!
//! **The split:** ZETS handles the STRUCTURAL/COMPOSITIONAL layer
//! natively. External tools handle the DENSE/PERCEPTUAL layer (pixels,
//! waveforms, long-range language). Most ZETS output uses BOTH —
//! native structure + orchestrated realization.
//!
//! ## Architecture
//!
//! ```text
//! Request: "write a short story about X"
//!    ↓
//! 1. PLAN: Build narrative skeleton (graph walk)
//!          — 5-act structure, characters, beats
//!          — NATIVE
//!    ↓
//! 2. COMPOSE: For each beat, select motifs that fit
//!          — "introduction-of-conflict" motif bank
//!          — "character-reveal" motif bank
//!          — NATIVE
//!    ↓
//! 3. REALIZE: Generate prose text for each motif
//!          — If simple template → NATIVE (template fill)
//!          — If rich prose → ORCHESTRATED (LLM)
//!    ↓
//! 4. CACHE: If succeeds, add new motif to graph
//!          — Meta-learning: graph grows from its own output
//! ```

pub mod motif_bank;
pub mod plan;
pub mod weaver;

pub use motif_bank::{Motif, MotifBank, MotifKind};
pub use plan::{CompositionPlan, Step, StepKind};
pub use weaver::{ComposerError, Weaver};
