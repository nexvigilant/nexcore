//! Centralized constants for all hooks.
//!
//! Single source of truth for magic numbers, thresholds, and configuration values.
//! Changing values here affects all hooks that use them.

/// Memory thresholds (in MB)
pub mod memory {
    /// Warn when available RAM drops below this threshold
    pub const LOW_MEMORY_THRESHOLD_MB: u64 = 1024;

    /// Critical memory threshold (may cause OOM)
    pub const CRITICAL_MEMORY_MB: u64 = 256;
}

/// Timeout values (in seconds)
pub mod timeouts {
    /// Default hook execution timeout
    pub const HOOK_TIMEOUT_SECS: u64 = 10;

    /// Clippy execution timeout (longer for large crates)
    pub const CLIPPY_TIMEOUT_SECS: u64 = 60;

    /// Cargo build timeout
    pub const CARGO_BUILD_TIMEOUT_SECS: u64 = 120;
}

/// File size thresholds
pub mod files {
    /// Maximum lines before considering a file "large"
    pub const MAX_FILE_LINES: usize = 10_000;

    /// Lines changed threshold for "large change" warning
    pub const LARGE_CHANGE_THRESHOLD: usize = 500;

    /// Maximum content size to analyze (in bytes)
    pub const MAX_CONTENT_SIZE: usize = 1_000_000;
}

/// Code quality thresholds
pub mod quality {
    /// Maximum cyclomatic complexity allowed
    pub const MAX_COMPLEXITY: u32 = 15;

    /// Maximum function length (lines)
    pub const MAX_FUNCTION_LINES: usize = 100;

    /// Clone depth threshold for excessive cloning
    pub const CLONE_DEPTH_THRESHOLD: usize = 3;
}

/// ANSI color codes for terminal output
pub mod colors {
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const RED: &str = "\x1b[31m";
    pub const BLUE: &str = "\x1b[34m";
    pub const BOLD: &str = "\x1b[1m";
    pub const RESET: &str = "\x1b[0m";
}

/// Exit codes for hooks
pub mod exit_codes {
    /// Success - allow tool execution
    pub const SUCCESS: i32 = 0;

    /// Warning - allow but show feedback
    pub const WARN: i32 = 1;

    /// Block - prevent tool execution
    pub const BLOCK: i32 = 2;
}

/// Environment variable names
pub mod env {
    /// Set to any value to enable minimal/silent mode
    pub const HOOKS_MINIMAL: &str = "HOOKS_MINIMAL";

    /// Set to any value to skip all hooks
    pub const HOOKS_DISABLED: &str = "HOOKS_DISABLED";

    /// Set to any value to enable verbose output
    pub const HOOKS_VERBOSE: &str = "HOOKS_VERBOSE";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thresholds_are_sensible() {
        assert!(memory::LOW_MEMORY_THRESHOLD_MB > memory::CRITICAL_MEMORY_MB);
        assert!(files::MAX_FILE_LINES > files::LARGE_CHANGE_THRESHOLD);
        assert!(timeouts::CLIPPY_TIMEOUT_SECS > timeouts::HOOK_TIMEOUT_SECS);
    }

    #[test]
    fn test_exit_codes() {
        assert_eq!(exit_codes::SUCCESS, 0);
        assert_eq!(exit_codes::WARN, 1);
        assert_eq!(exit_codes::BLOCK, 2);
    }
}
