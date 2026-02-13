//! Hook Metrics Reporter
//!
//! SessionEnd hook that generates and displays hook metrics summary.
//! Shows execution counts, durations, and identifies hooks needing optimization.
//!
//! Exit codes:
//! - 0: Success (always)

use nexcore_hooks::metrics::MetricsRegistry;
use nexcore_hooks::{exit_success_auto, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only run on SessionEnd
    if input.hook_event_name != "SessionEnd" {
        exit_success_auto();
    }

    // Load metrics
    let registry = match MetricsRegistry::load() {
        Ok(r) => r,
        Err(_) => exit_success_auto(),
    };

    // Only report if we have meaningful data
    if registry.total_executions() < 10 {
        exit_success_auto();
    }

    // Generate compact summary for terminal
    let summary = generate_compact_summary(&registry);
    eprintln!("{}", summary);

    exit_success_auto();
}

fn generate_compact_summary(registry: &MetricsRegistry) -> String {
    let mut lines = Vec::new();

    lines.push(format!(
        "📊 Hook Stats: {} calls, {}ms total",
        registry.total_executions(),
        registry.total_duration_ms()
    ));

    // Top 3 slowest hooks
    let slow_hooks: Vec<_> = registry.top_by_duration().into_iter().take(3).collect();
    if !slow_hooks.is_empty() {
        let slow_list: Vec<_> = slow_hooks
            .iter()
            .map(|m| format!("{}({:.0}ms)", m.hook_name, m.avg_duration_ms()))
            .collect();
        lines.push(format!("   Slowest: {}", slow_list.join(", ")));
    }

    // High false positive warning
    let high_fp = registry.high_false_positive_hooks(20.0);
    if !high_fp.is_empty() {
        let fp_list: Vec<_> = high_fp
            .iter()
            .map(|m| format!("{}({:.0}%)", m.hook_name, m.false_positive_rate()))
            .collect();
        lines.push(format!("   ⚠️ High FP: {}", fp_list.join(", ")));
    }

    lines.join("\n")
}
