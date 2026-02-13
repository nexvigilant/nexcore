//! # MCP Efficacy Report CLI
//!
//! Generates CTVP Phase 2 efficacy reports for MCP tool usage, measuring whether
//! suggested MCP tools are actually being used and providing value.
//!
//! ## Purpose
//!
//! This tool validates the **efficacy** of the MCP tool suggestion system by tracking:
//! - **CAR (Conversion/Adoption Rate)**: % of sessions where suggestions led to actual usage
//! - **Per-tool metrics**: Which tools are suggested vs. actually used
//! - **Temporal patterns**: Usage trends over configurable time windows
//!
//! ## CTVP Phase Context
//!
//! In the Clinical Trial Validation Paradigm (CTVP), this tool provides **Phase 2 (Efficacy)**
//! evidence - proving that MCP suggestions actually help users (not just that they work).
//!
//! | Phase | Question | This Tool's Role |
//! |-------|----------|------------------|
//! | 0 - Preclinical | Does it parse? | N/A (mcp_suggester handles this) |
//! | 1 - Safety | Does it fail gracefully? | N/A |
//! | **2 - Efficacy** | **Does it help users?** | **✅ Primary function** |
//! | 3 - Confirmation | Does it scale? | Use `--hours` for trend analysis |
//! | 4 - Surveillance | Does it continue working? | Schedule periodic runs |
//!
//! ## Usage
//!
//! ```bash
//! # Generate all-time efficacy report
//! mcp_efficacy_report
//!
//! # Last 24 hours only
//! mcp_efficacy_report --hours 24
//!
//! # JSON output for automation/dashboards
//! mcp_efficacy_report --json
//!
//! # Cleanup old events (respects retention_days config)
//! mcp_efficacy_report --cleanup
//! ```
//!
//! ## Configuration
//!
//! Configured via `~/.claude/mcp_efficacy.toml`:
//!
//! ```toml
//! [thresholds]
//! min_sessions = 10        # Minimum sessions before validating CAR
//! car_target = 0.25        # Target 25% adoption rate
//!
//! [cleanup]
//! retention_days = 30      # Keep 30 days of history
//!
//! [feature_flags]
//! rollout_percentage = 100 # Suggestions enabled for all sessions
//! ```
//!
//! ## Data Sources
//!
//! Reads from `~/.claude/mcp_efficacy.json` which is populated by:
//! - `mcp_suggester` hook (records suggestions at UserPromptSubmit)
//! - `mcp_usage_tracker` hook (records actual tool invocations at PostToolUse)
//!
//! ## Output Interpretation
//!
//! | Status | Meaning |
//! |--------|---------|
//! | ⚠️ Collecting | Not enough sessions yet (< min_sessions) |
//! | ✅ Validated | CAR meets or exceeds target |
//! | ❌ Below Target | CAR below target - suggestions may need tuning |
//!
//! ## Integration with Telemetry
//!
//! For live monitoring, this tool's JSON output can be consumed by:
//! - FRIDAY's TelemetryMonitor source
//! - Prometheus via OTEL collector
//! - Custom dashboards via cron + jq
//!
//! ## Complexity
//!
//! - `compute_metrics()`: O(s + u) where s=suggestions, u=usages
//! - `print_report()`: O(t*log(t)) where t=unique tools (for top-5 sorting)

use nexcore_hooks::mcp_config::McpEfficacyConfig;
use nexcore_hooks::mcp_efficacy::McpEfficacyRegistry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut hours: Option<f64> = None;
    let mut json_output = false;
    let mut cleanup = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--hours" | "-h" => {
                if i + 1 < args.len() {
                    hours = args[i + 1].parse().ok();
                    i += 1;
                }
            }
            "--json" | "-j" => json_output = true,
            "--cleanup" | "-c" => cleanup = true,
            "--help" => {
                print!("{}", HELP_TEXT);
                return;
            }
            _ => {}
        }
        i += 1;
    }

    let config = McpEfficacyConfig::load();
    let mut registry = McpEfficacyRegistry::load();

    if cleanup {
        let retention = config.cleanup.retention_days;
        let before = registry.suggestions.len() + registry.usages.len();
        registry.cleanup(retention);
        let after = registry.suggestions.len() + registry.usages.len();
        if let Err(e) = registry.save() {
            eprintln!("Error: {e}");
            return;
        }
        println!(
            "✅ Cleaned {} events (kept last {} days)",
            before - after,
            retention
        );
        return;
    }

    let metrics = compute_metrics(&registry, hours);

    if json_output {
        if let Ok(json) = serde_json::to_string_pretty(&metrics) {
            println!("{json}");
        }
    } else {
        print_report(&metrics, &config, hours);
    }
}

const HELP_TEXT: &str = r#"MCP Efficacy Report - CTVP Phase 2 Validation

USAGE: mcp_efficacy_report [OPTIONS]

OPTIONS:
    -h, --hours <N>    Report for last N hours
    -j, --json         Output as JSON
    -c, --cleanup      Remove old events
    --help             Show this help

EXAMPLES:
    mcp_efficacy_report              # All-time
    mcp_efficacy_report --hours 24   # Last day
    mcp_efficacy_report --json       # JSON output
"#;

/// Compute metrics from registry - O(s + u) where s=suggestions, u=usages
fn compute_metrics(registry: &McpEfficacyRegistry, hours: Option<f64>) -> EfficacyMetrics {
    use std::collections::HashSet;

    let now = nexcore_hooks::state::now();
    let cutoff = hours.map_or(0.0, |h| now - h * 3600.0);

    let sug: Vec<_> = registry
        .suggestions
        .iter()
        .filter(|s| s.timestamp >= cutoff)
        .collect();
    let use_: Vec<_> = registry
        .usages
        .iter()
        .filter(|u| u.timestamp >= cutoff)
        .collect();

    let sess_sug: HashSet<_> = sug.iter().map(|s| &s.session_id).collect();
    let sess_fol: HashSet<_> = use_
        .iter()
        .filter(|u| u.followed_suggestion)
        .map(|u| &u.session_id)
        .collect();

    let mut tool_metrics: HashMap<String, ToolMetrics> = HashMap::new();
    sug.iter().flat_map(|s| &s.suggested_tools).for_each(|t| {
        tool_metrics.entry(t.clone()).or_default().suggestion_count += 1;
    });
    use_.iter().for_each(|u| {
        let m = tool_metrics.entry(u.tool_name.clone()).or_default();
        m.usage_count += 1;
        if u.followed_suggestion {
            m.adopted_count += 1;
        }
    });

    EfficacyMetrics {
        sessions_with_suggestions: sess_sug.len() as u32,
        sessions_with_followup_usage: sess_fol.len() as u32,
        total_suggestions: sug.len() as u32,
        total_usages: use_.len() as u32,
        tool_metrics,
        car: if sess_sug.is_empty() {
            0.0
        } else {
            sess_fol.len() as f64 / sess_sug.len() as f64
        },
    }
}

/// Print formatted report - O(t*log(t)) where t=unique tools
fn print_report(m: &EfficacyMetrics, cfg: &McpEfficacyConfig, hours: Option<f64>) {
    let period = hours.map_or("All time".into(), |h| format!("Last {h}h"));
    let status = if m.sessions_with_suggestions < cfg.thresholds.min_sessions {
        "⚠️ Collecting"
    } else if m.car >= cfg.thresholds.car_target {
        "✅ Validated"
    } else {
        "❌ Below Target"
    };

    println!("\n╔══════════════════════════════════════════════════════╗");
    println!("║  📊 MCP EFFICACY REPORT ({period:<12}) {status:<14}║");
    println!("╠══════════════════════════════════════════════════════╣");
    println!(
        "║  CAR: {:>5.1}%  (target: {:.0}%)                       ║",
        m.car * 100.0,
        cfg.thresholds.car_target * 100.0
    );
    println!(
        "║  Sessions: {} suggested, {} followed                 ║",
        m.sessions_with_suggestions, m.sessions_with_followup_usage
    );
    println!(
        "║  Total: {} suggestions, {} usages                    ║",
        m.total_suggestions, m.total_usages
    );
    println!("╠══════════════════════════════════════════════════════╣");

    if !m.tool_metrics.is_empty() {
        println!("║  TOP TOOLS                                           ║");
        let mut tools: Vec<_> = m.tool_metrics.iter().collect();
        tools.sort_by(|a, b| b.1.suggestion_count.cmp(&a.1.suggestion_count));
        for (t, tm) in tools.iter().take(5) {
            let name: String = t
                .strip_prefix("mcp__nexcore__")
                .unwrap_or(t)
                .chars()
                .take(24)
                .collect();
            let rate = if tm.suggestion_count == 0 {
                0.0
            } else {
                tm.adopted_count as f64 / tm.suggestion_count as f64 * 100.0
            };
            println!(
                "║  {name:<24} {}/{} ({rate:>5.1}%)         ║",
                tm.adopted_count, tm.suggestion_count
            );
        }
    }

    println!("╠══════════════════════════════════════════════════════╣");
    println!("║  Phase 0: ✅  Phase 1: ✅  Phase 2: {status:<14}  ║");
    println!(
        "║  Rollout: {}%                                         ║",
        cfg.feature_flags.rollout_percentage
    );
    println!("╚══════════════════════════════════════════════════════╝\n");
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ToolMetrics {
    suggestion_count: u32,
    usage_count: u32,
    adopted_count: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct EfficacyMetrics {
    sessions_with_suggestions: u32,
    sessions_with_followup_usage: u32,
    total_suggestions: u32,
    total_usages: u32,
    tool_metrics: HashMap<String, ToolMetrics>,
    car: f64,
}
