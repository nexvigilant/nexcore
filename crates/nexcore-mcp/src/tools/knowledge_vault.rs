//! Knowledge Vault MCP tools — Obsidian-compatible markdown vault operations.
//!
//! Replaces the external `obsidian-mcp` npm package with native Rust file I/O.
//! No orphan processes. No npm dependency. Single nexcore process.
//!
//! Vault root is configured via `NEXVIGILANT_VAULT` env var,
//! defaulting to `~/Vaults/nexvigilant/`.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::params::knowledge_vault::{
    KnowledgeVaultListParams, KnowledgeVaultMoveParams, KnowledgeVaultReadParams,
    KnowledgeVaultSearchParams, KnowledgeVaultTagsParams, KnowledgeVaultWriteParams,
};

// ── Helpers ──────────────────────────────────────────────────────────────

fn vault_root() -> PathBuf {
    if let Ok(path) = std::env::var("NEXVIGILANT_VAULT") {
        PathBuf::from(path)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join("Vaults/nexvigilant")
    } else {
        PathBuf::from("/home/matthew/Vaults/nexvigilant")
    }
}

/// Resolve a relative vault path, ensuring it stays within the vault root.
/// Appends .md if the path has no extension.
fn resolve_path(relative: &str) -> Result<PathBuf, String> {
    let root = vault_root();
    let mut rel = PathBuf::from(relative.trim_start_matches('/'));

    // Append .md if no extension
    if rel.extension().is_none() {
        rel.set_extension("md");
    }

    let full = root.join(&rel);

    // Canonicalize parent to prevent traversal, but allow non-existent files
    let parent = full.parent().unwrap_or(&root);
    if parent.exists() {
        let canon = parent
            .canonicalize()
            .map_err(|e| format!("path resolution failed: {e}"))?;
        let root_canon = root
            .canonicalize()
            .map_err(|e| format!("vault root resolution failed: {e}"))?;
        if !canon.starts_with(&root_canon) {
            return Err("path traversal outside vault root is not allowed".to_string());
        }
    }

    Ok(full)
}

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn ok_text(text: String) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}

fn err_result(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

/// Collect all .md files under a directory.
fn walk_md_files(dir: &Path, recursive: bool) -> Vec<PathBuf> {
    let mut results = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return results,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip hidden directories (.obsidian, .trash, etc.)
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') || name.starts_with('_') {
                    continue;
                }
            }
            if recursive {
                results.extend(walk_md_files(&path, true));
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            results.push(path);
        }
    }
    results
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Read a note from the knowledge vault.
pub fn knowledge_vault_read(p: KnowledgeVaultReadParams) -> Result<CallToolResult, McpError> {
    let path = match resolve_path(&p.path) {
        Ok(p) => p,
        Err(e) => return err_result(&e),
    };

    if !path.exists() {
        return err_result(&format!(
            "Note not found: {}",
            path.strip_prefix(vault_root()).unwrap_or(&path).display()
        ));
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let relative = path
                .strip_prefix(vault_root())
                .unwrap_or(&path)
                .display()
                .to_string();
            // Return content directly for readability, with path header
            ok_text(format!("# {relative}\n\n{content}"))
        }
        Err(e) => err_result(&format!("Failed to read note: {e}")),
    }
}

/// Search vault content by query string (case-insensitive substring match).
pub fn knowledge_vault_search(p: KnowledgeVaultSearchParams) -> Result<CallToolResult, McpError> {
    let root = vault_root();
    let search_dir = if let Some(ref scope) = p.scope {
        root.join(scope)
    } else {
        root.clone()
    };

    if !search_dir.exists() {
        return err_result(&format!(
            "Search scope not found: {}",
            search_dir
                .strip_prefix(&root)
                .unwrap_or(&search_dir)
                .display()
        ));
    }

    let limit = p.limit.unwrap_or(20);
    let query_lower = p.query.to_lowercase();
    let files = walk_md_files(&search_dir, true);

    let mut results: Vec<serde_json::Value> = Vec::new();

    for file in &files {
        if results.len() >= limit {
            break;
        }

        let relative = file
            .strip_prefix(&root)
            .unwrap_or(file)
            .display()
            .to_string();

        // Check filename match
        let filename_match = relative.to_lowercase().contains(&query_lower);

        // Check content match
        let content = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let content_lower = content.to_lowercase();

        if filename_match || content_lower.contains(&query_lower) {
            // Extract matching line context
            let mut snippets: Vec<String> = Vec::new();
            for (i, line) in content.lines().enumerate() {
                if line.to_lowercase().contains(&query_lower) {
                    snippets.push(format!("L{}: {}", i + 1, line.trim()));
                    if snippets.len() >= 3 {
                        break;
                    }
                }
            }

            results.push(json!({
                "path": relative,
                "filename_match": filename_match,
                "content_match": content_lower.contains(&query_lower),
                "snippets": snippets,
                "size_bytes": content.len(),
            }));
        }
    }

    ok_json(json!({
        "query": p.query,
        "scope": p.scope.unwrap_or_else(|| "/".to_string()),
        "total_files_scanned": files.len(),
        "results_count": results.len(),
        "results": results,
    }))
}

/// List vault contents (files and directories).
pub fn knowledge_vault_list(p: KnowledgeVaultListParams) -> Result<CallToolResult, McpError> {
    let root = vault_root();
    let target = if let Some(ref path) = p.path {
        root.join(path)
    } else {
        root.clone()
    };

    if !target.exists() {
        return err_result(&format!(
            "Path not found: {}",
            target.strip_prefix(&root).unwrap_or(&target).display()
        ));
    }

    let recursive = p.recursive.unwrap_or(false);

    if recursive {
        let files = walk_md_files(&target, true);
        let paths: Vec<String> = files
            .iter()
            .filter_map(|f| f.strip_prefix(&root).ok())
            .map(|f| f.display().to_string())
            .collect();

        ok_json(json!({
            "path": p.path.unwrap_or_else(|| "/".to_string()),
            "total_notes": paths.len(),
            "notes": paths,
        }))
    } else {
        let entries = match std::fs::read_dir(&target) {
            Ok(e) => e,
            Err(e) => return err_result(&format!("Failed to list directory: {e}")),
        };

        let mut dirs: Vec<String> = Vec::new();
        let mut files: Vec<String> = Vec::new();

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name.starts_with('_') {
                continue;
            }
            if entry.path().is_dir() {
                dirs.push(format!("{name}/"));
            } else if name.ends_with(".md") {
                files.push(name);
            }
        }

        dirs.sort();
        files.sort();

        ok_json(json!({
            "path": p.path.unwrap_or_else(|| "/".to_string()),
            "directories": dirs,
            "notes": files,
            "total_dirs": dirs.len(),
            "total_notes": files.len(),
        }))
    }
}

/// Write or create a note in the knowledge vault.
pub fn knowledge_vault_write(p: KnowledgeVaultWriteParams) -> Result<CallToolResult, McpError> {
    let path = match resolve_path(&p.path) {
        Ok(p) => p,
        Err(e) => return err_result(&e),
    };

    let existed = path.exists();

    // Create parent directories
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return err_result(&format!("Failed to create directories: {e}"));
        }
    }

    match std::fs::write(&path, &p.content) {
        Ok(()) => {
            let relative = path
                .strip_prefix(vault_root())
                .unwrap_or(&path)
                .display()
                .to_string();
            ok_json(json!({
                "path": relative,
                "action": if existed { "updated" } else { "created" },
                "size_bytes": p.content.len(),
            }))
        }
        Err(e) => err_result(&format!("Failed to write note: {e}")),
    }
}

/// Move or rename a note in the knowledge vault.
pub fn knowledge_vault_move(p: KnowledgeVaultMoveParams) -> Result<CallToolResult, McpError> {
    let from = match resolve_path(&p.from) {
        Ok(p) => p,
        Err(e) => return err_result(&format!("Source: {e}")),
    };
    let to = match resolve_path(&p.to) {
        Ok(p) => p,
        Err(e) => return err_result(&format!("Destination: {e}")),
    };

    if !from.exists() {
        return err_result(&format!(
            "Source not found: {}",
            from.strip_prefix(vault_root()).unwrap_or(&from).display()
        ));
    }
    if to.exists() {
        return err_result(&format!(
            "Destination already exists: {}",
            to.strip_prefix(vault_root()).unwrap_or(&to).display()
        ));
    }

    // Create parent directories for destination
    if let Some(parent) = to.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return err_result(&format!("Failed to create directories: {e}"));
        }
    }

    match std::fs::rename(&from, &to) {
        Ok(()) => {
            let root = vault_root();
            ok_json(json!({
                "from": from.strip_prefix(&root).unwrap_or(&from).display().to_string(),
                "to": to.strip_prefix(&root).unwrap_or(&to).display().to_string(),
                "action": "moved",
            }))
        }
        Err(e) => err_result(&format!("Failed to move note: {e}")),
    }
}

/// List all tags used across the vault (Obsidian #tag format).
pub fn knowledge_vault_tags(p: KnowledgeVaultTagsParams) -> Result<CallToolResult, McpError> {
    let root = vault_root();
    let search_dir = if let Some(ref scope) = p.scope {
        root.join(scope)
    } else {
        root.clone()
    };

    if !search_dir.exists() {
        return err_result(&format!(
            "Scope not found: {}",
            search_dir
                .strip_prefix(&root)
                .unwrap_or(&search_dir)
                .display()
        ));
    }

    let files = walk_md_files(&search_dir, true);
    let mut tag_counts: BTreeMap<String, usize> = BTreeMap::new();

    // Match #tag patterns (not inside code blocks)
    let tag_re = regex::Regex::new(r"(?:^|\s)#([a-zA-Z][a-zA-Z0-9_/-]*)").unwrap_or_else(|_| {
        // Fallback: match nothing
        regex::Regex::new(r"^\b$").expect("fallback regex")
    });

    for file in &files {
        let content = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let mut in_code_block = false;
        for line in content.lines() {
            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }
            for cap in tag_re.captures_iter(line) {
                if let Some(tag) = cap.get(1) {
                    *tag_counts.entry(tag.as_str().to_string()).or_insert(0) += 1;
                }
            }
        }
    }

    let tags: Vec<serde_json::Value> = tag_counts
        .iter()
        .map(|(tag, count)| json!({"tag": format!("#{tag}"), "count": count}))
        .collect();

    ok_json(json!({
        "scope": p.scope.unwrap_or_else(|| "/".to_string()),
        "total_tags": tags.len(),
        "files_scanned": files.len(),
        "tags": tags,
    }))
}
