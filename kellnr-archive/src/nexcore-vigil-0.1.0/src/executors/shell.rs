use crate::executors::Executor;
use crate::models::{ExecutorResult, ExecutorType};
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::process::Command;
use tracing::{error, info};

pub struct ShellExecutor;

impl Default for ShellExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellExecutor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Executor for ShellExecutor {
    fn executor_type(&self) -> ExecutorType {
        ExecutorType::Shell
    }

    async fn execute(
        &self,
        action: &str,
        _params: serde_json::Value,
    ) -> anyhow::Result<ExecutorResult> {
        info!(command = %action, "executing_shell_command");

        if action.contains("rm -rf /") || action.contains(":(){ :|:& };:") {
            error!(command = %action, "blocked_dangerous_command");
            return Ok(ExecutorResult {
                executor: self.executor_type(),
                success: false,
                output: None,
                error: Some("Dangerous command blocked".to_string()),
                metadata: HashMap::new(),
            });
        }

        let output = Command::new("bash").arg("-c").arg(action).output().await?;

        Ok(ExecutorResult {
            executor: self.executor_type(),
            success: output.status.success(),
            output: Some(String::from_utf8_lossy(&output.stdout).to_string()),
            error: if !output.status.success() {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            } else {
                None
            },
            metadata: HashMap::new(),
        })
    }

    async fn health_check(&self) -> bool {
        true
    }
}
