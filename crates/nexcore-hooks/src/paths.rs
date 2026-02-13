//! Shared path utilities for session-scoped directories.
//!
//! Plans and handoffs are scoped by working directory to prevent
//! cross-session confusion when running multiple Claude sessions.
//!
//! Directory structure:
//! - `~/.claude/plans/{cwd-slug}/` - Session-scoped plans
//! - `~/.claude/handoffs/{cwd-slug}/` - Session-scoped handoffs
//! - `~/.claude/plans/archived/` - Archived legacy plans
//! - `~/.claude/handoffs/archived/` - Archived legacy handoffs

use std::collections::hash_map::DefaultHasher;
use std::env;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// Generate a deterministic slug from the current working directory.
/// Returns format: "{last_component}-{8_char_hash}"
///
/// Example: `/home/user/projects/NexBet` → `NexBet-a1b2c3d4`
pub fn cwd_slug() -> String {
    let cwd = env::current_dir().unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    cwd.to_string_lossy().hash(&mut hasher);
    let hash = hasher.finish() as u32;
    let name = cwd.file_name().and_then(|n| n.to_str()).unwrap_or("root");
    // Replace non-alphanumeric with underscore using simple byte check
    let clean: String = name
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' {
                b as char
            } else {
                '_'
            }
        })
        .collect();
    format!("{}-{:08x}", clean, hash)
}

/// Get the handoffs directory scoped to the current working directory.
///
/// Returns: `~/.claude/handoffs/{cwd-slug}/`
pub fn handoffs_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("handoffs")
        .join(cwd_slug())
}

/// Get the plans directory scoped to the current working directory.
///
/// Returns: `~/.claude/plans/{cwd-slug}/`
pub fn plans_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("plans")
        .join(cwd_slug())
}

/// Get the global archived directory for legacy handoffs.
///
/// Returns: `~/.claude/handoffs/archived/`
pub fn archived_handoffs_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("handoffs")
        .join("archived")
}

/// Get the global archived directory for legacy plans.
///
/// Returns: `~/.claude/plans/archived/`
pub fn archived_plans_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("plans")
        .join("archived")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cwd_slug_format() {
        let slug = cwd_slug();
        // Should have format "name-8hexchars"
        assert!(slug.contains('-'), "slug should contain hyphen: {}", slug);
        let parts: Vec<&str> = slug.rsplitn(2, '-').collect();
        assert_eq!(parts[0].len(), 8, "hash should be 8 chars: {}", slug);
    }

    #[test]
    fn test_handoffs_dir_contains_slug() {
        let dir = handoffs_dir();
        let path_str = dir.to_string_lossy();
        assert!(path_str.contains(".claude/handoffs/"));
        assert!(path_str.contains('-')); // slug separator
    }

    #[test]
    fn test_plans_dir_contains_slug() {
        let dir = plans_dir();
        let path_str = dir.to_string_lossy();
        assert!(path_str.contains(".claude/plans/"));
        assert!(path_str.contains('-')); // slug separator
    }
}
