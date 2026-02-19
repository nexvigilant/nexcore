//! nexcore-registry — SQLite registry for Claude Code skills, agents, metrics, and KPIs.
//!
//! Decomposes the monolithic `plugins.db` into component-specific databases.
//! This crate manages `skills.db`: the live registry of skills, agents,
//! invocation metrics, SMART goals, KPIs, and audit trail.
//!
//! # Architecture
//!
//! ```text
//! nexcore-mcp (MCP tool exposure)
//!     |
//! nexcore-registry (persistence + scanning + KPI computation)
//!     |
//! rusqlite + SQLite (bundled)
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use nexcore_registry::pool::RegistryPool;
//! use nexcore_registry::skills;
//!
//! let pool = RegistryPool::open_default().expect("open db");
//! pool.with_conn(|conn| {
//!     let all = skills::list_all(conn)?;
//!     println!("Found {} skills", all.len());
//!     Ok(())
//! }).expect("query");
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod agents;
pub mod assess;
pub mod audit;
pub mod error;
pub mod goals;
pub mod kpi;
pub mod metrics;
pub mod pool;
pub mod promote;
pub mod reports;
pub mod scanner;
pub mod schema;
pub mod skills;
pub mod tov;
pub mod writer;
