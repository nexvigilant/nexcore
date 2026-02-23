//! Claude Code Documentation MCP Server

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;
use std::time::Duration;

use nexcore_error::{Context, Result};
use moka::future::Cache;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, ListToolsResult, PaginatedRequestParams,
    ServerCapabilities, ServerInfo,
};
use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::stdio;
use rmcp::{ErrorData as McpError, ServerHandler, ServiceExt, tool, tool_router};

mod tooling;

/// Tier: T3 (Domain-specific documentation page)
/// Grounds to: T1 primitives via String fields
/// Ord: N/A (composite record)
#[derive(Debug, Clone)]
struct DocPage {
    title: String,
    url: String,
    path: String,
    description: String,
}

/// Tier: T3 (Domain-specific MCP server)
/// Grounds to: T1 primitives via Arc/Cache/String and tool router state
/// Ord: N/A (stateful service)
#[derive(Clone)]
pub struct ClaudeCodeDocsServer {
    http_client: reqwest::Client,
    cache: Cache<String, String>,
    pages: Arc<Vec<DocPage>>,
    tool_router: ToolRouter<Self>,
}

/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to: T1 primitives via String/Option
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ListPagesParams {
    /// Optional category filter (e.g., "setup", "integrations", "security")
    #[serde(default)]
    category: Option<String>,
}

/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to: T1 primitives via String
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GetPageParams {
    /// The page path or name (e.g., "mcp", "hooks-guide", "settings")
    page: String,
}

/// Tier: T3 (Domain-specific MCP tool parameters)
/// Grounds to: T1 primitives via String/Option/usize
/// Ord: N/A (composite parameters not orderable)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SearchDocsParams {
    /// The search query
    query: String,
    /// Maximum results to return (default: 5)
    #[serde(default)]
    limit: Option<usize>,
}

impl ClaudeCodeDocsServer {
    fn init_pages() -> Vec<DocPage> {
        vec![
            DocPage {
                title: "Overview".into(),
                path: "overview".into(),
                url: "https://code.claude.com/docs/en/overview".into(),
                description: "Learn about Claude Code".into(),
            },
            DocPage {
                title: "Quickstart".into(),
                path: "quickstart".into(),
                url: "https://code.claude.com/docs/en/quickstart".into(),
                description: "Getting started guide".into(),
            },
            DocPage {
                title: "Setup".into(),
                path: "setup".into(),
                url: "https://code.claude.com/docs/en/setup".into(),
                description: "Install and authenticate".into(),
            },
            DocPage {
                title: "How It Works".into(),
                path: "how-claude-code-works".into(),
                url: "https://code.claude.com/docs/en/how-claude-code-works".into(),
                description: "Agentic loop and tools".into(),
            },
            DocPage {
                title: "Common Workflows".into(),
                path: "common-workflows".into(),
                url: "https://code.claude.com/docs/en/common-workflows".into(),
                description: "Step-by-step guides".into(),
            },
            DocPage {
                title: "Best Practices".into(),
                path: "best-practices".into(),
                url: "https://code.claude.com/docs/en/best-practices".into(),
                description: "Tips and patterns".into(),
            },
            DocPage {
                title: "CLI Reference".into(),
                path: "cli-reference".into(),
                url: "https://code.claude.com/docs/en/cli-reference".into(),
                description: "Commands and flags".into(),
            },
            DocPage {
                title: "Settings".into(),
                path: "settings".into(),
                url: "https://code.claude.com/docs/en/settings".into(),
                description: "Configuration options".into(),
            },
            DocPage {
                title: "MCP".into(),
                path: "mcp".into(),
                url: "https://code.claude.com/docs/en/mcp".into(),
                description: "Model Context Protocol".into(),
            },
            DocPage {
                title: "Skills".into(),
                path: "skills".into(),
                url: "https://code.claude.com/docs/en/skills".into(),
                description: "Extend Claude's capabilities".into(),
            },
            DocPage {
                title: "Hooks Guide".into(),
                path: "hooks-guide".into(),
                url: "https://code.claude.com/docs/en/hooks-guide".into(),
                description: "Customize behavior".into(),
            },
            DocPage {
                title: "Hooks Reference".into(),
                path: "hooks".into(),
                url: "https://code.claude.com/docs/en/hooks".into(),
                description: "Hooks documentation".into(),
            },
            DocPage {
                title: "Subagents".into(),
                path: "sub-agents".into(),
                url: "https://code.claude.com/docs/en/sub-agents".into(),
                description: "Specialized AI agents".into(),
            },
            DocPage {
                title: "Plugins".into(),
                path: "plugins".into(),
                url: "https://code.claude.com/docs/en/plugins".into(),
                description: "Create plugins".into(),
            },
            DocPage {
                title: "VS Code".into(),
                path: "vs-code".into(),
                url: "https://code.claude.com/docs/en/vs-code".into(),
                description: "VS Code extension".into(),
            },
            DocPage {
                title: "JetBrains".into(),
                path: "jetbrains".into(),
                url: "https://code.claude.com/docs/en/jetbrains".into(),
                description: "JetBrains IDEs".into(),
            },
            DocPage {
                title: "GitHub Actions".into(),
                path: "github-actions".into(),
                url: "https://code.claude.com/docs/en/github-actions".into(),
                description: "CI/CD integration".into(),
            },
            DocPage {
                title: "Security".into(),
                path: "security".into(),
                url: "https://code.claude.com/docs/en/security".into(),
                description: "Security safeguards".into(),
            },
            DocPage {
                title: "Troubleshooting".into(),
                path: "troubleshooting".into(),
                url: "https://code.claude.com/docs/en/troubleshooting".into(),
                description: "Common issues".into(),
            },
        ]
    }

    async fn fetch_page(&self, url: &str) -> Result<String, String> {
        if let Some(cached) = self.cache.get(url).await {
            return Ok(cached);
        }
        let md_url = format!("{}.md", url);
        let response = self
            .http_client
            .get(&md_url)
            .send()
            .await
            .map_err(|e| format!("Fetch error: {}", e))?;
        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()));
        }
        let content = response
            .text()
            .await
            .map_err(|e| format!("Read error: {}", e))?;
        self.cache.insert(url.to_string(), content.clone()).await;
        Ok(content)
    }
}

#[tool_router]
impl ClaudeCodeDocsServer {
    pub fn new() -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("claude-code-docs-mcp/0.1.0")
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            http_client,
            cache: Cache::builder()
                .max_capacity(100)
                .time_to_live(Duration::from_secs(3600))
                .build(),
            pages: Arc::new(Self::init_pages()),
            tool_router: Self::tool_router(),
        })
    }

    #[tool(description = "List all available Claude Code documentation pages")]
    async fn list_pages(
        &self,
        Parameters(params): Parameters<ListPagesParams>,
    ) -> Result<CallToolResult, McpError> {
        let pages: Vec<_> = self
            .pages
            .iter()
            .filter(|p| {
                params.category.as_ref().is_none_or(|cat| {
                    let c = cat.to_lowercase();
                    p.path.contains(&c) || p.title.to_lowercase().contains(&c)
                })
            })
            .map(|p| format!("- **{}** (`{}`): {}", p.title, p.path, p.description))
            .collect();
        let result = if pages.is_empty() {
            "No pages found.".into()
        } else {
            format!(
                "# Claude Code Docs\n\n{}\n\n*Use get_page with path.*",
                pages.join("\n")
            )
        };
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Get content of a specific documentation page")]
    async fn get_page(
        &self,
        Parameters(params): Parameters<GetPageParams>,
    ) -> Result<CallToolResult, McpError> {
        let q = params.page.to_lowercase();
        let page = self
            .pages
            .iter()
            .find(|p| p.path == q || p.path.contains(&q));
        match page {
            Some(p) => match self.fetch_page(&p.url).await {
                Ok(c) => Ok(CallToolResult::success(vec![Content::text(format!(
                    "# {}\n\n{}",
                    p.title, c
                ))])),
                Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                    "Error: {}",
                    e
                ))])),
            },
            None => Ok(CallToolResult::success(vec![Content::text(
                "Page not found. Use list_pages.",
            )])),
        }
    }

    #[tool(description = "Search documentation for a topic")]
    async fn search_docs(
        &self,
        Parameters(params): Parameters<SearchDocsParams>,
    ) -> Result<CallToolResult, McpError> {
        let q = params.query.to_lowercase();
        let limit = params.limit.unwrap_or(5);
        let mut scored: Vec<_> = self
            .pages
            .iter()
            .filter_map(|p| {
                let mut s = 0i32;
                if p.title.to_lowercase().contains(&q) {
                    s += 50;
                }
                if p.path.contains(&q) {
                    s += 30;
                }
                if p.description.to_lowercase().contains(&q) {
                    s += 20;
                }
                if s > 0 { Some((p, s)) } else { None }
            })
            .collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        let results: Vec<_> = scored
            .into_iter()
            .take(limit)
            .map(|(p, _)| format!("- **{}** (`{}`): {}", p.title, p.path, p.description))
            .collect();
        let result = if results.is_empty() {
            format!("No results for '{}'.", params.query)
        } else {
            format!("# Results for '{}'\n\n{}", params.query, results.join("\n"))
        };
        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    #[tool(description = "Get the documentation index")]
    async fn get_docs_index(&self) -> Result<CallToolResult, McpError> {
        match self
            .fetch_page("https://code.claude.com/docs/llms.txt")
            .await
        {
            Ok(c) => Ok(CallToolResult::success(vec![Content::text(format!(
                "# Index\n\n{}",
                c
            ))])),
            Err(_) => {
                let list: Vec<_> = self
                    .pages
                    .iter()
                    .map(|p| format!("- {} ({})", p.title, p.path))
                    .collect();
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "# Index (cached)\n\n{}",
                    list.join("\n")
                ))]))
            }
        }
    }
}

impl ServerHandler for ClaudeCodeDocsServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Claude Code Documentation Server. Tools: list_pages (browse docs), \
                 get_page (fetch content), search_docs (find topics), get_docs_index (full index)."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            let tool_name = request.name.as_ref().to_string();
            if let Err(err) = tooling::tool_gate().check(&tool_name) {
                return Ok(tooling::gated_result(&tool_name, err));
            }
            let tcc = ToolCallContext::new(self, request, context);
            let result = self.tool_router.call(tcc).await?;
            Ok(tooling::wrap_result(&tool_name, result))
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
    let server = ClaudeCodeDocsServer::new()?;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
