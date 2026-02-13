//! SessionStart Preventative Guard Hook
//!
//! Reads `error_trends.json` and emits preventative actions to stderr.
//! Closes the Guardian homeostasis loop: logs → errors → trends → prevention.
//!
//! # Event
//! SessionStart
//!
//! # Input
//! `~/.claude/debug/error_trends.json`
//!
//! # Exit Codes
//! - 0: Always (advisory only, never blocks)

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
struct TrendReport {
    #[allow(dead_code)]
    generated: String,
    #[allow(dead_code)]
    sessions_analyzed: usize,
    total_errors: usize,
    errors_per_session: f64,
    top_categories: Vec<CategoryTrend>,
    new_errors: Vec<String>,
    recommendations: Vec<String>,
}

#[derive(Deserialize)]
struct CategoryTrend {
    category: String,
    count: usize,
    trend: String,
}

fn main() {
    let trends_path = debug_dir_path().join("error_trends.json");

    if !trends_path.exists() {
        std::process::exit(0);
    }

    let report = match load_report(&trends_path) {
        Some(r) => r,
        None => {
            std::process::exit(0);
        }
    };

    if report.recommendations.is_empty() && report.new_errors.is_empty() {
        std::process::exit(0);
    }

    emit_guard_actions(&report);
    std::process::exit(0);
}

fn debug_dir_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".claude").join("debug")
}

fn load_report(path: &PathBuf) -> Option<TrendReport> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn emit_guard_actions(report: &TrendReport) {
    eprintln!("🛡️ **DEBUG GUARDIAN** ─────────────────────────────────────");
    eprintln!(
        "   {} total errors | {:.2}/session",
        report.total_errors, report.errors_per_session
    );

    emit_category_actions(&report.top_categories);
    emit_new_category_alerts(&report.new_errors);
    emit_recommendations(&report.recommendations);

    eprintln!("───────────────────────────────────────────────────────────");
}

fn emit_category_actions(categories: &[CategoryTrend]) {
    for cat in categories {
        if cat.trend != "increasing" {
            continue;
        }
        let action = preventative_action(&cat.category);
        eprintln!(
            "   ⚠️ {} ↑ ({} hits): {}",
            cat.category, cat.count, action
        );
    }
}

fn emit_new_category_alerts(new_errors: &[String]) {
    for cat in new_errors {
        eprintln!("   🆕 New error type '{cat}' — flagged for human review");
    }
}

fn emit_recommendations(recommendations: &[String]) {
    if recommendations.is_empty() {
        return;
    }
    eprintln!("   📋 Recommendations:");
    for rec in recommendations {
        eprintln!("      • {rec}");
    }
}

fn preventative_action(category: &str) -> &'static str {
    match category {
        "mcp_disconnect" => "Rebuild: `cargo build --release -p nexcore-mcp`",
        "agent_failure" => "Check subagent configs in settings.json Task definitions",
        "timeout" => "Increase hook timeouts or reduce hook count",
        "permission_denied" => "Run `chmod` on affected paths or check sandbox settings",
        "executable_not_found" => "Verify binary paths in settings.json hook commands",
        "parse_failure" => "Validate SKILL.md frontmatter and JSON schemas",
        "stack_trace" => "Check Claude Code version for runtime bugs",
        "lock_contention" => "Benign — concurrent session lock collisions (no action)",
        _ => "Review error_index.jsonl for details",
    }
}
