//! posttool_edit_verifier - Auto-verify after Edit tool usage
//!
//! Fires on PostToolUse:Edit to run basic verification:
//! - Rust: cargo check on the edited file's crate
//! - TypeScript: tsc --noEmit
//! - Python: ruff check (warning only, 100% Rust policy)
//!
//! Protocol: Exit 0 (pass), warns on issues but doesn't block

use serde::Deserialize;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

#[derive(Deserialize)]
struct HookInput {
    tool_name: String,
    tool_input: ToolInput,
    tool_response: Option<ToolResponse>,
}

#[derive(Deserialize)]
struct ToolInput {
    file_path: Option<String>,
}

#[derive(Deserialize)]
struct ToolResponse {
    success: Option<bool>,
}

fn main() {
    let stdin = io::stdin();
    let input: HookInput = match serde_json::from_reader(stdin.lock()) {
        Ok(i) => i,
        Err(_) => std::process::exit(0), // Graceful degradation
    };

    // Only process Edit tool
    if input.tool_name != "Edit" {
        std::process::exit(0);
    }

    // Only verify if edit succeeded
    if let Some(ref resp) = input.tool_response {
        if resp.success == Some(false) {
            std::process::exit(0);
        }
    }

    let file_path = match input.tool_input.file_path {
        Some(p) => p,
        None => std::process::exit(0),
    };

    let path = Path::new(&file_path);
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    // Log to telemetry
    let telemetry_path = dirs::home_dir()
        .map(|h| h.join(".claude/brain/sessions/watchtower/hook_telemetry.jsonl"))
        .unwrap_or_default();

    let mut verification_result = String::new();

    match extension {
        "rs" => {
            // Find Cargo.toml by walking up
            if let Some(crate_dir) = find_cargo_toml(path) {
                let output = Command::new("cargo")
                    .arg("check")
                    .arg("--message-format=short")
                    .current_dir(&crate_dir)
                    .output();

                match output {
                    Ok(o) if o.status.success() => {
                        verification_result = format!("✅ Rust verified: {}", file_path);
                    }
                    Ok(o) => {
                        let stderr = String::from_utf8_lossy(&o.stderr);
                        let first_error = stderr.lines().take(3).collect::<Vec<_>>().join(" | ");
                        verification_result =
                            format!("⚠️ Rust check warnings: {} | {}", file_path, first_error);
                        eprintln!("{}", verification_result);
                    }
                    Err(e) => {
                        verification_result = format!("⚠️ cargo check failed: {}", e);
                    }
                }
            }
        }
        "ts" | "tsx" => {
            // TypeScript check
            if let Some(project_dir) = find_tsconfig(path) {
                let output = Command::new("npx")
                    .args(["tsc", "--noEmit", "--pretty", "false"])
                    .current_dir(&project_dir)
                    .output();

                match output {
                    Ok(o) if o.status.success() => {
                        verification_result = format!("✅ TypeScript verified: {}", file_path);
                    }
                    Ok(o) => {
                        let stdout = String::from_utf8_lossy(&o.stdout);
                        let errors = stdout.lines().take(2).collect::<Vec<_>>().join(" | ");
                        verification_result =
                            format!("⚠️ TypeScript warnings: {} | {}", file_path, errors);
                        eprintln!("{}", verification_result);
                    }
                    Err(_) => {} // tsc not available, skip
                }
            }
        }
        "py" => {
            // Python - warn about 100% Rust policy
            verification_result = format!(
                "⚠️ Python edit detected: {} - Flag for Rust migration",
                file_path
            );
            eprintln!("{}", verification_result);
        }
        _ => {
            // Other files - basic existence check
            if path.exists() {
                verification_result = format!("✅ Edit verified: {}", file_path);
            }
        }
    }

    // Write telemetry (best-effort, non-blocking)
    if !verification_result.is_empty() && telemetry_path.exists() {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .append(true)
            .open(&telemetry_path)
        {
            let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
            let entry = serde_json::json!({
                "timestamp": timestamp.to_string(),
                "hook": "posttool_edit_verifier",
                "event": "PostToolUse:Edit",
                "file": file_path,
                "result": verification_result
            });
            // Best-effort telemetry write - failure doesn't affect verification
            if let Err(e) = writeln!(file, "{}", entry) {
                eprintln!("Telemetry write failed (non-fatal): {}", e);
            }
        }
    }

    // Always exit 0 - verification is advisory
    std::process::exit(0);
}

fn find_cargo_toml(path: &Path) -> Option<std::path::PathBuf> {
    let mut current = path.parent()?;
    for _ in 0..10 {
        if current.join("Cargo.toml").exists() {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
    None
}

fn find_tsconfig(path: &Path) -> Option<std::path::PathBuf> {
    let mut current = path.parent()?;
    for _ in 0..10 {
        if current.join("tsconfig.json").exists() {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
    None
}
