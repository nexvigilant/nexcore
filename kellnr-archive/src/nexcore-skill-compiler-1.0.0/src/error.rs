//! Compiler error types.

use thiserror::Error;

/// Errors produced by the skill compiler pipeline.
#[derive(Debug, Error)]
pub enum CompilerError {
    /// A referenced skill was not found in the registry or filesystem.
    #[error("Skill not found: {name}")]
    SkillNotFound {
        /// Skill name that was not found.
        name: String,
    },

    /// The compound spec is invalid.
    #[error("Invalid spec: {message}")]
    InvalidSpec {
        /// Description of the validation failure.
        message: String,
    },

    /// Fewer than 2 skills were provided (minimum for composition).
    #[error("Insufficient skills: {count} (minimum 2 required)")]
    InsufficientSkills {
        /// Actual count provided.
        count: usize,
    },

    /// Skill dependency graph contains a cycle.
    #[error("Circular dependency detected: {}", cycle.join(" -> "))]
    CircularDependency {
        /// Cycle path.
        cycle: Vec<String>,
    },

    /// Code generation failed at a specific stage.
    #[error("Codegen failed at {stage}: {message}")]
    CodegenFailed {
        /// Pipeline stage that failed.
        stage: String,
        /// Error details.
        message: String,
    },

    /// `cargo build` returned a non-zero exit code.
    #[error("Build failed (exit {code}): {stderr}")]
    BuildFailed {
        /// Process exit code.
        code: i32,
        /// Captured stderr.
        stderr: String,
    },

    /// TOML deserialization error.
    #[error(transparent)]
    TomlParse(#[from] toml::de::Error),

    /// I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, CompilerError>;
