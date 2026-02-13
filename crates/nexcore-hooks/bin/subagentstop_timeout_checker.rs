//! Subagent Stop Timeout Checker Hook
//!
//! Event: SubagentStop
//! Checks if subagent exceeded 5-minute timeout.
//! Logs warning for performance analysis.
//! Emits telemetry with duration to watchtower.

use nexcore_hooks::{HookOutput, SubagentTelemetry, read_input};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// Default timeout in seconds (5 minutes)
const DEFAULT_TIMEOUT_SECS: u64 = 300;

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => {
            output_approve();
            return;
        }
    };

    let agent_id = input.agent_id.as_deref().unwrap_or("unknown");
    let agent_type = input.agent_type.as_deref().unwrap_or("unknown");

    let tracker_dir = dirs::home_dir()
        .map(|h| h.join(".claude/subagent_tracker"))
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/subagent_tracker"));

    let tracker_file = tracker_dir.join(format!("{}.json", agent_id));

    // Read start time from tracker
    let start_time = match fs::read_to_string(&tracker_file) {
        Ok(content) => serde_json::from_str::<serde_json::Value>(&content)
            .ok()
            .and_then(|v| v["start_time"].as_u64())
            .unwrap_or(0),
        Err(_) => 0,
    };

    // Calculate duration
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let duration_secs = now.saturating_sub(start_time);
    let duration_mins = duration_secs / 60;
    let duration_remaining_secs = duration_secs % 60;

    // Clean up tracker file
    if let Err(e) = fs::remove_file(&tracker_file) {
        eprintln!("Warning: Could not clean tracker file: {}", e);
    }

    // Emit telemetry for watchtower
    let exceeded = duration_secs > DEFAULT_TIMEOUT_SECS;
    let description = format!(
        "{}m{}s ({})",
        duration_mins,
        duration_remaining_secs,
        if exceeded { "exceeded" } else { "ok" }
    );
    SubagentTelemetry::stop(
        "subagentstop_timeout_checker",
        agent_id,
        agent_type,
        Some(&input.session_id),
        duration_secs * 1000, // Convert to ms
        Some(&description),
        exceeded,
    )
    .emit();

    // Check if exceeded timeout
    if exceeded {
        let overage_secs = duration_secs - DEFAULT_TIMEOUT_SECS;
        eprintln!(
            "⚠️ TIMEOUT EXCEEDED: {} ({}) ran {}m{}s ({}s over 5min limit)",
            agent_type,
            &agent_id[..8.min(agent_id.len())],
            duration_mins,
            duration_remaining_secs,
            overage_secs
        );

        // Log to performance file for analysis
        log_timeout_violation(agent_type, agent_id, duration_secs);

        HookOutput::warn(&format!(
            "Agent {} exceeded 5-minute timeout (ran {}m{}s)",
            agent_type, duration_mins, duration_remaining_secs
        ))
        .with_system_message(format!(
            "⏱️ SLOW AGENT: {} took {}m{}s (target: <5m). Consider narrower scope or max_turns cap.",
            agent_type, duration_mins, duration_remaining_secs
        ))
        .emit();
    } else {
        HookOutput::allow()
            .with_system_message(format!(
                "⏱️ {} completed in {}m{}s ✓",
                agent_type, duration_mins, duration_remaining_secs
            ))
            .emit();
    }

    std::process::exit(0);
}

fn output_approve() {
    println!(r#"{{"decision":"approve"}}"#);
    std::process::exit(0);
}

fn log_timeout_violation(agent_type: &str, agent_id: &str, duration_secs: u64) {
    let log_dir = dirs::home_dir()
        .map(|h| h.join(".claude/logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/claude_logs"));

    if let Err(e) = fs::create_dir_all(&log_dir) {
        eprintln!("Warning: Could not create log dir: {}", e);
        return;
    }

    let log_file = log_dir.join("subagent_timeouts.log");
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
    let entry = format!(
        "{} | {} | {} | {}s\n",
        timestamp,
        agent_type,
        &agent_id[..8.min(agent_id.len())],
        duration_secs
    );

    if let Err(e) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .and_then(|mut f| std::io::Write::write_all(&mut f, entry.as_bytes()))
    {
        eprintln!("Warning: Could not log timeout: {}", e);
    }
}
