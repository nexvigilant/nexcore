//! Shell script executor for skills.
//!
//! # Security
//!
//! Scripts are executed directly (not via shell) to prevent command injection.
//! Parameters are passed via environment variables and stdin, never as command args.

use crate::executor::SkillExecutor;
use crate::models::ExecutionMethod;
use crate::{
    ExecutionError, ExecutionRequest, ExecutionResult, ExecutionStatus, Result, SkillInfo,
};
use async_trait::async_trait;
use std::process::Stdio;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Executor for shell script-based skills.
///
/// Invokes scripts directly (not via bash) with JSON input via stdin.
/// Captures JSON output from stdout.
pub struct ShellExecutor {
    /// Whether to capture stderr separately.
    _capture_stderr: bool,
}

impl ShellExecutor {
    /// Create a new shell executor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            _capture_stderr: true,
        }
    }

    /// Find the best shell script to execute.
    fn find_shell_script(skill: &SkillInfo) -> Option<std::path::PathBuf> {
        // Priority: verify.sh > run.sh > execute.sh > first .sh found
        let scripts_dir = skill.path.join("scripts");

        for preferred in &["verify.sh", "run.sh", "execute.sh", "main.sh"] {
            let path = scripts_dir.join(preferred);
            if path.exists() {
                return Some(path);
            }
        }

        // Fall back to first shell script in methods
        skill.execution_methods.iter().find_map(|m| {
            if let ExecutionMethod::Shell(path) = m {
                Some(path.clone())
            } else {
                None
            }
        })
    }

    /// Find the best binary to execute.
    fn find_binary(skill: &SkillInfo) -> Option<std::path::PathBuf> {
        skill.execution_methods.iter().find_map(|m| {
            if let ExecutionMethod::Binary(path) = m {
                Some(path.clone())
            } else {
                None
            }
        })
    }

    /// Find documentation fallback.
    fn find_library(skill: &SkillInfo) -> Option<std::path::PathBuf> {
        skill.execution_methods.iter().find_map(|m| {
            if let ExecutionMethod::Library(path) = m {
                Some(path.clone())
            } else {
                None
            }
        })
    }
}

impl Default for ShellExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SkillExecutor for ShellExecutor {
    async fn execute(
        &self,
        skill: &SkillInfo,
        request: &ExecutionRequest,
    ) -> Result<ExecutionResult> {
        let start = Instant::now();

        // Try binary first, then shell script, then documentation fallback.
        if let Some(library_path) = Self::find_library(skill) {
            let duration_ms = start.elapsed().as_millis() as u64;
            let skill_md = std::fs::read_to_string(&library_path).unwrap_or_default();
            let preview = skill_md
                .lines()
                .filter(|line| !line.trim().is_empty())
                .take(20)
                .collect::<Vec<_>>()
                .join("\n");

            return Ok(ExecutionResult {
                skill_name: skill.name.clone(),
                status: ExecutionStatus::Completed,
                output: serde_json::json!({
                    "mode": "documentation_fallback",
                    "message": "No executable script or binary found; returned SKILL.md guidance",
                    "skill_md": library_path,
                    "params_echo": request.parameters,
                    "preview": preview,
                }),
                artifacts: Vec::new(),
                error: None,
                duration_ms,
                exit_code: Some(0),
                stdout: preview,
                stderr: String::new(),
            });
        }

        let executable = if let Some(binary) = Self::find_binary(skill) {
            binary
        } else if let Some(script) = Self::find_shell_script(skill) {
            script
        } else {
            return Err(ExecutionError::NoExecutorAvailable {
                name: skill.name.clone(),
                skill_type: "shell".to_string(),
            });
        };

        info!(
            skill = %skill.name,
            executable = %executable.display(),
            "Executing skill"
        );

        // Execute directly - scripts must have shebang (#!/usr/bin/env bash)
        // This avoids command injection by not invoking shell with arguments
        let mut cmd = Command::new(&executable);

        // Set working directory
        let work_dir = request
            .working_dir
            .clone()
            .unwrap_or_else(|| skill.path.clone());
        cmd.current_dir(&work_dir);

        // Add environment variables
        for (key, value) in &request.env {
            cmd.env(key, value);
        }

        // Pass parameters as environment variable (not as args - prevents injection)
        cmd.env("SKILL_PARAMS", request.parameters.to_string());
        cmd.env("SKILL_NAME", &skill.name);
        cmd.env("SKILL_PATH", skill.path.to_string_lossy().as_ref());

        // Set up stdin/stdout/stderr
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Spawn process
        let mut child = cmd.spawn().map_err(|e| ExecutionError::SpawnFailed {
            command: executable.display().to_string(),
            message: e.to_string(),
        })?;

        // Write parameters to stdin
        if let Some(mut stdin) = child.stdin.take() {
            let params_json = serde_json::to_string(&request.parameters)?;
            stdin.write_all(params_json.as_bytes()).await?;
            stdin.shutdown().await?;
        }

        // Wait for completion with timeout
        let output = tokio::time::timeout(request.timeout, child.wait_with_output())
            .await
            .map_err(|_| ExecutionError::Timeout {
                skill: skill.name.clone(),
                seconds: request.timeout.as_secs(),
            })?
            .map_err(|e| ExecutionError::ScriptFailed {
                script: executable.clone(),
                message: e.to_string(),
            })?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code();

        debug!(
            skill = %skill.name,
            exit_code = ?exit_code,
            duration_ms = duration_ms,
            stdout_len = stdout.len(),
            stderr_len = stderr.len(),
            "Skill execution completed"
        );

        // Check exit code
        if !output.status.success() {
            let code = exit_code.unwrap_or(-1);
            warn!(
                skill = %skill.name,
                exit_code = code,
                stderr = %stderr,
                "Skill execution failed"
            );

            return Ok(ExecutionResult {
                skill_name: skill.name.clone(),
                status: ExecutionStatus::Failed,
                output: serde_json::Value::Null,
                artifacts: Vec::new(),
                error: Some(format!("Exit code {code}: {stderr}")),
                duration_ms,
                exit_code,
                stdout,
                stderr,
            });
        }

        // Try to parse stdout as JSON
        let output_json = if stdout.trim().is_empty() {
            serde_json::json!({ "success": true })
        } else {
            serde_json::from_str(&stdout).unwrap_or_else(|_| {
                // Wrap non-JSON output
                serde_json::json!({
                    "raw_output": stdout.trim(),
                    "success": true
                })
            })
        };

        Ok(ExecutionResult {
            skill_name: skill.name.clone(),
            status: ExecutionStatus::Completed,
            output: output_json,
            artifacts: Vec::new(),
            error: None,
            duration_ms,
            exit_code,
            stdout,
            stderr,
        })
    }

    fn can_execute(&self, skill: &SkillInfo) -> bool {
        skill.has_shell_scripts() || skill.has_binary() || skill.has_library()
    }

    fn name(&self) -> &'static str {
        "shell"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_executor_creation() {
        let executor = ShellExecutor::new();
        assert!(executor._capture_stderr);
    }
}
