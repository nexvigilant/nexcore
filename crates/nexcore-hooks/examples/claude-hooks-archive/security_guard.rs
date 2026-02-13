//! Security Guard Hook - PreToolUse
//!
//! Blocks dangerous commands and protects sensitive files.
//!
//! # Security Rules
//!
//! 1. Block destructive commands (`rm -rf /`, `mkfs`, etc.)
//! 2. Block credential access (`.env`, `credentials.json`)
//! 3. Block system file modifications (`/etc/passwd`, etc.)
//! 4. Warn on sudo/admin commands
//!
//! # Usage
//!
//! ```bash
//! cargo build --example security_guard --release
//! # Add to ~/.claude/settings.json
//! ```

use claude_hooks::prelude::*;

/// Dangerous command patterns to block.
const DANGEROUS_COMMANDS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    "mkfs",
    "dd if=/dev/zero",
    ":(){:|:&};:",  // Fork bomb
    "chmod -R 777 /",
    "> /dev/sda",
];

/// Sensitive file patterns to protect.
const PROTECTED_PATHS: &[&str] = &[
    ".env",
    "credentials.json",
    "secrets.yaml",
    ".aws/credentials",
    ".ssh/id_",
    "/etc/passwd",
    "/etc/shadow",
];

/// Commands requiring warning (non-blocking).
const WARN_COMMANDS: &[&str] = &[
    "sudo ",
    "su ",
    "chmod ",
    "chown ",
];

fn main() -> HookResult<()> {
    // Parse input from stdin
    let input: PreToolUseInput = read_input()?;

    // Only check Bash and Write tools
    match input.tool_name.as_str() {
        "Bash" => check_bash_command(&input)?,
        "Write" | "Edit" => check_file_write(&input)?,
        _ => {} // Allow other tools
    }

    // Default: allow
    Ok(())
}

fn check_bash_command(input: &PreToolUseInput) -> HookResult<()> {
    let Some(command) = input.tool_input.get("command").and_then(|c| c.as_str()) else {
        return Ok(()); // No command field, allow
    };

    // Check for dangerous commands (block)
    for pattern in DANGEROUS_COMMANDS {
        if command.contains(pattern) {
            write_error(&format!(
                "🛡️ BLOCKED: Dangerous command detected\n\
                 Pattern: {}\n\
                 Command: {}",
                pattern,
                truncate(command, 100)
            ))?;
            exit_block("Security policy violation");
        }
    }

    // Check for warning commands (allow with warning)
    for pattern in WARN_COMMANDS {
        if command.contains(pattern) {
            // Write to stderr but don't block (exit 0)
            eprintln!(
                "⚠️ WARNING: Administrative command detected: {}",
                truncate(command, 50)
            );
        }
    }

    Ok(())
}

fn check_file_write(input: &PreToolUseInput) -> HookResult<()> {
    let Some(file_path) = input.tool_input.get("file_path").and_then(|p| p.as_str()) else {
        return Ok(()); // No file_path field, allow
    };

    // Check for protected paths
    for pattern in PROTECTED_PATHS {
        if file_path.contains(pattern) {
            write_error(&format!(
                "🛡️ BLOCKED: Protected file access\n\
                 Pattern: {}\n\
                 Path: {}",
                pattern,
                file_path
            ))?;
            exit_block("Security policy violation");
        }
    }

    Ok(())
}

/// Truncate string for display.
fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        &s[..max_len]
    }
}
