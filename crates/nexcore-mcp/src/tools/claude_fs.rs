//! Claude filesystem tools — CRUD operations on `~/.claude/` directory.
//!
//! Consolidated from `claude-fs-mcp` satellite server.
//! 9 tools: list, read, write, delete, search, tail, diff, stat, backup_now.
//!
//! Tier: T3 (π Persistence + σ Sequence + ∂ Boundary + μ Mapping)

use std::fs::File;
use std::io::{self, Read as IoRead};
use std::path::{Path, PathBuf};

use crate::params::{
    ClaudeFsDeleteParams, ClaudeFsDiffParams, ClaudeFsListParams, ClaudeFsReadParams,
    ClaudeFsSearchParams, ClaudeFsStatParams, ClaudeFsTailParams, ClaudeFsWriteParams,
};
use nexcore_fs::walk::WalkDir;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content, ErrorCode};
use serde_json::json;

const CLAUDE_ROOT: &str = "/home/matthew/.claude";
const BACKUP_DIR: &str = "/home/matthew/.claude/backup/sessions";

/// List files under a `.claude` path (non-recursive).
pub async fn claude_fs_list(params: ClaudeFsListParams) -> Result<CallToolResult, McpError> {
    let path = resolve_path(&params.path)?;
    let mut entries = Vec::new();
    let mut read_dir = tokio::fs::read_dir(&path).await.map_err(mcp_io)?;
    while let Some(entry) = read_dir.next_entry().await.map_err(mcp_io)? {
        let file_type = entry.file_type().await.map_err(mcp_io)?;
        entries.push(format!(
            "{}{}",
            entry.path().display(),
            if file_type.is_dir() { "/" } else { "" }
        ));
    }
    entries.sort();
    Ok(CallToolResult::success(vec![Content::text(
        entries.join("\n"),
    )]))
}

/// Read a file under `.claude`.
pub async fn claude_fs_read(params: ClaudeFsReadParams) -> Result<CallToolResult, McpError> {
    let path = resolve_path(&params.path)?;
    let content = tokio::fs::read_to_string(&path).await.map_err(mcp_io)?;
    Ok(CallToolResult::success(vec![Content::text(content)]))
}

/// Write a file under `.claude`.
pub async fn claude_fs_write(params: ClaudeFsWriteParams) -> Result<CallToolResult, McpError> {
    let path = resolve_path(&params.path)?;
    if params.create_dirs.unwrap_or(true) {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(mcp_io)?;
        }
    }
    tokio::fs::write(&path, params.content.as_bytes())
        .await
        .map_err(mcp_io)?;
    Ok(CallToolResult::success(vec![Content::text("ok")]))
}

/// Delete a file or directory under `.claude`.
pub async fn claude_fs_delete(params: ClaudeFsDeleteParams) -> Result<CallToolResult, McpError> {
    let path = resolve_path(&params.path)?;
    if !path.exists() {
        return Err(McpError::new(ErrorCode(404), "not found", None));
    }
    let meta = tokio::fs::metadata(&path).await.map_err(mcp_io)?;
    if meta.is_dir() {
        tokio::fs::remove_dir_all(&path).await.map_err(mcp_io)?;
    } else {
        tokio::fs::remove_file(&path).await.map_err(mcp_io)?;
    }
    Ok(CallToolResult::success(vec![Content::text("ok")]))
}

/// Search for a substring in files under `.claude`.
pub fn claude_fs_search(params: ClaudeFsSearchParams) -> Result<CallToolResult, McpError> {
    let root = params.root.as_deref().unwrap_or(".");
    let root = resolve_path(root)?;
    let max_results = params.max_results.unwrap_or(200);
    let mut matches = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if let Ok(mut file) = File::open(path) {
            let mut buf = String::new();
            if file.read_to_string(&mut buf).is_ok() && buf.contains(&params.query) {
                matches.push(path.display().to_string());
                if matches.len() >= max_results {
                    break;
                }
            }
        }
    }
    Ok(CallToolResult::success(vec![Content::text(
        matches.join("\n"),
    )]))
}

/// Tail last N lines of a file under `.claude`.
pub async fn claude_fs_tail(params: ClaudeFsTailParams) -> Result<CallToolResult, McpError> {
    let path = resolve_path(&params.path)?;
    let content = tokio::fs::read_to_string(&path).await.map_err(mcp_io)?;
    let count = params.lines.unwrap_or(100);
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(count);
    let slice = lines[start..].join("\n");
    Ok(CallToolResult::success(vec![Content::text(slice)]))
}

/// Diff two files under `.claude` (simple line diff).
pub async fn claude_fs_diff(params: ClaudeFsDiffParams) -> Result<CallToolResult, McpError> {
    let path_a = resolve_path(&params.path_a)?;
    let path_b = resolve_path(&params.path_b)?;
    let a = tokio::fs::read_to_string(&path_a).await.map_err(mcp_io)?;
    let b = tokio::fs::read_to_string(&path_b).await.map_err(mcp_io)?;
    if a == b {
        return Ok(CallToolResult::success(vec![Content::text("no diff")]));
    }
    let a_lines: Vec<&str> = a.lines().collect();
    let b_lines: Vec<&str> = b.lines().collect();
    let max = a_lines.len().max(b_lines.len());
    let mut out = Vec::new();
    for idx in 0..max {
        let left = a_lines.get(idx).copied().unwrap_or("");
        let right = b_lines.get(idx).copied().unwrap_or("");
        if left != right {
            out.push(format!("- {:4}: {}", idx + 1, left));
            out.push(format!("+ {:4}: {}", idx + 1, right));
        }
    }
    Ok(CallToolResult::success(vec![Content::text(out.join("\n"))]))
}

/// Stat a file under `.claude`.
pub async fn claude_fs_stat(params: ClaudeFsStatParams) -> Result<CallToolResult, McpError> {
    let path = resolve_path(&params.path)?;
    let meta = tokio::fs::metadata(&path).await.map_err(mcp_io)?;
    let modified = meta.modified().ok();
    let payload = json!({
        "path": path.display().to_string(),
        "is_dir": meta.is_dir(),
        "size": meta.len(),
        "modified": modified.and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()),
    });
    Ok(CallToolResult::success(vec![Content::text(
        payload.to_string(),
    )]))
}

/// Create a backup tar.gz of the full `.claude` directory.
pub fn claude_fs_backup_now() -> Result<CallToolResult, McpError> {
    match create_backup() {
        Ok(path) => Ok(CallToolResult::success(vec![Content::text(path)])),
        Err(err) => Ok(CallToolResult::success(vec![Content::text(
            err.to_string(),
        )])),
    }
}

// ---------------------------------------------------------------------------
// Backup helpers
// ---------------------------------------------------------------------------

fn create_backup() -> Result<String, io::Error> {
    use flate2::Compression;
    use flate2::write::GzEncoder;

    let root = Path::new(CLAUDE_ROOT);
    if !root.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "claude root not found",
        ));
    }

    let backup_dir = Path::new(BACKUP_DIR);
    std::fs::create_dir_all(backup_dir)?;

    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let backup_path = backup_dir.join(format!("claude-backup-{stamp}.tar.gz"));
    let tar_gz = File::create(&backup_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);

    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        let rel = match path.strip_prefix(root) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let name = Path::new(".claude").join(rel);
        let meta = match std::fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e),
        };
        if meta.is_dir() {
            if let Err(e) = tar.append_dir(name, path) {
                if e.kind() == io::ErrorKind::NotFound {
                    continue;
                }
                return Err(e);
            }
        } else if meta.is_file() {
            let mut file = match File::open(path) {
                Ok(f) => f,
                Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e),
            };
            if let Err(e) = tar.append_file(name, &mut file) {
                if e.kind() == io::ErrorKind::NotFound {
                    continue;
                }
                return Err(e);
            }
        }
    }
    let enc = tar.into_inner()?;
    enc.finish()?;

    rotate_backups(backup_dir)?;
    Ok(backup_path.display().to_string())
}

fn rotate_backups(backup_dir: &Path) -> io::Result<()> {
    let keep: usize = std::env::var("CLAUDE_BACKUP_KEEP")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(backup_dir)?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("gz"))
        .collect();
    entries.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
    if entries.len() > keep {
        let excess = entries.len() - keep;
        for entry in entries.into_iter().take(excess) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Path resolution
// ---------------------------------------------------------------------------

fn resolve_path(rel: &str) -> Result<PathBuf, McpError> {
    let root = Path::new(CLAUDE_ROOT);
    let rel = rel.trim_start_matches('/');
    let rel_path = Path::new(rel);
    for component in rel_path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(McpError::new(
                ErrorCode(403),
                "path escapes claude root",
                None,
            ));
        }
    }
    Ok(root.join(rel_path))
}

fn mcp_io(err: io::Error) -> McpError {
    McpError::new(ErrorCode(500), err.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_path_inside_root() {
        let path = resolve_path("skills").expect("valid path");
        assert!(path.to_string_lossy().contains(".claude"));
    }

    #[test]
    fn rejects_parent_traversal() {
        let result = resolve_path("../etc/passwd");
        assert!(result.is_err());
    }
}
