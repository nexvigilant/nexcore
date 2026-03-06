//! Pre-built vertical configs for the WebMCP Hub PV land grab.
//!
//! Each module exposes a `config()` function returning a ready-to-publish
//! `StationConfig`. Call `StationBuilder::to_moltbrowser_create()` on the
//! builder, or use the registry to batch-publish all verticals.

pub mod faers;
pub mod fda;
