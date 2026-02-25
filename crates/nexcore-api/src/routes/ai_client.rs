//! Sovereign Claude API client — thin wrapper over the Anthropic Messages API.
//!
//! Handles request construction, SSE stream parsing, and token forwarding.
//! Uses `reqwest` (already in workspace) for HTTP. No external SDK dependency.
//!
//! ## Primitive Grounding
//!
//! `σ(Sequence) + μ(Mapping) + →(Causality) + ∂(Boundary)`

use futures::StreamExt;
use nexcore_terminal::ai::{AiResponse, AiTool, AiToolCall};
use nexcore_terminal::conversation::ConversationContext;
use tokio::sync::mpsc;

// ─────────────────────────────────────────────────────────────────────────────
// Error Type
// ─────────────────────────────────────────────────────────────────────────────

/// NexChat error type.
///
/// GroundsTo: T1 — ∂(Boundary)
#[derive(Debug)]
pub enum NexChatError {
    /// API key not found in environment or config.
    ApiKeyMissing,
    /// HTTP request to Anthropic API failed.
    HttpError(reqwest::Error),
    /// API returned an error response.
    ApiError {
        status: u16,
        error_type: String,
        message: String,
    },
    /// SSE stream interrupted before message_stop.
    StreamInterrupted { tokens_received: usize },
    /// Rate limit exceeded.
    RateLimited { retry_after_secs: u64 },
    /// Tool dispatch to MCP failed.
    ToolDispatchFailed { tool_name: String, message: String },
    /// JSON parsing error in SSE stream.
    ParseError(String),
}

impl std::fmt::Display for NexChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiKeyMissing => write!(f, "ANTHROPIC_API_KEY not set"),
            Self::HttpError(e) => write!(f, "HTTP error: {e}"),
            Self::ApiError {
                status,
                error_type,
                message,
            } => write!(f, "API {status} ({error_type}): {message}"),
            Self::StreamInterrupted { tokens_received } => {
                write!(f, "Stream interrupted after {tokens_received} tokens")
            }
            Self::RateLimited { retry_after_secs } => {
                write!(f, "Rate limited, retry after {retry_after_secs}s")
            }
            Self::ToolDispatchFailed { tool_name, message } => {
                write!(f, "Tool '{tool_name}' failed: {message}")
            }
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
        }
    }
}

impl std::error::Error for NexChatError {}

impl From<reqwest::Error> for NexChatError {
    fn from(e: reqwest::Error) -> Self {
        Self::HttpError(e)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Stream Events
// ─────────────────────────────────────────────────────────────────────────────

/// Streaming event from the Claude API.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Text token from assistant.
    Token(String),
    /// Tool use request from assistant.
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    /// Message complete with usage stats.
    Done {
        stop_reason: String,
        input_tokens: u64,
        output_tokens: u64,
    },
    /// Error during streaming.
    Error(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// Configuration
// ─────────────────────────────────────────────────────────────────────────────

const API_BASE_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";
const DEFAULT_MODEL: &str = "claude-sonnet-4-6";
const DEFAULT_MAX_TOKENS: u32 = 4096;

/// Configuration for the Claude client.
#[derive(Debug, Clone)]
pub struct ClaudeConfig {
    /// API key (never logged or displayed).
    pub api_key: String,
    /// Model identifier.
    pub model: String,
    /// Max output tokens per response.
    pub max_tokens: u32,
    /// API base URL.
    pub base_url: String,
}

impl ClaudeConfig {
    /// Create config from an explicit API key.
    ///
    /// # Errors
    ///
    /// Returns `NexChatError::ApiKeyMissing` if the key is empty.
    pub fn from_key(api_key: impl Into<String>) -> Result<Self, NexChatError> {
        let api_key = api_key.into();
        if api_key.is_empty() {
            return Err(NexChatError::ApiKeyMissing);
        }

        Ok(Self {
            api_key,
            model: DEFAULT_MODEL.to_string(),
            max_tokens: DEFAULT_MAX_TOKENS,
            base_url: API_BASE_URL.to_string(),
        })
    }

    /// Create config from environment variable `ANTHROPIC_API_KEY`.
    ///
    /// # Errors
    ///
    /// Returns `NexChatError::ApiKeyMissing` if the env var is not set.
    pub fn from_env() -> Result<Self, NexChatError> {
        let api_key =
            std::env::var("ANTHROPIC_API_KEY").map_err(|_| NexChatError::ApiKeyMissing)?;
        Self::from_key(api_key)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Claude Client
// ─────────────────────────────────────────────────────────────────────────────

/// Sovereign Claude API client.
///
/// GroundsTo: T2-C — σ(Sequence) 0.35 + μ(Mapping) 0.30 + →(Causality) 0.20 + ∂(Boundary) 0.15
pub struct ClaudeClient {
    http: reqwest::Client,
    config: ClaudeConfig,
}

impl ClaudeClient {
    /// Create client from config.
    pub fn new(config: ClaudeConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            config,
        }
    }

    /// Create from environment variable `ANTHROPIC_API_KEY`.
    ///
    /// # Errors
    ///
    /// Returns `NexChatError::ApiKeyMissing` if the env var is not set.
    pub fn from_env() -> Result<Self, NexChatError> {
        let config = ClaudeConfig::from_env()?;
        Ok(Self::new(config))
    }

    /// Stream a conversation, sending events to the provided channel.
    ///
    /// # Errors
    ///
    /// Returns `NexChatError` on HTTP failures, API errors, or stream interruptions.
    pub async fn stream(
        &self,
        context: &ConversationContext,
        tools: &[AiTool],
        tx: mpsc::Sender<StreamEvent>,
    ) -> Result<(), NexChatError> {
        let body = self.build_request_body(context, tools, true);

        let response = self
            .http
            .post(&self.config.base_url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .body(body.to_string())
            .send()
            .await?;

        let status = response.status().as_u16();

        if status == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(30);
            return Err(NexChatError::RateLimited {
                retry_after_secs: retry_after,
            });
        }

        if status != 200 {
            let body_text = response.text().await.unwrap_or_default();
            let (error_type, message) = parse_api_error(&body_text);
            return Err(NexChatError::ApiError {
                status,
                error_type,
                message,
            });
        }

        // Parse SSE stream
        let byte_stream = response.bytes_stream();
        parse_sse_stream(byte_stream, tx).await
    }

    /// Non-streaming message (for tool-use continuation).
    ///
    /// # Errors
    ///
    /// Returns `NexChatError` on HTTP failures or API errors.
    pub async fn message(
        &self,
        context: &ConversationContext,
        tools: &[AiTool],
    ) -> Result<AiResponse, NexChatError> {
        let body = self.build_request_body(context, tools, false);

        let response = self
            .http
            .post(&self.config.base_url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .body(body.to_string())
            .send()
            .await?;

        let status = response.status().as_u16();
        let body_text = response.text().await.unwrap_or_default();

        if status != 200 {
            let (error_type, message) = parse_api_error(&body_text);
            return Err(NexChatError::ApiError {
                status,
                error_type,
                message,
            });
        }

        parse_message_response(&body_text)
    }

    /// Build the Anthropic Messages API request body.
    fn build_request_body(
        &self,
        context: &ConversationContext,
        tools: &[AiTool],
        stream: bool,
    ) -> serde_json::Value {
        let mut messages = Vec::new();

        for msg in context.messages_for_api() {
            match msg.role {
                nexcore_terminal::ai::AiRole::User => {
                    messages.push(serde_json::json!({
                        "role": "user",
                        "content": msg.content,
                    }));
                }
                nexcore_terminal::ai::AiRole::Assistant => {
                    if msg.tool_calls.is_empty() {
                        messages.push(serde_json::json!({
                            "role": "assistant",
                            "content": msg.content,
                        }));
                    } else {
                        // Assistant with tool_use blocks
                        let mut content_blocks: Vec<serde_json::Value> = Vec::new();
                        if !msg.content.is_empty() {
                            content_blocks.push(serde_json::json!({
                                "type": "text",
                                "text": msg.content,
                            }));
                        }
                        for call in &msg.tool_calls {
                            content_blocks.push(serde_json::json!({
                                "type": "tool_use",
                                "id": call.id,
                                "name": call.name,
                                "input": call.input,
                            }));
                        }
                        messages.push(serde_json::json!({
                            "role": "assistant",
                            "content": content_blocks,
                        }));
                    }
                }
                nexcore_terminal::ai::AiRole::ToolResult => {
                    // Tool results go in a user message with tool_result content blocks
                    let content_block = serde_json::json!({
                        "type": "tool_result",
                        "tool_use_id": msg.tool_use_id,
                        "content": msg.content,
                        "is_error": msg.is_error,
                    });
                    messages.push(serde_json::json!({
                        "role": "user",
                        "content": [content_block],
                    }));
                }
                nexcore_terminal::ai::AiRole::System | _ => {
                    // System messages go in the system field, not messages array
                }
            }
        }

        let mut body = serde_json::json!({
            "model": context.model(),
            "max_tokens": self.config.max_tokens,
            "stream": stream,
            "messages": messages,
        });

        // Add system prompt if present
        if let Some(system) = context.system_prompt() {
            body["system"] = serde_json::json!(system);
        }

        // Add tools if any
        if !tools.is_empty() {
            let tool_defs: Vec<serde_json::Value> = tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "description": t.description,
                        "input_schema": t.input_schema,
                    })
                })
                .collect();
            body["tools"] = serde_json::json!(tool_defs);
            body["tool_choice"] = serde_json::json!({"type": "auto"});
        }

        body
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SSE Parsing
// ─────────────────────────────────────────────────────────────────────────────

/// Parse SSE lines from a byte stream into StreamEvents.
async fn parse_sse_stream(
    stream: impl futures::Stream<Item = Result<::bytes::Bytes, reqwest::Error>>,
    tx: mpsc::Sender<StreamEvent>,
) -> Result<(), NexChatError> {
    tokio::pin!(stream);

    let mut buffer = String::new();
    let mut current_event_type = String::new();
    let mut input_tokens: u64 = 0;
    let mut output_tokens: u64 = 0;
    let mut tokens_received: usize = 0;

    // Tool use accumulation state
    let mut active_tool_id = String::new();
    let mut active_tool_name = String::new();
    let mut active_tool_input = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(NexChatError::HttpError)?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // Process complete lines
        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].trim_end_matches('\r').to_string();
            buffer = buffer[newline_pos + 1..].to_string();

            if line.is_empty() {
                // Blank line — process accumulated event
                continue;
            }

            if let Some(event_type) = line.strip_prefix("event: ") {
                current_event_type = event_type.to_string();
                continue;
            }

            if let Some(data) = line.strip_prefix("data: ") {
                if current_event_type == "ping" {
                    current_event_type.clear();
                    continue;
                }

                let parsed: serde_json::Value =
                    serde_json::from_str(data).unwrap_or(serde_json::Value::Null);

                match current_event_type.as_str() {
                    "message_start" => {
                        // Extract input token count
                        if let Some(usage) = parsed.get("message").and_then(|m| m.get("usage")) {
                            input_tokens = usage
                                .get("input_tokens")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);
                        }
                    }

                    "content_block_start" => {
                        if let Some(block) = parsed.get("content_block") {
                            let block_type =
                                block.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            if block_type == "tool_use" {
                                active_tool_id = block
                                    .get("id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                active_tool_name = block
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                active_tool_input.clear();
                            }
                        }
                    }

                    "content_block_delta" => {
                        if let Some(delta) = parsed.get("delta") {
                            let delta_type =
                                delta.get("type").and_then(|v| v.as_str()).unwrap_or("");

                            match delta_type {
                                "text_delta" => {
                                    if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                                        tokens_received += 1;
                                        if tx
                                            .send(StreamEvent::Token(text.to_string()))
                                            .await
                                            .is_err()
                                        {
                                            return Ok(()); // Receiver dropped
                                        }
                                    }
                                }
                                "input_json_delta" => {
                                    if let Some(partial) =
                                        delta.get("partial_json").and_then(|v| v.as_str())
                                    {
                                        active_tool_input.push_str(partial);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    "content_block_stop" => {
                        // If we accumulated tool input, emit ToolUse event
                        if !active_tool_id.is_empty() {
                            let input_value: serde_json::Value =
                                serde_json::from_str(&active_tool_input)
                                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                            if tx
                                .send(StreamEvent::ToolUse {
                                    id: active_tool_id.clone(),
                                    name: active_tool_name.clone(),
                                    input: input_value,
                                })
                                .await
                                .is_err()
                            {
                                return Ok(());
                            }

                            active_tool_id.clear();
                            active_tool_name.clear();
                            active_tool_input.clear();
                        }
                    }

                    "message_delta" => {
                        let stop_reason = parsed
                            .get("delta")
                            .and_then(|d| d.get("stop_reason"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("end_turn")
                            .to_string();

                        output_tokens = parsed
                            .get("usage")
                            .and_then(|u| u.get("output_tokens"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);

                        if tx
                            .send(StreamEvent::Done {
                                stop_reason,
                                input_tokens,
                                output_tokens,
                            })
                            .await
                            .is_err()
                        {
                            return Ok(());
                        }
                    }

                    "message_stop" => {
                        // Stream complete
                        return Ok(());
                    }

                    "error" => {
                        let err_msg = parsed
                            .get("error")
                            .and_then(|e| e.get("message"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown streaming error")
                            .to_string();

                        // Best-effort: send error to receiver, log if channel closed
                        if tx.send(StreamEvent::Error(err_msg)).await.is_err() {
                            tracing::debug!(
                                "SSE error event: receiver dropped before error delivery"
                            );
                        }
                        return Ok(());
                    }

                    _ => {}
                }

                current_event_type.clear();
            }
        }
    }

    // Stream ended without message_stop
    if tokens_received > 0 {
        Err(NexChatError::StreamInterrupted { tokens_received })
    } else {
        Ok(())
    }
}

/// Parse a non-streaming API response into AiResponse.
fn parse_message_response(body: &str) -> Result<AiResponse, NexChatError> {
    let parsed: serde_json::Value = serde_json::from_str(body)
        .map_err(|e| NexChatError::ParseError(format!("Invalid JSON: {e}")))?;

    let mut content = String::new();
    let mut tool_calls = Vec::new();

    if let Some(blocks) = parsed.get("content").and_then(|c| c.as_array()) {
        for block in blocks {
            let block_type = block.get("type").and_then(|v| v.as_str()).unwrap_or("");
            match block_type {
                "text" => {
                    if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                        content.push_str(text);
                    }
                }
                "tool_use" => {
                    let id = block
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = block
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let input = block
                        .get("input")
                        .cloned()
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                    tool_calls.push(AiToolCall::new(id, name, input));
                }
                _ => {}
            }
        }
    }

    let input_tokens = parsed
        .get("usage")
        .and_then(|u| u.get("input_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let output_tokens = parsed
        .get("usage")
        .and_then(|u| u.get("output_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    Ok(AiResponse::new(
        content,
        tool_calls,
        input_tokens,
        output_tokens,
    ))
}

/// Parse error body from Anthropic API.
fn parse_api_error(body: &str) -> (String, String) {
    let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    let error_type = parsed
        .get("error")
        .and_then(|e| e.get("type"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown_error")
        .to_string();
    let message = parsed
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown error")
        .to_string();
    (error_type, message)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_key_validates_empty() {
        // Test the validation path directly (env mutation is unsafe under Edition 2024)
        let result = ClaudeConfig::from_key("");
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), NexChatError::ApiKeyMissing),
            "Empty key should be ApiKeyMissing"
        );
    }

    #[test]
    fn config_from_key_valid() {
        let config = ClaudeConfig::from_key("sk-test-123").unwrap_or_else(|_| ClaudeConfig {
            api_key: String::new(),
            model: String::new(),
            max_tokens: 0,
            base_url: String::new(),
        });
        assert_eq!(config.api_key, "sk-test-123");
        assert_eq!(config.model, DEFAULT_MODEL);
    }

    #[test]
    fn config_defaults() {
        let config = ClaudeConfig {
            api_key: "test-key".to_string(),
            model: DEFAULT_MODEL.to_string(),
            max_tokens: DEFAULT_MAX_TOKENS,
            base_url: API_BASE_URL.to_string(),
        };
        assert_eq!(config.model, "claude-sonnet-4-6");
        assert_eq!(config.max_tokens, 4096);
    }

    #[test]
    fn build_request_body_basic() {
        let config = ClaudeConfig {
            api_key: "test".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            max_tokens: 1024,
            base_url: API_BASE_URL.to_string(),
        };
        let client = ClaudeClient::new(config);

        let mut ctx = ConversationContext::new("claude-sonnet-4-6", 100_000);
        ctx.set_system_prompt("You are helpful");
        ctx.add_user_message("Hello");

        let body = client.build_request_body(&ctx, &[], true);

        assert_eq!(body["model"], "claude-sonnet-4-6");
        assert_eq!(body["stream"], true);
        assert_eq!(body["system"], "You are helpful");
        assert!(body["messages"].as_array().is_some());
        assert!(body.get("tools").is_none()); // No tools = no tools field
    }

    #[test]
    fn build_request_body_with_tools() {
        let config = ClaudeConfig {
            api_key: "test".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            max_tokens: 1024,
            base_url: API_BASE_URL.to_string(),
        };
        let client = ClaudeClient::new(config);

        let ctx = ConversationContext::new("claude-sonnet-4-6", 100_000);
        let tools = vec![AiTool::new(
            "get_weather",
            "Get weather",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "location": { "type": "string" }
                }
            }),
        )];

        let body = client.build_request_body(&ctx, &tools, false);

        assert!(body.get("tools").is_some());
        assert_eq!(body["tools"][0]["name"], "get_weather");
        assert_eq!(body["tool_choice"]["type"], "auto");
        assert_eq!(body["stream"], false);
    }

    #[test]
    fn build_request_body_with_tool_calls() {
        let config = ClaudeConfig {
            api_key: "test".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            max_tokens: 1024,
            base_url: API_BASE_URL.to_string(),
        };
        let client = ClaudeClient::new(config);

        let mut ctx = ConversationContext::new("claude-sonnet-4-6", 100_000);
        ctx.add_user_message("What's the weather?");
        ctx.add_assistant_tool_calls(
            "Let me check.".to_string(),
            vec![AiToolCall::new(
                "call_1",
                "get_weather",
                serde_json::json!({"location": "SF"}),
            )],
        );
        ctx.add_tool_result("call_1", "72F sunny", false);

        let body = client.build_request_body(&ctx, &[], false);
        let empty_arr = Vec::new();
        let messages = body["messages"].as_array().unwrap_or(&empty_arr);

        // user, assistant (with tool_use), user (with tool_result)
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[1]["role"], "assistant");
        assert!(messages[1]["content"].is_array()); // Has content blocks
        assert_eq!(messages[2]["role"], "user");
        assert!(messages[2]["content"][0]["type"] == "tool_result");
    }

    #[test]
    fn parse_message_response_text_only() {
        let body = r#"{
            "id": "msg_1",
            "type": "message",
            "role": "assistant",
            "content": [{"type": "text", "text": "Hello!"}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;

        let resp = parse_message_response(body).expect("should parse");
        assert_eq!(resp.content, "Hello!");
        assert!(resp.tool_calls.is_empty());
        assert_eq!(resp.input_tokens, 10);
        assert_eq!(resp.output_tokens, 5);
    }

    #[test]
    fn parse_message_response_with_tool_use() {
        let body = r#"{
            "id": "msg_2",
            "type": "message",
            "role": "assistant",
            "content": [
                {"type": "text", "text": "Let me check."},
                {"type": "tool_use", "id": "call_1", "name": "get_weather", "input": {"location": "SF"}}
            ],
            "stop_reason": "tool_use",
            "usage": {"input_tokens": 20, "output_tokens": 30}
        }"#;

        let resp = parse_message_response(body).expect("should parse");
        assert_eq!(resp.content, "Let me check.");
        assert_eq!(resp.tool_calls.len(), 1);
        assert_eq!(resp.tool_calls[0].name, "get_weather");
        assert_eq!(resp.tool_calls[0].id, "call_1");
    }

    #[test]
    fn parse_api_error_body() {
        let body = r#"{
            "type": "error",
            "error": {
                "type": "rate_limit_error",
                "message": "Rate limit exceeded"
            }
        }"#;
        let (etype, msg) = parse_api_error(body);
        assert_eq!(etype, "rate_limit_error");
        assert_eq!(msg, "Rate limit exceeded");
    }

    #[test]
    fn parse_api_error_malformed() {
        let (etype, msg) = parse_api_error("not json");
        assert_eq!(etype, "unknown_error");
        assert_eq!(msg, "Unknown error");
    }

    #[test]
    fn nexchat_error_display() {
        let e = NexChatError::ApiKeyMissing;
        assert_eq!(format!("{e}"), "ANTHROPIC_API_KEY not set");

        let e = NexChatError::RateLimited {
            retry_after_secs: 30,
        };
        assert_eq!(format!("{e}"), "Rate limited, retry after 30s");
    }

    #[tokio::test]
    async fn sse_parse_text_tokens() {
        let sse_data = "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"usage\":{\"input_tokens\":10}}}\n\nevent: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\nevent: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\nevent: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\" world\"}}\n\nevent: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":0}\n\nevent: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":5}}\n\nevent: message_stop\ndata: {\"type\":\"message_stop\"}\n\n";

        let stream =
            futures::stream::once(async { Ok::<_, reqwest::Error>(bytes::Bytes::from(sse_data)) });

        let (tx, mut rx) = mpsc::channel(32);
        let parse_result = parse_sse_stream(stream, tx).await;
        assert!(parse_result.is_ok());

        let mut tokens = Vec::new();
        let mut done = false;
        while let Ok(event) = rx.try_recv() {
            match event {
                StreamEvent::Token(t) => tokens.push(t),
                StreamEvent::Done {
                    stop_reason,
                    input_tokens,
                    output_tokens,
                } => {
                    assert_eq!(stop_reason, "end_turn");
                    assert_eq!(input_tokens, 10);
                    assert_eq!(output_tokens, 5);
                    done = true;
                }
                _ => {}
            }
        }
        assert_eq!(tokens, vec!["Hello", " world"]);
        assert!(done);
    }

    #[tokio::test]
    async fn sse_parse_tool_use() {
        let sse_data = "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"usage\":{\"input_tokens\":15}}}\n\nevent: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"get_weather\",\"input\":{}}}\n\nevent: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"location\\\": \\\"SF\\\"}\"}}\n\nevent: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":0}\n\nevent: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":20}}\n\nevent: message_stop\ndata: {\"type\":\"message_stop\"}\n\n";

        let stream =
            futures::stream::once(async { Ok::<_, reqwest::Error>(bytes::Bytes::from(sse_data)) });

        let (tx, mut rx) = mpsc::channel(32);
        let parse_result = parse_sse_stream(stream, tx).await;
        assert!(parse_result.is_ok());

        let mut tool_use = None;
        while let Ok(event) = rx.try_recv() {
            if let StreamEvent::ToolUse { id, name, input } = event {
                tool_use = Some((id, name, input));
            }
        }
        let (id, name, input) = tool_use.expect("should have tool_use");
        assert_eq!(id, "toolu_1");
        assert_eq!(name, "get_weather");
        assert_eq!(input["location"], "SF");
    }

    #[tokio::test]
    async fn sse_parse_error_event() {
        let sse_data = "event: error\ndata: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"Overloaded\"}}\n\n";

        let stream =
            futures::stream::once(async { Ok::<_, reqwest::Error>(bytes::Bytes::from(sse_data)) });

        let (tx, mut rx) = mpsc::channel(32);
        if let Err(e) = parse_sse_stream(stream, tx).await {
            panic!("SSE parse should not fail on error event: {e}");
        }

        if let Ok(StreamEvent::Error(msg)) = rx.try_recv() {
            assert_eq!(msg, "Overloaded");
        } else {
            panic!("Expected error event");
        }
    }
}
