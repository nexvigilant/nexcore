//! Tool Failure Responder Hook
//!
//! Event: PostToolUseFailure
//!
//! Detects tool failures and provides recovery context to Claude.
//! When Rust compilation or test failures occur, this hook injects
//! context suggesting appropriate recovery agents.
//!
//! # Detection Patterns
//!
//! - **Rust Compiler Errors (E0xxx)**: Suggests rust-compiler-doctor
//! - **Borrow Checker Errors**: Suggests rust-borrow-doctor
//! - **Test Failures**: Suggests rust-test-architect
//! - **Clippy Failures**: Provides inline fix suggestions

use nexcore_hooks::{HookOutput, exit_success_auto, read_input};
use regex::Regex;

/// Error patterns and their recovery agents
struct ErrorRecovery {
    pattern: &'static str,
    agent: &'static str,
    description: &'static str,
}

const ERROR_RECOVERIES: &[ErrorRecovery] = &[
    ErrorRecovery {
        pattern: r"error\[E0(3\d{2})\]", // Ownership errors (E0300-E0399)
        agent: "rust-borrow-doctor",
        description: "Ownership error detected",
    },
    ErrorRecovery {
        pattern: r"error\[E0(4\d{2}|5\d{2})\]", // Borrowing errors (E0400-E0599)
        agent: "rust-borrow-doctor",
        description: "Borrowing/lifetime error detected",
    },
    ErrorRecovery {
        pattern: r"error\[E0(6\d{2}|7\d{2})\]", // Lifetime errors (E0600-E0799)
        agent: "rust-borrow-doctor",
        description: "Lifetime error detected",
    },
    ErrorRecovery {
        pattern: r"error\[E\d+\]", // Any compiler error
        agent: "rust-compiler-doctor",
        description: "Rust compiler error detected",
    },
    ErrorRecovery {
        pattern: r"test .+ \.\.\. FAILED",
        agent: "rust-test-architect",
        description: "Test failure detected",
    },
    ErrorRecovery {
        pattern: r"error: could not compile",
        agent: "rust-compiler-doctor",
        description: "Compilation failure detected",
    },
    ErrorRecovery {
        pattern: r"warning: .+ \(clippy::",
        agent: "rust-reviewer",
        description: "Clippy warnings detected",
    },
];

fn detect_error_type(output: &str) -> Option<&'static ErrorRecovery> {
    for recovery in ERROR_RECOVERIES {
        // INVARIANT: These regex patterns are compile-time constants and valid
        if let Ok(re) = Regex::new(recovery.pattern) {
            if re.is_match(output) {
                return Some(recovery);
            }
        }
    }
    None
}

fn extract_error_codes(output: &str) -> Vec<String> {
    let mut codes = Vec::new();
    // INVARIANT: This is a valid regex pattern
    if let Ok(re) = Regex::new(r"error\[(E\d+)\]") {
        for cap in re.captures_iter(output) {
            if let Some(code) = cap.get(1) {
                let code_str = code.as_str().to_string();
                if !codes.contains(&code_str) {
                    codes.push(code_str);
                }
            }
        }
    }
    codes
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only handle Bash tool failures (where we expect compiler output)
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if tool_name != "Bash" {
        exit_success_auto();
    }

    // Get the tool response (which contains error output)
    let response = match &input.tool_response {
        Some(r) => r.to_string(),
        None => exit_success_auto(),
    };

    // Detect error type
    let recovery = match detect_error_type(&response) {
        Some(r) => r,
        None => exit_success_auto(),
    };

    // Extract specific error codes for context
    let error_codes = extract_error_codes(&response);
    let codes_str = if error_codes.is_empty() {
        String::new()
    } else {
        format!(" ({})", error_codes.join(", "))
    };

    // Build recovery context
    let context = format!(
        r#"
🔧 TOOL FAILURE RECOVERY ─────────────────────────────────
   Issue: {}{}

   ⚡ SUGGESTED ACTION: Use the {} agent to diagnose and fix.

   You can invoke this agent with:
     Task tool, subagent_type: "{}"
     prompt: "Diagnose and fix the errors in the previous output"
───────────────────────────────────────────────────────────"#,
        recovery.description, codes_str, recovery.agent, recovery.agent
    );

    // Inject context for Claude
    let output = HookOutput {
        hook_specific_output: Some(nexcore_hooks::HookSpecificOutput::post_tool_use_context(
            context,
        )),
        ..Default::default()
    };

    output.emit();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ownership_error() {
        let output = "error[E0382]: borrow of moved value";
        let recovery = detect_error_type(output);
        assert!(recovery.is_some());
        assert_eq!(recovery.map(|r| r.agent), Some("rust-borrow-doctor"));
    }

    #[test]
    fn test_detect_lifetime_error() {
        let output = "error[E0621]: explicit lifetime required";
        let recovery = detect_error_type(output);
        assert!(recovery.is_some());
        assert_eq!(recovery.map(|r| r.agent), Some("rust-borrow-doctor"));
    }

    #[test]
    fn test_detect_generic_error() {
        let output = "error[E0277]: the trait bound is not satisfied";
        let recovery = detect_error_type(output);
        assert!(recovery.is_some());
        // Falls through to generic compiler doctor
    }

    #[test]
    fn test_detect_test_failure() {
        let output = "test my_test::test_something ... FAILED";
        let recovery = detect_error_type(output);
        assert!(recovery.is_some());
        assert_eq!(recovery.map(|r| r.agent), Some("rust-test-architect"));
    }

    #[test]
    fn test_extract_error_codes() {
        let output = "error[E0382]: borrow of moved value\nerror[E0499]: cannot borrow";
        let codes = extract_error_codes(output);
        assert_eq!(codes.len(), 2);
        assert!(codes.contains(&"E0382".to_string()));
        assert!(codes.contains(&"E0499".to_string()));
    }

    #[test]
    fn test_no_match() {
        let output = "Compiling project v0.1.0\nFinished in 1.23s";
        let recovery = detect_error_type(output);
        assert!(recovery.is_none());
    }
}
