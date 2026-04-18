//! WebSocket wire protocol — typed messages between client and server.
//!
//! These types are serialized as JSON over WebSocket frames. The TypeScript
//! types in `src/types/terminal.ts` mirror these exactly.

use serde::{Deserialize, Serialize};

use crate::keybindings::KeybindingSet;
use crate::layout::TerminalLayout;
use crate::preferences::TerminalPreferences;
use crate::session::{SessionStatus, TerminalMode};

/// Client → Server message sent over WebSocket.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Request current user preferences.
    GetPreferences,
    /// Update a single preference field by key.
    UpdatePreference {
        /// Preference field name (e.g. "font_size", "cursor_style").
        key: String,
        /// New value as JSON (e.g. `14`, `"bar"`, `"solarized_dark"`).
        value: serde_json::Value,
    },
    /// Request current layout tree.
    GetLayout,
    /// Update the full layout tree.
    UpdateLayout {
        /// The complete layout to persist.
        layout: TerminalLayout,
    },
    /// Request current keybindings.
    GetKeybindings,
    /// Update the full keybinding set.
    UpdateKeybindings {
        /// The complete keybinding set to persist.
        bindings: KeybindingSet,
    },
}

/// Server → Client message sent over WebSocket.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Full preferences snapshot (sent on connect and on request).
    Preferences {
        /// The user's complete terminal preferences.
        preferences: TerminalPreferences,
    },
    /// Confirmation that a single preference was updated.
    PreferenceUpdated {
        /// The preference field that changed.
        key: String,
        /// The new value after validation/clamping.
        value: serde_json::Value,
    },
    /// Full layout snapshot (sent on connect, on request, and after update).
    Layout {
        /// The user's complete terminal layout tree.
        layout: TerminalLayout,
    },
    /// Full keybindings snapshot (sent on connect, on request, and after update).
    Keybindings {
        /// The user's complete keybinding set.
        bindings: KeybindingSet,
    },
}

/// Session status payload within a `WsServerMessage::Status`.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Convenience: create a preferences snapshot message.
    #[must_use]
    pub fn preferences(prefs: TerminalPreferences) -> Self {
        Self::Preferences { preferences: prefs }
    }

    /// Convenience: create a preference-updated confirmation.
    #[must_use]
    pub fn preference_updated(key: impl Into<String>, value: serde_json::Value) -> Self {
        Self::PreferenceUpdated {
            key: key.into(),
            value,
        }
    }

    /// Convenience: create a layout snapshot message.
    #[must_use]
    pub fn layout(layout: TerminalLayout) -> Self {
        Self::Layout { layout }
    }

    /// Convenience: create a keybindings snapshot message.
    #[must_use]
    pub fn keybindings(bindings: KeybindingSet) -> Self {
        Self::Keybindings { bindings }
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

    #[test]
    fn client_get_preferences_deserializes() {
        let json = r#"{"type":"get_preferences"}"#;
        let msg: Result<WsClientMessage, _> = serde_json::from_str(json);
        assert!(msg.is_ok());
        assert!(matches!(
            msg.unwrap_or(WsClientMessage::Ping),
            WsClientMessage::GetPreferences
        ));
    }

    #[test]
    fn client_update_preference_deserializes() {
        let json = r#"{"type":"update_preference","key":"font_size","value":16}"#;
        let msg: Result<WsClientMessage, _> = serde_json::from_str(json);
        assert!(msg.is_ok());
        if let Ok(WsClientMessage::UpdatePreference { key, value }) = msg {
            assert_eq!(key, "font_size");
            assert_eq!(value, serde_json::json!(16));
        }
    }

    #[test]
    fn server_preferences_serializes() {
        let prefs = TerminalPreferences::default();
        let msg = WsServerMessage::preferences(prefs);
        let json = serde_json::to_string(&msg).unwrap_or_default();
        assert!(json.contains("\"type\":\"preferences\""));
        assert!(json.contains("\"font_size\""));
        assert!(json.contains("\"cursor_style\""));
    }

    #[test]
    fn server_preference_updated_serializes() {
        let msg = WsServerMessage::preference_updated("font_size", serde_json::json!(16));
        let json = serde_json::to_string(&msg).unwrap_or_default();
        assert!(json.contains("\"type\":\"preference_updated\""));
        assert!(json.contains("\"font_size\""));
        assert!(json.contains("16"));
    }

    #[test]
    fn update_preference_string_value_deserializes() {
        let json = r#"{"type":"update_preference","key":"cursor_style","value":"bar"}"#;
        let msg: Result<WsClientMessage, _> = serde_json::from_str(json);
        assert!(msg.is_ok());
        if let Ok(WsClientMessage::UpdatePreference { key, value }) = msg {
            assert_eq!(key, "cursor_style");
            assert_eq!(value, serde_json::json!("bar"));
        }
    }

    #[test]
    fn client_get_layout_deserializes() {
        let json = r#"{"type":"get_layout"}"#;
        let msg: Result<WsClientMessage, _> = serde_json::from_str(json);
        assert!(msg.is_ok());
        assert!(matches!(
            msg.unwrap_or(WsClientMessage::Ping),
            WsClientMessage::GetLayout
        ));
    }

    #[test]
    fn client_update_layout_deserializes() {
        let json = r#"{"type":"update_layout","layout":{"version":1,"root":{"type":"leaf","id":"pane-1","mode":"shell","session_id":null},"focused_pane":"pane-1"}}"#;
        let msg: Result<WsClientMessage, _> = serde_json::from_str(json);
        assert!(msg.is_ok());
        if let Ok(WsClientMessage::UpdateLayout { layout }) = msg {
            assert_eq!(layout.version, 1);
            assert_eq!(layout.focused_pane, "pane-1");
        }
    }

    #[test]
    fn server_layout_serializes() {
        let layout = crate::layout::default_layout();
        let msg = WsServerMessage::layout(layout);
        let json = serde_json::to_string(&msg).unwrap_or_default();
        assert!(json.contains("\"type\":\"layout\""));
        assert!(json.contains("\"focused_pane\""));
        assert!(json.contains("\"pane-1\""));
    }

    #[test]
    fn client_get_keybindings_deserializes() {
        let json = r#"{"type":"get_keybindings"}"#;
        let msg: Result<WsClientMessage, _> = serde_json::from_str(json);
        assert!(msg.is_ok());
        assert!(matches!(
            msg.unwrap_or(WsClientMessage::Ping),
            WsClientMessage::GetKeybindings
        ));
    }

    #[test]
    fn client_update_keybindings_deserializes() {
        let bindings = crate::keybindings::KeybindingSet::default_set();
        let json = serde_json::json!({
            "type": "update_keybindings",
            "bindings": bindings,
        });
        let msg: Result<WsClientMessage, _> = serde_json::from_str(&json.to_string());
        assert!(msg.is_ok());
        if let Ok(WsClientMessage::UpdateKeybindings { bindings: b }) = msg {
            assert_eq!(b.bindings.len(), 13);
        }
    }

    #[test]
    fn server_keybindings_serializes() {
        let bindings = crate::keybindings::KeybindingSet::default_set();
        let msg = WsServerMessage::keybindings(bindings);
        let json = serde_json::to_string(&msg).unwrap_or_default();
        assert!(json.contains("\"type\":\"keybindings\""));
        assert!(json.contains("\"bindings\""));
    }
}
