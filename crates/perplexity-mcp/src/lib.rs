// Copyright 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! MCP server for Perplexity AI search-grounded research.
//!
//! Exposes 4 tools via MCP (stdio transport):
//! - `perplexity_search` — General search with optional filters
//! - `perplexity_research` — Route by use case (general/competitive/regulatory)
//! - `perplexity_competitive` — Domain-filtered competitor intelligence
//! - `perplexity_regulatory` — FDA/EMA/ICH/WHO pre-filtered search

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(missing_docs)]
use nexcore_perplexity::client::PerplexityClient;
use nexcore_perplexity::research::{
    ResearchUseCase, research_competitive, research_general, research_regulatory,
};
use nexcore_perplexity::types::{ChatRequest, Model, SearchRecency};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, ErrorCode, Implementation, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// Param Structs
// ============================================================================

/// Parameters for general Perplexity search.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchParams {
    /// The search query.
    pub query: String,
    /// Model to use: "sonar" (fast), "sonar-pro" (deep), "sonar-deep-research" (agent).
    /// Defaults to "sonar".
    pub model: Option<String>,
    /// Recency filter: "hour", "day", "week", "month".
    pub recency: Option<String>,
    /// Domain filter list (e.g. \["fda.gov", "ema.europa.eu"\]).
    pub domains: Option<Vec<String>>,
}

/// Parameters for routed research by use case.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResearchParams {
    /// The research query.
    pub query: String,
    /// Use case: "general", "competitive", "regulatory".
    /// Defaults to "general".
    pub use_case: String,
}

/// Parameters for competitive intelligence search.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CompetitiveParams {
    /// The competitive intelligence query.
    pub query: String,
    /// Competitor domain list (e.g. \["competitor.com", "rival.io"\]).
    pub competitors: Vec<String>,
}

/// Parameters for regulatory intelligence search.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RegulatoryParams {
    /// The regulatory query.
    pub query: String,
    /// Recency filter: "hour", "day", "week", "month".
    /// Defaults to "month".
    pub recency: Option<String>,
}

// ============================================================================
// Helpers
// ============================================================================

fn format_result(text: String) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

fn format_error(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(format!(
        "Error: {msg}"
    ))]))
}

fn get_client() -> Result<PerplexityClient, String> {
    PerplexityClient::from_env().map_err(|e| format!("{e}"))
}

// ============================================================================
// Server
// ============================================================================

/// Perplexity AI MCP server with 4 search/research tools.
#[derive(Clone)]
pub struct PerplexityMcpServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl PerplexityMcpServer {
    /// Create a new server instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Search Perplexity AI with optional model, recency, and domain filters. Returns search-grounded answers with citations."
    )]
    async fn perplexity_search(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let client = match get_client() {
            Ok(c) => c,
            Err(e) => return format_error(&e),
        };

        let model = params
            .model
            .as_deref()
            .map(Model::from_str_or_default)
            .unwrap_or(Model::Sonar);

        let mut request = ChatRequest::simple(model, &params.query);

        if let Some(ref recency_str) = params.recency {
            if let Some(recency) = SearchRecency::from_str_opt(recency_str) {
                request = request.with_recency(recency);
            }
        }

        if let Some(ref domains) = params.domains {
            if !domains.is_empty() {
                request = request.with_domain_filter(domains.clone());
            }
        }

        match client.chat(request).await {
            Ok(response) => format_result(response.formatted()),
            Err(e) => format_error(&format!("{e}")),
        }
    }

    #[tool(
        description = "Route research by use case: 'general' (open web), 'competitive' (market intel with sonar-pro), or 'regulatory' (FDA/EMA/ICH/WHO filtered)."
    )]
    async fn perplexity_research(
        &self,
        Parameters(params): Parameters<ResearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let client = match get_client() {
            Ok(c) => c,
            Err(e) => return format_error(&e),
        };

        let use_case =
            ResearchUseCase::from_str_opt(&params.use_case).unwrap_or(ResearchUseCase::General);

        let result = match use_case {
            ResearchUseCase::General => research_general(&client, &params.query, None).await,
            ResearchUseCase::Competitive => {
                research_competitive(&client, &params.query, &[], Some(Model::SonarPro)).await
            }
            ResearchUseCase::Regulatory => {
                research_regulatory(&client, &params.query, None, None).await
            }
        };

        match result {
            Ok(r) => format_result(r.formatted()),
            Err(e) => format_error(&format!("{e}")),
        }
    }

    #[tool(
        description = "Competitive intelligence search filtered to specified competitor domains. Uses sonar-pro with month recency by default."
    )]
    async fn perplexity_competitive(
        &self,
        Parameters(params): Parameters<CompetitiveParams>,
    ) -> Result<CallToolResult, McpError> {
        let client = match get_client() {
            Ok(c) => c,
            Err(e) => return format_error(&e),
        };

        match research_competitive(&client, &params.query, &params.competitors, None).await {
            Ok(r) => format_result(r.formatted()),
            Err(e) => format_error(&format!("{e}")),
        }
    }

    #[tool(
        description = "Regulatory intelligence pre-filtered to FDA, EMA, ICH, WHO, ClinicalTrials.gov, and other regulatory domains. Uses sonar-pro."
    )]
    async fn perplexity_regulatory(
        &self,
        Parameters(params): Parameters<RegulatoryParams>,
    ) -> Result<CallToolResult, McpError> {
        let client = match get_client() {
            Ok(c) => c,
            Err(e) => return format_error(&e),
        };

        let recency = params
            .recency
            .as_deref()
            .and_then(SearchRecency::from_str_opt);

        match research_regulatory(&client, &params.query, recency, None).await {
            Ok(r) => format_result(r.formatted()),
            Err(e) => format_error(&format!("{e}")),
        }
    }
}

impl Default for PerplexityMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerHandler for PerplexityMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                r#"Perplexity MCP Server

Search-grounded AI research with citations via Perplexity Sonar API.
4 tools: search (general), research (routed), competitive, regulatory.
"#
                .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "perplexity-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: Some("Perplexity MCP Server".into()),
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_creates_successfully() {
        let server = PerplexityMcpServer::new();
        let info = server.get_info();
        assert_eq!(info.server_info.name, "perplexity-mcp");
    }

    #[test]
    fn search_params_serialize() {
        let params = SearchParams {
            query: "FDA drug approvals 2026".to_string(),
            model: Some("sonar-pro".to_string()),
            recency: Some("week".to_string()),
            domains: Some(vec!["fda.gov".to_string()]),
        };
        let json = serde_json::to_string(&params);
        assert!(json.is_ok());
    }

    #[test]
    fn research_params_serialize() {
        let params = ResearchParams {
            query: "test query".to_string(),
            use_case: "regulatory".to_string(),
        };
        let json = serde_json::to_string(&params);
        assert!(json.is_ok());
    }

    #[test]
    fn competitive_params_serialize() {
        let params = CompetitiveParams {
            query: "market analysis".to_string(),
            competitors: vec!["competitor.com".to_string()],
        };
        let json = serde_json::to_string(&params);
        assert!(json.is_ok());
    }

    #[test]
    fn regulatory_params_serialize() {
        let params = RegulatoryParams {
            query: "ICH E2E guidance".to_string(),
            recency: None,
        };
        let json = serde_json::to_string(&params);
        assert!(json.is_ok());
    }

    #[test]
    fn format_result_creates_success() {
        let result = format_result("test output".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn format_error_creates_success_with_error_text() {
        let result = format_error("something broke");
        assert!(result.is_ok());
    }

    #[test]
    fn default_creates_server() {
        let server = PerplexityMcpServer::default();
        let tools = server.tool_router.list_all();
        assert_eq!(tools.len(), 4);
    }
}
