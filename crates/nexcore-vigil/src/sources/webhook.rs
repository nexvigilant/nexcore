use crate::events::EventBus;
use crate::models::Event;
use crate::sources::Source;
use async_trait::async_trait;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use nexcore_chrono::DateTime;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
use tracing::{info, warn};

/// LLM usage statistics tracked per session
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LLMStats {
    /// Total number of LLM calls this session
    pub total_calls: u64,
    /// Total tokens used (input + output)
    pub total_tokens: u64,
    /// Total input tokens
    pub input_tokens: u64,
    /// Total output tokens
    pub output_tokens: u64,
    /// Session start time
    pub session_start: Option<DateTime>,
    /// Last call timestamp
    pub last_call: Option<DateTime>,
    /// LLM provider name
    pub provider: String,
    /// Model name
    pub model: String,
}

impl LLMStats {
    /// Create new stats for a provider/model
    pub fn new(provider: &str, model: &str) -> Self {
        Self {
            provider: provider.to_string(),
            model: model.to_string(),
            session_start: Some(DateTime::now()),
            ..Default::default()
        }
    }

    /// Record a call with token usage
    pub fn record_call(&mut self, tokens: u64) {
        self.total_calls += 1;
        self.total_tokens += tokens;
        self.last_call = Some(DateTime::now());
    }

    /// Get average tokens per call
    pub fn avg_tokens_per_call(&self) -> f64 {
        if self.total_calls == 0 {
            0.0
        } else {
            self.total_tokens as f64 / self.total_calls as f64
        }
    }
}

/// Global LLM stats (shared between orchestrator and webhook)
static LLM_STATS: OnceLock<Arc<RwLock<LLMStats>>> = OnceLock::new();

/// Get or initialize global LLM stats
pub fn get_llm_stats() -> &'static Arc<RwLock<LLMStats>> {
    LLM_STATS.get_or_init(|| Arc::new(RwLock::new(LLMStats::default())))
}

pub struct WebhookSource {
    bus: EventBus,
    port: u16,
    api_key: String,
}

impl WebhookSource {
    pub fn new(bus: EventBus, port: u16, api_key: String) -> Self {
        Self { bus, port, api_key }
    }
}

#[async_trait]
impl Source for WebhookSource {
    fn name(&self) -> &'static str {
        "webhook"
    }

    async fn run(&self) -> nexcore_error::Result<()> {
        // Webhook needs external access by design — configurable via WEBHOOK_BIND_ADDR
        let bind_host = std::env::var("WEBHOOK_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".into());
        let addr = format!("{}:{}", bind_host, self.port);
        info!(addr = %addr, "webhook_source_starting");

        let state = WebhookState {
            bus: self.bus.clone(),
            api_key: self.api_key.clone(),
        };

        let app = Router::new()
            .route("/webhook", post(handle_webhook))
            .route("/stats", get(handle_stats))
            .route("/health", get(handle_health))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

#[derive(Clone)]
struct WebhookState {
    bus: EventBus,
    api_key: String,
}

/// Constant-time comparison to prevent timing attacks on API key validation.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

async fn handle_webhook(
    State(state): State<WebhookState>,
    headers: HeaderMap,
    Json(event): Json<Event>,
) -> Result<&'static str, StatusCode> {
    let auth_header = headers.get("x-api-key").and_then(|h| h.to_str().ok());

    match auth_header {
        Some(provided) if constant_time_eq(provided.as_bytes(), state.api_key.as_bytes()) => {
            state.bus.emit(event).await;
            Ok("OK")
        }
        _ => {
            warn!("unauthorized_webhook_attempt");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// Returns current LLM usage statistics as JSON
async fn handle_stats() -> Json<LLMStats> {
    let stats = get_llm_stats().read().clone();
    Json(stats)
}

/// Simple health check endpoint
async fn handle_health() -> &'static str {
    "OK"
}
