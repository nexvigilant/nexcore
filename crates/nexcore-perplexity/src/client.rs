// Copyright 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Async HTTP client for the Perplexity AI Sonar API.
//!
//! ## Primitive Grounding
//!
//! `PerplexityClient`: T2-C (mu Mapping + boundary + state)
//! Dominant: mu -- the client maps queries to search-grounded responses.
//!
//! ## API Compatibility
//!
//! Perplexity uses an OpenAI-compatible chat completions endpoint:
//! `POST https://api.perplexity.ai/chat/completions`
//! with Bearer token authentication.

use crate::error::{PerplexityError, PerplexityResult};
use crate::types::{ChatRequest, ChatResponse, Model};
use std::env;
use std::time::Duration;
use tracing::debug;

/// Perplexity AI chat completions endpoint.
const API_URL: &str = "https://api.perplexity.ai/chat/completions";

/// Default request timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Deep research timeout -- longer because it runs multi-step agent.
const DEEP_RESEARCH_TIMEOUT_SECS: u64 = 120;

/// The Perplexity AI API client.
///
/// Tier: T2-C (mu Mapping + boundary + state)
/// Dominant: mu -- maps queries to grounded search responses.
///
/// ## Usage
///
/// ```rust,no_run
/// # use nexcore_perplexity::client::PerplexityClient;
/// # use nexcore_perplexity::types::Model;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = PerplexityClient::from_env()?;
/// let response = client.chat(
///     nexcore_perplexity::types::ChatRequest::simple(Model::Sonar, "What is Rust?")
/// ).await?;
/// println!("{}", response.text());
/// # Ok(())
/// # }
/// ```
pub struct PerplexityClient {
    /// HTTP client with configured timeout.
    http: reqwest::Client,
    /// API key for Bearer authentication.
    api_key: String,
    /// Default model for requests.
    default_model: Model,
}

impl PerplexityClient {
    /// Create a client from `PERPLEXITY_API_KEY` environment variable.
    ///
    /// Returns `PerplexityError::MissingApiKey` if the variable is not set.
    pub fn from_env() -> PerplexityResult<Self> {
        let api_key = env::var("PERPLEXITY_API_KEY").map_err(|_| PerplexityError::MissingApiKey)?;

        Self::new(api_key, Model::default())
    }

    /// Create a client with an explicit API key and default model.
    pub fn new(api_key: String, default_model: Model) -> PerplexityResult<Self> {
        if api_key.is_empty() {
            return Err(PerplexityError::MissingApiKey);
        }

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .map_err(PerplexityError::Http)?;

        Ok(Self {
            http,
            api_key,
            default_model,
        })
    }

    /// Get the default model.
    pub fn default_model(&self) -> Model {
        self.default_model
    }

    /// Send a chat completion request to the Perplexity API.
    ///
    /// This is the core API call -- all higher-level methods delegate here.
    pub async fn chat(&self, request: ChatRequest) -> PerplexityResult<ChatResponse> {
        let is_deep = request.model == "sonar-deep-research";

        // Deep research needs a longer timeout
        let client = if is_deep {
            reqwest::Client::builder()
                .timeout(Duration::from_secs(DEEP_RESEARCH_TIMEOUT_SECS))
                .build()
                .map_err(PerplexityError::Http)?
        } else {
            self.http.clone()
        };

        debug!(
            model = %request.model,
            messages = request.messages.len(),
            "Sending Perplexity API request"
        );

        let response = client
            .post(API_URL)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(PerplexityError::Http)?;

        let status = response.status();

        // Handle rate limiting
        if status.as_u16() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60);

            return Err(PerplexityError::RateLimited {
                retry_after_secs: retry_after,
            });
        }

        // Handle other error statuses
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(PerplexityError::Api {
                status: status.as_u16(),
                message: body,
            });
        }

        let chat_response: ChatResponse = response.json().await.map_err(PerplexityError::Http)?;

        debug!(
            id = %chat_response.id,
            citations = chat_response.citations.len(),
            tokens = chat_response.usage.total_tokens,
            "Perplexity API response received"
        );

        Ok(chat_response)
    }

    /// Simple query with the default model.
    pub async fn query(&self, query: &str) -> PerplexityResult<ChatResponse> {
        let request = ChatRequest::simple(self.default_model, query);
        self.chat(request).await
    }

    /// Query with a specific model.
    pub async fn query_with_model(
        &self,
        query: &str,
        model: Model,
    ) -> PerplexityResult<ChatResponse> {
        let request = ChatRequest::simple(model, query);
        self.chat(request).await
    }
}

// ============================================================================
// Standalone helper (for MCP tool use without managing client lifetime)
// ============================================================================

/// Get the API key from environment, returning empty string if not set.
///
/// Used by MCP tool wrappers that need a fallback path.
pub fn get_api_key() -> String {
    env::var("PERPLEXITY_API_KEY").unwrap_or_default()
}

/// Create a one-shot HTTP client with the configured timeout.
pub fn http_client(timeout_secs: u64) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .unwrap_or_default()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_rejects_empty_api_key() {
        let result = PerplexityClient::new(String::new(), Model::Sonar);
        assert!(result.is_err());
        let err = result.err().expect("expected error for empty key");
        assert!(
            matches!(err, PerplexityError::MissingApiKey),
            "expected MissingApiKey, got: {err:?}"
        );
    }

    #[test]
    fn client_accepts_valid_key() {
        let result = PerplexityClient::new("pplx-test-key".to_string(), Model::SonarPro);
        assert!(result.is_ok());
        let client = result.expect("valid key should create client");
        assert_eq!(client.default_model(), Model::SonarPro);
    }

    #[test]
    fn get_api_key_returns_empty_when_unset() {
        // In test environment, the key is typically not set
        // This just verifies the function doesn't panic
        let _key = get_api_key();
    }

    #[test]
    fn http_client_creates_successfully() {
        let client = http_client(30);
        // Verify client was created (no panic)
        drop(client);
    }
}
