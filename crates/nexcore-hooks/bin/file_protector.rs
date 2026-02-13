//! File Protector Hook (from claude-hooks library)
//!
//! PreToolUse:Edit|Write hook that blocks modifications to sensitive files.

use nexcore_hooks::{exit_block, exit_success_auto, read_input};

/// Protected paths and patterns
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
    "/etc/passwd",
    "/etc/shadow",
];

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
        } else if pattern.starts_with('/') {
            // Absolute path
            if path.starts_with(pattern) {
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

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only check Edit/Write
    if !input.is_write_tool() {
        exit_success_auto();
    }

    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    if let Some(pattern) = is_protected(file_path) {
        exit_block(&format!(
            "Protected file: '{}' matches pattern '{}'\n\
             This file is protected from modifications.\n\
             Ask the user for explicit permission if needed.",
            file_path, pattern
        ));
    }

    exit_success_auto();
}
