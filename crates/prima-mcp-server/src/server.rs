// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # MCP Protocol Server
//!
//! Stdio JSON-RPC handler for MCP protocol.
//!
//! ## Tier: T2-C (σ + μ + → + π)
//!
//! ## Protocol
//! - `initialize` → returns tool catalog
//! - `tools/list` → returns available tools
//! - `tools/call` → executes Prima function, returns result

use crate::codegen::{self, codegen_tools};
use crate::executor::{Executor, ExecutorError};
use prima_mcp::compile;
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use thiserror::Error;

/// Server errors.
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Executor error: {0}")]
    Executor(#[from] ExecutorError),
    #[error("Unknown method: {0}")]
    UnknownMethod(String),
    #[error("Invalid request")]
    InvalidRequest,
}

/// JSON-RPC request.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Request {
    jsonrpc: String,
    id: Option<JsonValue>,
    method: String,
    #[serde(default)]
    params: JsonValue,
}

/// JSON-RPC response.
#[derive(Debug, Serialize, Deserialize)]
struct Response {
    jsonrpc: String,
    id: JsonValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonValue>,
}

/// MCP Server.
///
/// ## Tier: T2-C (σ + μ + → + π)
pub struct Server {
    /// Prima source.
    source: String,
    /// Tool prefix.
    prefix: String,
    /// Executor instance.
    executor: Executor,
}

impl Server {
    /// Create new server from Prima source.
    pub fn new(source: &str, prefix: &str) -> Self {
        let executor = Executor::new(source);
        Self {
            source: source.to_string(),
            prefix: prefix.to_string(),
            executor,
        }
    }

    /// Run the server on stdio.
    pub fn run<R: BufRead, W: Write>(&self, reader: R, mut writer: W) -> Result<(), ServerError> {
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let request: Request = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    let error_response = Response {
                        jsonrpc: "2.0".to_string(),
                        id: JsonValue::Null,
                        result: None,
                        error: Some(json!({
                            "code": -32700,
                            "message": format!("Parse error: {}", e)
                        })),
                    };
                    writeln!(writer, "{}", serde_json::to_string(&error_response)?)?;
                    writer.flush()?;
                    continue;
                }
            };

            let response = self.handle_request(&request);
            writeln!(writer, "{}", serde_json::to_string(&response)?)?;
            writer.flush()?;
        }

        Ok(())
    }

    /// Handle a single request.
    fn handle_request(&self, request: &Request) -> Response {
        let id = request.id.clone().unwrap_or(JsonValue::Null);

        match request.method.as_str() {
            "initialize" => Response {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "prima-mcp-server",
                        "version": "0.1.0"
                    }
                })),
                error: None,
            },

            "tools/list" => {
                let catalog = compile(&self.source, &self.prefix);
                let empty_tools: Vec<JsonValue> = vec![];
                let prima_tools = catalog["tools"].as_array().unwrap_or(&empty_tools);

                // Combine Prima function tools with codegen tools
                let mut all_tools: Vec<JsonValue> = prima_tools.clone();
                all_tools.extend(codegen_tools());

                Response {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({ "tools": all_tools })),
                    error: None,
                }
            }

            "tools/call" => {
                let tool_name = request.params["name"].as_str().unwrap_or("");
                let arguments = &request.params["arguments"];

                // Handle codegen tools first
                match tool_name {
                    "prima_codegen" => {
                        let source = arguments["source"].as_str().unwrap_or("");
                        let target = arguments["target"].as_str().unwrap_or("rust");
                        match codegen::generate(source, target) {
                            Ok(result) => Response {
                                jsonrpc: "2.0".to_string(),
                                id,
                                result: Some(json!({
                                    "content": [{
                                        "type": "text",
                                        "text": serde_json::to_string_pretty(&result).unwrap_or_default()
                                    }]
                                })),
                                error: None,
                            },
                            Err(e) => Response {
                                jsonrpc: "2.0".to_string(),
                                id,
                                result: None,
                                error: Some(json!({
                                    "code": -32603,
                                    "message": format!("{}", e)
                                })),
                            },
                        }
                    }
                    "prima_targets" => Response {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: Some(json!({
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string_pretty(&codegen::list_targets()).unwrap_or_default()
                            }]
                        })),
                        error: None,
                    },
                    "prima_primitives" => {
                        let source = arguments["source"].as_str().unwrap_or("");
                        match codegen::list_primitives(source) {
                            Ok(result) => Response {
                                jsonrpc: "2.0".to_string(),
                                id,
                                result: Some(json!({
                                    "content": [{
                                        "type": "text",
                                        "text": serde_json::to_string_pretty(&result).unwrap_or_default()
                                    }]
                                })),
                                error: None,
                            },
                            Err(e) => Response {
                                jsonrpc: "2.0".to_string(),
                                id,
                                result: None,
                                error: Some(json!({
                                    "code": -32603,
                                    "message": format!("{}", e)
                                })),
                            },
                        }
                    }
                    // Fall through to Prima function execution
                    _ => {
                        // Strip prefix to get function name
                        let function_name = tool_name
                            .strip_prefix(&self.prefix)
                            .and_then(|s| s.strip_prefix("_"))
                            .unwrap_or(tool_name);

                        let params: HashMap<String, JsonValue> = if arguments.is_object() {
                            serde_json::from_value(arguments.clone()).unwrap_or_default()
                        } else {
                            HashMap::new()
                        };

                        match self.executor.execute(function_name, &params) {
                            Ok(result) => Response {
                                jsonrpc: "2.0".to_string(),
                                id,
                                result: Some(json!({
                                    "content": [{
                                        "type": "text",
                                        "text": serde_json::to_string_pretty(&result).unwrap_or_default()
                                    }]
                                })),
                                error: None,
                            },
                            Err(e) => Response {
                                jsonrpc: "2.0".to_string(),
                                id,
                                result: None,
                                error: Some(json!({
                                    "code": -32603,
                                    "message": format!("{}", e)
                                })),
                            },
                        }
                    }
                }
            }

            "notifications/initialized" => Response {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({})),
                error: None,
            },

            method => Response {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(json!({
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                })),
            },
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn make_request(method: &str, params: JsonValue) -> String {
        let req = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });
        format!("{}\n", serde_json::to_string(&req).unwrap_or_default())
    }

    #[test]
    fn test_server_initialize() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let server = Server::new(source, "prima");

        let input = make_request("initialize", json!({}));
        let reader = Cursor::new(input);
        let mut output = Vec::new();

        let result = server.run(reader, &mut output);
        assert!(result.is_ok());

        let response: Response = serde_json::from_slice(&output).unwrap_or(Response {
            jsonrpc: "2.0".to_string(),
            id: JsonValue::Null,
            result: None,
            error: Some(json!({"message": "parse failed"})),
        });
        assert!(response.result.is_some());
    }

    #[test]
    fn test_server_tools_list() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let server = Server::new(source, "prima");

        let input = make_request("tools/list", json!({}));
        let reader = Cursor::new(input);
        let mut output = Vec::new();

        let result = server.run(reader, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("prima_add"));
    }

    #[test]
    fn test_server_tools_call() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let server = Server::new(source, "prima");

        let input = make_request(
            "tools/call",
            json!({
                "name": "prima_add",
                "arguments": {"a": 3, "b": 4}
            }),
        );
        let reader = Cursor::new(input);
        let mut output = Vec::new();

        let result = server.run(reader, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("7"));
    }

    #[test]
    fn test_server_unknown_method() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let server = Server::new(source, "prima");

        let input = make_request("unknown/method", json!({}));
        let reader = Cursor::new(input);
        let mut output = Vec::new();

        let result = server.run(reader, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("Method not found"));
    }

    #[test]
    fn test_server_codegen_rust() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let server = Server::new(source, "prima");

        let input = make_request(
            "tools/call",
            json!({
                "name": "prima_codegen",
                "arguments": {
                    "source": "μ add(a: N, b: N) → N { a + b }",
                    "target": "rust"
                }
            }),
        );
        let reader = Cursor::new(input);
        let mut output = Vec::new();

        let result = server.run(reader, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("fn add"));
        assert!(output_str.contains("transfer_confidence"));
    }

    #[test]
    fn test_server_codegen_python() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let server = Server::new(source, "prima");

        let input = make_request(
            "tools/call",
            json!({
                "name": "prima_codegen",
                "arguments": {
                    "source": "μ add(a: N, b: N) → N { a + b }",
                    "target": "python"
                }
            }),
        );
        let reader = Cursor::new(input);
        let mut output = Vec::new();

        let result = server.run(reader, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("def add"));
    }

    #[test]
    fn test_server_targets() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let server = Server::new(source, "prima");

        let input = make_request(
            "tools/call",
            json!({
                "name": "prima_targets",
                "arguments": {}
            }),
        );
        let reader = Cursor::new(input);
        let mut output = Vec::new();

        let result = server.run(reader, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("rust"));
        assert!(output_str.contains("python"));
        assert!(output_str.contains("typescript"));
        assert!(output_str.contains("go"));
        assert!(output_str.contains("c"));
    }

    #[test]
    fn test_server_primitives() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let server = Server::new(source, "prima");

        let input = make_request(
            "tools/call",
            json!({
                "name": "prima_primitives",
                "arguments": {
                    "source": "μ add(a: N, b: N) → N { a + b }"
                }
            }),
        );
        let reader = Cursor::new(input);
        let mut output = Vec::new();

        let result = server.run(reader, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("primitives"));
    }

    #[test]
    fn test_server_tools_list_includes_codegen() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let server = Server::new(source, "prima");

        let input = make_request("tools/list", json!({}));
        let reader = Cursor::new(input);
        let mut output = Vec::new();

        let result = server.run(reader, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8_lossy(&output);
        // Should include both Prima function tools and codegen tools
        assert!(output_str.contains("prima_add"));
        assert!(output_str.contains("prima_codegen"));
        assert!(output_str.contains("prima_targets"));
        assert!(output_str.contains("prima_primitives"));
    }
}
