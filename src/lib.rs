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
pub mod edge_extraction;
pub mod persona;
pub mod hopfield;
pub mod session;
pub mod spreading_activation;
pub mod scenario;
pub mod dreaming;
pub mod skills;
pub mod meta_learning;
pub mod smart_walk;
pub mod inference;
pub mod hash_registry;
pub mod http_server;
pub mod atom_persist;
pub mod state_persist;
pub mod bootstrap;
pub mod ingestion;
pub mod encrypted_installer;
pub mod planner;
pub mod benchmarks;
pub mod llm_adapter;
pub mod learning_layer;
pub mod ethics_core;
pub mod dialogue;
pub mod distillation;
pub mod explain;
pub mod verify;
pub mod corpus_acquisition;
pub mod gemini_http;
pub mod fold;
pub mod search;
pub mod bitflag_edge;
pub mod path_mining;
pub mod mtreemap;
pub mod sense_graph;
pub mod procedure_atom;
pub mod wisdom_engines;
pub mod reader;
pub mod personal_graph;
pub mod secrets;
pub mod conversation;
pub mod guard;
pub mod procedure_template;
pub mod benchmark;
pub mod composition;
pub mod connectors;
pub mod error_store;
pub mod llm;
pub mod cognitive;
pub mod brain_profile;
pub mod capability_runtime;
