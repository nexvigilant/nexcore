use crate::llm::LLMClient;
use crate::models::{Event, Interaction};
use async_trait::async_trait;
use nexcore_chrono::DateTime;
use reqwest::{Client, header};
use serde_json::json;
use tracing::info;

pub struct ClaudeClient {
    client: Client,
    model: String,
}

impl ClaudeClient {
    pub fn new(api_key: String, model: String) -> nexcore_error::Result<Self> {
        let mut headers = header::HeaderMap::new();
        let api_key_value = header::HeaderValue::from_str(&api_key)
            .map_err(|e| nexcore_error::nexerror!("invalid_api_key: {e}"))?;
        headers.insert("x-api-key", api_key_value);
        headers.insert(
            "anthropic-version",
            header::HeaderValue::from_static("2023-06-01"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| nexcore_error::nexerror!("client_build_fail: {e}"))?;

        Ok(Self { client, model })
    }
}

#[async_trait]
impl LLMClient for ClaudeClient {
    async fn invoke(&self, context: &str, event: &Event) -> nexcore_error::Result<Interaction> {
        info!(model = %self.model, "invoking_claude");
        let body = json!({
            "model": self.model,
            "max_tokens": 1024,
            "messages": [{"role": "user", "content": context}],
            "system": "You are FRIDAY. To take an action, use the exact format: [ACTION: type] payload [/ACTION]"
        });

        let ne = |e: reqwest::Error| nexcore_error::NexError::new(e.to_string());
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .json(&body)
            .send()
            .await
            .map_err(ne)?;
        if !response.status().is_success() {
            return Err(nexcore_error::nexerror!(
                "API error: {}",
                response.text().await.map_err(ne)?
            ));
        }

        let resp_json: serde_json::Value = response.json().await.map_err(ne)?;
        let text = resp_json["content"][0]["text"].as_str().unwrap_or("");

        Ok(Interaction {
            id: resp_json["id"].as_str().unwrap_or_default().to_string(),
            event: event.clone(),
            prompt: context.to_string(),
            response: text.to_string(),
            timestamp: DateTime::now(),
            tokens_used: resp_json["usage"]["total_tokens"].as_i64().unwrap_or(0) as i32,
            contains_learning: text.contains("[LEARNING]"),
            actions_taken: self.extract_actions(text),
        })
    }

    async fn health_check(&self) -> bool {
        true
    }
}

impl ClaudeClient {
    /// Extracts multi-line actions from LLM response.
    /// Supports [ACTION: type] multiline payload [/ACTION]
    fn extract_actions(&self, response: &str) -> Vec<String> {
        let re = match regex::Regex::new(
            r"(?s)\[ACTION:\s*(?P<type>[^\]]+)\](?P<payload>.*?)\[/ACTION\]",
        ) {
            Ok(re) => re,
            Err(err) => {
                tracing::warn!(error = %err, "action_regex_compile_failed");
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
