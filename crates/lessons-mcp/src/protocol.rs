//! JSON-RPC protocol types
//! Tier: T2-P (wraps T1 serialization)

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

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
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Value, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(Error {
                code,
                message: message.into(),
            }),
        }
    }

    pub fn method_not_found(id: Value, method: &str) -> Self {
        Self::error(id, -32601, &format!("Method not found: {}", method))
    }

    pub fn parse_error(msg: &str) -> Self {
        Self::error(json!(null), -32700, &format!("Parse error: {}", msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_response() {
        let r = Response::success(json!(1), json!({"ok": true}));
        assert_eq!(r.jsonrpc, "2.0");
        assert!(r.result.is_some());
        assert!(r.error.is_none());
    }

    #[test]
    fn error_response() {
        let r = Response::error(json!(2), -32600, "bad request");
        assert!(r.result.is_none());
        assert!(r.error.is_some());
        assert_eq!(r.error.as_ref().map(|e| e.code), Some(-32600));
    }

    #[test]
    fn method_not_found_code() {
        let r = Response::method_not_found(json!(3), "foo");
        assert_eq!(r.error.as_ref().map(|e| e.code), Some(-32601));
    }

    #[test]
    fn parse_error_code() {
        let r = Response::parse_error("bad json");
        assert_eq!(r.error.as_ref().map(|e| e.code), Some(-32700));
        assert_eq!(r.id, json!(null));
    }
}
