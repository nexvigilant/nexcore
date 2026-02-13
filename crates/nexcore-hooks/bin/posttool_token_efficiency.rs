//! PostToolUse hook: Token Efficiency Tracker
//!
//! Event: PostToolUse (all tools)
//!
//! Tracks token consumption per atomic action (tool call) to compute efficiency metrics.
//! Since Claude Code doesn't expose raw token counts, we estimate using:
//!
//! - **Input estimation**: Tool input JSON size × compression factor (≈4 chars/token)
//! - **Output estimation**: Tool result size × compression factor
//! - **Atomic action**: One tool invocation = one atomic action
//!
//! Metrics tracked:
//! - Tokens per atomic action (TPA)
//! - Rolling average TPA by tool type
//! - Efficiency score = baseline_TPA / actual_TPA (higher = better)
//! - Insight extraction for optimization opportunities
//!
//! Data stored: `~/.claude/metrics/token_efficiency.json`
//!
//! Exit codes: Always 0 (observability hook, never blocks)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const CHARS_PER_TOKEN: f64 = 4.0;

fn main() {
    let input = read_stdin().unwrap_or_default();
    let hook_input: HookInput = serde_json::from_str(&input).unwrap_or_default();

    let action = build_action_record(&hook_input, &input);
    let _ = update_metrics(&action);

    std::process::exit(0);
}

fn read_stdin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn estimate_tokens(text: &str) -> u64 {
    (text.len() as f64 / CHARS_PER_TOKEN).ceil() as u64
}

fn build_action_record(hook_input: &HookInput, raw_input: &str) -> AtomicActionRecord {
    let input_tokens = estimate_tokens(raw_input);
    let output_tokens = hook_input
        .tool_result
        .as_ref()
        .map(|r| estimate_tokens(r))
        .unwrap_or(0);

    AtomicActionRecord {
        timestamp: now(),
        tool: hook_input.tool_name.clone(),
        input_tokens,
        output_tokens,
        total_tokens: input_tokens + output_tokens,
        session_id: hook_input.session_id.clone(),
    }
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

fn save_metrics(metrics: &TokenEfficiencyMetrics) -> io::Result<()> {
    let path = metrics_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(metrics)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(&path, json)
}

fn update_metrics(action: &AtomicActionRecord) -> io::Result<()> {
    let mut metrics = load_metrics();

    update_global_stats(&mut metrics, action);
    update_tool_stats(&mut metrics, action);
    update_session_stats(&mut metrics, action);
    update_recent_actions(&mut metrics, action);
    metrics.insights = generate_insights(&metrics);

    save_metrics(&metrics)
}

fn update_global_stats(metrics: &mut TokenEfficiencyMetrics, action: &AtomicActionRecord) {
    metrics.total_actions += 1;
    metrics.total_tokens += action.total_tokens;
    metrics.last_updated = action.timestamp;
    metrics.avg_tpa = metrics.total_tokens as f64 / metrics.total_actions as f64;
    metrics.global_efficiency = calculate_global_efficiency(metrics);
}

fn update_tool_stats(metrics: &mut TokenEfficiencyMetrics, action: &AtomicActionRecord) {
    let baseline = get_baseline_tpa(&action.tool);
    let stats = metrics
        .by_tool
        .entry(action.tool.clone())
        .or_insert_with(|| ToolStats::new(action.tool.clone(), baseline));

    stats.action_count += 1;
    stats.total_tokens += action.total_tokens;
    stats.avg_tpa = stats.total_tokens as f64 / stats.action_count as f64;
    stats.efficiency_score = (stats.baseline_tpa / stats.avg_tpa).clamp(0.1, 10.0);
}

fn update_session_stats(metrics: &mut TokenEfficiencyMetrics, action: &AtomicActionRecord) {
    let Some(sid) = &action.session_id else {
        return;
    };
    let stats = metrics
        .by_session
        .entry(sid.clone())
        .or_insert_with(|| SessionStats {
            session_id: sid.clone(),
            action_count: 0,
            total_tokens: 0,
            started_at: action.timestamp,
        });
    stats.action_count += 1;
    stats.total_tokens += action.total_tokens;
}

fn update_recent_actions(metrics: &mut TokenEfficiencyMetrics, action: &AtomicActionRecord) {
    metrics.recent_actions.push(action.clone());
    if metrics.recent_actions.len() > 100 {
        metrics.recent_actions.remove(0);
    }
}

fn get_baseline_tpa(tool: &str) -> f64 {
    const BASELINES: &[(&str, f64)] = &[
        ("Read", 150.0),
        ("Grep", 100.0),
        ("Glob", 50.0),
        ("Write", 500.0),
        ("Edit", 300.0),
        ("Bash", 200.0),
        ("Task", 1000.0),
        ("Skill", 800.0),
        ("WebFetch", 2000.0),
        ("WebSearch", 1500.0),
        ("mcp__", 250.0),
    ];
    for (prefix, baseline) in BASELINES {
        if tool.starts_with(prefix) {
            return *baseline;
        }
    }
    300.0
}

fn calculate_global_efficiency(metrics: &TokenEfficiencyMetrics) -> f64 {
    if metrics.by_tool.is_empty() {
        return 1.0;
    }
    let weighted: f64 = metrics
        .by_tool
        .values()
        .map(|ts| ts.efficiency_score * ts.action_count as f64)
        .sum();
    let total: f64 = metrics
        .by_tool
        .values()
        .map(|ts| ts.action_count as f64)
        .sum();
    if total > 0.0 {
        (weighted / total).clamp(0.1, 10.0)
    } else {
        1.0
    }
}

fn generate_insights(metrics: &TokenEfficiencyMetrics) -> Vec<Insight> {
    let mut insights = Vec::new();
    add_inefficiency_insights(&mut insights, metrics);
    add_volume_insights(&mut insights, metrics);
    add_overuse_insights(&mut insights, metrics);
    insights.sort_by(|a, b| {
        b.impact_score
            .partial_cmp(&a.impact_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    insights.truncate(5);
    insights
}

fn add_inefficiency_insights(insights: &mut Vec<Insight>, metrics: &TokenEfficiencyMetrics) {
    for (tool, stats) in &metrics.by_tool {
        if stats.efficiency_score < 0.5 && stats.action_count >= 5 {
            let pct = ((1.0 / stats.efficiency_score - 1.0) * 100.0) as u32;
            insights.push(Insight {
                insight_type: InsightType::Inefficiency,
                tool: Some(tool.clone()),
                message: format!("{} uses {}% more tokens than baseline", tool, pct),
                recommendation: recommend_optimization(tool),
                impact_score: (1.0 - stats.efficiency_score) * stats.action_count as f64,
            });
        }
    }
}

fn add_volume_insights(insights: &mut Vec<Insight>, metrics: &TokenEfficiencyMetrics) {
    let top = metrics.by_tool.values().max_by_key(|s| s.total_tokens);
    let Some(top) = top else { return };
    if top.action_count < 10 {
        return;
    }
    let pct = (top.total_tokens as f64 / metrics.total_tokens as f64 * 100.0) as u32;
    insights.push(Insight {
        insight_type: InsightType::HighVolume,
        tool: Some(top.tool.clone()),
        message: format!("{} consumes {}% of total tokens", top.tool, pct),
        recommendation: format!("Optimize {} calls or batch where possible", top.tool),
        impact_score: top.total_tokens as f64,
    });
}

fn add_overuse_insights(insights: &mut Vec<Insight>, metrics: &TokenEfficiencyMetrics) {
    let Some(ts) = metrics.by_tool.get("Task") else {
        return;
    };
    let ratio = ts.action_count as f64 / metrics.total_actions as f64;
    if ratio <= 0.15 {
        return;
    }
    insights.push(Insight {
        insight_type: InsightType::Overuse,
        tool: Some("Task".to_string()),
        message: format!(
            "Subagent usage at {}% (target: <15%)",
            (ratio * 100.0) as u32
        ),
        recommendation: "Consider direct tool calls for simple operations".to_string(),
        impact_score: ts.total_tokens as f64 * 0.5,
    });
}

fn recommend_optimization(tool: &str) -> String {
    match tool {
        t if t.starts_with("Read") => "Use line limits for large files".to_string(),
        t if t.starts_with("Grep") => "Use files_with_matches mode".to_string(),
        t if t.starts_with("Write") => "Consider Edit for partial changes".to_string(),
        t if t.starts_with("Task") => "Use Explore for searches".to_string(),
        t if t.starts_with("WebFetch") => "Use targeted extraction prompts".to_string(),
        _ => format!("Review {} usage patterns", tool),
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct HookInput {
    #[serde(default)]
    tool_name: String,
    tool_input: Option<String>,
    tool_result: Option<String>,
    session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AtomicActionRecord {
    timestamp: u64,
    tool: String,
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    session_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct TokenEfficiencyMetrics {
    #[serde(default = "default_version")]
    version: String,
    last_updated: u64,
    total_actions: u64,
    total_tokens: u64,
    avg_tpa: f64,
    global_efficiency: f64,
    #[serde(default)]
    by_tool: HashMap<String, ToolStats>,
    #[serde(default)]
    by_session: HashMap<String, SessionStats>,
    #[serde(default)]
    recent_actions: Vec<AtomicActionRecord>,
    #[serde(default)]
    insights: Vec<Insight>,
}

fn default_version() -> String {
    "1.0.0".to_string()
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

impl ToolStats {
    fn new(tool: String, baseline: f64) -> Self {
        Self {
            tool,
            action_count: 0,
            total_tokens: 0,
            avg_tpa: 0.0,
            baseline_tpa: baseline,
            efficiency_score: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionStats {
    session_id: String,
    action_count: u64,
    total_tokens: u64,
    started_at: u64,
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
