//! Lifecycle Completeness Checker Hook
//!
//! PostToolUse hook that warns when synthesized skills lack complete T1 lifecycle.
//! Checks for begins/ends verbs in SKILL.md files.
//!
//! Exit codes:
//! - 0: Complete lifecycle (begins + ends present)
//! - 1: Warning - incomplete lifecycle (missing begins/ends)
//! - 2: Block - no T1 verbs detected (invalid synthesis)

use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

/// T1 State verbs to detect
const T1_VERBS: [&str; 5] = ["exists", "changes", "persists", "begins", "ends"];

/// Synonyms for begins (initialization)
const BEGINS_SYNONYMS: [&str; 6] = [
    "begins",
    "initialization",
    "initialize",
    "creates",
    "starts",
    "on_start",
];

/// Synonyms for ends (termination)
const ENDS_SYNONYMS: [&str; 7] = [
    "ends",
    "termination",
    "cleanup",
    "closes",
    "on_complete",
    "finalize",
    "shutdown",
];

fn main() -> ExitCode {
    // Read tool result from stdin (Claude Code hook protocol)
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        return ExitCode::SUCCESS; // Can't read, don't block
    }

    // Parse the hook input to get file path
    let file_path = match extract_file_path(&input) {
        Some(path) => path,
        None => {
            // Also check environment variable
            match env::var("CLAUDE_FILE_PATH") {
                Ok(path) => path,
                Err(_) => return ExitCode::SUCCESS, // No file path, skip
            }
        }
    };

    // Only check SKILL.md files
    if !file_path.ends_with("SKILL.md") {
        return ExitCode::SUCCESS;
    }

    // Read the file content
    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c.to_lowercase(),
        Err(_) => return ExitCode::SUCCESS, // Can't read file, don't block
    };

    // Check for T1 verbs
    let t1_count = T1_VERBS
        .iter()
        .filter(|verb| content.contains(*verb))
        .count();

    if t1_count == 0 {
        eprintln!("🚫 LIFECYCLE ERROR");
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("  No T1 primitive verbs detected in SKILL.md");
        eprintln!("  Required: exists, changes, persists, begins, ends");
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        return ExitCode::from(2); // Block
    }

    // Check for begins (initialization)
    let has_begins = BEGINS_SYNONYMS.iter().any(|syn| content.contains(syn));

    // Check for ends (termination)
    let has_ends = ENDS_SYNONYMS.iter().any(|syn| content.contains(syn));

    // Check for lifecycle-complete flag
    let has_flag = content.contains("lifecycle-complete: true");

    if !has_begins || !has_ends {
        eprintln!("⚠️  LIFECYCLE WARNING");
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        if !has_begins {
            eprintln!("  Missing: begins (initialization)");
            eprintln!("  Suggest: Add begins(entity) for startup logic");
            eprintln!("           - Resource acquisition");
            eprintln!("           - Transaction initialization");
            eprintln!("           - State setup");
        }

        if !has_ends {
            eprintln!();
            eprintln!("  Missing: ends (termination)");
            eprintln!("  Suggest: Add ends(entity) for cleanup logic");
            eprintln!("           - Resource release");
            eprintln!("           - Transaction commit/rollback");
            eprintln!("           - State cleanup");
        }

        eprintln!();
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("  Complete lifecycle pattern:");
        eprintln!("  begins → exists → changes/persists → ends");
        eprintln!();
        eprintln!("  Add to frontmatter when complete:");
        eprintln!("  lifecycle-complete: true");
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        return ExitCode::from(1); // Warn
    }

    // Complete lifecycle
    if has_flag {
        eprintln!("✓ Lifecycle complete: begins → ... → ends");
    }

    ExitCode::SUCCESS
}

/// Extract file path from hook input JSON
fn extract_file_path(input: &str) -> Option<String> {
    // Simple JSON parsing for "file_path" or "path" field
    // Hook input format: {"tool": "Write", "file_path": "...", ...}

    // Try to find file_path
    if let Some(start) = input.find("\"file_path\"") {
        return extract_string_value(input, start);
    }

    // Try to find path
    if let Some(start) = input.find("\"path\"") {
        return extract_string_value(input, start);
    }

    None
}

fn extract_string_value(input: &str, key_start: usize) -> Option<String> {
    let after_key = &input[key_start..];

    // Find the colon after the key
    let colon_pos = after_key.find(':')?;
    let after_colon = &after_key[colon_pos + 1..];

    // Find the opening quote
    let quote_start = after_colon.find('"')?;
    let after_quote = &after_colon[quote_start + 1..];

    // Find the closing quote
    let quote_end = after_quote.find('"')?;

    Some(after_quote[..quote_end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_file_path() {
        let input = r#"{"tool": "Write", "file_path": "/home/user/skills/test/SKILL.md"}"#;
        assert_eq!(
            extract_file_path(input),
            Some("/home/user/skills/test/SKILL.md".to_string())
        );
    }

    #[test]
    fn test_lifecycle_detection() {
        let complete = "begins exists changes persists ends";
        let incomplete = "exists changes persists";

        assert!(BEGINS_SYNONYMS.iter().any(|s| complete.contains(s)));
        assert!(ENDS_SYNONYMS.iter().any(|s| complete.contains(s)));
        assert!(!BEGINS_SYNONYMS.iter().any(|s| incomplete.contains(s)));
        assert!(!ENDS_SYNONYMS.iter().any(|s| incomplete.contains(s)));
    }
}
