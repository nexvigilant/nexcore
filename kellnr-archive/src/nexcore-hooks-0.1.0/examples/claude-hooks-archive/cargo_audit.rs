use claude_hooks::{
    exit_success, read_input, write_output,
    input::{PreToolUseInput, ToolInput},
    output::PreToolUseOutput,
    HookResult,
};
use std::fs;
use std::process::Command;

fn main() -> HookResult<()> {
    let input: PreToolUseInput = read_input()?;

    // Parse the raw tool_input JSON into the structured ToolInput enum
    let tool_input: ToolInput = serde_json::from_value(input.tool_input.clone())
        .unwrap_or(ToolInput::Other(input.tool_input.clone()));

    // Only process Cargo.toml
    let path = match tool_input.file_path() {
        Some(p) if p.ends_with("Cargo.toml") => p,
        _ => exit_success(),
    };

    // Get the content as it would be after the tool executes
    let content = match &tool_input {
        ToolInput::Write(w) => w.content.clone(),
        ToolInput::Edit(e) => {
            let current = fs::read_to_string(path).unwrap_or_default();
            if e.replace_all.unwrap_or(false) {
                current.replace(&e.old_string, &e.new_string)
            } else {
                current.replacen(&e.old_string, &e.new_string, 1)
            }
        }
        _ => exit_success(),
    };

    let deps = extract_deps(&content);
    if deps.is_empty() {
        exit_success();
    }

    let mut missing = Vec::new();
    for name in &deps {
        // Skip common/known crates to speed up
        if is_builtin_or_common(name) {
            continue;
        }

        if !verify_crate_exists(name) {
            missing.push(name.clone());
        }
    }

    if !missing.is_empty() {
        let msg = format!(
            "CARGO DEPENDENCY AUDIT FAILED\n\n\
             The following crates were not found on crates.io:\n\
             - {}\n\n\
             Please verify the crate names before adding them to Cargo.toml.",
            missing.join("\n- ")
        );
        let output = PreToolUseOutput::deny(msg);
        write_output(&output)?;
        return Ok(());
    }

    exit_success();
}

fn is_builtin_or_common(name: &str) -> bool {
    let common = [
        "serde", "serde_json", "tokio", "anyhow", "thiserror", 
        "regex", "chrono", "itertools", "clap", "log"
    ];
    common.contains(&name)
}

fn verify_crate_exists(name: &str) -> bool {
    // We use cargo search as a lightweight way to check crates.io
    let output = Command::new("cargo")
        .args(["search", "--limit", "1", name])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Search returns matching crates; we look for the exact name
            stdout.lines().any(|line| line.trim().starts_with(name))
        }
        _ => false,
    }
}

fn extract_deps(content: &str) -> Vec<String> {
    let mut names = Vec::new();
    let parsed: toml::Value = match toml::from_str(content) {
        Ok(v) => v,
        Err(_) => return names,
    };

    if let Some(deps) = parsed.get("dependencies").and_then(|v| v.as_table()) {
        for (name, val) in deps {
            // Skip workspace or path dependencies
            if let Some(table) = val.as_table() {
                if table.contains_key("path") || table.contains_key("workspace") {
                    continue;
                }
            }
            names.push(name.clone());
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_deps() {
        let content = r#"[package]
name = "test"
[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
internal = { path = "../internal" }
"#;
        let deps = extract_deps(content);
        assert!(deps.contains(&"serde".to_string()));
        assert!(deps.contains(&"tokio".to_string()));
        assert!(!deps.contains(&"internal".to_string()));
    }
}
