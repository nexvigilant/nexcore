//! Model orchestrator with fallback chain.
//! Supports async generation with automatic failover.

use crate::ai::clients::{GenerationOptions, ModelClient};
use anyhow::{Result, anyhow};
use std::collections::HashMap;

/// Orchestrator that manages multiple model clients with fallback.
pub struct ModelOrchestrator {
    clients: HashMap<String, Box<dyn ModelClient>>,
    fallback_chain: Vec<String>,
}

impl Default for ModelOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelOrchestrator {
    /// Create a new empty orchestrator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            fallback_chain: Vec::new(),
        }
    }

    /// Add a client to the orchestrator.
    pub fn add_client(&mut self, name: &str, client: Box<dyn ModelClient>) {
        self.clients.insert(name.to_string(), client);
        self.fallback_chain.push(name.to_string());
    }

    /// Generate text, trying each client in the fallback chain.
    ///
    /// # Errors
    /// Returns error if all clients fail or no clients registered.
    pub async fn generate(&self, prompt: &str) -> Result<String> {
        self.generate_with_options(prompt, None).await
    }

    /// Generate text with options, trying each client in the fallback chain.
    ///
    /// # Errors
    /// Returns error if all clients fail or no clients registered.
    pub async fn generate_with_options(
        &self,
        prompt: &str,
        options: Option<GenerationOptions>,
    ) -> Result<String> {
        let mut last_error = None;

        for client_name in &self.fallback_chain {
            if let Some(client) = self.clients.get(client_name) {
                match client.generate(prompt, options.clone()).await {
                    Ok(resp) => return Ok(resp),
                    Err(e) => {
                        last_error = Some(e);
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("No clients registered in orchestrator")))
    }

    /// Get the list of registered client names.
    #[must_use]
    pub fn client_names(&self) -> Vec<&str> {
        self.fallback_chain.iter().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_creation() {
        let orch = ModelOrchestrator::new();
        assert!(orch.client_names().is_empty());
    }
}
