//! nexcore-db — Persistent SQLite storage for NexCore Brain working memory.
//!
//! Replaces the file-based JSON storage with a single SQLite database,
//! providing ACID transactions, WAL mode for concurrent reads, and
//! structured queries across sessions, artifacts, implicit knowledge,
//! and code tracking data.
//!
//! # Architecture
//!
//! ```text
//! nexcore-brain (business logic)
//!     ↓
//! nexcore-db (persistence)
//!     ↓
//! rusqlite + SQLite (bundled)
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use nexcore_db::pool::DbPool;
//! use nexcore_db::sessions;
//!
//! let pool = DbPool::open_default().expect("open db");
//! pool.with_conn(|conn| {
//!     let all = sessions::list_all(conn)?;
//!     println!("Found {} sessions", all.len());
//!     Ok(())
//! }).expect("query");
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![allow(
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::disallowed_types,
    clippy::iter_over_hash_type,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    clippy::shadow_unrelated,
    clippy::too_many_arguments,
    clippy::map_err_ignore,
    reason = "Migration and persistence layers prioritize backward-compatible schema ingestion over style-only lint constraints"
)]

pub mod artifacts;
pub mod decisions;
pub mod error;
pub mod implicit;
pub mod knowledge;
pub mod migrate;
pub mod pool;
pub mod schema;
pub mod sessions;
pub mod telemetry;
pub mod tracker;
