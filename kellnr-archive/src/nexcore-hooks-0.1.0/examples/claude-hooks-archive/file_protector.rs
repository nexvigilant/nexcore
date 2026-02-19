//! File protection hook example.
//! 
//! This hook blocks modifications to sensitive files and directories.
//! 
//! ## Usage
//! 
//! Build and configure in settings.json:
//! 
//! ```json
//! {
//!   "hooks": {
//!     "PreToolUse": [{
//!       "matcher": "Edit|Write",
//!       "hooks": [{
//!         "type": "command",
//!         "command": "/path/to/file_protector"
//!       }]
//!     }]
//!   }
//! }
//! ```

use claude_hooks::{
    exit_success, read_input, write_output,
    input::PreToolUseInput,
    output::PreToolUseOutput,
    HookResult,
};

/// Protected paths and patterns.
const PROTECTED_PATTERNS: &[&str] = &[
    ".env",
    ".env.local",
    ".env.production",
    "package-lock.json",
    "Cargo.lock",
    "yarn.lock",
    "pnpm-lock.yaml",
    ".git/",
    ".ssh/",
    "id_rsa",
    "id_ed25519",
    "*.pem",
    "*.key",
    "secrets/",
    "credentials/",
];

/// Check if a path matches any protected pattern.
fn is_protected(path: &str) -> Option<&'static str> {
    for pattern in PROTECTED_PATTERNS {
        if pattern.ends_with('/') {
            // Directory pattern
            let dir = pattern.trim_end_matches('/');
            if path.contains(&format!("/{}/", dir)) || path.ends_with(&format!("/{}", dir)) {
                return Some(pattern);
            }
        } else if let Some(suffix) = pattern.strip_prefix('*') {
            // Wildcard suffix pattern
            if path.ends_with(suffix) {
                return Some(pattern);
            }
        } else {
            // Exact match or contains
            if path.ends_with(pattern) || path.contains(&format!("/{}", pattern)) {
                return Some(pattern);
            }
        }
    }
    None
}

fn main() -> HookResult<()> {
    let input: PreToolUseInput = read_input()?;
    
    // Extract file path from tool input
    let file_path = input.tool_input
        .get("file_path")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    if file_path.is_empty() {
        exit_success();
    }
    
    // Check if the file is protected
    if let Some(pattern) = is_protected(file_path) {
        // Use JSON output for a cleaner deny message
        let output = PreToolUseOutput::deny(format!(
            "Protected file: '{}' matches pattern '{}'. \
             This file is protected from modifications. \
             If you need to modify it, please ask the user for explicit permission.",
            file_path, pattern
        ));
        write_output(&output)?;
        exit_success();
    }
    
    // Not protected, allow the operation
    exit_success();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_files() {
        assert!(is_protected("/project/.env").is_some());
        assert!(is_protected("/project/.env.local").is_some());
        assert!(is_protected("/project/.env.production").is_some());
    }

    #[test]
    fn test_lock_files() {
        assert!(is_protected("/project/package-lock.json").is_some());
        assert!(is_protected("/project/Cargo.lock").is_some());
    }

    #[test]
    fn test_git_directory() {
        assert!(is_protected("/project/.git/config").is_some());
        assert!(is_protected("/project/.git/hooks/pre-commit").is_some());
    }

    #[test]
    fn test_key_files() {
        assert!(is_protected("/home/user/.ssh/id_rsa").is_some());
        assert!(is_protected("/project/secrets/api.key").is_some());
        assert!(is_protected("/project/cert.pem").is_some());
    }

    #[test]
    fn test_allowed_files() {
        assert!(is_protected("/project/src/main.rs").is_none());
        assert!(is_protected("/project/package.json").is_none());
        assert!(is_protected("/project/README.md").is_none());
    }
}
