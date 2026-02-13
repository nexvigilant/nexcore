//! Directory Organizer - PostToolUse:Write Event
//!
//! Automatically organizes files in ~/.claude/ root directory by moving them
//! to appropriate subdirectories based on file type and naming conventions.
//!
//! Organization Rules:
//! - settings*.json → config/settings-archive/ (except settings.json)
//! - *.json (state/tracking) → state/
//! - *.json (cache-like) → cache/
//! - *.sh → config/
//! - *.log, *.jsonl → logs/ (created if needed)
//! - *.md → docs/ (created if needed)
//!
//! Preserves:
//! - settings.json (active config)
//! - statusline.sh (active status line)
//! - .credentials.json (auth)
//! - history.jsonl (session history)
//! - commands (empty file)

use nexcore_hooks::{exit_success_auto, exit_success_auto_with, read_input};
use std::fs;
use std::path::{Path, PathBuf};

/// Files that should NEVER be moved from root
const PROTECTED_FILES: &[&str] = &[
    "settings.json",
    "statusline.sh",
    ".credentials.json",
    "history.jsonl",
    "commands",
];

/// Patterns for state-related JSON files
const STATE_PATTERNS: &[&str] = &[
    "session_state",
    "task_state",
    "tracking_registry",
    "problem_registry",
    "mcp_efficacy",
];

/// Patterns for cache-related JSON files
const CACHE_PATTERNS: &[&str] = &["cache", "stats"];

/// Determine the destination directory for a file based on its name and extension
fn classify_file(filename: &str, claude_dir: &Path) -> Option<PathBuf> {
    // Skip protected files
    if PROTECTED_FILES.contains(&filename) {
        return None;
    }

    // Skip hidden files (except those we explicitly handle)
    if filename.starts_with('.') && filename != ".credentials.json" {
        return None;
    }

    // Settings variants → config/settings-archive/
    if filename.starts_with("settings")
        && filename.ends_with(".json")
        && filename != "settings.json"
    {
        return Some(claude_dir.join("config/settings-archive"));
    }

    // Backup files → config/settings-archive/
    if filename.contains(".backup") || filename.contains(".bak") {
        return Some(claude_dir.join("config/settings-archive"));
    }

    // JSON files - classify by content type
    if filename.ends_with(".json") {
        // State-related files
        for pattern in STATE_PATTERNS {
            if filename.contains(pattern) {
                return Some(claude_dir.join("state"));
            }
        }
        // Cache-related files
        for pattern in CACHE_PATTERNS {
            if filename.contains(pattern) {
                return Some(claude_dir.join("cache"));
            }
        }
        // Other JSON → state/ by default
        return Some(claude_dir.join("state"));
    }

    // JSONL files (logs) → keep history.jsonl in root, others to logs/
    if filename.ends_with(".jsonl") && filename != "history.jsonl" {
        return Some(claude_dir.join("logs"));
    }

    // Shell scripts → config/
    if filename.ends_with(".sh") && filename != "statusline.sh" {
        return Some(claude_dir.join("config"));
    }

    // Log files → logs/
    if filename.ends_with(".log") {
        return Some(claude_dir.join("logs"));
    }

    // Markdown files in root → docs/
    if filename.ends_with(".md") {
        return Some(claude_dir.join("docs"));
    }

    // YAML/TOML config files → config/
    if filename.ends_with(".yaml") || filename.ends_with(".yml") || filename.ends_with(".toml") {
        return Some(claude_dir.join("config"));
    }

    None
}

/// Move a file to its destination directory, creating the directory if needed
fn organize_file(file_path: &Path, dest_dir: &Path) -> Result<PathBuf, String> {
    // Create destination directory if it doesn't exist
    fs::create_dir_all(dest_dir)
        .map_err(|e| format!("Failed to create directory {:?}: {}", dest_dir, e))?;

    let filename = file_path
        .file_name()
        .ok_or_else(|| "No filename".to_string())?;
    let dest_path = dest_dir.join(filename);

    // Don't overwrite existing files - add suffix if needed
    let final_dest = if dest_path.exists() {
        let stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        if ext.is_empty() {
            dest_dir.join(format!("{}_{}", stem, timestamp))
        } else {
            dest_dir.join(format!("{}_{}.{}", stem, timestamp, ext))
        }
    } else {
        dest_path
    };

    fs::rename(file_path, &final_dest).map_err(|e| format!("Failed to move file: {}", e))?;

    Ok(final_dest)
}

/// Scan root directory and organize any misplaced files
fn organize_root_directory(claude_dir: &Path) -> Vec<String> {
    let mut actions = Vec::new();

    let entries = match fs::read_dir(claude_dir) {
        Ok(e) => e,
        Err(_) => return actions,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // Skip directories
        if path.is_dir() {
            continue;
        }

        let filename = match path.file_name().and_then(|s| s.to_str()) {
            Some(f) => f.to_string(),
            None => continue,
        };

        // Determine destination
        if let Some(dest_dir) = classify_file(&filename, claude_dir) {
            match organize_file(&path, &dest_dir) {
                Ok(new_path) => {
                    actions.push(format!(
                        "Moved `{}` → `{}`",
                        filename,
                        new_path
                            .strip_prefix(claude_dir)
                            .unwrap_or(&new_path)
                            .display()
                    ));
                }
                Err(e) => {
                    actions.push(format!("Failed to move `{}`: {}", filename, e));
                }
            }
        }
    }

    actions
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only process Write tool completions
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if tool_name != "Write" {
        exit_success_auto();
    }

    // Get the file that was written
    let written_file = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    // Only trigger if writing to ~/.claude/ root
    let claude_dir = dirs::home_dir()
        .map(|h| h.join(".claude"))
        .unwrap_or_else(|| PathBuf::from("/home/matthew/.claude"));

    // Check if the written file is in the root of ~/.claude/
    let written_path = Path::new(&written_file);
    let parent = written_path.parent();

    let is_claude_root = parent.map(|p| p == claude_dir).unwrap_or(false);

    if !is_claude_root {
        exit_success_auto();
    }

    // Skip if this is a protected file
    let filename = written_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    if PROTECTED_FILES.contains(&filename) {
        exit_success_auto();
    }

    // Organize the root directory
    let actions = organize_root_directory(&claude_dir);

    if actions.is_empty() {
        exit_success_auto();
    }

    // Report what was organized
    let message = format!(
        "📁 Auto-organized ~/.claude/ root:\n{}",
        actions
            .iter()
            .map(|a| format!("  • {}", a))
            .collect::<Vec<_>>()
            .join("\n")
    );

    exit_success_auto_with(&message);
}
