//! System Graph — ZETS's homoiconic core.
//!
//! The "how we think" counterpart to the data graph's "what we know".
//! Contains routes (procedures) expressed as bytecode that a minimal Rust
//! VM can execute. Learning methods, morphology dispatch, Hearst pattern
//! extraction — all live here as data, not code.
//!
//! Design principles (from Idan):
//!   1. Tiered loading (Hot/Warm/Cold/Archive) same as data graph.
//!   2. Routes can call other routes (bounded recursion) — "routes that
//!      drive routes".
//!   3. No Turing-completeness: bounded execution, bounded depth.
//!   4. Cross-platform safe: pure data, no dynamic code loading.
//!   5. Distributable: routes can ship as encrypted bundles per domain.
//!
//! Public surface:
//!   SystemGraph::new_bootstrap() — fresh instance with core routes.
//!   Vm::new(&graph.routes()).run(id, params, &mut host) — execute.

pub mod opcodes;
pub mod value;
pub mod routes;
pub mod vm;
pub mod bootstrap;
pub mod graph;
pub mod reasoning;

pub use opcodes::Opcode;
pub use value::Value;
pub use routes::{Route, RouteId, Tier};
pub use vm::{Host, Vm, VmError};
pub use bootstrap::{
    R_EXTRACT_HEARST_X_IS_A_Y, R_LEARN_FROM_DEFINITION, R_MORPH_LOOKUP_FALLBACK,
    all_bootstrap_routes,
};
pub use graph::{SystemGraph, SystemGraphStats};
pub use reasoning::{all_reasoning_routes, R_IS_ANCESTOR};
