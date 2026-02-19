use crate::models::{ExecutorResult, ExecutorType};
use async_trait::async_trait;

#[async_trait]
pub trait Executor: Send + Sync {
    fn executor_type(&self) -> ExecutorType;
    async fn execute(
        &self,
        action: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<ExecutorResult>;
    async fn health_check(&self) -> bool;
}

pub mod browser;
pub mod maestro;
pub mod notify;
pub mod shell;
pub mod speech;
pub mod workflow;
