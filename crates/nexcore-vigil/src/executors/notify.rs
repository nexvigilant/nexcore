use crate::executors::Executor;
use crate::models::{ExecutorResult, ExecutorType};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct NotifyExecutor;

#[async_trait]
impl Executor for NotifyExecutor {
    fn executor_type(&self) -> ExecutorType {
        ExecutorType::Notify
    }

    async fn execute(
        &self,
        action: &str,
        _params: serde_json::Value,
    ) -> nexcore_error::Result<ExecutorResult> {
        Ok(ExecutorResult {
            executor: self.executor_type(),
            success: true,
            output: Some(format!("Notification sent: {}", action)),
            error: None,
            metadata: HashMap::new(),
        })
    }

    async fn health_check(&self) -> bool {
        true
    }
}
