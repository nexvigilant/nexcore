use crate::config::WebMcpConfig;
use crate::registry::Registry;

/// Hub sync client — bidirectional sync with webmcp-hub.com.
///
/// NexCore is the source of truth. The hub is the distribution mirror.
/// Sync pushes local configs to the hub and pulls any changes back.
pub struct HubClient {
    base_url: String,
    api_key: String,
    client: reqwest::Client,
}

impl HubClient {
    /// Create a new hub client.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            base_url: "https://www.webmcp-hub.com".into(),
            api_key: api_key.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Create a hub client with a custom base URL (for testing).
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Verify the API key is valid.
    pub async fn verify(&self) -> Result<String, HubError> {
        let resp = self.client
            .get(format!("{}/api/me", self.base_url))
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map_err(|e| HubError::Network(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(HubError::Unauthorized);
        }
        if !resp.status().is_success() {
            return Err(HubError::ApiError(resp.status().as_u16(), "verify failed".into()));
        }

        let body: serde_json::Value = resp.json().await
            .map_err(|e| HubError::Parse(e.to_string()))?;

        body.get("username")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| HubError::Parse("no username in response".into()))
    }

    /// Pull configs for a domain from the hub.
    pub async fn pull_domain(&self, domain: &str) -> Result<Vec<WebMcpConfig>, HubError> {
        let resp = self.client
            .get(format!("{}/api/configs/lookup", self.base_url))
            .query(&[("domain", domain), ("executable", "true")])
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map_err(|e| HubError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(HubError::ApiError(resp.status().as_u16(), format!("pull {domain}")));
        }

        let body: serde_json::Value = resp.json().await
            .map_err(|e| HubError::Parse(e.to_string()))?;

        let configs_val = body.get("configs").cloned().unwrap_or(serde_json::Value::Array(vec![]));
        let configs: Vec<WebMcpConfig> = serde_json::from_value(configs_val)
            .map_err(|e| HubError::Parse(e.to_string()))?;

        Ok(configs)
    }

    /// Push a config to the hub. Returns the hub-assigned ID.
    pub async fn push_config(&self, config: &WebMcpConfig) -> Result<String, HubError> {
        let resp = self.client
            .post(format!("{}/api/configs", self.base_url))
            .bearer_auth(&self.api_key)
            .json(config)
            .send()
            .await
            .map_err(|e| HubError::Network(e.to_string()))?;

        let status = resp.status().as_u16();
        let body: serde_json::Value = resp.json().await
            .map_err(|e| HubError::Parse(e.to_string()))?;

        match status {
            201 => body.get("id")
                .and_then(|v| v.as_str())
                .map(String::from)
                .ok_or_else(|| HubError::Parse("no id in response".into())),
            409 => {
                // Already exists — return existing ID
                let existing = body.get("existingId")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_default();
                Err(HubError::AlreadyExists(existing))
            }
            _ => Err(HubError::ApiError(status, format!("{body}"))),
        }
    }

    /// Upvote a tool within a config.
    pub async fn upvote_tool(&self, config_id: &str, tool_name: &str) -> Result<(), HubError> {
        let resp = self.client
            .post(format!("{}/api/configs/{config_id}/vote", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({"toolName": tool_name, "vote": 1}))
            .send()
            .await
            .map_err(|e| HubError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(HubError::ApiError(resp.status().as_u16(), format!("vote {tool_name}")));
        }
        Ok(())
    }

    /// Sync a local registry to the hub. Pushes new configs, skips existing.
    /// Returns (pushed, skipped, errors).
    pub async fn sync_push(&self, registry: &Registry) -> (usize, usize, Vec<String>) {
        let mut pushed = 0;
        let mut skipped = 0;
        let mut errors = Vec::new();

        for config in registry.iter() {
            match self.push_config(config).await {
                Ok(_) => pushed += 1,
                Err(HubError::AlreadyExists(_)) => skipped += 1,
                Err(e) => errors.push(format!("{}: {e}", config.title)),
            }
        }

        (pushed, skipped, errors)
    }

    /// Pull all configs for given domains into a registry.
    pub async fn sync_pull(&self, domains: &[&str]) -> Result<Registry, HubError> {
        let mut registry = Registry::new();
        for domain in domains {
            let configs = self.pull_domain(domain).await?;
            for config in configs {
                registry.insert(config);
            }
        }
        Ok(registry)
    }
}

/// Errors from hub operations.
#[derive(Debug)]
pub enum HubError {
    /// Network/transport error.
    Network(String),
    /// API returned unauthorized (401).
    Unauthorized,
    /// Config already exists on hub (409).
    AlreadyExists(String),
    /// API returned an error status.
    ApiError(u16, String),
    /// Response parsing failed.
    Parse(String),
}

impl std::fmt::Display for HubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "network error: {e}"),
            Self::Unauthorized => write!(f, "unauthorized — check API key"),
            Self::AlreadyExists(id) => write!(f, "config already exists: {id}"),
            Self::ApiError(status, msg) => write!(f, "API error {status}: {msg}"),
            Self::Parse(e) => write!(f, "parse error: {e}"),
        }
    }
}

impl std::error::Error for HubError {}
