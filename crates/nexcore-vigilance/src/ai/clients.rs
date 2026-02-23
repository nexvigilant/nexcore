//! AI model client implementations.

use nexcore_error::{Result, nexerror};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

/// Options for text generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenerationOptions {
    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,
    /// Temperature for sampling (0.0-1.0).
    pub temperature: Option<f32>,
    /// Top-p nucleus sampling threshold.
    pub top_p: Option<f32>,
}

/// Trait for AI model clients.
#[async_trait]
pub trait ModelClient: Send + Sync {
    /// Generate text from a prompt.
    async fn generate(&self, prompt: &str, options: Option<GenerationOptions>) -> Result<String>;
    /// Get the model name.
    fn model_name(&self) -> &str;
}

/// Anthropic Claude API client.
pub struct ClaudeClient {
    client: Client,
    api_key: String,
    model: String,
}

impl ClaudeClient {
    /// Create a new Claude client.
    ///
    /// # Errors
    /// Returns error if ANTHROPIC_API_KEY is not set.
    pub fn new(api_key: Option<String>, model: &str) -> Result<Self> {
        let key = api_key
            .or_else(|| env::var("ANTHROPIC_API_KEY").ok())
            .or_else(|| env::var("CLAUDE_API_KEY").ok())
            .ok_or_else(|| nexerror!("ANTHROPIC_API_KEY not set"))?;

        Ok(Self {
            client: Client::new(),
            api_key: key,
            model: model.to_string(),
        })
    }
}

#[async_trait]
impl ModelClient for ClaudeClient {
    fn model_name(&self) -> &str {
        &self.model
    }

    async fn generate(&self, prompt: &str, options: Option<GenerationOptions>) -> Result<String> {
        let options = options.unwrap_or_default();
        let payload = json!({
            "model": self.model,
            "max_tokens": options.max_tokens.unwrap_or(4096),
            "messages": [{"role": "user", "content": prompt}]
        });

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;

        let text = resp["content"][0]["text"]
            .as_str()
            .ok_or_else(|| nexerror!("Invalid response format from Claude"))?;

        Ok(text.to_string())
    }
}

/// Google Gemini API client.
pub struct GeminiClient {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiClient {
    /// Create a new Gemini client.
    ///
    /// # Errors
    /// Returns error if GEMINI_API_KEY is not set.
    pub fn new(api_key: Option<String>, model: &str) -> Result<Self> {
        let key = api_key
            .or_else(|| env::var("GEMINI_API_KEY").ok())
            .ok_or_else(|| nexerror!("GEMINI_API_KEY not set"))?;

        Ok(Self {
            client: Client::new(),
            api_key: key,
            model: model.to_string(),
        })
    }
}

#[async_trait]
impl ModelClient for GeminiClient {
    fn model_name(&self) -> &str {
        &self.model
    }

    async fn generate(&self, prompt: &str, _options: Option<GenerationOptions>) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let payload = json!({
            "contents": [{"parts": [{"text": prompt}]}]
        });

        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;

        let text = resp["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| nexerror!("Invalid response format from Gemini"))?;

        Ok(text.to_string())
    }
}
