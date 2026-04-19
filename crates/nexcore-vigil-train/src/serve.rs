//! Claude-compatible HTTP inference server — Phase 1 of the Vigil API.
//!
//! Mirrors Anthropic's `POST /v1/messages` wire protocol so existing
//! `anthropic` SDK users can point at `api.vigil.nexvigilant.com` (or
//! `localhost:8080`) and their code works unchanged.
//!
//! Endpoints shipped:
//!
//! | Method | Path | Purpose |
//! |--------|------|---------|
//! | GET | `/health` | Liveness probe (Cloud Run compatible) |
//! | GET | `/v1/models` | Lists available model IDs |
//! | POST | `/v1/messages` | Generate a completion (Claude-compatible) |
//!
//! **Concurrency:** a single global model is shared behind a `tokio::sync::Mutex`.
//! Requests serialize at the model boundary. This is intentional for Phase 1 —
//! pharma PV inference is bursty and a queued single-worker keeps the server
//! memory-bounded on Cloud Run's ~2-4 GB CPU instance.
//!
//! **Auth:** not yet — Phase 3 adds API keys. Currently accepts any request
//! to allow local testing + SDK-drop-in validation before Cloud Run deploy.

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use candle_transformers::models::qwen2::ModelForCausalLM as NativeQwen2;
use nexcore_error::{NexError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokenizers::Tokenizer;
use tokio::sync::Mutex;

use crate::inference::{self, SamplingConfig};
use crate::model;
use crate::tokenizer as tk;

// ─────────────────────────────────────────────────────────────────────────────
// Server config
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ServeConfig {
    pub bind: String,
    pub model_dir: PathBuf,
    pub tokenizer_path: Option<PathBuf>,
    pub dtype: model::NativeDType,
    pub model_id: String,
}

impl Default for ServeConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".into());
        let merged = PathBuf::from(format!("{home}/.claude/brain/vigil-qwen-v1"));
        let base = PathBuf::from(format!("{home}/.claude/brain/vigil-base-v1"));
        let chosen = if merged.join("model.safetensors").is_file() {
            merged
        } else {
            base
        };
        Self {
            bind: "127.0.0.1:8080".into(),
            model_dir: chosen,
            tokenizer_path: None,
            dtype: model::NativeDType::F32,
            model_id: "vigil-qwen-v1".into(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Wire types — matches Anthropic's /v1/messages request/response
// ─────────────────────────────────────────────────────────────────────────────

/// Anthropic-compatible message request.
///
/// Spec: <https://docs.anthropic.com/en/api/messages>
#[derive(Debug, Deserialize)]
struct MessagesRequest {
    #[serde(default = "default_model")]
    model: String,
    #[serde(default = "default_max_tokens")]
    max_tokens: usize,
    #[serde(default)]
    system: Option<SystemField>,
    messages: Vec<WireMessage>,
    #[serde(default = "default_temperature")]
    temperature: f64,
    #[serde(default)]
    seed: Option<u64>,
    #[serde(default)]
    stream: bool,
}

/// The system prompt field in an Anthropic `/v1/messages` request accepts
/// either a plain string or an array of content blocks (with text blocks).
/// This untagged enum handles both; `into_string` concatenates block text.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SystemField {
    Text(String),
    Blocks(Vec<SystemBlock>),
}

#[derive(Debug, Deserialize)]
struct SystemBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(default)]
    text: String,
}

impl SystemField {
    fn into_string(self) -> String {
        match self {
            Self::Text(s) => s,
            Self::Blocks(blocks) => blocks
                .into_iter()
                .filter(|b| b.block_type == "text")
                .map(|b| b.text)
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

fn default_model() -> String {
    "vigil-qwen-v1".into()
}

fn default_max_tokens() -> usize {
    1024
}

fn default_temperature() -> f64 {
    0.7
}

#[derive(Debug, Deserialize)]
struct WireMessage {
    role: String,
    content: WireContent,
}

/// Content may be a bare string OR an array of content blocks (per Anthropic spec).
/// We handle both but only extract text blocks.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum WireContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

impl WireContent {
    fn as_text(&self) -> String {
        match self {
            WireContent::Text(s) => s.clone(),
            WireContent::Blocks(blocks) => blocks
                .iter()
                .filter(|b| b.block_type == "text")
                .filter_map(|b| b.text.clone())
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

/// Anthropic-compatible response.
#[derive(Debug, Serialize)]
struct MessagesResponse {
    id: String,
    #[serde(rename = "type")]
    msg_type: &'static str,
    role: &'static str,
    content: Vec<ResponseBlock>,
    model: String,
    stop_reason: &'static str,
    stop_sequence: Option<String>,
    usage: Usage,
}

#[derive(Debug, Serialize)]
struct ResponseBlock {
    #[serde(rename = "type")]
    block_type: &'static str,
    text: String,
}

#[derive(Debug, Serialize)]
struct Usage {
    input_tokens: usize,
    output_tokens: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// Models list response (mirrors Anthropic's GET /v1/models)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct ModelsListResponse {
    data: Vec<ModelCard>,
    has_more: bool,
    first_id: Option<String>,
    last_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct ModelCard {
    #[serde(rename = "type")]
    card_type: &'static str,
    id: String,
    display_name: String,
    created_at: &'static str,
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared state
// ─────────────────────────────────────────────────────────────────────────────

struct AppState {
    model: Mutex<NativeQwen2>,
    tokenizer: Tokenizer,
    device: candle_core::Device,
    model_id: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "vigil-api",
        "backend": "nexcore-vigil-train (Candle, pure Rust)",
    }))
}

async fn models_list(State(state): State<Arc<AppState>>) -> Json<ModelsListResponse> {
    let card = ModelCard {
        card_type: "model",
        id: state.model_id.clone(),
        display_name: "Vigil Qwen v1 (pharmacovigilance-specialized)".into(),
        created_at: "2026-04-17T00:00:00Z",
    };
    let first_id = Some(card.id.clone());
    let last_id = first_id.clone();
    Json(ModelsListResponse {
        data: vec![card],
        has_more: false,
        first_id,
        last_id,
    })
}

async fn messages(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MessagesRequest>,
) -> std::result::Result<Json<MessagesResponse>, (StatusCode, Json<serde_json::Value>)> {
    if req.stream {
        return Err((
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({
                "type": "error",
                "error": {
                    "type": "feature_not_implemented",
                    "message": "Streaming (stream: true) is not yet implemented in Phase 1. Pass stream: false (default)."
                }
            })),
        ));
    }
    if req.messages.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "type": "error",
                "error": {"type": "invalid_request_error", "message": "messages array is empty"}
            })),
        ));
    }

    // Build ChatML by flattening the messages array. For Phase 1 we support
    // system + one user turn; multi-turn will compose full conversation history.
    let system = req.system.map(SystemField::into_string).unwrap_or_default();
    let user = req
        .messages
        .iter()
        .filter(|m| m.role == "user")
        .map(|m| m.content.as_text())
        .collect::<Vec<_>>()
        .join("\n\n");
    if user.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "type": "error",
                "error": {"type": "invalid_request_error", "message": "no user message content found"}
            })),
        ));
    }

    let chatml = tk::format_chatml(&system, &user);

    // Count input tokens (approximate — includes system + chatml template overhead).
    let input_tokens = tk::encode(&state.tokenizer, &chatml)
        .map(|ids| ids.len())
        .unwrap_or(0);

    let cfg = SamplingConfig {
        temperature: req.temperature,
        top_p: Some(0.9),
        seed: req.seed.unwrap_or(42),
        max_new_tokens: req.max_tokens,
    };

    // Serialize access to the model — one inference at a time per server.
    let mut guard = state.model.lock().await;
    inference::clear_native_cache(&mut guard);
    let result =
        inference::generate_native(&mut guard, &state.tokenizer, &state.device, &chatml, &cfg);
    drop(guard);

    match result {
        Ok((text, n_tok)) => {
            let id = format!("msg_{}", chrono_timestamp());
            Ok(Json(MessagesResponse {
                id,
                msg_type: "message",
                role: "assistant",
                content: vec![ResponseBlock {
                    block_type: "text",
                    text,
                }],
                model: req.model,
                stop_reason: "end_turn",
                stop_sequence: None,
                usage: Usage {
                    input_tokens,
                    output_tokens: n_tok,
                },
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "type": "error",
                "error": {"type": "api_error", "message": e.to_string()}
            })),
        )),
    }
}

fn chrono_timestamp() -> String {
    // Simple unix-nanos-based message ID — avoids pulling in chrono.
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| format!("{:x}", d.as_nanos()))
        .unwrap_or_else(|_| "0".into())
}

// ─────────────────────────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────────────────────────

pub fn run(cfg: &ServeConfig) -> Result<()> {
    // Load model + tokenizer once (synchronous — runs before the tokio runtime).
    tracing::info!(
        "[vigil-serve] loading model from {}",
        cfg.model_dir.display()
    );
    let device = model::pick_device()?;
    let tok = tk::load(cfg.tokenizer_path.as_deref())?;
    let loaded = model::load_native(&cfg.model_dir, cfg.dtype, &device)?;
    let model_inst = match loaded {
        model::VigilModel::Native(m) => m,
        model::VigilModel::Quantized(_) => {
            return Err(NexError::new("serve expects native model, got quantized"));
        }
    };

    let state = Arc::new(AppState {
        model: Mutex::new(model_inst),
        tokenizer: tok,
        device,
        model_id: cfg.model_id.clone(),
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/models", get(models_list))
        .route("/v1/messages", post(messages))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let bind = cfg.bind.clone();

    // Build a single-threaded tokio runtime (serves fine for mutex-bounded inference).
    let rt =
        tokio::runtime::Runtime::new().map_err(|e| NexError::new(format!("tokio runtime: {e}")))?;

    rt.block_on(async move {
        let listener = tokio::net::TcpListener::bind(&bind)
            .await
            .map_err(|e| NexError::new(format!("bind {bind}: {e}")))?;
        tracing::info!("[vigil-serve] listening on http://{bind}");
        tracing::info!("[vigil-serve] try: curl http://{bind}/health");
        tracing::info!("[vigil-serve] SDK drop-in: anthropic.Anthropic(base_url='http://{bind}', api_key='test')");
        axum::serve(listener, app)
            .await
            .map_err(|e| NexError::new(format!("axum serve: {e}")))?;
        Ok::<(), NexError>(())
    })?;

    Ok(())
}
