//! Cargo Dependency Auditor - Validates crates exist on crates.io.
//!
//! PreToolUse hook that checks Cargo.toml edits to ensure:
//! 1. All dependencies exist on crates.io
//! 2. Dependencies have reasonable download counts (not abandoned)
//! 3. No wildcard versions are used

use nexcore_hooks::{exit_block, exit_success_auto, read_input};
use std::time::Duration;

/// Minimum downloads to consider a crate "established"
const MIN_DOWNLOADS: u64 = 1000;

/// Timeout for crates.io API calls
const API_TIMEOUT: Duration = Duration::from_secs(5);

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Skip API calls in plan mode
    if input.is_plan_mode() {
        exit_success_auto();
    }

    let file_path = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    // Only check Cargo.toml files
    if !file_path.ends_with("Cargo.toml") {
        exit_success_auto();
    }

    let content = match input.get_written_content() {
        Some(c) => c,
        None => exit_success_auto(),
    };

    let deps = extract_deps(content);
    if deps.is_empty() {
        exit_success_auto();
    }

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for (name, ver) in &deps {
        if ver == "*" {
            warnings.push(format!("{name}: wildcard version not recommended"));
        }
        match check_crate(name) {
            Ok(downloads) if downloads < MIN_DOWNLOADS => {
                warnings.push(format!("{name}: low downloads ({downloads})"));
            }
            Err(e) => errors.push(format!("{name}: {e}")),
            _ => {}
        }
    }

    if !errors.is_empty() {
        let mut msg = format!(
            "DEPENDENCY AUDIT FAILED\n\nFound {} error(s):\n",
            errors.len()
        );
        for e in &errors {
            msg.push_str(&format!("  - {e}\n"));
        }
        if !warnings.is_empty() {
            msg.push_str(&format!("\nWarnings ({}):\n", warnings.len()));
            for w in &warnings {
                msg.push_str(&format!("  - {w}\n"));
            }
        }
        msg.push_str("\nVerify crate names at https://crates.io before proceeding.");
        exit_block(&msg);
    }

    exit_success_auto();
}

/// Extract external dependencies from Cargo.toml content
///
/// Skips workspace-internal dependencies:
/// - `path = "..."` dependencies (local crates)
/// - `workspace = true` dependencies (inherited from workspace root)
fn extract_deps(content: &str) -> Vec<(String, String)> {
    let parsed: toml::Table = match toml::from_str(content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    // Use flat_map to avoid nested loops - O(n) where n = total deps
    ["dependencies", "dev-dependencies", "build-dependencies"]
        .iter()
        .filter_map(|section| parsed.get(*section))
        .filter_map(|v| v.as_table())
        .flat_map(|table| table.iter())
        .filter_map(|(name, val)| parse_dep_entry(name, val))
        .collect()
}

/// Parse a single dependency entry, returning None for workspace-internal deps
fn parse_dep_entry(name: &str, val: &toml::Value) -> Option<(String, String)> {
    match val {
        toml::Value::String(s) => {
            // Simple version string - external crate
            Some((name.to_string(), s.to_string()))
        }
        toml::Value::Table(t) => {
            // Skip path dependencies (workspace-internal)
            if t.contains_key("path") {
                return None;
            }
            // Skip workspace = true dependencies (inherited)
            if t.get("workspace").and_then(|v| v.as_bool()) == Some(true) {
                return None;
            }
            // External crate with features/version table
            let ver = t
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("*")
                .to_string();
            Some((name.to_string(), ver))
        }
        _ => None,
    }
}

/// Check if a crate exists on crates.io and get download count
fn check_crate(name: &str) -> Result<u64, String> {
    let url = format!("https://crates.io/api/v1/crates/{name}");

    // ureq v3: timeout is set on agent, not request
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(API_TIMEOUT))
        .user_agent("nexcore-hooks/1.0")
        .build()
        .new_agent();

    let resp = agent.get(&url).call().map_err(|e| match e {
        ureq::Error::StatusCode(404) => "not found on crates.io".to_string(),
        _ => format!("HTTP error: {e}"),
    })?;

    let body = resp
        .into_body()
        .read_to_string()
        .map_err(|e| format!("read error: {e}"))?;

    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("JSON error: {e}"))?;

    let downloads = json
        .get("crate")
        .and_then(|c| c.get("downloads"))
        .and_then(|d| d.as_u64())
        .unwrap_or(0);

    Ok(downloads)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_deps() {
        // Note: no leading newline - toml parser is strict
        let content = r#"[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
"#;
        let deps = extract_deps(content);
        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|(n, _)| n == "serde"));
        assert!(deps.iter().any(|(n, _)| n == "tokio"));
    }

    #[test]
    fn test_skip_workspace_deps() {
        // Workspace deps inherit from [workspace.dependencies] - skip them
        let content = r#"[dependencies]
serde = { workspace = true }
tokio = "1.0"
"#;
        let deps = extract_deps(content);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].0, "tokio");
    }

    #[test]
    fn test_skip_path_deps() {
        // Path deps are workspace-internal - skip them
        let content = r#"[dependencies]
nexcore-foundation = { path = "../nexcore-foundation" }
serde = "1.0"
"#;
        let deps = extract_deps(content);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].0, "serde");
    }

    #[test]
    fn test_skip_mixed_internal_deps() {
        // Combined test for all internal dep types
        let content = r#"[dependencies]
internal-path = { path = "../internal" }
internal-workspace = { workspace = true }
external-simple = "1.0"
external-features = { version = "2.0", features = ["full"] }

[dev-dependencies]
test-helper = { path = "../test-helper" }
proptest = "1.0"
"#;
        let deps = extract_deps(content);
        assert_eq!(deps.len(), 3);
        assert!(deps.iter().any(|(n, _)| n == "external-simple"));
        assert!(deps.iter().any(|(n, _)| n == "external-features"));
        assert!(deps.iter().any(|(n, _)| n == "proptest"));
        // Internal deps should NOT be present
        assert!(!deps.iter().any(|(n, _)| n == "internal-path"));
        assert!(!deps.iter().any(|(n, _)| n == "internal-workspace"));
        assert!(!deps.iter().any(|(n, _)| n == "test-helper"));
    }
}
