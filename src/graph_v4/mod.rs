//! graph_v4 — path graph עם phrases, paths, IDF, deterministic re-learning.
//!
//! מבוסס על v4_path_graph.py (ראה docs/50_working/). עברו 11/11 correctness
//! tests על 300 articles (95% top-1 accuracy, 100% path reconstruction).
//!
//! מודולים:
//!   types     — Atom / Edge / Graph / AtomKind / Relation
//!   tokenize  — tokenization אנגלית + עברית
//!   phrase    — n-gram extraction + greedy matching
//!   build     — builds Graph from corpus
//!   retrieve  — IDF-weighted path retrieval + answer()

pub mod types;
pub mod tokenize;
pub mod phrase;
pub mod build;
pub mod retrieve;
pub mod persist;
pub mod wiki_reader;
pub mod cleaner;
pub mod morphology;

pub use types::{Atom, AtomId, AtomKind, Edge, Graph, Relation, Stats};
pub use build::{build_graph, BuildConfig};
pub use persist::{save, load, PersistError};
pub use retrieve::{answer, compute_idf, phrases_from_graph, Answer, IdfTable};
