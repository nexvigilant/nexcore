//! Skill error types

use nexcore_error::Error;

/// Skill execution errors
#[derive(Debug, Error)]
pub enum SkillError {
    /// Input validation failed
    #[error("validation error: {0}")]
    Validation(String),
    /// Execution failed
    #[error("execution error: {0}")]
    Execution(String),
    /// Dependency not available
    #[error("dependency not found: {0}")]
    DependencyNotFound(String),
    /// MCP tool call failed
    #[error("mcp error: {0}")]
    McpError(String),
}

/// Result type for skill operations
pub type SkillResult = Result<super::SkillOutput, SkillError>;
