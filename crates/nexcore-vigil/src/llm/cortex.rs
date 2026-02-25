//! # Cortex LLM Client
//!
//! Local LLM inference via nexcore-cortex (HuggingFace Candle).
//! Implements `LLMClient` for Guardian's COMPARE phase.
//!
//! ## Homeostasis Flow
//! ```text
//! SENSE (PAMPs/DAMPs) → COMPARE (cortex: local) → ACT (executor)
//!                            ↓ (fallback if uncertain)
//!                        Claude/Gemini API
//! ```

use crate::llm::LLMClient;
use crate::models::{Event, Interaction};
use async_trait::async_trait;
use nexcore_chrono::DateTime;
use nexcore_cortex::engine::InferenceEngine;
use nexcore_cortex::generate::GenerateParams;
use std::sync::Arc;
use tracing::info;

/// Local LLM client backed by nexcore-cortex InferenceEngine.
///
/// Provides zero-latency, zero-cost inference for Guardian's homeostasis loop.
/// Uses quantized GGUF models on CPU for fast triage decisions.
///
/// ## Tier: T3 (composes σ + μ + ς + ∂ + N + → + ∃)
pub struct CortexClient {
    engine: Arc<InferenceEngine>,
    params: GenerateParams,
}

impl CortexClient {
    /// Create a new CortexClient wrapping a loaded InferenceEngine.
    pub fn new(engine: Arc<InferenceEngine>) -> Self {
        Self {
            engine,
            params: GenerateParams::default(),
        }
    }

    /// Create with custom generation parameters.
    pub fn with_params(engine: Arc<InferenceEngine>, params: GenerateParams) -> Self {
        Self { engine, params }
    }

    /// Check if the underlying model is loaded and ready.
    pub fn is_loaded(&self) -> bool {
        self.engine.is_loaded()
    }

    /// Format an event into a prompt suitable for the local model.
    fn format_prompt(context: &str, event: &Event) -> String {
        format!(
            "You are FRIDAY's local cortex. Analyze this event and decide the action.\n\
             Source: {}\nType: {}\nPriority: {:?}\n\
             Context: {}\n\
             Respond with [ACTION: type] payload [/ACTION] for any actions.",
            event.source, event.event_type, event.priority, context
        )
    }

    /// Extract actions from response text (same pattern as ClaudeClient).
    fn extract_actions(response: &str) -> Vec<String> {
        let re = match regex::Regex::new(
            r"(?s)\[ACTION:\s*(?P<type>[^\]]+)\](?P<payload>.*?)\[/ACTION\]",
        ) {
            Ok(re) => re,
            Err(err) => {
                tracing::warn!(error = %err, "cortex_action_regex_failed");
                return Vec::new();
            }
        };
        re.captures_iter(response)
            .filter_map(|cap| {
                let action_type = cap.name("type")?.as_str().trim();
                let payload = cap.name("payload")?.as_str().trim();
                Some(format!("{}: {}", action_type, payload))
            })
            .collect()
    }
}

#[async_trait]
impl LLMClient for CortexClient {
    async fn invoke(&self, context: &str, event: &Event) -> nexcore_error::Result<Interaction> {
        info!(source = %event.source, event_type = %event.event_type, "cortex_local_invoke");

        let prompt = Self::format_prompt(context, event);
        let response = self.engine.generate(&prompt, &self.params)?;
        let actions = Self::extract_actions(&response);

        Ok(Interaction {
            id: format!("cortex-{}", DateTime::now().timestamp_millis()),
            event: event.clone(),
            prompt,
            response: response.clone(),
            timestamp: DateTime::now(),
            tokens_used: 0, // Local inference — no API token cost
            contains_learning: response.contains("[LEARNING]"),
            actions_taken: actions,
        })
    }

    async fn health_check(&self) -> bool {
        self.is_loaded()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_prompt_includes_event_fields() {
        let event = Event {
            source: "voice".to_string(),
            event_type: "user_spoke".to_string(),
            ..Event::default()
        };
        let prompt = CortexClient::format_prompt("test context", &event);
        assert!(prompt.contains("voice"));
        assert!(prompt.contains("user_spoke"));
        assert!(prompt.contains("test context"));
    }

    #[test]
    fn extract_actions_parses_action_tags() {
        let response = "Analysis complete. [ACTION: SilentLog] event logged [/ACTION]";
        let actions = CortexClient::extract_actions(response);
        assert_eq!(actions.len(), 1);
        assert!(actions[0].contains("SilentLog"));
    }

    #[test]
    fn extract_actions_handles_empty() {
        let actions = CortexClient::extract_actions("no actions here");
        assert!(actions.is_empty());
    }

    #[test]
    fn extract_actions_multiple() {
        let response = "[ACTION: Log] first [/ACTION] middle [ACTION: Escalate] second [/ACTION]";
        let actions = CortexClient::extract_actions(response);
        assert_eq!(actions.len(), 2);
    }
}
