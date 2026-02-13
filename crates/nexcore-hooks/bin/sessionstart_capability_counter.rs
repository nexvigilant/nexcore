//! SessionStart hook: Capability Counter
//!
//! Tracks capability growth for exponential improvement monitoring.
//! Counts skills, hooks, MCP tools, subagents, and vocabulary shorthands.
//!
//! Provides:
//! - Current capability snapshot
//! - Weekly growth rate calculation
//! - Progress toward 1.5x target
//!
//! ToV Alignment:
//! - Feedback Loop (ℱ): Measures improvement velocity
//! - Accountability: Visible progress tracking
//!
//! Exit codes:
//! - 0: Success (metrics updated and displayed)

use nexcore_hooks::{exit_skip_session, exit_with_session_context, read_input};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const SECONDS_PER_WEEK: u64 = 604800;
const GROWTH_TARGET: f64 = 1.5;

#[derive(Debug, Serialize, Deserialize)]
struct CapabilityMetrics {
    version: String,
    created: String,
    history: Vec<CapabilitySnapshot>,
    current: CapabilitySnapshot,
    growth_rate: GrowthRate,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct CapabilitySnapshot {
    timestamp: u64,
    skills: u32,
    hooks: u32,
    mcp_tools: u32,
    subagents: u32,
    vocabulary_shorthands: u32,
    total: u32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct GrowthRate {
    weekly: f64,
    target: f64,
}

fn main() {
    let _input = match read_input() {
        Some(i) => i,
        None => exit_skip_session(),
    };

    // Count current capabilities
    let snapshot = count_capabilities();

    // Load and update metrics
    let mut metrics = load_metrics();
    let now = current_timestamp();

    // Check if we should record a new snapshot (once per day max)
    let should_record = metrics.history.is_empty() || (now - metrics.current.timestamp) > 86400;

    if should_record && metrics.current.total > 0 {
        // Archive current to history
        metrics.history.push(metrics.current.clone());
        // Keep only last 52 weeks
        if metrics.history.len() > 52 {
            metrics.history.remove(0);
        }
    }

    // Update current
    metrics.current = snapshot.clone();
    metrics.current.timestamp = now;

    // Calculate weekly growth rate
    metrics.growth_rate.weekly = calculate_growth_rate(&metrics.history, &metrics.current);
    metrics.growth_rate.target = GROWTH_TARGET;

    // Save metrics
    save_metrics(&metrics);

    // Build context output
    let context = format_context(&metrics);
    exit_with_session_context(&context);
}

fn count_capabilities() -> CapabilitySnapshot {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let home_path = PathBuf::from(&home);

    // Count skills
    let skills = count_dirs(&home_path.join(".claude/skills"));

    // Count hooks (compiled binaries)
    let hooks = count_files_with_ext(
        &home_path.join(".nexcore/crates/nexcore-hooks/target/release"),
        None, // No extension for binaries
    );

    // Count subagents
    let subagents = count_files_with_ext(&home_path.join(".config/agents"), Some("yaml"));

    // Count vocabulary shorthands
    let vocabulary_shorthands = count_vocabulary_shorthands(&home_path);

    // MCP tools (hardcoded for now - could query nexcore)
    let mcp_tools = 112; // Known nexcore MCP tool count

    let total = skills + hooks + subagents + vocabulary_shorthands + mcp_tools;

    CapabilitySnapshot {
        timestamp: 0,
        skills,
        hooks,
        mcp_tools,
        subagents,
        vocabulary_shorthands,
        total,
    }
}

fn count_dirs(path: &PathBuf) -> u32 {
    fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .count() as u32
        })
        .unwrap_or(0)
}

fn count_files_with_ext(path: &PathBuf, ext: Option<&str>) -> u32 {
    fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let p = e.path();
                    if !p.is_file() {
                        return false;
                    }
                    match ext {
                        Some(expected) => p.extension().is_some_and(|x| x == expected),
                        None => p.extension().is_none(), // Binary files have no extension
                    }
                })
                .count() as u32
        })
        .unwrap_or(0)
}

fn count_vocabulary_shorthands(home: &PathBuf) -> u32 {
    let vocab_path = home.join(".claude/implicit/vocabulary.json");
    fs::read_to_string(&vocab_path)
        .ok()
        .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
        .and_then(|v| v.get("shorthands")?.as_object().map(|o| o.len() as u32))
        .unwrap_or(0)
}

fn calculate_growth_rate(history: &[CapabilitySnapshot], current: &CapabilitySnapshot) -> f64 {
    // Find snapshot from ~1 week ago
    let now = current.timestamp;
    let week_ago = now.saturating_sub(SECONDS_PER_WEEK);

    let baseline = history
        .iter()
        .filter(|s| s.timestamp <= week_ago)
        .last()
        .or(history.first());

    match baseline {
        Some(base) if base.total > 0 => current.total as f64 / base.total as f64,
        _ => 1.0, // No baseline = no growth measured yet
    }
}

fn format_context(metrics: &CapabilityMetrics) -> String {
    let c = &metrics.current;
    let g = &metrics.growth_rate;

    // Progress bar toward target
    let progress = (g.weekly / g.target * 100.0).min(100.0);
    let filled = (progress / 10.0) as usize;
    let bar: String = "█".repeat(filled) + &"░".repeat(10 - filled);

    // Status emoji
    let status = if g.weekly >= g.target {
        "🚀"
    } else if g.weekly >= 1.2 {
        "📈"
    } else if g.weekly >= 1.0 {
        "➡️"
    } else {
        "📉"
    };

    format!(
        "📊 **CAPABILITY GROWTH** ─────────────────────────────────\n\
         {} Growth: {:.2}x/week [{}] Target: {:.1}x\n\n\
         │ Skills: {:>3} │ Hooks: {:>3} │ MCP: {:>3} │\n\
         │ Agents: {:>3} │ Vocab: {:>3} │ **Total: {:>3}** │\n\
         ───────────────────────────────────────────────────────────\n",
        status,
        g.weekly,
        bar,
        g.target,
        c.skills,
        c.hooks,
        c.mcp_tools,
        c.subagents,
        c.vocabulary_shorthands,
        c.total,
    )
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn metrics_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".claude/metrics/capabilities.json")
}

fn load_metrics() -> CapabilityMetrics {
    let path = metrics_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_else(|| CapabilityMetrics {
            version: "1.0.0".to_string(),
            created: "2026-02-01".to_string(),
            history: Vec::new(),
            current: CapabilitySnapshot::default(),
            growth_rate: GrowthRate {
                weekly: 1.0,
                target: GROWTH_TARGET,
            },
        })
}

fn save_metrics(metrics: &CapabilityMetrics) {
    let path = metrics_path();
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Warning: Failed to create metrics directory: {e}");
            return;
        }
    }
    if let Err(e) = fs::write(
        &path,
        serde_json::to_string_pretty(metrics).unwrap_or_default(),
    ) {
        eprintln!("Warning: Failed to save metrics: {e}");
    }
}
