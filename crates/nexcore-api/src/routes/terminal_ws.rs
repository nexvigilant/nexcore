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
//! **Client → Server:** `WsClientMessage` (input, command, resize, mode_switch, ping)
//! **Server → Client:** `WsServerMessage` (output, result, ai_token, status, error, pong)

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

use nexcore_mcp::NexCoreMcpServer;

use nexcore_terminal::formatter::format_mcp_result;
use nexcore_terminal::protocol::{SessionStatusMsg, WsClientMessage, WsServerMessage};
use nexcore_terminal::pty::{PtyConfig, PtyProcess, PtySize};
use nexcore_terminal::registry::{RegistryError, SessionRegistry};
use nexcore_terminal::router::{RoutedCommand, route_command};
use nexcore_terminal::session::{SessionStatus, TerminalMode, TerminalSession};

use crate::ApiState;
use crate::mcp_bridge;

// =============================================================================
// Shared State
// =============================================================================

/// Terminal subsystem state shared across all connections.
pub struct TerminalState {
    /// Session registry with per-tenant concurrency limits.
    pub registry: SessionRegistry,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            registry: SessionRegistry::new(),
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
/// The client passes tenant/user identity and preferred mode as query params.
/// In production, these would come from a verified JWT — for now, explicit params.
#[derive(Debug, Deserialize)]
pub struct TerminalConnectParams {
    /// Tenant identifier (UUID).
    pub tenant_id: Option<String>,
    /// User identifier (UUID).
    pub user_id: Option<String>,
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
    tracing::info!("Terminal WS: new connection request");
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

    // Spawn PTY process
    let pty_config = PtyConfig::new("/bin/bash", "/tmp");

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

    // Bidirectional I/O loop
    let mut current_mode = mode;

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
                        tracing::error!(session_id = %session_id, error = %e, "Terminal WS: PTY read error");
                        let err = WsServerMessage::error("PTY_READ_ERROR", e.to_string());
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
) {
    match msg {
        WsClientMessage::Input { data } => {
            // Route based on current mode
            let routed = route_command(&data, *current_mode);
            dispatch_routed_command(routed, socket, pty).await;
        }
        WsClientMessage::Command { command } => {
            // Explicit command — route with prefix detection
            let routed = route_command(&command, *current_mode);
            dispatch_routed_command(routed, socket, pty).await;
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
        // non_exhaustive: ignore unknown variants gracefully
        _ => {}
    }
    term_state.registry.touch(session_id).await;
}

/// Dispatch a routed command to the appropriate backend.
async fn dispatch_routed_command(
    routed: RoutedCommand,
    socket: &mut WebSocket,
    pty: &mut PtyProcess,
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
        RoutedCommand::Ai { message, .. } => {
            // AI dispatch — route to Claude backend
            // For now, return a placeholder; real AI integration in P5
            let msg = WsServerMessage::Result {
                source: "ai".to_string(),
                content: serde_json::json!({
                    "message": message,
                    "response": "AI dispatch pending (Phase 5)",
                }),
            };
            if let Ok(json) = serde_json::to_string(&msg) {
                if socket.send(Message::Text(json.into())).await.is_err() {
                    tracing::debug!("Terminal WS: client gone during AI result send");
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
