//! Workflow intelligence — map workflows, identify gaps, improve live.
//!
//! This crate analyzes brain.db session data to produce:
//! - **Workflow maps** — DAGs of tool transitions across sessions
//! - **Gap analysis** — missing automations, error-prone tools, underused capabilities
//! - **Bottleneck detection** — high-failure tools, repeated reads, bash overuse
//! - **Live intel** — similar past workflows, suggested next tools, warnings
//!
//! # Data Sources
//!
//! All analysis reads from `brain.db` (read-only):
//! - `decision_audit` — per-tool-call events with risk level
//! - `tool_usage` — aggregate tool success/failure counts
//! - `autopsy_records` — per-session outcome metrics
//! - `skill_invocations` — skill usage frequency
//!
//! # Example
//!
//! ```ignore
//! use nexcore_workflow_intel::{db, analysis};
//!
//! let conn = db::open_brain_db(&db::default_brain_path())?;
//! let map = analysis::build_workflow_map(&conn, 30)?;
//! println!("{} sessions, {} events", map.sessions_analyzed, map.total_events);
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

pub mod analysis;
pub mod db;
pub mod error;
pub mod types;

pub use analysis::{analyze_gaps, build_workflow_map, find_bottlenecks, live_intel};
pub use db::{default_brain_path, open_brain_db};
pub use error::{Result, WorkflowError};
pub use types::*;
