//! Subagent Timeout Tracker Hook
//!
//! Event: SubagentStart
//! Records start time for duration tracking.
//! Injects max_turns recommendation to cap execution.
//! Emits telemetry to watchtower for session monitoring.

use nexcore_hooks::{HookOutput, SubagentTelemetry, exit_success_auto, read_input};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// Default timeout in seconds (5 minutes)
const DEFAULT_TIMEOUT_SECS: u64 = 300;

/// Estimated turns per minute for different agent types
const TURNS_PER_MINUTE: &[(&str, u64)] = &[
    ("Explore", 3),
    ("Plan", 2),
    ("general-purpose", 4),
    ("Bash", 6),
];

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    let agent_id = input.agent_id.as_deref().unwrap_or("unknown");
    let agent_type = input.agent_type.as_deref().unwrap_or("unknown");

    // Record start time
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let tracker_dir = dirs::home_dir()
        .map(|h| h.join(".claude/subagent_tracker"))
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/subagent_tracker"));

    if let Err(e) = fs::create_dir_all(&tracker_dir) {
        eprintln!("Warning: Could not create tracker dir: {}", e);
    }

    let tracker_file = tracker_dir.join(format!("{}.json", agent_id));
    let tracker_data = serde_json::json!({
        "agent_id": agent_id,
        "agent_type": agent_type,
        "start_time": start_time,
        "timeout_secs": DEFAULT_TIMEOUT_SECS
    });

    if let Err(e) = fs::write(&tracker_file, tracker_data.to_string()) {
        eprintln!("Warning: Could not write tracker file: {}", e);
    }

    // Emit telemetry for watchtower
    SubagentTelemetry::start(
        "subagent_timeout_tracker",
        agent_id,
        agent_type,
        Some(&input.session_id),
    )
    .emit();

    // Calculate recommended max_turns based on agent type and timeout
    let turns_per_min = TURNS_PER_MINUTE
        .iter()
        .find(|(t, _)| *t == agent_type)
        .map(|(_, tpm)| *tpm)
        .unwrap_or(3);

    let timeout_mins = DEFAULT_TIMEOUT_SECS / 60;
    let recommended_turns = turns_per_min * timeout_mins;

    HookOutput::allow()
        .with_system_message(format!(
            "⏱️ Timeout tracker: {} starts (5min cap ≈ {} turns)",
            agent_type, recommended_turns
        ))
        .emit();

    std::process::exit(0);
}
