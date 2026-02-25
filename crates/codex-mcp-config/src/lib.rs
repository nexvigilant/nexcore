//! Codex MCP Config Library
//!
//! Tier: T3 (Domain-Specific Logic)

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use nexcore_error::Result;

use toml_edit::{DocumentMut, Item, value};

pub fn set_stdio_server(doc: &mut DocumentMut, name: &str, command: &str, args: &[&str]) {
    doc["mcp_servers"][name]["command"] = value(command);

    let mut arr = toml_edit::Array::new();

    for arg in args {
        arr.push(*arg);
    }

    doc["mcp_servers"][name]["args"] = Item::Value(toml_edit::Value::Array(arr));

    doc["mcp_servers"][name]["env"] = toml_edit::table();
}

pub fn update_mcp_config_path(doc: &mut DocumentMut, global_mcp: &str) {
    if !doc.contains_table("mcp") {
        doc["mcp"] = toml_edit::table();
    }

    doc["mcp"]["config_path"] = value(global_mcp);
}

#[cfg(test)]

mod tests {

    use super::*;

    #[test]

    fn test_set_stdio_server() {
        let mut doc = DocumentMut::new();

        doc["mcp_servers"] = toml_edit::table();

        set_stdio_server(&mut doc, "test", "/bin/ls", &["-l"]);

        assert_eq!(
            doc["mcp_servers"]["test"]["command"].as_str(),
            Some("/bin/ls")
        );
    }

    #[test]

    fn test_update_config_path() {
        let mut doc = DocumentMut::new();

        update_mcp_config_path(&mut doc, "/p/m.json");

        assert_eq!(doc["mcp"]["config_path"].as_str(), Some("/p/m.json"));
    }
}
