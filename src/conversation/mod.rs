//! # ConversationStore — persistent memory of who said what when
//!
//! Every message in/out of ZETS is recorded here. The store is:
//!   - **append-only** — never mutate past entries
//!   - **per-source** — each Source has its own conversation timeline
//!   - **session-aware** — messages group into sessions (bounded conversations)
//!   - **queryable** — "show me the last 10 messages from client X"
//!
//! This is what the Reader reads from: `HistoryEntry[]` passed to
//! `Reader::read()` should come from here.
//!
//! ## Not yet implemented
//!
//! - Cold storage migration (old sessions → archive file)
//! - Encryption of message content (may be sensitive)
//! - Cross-source search ("find conversations mentioning X")
//!
//! ## Storage model
//!
//! In-memory: `HashMap<Source-identifier, Vec<HistoryEntry>>`
//! Persistence: append-only JSONL files, one per source identifier.
//! Path convention: `data/conversations/{source_id_safe}.jsonl`

pub mod session;
pub mod store;

pub use session::{Session, SessionBoundary};
pub use store::{ConversationStore, StoreError};
