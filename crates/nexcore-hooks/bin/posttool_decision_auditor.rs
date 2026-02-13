//! Decision Auditor Hook
//!
//! PostToolUse hook that tracks significant decisions for the 50/50 governance model.
//!
//! # Event
//! PostToolUse
//!
//! # Purpose
//! Logs all significant decisions (file writes, edits, bash commands) with
//! rationale tracking. Enables CEO review of CAIO decision-making.
//!
//! # Tracked Decisions
//! - Write: New file creation
//! - Edit: Code modifications
//! - Bash: System commands
//!
//! # Output
//! Appends to ~/.claude/decision-audit/decisions.jsonl
//!
//! # Partnership Protocol
//! Both parties can review the decision log. Supports the mutual trust model
//! by making all CAIO actions auditable.
//!
//! # Exit Codes
//! - 0: Always (audit is non-blocking)

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct Decision {
    timestamp: String,
    session_id: String,
    tool: String,
    action: String,
    target: String,
    risk_level: String,
    reversible: bool,
}

fn audit_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".claude").join("decision-audit")
}

fn assess_risk(tool: &str, input: &serde_json::Value) -> (&'static str, bool) {
    match tool {
        "Write" => {
            let path = input
                .get("file_path")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if path.contains(".env") || path.contains("secret") || path.contains("credential") {
                ("HIGH", true)
            } else if path.contains("Cargo.toml") || path.contains("CLAUDE.md") {
                ("MEDIUM", true)
            } else {
                ("LOW", true)
            }
        }
        "Edit" => ("LOW", true),
        "Bash" => {
            let cmd = input.get("command").and_then(|v| v.as_str()).unwrap_or("");
            if cmd.contains("rm ") || cmd.contains("delete") || cmd.contains("drop") {
                ("HIGH", false)
            } else if cmd.contains("git push") || cmd.contains("deploy") {
                ("MEDIUM", false)
            } else {
                ("LOW", true)
            }
        }
        _ => ("LOW", true),
    }
}

fn main() {
    let mut buffer = String::new();
    if std::io::Read::read_to_string(&mut std::io::stdin(), &mut buffer).is_err() {
        std::process::exit(0);
    }
    let input: serde_json::Value = serde_json::from_str(&buffer).unwrap_or_default();

    let tool_name = input
        .get("tool_name")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Only audit significant tools
    let significant_tools = ["Write", "Edit", "Bash"];
    if !significant_tools.contains(&tool_name) {
        std::process::exit(0);
    }

    let session_id = input
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let tool_input = input.get("tool_input").cloned().unwrap_or_default();
    let (risk_level, reversible) = assess_risk(tool_name, &tool_input);

    let target = match tool_name {
        "Write" | "Edit" => tool_input
            .get("file_path")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        "Bash" => tool_input
            .get("command")
            .and_then(|v| v.as_str())
            .map(|s| s.chars().take(100).collect())
            .unwrap_or_else(|| "unknown".to_string()),
        _ => "unknown".to_string(),
    };

    let action = [tool_name, " operation"].concat();

    let decision = Decision {
        timestamp: Utc::now().to_rfc3339(),
        session_id,
        tool: tool_name.to_string(),
        action,
        target: target.clone(),
        risk_level: risk_level.to_string(),
        reversible,
    };

    // Append to audit log
    let dir = audit_dir();
    if fs::create_dir_all(&dir).is_ok() {
        let path = dir.join("decisions.jsonl");
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            if let Ok(json) = serde_json::to_string(&decision) {
                if let Err(e) = writeln!(file, "{}", json) {
                    eprintln!("Warning: Could not write decision audit: {}", e);
                }
            }
        }
    }

    // Warn on high-risk decisions
    if risk_level == "HIGH" {
        eprintln!("⚠️ HIGH-RISK DECISION: {} on {}", tool_name, target);
    }

    std::process::exit(0);
}
