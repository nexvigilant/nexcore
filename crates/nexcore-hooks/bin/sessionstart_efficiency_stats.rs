//! SessionStart hook: Display Token Efficiency Stats
//!
//! Event: SessionStart
//!
//! Shows current token efficiency metrics at session start:
//! - Global efficiency score (1.0 = baseline, >1 = better)
//! - TPA (Tokens Per Action) average
//! - Top insights for optimization
//!
//! Exit codes: Always 0 (informational only)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let metrics = load_metrics();
    if metrics.total_actions == 0 {
        std::process::exit(0);
    }

    print_efficiency_summary(&metrics);
    std::process::exit(0);
}

fn metrics_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".claude/metrics/token_efficiency.json")
}

fn load_metrics() -> TokenEfficiencyMetrics {
    let path = metrics_path();
    if !path.exists() {
        return TokenEfficiencyMetrics::default();
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn print_efficiency_summary(metrics: &TokenEfficiencyMetrics) {
    let eff_pct = (metrics.global_efficiency * 100.0) as u32;
    let eff_bar = efficiency_bar(metrics.global_efficiency);

    eprintln!();
    eprintln!("⚡ **TOKEN EFFICIENCY** ─────────────────────────────────────");
    eprintln!(
        "   Efficiency: {:.2}x [{eff_bar}] {}%",
        metrics.global_efficiency, eff_pct
    );
    eprintln!(
        "   Avg TPA: {:.0} │ Total: {} actions │ {} tokens",
        metrics.avg_tpa,
        metrics.total_actions,
        format_tokens(metrics.total_tokens)
    );
    eprintln!();

    print_top_tools(metrics);
    print_insights(metrics);

    eprintln!("───────────────────────────────────────────────────────────────");
    eprintln!();
}

fn efficiency_bar(eff: f64) -> String {
    let filled = ((eff.clamp(0.0, 2.0) / 2.0) * 10.0) as usize;
    let empty = 10 - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn format_tokens(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        format!("{}", tokens)
    }
}

fn print_top_tools(metrics: &TokenEfficiencyMetrics) {
    let mut tools: Vec<_> = metrics.by_tool.values().collect();
    tools.sort_by(|a, b| b.total_tokens.cmp(&a.total_tokens));

    if tools.is_empty() {
        return;
    }

    eprintln!("   Top tools by token usage:");
    for (i, tool) in tools.iter().take(3).enumerate() {
        let eff_icon = if tool.efficiency_score >= 1.0 {
            "✓"
        } else {
            "⚠"
        };
        eprintln!(
            "   {}. {} {}: {:.0} TPA ({:.1}x eff)",
            i + 1,
            eff_icon,
            tool.tool,
            tool.avg_tpa,
            tool.efficiency_score
        );
    }
    eprintln!();
}

fn print_insights(metrics: &TokenEfficiencyMetrics) {
    if metrics.insights.is_empty() {
        return;
    }

    eprintln!("   💡 Top insight:");
    if let Some(insight) = metrics.insights.first() {
        eprintln!("   • {}", insight.message);
        eprintln!("   → {}", insight.recommendation);
    }
    eprintln!();
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct TokenEfficiencyMetrics {
    #[serde(default)]
    version: String,
    last_updated: u64,
    total_actions: u64,
    total_tokens: u64,
    avg_tpa: f64,
    global_efficiency: f64,
    #[serde(default)]
    by_tool: HashMap<String, ToolStats>,
    #[serde(default)]
    insights: Vec<Insight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolStats {
    tool: String,
    action_count: u64,
    total_tokens: u64,
    avg_tpa: f64,
    baseline_tpa: f64,
    efficiency_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Insight {
    insight_type: InsightType,
    tool: Option<String>,
    message: String,
    recommendation: String,
    impact_score: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum InsightType {
    Inefficiency,
    HighVolume,
    Overuse,
}
