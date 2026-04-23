//! # Wisdom Engines — content layer for personalized client outputs
//!
//! These engines were ported from cortex-v7 (now archived) on 23.04.26.
//! They provide DOMAIN KNOWLEDGE engines used to enrich user profiles
//! with astrology, numerology, gematria, Human Design, kabbalistic
//! commentary, and Torah references.
//!
//! ## Engines
//!
//! | Engine | Purpose | Lines |
//! |--------|---------|-------|
//! | astro | Astrology — natal chart, transits, Hebrew months | 991 |
//! | gematria | Hebrew letter values + 6 calc methods | 610 |
//! | human_design | BodyGraph from birth date+time | 255 |
//! | human, human_logic | Sefirot-based logic | 1386+978 |
//! | numerology | Western & Hebrew numerology | 340 |
//! | birur, birur_tuning | 10-dim confidence + 42 gates routing | 183+960 |
//! | mefaresh | Self-explaining queries before processing | 708 |
//! | torah | Tanakh text engine | 291 |
//! | gates231 | 231 Gates of Sefer Yetzirah | 356 |
//! | cross_tradition | Hebrew↔Greek↔Arabic mappings | 424 |
//! | installation_profile | Same brain different goals | 403 |
//!
//! ## Status
//!
//! Imported as-is. Each engine is self-contained (Cortex was monolithic),
//! and most have ZERO_DEPS = pure Rust. Some refer to Cortex internals
//! that don't exist in ZETS — those calls will need rewriting (commented
//! out for now via cfg-flag).
//!
//! ## Usage in ZETS
//!
//! These are CONTENT engines, not core graph engines. They are invoked
//! BY procedures (in procedure_atom) when a user asks for personalized
//! output (horoscope, name analysis, body graph). The engines compute
//! results from input data; they do not learn or modify the graph.
//!
//! Future direction: each engine becomes a procedure that ZETS can
//! discover via sense_key matching (e.g. "compute.astrology.natal").

#![allow(dead_code)]   // wisdom modules expose lots of helpers, not all used yet
#![allow(unused_variables)]

pub mod astro;
pub mod gematria;
pub mod human_design;
pub mod numerology;
pub mod torah;
pub mod gates231;
pub mod cross_tradition;
// engines that depend on cortex-v7 internals — disabled until adapted
// pub mod human;
// pub mod human_logic;
pub mod birur;
pub mod birur_tuning;
// pub mod mefaresh;
// pub mod installation_profile;
