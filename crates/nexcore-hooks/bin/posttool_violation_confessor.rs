//! Violation Confessor Hook
//!
//! Event: PostToolUse:Edit|Write
//! Tier: dev, review, deploy
//! Exit: 0 (allow) | 2 (block — confession required)
//!
//! Monitors for Primitive Codex violations in written/edited files and enforces
//! the structured confession template. When violation indicators (unwrap, panic,
//! naked primitives, etc.) are detected without a corresponding VIOLATION CONFESSION
//! block, the hook blocks further writes until the agent confesses.
//!
//! Confession format requires: Commandment Violated, Code in Question,
//! Nature of Violation, Root Cause, Corrected Code, and Prevention.
//!
//! Implements Commandment XII (ENFORCE) — the compiler (and hooks) are the
//! arbiters of truth. Violations must be acknowledged, not silenced.

use nexcore_hooks::{HookTelemetry, exit_block, exit_ok, get_content, get_file_path, read_input};
use serde_json::json;

/// The 12 Commandments of the Primitive Codex
const COMMANDMENTS: &[(&str, &str)] = &[
    ("1", "QUANTIFY"),
    ("2", "CLASSIFY"),
    ("3", "GROUND"),
    ("4", "WRAP"),
    ("5", "COMPARE"),
    ("6", "MATCH"),
    ("7", "TYPE THE TEST"),
    ("8", "VERSION"),
    ("9", "MEASURE"),
    ("10", "ACKNOWLEDGE"),
    ("11", "CORRECT"),
    ("12", "ENFORCE"),
];

/// Violation indicators in code/comments
const VIOLATION_INDICATORS: &[&str] = &[
    "violat",
    "breaks commandment",
    "codex violation",
    "primitive violation",
    "TODO: fix violation",
    "FIXME: violation",
    "unwrap()", // Commandment 12 violation
    "expect()", // Commandment 12 violation
    "panic!",   // Commandment 12 violation
];

/// Check if content contains violation indicators
fn has_violation_indicators(content: &str) -> Vec<String> {
    let lower = content.to_lowercase();
    let mut found = Vec::new();

    for indicator in VIOLATION_INDICATORS {
        if lower.contains(&indicator.to_lowercase()) {
            found.push(indicator.to_string());
        }
    }

    // Check for naked primitives (Commandment 4: WRAP)
    if content.contains("u8")
        || content.contains("u16")
        || content.contains("u32")
        || content.contains("u64")
    {
        // Only flag if it's a struct field without newtype
        if content.contains("pub") && !content.contains("struct") {
            found.push("naked primitive (WRAP)".to_string());
        }
    }

    found
}

/// Check if confession template is present
fn has_confession_template(content: &str) -> bool {
    content.contains("VIOLATION CONFESSION")
        && content.contains("Commandment Violated:")
        && content.contains("I have confessed")
}

/// Extract which commandment might be violated
fn detect_commandment_violation(content: &str) -> Option<(u8, &'static str)> {
    // Commandment 12: ENFORCE - unwrap/expect/panic in non-test code
    if (content.contains("unwrap()") || content.contains("expect(") || content.contains("panic!"))
        && !content.contains("#[test]")
        && !content.contains("#[cfg(test)]")
    {
        return Some((12, "ENFORCE"));
    }

    // Commandment 4: WRAP - naked primitives
    if content.contains("pub ")
        && (content.contains(": u8") || content.contains(": u32") || content.contains(": i32"))
    {
        if !content.contains("struct") && !content.contains("newtype") {
            return Some((4, "WRAP"));
        }
    }

    // Commandment 6: MATCH - non-exhaustive patterns
    if content.contains("_ =>") && content.contains("unreachable!") {
        return Some((6, "MATCH"));
    }

    None
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_ok(),
    };

    // Only check Edit and Write tools
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if tool_name != "Edit" && tool_name != "Write" {
        exit_ok();
    }

    let tool_input = match &input.tool_input {
        Some(v) => v,
        None => exit_ok(),
    };

    let file_path = match get_file_path(tool_input) {
        Some(p) => p,
        None => exit_ok(),
    };

    // Only check Rust files
    if !file_path.ends_with(".rs") {
        exit_ok();
    }

    // Skip test files
    if file_path.contains("/tests/") || file_path.contains("_test.rs") {
        exit_ok();
    }

    let content = match get_content(tool_input) {
        Some(c) => c,
        None => exit_ok(),
    };

    // Check for violations
    let violations = has_violation_indicators(&content);

    if violations.is_empty() {
        exit_ok();
    }

    // Check if confession is present
    if has_confession_template(&content) {
        // Confession present - emit telemetry and allow
        HookTelemetry::new("posttool_violation_confessor", "PostToolUse")
            .with_tool(tool_name)
            .with_session(&input.session_id)
            .with_result("confession_accepted")
            .with_extra(json!({
                "file": file_path,
                "violations": violations
            }))
            .emit();

        exit_ok();
    }

    // Violation without confession - require template
    let commandment = detect_commandment_violation(&content);
    let (num, name) = commandment.unwrap_or((0, "UNKNOWN"));

    // Emit telemetry for violation
    HookTelemetry::new("posttool_violation_confessor", "PostToolUse")
        .with_tool(tool_name)
        .with_session(&input.session_id)
        .with_result("confession_required")
        .with_extra(json!({
            "file": file_path,
            "violations": violations,
            "commandment": num,
            "commandment_name": name
        }))
        .emit();

    let template = format!(
        r#"
VIOLATION DETECTED — Confession Required

Indicators found: {}
Suspected Commandment: {} — {}

Required confession format:

```
VIOLATION CONFESSION

Commandment Violated: {} — {}
Code in Question: [SNIPPET]
Nature of Violation: [EXPLANATION]
Root Cause: [WHY IT HAPPENED]
Corrected Code: [FIXED VERSION]
Prevention: [HOW TO AVOID IN FUTURE]

I have confessed. The record stands.
```

Fix the violation or add confession comment to proceed."#,
        violations.join(", "),
        num,
        name,
        num,
        name
    );

    exit_block(&template);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_violation_detection() {
        let code = "fn foo() { x.unwrap(); }";
        let violations = has_violation_indicators(code);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_confession_template() {
        let content = r#"
// VIOLATION CONFESSION
// Commandment Violated: 12 — ENFORCE
// I have confessed. The record stands.
"#;
        assert!(has_confession_template(content));
    }

    #[test]
    fn test_commandment_detection() {
        let code = "fn foo() { x.unwrap(); }";
        let cmd = detect_commandment_violation(code);
        assert_eq!(cmd, Some((12, "ENFORCE")));
    }
}
