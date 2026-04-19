//! vigil-nexdev — autonomous Vigil daemon.
//!
//! Single-binary HTTP service that wraps the headless Claude Code CLI on nexdev.
//! Endpoints:
//!   GET  /health       — status, scope, uptime
//!   POST /prompt       — one-shot prompt, streams stream-json as Server-Sent Events
//!   POST /wake         — one-shot prompt, blocks until complete, returns final text
//!
//! Scope: UNBOUNDED. The VM itself is the sandbox. All actions audited via
//! journald + brain autopsy records after the fact.

use std::{
    process::Stdio,
    sync::Arc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Sse, sse::Event},
    routing::{get, post},
};
use futures_util::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    sync::Mutex,
};
use tokio_stream::wrappers::LinesStream;
use tracing::{error, info, warn};

const BIND_ADDR: &str = "127.0.0.1:7823";
const CLAUDE_BIN: &str = "/usr/bin/claude";
const DEFAULT_MODEL: &str = "claude-sonnet-4-6";
const DEFAULT_CWD: &str = "/home/matthew/Projects/Active/nucleus/workspaces/nexcore";
const SCOPE: &str = "unbounded";

// -- State -------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    anthropic_key: Arc<String>,
    /// Sovereign inference base URL if configured (e.g. "http://127.0.0.1:8090").
    /// When set, gets forwarded to the claude CLI as ANTHROPIC_BASE_URL so the
    /// model call never leaves the box.
    anthropic_base_url: Arc<Option<String>>,
    started_at: Instant,
    last_prompt_epoch: Arc<Mutex<Option<u64>>>,
}

impl AppState {
    async fn touch(&self) {
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        *self.last_prompt_epoch.lock().await = Some(epoch);
    }
}

// -- Requests / responses ----------------------------------------------------

#[derive(Deserialize)]
struct PromptReq {
    prompt: String,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
}

#[derive(Deserialize)]
struct WakeReq {
    source: String,
    prompt: String,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
}

#[derive(Serialize)]
struct HealthResp {
    status: &'static str,
    scope: &'static str,
    claude_bin: &'static str,
    default_model: &'static str,
    default_cwd: &'static str,
    uptime_secs: u64,
    last_prompt_epoch: Option<u64>,
}

#[derive(Serialize)]
struct WakeResp {
    source: String,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

// -- Key fetch (startup only) ------------------------------------------------

fn fetch_anthropic_key() -> Result<String, String> {
    if let Ok(k) = std::env::var("ANTHROPIC_API_KEY") {
        if !k.is_empty() {
            info!("ANTHROPIC_API_KEY loaded from env (len={})", k.len());
            return Ok(k);
        }
    }
    let out = std::process::Command::new("/usr/bin/gcloud")
        .args([
            "secrets",
            "versions",
            "access",
            "latest",
            "--secret=anthropic-api-key",
        ])
        .output()
        .map_err(|e| format!("gcloud spawn: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "gcloud exit={}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let key = String::from_utf8(out.stdout)
        .map_err(|e| format!("secret utf8: {e}"))?
        .trim()
        .to_string();
    if !key.starts_with("sk-ant-") {
        return Err(format!(
            "secret did not look like an Anthropic key (len={})",
            key.len()
        ));
    }
    info!(
        "ANTHROPIC_API_KEY loaded from Secret Manager (len={})",
        key.len()
    );
    Ok(key)
}

// -- Claude invocation -------------------------------------------------------

/// Build a Claude Code child command with unbounded posture.
///
/// `stream_json` controls the output surface:
///   - true  → stream-json events, one per line (for SSE forwarding)
///   - false → plain text (for /wake blocking call)
fn claude_cmd(
    key: &str,
    base_url: Option<&str>,
    model: &str,
    cwd: &str,
    prompt: &str,
    stream_json: bool,
) -> Command {
    let mut cmd = Command::new(CLAUDE_BIN);
    cmd.current_dir(cwd)
        .env("ANTHROPIC_API_KEY", key)
        .env("CLAUDECODE", "1");
    if let Some(url) = base_url {
        // Route through sovereign inference (nexcore-vigil-train serve).
        cmd.env("ANTHROPIC_BASE_URL", url);
    }
    cmd.arg("--print")
        .arg("--dangerously-skip-permissions")
        .arg("--model")
        .arg(model);
    if stream_json {
        cmd.arg("--output-format")
            .arg("stream-json")
            .arg("--verbose");
    }
    cmd.arg(prompt)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}

// -- Handlers ----------------------------------------------------------------

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let last = *state.last_prompt_epoch.lock().await;
    Json(HealthResp {
        status: "ok",
        scope: SCOPE,
        claude_bin: CLAUDE_BIN,
        default_model: DEFAULT_MODEL,
        default_cwd: DEFAULT_CWD,
        uptime_secs: state.started_at.elapsed().as_secs(),
        last_prompt_epoch: last,
    })
}

async fn prompt(
    State(state): State<AppState>,
    Json(req): Json<PromptReq>,
) -> Result<Sse<impl Stream<Item = Result<Event, std::io::Error>>>, (StatusCode, String)> {
    state.touch().await;

    let model = req.model.as_deref().unwrap_or(DEFAULT_MODEL);
    let cwd = req.cwd.as_deref().unwrap_or(DEFAULT_CWD);

    info!(
        model,
        cwd,
        prompt_len = req.prompt.len(),
        "Spawning claude (SSE)"
    );

    let mut child = claude_cmd(
        &state.anthropic_key,
        state.anthropic_base_url.as_deref(),
        model,
        cwd,
        &req.prompt,
        true,
    )
    .spawn()
    .map_err(|e| {
        error!(error = %e, "claude spawn failed");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("claude spawn: {e}"),
        )
    })?;

    let stdout = child.stdout.take().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "claude stdout missing".to_string(),
        )
    })?;

    let lines = BufReader::new(stdout).lines();
    let stream = LinesStream::new(lines).map(|res| {
        res.map(|line| {
            // Each line is a JSON event from stream-json; forward as SSE data.
            Event::default().data(line)
        })
    });

    // Best-effort reap in the background.
    tokio::spawn(async move {
        match child.wait().await {
            Ok(status) => info!(?status, "claude (SSE) exited"),
            Err(e) => warn!(error = %e, "claude wait failed"),
        }
    });

    Ok(Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default()))
}

async fn wake(
    State(state): State<AppState>,
    Json(req): Json<WakeReq>,
) -> Result<Json<WakeResp>, (StatusCode, String)> {
    state.touch().await;

    let model = req.model.as_deref().unwrap_or(DEFAULT_MODEL);
    let cwd = req.cwd.as_deref().unwrap_or(DEFAULT_CWD);

    info!(
        source = %req.source,
        model,
        cwd,
        prompt_len = req.prompt.len(),
        "Wake invoked"
    );

    let mut child = claude_cmd(
        &state.anthropic_key,
        state.anthropic_base_url.as_deref(),
        model,
        cwd,
        &req.prompt,
        false,
    )
    .spawn()
    .map_err(|e| {
        error!(error = %e, "claude spawn failed");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("claude spawn: {e}"),
        )
    })?;

    // Collect stdout + stderr to completion.
    let mut stdout_buf = Vec::new();
    let mut stderr_buf = Vec::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = tokio::io::AsyncReadExt::read_to_end(&mut out, &mut stdout_buf).await;
    }
    if let Some(mut err) = child.stderr.take() {
        let _ = tokio::io::AsyncReadExt::read_to_end(&mut err, &mut stderr_buf).await;
    }
    let status = child
        .wait()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("wait: {e}")))?;

    Ok(Json(WakeResp {
        source: req.source,
        exit_code: status.code(),
        stdout: String::from_utf8_lossy(&stdout_buf).to_string(),
        stderr: String::from_utf8_lossy(&stderr_buf).to_string(),
    }))
}

// Silence unused-import lint when AsyncWriteExt ever becomes useful again.
#[allow(dead_code)]
async fn _flush_stdin<W: tokio::io::AsyncWrite + Unpin>(mut w: W) -> std::io::Result<()> {
    w.flush().await
}

// -- Direct sovereign path (bypasses claude CLI) -----------------------------

#[derive(Deserialize)]
struct SovereignReq {
    prompt: String,
    #[serde(default = "default_max_tokens")]
    max_tokens: usize,
    #[serde(default)]
    system: Option<String>,
}

#[derive(Serialize)]
struct SovereignResp {
    text: String,
    model: String,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    latency_ms: u128,
}

fn default_max_tokens() -> usize {
    512
}

/// POST /sovereign — bypass the claude CLI entirely. Forwards directly to the
/// Anthropic-compatible /v1/messages endpoint configured via ANTHROPIC_BASE_URL.
///
/// This is the autonomous-agent surface: no harness, no interactive TUI, no
/// Node.js, no external auth. Pure Rust-to-Rust HTTP to the sovereign backend.
async fn sovereign(
    State(state): State<AppState>,
    Json(req): Json<SovereignReq>,
) -> Result<Json<SovereignResp>, (StatusCode, String)> {
    state.touch().await;

    let base = state
        .anthropic_base_url
        .as_ref()
        .as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "no ANTHROPIC_BASE_URL configured — sovereign path requires local serve".into(),
            )
        })?
        .clone();

    let url = format!("{}/v1/messages", base.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": "vigil-qwen-v1",
        "max_tokens": req.max_tokens,
        "system": req.system.unwrap_or_default(),
        "messages": [{"role": "user", "content": req.prompt}]
    });

    let started = Instant::now();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("http client: {e}"),
            )
        })?;

    let resp = client.post(&url).json(&body).send().await.map_err(|e| {
        error!(error = %e, url = %url, "sovereign POST failed");
        (StatusCode::BAD_GATEWAY, format!("sovereign POST: {e}"))
    })?;

    let status = resp.status();
    let json: serde_json::Value = resp.json().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("sovereign response parse: {e}"),
        )
    })?;

    if !status.is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("sovereign {status}: {json}"),
        ));
    }

    let text = json
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|b| b.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    let input_tokens = json
        .get("usage")
        .and_then(|u| u.get("input_tokens"))
        .and_then(|v| v.as_u64());
    let output_tokens = json
        .get("usage")
        .and_then(|u| u.get("output_tokens"))
        .and_then(|v| v.as_u64());
    let model = json
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("vigil-qwen-v1")
        .to_string();

    Ok(Json(SovereignResp {
        text,
        model,
        input_tokens,
        output_tokens,
        latency_ms: started.elapsed().as_millis(),
    }))
}

// -- Entrypoint --------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("vigil_nexdev=info")),
        )
        .init();

    let key = fetch_anthropic_key().map_err(|e| format!("key fetch failed: {e}"))?;
    let base_url = std::env::var("ANTHROPIC_BASE_URL")
        .ok()
        .filter(|s| !s.is_empty());
    if let Some(ref u) = base_url {
        info!(base_url = %u, "Sovereign inference endpoint configured — routing via ANTHROPIC_BASE_URL");
    }
    let state = AppState {
        anthropic_key: Arc::new(key),
        anthropic_base_url: Arc::new(base_url),
        started_at: Instant::now(),
        last_prompt_epoch: Arc::new(Mutex::new(None)),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/prompt", post(prompt))
        .route("/wake", post(wake))
        .route("/sovereign", post(sovereign))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(BIND_ADDR).await?;
    info!(addr = %listener.local_addr()?, scope = SCOPE, "vigil-nexdev listening");
    axum::serve(listener, app).await?;
    Ok(())
}
