//! SessionStart Error Extractor Hook
//!
//! Parses debug files for `[ERROR]` lines and writes a structured index.
//!
//! # Event
//! SessionStart
//!
//! # Output
//! `~/.claude/debug/error_index.jsonl` — one JSON object per error occurrence.
//!
//! # Exit Codes
//! - 0: Always (best-effort, never blocks)

use serde::Serialize;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Serialize)]
struct ErrorEntry {
    timestamp: String,
    session: String,
    category: String,
    message: String,
    count: u32,
}

fn main() {
    let debug_dir = debug_dir_path();
    if !debug_dir.is_dir() {
        std::process::exit(0);
    }

    let index_path = debug_dir.join("error_index.jsonl");
    let last_indexed = index_modified_time(&index_path);
    let entries = scan_files(&debug_dir, last_indexed);

    if entries.is_empty() {
        std::process::exit(0);
    }

    write_index(&index_path, &entries);
    eprintln!(
        "[error-extractor] Indexed {} errors from debug logs",
        entries.len()
    );

    std::process::exit(0);
}

fn debug_dir_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".claude").join("debug")
}

fn index_modified_time(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok().and_then(|m| m.modified().ok())
}

fn scan_files(dir: &Path, since: Option<SystemTime>) -> Vec<ErrorEntry> {
    let mut entries = Vec::new();

    let files: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "txt"))
        .filter(|p| is_newer_than(p, since))
        .collect();

    for path in files {
        let session_id = extract_session_id(&path);
        extract_errors(&path, &session_id, &mut entries);
    }

    entries
}

fn is_newer_than(path: &Path, since: Option<SystemTime>) -> bool {
    let Some(threshold) = since else {
        return true;
    };
    fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .is_some_and(|m| m > threshold)
}

fn extract_session_id(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    // For temporal names: 2026-02-01T03-08-47_9d00dd7a → 9d00dd7a
    if let Some(idx) = stem.rfind('_') {
        return stem[idx + 1..].to_string();
    }

    // For UUID names: first 8 chars
    stem[..stem.len().min(8)].to_string()
}

fn extract_errors(path: &Path, session_id: &str, entries: &mut Vec<ErrorEntry>) {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return,
    };

    let reader = BufReader::new(file);
    for line in reader.lines().map_while(Result::ok) {
        if !line.contains("[ERROR]") {
            continue;
        }
        let timestamp = line.get(..23).unwrap_or("").to_string();
        let message = extract_message(&line);
        let category = classify_error(&message);

        entries.push(ErrorEntry {
            timestamp,
            session: session_id.to_string(),
            category,
            message,
            count: 1,
        });
    }
}

fn extract_message(line: &str) -> String {
    // Format: "2026-02-01T03:09:58.637Z [ERROR] Error: actual message"
    let after_error = line
        .find("[ERROR]")
        .map(|i| &line[i + 8..])
        .unwrap_or(line);

    // Strip leading "Error: " prefix if present
    let msg = after_error.trim();
    let msg = msg.strip_prefix("Error: ").unwrap_or(msg);

    // Truncate to 200 chars for index compactness
    if msg.len() > 200 {
        format!("{}...", &msg[..197])
    } else {
        msg.to_string()
    }
}

fn classify_error(message: &str) -> String {
    let m = message.to_lowercase();

    if m.contains("agent") && m.contains("fail") {
        return "agent_failure".to_string();
    }
    if m.contains("connection closed") || m.contains("-32000") || m.contains("mcp") {
        return "mcp_disconnect".to_string();
    }
    if m.contains("enoent") || m.contains("not found") {
        return "executable_not_found".to_string();
    }
    if m.contains("eexist") || m.contains("lock") || m.contains("pid") {
        return "lock_contention".to_string();
    }
    if m.contains("eacces") || m.contains("permission") {
        return "permission_denied".to_string();
    }
    if m.contains("at ") && (m.contains("native:") || m.contains("bunfs")) {
        return "stack_trace".to_string();
    }
    if m.contains("timeout") || m.contains("timed out") || m.contains("econnaborted") {
        return "timeout".to_string();
    }
    if m.contains("parse") || m.contains("frontmatter") || m.contains("schema") {
        return "parse_failure".to_string();
    }

    "uncategorized".to_string()
}

fn write_index(path: &Path, entries: &[ErrorEntry]) {
    // Append mode: preserve existing entries
    let mut file = match fs::OpenOptions::new().create(true).append(true).open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[error-extractor] Cannot write index: {e}");
            return;
        }
    };

    for entry in entries {
        let Ok(json) = serde_json::to_string(entry) else {
            continue;
        };
        let _ = writeln!(file, "{json}");
    }
}
