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
    /// MCP tool invocation — dispatch via in-process NexCore bridge.
    Mcp {
        /// Tool name (e.g., "faers_search").
        tool_name: String,
        /// Tool parameters as JSON.
        params: serde_json::Value,
    },
    /// NexVigilant Station tool — dispatch via HTTP to mcp.nexvigilant.com.
    Station {
        /// Tool name (e.g., "search_adverse_events").
        tool_name: String,
        /// Tool parameters as JSON.
        params: serde_json::Value,
    },
    /// Microgram decision tree — dispatch via rsk subprocess.
    Microgram {
        /// Microgram name (e.g., "prr-signal").
        name: String,
        /// Input variables as JSON.
        variables: serde_json::Value,
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

    // Station prefix: route to NexVigilant Station (mcp.nexvigilant.com)
    if let Some(rest) = trimmed
        .strip_prefix("station>")
        .or_else(|| trimmed.strip_prefix("station> "))
    {
        return parse_station_command(rest.trim());
    }

    // Microgram prefix: route to rsk kernel decision trees
    if let Some(rest) = trimmed
        .strip_prefix("mcg>")
        .or_else(|| trimmed.strip_prefix("mcg> "))
    {
        return parse_microgram_command(rest.trim());
    }

    // NexCore MCP prefix
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

/// Parse a string as a Station tool command: `tool_name arg1 --flag value`.
///
/// Uses the same arg parsing as MCP commands but routes to Station backend.
fn parse_station_command(input: &str) -> RoutedCommand {
    let trimmed = input.trim();
    let (tool_name, rest) = match trimmed.split_once(' ') {
        Some((name, args)) => (name.to_string(), args.trim()),
        None => (trimmed.to_string(), ""),
    };

    let params = if rest.is_empty() {
        serde_json::Value::Object(serde_json::Map::new())
    } else {
        parse_args_to_json(rest)
    };

    RoutedCommand::Station { tool_name, params }
}

/// Parse a microgram command: `name key=value key=value`.
///
/// Supports both `key=value` pairs and `--key value` flags, converted to
/// JSON variables for the microgram runtime.
fn parse_microgram_command(input: &str) -> RoutedCommand {
    let trimmed = input.trim();
    let (name, rest) = match trimmed.split_once(' ') {
        Some((n, args)) => (n.to_string(), args.trim()),
        None => (trimmed.to_string(), ""),
    };

    let variables = if rest.is_empty() {
        serde_json::Value::Object(serde_json::Map::new())
    } else {
        parse_microgram_args(rest)
    };

    RoutedCommand::Microgram { name, variables }
}

/// Parse microgram-style args: `drug=metformin event=nausea count=42`.
///
/// Supports `key=value` (preferred for mcg) and falls back to `--key value`.
fn parse_microgram_args(input: &str) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    let parts: Vec<&str> = input.split_whitespace().collect();
    let mut i = 0;

    while i < parts.len() {
        if let Some(part) = parts.get(i) {
            if let Some((key, value)) = part.split_once('=') {
                // key=value syntax
                let json_val = if let Ok(n) = value.parse::<f64>() {
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(n)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    )
                } else if value == "true" {
                    serde_json::Value::Bool(true)
                } else if value == "false" {
                    serde_json::Value::Bool(false)
                } else {
                    serde_json::Value::String(value.to_string())
                };
                map.insert(key.to_string(), json_val);
                i = i.wrapping_add(1);
            } else if let Some(key) = part.strip_prefix("--") {
                // --key value fallback
                if let Some(next) = parts.get(i.wrapping_add(1)) {
                    if next.starts_with("--") || next.contains('=') {
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
                    map.insert(key.to_string(), serde_json::Value::Bool(true));
                    i = i.wrapping_add(1);
                }
            } else {
                // Positional: treat as microgram name args (rare)
                i = i.wrapping_add(1);
            }
        } else {
            i = i.wrapping_add(1);
        }
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
    fn station_prefix_routes_to_station() {
        let cmd = route_command(
            "station> search_adverse_events --drug metformin",
            TerminalMode::Shell,
        );
        if let RoutedCommand::Station { tool_name, params } = cmd {
            assert_eq!(tool_name, "search_adverse_events");
            assert_eq!(
                params.get("drug").and_then(|v| v.as_str()),
                Some("metformin")
            );
        } else {
            panic!("Expected Station route");
        }
    }

    #[test]
    fn station_prefix_with_space() {
        let cmd = route_command(
            "station> compute_prr --drug aspirin --event rash",
            TerminalMode::Shell,
        );
        if let RoutedCommand::Station { tool_name, params } = cmd {
            assert_eq!(tool_name, "compute_prr");
            assert_eq!(params.get("drug").and_then(|v| v.as_str()), Some("aspirin"));
            assert_eq!(params.get("event").and_then(|v| v.as_str()), Some("rash"));
        } else {
            panic!("Expected Station route");
        }
    }

    #[test]
    fn mcg_prefix_routes_to_microgram() {
        let cmd = route_command(
            "mcg> prr-signal drug=metformin event=nausea",
            TerminalMode::Shell,
        );
        if let RoutedCommand::Microgram { name, variables } = cmd {
            assert_eq!(name, "prr-signal");
            assert_eq!(
                variables.get("drug").and_then(|v| v.as_str()),
                Some("metformin")
            );
            assert_eq!(
                variables.get("event").and_then(|v| v.as_str()),
                Some("nausea")
            );
        } else {
            panic!("Expected Microgram route");
        }
    }

    #[test]
    fn mcg_prefix_parses_numeric_values() {
        let cmd = route_command("mcg> prr-signal a=10 b=20 c=30 d=40", TerminalMode::Shell);
        if let RoutedCommand::Microgram { variables, .. } = cmd {
            assert_eq!(variables.get("a").and_then(|v| v.as_f64()), Some(10.0));
            assert_eq!(variables.get("d").and_then(|v| v.as_f64()), Some(40.0));
        } else {
            panic!("Expected Microgram route");
        }
    }

    #[test]
    fn mcg_no_args() {
        let cmd = route_command("mcg> workflow-router", TerminalMode::Shell);
        if let RoutedCommand::Microgram { name, variables } = cmd {
            assert_eq!(name, "workflow-router");
            assert!(variables.as_object().map_or(false, |m| m.is_empty()));
        } else {
            panic!("Expected Microgram route");
        }
    }

    #[test]
    fn station_prefix_overrides_any_mode() {
        let cmd = route_command("station> get_drug_label --name aspirin", TerminalMode::Ai);
        assert!(matches!(cmd, RoutedCommand::Station { .. }));
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
