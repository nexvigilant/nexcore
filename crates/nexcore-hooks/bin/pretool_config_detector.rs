//! Config Detector Hook (HOOK-M5-50)
//!
//! PreToolUse hook that detects hardcoded configuration values.
//! Warns on Medium severity, blocks on High severity issues.
//!
//! Exit codes:
//! - 0: Allow (no issues found)
//! - 1: Warn (medium/low severity findings)
//! - 2: Block (high severity findings)

use nexcore_hooks::deployment::config::{detect_hardcoded_config, format_config_issues};
use nexcore_hooks::{exit_block, exit_success_auto, exit_warn, is_rust_file, read_input};

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only scan on Write tool (Edit doesn't have full content)
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if tool_name != "Write" {
        exit_success_auto();
    }

    // Get file path
    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    // Only scan Rust files and common config files
    let should_scan = is_rust_file(file_path)
        || file_path.ends_with(".toml")
        || file_path.ends_with(".yaml")
        || file_path.ends_with(".yml")
        || file_path.ends_with(".json")
        || file_path.ends_with(".env");

    if !should_scan {
        exit_success_auto();
    }

    // Get content from tool input
    let content = match input.get_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    // Detect hardcoded config
    let issues = detect_hardcoded_config(file_path, content);

    if issues.is_empty() {
        exit_success_auto();
    }

    // Check severity levels
    let has_high_severity = issues.iter().any(|i| i.severity.should_block());

    if has_high_severity {
        let report = format_config_issues(file_path, &issues);
        exit_block(&report);
    } else {
        // Medium/low findings - warn but allow
        let count = issues.len();
        let msg = format!(
            "⚠️ Found {} hardcoded config value(s) in {}\n\
             Consider using environment variables for better configurability.",
            count, file_path
        );
        exit_warn(&msg);
    }
}
