//! SMART Goal Failure Tracker Hook
//!
//! Event: PostToolUseFailure
//!
//! Tracks consecutive failures across all types (hook blocks, test failures,
//! explicit /fail commands). When threshold (3) is reached, triggers a goal
//! review by injecting context into the conversation.
//!
//! # Failure Types Tracked
//!
//! - **Hook Blocks**: Exit code 2 from any hook
//! - **Test Failures**: cargo test / npm test failures
//! - **Explicit Fails**: /fail skill invocations
//!
//! # Storage
//!
//! Failures tracked in ~/.claude/brain/goals/failure_tracker.json
//! Goals stored in ~/.claude/brain/goals/master_goals.md

use chrono::Utc;
use nexcore_hooks::{HookOutput, exit_success_auto, read_input};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const FAILURE_THRESHOLD: u32 = 3;

#[derive(Debug, Serialize, Deserialize)]
struct FailureTracker {
    version: u32,
    threshold: u32,
    consecutive_failures: u32,
    total_failures: u32,
    last_failure: Option<String>,
    failure_types: FailureTypes,
    failures: Vec<FailureRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FailureTypes {
    hook_block: u32,
    test_failure: u32,
    explicit_fail: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct FailureRecord {
    timestamp: String,
    failure_type: String,
    description: String,
}

impl Default for FailureTracker {
    fn default() -> Self {
        Self {
            version: 1,
            threshold: FAILURE_THRESHOLD,
            consecutive_failures: 0,
            total_failures: 0,
            last_failure: None,
            failure_types: FailureTypes {
                hook_block: 0,
                test_failure: 0,
                explicit_fail: 0,
            },
            failures: Vec::new(),
        }
    }
}

fn tracker_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".claude/brain/goals/failure_tracker.json")
}

fn goals_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".claude/brain/goals/master_goals.md")
}

fn load_tracker() -> FailureTracker {
    let path = tracker_path();
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        FailureTracker::default()
    }
}

fn save_tracker(tracker: &FailureTracker) -> Result<(), std::io::Error> {
    let path = tracker_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(tracker)?;
    fs::write(path, json)
}

fn load_goals() -> String {
    let path = goals_path();
    fs::read_to_string(path).unwrap_or_else(|_| String::from("No goals file found"))
}

#[derive(Debug)]
enum FailureType {
    HookBlock,
    TestFailure,
    ExplicitFail,
}

fn detect_failure_type(response: &str) -> Option<(FailureType, String)> {
    // Hook block detection (exit code 2)
    if response.contains("hook blocking error")
        || response.contains("Exit code: 2")
        || response.contains("BLOCK:")
    {
        return Some((FailureType::HookBlock, "Hook blocked operation".to_string()));
    }

    // Test failure detection
    // INVARIANT: These regex patterns are compile-time constants and valid
    if let Ok(re) = Regex::new(r"test .+ \.\.\. FAILED") {
        if re.is_match(response) {
            return Some((FailureType::TestFailure, "Test failure".to_string()));
        }
    }

    if response.contains("FAILED") && response.contains("cargo test") {
        return Some((FailureType::TestFailure, "Cargo test failure".to_string()));
    }

    if response.contains("npm ERR!") && response.contains("test") {
        return Some((FailureType::TestFailure, "NPM test failure".to_string()));
    }

    // Compilation failure
    if response.contains("error: could not compile") || response.contains("error[E") {
        return Some((FailureType::HookBlock, "Compilation failure".to_string()));
    }

    None
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Get the tool response
    let response = match &input.tool_response {
        Some(r) => r.to_string(),
        None => exit_success_auto(),
    };

    // Detect failure type
    let (failure_type, description) = match detect_failure_type(&response) {
        Some(ft) => ft,
        None => exit_success_auto(), // No failure detected
    };

    // Load and update tracker
    let mut tracker = load_tracker();
    tracker.consecutive_failures += 1;
    tracker.total_failures += 1;
    tracker.last_failure = Some(Utc::now().to_rfc3339());

    match failure_type {
        FailureType::HookBlock => tracker.failure_types.hook_block += 1,
        FailureType::TestFailure => tracker.failure_types.test_failure += 1,
        FailureType::ExplicitFail => tracker.failure_types.explicit_fail += 1,
    }

    // Record failure
    tracker.failures.push(FailureRecord {
        timestamp: Utc::now().to_rfc3339(),
        failure_type: format!("{failure_type:?}"),
        description: description.clone(),
    });

    // Keep only last 50 failures
    if tracker.failures.len() > 50 {
        tracker.failures = tracker.failures.split_off(tracker.failures.len() - 50);
    }

    // Save tracker
    if let Err(e) = save_tracker(&tracker) {
        eprintln!("Failed to save tracker: {e}");
    }

    // Check if threshold reached
    if tracker.consecutive_failures >= FAILURE_THRESHOLD {
        let goals = load_goals();

        // Build goal review prompt
        let context = format!(
            r#"
⚠️ SMART GOAL REVIEW TRIGGERED ────────────────────────────────
   Consecutive Failures: {} (threshold: {})
   Last Failure: {}

   📋 CURRENT MASTER GOALS:
{}

   🔄 ACTION REQUIRED:
   1. Review which goal was violated
   2. Identify the root cause of repeated failures
   3. Update goals if they need refinement
   4. Reset failure counter after review

   To update goals, edit: ~/.claude/brain/goals/master_goals.md
   To reset counter: Set consecutive_failures to 0 in failure_tracker.json
────────────────────────────────────────────────────────────────"#,
            tracker.consecutive_failures,
            FAILURE_THRESHOLD,
            description,
            goals.lines().take(40).collect::<Vec<_>>().join("\n")
        );

        // Inject context for Claude
        let output = HookOutput {
            hook_specific_output: Some(nexcore_hooks::HookSpecificOutput::post_tool_use_context(
                context,
            )),
            ..Default::default()
        };

        output.emit();
    } else {
        // Just report the failure count
        let context = format!(
            "📊 Failure #{} of {} threshold: {}",
            tracker.consecutive_failures, FAILURE_THRESHOLD, description
        );

        let output = HookOutput {
            hook_specific_output: Some(nexcore_hooks::HookSpecificOutput::post_tool_use_context(
                context,
            )),
            ..Default::default()
        };

        output.emit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_hook_block() {
        let response = "PostToolUse hook blocking error from command";
        let result = detect_failure_type(response);
        assert!(matches!(result, Some((FailureType::HookBlock, _))));
    }

    #[test]
    fn test_detect_test_failure() {
        let response = "test my_module::test_something ... FAILED";
        let result = detect_failure_type(response);
        assert!(matches!(result, Some((FailureType::TestFailure, _))));
    }

    #[test]
    fn test_detect_compilation_error() {
        let response = "error[E0382]: borrow of moved value";
        let result = detect_failure_type(response);
        assert!(matches!(result, Some((FailureType::HookBlock, _))));
    }

    #[test]
    fn test_no_failure() {
        let response = "Compiling project v0.1.0\nFinished in 1.23s";
        let result = detect_failure_type(response);
        assert!(result.is_none());
    }

    #[test]
    fn test_tracker_default() {
        let tracker = FailureTracker::default();
        assert_eq!(tracker.consecutive_failures, 0);
        assert_eq!(tracker.threshold, 3);
    }
}
