#![forbid(unsafe_code)]

use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use flate2::Compression;
use flate2::write::GzEncoder;
use nexcore_fs::walk::WalkDir;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ErrorCode, Implementation, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::fs;
use tokio::io::AsyncWriteExt;

const CLAUDE_ROOT: &str = "/home/matthew/.claude";
const BACKUP_DIR: &str = "/home/matthew/.claude/backup/sessions";

#[derive(Debug, nexcore_error::Error)]
pub enum FsError {
    #[error("path escapes claude root: {0}")]
    PathEscape(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    Io(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PathParam {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WriteParam {
    pub path: String,
    pub content: String,
    pub create_dirs: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchParam {
    pub query: String,
    pub root: Option<String>,
    pub max_results: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListParam {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TailParam {
    pub path: String,
    pub lines: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DiffParam {
    pub path_a: String,
    pub path_b: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StatParam {
    pub path: String,
}

#[derive(Clone)]
pub struct ClaudeFsMcpServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl ClaudeFsMcpServer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "List files under a .claude path (non-recursive).")]
    async fn claude_fs_list(
        &self,
        Parameters(params): Parameters<ListParam>,
    ) -> Result<CallToolResult, McpError> {
        let path = resolve_path(&params.path)?;
        let mut entries = Vec::new();
        let mut read_dir = fs::read_dir(&path).await.map_err(mcp_io)?;
        while let Some(entry) = read_dir.next_entry().await.map_err(mcp_io)? {
            let file_type = entry.file_type().await.map_err(mcp_io)?;
            entries.push(format!(
                "{}{}",
                entry.path().display(),
                if file_type.is_dir() { "/" } else { "" }
            ));
        }
        entries.sort();
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            entries.join("\n"),
        )]))
    }

    #[tool(description = "Read a file under .claude.")]
    async fn claude_fs_read(
        &self,
        Parameters(params): Parameters<PathParam>,
    ) -> Result<CallToolResult, McpError> {
        let path = resolve_path(&params.path)?;
        let content = fs::read_to_string(&path).await.map_err(mcp_io)?;
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            content,
        )]))
    }

    #[tool(description = "Write a file under .claude.")]
    async fn claude_fs_write(
        &self,
        Parameters(params): Parameters<WriteParam>,
    ) -> Result<CallToolResult, McpError> {
        let path = resolve_path(&params.path)?;
        if params.create_dirs.unwrap_or(true) {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await.map_err(mcp_io)?;
            }
        }
        let mut file = fs::File::create(&path).await.map_err(mcp_io)?;
        file.write_all(params.content.as_bytes())
            .await
            .map_err(mcp_io)?;
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            "ok",
        )]))
    }

    #[tool(description = "Delete a file under .claude.")]
    async fn claude_fs_delete(
        &self,
        Parameters(params): Parameters<PathParam>,
    ) -> Result<CallToolResult, McpError> {
        let path = resolve_path(&params.path)?;
        if !path.exists() {
            return Err(McpError::new(ErrorCode(404), "not found", None));
        }
        let meta = fs::metadata(&path).await.map_err(mcp_io)?;
        if meta.is_dir() {
            fs::remove_dir_all(&path).await.map_err(mcp_io)?;
        } else {
            fs::remove_file(&path).await.map_err(mcp_io)?;
        }
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            "ok",
        )]))
    }

    #[tool(description = "Search for a substring under .claude.")]
    async fn claude_fs_search(
        &self,
        Parameters(params): Parameters<SearchParam>,
    ) -> Result<CallToolResult, McpError> {
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
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            matches.join("\n"),
        )]))
    }

    #[tool(description = "Tail last N lines of a file under .claude.")]
    async fn claude_fs_tail(
        &self,
        Parameters(params): Parameters<TailParam>,
    ) -> Result<CallToolResult, McpError> {
        let path = resolve_path(&params.path)?;
        let content = fs::read_to_string(&path).await.map_err(mcp_io)?;
        let count = params.lines.unwrap_or(100);
        let lines: Vec<&str> = content.lines().collect();
        let start = lines.len().saturating_sub(count);
        let slice = lines[start..].join("\n");
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            slice,
        )]))
    }

    #[tool(description = "Diff two files under .claude (simple line diff).")]
    async fn claude_fs_diff(
        &self,
        Parameters(params): Parameters<DiffParam>,
    ) -> Result<CallToolResult, McpError> {
        let path_a = resolve_path(&params.path_a)?;
        let path_b = resolve_path(&params.path_b)?;
        let a = fs::read_to_string(&path_a).await.map_err(mcp_io)?;
        let b = fs::read_to_string(&path_b).await.map_err(mcp_io)?;
        if a == b {
            return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                "no diff",
            )]));
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
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            out.join("\n"),
        )]))
    }

    #[tool(description = "Stat a file under .claude.")]
    async fn claude_fs_stat(
        &self,
        Parameters(params): Parameters<StatParam>,
    ) -> Result<CallToolResult, McpError> {
        let path = resolve_path(&params.path)?;
        let meta = fs::metadata(&path).await.map_err(mcp_io)?;
        let modified = meta.modified().ok();
        let payload = json!({
            "path": path.display().to_string(),
            "is_dir": meta.is_dir(),
            "size": meta.len(),
            "modified": modified.and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()),
        });
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            payload.to_string(),
        )]))
    }

    #[tool(description = "Create a backup tar.gz of the full .claude directory.")]
    async fn claude_backup_now(&self) -> Result<CallToolResult, McpError> {
        match create_backup() {
            Ok(path) => Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                path,
            )])),
            Err(err) => Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                err.to_string(),
            )])),
        }
    }
}

impl Default for ClaudeFsMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerHandler for ClaudeFsMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                r#"Claude FS MCP Server

Full-access tools for .claude filesystem operations. A backup is created on server start.
"#
                .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "claude-fs-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: Some("Claude FS MCP Server".into()),
                icons: None,
                website_url: None,
            },
            ..Default::default()
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            let tcc = ToolCallContext::new(self, request, context);
            let result = self.tool_router.call(tcc).await?;
            Ok(result)
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListToolsResult {
            tools: self.tool_router.list_all(),
            meta: None,
            next_cursor: None,
        }))
    }
}

pub fn create_backup() -> Result<String, FsError> {
    let root = Path::new(CLAUDE_ROOT);
    if !root.exists() {
        return Err(FsError::NotFound(CLAUDE_ROOT.to_string()));
    }

    let backup_dir = Path::new(BACKUP_DIR);
    std::fs::create_dir_all(backup_dir).map_err(|err| FsError::Io(err.to_string()))?;

    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let backup_path = backup_dir.join(format!("claude-backup-{stamp}.tar.gz"));
    let tar_gz = File::create(&backup_path).map_err(|err| FsError::Io(err.to_string()))?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);

    // Best-effort archive: skip files that disappear during traversal.
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
            Err(e) => return Err(FsError::Io(e.to_string())),
        };
        if meta.is_dir() {
            if let Err(e) = tar.append_dir(name, path) {
                if e.kind() == io::ErrorKind::NotFound {
                    continue;
                }
                return Err(FsError::Io(e.to_string()));
            }
        } else if meta.is_file() {
            let mut file = match File::open(path) {
                Ok(f) => f,
                Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
                Err(e) => return Err(FsError::Io(e.to_string())),
            };
            if let Err(e) = tar.append_file(name, &mut file) {
                if e.kind() == io::ErrorKind::NotFound {
                    continue;
                }
                return Err(FsError::Io(e.to_string()));
            }
        }
    }
    let enc = tar
        .into_inner()
        .map_err(|err| FsError::Io(err.to_string()))?;
    enc.finish().map_err(|err| FsError::Io(err.to_string()))?;

    rotate_backups(backup_dir).map_err(|err| FsError::Io(err.to_string()))?;
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
    fn serialize_params() {
        let params = WriteParam {
            path: "brain/test.txt".to_string(),
            content: "hello".to_string(),
            create_dirs: Some(true),
        };
        let json = serde_json::to_string(&params).expect("serialize");
        assert!(json.contains("\"path\""));
    }
}
