use crate::executors::Executor;
use crate::models::{ExecutorResult, ExecutorType};
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::process::Command;
use tracing::info;

pub struct SpeechExecutor;

#[async_trait]
impl Executor for SpeechExecutor {
    fn executor_type(&self) -> ExecutorType {
        ExecutorType::Speech
    }

    async fn execute(
        &self,
        action: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<ExecutorResult> {
        let text = params
            .get("text")
            .and_then(|t: &serde_json::Value| t.as_str())
            .unwrap_or(action);

        info!(text, "speaking");

        let cmd = if cfg!(target_os = "macos") {
            "say"
        } else {
            "espeak"
        };

        let output = Command::new(cmd).arg(text).output().await?;

        Ok(ExecutorResult {
            executor: self.executor_type(),
            success: output.status.success(),
            output: Some(text.to_string()),
            error: None,
            metadata: HashMap::new(),
        })
    }

    async fn health_check(&self) -> bool {
        true
    }
}
