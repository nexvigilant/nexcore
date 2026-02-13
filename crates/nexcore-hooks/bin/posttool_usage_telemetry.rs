//! PostToolUse hook: Usage Telemetry
//!
//! Tracks skill and MCP tool invocations to identify high-value targets.
//! Data feeds into prioritization for improvement bonds.
//!
//! Captures:
//! - Tool name and frequency
//! - Skill invocations
//! - Success/failure rates
//!
//! ToV Alignment:
//! - Feedback Loop (ℱ): Usage data informs improvement priorities
//! - Evidence-Based: Decisions driven by actual usage patterns
//!
//! Exit codes:
//! - 0: Success (telemetry recorded)

use nexcore_hooks::{exit_success, read_input};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Default)]
struct UsageTelemetry {
    version: String,
    last_updated: u64,
    tools: HashMap<String, ToolUsage>,
    skills: HashMap<String, SkillUsage>,
    daily_totals: Vec<DailyTotal>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct ToolUsage {
    total_calls: u64,
    success_count: u64,
    failure_count: u64,
    last_used: u64,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct SkillUsage {
    total_invocations: u64,
    last_used: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DailyTotal {
    date: String,
    tool_calls: u64,
    skill_invocations: u64,
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success(),
    };

    let tool_name = input.tool_name.as_deref().unwrap_or("unknown");
    let now = current_timestamp();

    // Load telemetry
    let mut telemetry = load_telemetry();
    telemetry.last_updated = now;

    // Track tool usage
    let entry = telemetry.tools.entry(tool_name.to_string()).or_default();
    entry.total_calls += 1;
    entry.last_used = now;

    // Check if this was a Skill invocation
    if tool_name == "Skill" {
        if let Some(ref tool_input) = input.tool_input {
            if let Some(skill_name) = tool_input.get("skill").and_then(|v| v.as_str()) {
                let skill_entry = telemetry.skills.entry(skill_name.to_string()).or_default();
                skill_entry.total_invocations += 1;
                skill_entry.last_used = now;
            }
        }
    }

    // Update daily totals
    let today = format_date(now);
    if let Some(daily) = telemetry.daily_totals.last_mut() {
        if daily.date == today {
            daily.tool_calls += 1;
            if tool_name == "Skill" {
                daily.skill_invocations += 1;
            }
        } else {
            telemetry.daily_totals.push(DailyTotal {
                date: today,
                tool_calls: 1,
                skill_invocations: if tool_name == "Skill" { 1 } else { 0 },
            });
        }
    } else {
        telemetry.daily_totals.push(DailyTotal {
            date: today,
            tool_calls: 1,
            skill_invocations: if tool_name == "Skill" { 1 } else { 0 },
        });
    }

    // Keep only last 30 days
    if telemetry.daily_totals.len() > 30 {
        telemetry.daily_totals.remove(0);
    }

    save_telemetry(&telemetry);
    exit_success();
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn format_date(timestamp: u64) -> String {
    // Simple date formatting (YYYY-MM-DD)
    let days_since_epoch = timestamp / 86400;
    let years = 1970 + (days_since_epoch / 365);
    let day_of_year = days_since_epoch % 365;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;
    format!("{:04}-{:02}-{:02}", years, month.min(12), day.min(28))
}

fn telemetry_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".claude/metrics/usage_telemetry.json")
}

fn load_telemetry() -> UsageTelemetry {
    let path = telemetry_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_else(|| UsageTelemetry {
            version: "1.0.0".to_string(),
            ..Default::default()
        })
}

fn save_telemetry(telemetry: &UsageTelemetry) {
    let path = telemetry_path();
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Warning: Failed to create telemetry directory: {e}");
            return;
        }
    }
    if let Err(e) = fs::write(
        &path,
        serde_json::to_string_pretty(telemetry).unwrap_or_default(),
    ) {
        eprintln!("Warning: Failed to save telemetry: {e}");
    }
}
