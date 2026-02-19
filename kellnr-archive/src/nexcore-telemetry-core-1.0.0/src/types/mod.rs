//! Core telemetry types for source monitoring.
//!
//! Provides structured types for parsing and analyzing
//! external telemetry sources and their snapshots.

mod operation;
mod snapshot;
mod source;

pub use operation::*;
pub use snapshot::*;
pub use source::*;
