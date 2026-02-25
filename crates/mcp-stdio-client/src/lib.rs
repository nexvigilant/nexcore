//! MCP Stdio Client Library
//!
//! Tier: T3 (Domain-Specific Logic)

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use serde_json::Value;

pub fn summarize(value: &Value) -> String {
    if let Some(result) = value.get("result") {
        if let Some(info) = result.get("serverInfo") {
            return info.to_string();
        }

        return result.to_string();
    }

    value.to_string()
}

pub fn format_output(value: &Value, raw: bool) -> String {
    if raw {
        return value.to_string();
    }

    if let Some(result) = value.get("result") {
        if let Some(content) = result.get("content") {
            return content.to_string();
        }

        return result.to_string();
    }

    value.to_string()
}

#[cfg(test)]

mod tests {

    use super::*;

    use serde_json::json;

    #[test]

    fn test_summarize_server_info() {
        let val = json!({"result": {"serverInfo": {"name": "test", "version": "1"}}});

        assert!(summarize(&val).contains("test"));
    }

    #[test]

    fn test_summarize_no_result() {
        let val = json!({"error": "oops"});

        assert_eq!(summarize(&val), val.to_string());
    }

    #[test]

    fn test_format_output_raw() {
        let val = json!({"a": 1});

        assert_eq!(format_output(&val, true), val.to_string());
    }

    #[test]

    fn test_format_output_content() {
        let val = json!({"result": {"content": "hello"}});

        assert_eq!(format_output(&val, false), "\"hello\"");
    }
}
