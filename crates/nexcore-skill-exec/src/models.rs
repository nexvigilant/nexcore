//! Data models for skill execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Request to execute a skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    /// Name of the skill to execute.
    pub skill_name: String,
    /// Parameters to pass to the skill.
    pub parameters: serde_json::Value,
    /// Execution timeout.
    #[serde(default = "default_timeout")]
    pub timeout: Duration,
    /// Environment variables to set.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Working directory override.
    #[serde(default)]
    pub working_dir: Option<PathBuf>,
}

fn default_timeout() -> Duration {
    Duration::from_secs(60)
}

impl ExecutionRequest {
    /// Create a new execution request.
    #[must_use]
    pub fn new(skill_name: impl Into<String>, parameters: serde_json::Value) -> Self {
        Self {
            skill_name: skill_name.into(),
            parameters,
            timeout: default_timeout(),
            env: HashMap::new(),
            working_dir: None,
        }
    }

    /// Set timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Add environment variable.
    #[must_use]
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set working directory.
    #[must_use]
    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

/// Result of skill execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Skill that was executed.
    pub skill_name: String,
    /// Execution status.
    pub status: ExecutionStatus,
    /// Output data (JSON).
    pub output: serde_json::Value,
    /// Artifacts created during execution.
    #[serde(default)]
    pub artifacts: Vec<PathBuf>,
    /// Error message if failed.
    #[serde(default)]
    pub error: Option<String>,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
    /// Exit code from process (if applicable).
    #[serde(default)]
    pub exit_code: Option<i32>,
    /// Stdout from process.
    #[serde(default)]
    pub stdout: String,
    /// Stderr from process.
    #[serde(default)]
    pub stderr: String,
}

impl ExecutionResult {
    /// Create a successful result.
    #[must_use]
    pub fn success(skill_name: String, output: serde_json::Value, duration_ms: u64) -> Self {
        Self {
            skill_name,
            status: ExecutionStatus::Completed,
            output,
            artifacts: Vec::new(),
            error: None,
            duration_ms,
            exit_code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    /// Create a failed result.
    #[must_use]
    pub fn failed(skill_name: String, error: String, duration_ms: u64) -> Self {
        Self {
            skill_name,
            status: ExecutionStatus::Failed,
            output: serde_json::Value::Null,
            artifacts: Vec::new(),
            error: Some(error),
            duration_ms,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    /// Check if execution was successful.
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self.status, ExecutionStatus::Completed)
    }
}

/// Execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    /// Execution completed successfully.
    Completed,
    /// Execution failed.
    Failed,
    /// Execution timed out.
    Timeout,
    /// Execution was cancelled.
    Cancelled,
}

/// Information about a skill for execution purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    /// Skill name.
    pub name: String,
    /// Skill directory path.
    pub path: PathBuf,
    /// Available execution methods.
    pub execution_methods: Vec<ExecutionMethod>,
    /// Input schema (if defined).
    #[serde(default)]
    pub input_schema: Option<serde_json::Value>,
    /// Output schema (if defined).
    #[serde(default)]
    pub output_schema: Option<serde_json::Value>,
}

impl SkillInfo {
    /// Check if skill has shell scripts.
    #[must_use]
    pub fn has_shell_scripts(&self) -> bool {
        self.execution_methods
            .iter()
            .any(|m| matches!(m, ExecutionMethod::Shell(_)))
    }

    /// Check if skill has a binary.
    #[must_use]
    pub fn has_binary(&self) -> bool {
        self.execution_methods
            .iter()
            .any(|m| matches!(m, ExecutionMethod::Binary(_)))
    }

    /// Check if skill has documentation/library fallback.
    #[must_use]
    pub fn has_library(&self) -> bool {
        self.execution_methods
            .iter()
            .any(|m| matches!(m, ExecutionMethod::Library(_)))
    }

    /// Get the primary execution method.
    #[must_use]
    pub fn primary_method(&self) -> Option<&ExecutionMethod> {
        // Prefer: Binary > Shell > None
        self.execution_methods
            .iter()
            .find(|m| matches!(m, ExecutionMethod::Binary(_)))
            .or_else(|| {
                self.execution_methods
                    .iter()
                    .find(|m| matches!(m, ExecutionMethod::Shell(_)))
            })
            .or_else(|| {
                self.execution_methods
                    .iter()
                    .find(|m| matches!(m, ExecutionMethod::Library(_)))
            })
    }
}

/// Method for executing a skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionMethod {
    /// Shell script execution.
    Shell(PathBuf),
    /// Compiled binary execution.
    Binary(PathBuf),
    /// Rust library (dynamic loading - future).
    Library(PathBuf),
}
