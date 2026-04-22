//! ZETS — deterministic symbolic knowledge graph engine.
//!
//! Public modules:
//!   piece_graph, piece_graph_loader — graph data structures & loading
//!   pack, mmap_core, mmap_lang      — binary pack format & lazy loading
//!   wal, crypto                      — persistence & security
//!   engine                           — unified facade (ZetsEngine)
//!   morphology                       — per-language morphology rules
//!   system_graph                     — homoiconic routes + bytecode VM

#![deny(unsafe_code)]

pub mod piece_graph;
pub mod piece_graph_loader;
pub mod pack;
pub mod mmap_core;
pub mod mmap_lang;
pub mod wal;
pub mod crypto;
pub mod engine;
pub mod morphology;
pub mod system_graph;
pub mod scopes;
pub mod testing_sandbox;
pub mod metacognition;
pub mod cognitive_modes;
pub mod atoms;
pub mod prototype;
pub mod relations;
pub mod appraisal;
