//! Decision Aggregator Hook
//!
//! PostToolUse hook that runs LAST to aggregate all hook results into a single report.
//! This provides the user with one unified message instead of up to 37 separate ones.
//!
//! Exit codes:
//! - 0: All hooks passed (or only warnings)
//! - 1: Warnings present - show aggregated warning report
//! - 2: Block - at least one hook blocked (should not normally reach here as
//!      PreToolUse hooks already blocked, but handles edge cases)
//!
//! ## Data Flow
//!
//! 1. Other PostToolUse hooks write findings to `HookResultRegistry`
//! 2. This hook runs LAST (ordering enforced by hook config)
//! 3. Loads `~/.cache/nexcore-hooks/results_{tool_use_id}.json`
//! 4. Aggregates all findings by severity (CRITICAL → INFO)
//! 5. Emits single compact report, then cleans up registry file
//!
//! ## Scope
//!
//! Only aggregates for Write/Edit tool operations. Other tools pass through.

use nexcore_hooks::{Decision, HookResultRegistry, exit_success_auto, exit_warn, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only aggregate for Edit/Write operations
    if !input.is_write_tool() {
        exit_success_auto();
    }

    // Get tool_use_id (fall back to session_id)
    let tool_use_id = input.tool_use_id_or_session();

    // Load the registry
    let registry = match HookResultRegistry::load(tool_use_id) {
        Ok(r) => r,
        Err(_) => {
            // No registry means no hooks ran or all passed silently
            exit_success_auto();
        }
    };

    // If no results, nothing to report
    if registry.results.is_empty() {
        cleanup_and_exit(tool_use_id);
    }

    // Aggregate results
    let aggregated = registry.aggregate();

    // Clean up the registry file
    let _ = HookResultRegistry::remove(tool_use_id);

    // Report based on decision
    match aggregated.decision {
        Decision::Allow => {
            // All passed - no output needed
            exit_success_auto();
        }
        Decision::Warn => {
            // Format and show warnings
            let report = format_compact_report(&aggregated);
            exit_warn(&report);
        }
        Decision::Block => {
            // Should not normally reach here (PreToolUse hooks block before PostToolUse)
            // But handle gracefully
            let report = aggregated.format_report();
            eprintln!("{}", report);
            exit_success_auto(); // PostToolUse can't block, just report
        }
    }
}

fn cleanup_and_exit(tool_use_id: &str) -> ! {
    let _ = HookResultRegistry::remove(tool_use_id);
    exit_success_auto();
}

/// Format a compact warning report (for terminal display)
fn format_compact_report(agg: &nexcore_hooks::AggregatedResult) -> String {
    let mut lines = Vec::new();

    lines.push(format!(
        "Hook Summary: {} findings in {}ms",
        agg.total_findings, agg.total_duration_ms
    ));

    // Group findings by severity
    let mut by_severity: std::collections::HashMap<String, Vec<&str>> =
        std::collections::HashMap::new();

    for group_result in agg.by_group.values() {
        for finding in &group_result.findings {
            let sev = match finding.severity {
                nexcore_hooks::Severity::Critical => "CRITICAL",
                nexcore_hooks::Severity::High => "HIGH",
                nexcore_hooks::Severity::Medium => "MEDIUM",
                nexcore_hooks::Severity::Low => "LOW",
                nexcore_hooks::Severity::Info => "INFO",
            };
            by_severity
                .entry(sev.to_string())
                .or_default()
                .push(&finding.message);
        }
    }

    // Output by severity (most severe first)
    for sev in ["CRITICAL", "HIGH", "MEDIUM", "LOW", "INFO"] {
        if let Some(messages) = by_severity.get(sev) {
            for msg in messages {
                lines.push(format!("  [{}] {}", sev, msg));
            }
        }
    }

    lines.join("\n")
}
