//! NexCore Bridge MCP Server Library
//!
//! Connects MCP tool calls to the NexCore REST API.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod params;

use reqwest::Client;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ErrorCode, Implementation, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_router};

/// NexCore Bridge MCP Server
///
/// Acts as a bridge between MCP and the NexCore REST API.
#[derive(Clone)]
pub struct NexCoreBridgeMcpServer {
    /// HTTP client
    pub client: Client,
    /// Base URL for the NexCore API
    pub api_url: String,
    /// Optional API Key for authentication
    pub api_key: Option<String>,
    /// Tool router
    tool_router: ToolRouter<Self>,
}

impl NexCoreBridgeMcpServer {
    /// Create a new bridge server
    pub fn new(api_url: String, api_key: Option<String>) -> anyhow::Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            api_url: api_url.trim_end_matches('/').to_string(),
            api_key,
            tool_router: Self::tool_router(),
        })
    }

    /// Helper to send a POST request to the API
    async fn post<P: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        payload: &P,
    ) -> Result<R, McpError> {
        let url = format!("{}{}", self.api_url, path);
        let mut req = self.client.post(&url).json(payload);

        if let Some(key) = &self.api_key {
            req = req.header("X-API-Key", key);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| McpError::new(ErrorCode(500), format!("API request failed: {e}"), None))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let error_text = resp
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(McpError::new(
                ErrorCode(500),
                format!("API returned error ({status}): {error_text}"),
                None,
            ));
        }

        resp.json::<R>().await.map_err(|e| {
            McpError::new(
                ErrorCode(500),
                format!("Failed to parse API response: {e}"),
                None,
            )
        })
    }
}

impl ServerHandler for NexCoreBridgeMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("NexCore Bridge MCP Server connects to NexCore REST API".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "nexcore-bridge-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: Some("NexCore Bridge MCP Server".into()),
                ..Default::default()
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

#[tool_router]
impl NexCoreBridgeMcpServer {
    #[tool(description = "Check health of the bridge and the connected NexCore API")]
    async fn bridge_health(&self) -> Result<CallToolResult, McpError> {
        let url = format!("{}/health/ready", self.api_url);
        let mut req = self.client.get(&url);

        if let Some(key) = &self.api_key {
            req = req.header("X-API-Key", key);
        }

        let resp = req.send().await.map_err(|e| {
            McpError::new(
                ErrorCode(500),
                format!("API health check failed: {e}"),
                None,
            )
        })?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();

        let result = serde_json::json!({
            "bridge": "healthy",
            "api_url": self.api_url,
            "api_status": status.to_string(),
            "api_response": body
        });

        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Calculate Levenshtein distance via NexCore API bridge")]
    async fn bridge_foundation_levenshtein(
        &self,
        Parameters(params): Parameters<params::LevenshteinParams>,
    ) -> Result<CallToolResult, McpError> {
        let body: serde_json::Value = self.post("/api/v1/foundation/levenshtein", &params).await?;
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&body).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Calculate SHA-256 hash via NexCore API bridge")]
    async fn bridge_foundation_sha256(
        &self,
        Parameters(params): Parameters<params::Sha256Params>,
    ) -> Result<CallToolResult, McpError> {
        let body: serde_json::Value = self.post("/api/v1/foundation/sha256", &params).await?;
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&body).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Parse YAML to JSON via NexCore API bridge")]
    async fn bridge_foundation_yaml_parse(
        &self,
        Parameters(params): Parameters<params::YamlParseParams>,
    ) -> Result<CallToolResult, McpError> {
        let body: serde_json::Value = self.post("/api/v1/foundation/yaml/parse", &params).await?;
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&body).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Perform complete PV signal analysis via NexCore API bridge")]

    async fn bridge_pv_signal_complete(
        &self,

        Parameters(params): Parameters<params::SignalCompleteParams>,
    ) -> Result<CallToolResult, McpError> {
        let body: serde_json::Value = self.post("/api/v1/pv/signal/complete", &params).await?;

        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&body).unwrap_or_default(),
        )]))
    }

    #[tool(
        description = "Extract metadata (input/output schema) for a specific skill via NexCore API bridge"
    )]

    async fn bridge_skill_schema(
        &self,

        Parameters(params): Parameters<params::SkillSchemaParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!("/api/v1/skills/{}/schema", params.name);

        // Use manual GET instead of self.post helper since it's a GET request

        let url = format!("{}{}", self.api_url, path);

        let mut req = self.client.get(&url);

        if let Some(key) = &self.api_key {
            req = req.header("X-API-Key", key);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| McpError::new(ErrorCode(500), format!("API request failed: {e}"), None))?;

        if !resp.status().is_success() {
            let status = resp.status();

            let error_text = resp
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(McpError::new(
                ErrorCode(500),
                format!("API returned error ({status}): {error_text}"),
                None,
            ));
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            McpError::new(
                ErrorCode(500),
                format!("Failed to parse API response: {e}"),
                None,
            )
        })?;

        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&body).unwrap_or_default(),
        )]))
    }

    #[tool(
        description = "Get full details (metadata, tags, compliance) for a specific skill via NexCore API bridge"
    )]

    async fn bridge_skill_get(
        &self,

        Parameters(params): Parameters<params::SkillSchemaParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!("/api/v1/skills/{}", params.name);

        let url = format!("{}{}", self.api_url, path);

        let mut req = self.client.get(&url);

        if let Some(key) = &self.api_key {
            req = req.header("X-API-Key", key);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| McpError::new(ErrorCode(500), format!("API request failed: {e}"), None))?;

        if !resp.status().is_success() {
            let status = resp.status();

            let error_text = resp
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(McpError::new(
                ErrorCode(500),
                format!("API returned error ({status}): {error_text}"),
                None,
            ));
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            McpError::new(
                ErrorCode(500),
                format!("Failed to parse API response: {e}"),
                None,
            )
        })?;

        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&body).unwrap_or_default(),
        )]))
    }
}
