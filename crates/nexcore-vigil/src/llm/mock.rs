use crate::llm::LLMClient;
use crate::models::{Event, Interaction};
use async_trait::async_trait;
use chrono::Utc;

pub struct MockLLMClient {
    pub response: String,
}

#[async_trait]
impl LLMClient for MockLLMClient {
    async fn invoke(&self, _context: &str, event: &Event) -> nexcore_error::Result<Interaction> {
        Ok(Interaction {
            id: "mock-id".to_string(),
            event: event.clone(),
            prompt: "".to_string(),
            response: self.response.clone(),
            timestamp: Utc::now(),
            tokens_used: 10,
            contains_learning: false,
            actions_taken: vec![],
        })
    }

    async fn health_check(&self) -> bool {
        true
    }
}
