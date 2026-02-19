use crate::models::{Event, Interaction};
use async_trait::async_trait;

#[async_trait]
pub trait LLMClient: Send + Sync {
    async fn invoke(&self, context: &str, event: &Event) -> anyhow::Result<Interaction>;
    async fn health_check(&self) -> bool;
}

pub mod claude;
pub mod cortex;
pub mod forge_harness;
pub mod gemini;
pub mod mock;
