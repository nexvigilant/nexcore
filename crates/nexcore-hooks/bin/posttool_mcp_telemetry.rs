//! MCP Tool Telemetry Logger
//!
//! PostToolUse hook that logs MCP tool calls for observability.
//!
//! # Event
//! PostToolUse (mcp__* tools only)
//!
//! # Purpose
//! Captures telemetry for MCP tool usage including:
//! - Tool name and server
//! - Arguments passed
//! - Response summary (truncated)
//!
//! # Output
//! Writes to watchtower telemetry system at ~/.claude/telemetry/
//!
//! # Exit Codes
//! - 0: Always (non-blocking)

use nexcore_hooks::{HookTelemetry, read_input};
use serde_json::{Value, json};

/// Maximum length for argument/response summaries
const MAX_SUMMARY_LEN: usize = 500;

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => std::process::exit(0),
    };

    // Only process MCP tool calls
    let tool_name = match input.tool_name.as_deref() {
        Some(name) if name.starts_with("mcp__") => name,
        _ => std::process::exit(0),
    };

    // Parse MCP tool name components
    let parts: Vec<&str> = tool_name.split("__").collect();
    let (server, tool) = if parts.len() >= 3 {
        (parts[1], parts[2..].join("__"))
    } else {
        ("unknown", tool_name.to_string())
    };

    // Extract argument summary
    let args_summary = input
        .tool_input
        .as_ref()
        .map(|v| summarize_value(v, MAX_SUMMARY_LEN))
        .unwrap_or_default();

    // Extract response summary
    let response_summary = input
        .tool_response
        .as_ref()
        .map(|v| summarize_response(v))
        .unwrap_or_else(|| "no_response".to_string());

    // Emit telemetry
    HookTelemetry::new("posttool_mcp_telemetry", "PostToolUse")
        .with_tool(tool_name)
        .with_session(&input.session_id)
        .with_result(&response_summary)
        .with_extra(json!({
            "mcp_server": server,
            "mcp_tool": tool,
            "args": args_summary,
            "cwd": input.cwd
        }))
        .emit();

    std::process::exit(0);
}

/// Summarize a JSON value to a string with length limit
fn summarize_value(value: &Value, max_len: usize) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            if s.len() > max_len {
                format!("{}...", &s[..max_len])
            } else {
                s.clone()
            }
        }
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().take(5).map(|v| summarize_value(v, 50)).collect();
            let summary = format!("[{}]", items.join(", "));
            if arr.len() > 5 {
                format!("{} (+{} more)", summary, arr.len() - 5)
            } else {
                summary
            }
        }
        Value::Object(obj) => {
            // For MCP args, extract key parameters
            let mut parts = Vec::new();
            for (key, val) in obj.iter().take(10) {
                let val_summary = match val {
                    Value::String(s) => {
                        if s.len() > 50 {
                            format!("\"{}...\"", &s[..50])
                        } else {
                            format!("\"{}\"", s)
                        }
                    }
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Array(a) => format!("[{} items]", a.len()),
                    Value::Object(_) => "{...}".to_string(),
                    Value::Null => "null".to_string(),
                };
                parts.push(format!("{}={}", key, val_summary));
            }
            let summary = parts.join(", ");
            if summary.len() > max_len {
                format!("{}...", &summary[..max_len])
            } else if obj.len() > 10 {
                format!("{} (+{} more fields)", summary, obj.len() - 10)
            } else {
                summary
            }
        }
    }
}

/// Summarize response - focus on success/error status
fn summarize_response(value: &Value) -> String {
    if let Some(obj) = value.as_object() {
        // Check for error fields
        if let Some(error) = obj.get("error") {
            return format!("error: {}", summarize_value(error, 100));
        }

        // Check for success indicators
        if let Some(success) = obj.get("success") {
            if success.as_bool() == Some(true) {
                return "success".to_string();
            } else if success.as_bool() == Some(false) {
                return "failed".to_string();
            }
        }

        // Check result field
        if let Some(result) = obj.get("result") {
            let summary = summarize_value(result, 100);
            return format!("result: {}", summary);
        }

        // Default: show first few keys
        let keys: Vec<&String> = obj.keys().take(5).collect();
        format!(
            "keys: [{}]",
            keys.iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    } else {
        summarize_value(value, 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_string() {
        let val = Value::String("hello".to_string());
        assert_eq!(summarize_value(&val, 100), "hello");

        let long_val = Value::String("a".repeat(200));
        let summary = summarize_value(&long_val, 50);
        assert!(summary.len() <= 54); // 50 + "..."
    }

    #[test]
    fn test_summarize_object() {
        let val = json!({
            "source": "kitten",
            "target": "sitting"
        });
        let summary = summarize_value(&val, 500);
        assert!(summary.contains("source"));
        assert!(summary.contains("kitten"));
    }

    #[test]
    fn test_summarize_response_success() {
        let val = json!({"success": true, "data": [1,2,3]});
        assert_eq!(summarize_response(&val), "success");
    }

    #[test]
    fn test_summarize_response_error() {
        let val = json!({"error": "not found"});
        assert!(summarize_response(&val).contains("error"));
    }
}
