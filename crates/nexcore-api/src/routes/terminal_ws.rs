//! Terminal WebSocket endpoint — multi-tenant, AI-augmented terminal sessions.
//!
//! Bridges the `nexcore-terminal` crate to WebSocket clients (xterm.js in Studio).
//! Each connection gets a session scoped to a tenant+user pair with per-tier
//! resource limits enforced by the `SessionRegistry`.
//!
//! ## Primitive Grounding
//!
//! `∂(Boundary) + ς(State) + σ(Sequence) + μ(Mapping) + →(Causality)`
//!
//! ## Protocol
//!
//! **Client → Server:** `WsClientMessage` (input, command, resize, mode_switch, ping, get_preferences, update_preference, get_layout, update_layout, get_keybindings, update_keybindings)
//! **Server → Client:** `WsServerMessage` (output, result, ai_token, status, error, pong, preferences, preference_updated, layout, keybindings)

use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;
use vr_core::ids::{TenantId, UserId};
use vr_core::tenant::SubscriptionTier;

#[cfg(not(feature = "mcp-bridge"))]
use crate::mcp_bridge::NexCoreMcpServer;
#[cfg(feature = "mcp-bridge")]
use nexcore_mcp::NexCoreMcpServer;

use nexcore_terminal::formatter::format_mcp_result;
use nexcore_terminal::keybindings::{InMemoryKeybindingsStore, KeybindingsStore};
use nexcore_terminal::layout::{InMemoryLayoutStore, LayoutStore};
use nexcore_terminal::microgram::{MicrogramConfig, run_microgram};
use nexcore_terminal::preferences::{InMemoryPreferencesStore, PreferencesStore};
use nexcore_terminal::protocol::{SessionStatusMsg, WsClientMessage, WsServerMessage};
use nexcore_terminal::pty::{PtyConfig, PtyProcess, PtySize};
use nexcore_terminal::registry::{RegistryError, SessionRegistry};
use nexcore_terminal::relay::{RelayConfig, query_relay};
use nexcore_terminal::router::{RoutedCommand, route_command};
use nexcore_terminal::session::{SessionStatus, TerminalMode, TerminalSession};
use nexcore_terminal::station::{StationConfig, call_station_tool};

use crate::ApiState;
use crate::mcp_bridge;
use crate::routes::ai_bridge::{AiMcpBridge, ToolScope};
use crate::routes::ai_client::{ClaudeClient, ClaudeConfig, StreamEvent};
use nexcore_terminal::conversation::ConversationContext;

// =============================================================================
// Shared State
// =============================================================================

/// Terminal subsystem state shared across all connections.
pub struct TerminalState {
    /// Session registry with per-tenant concurrency limits.
    pub registry: SessionRegistry,
    /// Per-user terminal preferences (in-memory, shared across reconnects).
    pub preferences: InMemoryPreferencesStore,
    /// Per-user terminal layout (in-memory, shared across reconnects).
    pub layouts: InMemoryLayoutStore,
    /// Per-user keybindings (in-memory, shared across reconnects).
    pub keybindings: InMemoryKeybindingsStore,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            registry: SessionRegistry::new(),
            preferences: InMemoryPreferencesStore::new(),
            layouts: InMemoryLayoutStore::new(),
            keybindings: InMemoryKeybindingsStore::new(),
        }
    }
}

/// Global terminal state, lazily initialized.
static TERMINAL_STATE: std::sync::OnceLock<Arc<TerminalState>> = std::sync::OnceLock::new();

/// Get or initialize the global terminal state.
fn get_terminal_state() -> Arc<TerminalState> {
    TERMINAL_STATE
        .get_or_init(|| Arc::new(TerminalState::default()))
        .clone()
}

// =============================================================================
// Query Parameters
// =============================================================================

/// Query parameters for the terminal WebSocket upgrade request.
///
/// The client passes identity via Firebase JWT token in the `token` query param
/// (browser WebSocket API cannot set custom headers). Falls back to explicit
/// tenant_id/user_id for development.
#[derive(Debug, Deserialize)]
pub struct TerminalConnectParams {
    /// Tenant identifier (UUID) — dev/fallback only.
    pub tenant_id: Option<String>,
    /// User identifier (UUID) — dev/fallback only.
    pub user_id: Option<String>,
    /// Firebase ID token (JWT) — production auth. Browser passes via query param
    /// because the WebSocket API does not support custom request headers.
    pub token: Option<String>,
    /// Initial terminal mode (defaults to "hybrid").
    pub mode: Option<String>,
    /// Subscription tier for resource limits (defaults to "explorer").
    pub tier: Option<String>,
}

// =============================================================================
// WebSocket Handler
// =============================================================================

/// WebSocket upgrade handler for the terminal endpoint.
///
/// Mounted at `/api/v1/terminal/ws`. Accepts a WebSocket upgrade request,
/// creates a session in the registry, and spawns the bidirectional I/O loop.
pub async fn ws_terminal_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<TerminalConnectParams>,
    State(_state): State<ApiState>,
) -> impl IntoResponse {
    let has_token = params.token.is_some();
    tracing::info!(
        has_token,
        mode = params.mode.as_deref().unwrap_or("hybrid"),
        "Terminal WS: new connection request"
    );
    ws.on_upgrade(move |socket| handle_terminal_socket(socket, params))
}

/// Core terminal WebSocket loop.
///
/// 1. Parse connection params → create session → register with tier limits
/// 2. Spawn PTY process
/// 3. Bidirectional select loop: client input → PTY/MCP/AI, PTY output → client
/// 4. Cleanup on disconnect
async fn handle_terminal_socket(mut socket: WebSocket, params: TerminalConnectParams) {
    let term_state = get_terminal_state();

    // Parse connection parameters (fall back to defaults for dev/testing)
    let tenant_id = params
        .tenant_id
        .and_then(|s| TenantId::parse(&s))
        .unwrap_or_else(TenantId::new);
    let user_id = params
        .user_id
        .and_then(|s| UserId::parse(&s))
        .unwrap_or_else(UserId::new);
    let mode = parse_mode(params.mode.as_deref());
    let tier = parse_tier(params.tier.as_deref());

    // Create and register the session
    let session = TerminalSession::new(tenant_id, user_id, mode);

    let session_id = match term_state.registry.register(session, &tier).await {
        Ok(id) => id,
        Err(RegistryError::SessionLimitExceeded { limit, .. }) => {
            let err = WsServerMessage::error(
                "SESSION_LIMIT",
                format!("Maximum concurrent sessions ({limit}) reached for your tier"),
            );
            if let Ok(json) = serde_json::to_string(&err) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::warn!("Terminal WS: failed to send limit error to client");
                }
            }
            return;
        }
        Err(e) => {
            tracing::error!(error = ?e, "Terminal WS: session registration failed");
            let err = WsServerMessage::error("REGISTRATION_FAILED", format!("{e}"));
            if let Ok(json) = serde_json::to_string(&err) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::warn!("Terminal WS: failed to send registration error");
                }
            }
            return;
        }
    };

    tracing::info!(
        session_id = %session_id,
        tenant_id = %tenant_id,
        mode = ?mode,
        "Terminal WS: session created"
    );

    // Resolve per-user persistent home directory.
    // GCS FUSE mounts gs://nexvigilant-cloud-shell-homes at /home/cloud-shell/.
    // Each user gets /home/cloud-shell/{user_id}/ — persists across container restarts.
    // Falls back to /tmp if the FUSE mount isn't available (local dev).
    let cloud_shell_base = std::path::PathBuf::from("/home/cloud-shell");
    let user_home = cloud_shell_base.join(user_id.to_string());
    let working_dir = if cloud_shell_base.exists() {
        // Production: GCS FUSE mounted — create user dir if first login
        if !user_home.exists() {
            if let Err(e) = std::fs::create_dir_all(&user_home) {
                tracing::warn!(
                    user_id = %user_id,
                    error = %e,
                    "Terminal WS: failed to create user home, falling back to /tmp"
                );
            }
        }
        if user_home.exists() {
            user_home.to_string_lossy().to_string()
        } else {
            "/tmp".to_string()
        }
    } else {
        // Local dev: no FUSE mount
        "/tmp".to_string()
    };

    tracing::info!(
        user_id = %user_id,
        working_dir = %working_dir,
        "Terminal WS: resolved user home directory"
    );

    // Spawn PTY process in the user's persistent home
    let pty_config = PtyConfig::new("/bin/bash", &working_dir)
        .with_env("HOME", &working_dir)
        .with_env("USER", "appuser")
        .with_env("TERM", "xterm-256color")
        .with_env("COLORTERM", "truecolor")
        .with_env("LANG", "en_US.UTF-8")
        .with_env("NEXVIGILANT_STATION_URL", "https://mcp.nexvigilant.com");

    let mut pty = match PtyProcess::spawn(pty_config) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "Terminal WS: failed to spawn PTY");
            let err = WsServerMessage::error("PTY_SPAWN_FAILED", e.to_string());
            if let Ok(json) = serde_json::to_string(&err) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::warn!("Terminal WS: failed to send spawn error to client");
                }
            }
            term_state
                .registry
                .update_status(&session_id, SessionStatus::Terminated)
                .await;
            return;
        }
    };

    // Activate the session
    term_state
        .registry
        .update_status(&session_id, SessionStatus::Active)
        .await;

    // Send welcome status
    let welcome = WsServerMessage::Status {
        session: SessionStatusMsg::new(
            SessionStatus::Active,
            "Terminal session active",
            session_id.to_string(),
            mode,
        ),
    };
    if let Ok(json) = serde_json::to_string(&welcome) {
        if socket.send(Message::Text(json.into())).await.is_err() {
            tracing::warn!("Terminal WS: client disconnected during welcome");
            cleanup_session(&term_state, &session_id, &mut pty).await;
            return;
        }
    }

    // Send user preferences immediately after welcome
    match term_state.preferences.load(&tenant_id, &user_id).await {
        Ok(prefs) => {
            let prefs_msg = WsServerMessage::preferences(prefs);
            if let Ok(json) = serde_json::to_string(&prefs_msg) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::warn!("Terminal WS: client disconnected during preferences send");
                    cleanup_session(&term_state, &session_id, &mut pty).await;
                    return;
                }
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "Terminal WS: failed to load preferences, using defaults");
            let prefs_msg = WsServerMessage::preferences(term_state.preferences.defaults());
            if let Ok(json) = serde_json::to_string(&prefs_msg) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::warn!(
                        "Terminal WS: client disconnected during default preferences send"
                    );
                    cleanup_session(&term_state, &session_id, &mut pty).await;
                    return;
                }
            }
        }
    }

    // Send user layout immediately after preferences
    match term_state.layouts.load(&tenant_id, &user_id).await {
        Ok(layout) => {
            let layout_msg = WsServerMessage::layout(layout);
            if let Ok(json) = serde_json::to_string(&layout_msg) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::warn!("Terminal WS: client disconnected during layout send");
                    cleanup_session(&term_state, &session_id, &mut pty).await;
                    return;
                }
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "Terminal WS: failed to load layout, using defaults");
            let layout_msg = WsServerMessage::layout(term_state.layouts.defaults());
            if let Ok(json) = serde_json::to_string(&layout_msg) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::warn!("Terminal WS: client disconnected during default layout send");
                    cleanup_session(&term_state, &session_id, &mut pty).await;
                    return;
                }
            }
        }
    }

    // Send user keybindings immediately after layout
    match term_state.keybindings.load(&tenant_id, &user_id).await {
        Ok(bindings) => {
            let kb_msg = WsServerMessage::keybindings(bindings);
            if let Ok(json) = serde_json::to_string(&kb_msg) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::warn!("Terminal WS: client disconnected during keybindings send");
                    cleanup_session(&term_state, &session_id, &mut pty).await;
                    return;
                }
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "Terminal WS: failed to load keybindings, using defaults");
            let kb_msg = WsServerMessage::keybindings(term_state.keybindings.defaults());
            if let Ok(json) = serde_json::to_string(&kb_msg) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::warn!(
                        "Terminal WS: client disconnected during default keybindings send"
                    );
                    cleanup_session(&term_state, &session_id, &mut pty).await;
                    return;
                }
            }
        }
    }

    // Bidirectional I/O loop
    let mut current_mode = mode;

    // AI conversation state — persists across messages within the session
    let mut conversation = ConversationContext::new("claude-sonnet-4-6", 200_000);
    conversation.set_system_prompt(
        "You are NexChat, an AI assistant integrated into NexVigilant's terminal. \
         You have access to MCP tools for pharmacovigilance, signal detection, \
         regulatory intelligence, and data analysis. Use tools when they help \
         answer the user's question accurately.",
    );

    loop {
        tokio::select! {
            // PTY stdout → WebSocket client
            read_result = pty.read(4096) => {
                match read_result {
                    Ok(data) if data.is_empty() => {
                        // PTY process exited
                        tracing::info!(session_id = %session_id, "Terminal WS: PTY process exited");
                        let status = WsServerMessage::Status {
                            session: SessionStatusMsg::new(
                                SessionStatus::Terminated,
                                "Process exited",
                                session_id.to_string(),
                                current_mode,
                            ),
                        };
                        if let Ok(json) = serde_json::to_string(&status) {
                            // Best-effort final status — client may already be gone
                            if socket.send(Message::Text(json.into())).await.is_err() {
                                tracing::debug!("Terminal WS: client gone before exit status sent");
                            }
                        }
                        break;
                    }
                    Ok(data) => {
                        let text = String::from_utf8_lossy(&data);
                        let msg = WsServerMessage::output(text.as_ref());
                        if let Ok(json) = serde_json::to_string(&msg) {
                            if socket.send(Message::Text(json.into())).await.is_err() {
                                tracing::info!(session_id = %session_id, "Terminal WS: client disconnected (send failed)");
                                break;
                            }
                        }
                        term_state.registry.touch(&session_id).await;
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        // "no data available" is transient (PTY has nothing to output) — skip, don't break
                        if err_msg.contains("no data available") || err_msg.contains("Resource temporarily unavailable") {
                            tracing::trace!(session_id = %session_id, "Terminal WS: PTY no data (transient)");
                            continue;
                        }
                        // Real PTY errors — log and break
                        tracing::error!(session_id = %session_id, error = %e, "Terminal WS: PTY read error");
                        let err = WsServerMessage::error("PTY_READ_ERROR", err_msg);
                        if let Ok(json) = serde_json::to_string(&err) {
                            if socket.send(Message::Text(json.into())).await.is_err() {
                                tracing::debug!("Terminal WS: client gone before error sent");
                            }
                        }
                        break;
                    }
                }
            }

            // WebSocket client → route to PTY/MCP/AI
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<WsClientMessage>(&text) {
                            Ok(client_msg) => {
                                handle_client_message(
                                    client_msg,
                                    &mut socket,
                                    &mut pty,
                                    &term_state,
                                    &session_id,
                                    &mut current_mode,
                                    &mut conversation,
                                    &tenant_id,
                                    &user_id,
                                ).await;
                            }
                            Err(_) => {
                                let err = WsServerMessage::error(
                                    "INVALID_MESSAGE",
                                    "Failed to parse client message",
                                );
                                if let Ok(json) = serde_json::to_string(&err) {
                                    if socket.send(Message::Text(json.into())).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        tracing::info!(session_id = %session_id, "Terminal WS: client disconnected");
                        break;
                    }
                    _ => {
                        // Binary, Pong, or other frame types — ignore
                    }
                }
            }
        }
    }

    cleanup_session(&term_state, &session_id, &mut pty).await;
    tracing::info!(session_id = %session_id, "Terminal WS: session closed");
}

// =============================================================================
// Client Message Dispatch
// =============================================================================

/// Handle a typed client message by routing it to the appropriate backend.
async fn handle_client_message(
    msg: WsClientMessage,
    socket: &mut WebSocket,
    pty: &mut PtyProcess,
    term_state: &TerminalState,
    session_id: &vr_core::ids::TerminalSessionId,
    current_mode: &mut TerminalMode,
    conversation: &mut ConversationContext,
    tenant_id: &TenantId,
    user_id: &UserId,
) {
    match msg {
        WsClientMessage::Input { data } => {
            // Route based on current mode
            let routed = route_command(&data, *current_mode);
            dispatch_routed_command(routed, socket, pty, conversation).await;
        }
        WsClientMessage::Command { command } => {
            // Explicit command — route with prefix detection
            let routed = route_command(&command, *current_mode);
            dispatch_routed_command(routed, socket, pty, conversation).await;
        }
        WsClientMessage::Resize { cols, rows } => {
            pty.resize(PtySize::new(cols, rows));
            tracing::debug!(cols, rows, "Terminal WS: resize");
        }
        WsClientMessage::ModeSwitch { mode } => {
            *current_mode = mode;
            let status = WsServerMessage::Status {
                session: SessionStatusMsg::new(
                    SessionStatus::Active,
                    format!("Mode switched to {mode:?}"),
                    session_id.to_string(),
                    mode,
                ),
            };
            if let Ok(json) = serde_json::to_string(&status) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::debug!("Terminal WS: client gone during mode switch ack");
                }
            }
        }
        WsClientMessage::Ping => {
            let pong = WsServerMessage::pong();
            if let Ok(json) = serde_json::to_string(&pong) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::debug!("Terminal WS: client gone during pong");
                }
            }
        }
        WsClientMessage::GetPreferences => {
            let prefs = match term_state.preferences.load(tenant_id, user_id).await {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(error = %e, "Terminal WS: preferences load failed");
                    term_state.preferences.defaults()
                }
            };
            let msg = WsServerMessage::preferences(prefs);
            if let Ok(json) = serde_json::to_string(&msg) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::debug!("Terminal WS: client gone during preferences send");
                }
            }
        }
        WsClientMessage::UpdatePreference { key, value } => {
            handle_update_preference(socket, term_state, tenant_id, user_id, &key, value).await;
        }
        WsClientMessage::GetLayout => {
            let layout = match term_state.layouts.load(tenant_id, user_id).await {
                Ok(l) => l,
                Err(e) => {
                    tracing::warn!(error = %e, "Terminal WS: layout load failed");
                    term_state.layouts.defaults()
                }
            };
            let msg = WsServerMessage::layout(layout);
            if let Ok(json) = serde_json::to_string(&msg) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::debug!("Terminal WS: client gone during layout send");
                }
            }
        }
        WsClientMessage::UpdateLayout { layout } => {
            if let Err(e) = term_state.layouts.save(tenant_id, user_id, &layout).await {
                tracing::warn!(error = %e, "Terminal WS: layout save failed");
                let err = WsServerMessage::error("LAYOUT_SAVE_ERROR", e.to_string());
                if let Ok(json) = serde_json::to_string(&err) {
                    if socket.send(Message::Text(json.into())).await.is_err() {
                        tracing::debug!("Terminal WS: client gone during layout error send");
                    }
                }
            } else {
                // Echo back the saved layout as confirmation
                let msg = WsServerMessage::layout(layout);
                if let Ok(json) = serde_json::to_string(&msg) {
                    if socket.send(Message::Text(json.into())).await.is_err() {
                        tracing::debug!("Terminal WS: client gone during layout ack");
                    }
                }
            }
        }
        WsClientMessage::GetKeybindings => {
            let bindings = match term_state.keybindings.load(tenant_id, user_id).await {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!(error = %e, "Terminal WS: keybindings load failed");
                    term_state.keybindings.defaults()
                }
            };
            let msg = WsServerMessage::keybindings(bindings);
            if let Ok(json) = serde_json::to_string(&msg) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::debug!("Terminal WS: client gone during keybindings send");
                }
            }
        }
        WsClientMessage::UpdateKeybindings { bindings } => {
            if let Err(e) = term_state
                .keybindings
                .save(tenant_id, user_id, &bindings)
                .await
            {
                tracing::warn!(error = %e, "Terminal WS: keybindings save failed");
                let err = WsServerMessage::error("KEYBINDINGS_SAVE_ERROR", e.to_string());
                if let Ok(json) = serde_json::to_string(&err) {
                    if socket.send(Message::Text(json.into())).await.is_err() {
                        tracing::debug!("Terminal WS: client gone during keybindings error send");
                    }
                }
            } else {
                // Echo back the saved keybindings as confirmation
                let msg = WsServerMessage::keybindings(bindings);
                if let Ok(json) = serde_json::to_string(&msg) {
                    if socket.send(Message::Text(json.into())).await.is_err() {
                        tracing::debug!("Terminal WS: client gone during keybindings ack");
                    }
                }
            }
        }
        // non_exhaustive: ignore unknown variants gracefully
        _ => {}
    }
    term_state.registry.touch(session_id).await;
}

/// Apply a single preference update, validate, persist, and ack.
async fn handle_update_preference(
    socket: &mut WebSocket,
    term_state: &TerminalState,
    tenant_id: &TenantId,
    user_id: &UserId,
    key: &str,
    value: serde_json::Value,
) {
    use nexcore_terminal::preferences::{
        ColorScheme, CursorStyle, FontFamily, FontSize, LineHeight, ScrollbackSize,
    };

    // Load current preferences (or defaults)
    let mut prefs = match term_state.preferences.load(tenant_id, user_id).await {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(error = %e, "Terminal WS: preferences load for update failed");
            term_state.preferences.defaults()
        }
    };

    // Apply the field update with validation/clamping
    let applied_value = match key {
        "font_size" => {
            if let Some(n) = value.as_u64() {
                let size = FontSize::new(n.min(255) as u8);
                prefs.font_size = size;
                serde_json::json!(size.value())
            } else {
                send_preference_error(socket, key, "expected integer").await;
                return;
            }
        }
        "font_family" => {
            if let Some(s) = value.as_str() {
                let family = FontFamily::new(s);
                let val = serde_json::json!(family.as_str());
                prefs.font_family = family;
                val
            } else {
                send_preference_error(socket, key, "expected string").await;
                return;
            }
        }
        "line_height" => {
            if let Some(n) = value.as_f64() {
                #[allow(
                    clippy::as_conversions,
                    reason = "f64 → f32 narrowing is intentional; clamp handles range"
                )]
                let height = LineHeight::new(n as f32);
                prefs.line_height = height;
                serde_json::json!(height.value())
            } else {
                send_preference_error(socket, key, "expected number").await;
                return;
            }
        }
        "cursor_style" => match serde_json::from_value::<CursorStyle>(value.clone()) {
            Ok(style) => {
                prefs.cursor_style = style;
                serde_json::to_value(style).unwrap_or(value)
            }
            Err(_) => {
                send_preference_error(socket, key, "expected \"block\", \"underline\", or \"bar\"")
                    .await;
                return;
            }
        },
        "cursor_blink" => {
            if let Some(b) = value.as_bool() {
                prefs.cursor_blink = b;
                serde_json::json!(b)
            } else {
                send_preference_error(socket, key, "expected boolean").await;
                return;
            }
        }
        "scrollback" => {
            if let Some(n) = value.as_u64() {
                let size = ScrollbackSize::new(n.min(u64::from(u32::MAX)) as u32);
                prefs.scrollback = size;
                serde_json::json!(size.value())
            } else {
                send_preference_error(socket, key, "expected integer").await;
                return;
            }
        }
        "color_scheme" => match serde_json::from_value::<ColorScheme>(value.clone()) {
            Ok(scheme) => {
                prefs.color_scheme = scheme;
                serde_json::to_value(scheme).unwrap_or(value)
            }
            Err(_) => {
                send_preference_error(
                        socket,
                        key,
                        "expected \"nexvigilant_dark\", \"high_contrast\", \"light\", or \"solarized_dark\"",
                    )
                    .await;
                return;
            }
        },
        unknown => {
            send_preference_error(socket, unknown, "unknown preference key").await;
            return;
        }
    };

    // Persist
    if let Err(e) = term_state
        .preferences
        .save(tenant_id, user_id, &prefs)
        .await
    {
        tracing::error!(error = %e, "Terminal WS: failed to save preferences");
        let err = WsServerMessage::error("PREFERENCES_SAVE_ERROR", e.to_string());
        if let Ok(json) = serde_json::to_string(&err) {
            if socket.send(Message::Text(json.into())).await.is_err() {
                tracing::debug!("Terminal WS: client gone during preferences save error send");
            }
        }
        return;
    }

    // Ack with the validated/clamped value
    let ack = WsServerMessage::preference_updated(key, applied_value);
    if let Ok(json) = serde_json::to_string(&ack) {
        if socket.send(Message::Text(json.into())).await.is_err() {
            tracing::debug!("Terminal WS: client gone during preference update ack");
        }
    }
}

/// Send a preference validation error to the client.
async fn send_preference_error(socket: &mut WebSocket, key: &str, reason: &str) {
    let err = WsServerMessage::error("INVALID_PREFERENCE", format!("{key}: {reason}"));
    if let Ok(json) = serde_json::to_string(&err) {
        if socket.send(Message::Text(json.into())).await.is_err() {
            tracing::debug!("Terminal WS: client gone during preference error send");
        }
    }
}

/// Dispatch a routed command to the appropriate backend.
async fn dispatch_routed_command(
    routed: RoutedCommand,
    socket: &mut WebSocket,
    pty: &mut PtyProcess,
    conversation: &mut ConversationContext,
) {
    match routed {
        RoutedCommand::Shell(cmd) => {
            // Write to PTY stdin — output comes back via the read loop
            if let Err(e) = pty.write(cmd.as_bytes()).await {
                let err = WsServerMessage::error("PTY_WRITE_ERROR", e.to_string());
                if let Ok(json) = serde_json::to_string(&err) {
                    if socket.send(Message::Text(json.into())).await.is_err() {
                        tracing::debug!("Terminal WS: client gone during PTY write error");
                    }
                }
            }
        }
        RoutedCommand::Mcp { tool_name, params } => {
            // MCP tool dispatch — in-process via nexcore-mcp bridge
            let server = NexCoreMcpServer::new();
            match mcp_bridge::call_tool(&tool_name, params, &server).await {
                Ok(result) => {
                    // ANSI-formatted output for terminal display
                    let formatted = format_mcp_result(&tool_name, &result);
                    let output = WsServerMessage::output(formatted);
                    if let Ok(json) = serde_json::to_string(&output) {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            tracing::debug!("Terminal WS: client gone during MCP output send");
                            return;
                        }
                    }
                    // Structured result for programmatic consumption
                    let structured = WsServerMessage::Result {
                        source: format!("mcp:{tool_name}"),
                        content: result,
                    };
                    if let Ok(json) = serde_json::to_string(&structured) {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            tracing::debug!("Terminal WS: client gone during MCP result send");
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        tool = %tool_name,
                        error = %e,
                        "Terminal WS: MCP dispatch failed"
                    );
                    let err = WsServerMessage::error("MCP_DISPATCH_ERROR", e.to_string());
                    if let Ok(json) = serde_json::to_string(&err) {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            tracing::debug!("Terminal WS: client gone during MCP error send");
                        }
                    }
                }
            }
        }
        RoutedCommand::Station { tool_name, params } => {
            // Station dispatch — HTTP to mcp.nexvigilant.com
            let config = StationConfig::default();
            match call_station_tool(&tool_name, params, &config).await {
                Ok(result) => {
                    // ANSI-formatted output
                    let value_str = serde_json::to_string_pretty(&result.value)
                        .unwrap_or_else(|e| format!("(serialization error: {e})"));
                    let formatted = format!(
                        "\x1b[36m[station:{tool_name}]\x1b[0m ({:.0}ms)\n{}",
                        result.elapsed.as_millis(),
                        value_str
                    );
                    let output = WsServerMessage::output(formatted);
                    if let Ok(json) = serde_json::to_string(&output) {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            tracing::debug!("Terminal WS: client gone during Station output");
                            return;
                        }
                    }
                    // Structured result
                    let structured = WsServerMessage::Result {
                        source: format!("station:{tool_name}"),
                        content: result.value,
                    };
                    if let Ok(json) = serde_json::to_string(&structured) {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            tracing::debug!("Terminal WS: client gone during Station result");
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        tool = %tool_name,
                        error = %e,
                        "Terminal WS: Station dispatch failed"
                    );
                    let err = WsServerMessage::error("STATION_DISPATCH_ERROR", e.to_string());
                    if let Ok(json) = serde_json::to_string(&err) {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            tracing::debug!("Terminal WS: client gone during Station error");
                        }
                    }
                }
            }
        }
        RoutedCommand::Microgram { name, variables } => {
            // Microgram dispatch — rsk subprocess
            let config = MicrogramConfig::default();
            let result = run_microgram(&name, &variables, &config).await;

            let formatted = if result.success {
                let output_str = serde_json::to_string_pretty(&result.output)
                    .unwrap_or_else(|e| format!("(serialization error: {e})"));
                format!(
                    "\x1b[32m[mcg:{name}]\x1b[0m ({}ms)\n{}",
                    result.elapsed_ms, output_str
                )
            } else {
                format!(
                    "\x1b[31m[mcg:{name}]\x1b[0m ERROR ({}ms)\n{}",
                    result.elapsed_ms,
                    result.error.as_deref().unwrap_or("unknown error")
                )
            };

            let output = WsServerMessage::output(formatted);
            if let Ok(json) = serde_json::to_string(&output) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::debug!("Terminal WS: client gone during mcg output");
                    return;
                }
            }
            // Structured result
            let structured = WsServerMessage::Result {
                source: format!("mcg:{name}"),
                content: serde_json::to_value(&result)
                    .unwrap_or_else(|_| serde_json::json!({"error": "serialization failed"})),
            };
            if let Ok(json) = serde_json::to_string(&structured) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::debug!("Terminal WS: client gone during mcg result");
                }
            }
        }
        RoutedCommand::Ai { message, .. } => {
            // AI dispatch — try relay first (Agent SDK + Station MCP), fall back to direct Claude.
            // Relay call is wrapped in timeout + catch_unwind to prevent WS connection drops.
            let relay_config = RelayConfig::default();
            let relay_result = tokio::time::timeout(
                std::time::Duration::from_secs(35),
                query_relay(&message, &relay_config),
            )
            .await;

            match relay_result {
                Ok(Ok(result)) => {
                    let formatted = format!(
                        "\x1b[33m[relay]\x1b[0m ({}ms relay, {}ms total)\n{}",
                        result.relay_elapsed_ms, result.total_elapsed_ms, result.text
                    );
                    let output = WsServerMessage::output(formatted);
                    if let Ok(json) = serde_json::to_string(&output) {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            tracing::debug!("Terminal WS: client gone during relay output");
                            return;
                        }
                    }
                    let done = WsServerMessage::AiToken {
                        token: String::new(),
                        done: true,
                    };
                    if let Ok(json) = serde_json::to_string(&done) {
                        let _ = socket.send(Message::Text(json.into())).await;
                    }
                }
                Ok(Err(e)) => {
                    tracing::info!(error = %e, "Relay error, falling back to direct Claude");
                    dispatch_ai_message(&message, socket, conversation).await;
                }
                Err(_) => {
                    tracing::info!("Relay timed out after 35s, falling back to direct Claude");
                    dispatch_ai_message(&message, socket, conversation).await;
                }
            }
        }
        RoutedCommand::Control(ctrl) => {
            // Control commands handled in handle_client_message already
            // This branch catches anything that slipped through routing
            tracing::debug!(command = ?ctrl, "Terminal WS: unhandled control command via router");
        }
        // non_exhaustive: ignore unknown variants
        _ => {}
    }
}

// =============================================================================
// AI Dispatch
// =============================================================================

/// Stream a Claude response for an AI message, handling tool_use loops.
///
/// Grounding: σ(Sequence) + →(Causality) + μ(Mapping) + ∂(Boundary)
async fn dispatch_ai_message(
    user_message: &str,
    socket: &mut WebSocket,
    conversation: &mut ConversationContext,
) {
    // 1. Build client from env
    let client = match ClaudeConfig::from_env() {
        Ok(config) => ClaudeClient::new(config),
        Err(e) => {
            let err = WsServerMessage::error("AI_CONFIG_ERROR", e.to_string());
            if let Ok(json) = serde_json::to_string(&err) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::debug!("Terminal WS: client gone during AI config error");
                }
            }
            return;
        }
    };

    // 2. Add user message to conversation
    conversation.add_user_message(user_message);

    // 3. Auto-discover tools
    let server = NexCoreMcpServer::new();
    let bridge = AiMcpBridge::new(&server, ToolScope::All);
    let tools = bridge.available_tools();

    // 4. Stream response (with tool_use loop, max 5 rounds)
    let max_tool_rounds = 5;
    for _round in 0..max_tool_rounds {
        let (tx, mut rx) = tokio::sync::mpsc::channel(64);

        // Spawn stream task
        let stream_result = {
            let stream_future = client.stream(conversation, &tools, tx);
            stream_future.await
        };

        // Collect tokens and tool_use events from channel
        let mut full_text = String::new();
        let mut tool_calls = Vec::new();
        let mut stop_reason = String::new();

        while let Some(event) = rx.recv().await {
            match event {
                StreamEvent::Token(token) => {
                    full_text.push_str(&token);
                    let ai_token = WsServerMessage::AiToken { token, done: false };
                    if let Ok(json) = serde_json::to_string(&ai_token) {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            tracing::debug!("Terminal WS: client gone during AI token send");
                            return;
                        }
                    }
                }
                StreamEvent::ToolUse { id, name, input } => {
                    tool_calls.push((id, name, input));
                }
                StreamEvent::Done {
                    stop_reason: sr,
                    input_tokens,
                    output_tokens,
                } => {
                    stop_reason = sr;
                    tracing::debug!(input_tokens, output_tokens, "Terminal WS: AI stream done");
                }
                StreamEvent::Error(msg) => {
                    let err = WsServerMessage::error("AI_STREAM_ERROR", msg);
                    if let Ok(json) = serde_json::to_string(&err) {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            tracing::debug!("Terminal WS: client gone during AI error send");
                        }
                    }
                    return;
                }
            }
        }

        // Handle stream-level errors
        if let Err(e) = stream_result {
            let err = WsServerMessage::error("AI_REQUEST_ERROR", e.to_string());
            if let Ok(json) = serde_json::to_string(&err) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::debug!("Terminal WS: client gone during AI request error");
                }
            }
            return;
        }

        // 5. If no tool calls, we're done
        if tool_calls.is_empty() || stop_reason != "tool_use" {
            // Send final done token
            let done = WsServerMessage::AiToken {
                token: String::new(),
                done: true,
            };
            if let Ok(json) = serde_json::to_string(&done) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::debug!("Terminal WS: client gone during AI done send");
                }
            }

            // Record assistant response in conversation
            if !full_text.is_empty() {
                conversation.add_assistant_message(full_text);
            }
            return;
        }

        // 6. Tool use loop: dispatch each tool, feed results back
        let ai_tool_calls: Vec<nexcore_terminal::ai::AiToolCall> = tool_calls
            .iter()
            .map(|(id, name, input)| {
                nexcore_terminal::ai::AiToolCall::new(id.clone(), name.clone(), input.clone())
            })
            .collect();

        conversation.add_assistant_tool_calls(full_text.clone(), ai_tool_calls);

        for (id, name, input) in &tool_calls {
            // Notify client that a tool is being called
            let tool_status = WsServerMessage::Result {
                source: format!("ai:tool:{name}"),
                content: serde_json::json!({
                    "status": "executing",
                    "tool": name,
                    "input": input,
                }),
            };
            if let Ok(json) = serde_json::to_string(&tool_status) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::debug!("Terminal WS: client gone during tool status send");
                    return;
                }
            }

            // Execute tool
            let (tool_result, is_error) = match bridge.execute_tool_call(name, input.clone()).await
            {
                Ok(result) => (result, false),
                Err(e) => (format!("Tool error: {e}"), true),
            };

            // Record tool result in conversation
            conversation.add_tool_result(id, &tool_result, is_error);
        }

        // Loop back to stream the next response with tool results
    }

    // Max rounds exceeded
    let done = WsServerMessage::AiToken {
        token: String::new(),
        done: true,
    };
    if let Ok(json) = serde_json::to_string(&done) {
        if socket.send(Message::Text(json.into())).await.is_err() {
            tracing::debug!("Terminal WS: client gone during AI max-rounds done send");
        }
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// Clean up session state and kill the PTY process.
async fn cleanup_session(
    term_state: &TerminalState,
    session_id: &vr_core::ids::TerminalSessionId,
    pty: &mut PtyProcess,
) {
    term_state
        .registry
        .update_status(session_id, SessionStatus::Terminated)
        .await;
    if let Err(e) = pty.kill().await {
        tracing::warn!(
            session_id = %session_id,
            error = %e,
            "Terminal WS: failed to kill PTY on cleanup"
        );
    }
}

/// Parse terminal mode from query string.
fn parse_mode(mode: Option<&str>) -> TerminalMode {
    match mode {
        Some("shell") => TerminalMode::Shell,
        Some("regulatory") => TerminalMode::Regulatory,
        Some("ai") => TerminalMode::Ai,
        _ => TerminalMode::Hybrid,
    }
}

/// Parse subscription tier from query string.
fn parse_tier(tier: Option<&str>) -> SubscriptionTier {
    match tier {
        Some("accelerator") => SubscriptionTier::Accelerator,
        Some("enterprise") => SubscriptionTier::Enterprise,
        _ => SubscriptionTier::Explorer,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mode_defaults_to_hybrid() {
        assert_eq!(parse_mode(None), TerminalMode::Hybrid);
        assert_eq!(parse_mode(Some("unknown")), TerminalMode::Hybrid);
    }

    #[test]
    fn parse_mode_recognizes_all_modes() {
        assert_eq!(parse_mode(Some("shell")), TerminalMode::Shell);
        assert_eq!(parse_mode(Some("regulatory")), TerminalMode::Regulatory);
        assert_eq!(parse_mode(Some("ai")), TerminalMode::Ai);
    }

    #[test]
    fn parse_tier_defaults_to_explorer() {
        assert_eq!(parse_tier(None), SubscriptionTier::Explorer);
        assert_eq!(parse_tier(Some("unknown")), SubscriptionTier::Explorer);
    }

    #[test]
    fn parse_tier_recognizes_tiers() {
        assert_eq!(
            parse_tier(Some("accelerator")),
            SubscriptionTier::Accelerator
        );
        assert_eq!(parse_tier(Some("enterprise")), SubscriptionTier::Enterprise);
    }

    #[test]
    fn terminal_state_default_creates_empty_registry() {
        let state = TerminalState::default();
        // Verify construction succeeds — registry is async so no deep test here
        drop(state);
    }

    #[tokio::test]
    async fn session_registration_and_cleanup() {
        let state = TerminalState::default();
        let tenant = TenantId::new();
        let user = UserId::new();
        let session = TerminalSession::new(tenant, user, TerminalMode::Hybrid);

        let result = state
            .registry
            .register(session, &SubscriptionTier::Enterprise)
            .await;
        assert!(result.is_ok());

        let id = result.expect("registration should succeed");
        assert_eq!(state.registry.active_count().await, 1);

        state
            .registry
            .update_status(&id, SessionStatus::Terminated)
            .await;
        assert_eq!(state.registry.active_count().await, 0);
    }
}
