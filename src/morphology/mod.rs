//! morphology — data-driven morphology engine.
//!
//! DESIGN (replacing the old per-language trait-impl approach):
//!
//! There is ONE engine — `MorphologyCore` — which contains rules (as data)
//! and unified logic (as methods). Each language is a FACTORY FUNCTION that
//! builds a `MorphologyCore` instance with its rules. Language families
//! (Semitic, Romance, Germanic, Isolating) are also factory functions that
//! return a pre-filled core, which per-language factories extend.
//!
//! This replaces class-inheritance with function-composition:
//!
//!   hebrew()  = semitic()  + hebrew-specific rules
//!   arabic()  = semitic()  + arabic-specific rules
//!   spanish() = romance()  + spanish-specific rules
//!   english() = germanic() + english-specific rules
//!   vietnamese() = isolating() (no rules needed — particle-based)
//!
//! Adding a new language is ~30 lines (just rule declarations).
//! The system can learn new rules at runtime via WAL records.
//!
//! No traits, no dyn dispatch, no vtables. Just data + unified functions.

pub mod rules;
pub mod core;
pub mod families;
pub mod languages;
pub mod registry;

pub use core::{Analysis, Feature, MorphologyCore, Typology};
pub use registry::MorphologyRegistry;
pub use rules::{IrregularForm, PrefixRule, SuffixRule};
