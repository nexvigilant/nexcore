//! Command routing — dispatches terminal input to the appropriate handler.
//!
//! Every line of input is classified as Shell, MCP (regulatory), AI, or
//! Control based on prefix detection and the current terminal mode.

use serde::{Deserialize, Serialize};

use crate::session::TerminalMode;

/// Parsed and classified terminal input.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum RoutedCommand {
    /// Raw shell command — write directly to PTY.
    Shell(String),
    /// MCP tool invocation — dispatch via in-process bridge.
    Mcp {
        /// Tool name (e.g., "faers_search").
        tool_name: String,
        /// Tool parameters as JSON.
        params: serde_json::Value,
    },
    /// Natural language — route to AI backend.
    Ai {
        /// The user's message.
        message: String,
        /// Existing conversation to continue, if any.
        conversation_id: Option<String>,
    },
    /// Terminal control command.
    Control(ControlCommand),
}

/// Terminal control commands (not routed to shell or tools).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlCommand {
    /// Resize the terminal.
    Resize {
        /// New column count.
        cols: u16,
        /// New row count.
        rows: u16,
    },
    /// Switch the active mode.
    SwitchMode(TerminalMode),
    /// Heartbeat.
    Ping,
    /// Graceful disconnect.
    Disconnect,
}

/// Route terminal input to the appropriate handler based on prefix and mode.
///
/// Routing priority:
/// 1. Explicit prefix `nexvig>` or `/` → MCP (regulatory)
/// 2. Explicit prefix `@claude` → AI
/// 3. Mode-based: Shell→PTY, Regulatory→MCP, Ai→AI, Hybrid→auto-detect
#[must_use]
pub fn route_command(input: &str, current_mode: TerminalMode) -> RoutedCommand {
    let trimmed = input.trim();

    // Explicit prefixes always win regardless of mode
    if let Some(rest) = trimmed
        .strip_prefix("nexvig>")
        .or_else(|| trimmed.strip_prefix("nexvig> "))
    {
        return parse_mcp_command(rest.trim());
    }

    if let Some(rest) = trimmed.strip_prefix('/') {
        // Only treat as MCP if it looks like a tool name (no spaces before first arg)
        let first_space = rest.find(' ');
        let cmd_part = match first_space {
            Some(pos) => rest.get(..pos).unwrap_or(rest),
            None => rest,
        };
        if !cmd_part.is_empty() && cmd_part.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return parse_mcp_command(rest);
        }
    }

    if let Some(rest) = trimmed.strip_prefix("@claude") {
        return RoutedCommand::Ai {
            message: rest.trim().to_string(),
            conversation_id: None,
        };
    }

    // Mode-based routing
    match current_mode {
        TerminalMode::Shell => RoutedCommand::Shell(trimmed.to_string()),
        TerminalMode::Regulatory => parse_mcp_command(trimmed),
        TerminalMode::Ai => RoutedCommand::Ai {
            message: trimmed.to_string(),
            conversation_id: None,
        },
        TerminalMode::Hybrid => auto_detect_route(trimmed),
    }
}

/// Parse a string as an MCP tool command: `tool_name arg1 arg2 --flag value`.
fn parse_mcp_command(input: &str) -> RoutedCommand {
    let trimmed = input.trim();
    let (tool_name, rest) = match trimmed.split_once(' ') {
        Some((name, args)) => (name.to_string(), args.trim()),
        None => (trimmed.to_string(), ""),
    };

    // Parse remaining args as key-value params (simple heuristic)
    let params = if rest.is_empty() {
        serde_json::Value::Object(serde_json::Map::new())
    } else {
        parse_args_to_json(rest)
    };

    RoutedCommand::Mcp { tool_name, params }
}

/// Parse command-line style arguments into a JSON object.
///
/// Supports: `--key value`, `--flag` (boolean true), positional args as "args" array.
fn parse_args_to_json(input: &str) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    let mut positional = Vec::new();
    let parts: Vec<&str> = input.split_whitespace().collect();
    let mut i = 0;

    while i < parts.len() {
        if let Some(key) = parts.get(i).and_then(|p| p.strip_prefix("--")) {
            // Check if next part is a value (not another flag)
            if let Some(next) = parts.get(i.wrapping_add(1)) {
                if next.starts_with("--") {
                    // Boolean flag
                    map.insert(key.to_string(), serde_json::Value::Bool(true));
                    i = i.wrapping_add(1);
                } else {
                    map.insert(
                        key.to_string(),
                        serde_json::Value::String((*next).to_string()),
                    );
                    i = i.wrapping_add(2);
                }
            } else {
                // Last arg is a boolean flag
                map.insert(key.to_string(), serde_json::Value::Bool(true));
                i = i.wrapping_add(1);
            }
        } else if let Some(part) = parts.get(i) {
            positional.push(serde_json::Value::String((*part).to_string()));
            i = i.wrapping_add(1);
        } else {
            i = i.wrapping_add(1);
        }
    }

    if !positional.is_empty() {
        map.insert("args".to_string(), serde_json::Value::Array(positional));
    }

    serde_json::Value::Object(map)
}

/// Auto-detect whether input is natural language or a shell command.
///
/// Heuristic: if it contains a question mark or starts with common
/// conversational patterns, route to AI. Otherwise, treat as shell.
fn auto_detect_route(input: &str) -> RoutedCommand {
    let lower = input.to_lowercase();

    // Question mark → likely natural language
    if input.contains('?') {
        return RoutedCommand::Ai {
            message: input.to_string(),
            conversation_id: None,
        };
    }

    // Conversational starters
    let ai_patterns = [
        "what ",
        "how ",
        "why ",
        "when ",
        "where ",
        "who ",
        "which ",
        "explain ",
        "describe ",
        "analyze ",
        "compare ",
        "summarize ",
        "tell me ",
        "show me ",
        "help me ",
        "can you ",
        "please ",
    ];

    for pattern in &ai_patterns {
        if lower.starts_with(pattern) {
            return RoutedCommand::Ai {
                message: input.to_string(),
                conversation_id: None,
            };
        }
    }

    // Default: shell
    RoutedCommand::Shell(input.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nexvig_prefix_routes_to_mcp() {
        let cmd = route_command("nexvig> faers_search --drug aspirin", TerminalMode::Shell);
        if let RoutedCommand::Mcp { tool_name, params } = cmd {
            assert_eq!(tool_name, "faers_search");
            assert_eq!(params.get("drug").and_then(|v| v.as_str()), Some("aspirin"));
        } else {
            panic!("Expected MCP route");
        }
    }

    #[test]
    fn slash_prefix_routes_to_mcp() {
        let cmd = route_command("/pv_signal_complete aspirin", TerminalMode::Shell);
        if let RoutedCommand::Mcp { tool_name, params } = cmd {
            assert_eq!(tool_name, "pv_signal_complete");
            assert!(params.get("args").is_some());
        } else {
            panic!("Expected MCP route");
        }
    }

    #[test]
    fn claude_prefix_routes_to_ai() {
        let cmd = route_command("@claude what is the PRR for aspirin?", TerminalMode::Shell);
        if let RoutedCommand::Ai { message, .. } = cmd {
            assert_eq!(message, "what is the PRR for aspirin?");
        } else {
            panic!("Expected AI route");
        }
    }

    #[test]
    fn shell_mode_routes_to_shell() {
        let cmd = route_command("ls -la", TerminalMode::Shell);
        if let RoutedCommand::Shell(cmd_str) = cmd {
            assert_eq!(cmd_str, "ls -la");
        } else {
            panic!("Expected Shell route");
        }
    }

    #[test]
    fn hybrid_mode_question_routes_to_ai() {
        let cmd = route_command("What is the safety profile?", TerminalMode::Hybrid);
        assert!(matches!(cmd, RoutedCommand::Ai { .. }));
    }

    #[test]
    fn hybrid_mode_command_routes_to_shell() {
        let cmd = route_command("cargo build", TerminalMode::Hybrid);
        assert!(matches!(cmd, RoutedCommand::Shell(_)));
    }

    #[test]
    fn regulatory_mode_routes_to_mcp() {
        let cmd = route_command("faers_search --drug aspirin", TerminalMode::Regulatory);
        assert!(matches!(cmd, RoutedCommand::Mcp { .. }));
    }

    #[test]
    fn explicit_prefix_overrides_mode() {
        let cmd = route_command("@claude help", TerminalMode::Shell);
        assert!(matches!(cmd, RoutedCommand::Ai { .. }));
    }

    #[test]
    fn args_parsing_handles_flags() {
        let json = parse_args_to_json("--verbose --format json");
        assert_eq!(json.get("verbose"), Some(&serde_json::Value::Bool(true)));
        assert_eq!(json.get("format").and_then(|v| v.as_str()), Some("json"));
    }

    #[test]
    fn args_parsing_handles_positional() {
        let json = parse_args_to_json("aspirin hepatotoxicity");
        let args = json.get("args").and_then(|v| v.as_array());
        assert!(args.is_some());
        let args = args.unwrap_or(&Vec::new()).clone();
        assert_eq!(args.len(), 2);
    }
}
