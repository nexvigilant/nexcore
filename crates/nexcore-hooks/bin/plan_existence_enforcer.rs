//! Plan Existence Enforcer Hook
//!
//! Stop hook that warns when non-trivial sessions end without a plan file.
//!
//! # Event
//! Stop
//!
//! # Purpose
//! Ensures sessions leave documented plans behind for continuity.
//!
//! # Logic
//! 1. Check if session had >3 tool uses (non-trivial)
//! 2. Check if plan file exists in ~/.claude/plans/
//! 3. If no plan AND non-trivial session: WARN with guidance
//!
//! # Exit Codes
//! - 0: Always (warning only, non-blocking)

use nexcore_hooks::paths::plans_dir;
use nexcore_hooks::state::SessionState;
use std::fs;
use std::path::PathBuf;

fn find_recent_plan(session_start: f64) -> Option<PathBuf> {
    let dir = plans_dir();
    if !dir.exists() {
        return None;
    }

    // Look for any plan file modified during this session
    let plans: Vec<_> = fs::read_dir(&dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .filter(|e| {
            // Check if modified after session start
            e.metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs_f64() >= session_start)
                .unwrap_or(false)
        })
        .collect();

    plans.first().map(|e| e.path())
}

fn count_tool_uses_from_transcript(transcript_path: &Option<String>) -> usize {
    // Try to count tool uses from transcript
    let path = match transcript_path {
        Some(p) => p,
        None => return 0,
    };

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return 0,
    };

    // Count lines that indicate tool use (rough heuristic)
    // JSONL format - each line is a message, tool uses have "tool_use" type
    content
        .lines()
        .filter(|line| line.contains("\"tool_use\"") || line.contains("\"tool_result\""))
        .count()
        / 2 // Divide by 2 since each tool has use + result
}

fn main() {
    let state = SessionState::load();

    // Read stdin for hook input (Stop hooks receive input too)
    let mut buffer = String::new();
    let _ = std::io::Read::read_to_string(&mut std::io::stdin(), &mut buffer);
    let input: serde_json::Value = serde_json::from_str(&buffer).unwrap_or_default();

    let transcript_path = input
        .get("transcript_path")
        .and_then(|v| v.as_str())
        .map(String::from);

    // Count tool uses to determine if session was non-trivial
    let tool_count = count_tool_uses_from_transcript(&transcript_path);

    // Also use lines/files from state as a proxy
    let is_non_trivial =
        tool_count > 3 || state.lines_since_verification > 50 || state.files_since_verification > 2;

    // Check for recent plan
    let recent_plan = find_recent_plan(state.session_start);

    let (decision, message) = if !is_non_trivial {
        // Trivial session - no plan needed
        (
            "approve",
            "📝 Trivial session - no plan required".to_string(),
        )
    } else if recent_plan.is_some() {
        // Plan exists - good!
        let plan_name = recent_plan
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        ("approve", format!("📋 Plan exists: {}", plan_name))
    } else {
        // Non-trivial session without plan - warn!
        (
            "approve", // Don't block, just warn strongly
            format!(
                "⚠️ NON-TRIVIAL SESSION WITHOUT PLAN\n\
                 \n\
                 This session had {} tool uses but no plan file.\n\
                 \n\
                 Consider creating a plan before ending:\n\
                 1. Write to ~/.claude/plans/<task_name>.md\n\
                 2. Include: Objective, Implementation, Verification sections\n\
                 3. Document blockers and next steps\n\
                 \n\
                 Plans help the next session continue your work.",
                tool_count
            ),
        )
    };

    // Ensure plans directory exists
    let _ = fs::create_dir_all(plans_dir());

    let output = serde_json::json!({
        "continue": true,
        "decision": decision,
        "stopReason": format!("Plan check: {} (tool uses: {})",
            if recent_plan.is_some() { "Plan found" } else { "No plan" },
            tool_count),
        "systemMessage": message
    });
    println!("{}", output);
}
