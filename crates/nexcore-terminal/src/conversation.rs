//! Conversation context management with token-aware windowing.
//!
//! Manages conversation history for AI-powered terminal sessions.
//! Enforces a token budget by truncating oldest messages while
//! preserving the system prompt.
//!
//! ## Primitive Grounding
//!
//! `π(Persistence) + σ(Sequence) + N(Quantity) + ∂(Boundary)`

use crate::ai::{AiMessage, AiRole, AiToolCall};
use serde::{Deserialize, Serialize};

/// Manages conversation history with token-aware windowing.
///
/// Conversations grow unbounded without management. This type enforces
/// a token budget by truncating oldest messages (preserving system prompt).
///
/// GroundsTo: T2-C — π(Persistence) 0.40 + σ(Sequence) 0.30 + N(Quantity) 0.20 + ∂(Boundary) 0.10
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    /// System prompt (always retained, never truncated).
    system_prompt: Option<String>,
    /// Ordered message history.
    messages: Vec<ConversationMessage>,
    /// Estimated token count for current messages.
    estimated_tokens: usize,
    /// Maximum token budget for context window.
    max_context_tokens: usize,
    /// Model identifier.
    model: String,
}

/// A message in the conversation, richer than AiMessage to support tool use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Who sent this message.
    pub role: AiRole,
    /// Text content.
    pub content: String,
    /// Tool calls requested (assistant only, may be empty).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<AiToolCall>,
    /// Tool use ID this result responds to (tool_result only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    /// Whether this tool result is an error.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_error: bool,
}

impl ConversationContext {
    /// Create a new conversation with model and token budget.
    #[must_use]
    pub fn new(model: impl Into<String>, max_context_tokens: usize) -> Self {
        Self {
            system_prompt: None,
            messages: Vec::new(),
            estimated_tokens: 0,
            max_context_tokens,
            model: model.into(),
        }
    }

    /// Set the system prompt.
    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        let prompt = prompt.into();
        // Subtract old system prompt tokens, add new
        if let Some(ref old) = self.system_prompt {
            self.estimated_tokens = self.estimated_tokens.saturating_sub(estimate_tokens(old));
        }
        self.estimated_tokens = self
            .estimated_tokens
            .saturating_add(estimate_tokens(&prompt));
        self.system_prompt = Some(prompt);
    }

    /// Add a user message.
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        let content = content.into();
        self.estimated_tokens = self
            .estimated_tokens
            .saturating_add(estimate_tokens(&content));
        self.messages.push(ConversationMessage {
            role: AiRole::User,
            content,
            tool_calls: Vec::new(),
            tool_use_id: None,
            is_error: false,
        });
        self.apply_windowing();
    }

    /// Add an assistant text response.
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        let content = content.into();
        self.estimated_tokens = self
            .estimated_tokens
            .saturating_add(estimate_tokens(&content));
        self.messages.push(ConversationMessage {
            role: AiRole::Assistant,
            content,
            tool_calls: Vec::new(),
            tool_use_id: None,
            is_error: false,
        });
    }

    /// Add an assistant message that includes tool calls.
    pub fn add_assistant_tool_calls(&mut self, content: String, tool_calls: Vec<AiToolCall>) {
        let mut tokens = estimate_tokens(&content);
        for call in &tool_calls {
            tokens = tokens.saturating_add(estimate_tokens(&call.name));
            tokens = tokens.saturating_add(estimate_tokens(&call.input.to_string()));
        }
        self.estimated_tokens = self.estimated_tokens.saturating_add(tokens);
        self.messages.push(ConversationMessage {
            role: AiRole::Assistant,
            content,
            tool_calls,
            tool_use_id: None,
            is_error: false,
        });
    }

    /// Add a tool result message.
    pub fn add_tool_result(
        &mut self,
        tool_use_id: &str,
        content: impl Into<String>,
        is_error: bool,
    ) {
        let content = content.into();
        self.estimated_tokens = self
            .estimated_tokens
            .saturating_add(estimate_tokens(&content));
        self.messages.push(ConversationMessage {
            role: AiRole::ToolResult,
            content,
            tool_calls: Vec::new(),
            tool_use_id: Some(tool_use_id.to_string()),
            is_error,
        });
    }

    /// Get messages formatted for the Anthropic Messages API.
    ///
    /// Converts internal representation to `AiMessage` vec.
    /// System prompt is NOT included here — it goes in a separate field.
    #[must_use]
    pub fn messages_for_api(&self) -> &[ConversationMessage] {
        &self.messages
    }

    /// Get the system prompt.
    #[must_use]
    pub fn system_prompt(&self) -> Option<&str> {
        self.system_prompt.as_deref()
    }

    /// Get the model identifier.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Estimated token count.
    #[must_use]
    pub fn estimated_tokens(&self) -> usize {
        self.estimated_tokens
    }

    /// Number of messages in history.
    #[must_use]
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Clear conversation history (retains system prompt).
    pub fn clear(&mut self) {
        self.messages.clear();
        self.estimated_tokens = self
            .system_prompt
            .as_ref()
            .map_or(0, |p| estimate_tokens(p));
    }

    /// Apply token windowing: if over 90% budget, drop oldest until under 75%.
    ///
    /// Preserves message pairs: if a tool_result follows an assistant tool_use,
    /// they are dropped together to maintain conversation coherence.
    fn apply_windowing(&mut self) {
        let threshold = self.max_context_tokens.saturating_mul(9) / 10;
        let target = self.max_context_tokens.saturating_mul(3) / 4;

        if self.estimated_tokens <= threshold {
            return;
        }

        while self.estimated_tokens > target && !self.messages.is_empty() {
            let msg = self.messages.remove(0);
            self.estimated_tokens = self
                .estimated_tokens
                .saturating_sub(estimate_message_tokens(&msg));

            // If we just dropped an assistant message with tool_calls,
            // also drop all immediately following ToolResult messages
            // (they reference the tool_use IDs we just removed).
            if msg.role == AiRole::Assistant && !msg.tool_calls.is_empty() {
                while self
                    .messages
                    .first()
                    .is_some_and(|m| m.role == AiRole::ToolResult)
                {
                    let orphan = self.messages.remove(0);
                    self.estimated_tokens = self
                        .estimated_tokens
                        .saturating_sub(estimate_message_tokens(&orphan));
                }
            }

            // If we just dropped a ToolResult as the oldest message
            // (shouldn't happen with well-formed history, but guard it),
            // continue dropping to avoid orphaned tool_results at the front.
            if msg.role == AiRole::ToolResult {
                while self
                    .messages
                    .first()
                    .is_some_and(|m| m.role == AiRole::ToolResult)
                {
                    let orphan = self.messages.remove(0);
                    self.estimated_tokens = self
                        .estimated_tokens
                        .saturating_sub(estimate_message_tokens(&orphan));
                }
            }
        }
    }
}

/// Estimate token count for a string (~4 chars per token heuristic).
///
/// For precise counting, use the Anthropic count_tokens endpoint.
/// This heuristic is intentionally conservative (overestimates slightly)
/// to avoid exceeding the context window.
#[must_use]
fn estimate_tokens(text: &str) -> usize {
    // ~4 chars per token is reasonable for English text.
    // Add 3 before dividing to round up.
    text.len().saturating_add(3) / 4
}

/// Estimate tokens for a full conversation message.
fn estimate_message_tokens(msg: &ConversationMessage) -> usize {
    let mut tokens = estimate_tokens(&msg.content);
    for call in &msg.tool_calls {
        tokens = tokens.saturating_add(estimate_tokens(&call.name));
        tokens = tokens.saturating_add(estimate_tokens(&call.input.to_string()));
    }
    if let Some(ref id) = msg.tool_use_id {
        tokens = tokens.saturating_add(estimate_tokens(id));
    }
    // Role overhead
    tokens.saturating_add(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_context_has_zero_messages() {
        let ctx = ConversationContext::new("claude-sonnet-4-6", 100_000);
        assert_eq!(ctx.message_count(), 0);
        assert_eq!(ctx.estimated_tokens(), 0);
        assert!(ctx.system_prompt().is_none());
    }

    #[test]
    fn add_user_message_increases_count() {
        let mut ctx = ConversationContext::new("claude-sonnet-4-6", 100_000);
        ctx.add_user_message("Hello");
        assert_eq!(ctx.message_count(), 1);
        assert!(ctx.estimated_tokens() > 0);
    }

    #[test]
    fn add_assistant_message_preserves_order() {
        let mut ctx = ConversationContext::new("claude-sonnet-4-6", 100_000);
        ctx.add_user_message("Hello");
        ctx.add_assistant_message("Hi there");
        assert_eq!(ctx.message_count(), 2);
        assert_eq!(ctx.messages_for_api()[0].role, AiRole::User);
        assert_eq!(ctx.messages_for_api()[1].role, AiRole::Assistant);
    }

    #[test]
    fn system_prompt_survives_clear() {
        let mut ctx = ConversationContext::new("claude-sonnet-4-6", 100_000);
        ctx.set_system_prompt("You are helpful");
        ctx.add_user_message("Hello");
        ctx.clear();
        assert_eq!(ctx.message_count(), 0);
        assert_eq!(ctx.system_prompt(), Some("You are helpful"));
        assert!(ctx.estimated_tokens() > 0); // system prompt tokens remain
    }

    #[test]
    fn token_estimation_in_expected_range() {
        // "Hello world" = 11 chars, ~3 tokens expected
        let tokens = estimate_tokens("Hello world");
        assert!(tokens >= 2 && tokens <= 5, "Got {tokens}");
    }

    #[test]
    fn windowing_drops_oldest_messages() {
        // Small budget: 20 tokens
        let mut ctx = ConversationContext::new("claude-sonnet-4-6", 20);
        ctx.add_user_message("First message that is fairly long to consume tokens");
        ctx.add_assistant_message("Reply");
        // This should trigger windowing
        ctx.add_user_message("Third message that pushes over the token budget limit");
        // Oldest messages should have been dropped
        assert!(
            ctx.estimated_tokens() <= 20,
            "Tokens: {}",
            ctx.estimated_tokens()
        );
    }

    #[test]
    fn windowing_preserves_system_prompt() {
        let mut ctx = ConversationContext::new("claude-sonnet-4-6", 30);
        ctx.set_system_prompt("System");
        ctx.add_user_message("Long message to fill the budget quickly here");
        ctx.add_user_message("Another long message that triggers windowing now");
        assert_eq!(ctx.system_prompt(), Some("System"));
    }

    #[test]
    fn tool_call_roundtrip() {
        let mut ctx = ConversationContext::new("claude-sonnet-4-6", 100_000);
        ctx.add_user_message("What is PRR for aspirin?");

        let tool_calls = vec![crate::ai::AiToolCall {
            id: "call_1".to_string(),
            name: "pv_signal_complete".to_string(),
            input: serde_json::json!({"drug": "aspirin", "event": "hepatotoxicity"}),
        }];
        ctx.add_assistant_tool_calls(String::new(), tool_calls);
        ctx.add_tool_result("call_1", "PRR=2.5, chi2=4.1", false);

        assert_eq!(ctx.message_count(), 3);
        assert_eq!(ctx.messages_for_api()[1].role, AiRole::Assistant);
        assert_eq!(ctx.messages_for_api()[1].tool_calls.len(), 1);
        assert_eq!(ctx.messages_for_api()[2].role, AiRole::ToolResult);
        assert_eq!(
            ctx.messages_for_api()[2].tool_use_id.as_deref(),
            Some("call_1")
        );
    }

    #[test]
    fn model_getter_returns_configured_model() {
        let ctx = ConversationContext::new("claude-opus-4-6", 200_000);
        assert_eq!(ctx.model(), "claude-opus-4-6");
    }

    #[test]
    fn estimated_tokens_tracks_additions() {
        let mut ctx = ConversationContext::new("claude-sonnet-4-6", 100_000);
        let before = ctx.estimated_tokens();
        ctx.add_user_message("Some text");
        assert!(ctx.estimated_tokens() > before);
    }

    #[test]
    fn messages_for_api_returns_all_when_under_budget() {
        let mut ctx = ConversationContext::new("claude-sonnet-4-6", 100_000);
        ctx.add_user_message("Hello");
        ctx.add_assistant_message("Hi");
        ctx.add_user_message("How are you?");
        assert_eq!(ctx.messages_for_api().len(), 3);
    }

    #[test]
    fn empty_context_returns_empty_messages() {
        let ctx = ConversationContext::new("claude-sonnet-4-6", 100_000);
        assert!(ctx.messages_for_api().is_empty());
    }
}
