//! Documentation Filer - PostToolUse:Read Event
//!
//! Automatically detects documentation files when read and files them
//! to the brain system as artifacts for session persistence.
//!
//! Triggers on:
//! - README.md, CLAUDE.md files
//! - Files in docs/, reference/, knowledge/ directories
//! - Files with documentation patterns (SKILL.md, MANIFEST.md, etc.)
//!
//! Files to: ~/.claude/brain/sessions/{current}/docs/{filename}

use nexcore_hooks::{exit_success_auto, exit_success_auto_with, read_input};
use std::fs;
use std::path::{Path, PathBuf};

/// Documentation file patterns to capture
const DOC_PATTERNS: &[&str] = &[
    "README.md",
    "CLAUDE.md",
    "SKILL.md",
    "MANIFEST.md",
    "CHANGELOG.md",
    "CONTRIBUTING.md",
    "LICENSE.md",
    "ARCHITECTURE.md",
    "DESIGN.md",
    "SPEC.md",
    "PROTOCOL.md",
];

/// Directory patterns that indicate documentation
const DOC_DIRS: &[&str] = &[
    "/docs/",
    "/reference/",
    "/knowledge/",
    "/specifications/",
    "/architecture/",
    "/design/",
];

/// Check if a file path is a documentation file
fn is_documentation(path: &str) -> bool {
    let path_lower = path.to_lowercase();

    // Check filename patterns
    for pattern in DOC_PATTERNS {
        if path_lower.ends_with(&pattern.to_lowercase()) {
            return true;
        }
    }

    // Check directory patterns
    for dir in DOC_DIRS {
        if path_lower.contains(dir) && path_lower.ends_with(".md") {
            return true;
        }
    }

    // Check for constitutional documents (Primitive Codex)
    if path_lower.contains("primitive_codex") || path_lower.contains("primitive-codex") {
        return true;
    }

    // Check for skill documentation
    if path_lower.contains("/skills/") && path_lower.ends_with(".md") {
        return true;
    }

    false
}

/// Get current brain session directory
fn get_brain_session_dir() -> Option<PathBuf> {
    let brain_dir = dirs::home_dir()?.join(".claude/brain/sessions");

    // Find most recent session (directories are UUIDs)
    let entries: Vec<_> = fs::read_dir(&brain_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter(|e| {
            // Skip non-UUID directories like "watchtower"
            e.file_name()
                .to_str()
                .map(|s| s.len() == 36 && s.contains('-'))
                .unwrap_or(false)
        })
        .collect();

    // Get most recently modified session
    entries
        .into_iter()
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())
        .map(|e| e.path())
}

/// Generate a safe filename for the artifact
fn safe_artifact_name(original_path: &str) -> String {
    let path = Path::new(original_path);
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown.md");

    // Get parent directory name for context
    let parent = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("");

    if parent.is_empty() || parent == "." {
        filename.to_string()
    } else {
        format!("{}-{}", parent, filename)
    }
}

/// File documentation to brain - returns None on failure, Some(message) on success
fn file_to_brain(doc_path: &str, content: &str) -> Option<String> {
    let session_dir = get_brain_session_dir()?;

    let docs_dir = session_dir.join("docs");
    fs::create_dir_all(&docs_dir).ok()?;

    let artifact_name = safe_artifact_name(doc_path);
    let artifact_path = docs_dir.join(&artifact_name);

    // Only file if it doesn't already exist or is different
    let should_write = if artifact_path.exists() {
        let existing = fs::read_to_string(&artifact_path).unwrap_or_default();
        existing != content
    } else {
        true
    };

    if should_write {
        fs::write(&artifact_path, content).ok()?;
        Some(format!(
            "Filed: {} → brain/docs/{}",
            doc_path, artifact_name
        ))
    } else {
        Some(format!("Skipped (unchanged): {}", artifact_name))
    }
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only process Read tool completions
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if tool_name != "Read" {
        exit_success_auto();
    }

    // Get the file that was read
    let read_file = match input.get_file_path() {
        Some(p) => p,
        None => exit_success_auto(),
    };

    // Check if it's documentation
    if !is_documentation(&read_file) {
        exit_success_auto();
    }

    // Get the content from tool response
    let content = input
        .tool_response
        .as_ref()
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if content.is_empty() || content.len() < 100 {
        // Skip very short content (probably error messages)
        exit_success_auto();
    }

    // File to brain
    if let Some(msg) = file_to_brain(&read_file, content) {
        exit_success_auto_with(&format!("📚 {}", msg));
    } else {
        // Silently fail - don't block the user
        exit_success_auto();
    }
}
