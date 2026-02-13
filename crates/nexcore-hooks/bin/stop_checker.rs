//! Stop Checker Hook (from claude-hooks library)
//!
//! Stop hook that verifies task completion before allowing Claude to stop.

use nexcore_hooks::{exit_success_auto, exit_warn, read_input};
use std::fs;

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Don't re-check if we're already in a stop hook
    if input.is_stop_hook_active() {
        exit_success_auto();
    }

    // Try to load transcript and check for issues
    if let Some(ref transcript_path) = input.transcript_path {
        if let Ok(content) = fs::read_to_string(transcript_path) {
            // Check for errors in transcript
            let error_count = content.matches("\"error\":").count();
            if error_count > 0 {
                // Check if last tool use had an error
                if content.contains("\"error\":") {
                    // Look for recent errors
                    let lines: Vec<&str> = content.lines().collect();
                    let last_1000: String = lines
                        .iter()
                        .rev()
                        .take(100)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("\n");

                    if last_1000.contains("\"error\":") && !last_1000.contains("error\": null") {
                        exit_warn(&format!(
                            "Session has {} error(s). Review before stopping.",
                            error_count
                        ));
                    }
                }
            }

            // Check if tests were mentioned but not run successfully
            if content.contains("cargo test") || content.contains("npm test") {
                let has_test_success = content.contains("test result: ok")
                    || content.contains("Tests:") && content.contains("passed");

                if !has_test_success && content.contains("FAILED") {
                    exit_warn("Tests may have failed. Please verify tests pass before stopping.");
                }
            }
        }
    }

    exit_success_auto();
}
