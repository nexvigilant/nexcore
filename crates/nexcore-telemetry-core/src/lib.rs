//! Telemetry aggregation for external coding assistants.
//!
//! This crate provides structured parsing, analysis, and intelligence
//! report generation for telemetry data from external AI coding assistants.
//!
//! # Architecture
//!
//! The crate is organized into several modules:
//!
//! - [`types`] - Core data types for sources, operations, and snapshots
//! - [`error`] - Unified error handling
//! - [`parser`] - Telemetry log parsing utilities
//! - [`analysis`] - Cross-reference and pattern analysis
//! - [`intel`] - Intelligence report generation
//!
//! # Example
//!
//! ```ignore
//! use nexcore_telemetry_core::{Source, Snapshot, intel::generate_report};
//!
//! let sources: Vec<Source> = vec![]; // Load from telemetry files
//! let snapshots: Vec<Snapshot> = vec![]; // Load from brain artifacts
//!
//! let report = generate_report(&sources, &snapshots);
//! println!("Analyzed {} sources", report.sources_analyzed);
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![allow(missing_docs)]
#![warn(missing_docs)]
pub mod analysis;
pub mod error;
pub mod grounding;
pub mod intel;
pub mod parser;
pub mod types;

// Re-export core types for convenience
pub use error::{Result, TelemetryError};
pub use types::*;

// Re-export key intel functions
pub use intel::{
    ActivitySummary, FileAccessPattern, GovernanceAccess, IntelReport, generate_report,
};
