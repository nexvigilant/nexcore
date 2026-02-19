//! Parser modules for telemetry data sources.
//!
//! Provides parsing capabilities for:
//! - Source sessions (JSON chat logs)
//! - Snapshots (brain artifacts with versioning)

mod snapshot_parser;
mod source_parser;

pub use snapshot_parser::{
    BrainSession, discover_brain_sessions, get_resolved_version, parse_snapshots,
    parse_snapshots_from_dir,
};
pub use source_parser::{
    DiscoveredSource, discover_sources, discover_sources_for_project, parse_all_sources,
    parse_source,
};
