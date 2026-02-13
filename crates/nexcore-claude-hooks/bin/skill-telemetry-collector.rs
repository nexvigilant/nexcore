//! Skill Telemetry Collector - PostToolUse Hook
//!
//! Captures skill invocation telemetry for analytics.
//! Triggers on PostToolUse when tool_name is "Skill".
//!
//! # Codex Compliance
//!
//! - **Tier**: T2-C (Cross-Domain Composite)
//! - **Grounding**: Types ground to T1 via T2-P newtypes.
//!
//! Hook Protocol:
//! - Input: JSON on stdin with tool_name, tool_input, tool_result, session_id
//! - Output: Empty JSON `{}` on stdout
//! - Exit: 0 = pass (telemetry is non-blocking)
//!
//! Persists to: ~/.claude/brain/telemetry/skill_invocations.jsonl
//!
//! # Cytokine Integration
//! - **Skill Invoked**: Emits IL-2 (growth/proliferation) via cytokine bridge

use chrono::{DateTime, Utc};
use nexcore_hook_lib::cytokine::emit_skill_invoked;
use nexcore_hook_lib::pass;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;

/// Telemetry output directory
const TELEMETRY_DIR: &str = "/home/matthew/.claude/brain/telemetry";
/// JSONL file for skill invocations
const TELEMETRY_FILE: &str = "skill_invocations.jsonl";
/// Maximum args length to store
const MAX_ARGS_LEN: usize = 500;

/// PostToolUse input structure from Claude Code.
///
/// # Tier: T2-C
/// Grounds to: T1(String) via Option.
#[derive(Debug, Deserialize)]
struct PostToolInput {
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    tool_name: Option<String>,
    #[serde(default)]
    tool_input: Option<SkillToolInput>,
    #[serde(default)]
    tool_result: Option<ToolResult>,
}

/// Skill-specific tool input.
///
/// # Tier: T2-C
/// Grounds to: T1(String) via Option.
#[derive(Debug, Deserialize)]
struct SkillToolInput {
    #[serde(default)]
    skill_name: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<serde_json::Value>,
    #[serde(default)]
    args: Option<serde_json::Value>,
    #[serde(default)]
    prompt: Option<String>,
}

/// Tool execution result.
///
/// # Tier: T2-C
/// Grounds to: T1(String, bool, u64) via Option.
#[derive(Debug, Deserialize)]
struct ToolResult {
    #[serde(default)]
    success: Option<bool>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    duration_ms: Option<u64>,
}

/// Skill invocation telemetry record.
///
/// # Tier: T2-C
/// Grounds to: T1 via serde serialization.
#[derive(Debug, Serialize)]
struct SkillInvocationRecord {
    timestamp: DateTime<Utc>,
    skill: String,
    args: String,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<u64>,
    session_id: String,
}

fn main() {
    let input = match read_hook_input() {
        Some(i) => i,
        None => pass(),
    };

    if !is_skill_tool(&input) {
        pass();
    }

    let record = match build_record(&input) {
        Some(r) => r,
        None => pass(),
    };

    if let Err(e) = append_telemetry(&record) {
        eprintln!("[skill-telemetry-collector] Warning: {e}");
    }

    // Emit IL-2 cytokine (growth/proliferation) for skill invocation
    emit_skill_invoked(&record.skill, Some(&record.session_id));

    pass();
}

/// Read and parse hook input from stdin.
fn read_hook_input() -> Option<PostToolInput> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).ok()?;
    serde_json::from_str(&buffer).ok()
}

/// Check if this is a Skill tool invocation.
fn is_skill_tool(input: &PostToolInput) -> bool {
    input.tool_name.as_deref() == Some("Skill")
}

/// Build telemetry record from input.
fn build_record(input: &PostToolInput) -> Option<SkillInvocationRecord> {
    let skill_input = input.tool_input.as_ref()?;
    let skill_name = extract_skill_name(skill_input);
    let args = extract_args(skill_input);
    let (success, duration_ms) = extract_result_info(&input.tool_result);
    let session_id = input
        .session_id
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    Some(SkillInvocationRecord {
        timestamp: Utc::now(),
        skill: skill_name,
        args: truncate_args(&args, MAX_ARGS_LEN),
        success,
        duration_ms,
        session_id,
    })
}

/// Extract skill name from tool input.
fn extract_skill_name(input: &SkillToolInput) -> String {
    input
        .skill_name
        .as_deref()
        .or(input.name.as_deref())
        .unwrap_or("unknown")
        .to_string()
}

/// Extract arguments as JSON string.
fn extract_args(input: &SkillToolInput) -> String {
    input
        .arguments
        .as_ref()
        .or(input.args.as_ref())
        .map(|v| v.to_string())
        .or_else(|| input.prompt.clone())
        .unwrap_or_else(|| "{}".to_string())
}

/// Extract success status and duration from result.
fn extract_result_info(result: &Option<ToolResult>) -> (bool, Option<u64>) {
    match result {
        Some(r) => {
            let success = r.success.unwrap_or_else(|| r.error.is_none());
            (success, r.duration_ms)
        }
        None => (true, None),
    }
}

/// Truncate args to avoid bloating telemetry file.
fn truncate_args(args: &str, max_len: usize) -> String {
    if args.len() <= max_len {
        args.to_string()
    } else {
        format!("{}...", &args[..max_len.saturating_sub(3)])
    }
}

/// Append a telemetry record to the JSONL file.
fn append_telemetry(record: &SkillInvocationRecord) -> Result<(), std::io::Error> {
    let dir = PathBuf::from(TELEMETRY_DIR);
    fs::create_dir_all(&dir)?;

    let file_path = dir.join(TELEMETRY_FILE);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)?;

    let json_line = serde_json::to_string(record)?;
    writeln!(file, "{json_line}")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_args_short() {
        let result = truncate_args("short", 100);
        assert_eq!(result, "short");
    }

    #[test]
    fn test_truncate_args_long() {
        let long_args = "a".repeat(600);
        let result = truncate_args(&long_args, 500);
        assert!(result.len() <= 500);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_is_skill_tool_true() {
        let input = PostToolInput {
            session_id: None,
            tool_name: Some("Skill".to_string()),
            tool_input: None,
            tool_result: None,
        };
        assert!(is_skill_tool(&input));
    }

    #[test]
    fn test_is_skill_tool_false() {
        let input = PostToolInput {
            session_id: None,
            tool_name: Some("Edit".to_string()),
            tool_input: None,
            tool_result: None,
        };
        assert!(!is_skill_tool(&input));
    }

    #[test]
    fn test_extract_skill_name_from_skill_name() {
        let input = SkillToolInput {
            skill_name: Some("test-skill".to_string()),
            name: None,
            arguments: None,
            args: None,
            prompt: None,
        };
        assert_eq!(extract_skill_name(&input), "test-skill");
    }

    #[test]
    fn test_extract_skill_name_from_name() {
        let input = SkillToolInput {
            skill_name: None,
            name: Some("fallback-name".to_string()),
            arguments: None,
            args: None,
            prompt: None,
        };
        assert_eq!(extract_skill_name(&input), "fallback-name");
    }

    #[test]
    fn test_extract_result_info_success() {
        let result = Some(ToolResult {
            success: Some(true),
            error: None,
            duration_ms: Some(150),
        });
        let (success, duration) = extract_result_info(&result);
        assert!(success);
        assert_eq!(duration, Some(150));
    }

    #[test]
    fn test_extract_result_info_failure() {
        let result = Some(ToolResult {
            success: Some(false),
            error: Some("failed".to_string()),
            duration_ms: None,
        });
        let (success, duration) = extract_result_info(&result);
        assert!(!success);
        assert_eq!(duration, None);
    }
}
