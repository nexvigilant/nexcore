//! Metrics Collector Hook
//!
//! Runs on PostToolUse to record execution metrics for all hooks.
//! Stores data in ~/.cache/nexcore-hooks/metrics.json
//!
//! This hook is passive - it always allows and just logs.

use nexcore_hooks::{exit_success_auto, metrics::MetricsRegistry, protocol::Decision, read_input};
use std::time::Instant;

fn main() {
    let start = Instant::now();

    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Get tool name from input
    let tool_name = input
        .tool_name
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    // Record this metric (we don't know the actual decision, so mark as Allow)
    // In practice, this captures "tool was used" events
    let duration_ms = start.elapsed().as_millis() as u64;

    // Atomic load-record-save
    if let Err(e) = MetricsRegistry::record_atomic(&tool_name, Decision::Allow, duration_ms) {
        // Silent failure - don't disrupt workflow
        eprintln!("metrics_collector: failed to record: {e}");
    }

    exit_success_auto();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_metrics_collector_compiles() {
        // Just verify the module compiles
        assert!(true);
    }
}
