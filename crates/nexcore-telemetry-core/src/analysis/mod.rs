//! Analysis module for telemetry data.
//!
//! Provides tools for analyzing file access patterns, cross-referencing
//! with governance modules, and comparing artifact versions.

mod crossref;
mod diff;
mod file_tracker;

pub use crossref::*;
pub use diff::*;
pub use file_tracker::*;
