//! Error types for skill execution.

use nexcore_error::Error;
use std::path::PathBuf;

/// Result type for skill execution operations.
pub type Result<T> = std::result::Result<T, ExecutionError>;

/// Errors that can occur during skill execution.
#[derive(Debug, Error)]
pub enum ExecutionError {
    /// Skill not found at expected path.
    #[error("Skill not found: {name} at {path:?}")]
    SkillNotFound {
        /// Skill name.
        name: String,
        /// Expected path.
        path: PathBuf,
    },

    /// No executor available for this skill type.
    #[error("No executor available for skill: {name} (type: {skill_type})")]
    NoExecutorAvailable {
        /// Skill name.
        name: String,
        /// Skill type (shell, binary, library).
        skill_type: String,
    },

    /// Script execution failed.
    #[error("Script execution failed: {script:?} - {message}")]
    ScriptFailed {
        /// Script path.
        script: PathBuf,
        /// Error message.
        message: String,
    },

    /// Parameter validation failed.
    #[error("Parameter validation failed: {message}")]
    ValidationFailed {
        /// Validation error message.
        message: String,
    },

    /// Timeout during execution.
    #[error("Execution timeout after {seconds}s for skill: {skill}")]
    Timeout {
        /// Skill name.
        skill: String,
        /// Timeout in seconds.
        seconds: u64,
    },

    /// IO error during execution.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Process spawn failed.
    #[error("Failed to spawn process: {command} - {message}")]
    SpawnFailed {
        /// Command that failed.
        command: String,
        /// Error message.
        message: String,
    },

    /// Non-zero exit code.
    #[error("Process exited with code {code}: {stderr}")]
    NonZeroExit {
        /// Exit code.
        code: i32,
        /// Stderr output.
        stderr: String,
    },
}
