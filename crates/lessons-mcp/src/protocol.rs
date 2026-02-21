//! JSON-RPC protocol types
//! Tier: T2-P (wraps T1 serialization)

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct Request {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Serialize)]
pub struct Response {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,
}

#[derive(Serialize)]
pub struct Error {
    pub code: i32,
    pub message: String,
}

impl Response {
    pub fn success(id: Value, result: Value) -> Self {
        Self { jsonrpc: "2.0".into(), id, result: Some(result), error: None }
    }

    pub fn error(id: Value, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(Error { code, message: message.into() }),
        }
    }

    pub fn method_not_found(id: Value, method: &str) -> Self {
        Self::error(id, -32601, &format!("Method not found: {}", method))
    }

    pub fn parse_error(msg: &str) -> Self {
        Self::error(json!(null), -32700, &format!("Parse error: {}", msg))
    }
}
