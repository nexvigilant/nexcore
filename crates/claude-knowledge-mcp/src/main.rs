use anyhow::Result;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ErrorCode, Implementation, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::stdio;
use rmcp::{ErrorData as McpError, ServerHandler, ServiceExt, tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use tokio::fs;

const CLAUDE_ROOT: &str = "/home/matthew/.claude";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SqlParams {
    /// SQL SELECT query to run against anatomy.db
    pub sql: String,
    /// Maximum rows to return (default 50, max 200)
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchParams {
    /// Text to search for in knowledge/brain files
    pub query: String,
    /// Optional subdirectory to search (e.g. "knowledge", "brain", "memory")
    pub sub_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SkillListParams {
    /// Optional tag to filter skills
    pub tag: Option<String>,
}

#[derive(Clone)]
pub struct ClaudeKnowledgeMcpServer {
    tool_router: ToolRouter<Self>,
    root: PathBuf,
}

#[tool_router]
impl ClaudeKnowledgeMcpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            root: PathBuf::from(CLAUDE_ROOT),
        }
    }

    #[tool(description = "Query Claude's anatomical database (anatomy.db). Read-only SELECT queries only.")]
    async fn claude_anatomy_query(
        &self,
        Parameters(params): Parameters<SqlParams>,
    ) -> Result<CallToolResult, McpError> {
        let db_path = self.root.join("anatomy/anatomy.db");
        if !db_path.exists() {
            return Err(McpError::new(ErrorCode(404), "anatomy.db not found", None));
        }

        let sql = params.sql.trim();
        if !sql.to_uppercase().starts_with("SELECT") {
            return Err(McpError::new(ErrorCode(403), "Only SELECT queries allowed", None));
        }

        let limit = params.limit.unwrap_or(50).min(200);
        
        let conn = rusqlite::Connection::open(&db_path).map_err(|e| McpError::new(ErrorCode(500), e.to_string(), None))?;
        
        let mut stmt = conn.prepare(sql).map_err(|e| McpError::new(ErrorCode(400), e.to_string(), None))?;
        let col_count = stmt.column_count();
        let col_names: Vec<String> = (0..col_count).map(|i| stmt.column_name(i).unwrap_or("?").to_string()).collect();

        let rows: Vec<serde_json::Value> = stmt.query_map([], |row| {
            let mut obj = serde_json::Map::new();
            for (i, name) in col_names.iter().enumerate() {
                let val: rusqlite::types::Value = row.get(i)?;
                let json_val = match val {
                    rusqlite::types::Value::Null => serde_json::Value::Null,
                    rusqlite::types::Value::Integer(n) => json!(n),
                    rusqlite::types::Value::Real(f) => json!(f),
                    rusqlite::types::Value::Text(s) => json!(s),
                    rusqlite::types::Value::Blob(b) => json!(format!("<blob:{} bytes>", b.len())),
                };
                obj.insert(name.clone(), json_val);
            }
            Ok(serde_json::Value::Object(obj))
        }).map_err(|e| McpError::new(ErrorCode(500), e.to_string(), None))?
        .take(limit)
        .filter_map(|r| r.ok())
        .collect();

        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&rows).unwrap_or_default(),
        )]))
    }

    #[tool(description = "List registered skills in Claude's skill system.")]
    async fn claude_skills_list(
        &self,
        Parameters(params): Parameters<SkillListParams>,
    ) -> Result<CallToolResult, McpError> {
        let skills_path = self.root.join("skills");
        if !skills_path.exists() {
            return Err(McpError::new(ErrorCode(404), "skills directory not found", None));
        }

        let mut entries = fs::read_dir(skills_path).await.map_err(|e| McpError::new(ErrorCode(500), e.to_string(), None))?;
        let mut skills = Vec::new();

        while let Some(entry) = entries.next_entry().await.map_err(|e| McpError::new(ErrorCode(500), e.to_string(), None))? {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || name.starts_with('_') {
                    continue;
                }
                
                let skill_md = path.join("SKILL.md");
                let intent = if skill_md.exists() {
                    let content = fs::read_to_string(&skill_md).await.unwrap_or_default();
                    // Try to extract intent from frontmatter or first lines
                    content.lines()
                        .find(|l| l.contains("intent:") || l.contains("Intent:"))
                        .map(|l| l.split(':').nth(1).unwrap_or("").trim().to_string())
                        .unwrap_or_else(|| "No intent specified".to_string())
                } else {
                    "Missing SKILL.md".to_string()
                };

                if let Some(tag) = &params.tag {
                    if !intent.to_lowercase().contains(&tag.to_lowercase()) && !name.to_lowercase().contains(&tag.to_lowercase()) {
                        continue;
                    }
                }

                skills.push(json!({
                    "name": name,
                    "intent": intent,
                    "path": path.to_string_lossy()
                }));
            }
        }

        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&skills).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Search Claude's knowledge management system.")]
    async fn claude_knowledge_search(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let sub_dir = params.sub_dir.as_deref().unwrap_or("knowledge");
        let search_path = self.root.join(sub_dir);
        if !search_path.exists() {
            return Err(McpError::new(ErrorCode(404), format!("Directory not found: {}", sub_dir), None));
        }

        let mut results = Vec::new();
        let mut it = walkdir::WalkDir::new(search_path).into_iter().filter_map(|e| e.ok());
        
        while let Some(entry) = it.next() {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Ok(content) = std::fs::read_to_string(path) {
                    if content.to_lowercase().contains(&params.query.to_lowercase()) {
                        results.push(json!({
                            "path": path.to_string_lossy(),
                            "preview": content.chars().take(200).collect::<String>()
                        }));
                        if results.len() >= 20 {
                            break;
                        }
                    }
                }
            }
        }

        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&results).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Peek into recent brain activity.")]
    async fn claude_brain_peek(&self) -> Result<CallToolResult, McpError> {
        let brain_path = self.root.join("brain");
        if !brain_path.exists() {
            return Err(McpError::new(ErrorCode(404), "brain directory not found", None));
        }

        let mut entries = Vec::new();
        let mut read_dir = fs::read_dir(brain_path).await.map_err(|e| McpError::new(ErrorCode(500), e.to_string(), None))?;
        
        while let Some(entry) = read_dir.next_entry().await.map_err(|e| McpError::new(ErrorCode(500), e.to_string(), None))? {
            let meta = entry.metadata().await.map_err(|e| McpError::new(ErrorCode(500), e.to_string(), None))?;
            entries.push(json!({
                "name": entry.file_name().to_string_lossy(),
                "modified": meta.modified().ok().and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()),
                "size": meta.len()
            }));
        }

        entries.sort_by_key(|e| e["modified"].as_u64().unwrap_or(0));
        entries.reverse();

        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&entries.iter().take(20).collect::<Vec<_>>()).unwrap_or_default(),
        )]))
    }
}

impl ServerHandler for ClaudeKnowledgeMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Claude Knowledge MCP Server - Active tapping into knowledge and anatomy.".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "claude-knowledge-mcp".into(),
                version: "1.0.0".into(),
                title: Some("Claude Knowledge MCP Server".into()),
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
            self.tool_router.call(tcc).await
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

#[tokio::main]
async fn main() -> Result<()> {
    let server = ClaudeKnowledgeMcpServer::new();
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
