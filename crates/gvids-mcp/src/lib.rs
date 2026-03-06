#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

//! Google Vids MCP Server — edit Google Vids videos via Google Slides API.
//!
//! Google Vids uses the same underlying engine as Google Slides. This server
//! exposes the Slides API `presentations.batchUpdate` as MCP tools, enabling
//! reliable text editing (with spaces!), scene management, and batch operations.
//!
//! Tier: T3 (μ Mapping + σ Sequence + ∂ Boundary + ς State + π Persistence + ∃ Existence)

pub mod auth;
pub mod client;
pub mod types;

use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ErrorCode, Implementation, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_router};
use serde_json::json;

use crate::client::VidsClient;
use crate::types::{
    AddSceneParam, BatchUpdateParam, CreateTextBoxParam, DeleteObjectParam, GetSceneParam,
    InsertTextParam, PresentationIdParam, ReplaceTextParam, SetTextParam,
};

/// MCP server for Google Vids editing via Slides API.
#[derive(Clone)]
pub struct GVidsMcpServer {
    tool_router: ToolRouter<Self>,
    client: Arc<VidsClient>,
}

#[tool_router]
impl GVidsMcpServer {
    /// Create the server, authenticating with Google on startup.
    pub async fn new() -> Result<Self, nexcore_error::NexError> {
        let client = VidsClient::new().await?;
        Ok(Self {
            tool_router: Self::tool_router(),
            client: Arc::new(client),
        })
    }

    // -----------------------------------------------------------------------
    // Tool: list_scenes
    // -----------------------------------------------------------------------

    #[tool(
        description = "List all scenes (slides/pages) in a Google Vids video. Returns scene IDs, indices, element counts, and text summaries."
    )]
    async fn gvids_list_scenes(
        &self,
        Parameters(params): Parameters<PresentationIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let presentation = self
            .client
            .get_presentation(&params.presentation_id)
            .await
            .map_err(vids_err)?;

        let title = presentation.title.as_deref().unwrap_or("(untitled)");
        let mut lines = Vec::new();
        lines.push(format!("Video: {title}"));
        lines.push(format!("Scenes: {}", presentation.slides.len()));

        if let Some(ref size) = presentation.page_size {
            let w = size.width.as_ref().and_then(|d| d.magnitude).unwrap_or(0.0);
            let h = size
                .height
                .as_ref()
                .and_then(|d| d.magnitude)
                .unwrap_or(0.0);
            lines.push(format!("Page size: {w:.0} x {h:.0} EMU"));
        }

        lines.push(String::new());

        for (i, slide) in presentation.slides.iter().enumerate() {
            let elem_count = slide.page_elements.len();
            let text_elements: Vec<String> = slide
                .page_elements
                .iter()
                .filter_map(|pe| {
                    let text = pe.text_content()?;
                    let trimmed = text.trim().replace('\n', " ");
                    let preview = if trimmed.len() > 60 {
                        format!("{}...", &trimmed[..57])
                    } else {
                        trimmed
                    };
                    Some(format!(
                        "    {} [{}]: \"{}\"",
                        pe.object_id,
                        pe.shape_type().unwrap_or("?"),
                        preview
                    ))
                })
                .collect();

            lines.push(format!(
                "Scene {} | id={} | elements={}",
                i + 1,
                slide.object_id,
                elem_count
            ));

            if text_elements.is_empty() {
                lines.push("    (no text elements)".to_string());
            } else {
                for te in text_elements {
                    lines.push(te);
                }
            }
        }

        Ok(text_result(&lines.join("\n")))
    }

    // -----------------------------------------------------------------------
    // Tool: get_scene
    // -----------------------------------------------------------------------

    #[tool(
        description = "Get detailed information about a specific scene including all elements, text content, positions, and sizes."
    )]
    async fn gvids_get_scene(
        &self,
        Parameters(params): Parameters<GetSceneParam>,
    ) -> Result<CallToolResult, McpError> {
        let page = self
            .client
            .get_page(&params.presentation_id, &params.page_id)
            .await
            .map_err(vids_err)?;

        let mut lines = Vec::new();
        lines.push(format!(
            "Scene: {} (type: {})",
            page.object_id,
            page.page_type.as_deref().unwrap_or("SLIDE")
        ));
        lines.push(format!("Elements: {}", page.page_elements.len()));
        lines.push(String::new());

        for pe in &page.page_elements {
            lines.push(format!("--- Element: {} ---", pe.object_id));

            if let Some(shape_type) = pe.shape_type() {
                lines.push(format!("  Type: {shape_type}"));
            }
            if let Some(ph_type) = pe.placeholder_type() {
                lines.push(format!("  Placeholder: {ph_type}"));
            }
            if let Some(ref size) = pe.size {
                let w = size.width.as_ref().and_then(|d| d.magnitude).unwrap_or(0.0);
                let h = size
                    .height
                    .as_ref()
                    .and_then(|d| d.magnitude)
                    .unwrap_or(0.0);
                lines.push(format!("  Size: {w:.0} x {h:.0} EMU"));
            }
            if let Some(text) = pe.text_content() {
                let display = text.trim().replace('\n', "\\n");
                lines.push(format!("  Text: \"{display}\""));
            }
            if pe.image.is_some() {
                lines.push("  [IMAGE]".to_string());
            }
            if pe.video.is_some() {
                lines.push("  [VIDEO]".to_string());
            }
        }

        Ok(text_result(&lines.join("\n")))
    }

    // -----------------------------------------------------------------------
    // Tool: set_text
    // -----------------------------------------------------------------------

    #[tool(
        description = "Set text on a shape element, replacing all existing text. This is the primary text editing tool — it correctly handles spaces, unlike Chrome DevTools fill()."
    )]
    async fn gvids_set_text(
        &self,
        Parameters(params): Parameters<SetTextParam>,
    ) -> Result<CallToolResult, McpError> {
        let resp = self
            .client
            .set_text(&params.presentation_id, &params.object_id, &params.text)
            .await
            .map_err(vids_err)?;

        Ok(text_result(&format!(
            "Text set successfully on {}.\nReplies: {}",
            params.object_id,
            resp.replies.len()
        )))
    }

    // -----------------------------------------------------------------------
    // Tool: insert_text
    // -----------------------------------------------------------------------

    #[tool(
        description = "Insert text at a specific position in a shape element. Use insertion_index=0 for beginning, or omit to append."
    )]
    async fn gvids_insert_text(
        &self,
        Parameters(params): Parameters<InsertTextParam>,
    ) -> Result<CallToolResult, McpError> {
        let idx = params.insertion_index.unwrap_or(0);
        let requests = vec![json!({
            "insertText": {
                "objectId": params.object_id,
                "insertionIndex": idx,
                "text": params.text
            }
        })];

        let resp = self
            .client
            .batch_update(&params.presentation_id, requests)
            .await
            .map_err(vids_err)?;

        Ok(text_result(&format!(
            "Text inserted at index {} in {}.\nReplies: {}",
            idx,
            params.object_id,
            resp.replies.len()
        )))
    }

    // -----------------------------------------------------------------------
    // Tool: replace_text
    // -----------------------------------------------------------------------

    #[tool(
        description = "Find and replace text across ALL scenes in the video. Useful for batch text corrections."
    )]
    async fn gvids_replace_text(
        &self,
        Parameters(params): Parameters<ReplaceTextParam>,
    ) -> Result<CallToolResult, McpError> {
        let resp = self
            .client
            .replace_all_text(
                &params.presentation_id,
                &params.find,
                &params.replace_with,
                params.match_case,
            )
            .await
            .map_err(vids_err)?;

        // The reply contains occurrencesChanged
        let changed = resp
            .replies
            .first()
            .and_then(|r| r.get("replaceAllText"))
            .and_then(|r| r.get("occurrencesChanged"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        Ok(text_result(&format!(
            "Replaced '{}' → '{}': {changed} occurrence(s) changed",
            params.find, params.replace_with
        )))
    }

    // -----------------------------------------------------------------------
    // Tool: add_scene
    // -----------------------------------------------------------------------

    #[tool(
        description = "Add a new blank scene (slide) to the video. Optionally specify position and layout."
    )]
    async fn gvids_add_scene(
        &self,
        Parameters(params): Parameters<AddSceneParam>,
    ) -> Result<CallToolResult, McpError> {
        let resp = self
            .client
            .create_slide(&params.presentation_id, params.insertion_index)
            .await
            .map_err(vids_err)?;

        // Extract the created slide's object ID from the reply
        let new_id = resp
            .replies
            .first()
            .and_then(|r| r.get("createSlide"))
            .and_then(|r| r.get("objectId"))
            .and_then(|v| v.as_str())
            .unwrap_or("(unknown)");

        Ok(text_result(&format!(
            "Scene created: {new_id} (at index {})",
            params
                .insertion_index
                .map_or("end".to_string(), |i| i.to_string())
        )))
    }

    // -----------------------------------------------------------------------
    // Tool: delete_object
    // -----------------------------------------------------------------------

    #[tool(description = "Delete a scene (page) or element (shape/image) by its object ID.")]
    async fn gvids_delete_object(
        &self,
        Parameters(params): Parameters<DeleteObjectParam>,
    ) -> Result<CallToolResult, McpError> {
        let resp = self
            .client
            .delete_object(&params.presentation_id, &params.object_id)
            .await
            .map_err(vids_err)?;

        Ok(text_result(&format!(
            "Deleted object: {}\nReplies: {}",
            params.object_id,
            resp.replies.len()
        )))
    }

    // -----------------------------------------------------------------------
    // Tool: create_text_box
    // -----------------------------------------------------------------------

    #[tool(
        description = "Create a new text box on a scene with specified text, position, and size. Position/size use EMU (1 inch = 914400 EMU)."
    )]
    async fn gvids_create_text_box(
        &self,
        Parameters(params): Parameters<CreateTextBoxParam>,
    ) -> Result<CallToolResult, McpError> {
        let resp = self
            .client
            .create_text_box(
                &params.presentation_id,
                &params.page_id,
                &params.text,
                params.x_emu,
                params.y_emu,
                params.width_emu,
                params.height_emu,
            )
            .await
            .map_err(vids_err)?;

        // Extract created shape ID
        let new_id = resp
            .replies
            .first()
            .and_then(|r| r.get("createShape"))
            .and_then(|r| r.get("objectId"))
            .and_then(|v| v.as_str())
            .unwrap_or("(unknown)");

        Ok(text_result(&format!(
            "Text box created: {new_id} on page {}\nText: \"{}\"",
            params.page_id, params.text
        )))
    }

    // -----------------------------------------------------------------------
    // Tool: batch_update
    // -----------------------------------------------------------------------

    #[tool(
        description = "Execute a raw batchUpdate with custom request objects. For advanced operations not covered by other tools. See Google Slides API batchUpdate docs for request format."
    )]
    async fn gvids_batch_update(
        &self,
        Parameters(params): Parameters<BatchUpdateParam>,
    ) -> Result<CallToolResult, McpError> {
        let req_count = params.requests.len();
        let resp = self
            .client
            .batch_update(&params.presentation_id, params.requests)
            .await
            .map_err(vids_err)?;

        let reply_summary = serde_json::to_string_pretty(&resp.replies).unwrap_or_default();
        Ok(text_result(&format!(
            "Batch update complete: {req_count} request(s), {} reply(ies)\n{reply_summary}",
            resp.replies.len()
        )))
    }
}

// ---------------------------------------------------------------------------
// ServerHandler impl
// ---------------------------------------------------------------------------

impl ServerHandler for GVidsMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Google Vids MCP Server\n\nEdit Google Vids videos via Google Slides API.\nSet text (with spaces!), manage scenes, find/replace, create text boxes.\nAuthentication: gcloud ADC or service account."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "gvids-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: Some("Google Vids MCP Server".into()),
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a `ClientError` to an MCP error.
fn vids_err(e: crate::client::ClientError) -> McpError {
    McpError::new(ErrorCode(500), e.to_string(), None)
}

/// Shorthand for a text-only `CallToolResult`.
fn text_result(s: &str) -> CallToolResult {
    CallToolResult::success(vec![rmcp::model::Content::text(s)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_result_creates_success() {
        let result = text_result("hello");
        // CallToolResult has is_error field; success means it's not set or false
        assert!(!result.is_error.unwrap_or(false));
    }

    #[test]
    fn vids_err_uses_500() {
        let err = vids_err(crate::client::ClientError::Http("test".into()));
        assert_eq!(err.code.0, 500);
    }
}
