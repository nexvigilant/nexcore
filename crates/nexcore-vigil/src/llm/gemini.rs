use crate::llm::LLMClient;
use crate::models::{Event, Interaction};
use crate::sources::webhook::get_llm_stats;
use async_trait::async_trait;
use chrono::Utc;
use nexcore_vigilance::telemetry::{GeminiLogEntry, append_log};
use reqwest::Client;
use serde_json::json;
use std::time::Instant;
use tracing::info;

pub struct GeminiClient {
    client: Client,
    model: String,
    api_key: Option<String>,
    project_id: Option<String>,
    use_vertex: bool,
}

impl GeminiClient {
    /// Build a reusable HTTP client with sensible defaults
    fn build_client() -> Client {
        Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .pool_max_idle_per_host(10)
            .build()
            .unwrap_or_else(|_| Client::new())
    }

    /// Create with API key (Google AI Studio)
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Self::build_client(),
            model,
            api_key: Some(api_key),
            project_id: None,
            use_vertex: false,
        }
    }

    /// Create with Vertex AI (uses ADC - Application Default Credentials)
    pub fn vertex(project_id: String, model: String) -> Self {
        Self {
            client: Self::build_client(),
            model,
            api_key: None,
            project_id: Some(project_id),
            use_vertex: true,
        }
    }

    /// Get access token from GCloud ADC
    async fn get_access_token(&self) -> nexcore_error::Result<String> {
        let output = tokio::process::Command::new("gcloud")
            .args(["auth", "application-default", "print-access-token"])
            .output()
            .await?;

        if !output.status.success() {
            return Err(nexcore_error::nexerror!(
                "Failed to get access token: {:?}",
                output.stderr
            ));
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }
}

#[async_trait]
impl LLMClient for GeminiClient {
    async fn invoke(&self, context: &str, event: &Event) -> nexcore_error::Result<Interaction> {
        info!(model = %self.model, use_vertex = %self.use_vertex, "invoking_gemini");
        let start = Instant::now();

        let (url, auth_header) = if self.use_vertex {
            // Vertex AI endpoint - use global for Gemini 3+, us-central1 for older models
            let project = self
                .project_id
                .as_ref()
                .ok_or_else(|| nexcore_error::nexerror!("Project ID required for Vertex AI"))?;
            let token = self.get_access_token().await?;
            let location = if self.model.starts_with("gemini-3") {
                "global"
            } else {
                "us-central1"
            };
            let url = format!(
                "https://aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
                project, location, self.model
            );
            (url, format!("Bearer {}", token))
        } else {
            // Google AI Studio endpoint
            let key = self
                .api_key
                .as_ref()
                .ok_or_else(|| nexcore_error::nexerror!("API key required"))?;
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                self.model, key
            );
            (url, String::new())
        };

        let body = json!({
            "contents": [{
                "role": "user",
                "parts": [{"text": context}]
            }],
            "systemInstruction": {
                "parts": [{"text": "You are FRIDAY, an AI assistant with access to NexCore MCP tools. To take an action, use the exact format: [ACTION: type] payload [/ACTION]

Available actions:
- [ACTION: shell] command [/ACTION] - Execute shell command
- [ACTION: mcp] tool_name {\"params\": ...} [/ACTION] - Call MCP tool
- [ACTION: tts] message [/ACTION] - Text to speech
- [ACTION: notify] message [/ACTION] - Send notification"}]
            },
            "generationConfig": {
                "maxOutputTokens": 2048,
                "temperature": 0.7
            }
        });

        let request = if auth_header.is_empty() {
            self.client.post(&url).json(&body)
        } else {
            self.client
                .post(&url)
                .json(&body)
                .header("Authorization", &auth_header)
        };

        let response: reqwest::Response = request.send().await?;
        if !response.status().is_success() {
            return Err(nexcore_error::nexerror!("API error: {}", response.text().await?));
        }

        let resp_json: serde_json::Value = response.json().await?;

        // Extract text from Gemini response structure
        let text = resp_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("");

        // Token usage
        let tokens_used = resp_json["usageMetadata"]["totalTokenCount"]
            .as_i64()
            .unwrap_or(0) as i32;

        // Extract input/output tokens separately
        let input_tokens = resp_json["usageMetadata"]["promptTokenCount"]
            .as_u64()
            .unwrap_or(0);
        let output_tokens = resp_json["usageMetadata"]["candidatesTokenCount"]
            .as_u64()
            .unwrap_or(0);

        // Record stats in global tracker
        {
            let mut stats = get_llm_stats().write();
            if stats.provider.is_empty() {
                stats.provider = "gemini".to_string();
                stats.model = self.model.clone();
                stats.session_start = Some(Utc::now());
            }
            stats.total_calls += 1;
            stats.total_tokens += tokens_used as u64;
            stats.input_tokens += input_tokens;
            stats.output_tokens += output_tokens;
            stats.last_call = Some(Utc::now());
        }

        // Log to Watchtower telemetry
        let latency_ms = start.elapsed().as_millis() as u64;
        let session_id = format!("vigil-{}", Utc::now().timestamp_millis());
        let log_entry = GeminiLogEntry::success(
            session_id,
            "vigil",
            "invoke",
            &self.model,
            latency_ms,
            input_tokens,
            output_tokens,
        );
        if let Err(e) = append_log(&log_entry) {
            tracing::warn!("Failed to write Gemini telemetry: {}", e);
        }

        Ok(Interaction {
            id: format!("gemini-{}", Utc::now().timestamp_millis()),
            event: event.clone(),
            prompt: context.to_string(),
            response: text.to_string(),
            timestamp: Utc::now(),
            tokens_used,
            contains_learning: text.contains("[LEARNING]"),
            actions_taken: self.extract_actions(text),
        })
    }

    async fn health_check(&self) -> bool {
        true
    }
}

impl GeminiClient {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_actions() {
        let client = GeminiClient::new("dummy".to_string(), "gemini-3-flash-preview".to_string());

        let response = "I will check the weather.
[ACTION: weather]
location: New York
[/ACTION]
And then I will tell you.";

        let actions = client.extract_actions(response);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], "weather: location: New York");
    }

    #[test]
    fn test_extract_multiple_actions() {
        let client = GeminiClient::new("dummy".to_string(), "gemini-1.5-flash".to_string());

        let response = "First action:
[ACTION: shell]
ls -la
[/ACTION]
Second action:
[ACTION: tts]
Hello world
[/ACTION]";

        let actions = client.extract_actions(response);
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0], "shell: ls -la");
        assert_eq!(actions[1], "tts: Hello world");
    }
}
