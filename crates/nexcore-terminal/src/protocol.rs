//! WebSocket wire protocol — typed messages between client and server.
//!
//! These types are serialized as JSON over WebSocket frames. The TypeScript
//! types in `src/types/terminal.ts` mirror these exactly.

use serde::{Deserialize, Serialize};

use crate::session::{SessionStatus, TerminalMode};

/// Client → Server message sent over WebSocket.
#[non_exhaustive]
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientMessage {
    /// Raw terminal input (keystrokes).
    Input {
        /// Raw input data (may be single char or paste buffer).
        data: String,
    },
    /// Structured command (MCP or AI prefix detected client-side).
    Command {
        /// Full command string including any prefix.
        command: String,
    },
    /// Terminal resize event.
    Resize {
        /// New column count.
        cols: u16,
        /// New row count.
        rows: u16,
    },
    /// Mode switch request.
    ModeSwitch {
        /// Requested mode.
        mode: TerminalMode,
    },
    /// Heartbeat ping.
    Ping,
}

/// Server → Client message sent over WebSocket.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsServerMessage {
    /// Terminal output (raw bytes from PTY or formatted text).
    Output {
        /// Output data (may contain ANSI escape codes).
        data: String,
    },
    /// Structured result from MCP tool or AI query.
    Result {
        /// Source of the result: "mcp", "ai", or "shell".
        source: String,
        /// Structured result content.
        content: serde_json::Value,
    },
    /// Streaming AI token.
    AiToken {
        /// The token text.
        token: String,
        /// Whether this is the final token in the response.
        done: bool,
    },
    /// Session status change notification.
    Status {
        /// Updated session state.
        session: SessionStatusMsg,
    },
    /// Error notification.
    Error {
        /// Machine-readable error code.
        code: String,
        /// Human-readable error message.
        message: String,
    },
    /// Heartbeat response.
    Pong,
}

/// Session status payload within a `WsServerMessage::Status`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct SessionStatusMsg {
    /// Current session status.
    pub status: SessionStatus,
    /// Human-readable status message.
    pub message: String,
    /// Session identifier.
    pub session_id: String,
    /// Current terminal mode.
    pub mode: TerminalMode,
}

impl SessionStatusMsg {
    /// Create a new session status message.
    #[must_use]
    pub fn new(
        status: SessionStatus,
        message: impl Into<String>,
        session_id: impl Into<String>,
        mode: TerminalMode,
    ) -> Self {
        Self {
            status,
            message: message.into(),
            session_id: session_id.into(),
            mode,
        }
    }
}

impl WsServerMessage {
    /// Convenience: create an output message.
    #[must_use]
    pub fn output(data: impl Into<String>) -> Self {
        Self::Output { data: data.into() }
    }

    /// Convenience: create an error message.
    #[must_use]
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Error {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Convenience: create a pong response.
    #[must_use]
    pub fn pong() -> Self {
        Self::Pong
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_input_deserializes() {
        let json = r#"{"type":"input","data":"ls\n"}"#;
        let msg: Result<WsClientMessage, _> = serde_json::from_str(json);
        assert!(msg.is_ok());
        if let Ok(WsClientMessage::Input { data }) = msg {
            assert_eq!(data, "ls\n");
        }
    }

    #[test]
    fn client_resize_deserializes() {
        let json = r#"{"type":"resize","cols":120,"rows":40}"#;
        let msg: Result<WsClientMessage, _> = serde_json::from_str(json);
        assert!(msg.is_ok());
        if let Ok(WsClientMessage::Resize { cols, rows }) = msg {
            assert_eq!(cols, 120);
            assert_eq!(rows, 40);
        }
    }

    #[test]
    fn client_ping_deserializes() {
        let json = r#"{"type":"ping"}"#;
        let msg: Result<WsClientMessage, _> = serde_json::from_str(json);
        assert!(msg.is_ok());
    }

    #[test]
    fn server_output_serializes() {
        let msg = WsServerMessage::output("hello world");
        let json = serde_json::to_string(&msg).unwrap_or_default();
        assert!(json.contains("\"type\":\"output\""));
        assert!(json.contains("hello world"));
    }

    #[test]
    fn server_error_serializes() {
        let msg = WsServerMessage::error("AUTH_FAILED", "Invalid token");
        let json = serde_json::to_string(&msg).unwrap_or_default();
        assert!(json.contains("\"type\":\"error\""));
        assert!(json.contains("AUTH_FAILED"));
    }

    #[test]
    fn server_ai_token_serializes() {
        let msg = WsServerMessage::AiToken {
            token: "The".to_string(),
            done: false,
        };
        let json = serde_json::to_string(&msg).unwrap_or_default();
        assert!(json.contains("\"type\":\"ai_token\""));
        assert!(json.contains("\"done\":false"));
    }

    #[test]
    fn mode_switch_deserializes() {
        let json = r#"{"type":"mode_switch","mode":"regulatory"}"#;
        let msg: Result<WsClientMessage, _> = serde_json::from_str(json);
        assert!(msg.is_ok());
        if let Ok(WsClientMessage::ModeSwitch { mode }) = msg {
            assert_eq!(mode, TerminalMode::Regulatory);
        }
    }
}
