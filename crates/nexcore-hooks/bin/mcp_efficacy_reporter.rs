//! MCP Efficacy Reporter
//!
//! Generates CTVP Phase 2 efficacy reports for MCP tool adoption.
//!
//! Can be run standalone or as a hook (SessionStart/SessionEnd) to display metrics.
//!
//! # Usage
//! ```bash
//! # Generate all-time report
//! mcp_efficacy_reporter
//!
//! # Generate report for last 24 hours
//! mcp_efficacy_reporter 24
//! ```

use nexcore_hooks::mcp_efficacy::McpEfficacyRegistry;
use std::env;

fn main() {
    // Parse optional hours argument
    let hours: Option<f64> = env::args().nth(1).and_then(|s| s.parse().ok());

    // Load registry and generate report
    let registry = McpEfficacyRegistry::load();
    let report = registry.report(hours);

    println!("{report}");
}
