//! Claude MCP Config Library
//!
//! Tier: T3 (Domain-Specific Logic)

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde_json::{Value, json};

pub fn add_server_to_root(
    root: &mut Value,
    name: &str,
    command: &str,
    args: Vec<String>,
    desc: Option<String>,
) {
    if !root.is_object() {
        *root = json!({});
    }

    if let Some(obj_root) = root.as_object_mut() {
        let mcp = obj_root.entry("mcpServers").or_insert_with(|| json!({}));
        if let Some(obj) = mcp.as_object_mut() {
            obj.insert(
                name.to_string(),
                json!({
                    "command": command,
                    "args": args,
                    "env": {},
                    "description": desc.unwrap_or_default()
                }),
            );
        }
    }
}

pub fn allow_tool_pattern(root: &mut Value, pattern: &str) {
    if !root.is_object() {
        *root = json!({});
    }

    if let Some(obj_root) = root.as_object_mut() {
        let perms = obj_root.entry("permissions").or_insert_with(|| json!({}));
        if let Some(obj) = perms.as_object_mut() {
            let list = obj
                .entry("allow")
                .or_insert_with(|| Value::Array(Vec::new()));
            if let Some(arr) = list.as_array_mut() {
                if !arr.iter().any(|v| v.as_str() == Some(pattern)) {
                    arr.push(Value::String(pattern.to_string()));
                }
            }
        }
    }
}

#[cfg(test)]

mod tests {

    use super::*;

    #[test]

    fn test_add_server() {
        let mut root = json!({});

        add_server_to_root(&mut root, "t", "c", vec![], None);

        assert!(root["mcpServers"]["t"].is_object());
    }

    #[test]

    fn test_allow_tool() {
        let mut root = json!({});

        allow_tool_pattern(&mut root, "p1");

        allow_tool_pattern(&mut root, "p1"); // duplicate

        assert_eq!(
            root["permissions"]["allow"]
                .as_array()
                .expect("array")
                .len(),
            1
        ); // INVARIANT: test
    }
}
