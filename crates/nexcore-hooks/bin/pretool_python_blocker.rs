//! Python File Blocker - Enforces 100% Rust Policy
//!
//! # Purpose
//! Blocks creation/editing of `.py` files to enforce the codebase's 100% Rust policy.
//! Python is treated as legacy technical debt requiring migration, not extension.
//!
//! # Event
//! `PreToolUse:Edit|Write`
//!
//! # Behavior
//! - **Exit 0**: Non-Python files pass through (fast path, no JSON parsing)
//! - **Exit 2**: Blocks `.py` file operations with error message
//!
//! # Performance
//! Optimized for minimal overhead:
//! 1. Fast string search for `.py` before JSON parsing
//! 2. Only parses JSON if `.py` detected in input
//! 3. No external dependencies beyond serde_json
//!
//! # Policy Reference
//! From `CLAUDE.md`:
//! > **100% Rust Development**: All new code MUST be Rust. No exceptions.
//! > Python is legacy debt to be migrated.
//!
//! # Tiers
//! Active in: `dev`, `review`, `deploy` (all tiers - non-negotiable policy)

fn main() {
    // Fast path: read stdin, check for .py in file_path
    let mut buf = String::new();
    if std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf).is_err() {
        println!("{{}}");
        return;
    }

    // Check if file_path contains .py (fast string search, no JSON parsing for pass case)
    if !buf.contains(".py") {
        println!("{{}}");
        return;
    }

    // Only parse JSON if we might need to block
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&buf) {
        if let Some(path) = v
            .get("tool_input")
            .and_then(|t| t.get("file_path"))
            .and_then(|p| p.as_str())
        {
            if path.ends_with(".py") {
                eprintln!("BLOCKED: .py files violate 100% Rust policy");
                std::process::exit(2);
            }
        }
    }

    println!("{{}}");
}
