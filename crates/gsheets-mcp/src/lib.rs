#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Google Sheets MCP Server — exposes Sheets API v4 as MCP tools.
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

use crate::client::SheetsClient;
use crate::types::{
    AppendParam, BatchReadParam, ReadRangeParam, SearchParam, SpreadsheetIdParam, WriteRangeParam,
};

/// MCP server for Google Sheets API v4.
#[derive(Clone)]
pub struct GSheetsMcpServer {
    tool_router: ToolRouter<Self>,
    client: Arc<SheetsClient>,
}

#[tool_router]
impl GSheetsMcpServer {
    /// Create the server, authenticating with Google on startup.
    pub async fn new() -> Result<Self, nexcore_error::NexError> {
        let client = SheetsClient::new().await?;
        Ok(Self {
            tool_router: Self::tool_router(),
            client: Arc::new(client),
        })
    }

    // -----------------------------------------------------------------------
    // Tool: list_sheets
    // -----------------------------------------------------------------------

    #[tool(
        description = "List all sheet tabs in a Google Spreadsheet. Returns tab names, IDs, and indices."
    )]
    async fn gsheets_list_sheets(
        &self,
        Parameters(params): Parameters<SpreadsheetIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let meta = self
            .client
            .get_spreadsheet(&params.spreadsheet_id)
            .await
            .map_err(sheets_err)?;

        let mut lines = Vec::new();
        lines.push(format!("Spreadsheet: {}", meta.properties.title));
        lines.push(format!("Tabs ({}): ", meta.sheets.len()));
        for sheet in &meta.sheets {
            let p = &sheet.properties;
            lines.push(format!(
                "  [{}] {} (id={}, type={})",
                p.index,
                p.title,
                p.sheet_id,
                p.sheet_type.as_deref().unwrap_or("GRID")
            ));
        }
        Ok(text_result(&lines.join("\n")))
    }

    // -----------------------------------------------------------------------
    // Tool: read_range
    // -----------------------------------------------------------------------

    #[tool(
        description = "Read a cell range from a Google Spreadsheet. Use A1 notation, e.g. 'Sheet1!A1:C10'."
    )]
    async fn gsheets_read_range(
        &self,
        Parameters(params): Parameters<ReadRangeParam>,
    ) -> Result<CallToolResult, McpError> {
        let vr = self
            .client
            .read_range(&params.spreadsheet_id, &params.range)
            .await
            .map_err(sheets_err)?;

        let formatted = format_value_range(&vr);
        Ok(text_result(&formatted))
    }

    // -----------------------------------------------------------------------
    // Tool: batch_read
    // -----------------------------------------------------------------------

    #[tool(description = "Read multiple cell ranges in one call. Returns all ranges with headers.")]
    async fn gsheets_batch_read(
        &self,
        Parameters(params): Parameters<BatchReadParam>,
    ) -> Result<CallToolResult, McpError> {
        let batch = self
            .client
            .batch_read(&params.spreadsheet_id, &params.ranges)
            .await
            .map_err(sheets_err)?;

        let mut out = Vec::new();
        for vr in &batch.value_ranges {
            if let Some(ref range) = vr.range {
                out.push(format!("--- {range} ---"));
            }
            out.push(format_value_range(vr));
            out.push(String::new());
        }
        Ok(text_result(&out.join("\n")))
    }

    // -----------------------------------------------------------------------
    // Tool: write_range
    // -----------------------------------------------------------------------

    #[tool(
        description = "Write values to a cell range. Provide a 2D array of strings in row-major order."
    )]
    async fn gsheets_write_range(
        &self,
        Parameters(params): Parameters<WriteRangeParam>,
    ) -> Result<CallToolResult, McpError> {
        let resp = self
            .client
            .write_range(&params.spreadsheet_id, &params.range, params.values)
            .await
            .map_err(sheets_err)?;

        let summary = json!({
            "updatedRange": resp.updated_range,
            "updatedRows": resp.updated_rows,
            "updatedCells": resp.updated_cells,
        });
        Ok(text_result(
            &serde_json::to_string_pretty(&summary).unwrap_or_default(),
        ))
    }

    // -----------------------------------------------------------------------
    // Tool: append
    // -----------------------------------------------------------------------

    #[tool(description = "Append rows to the end of a table in a spreadsheet.")]
    async fn gsheets_append(
        &self,
        Parameters(params): Parameters<AppendParam>,
    ) -> Result<CallToolResult, McpError> {
        let resp = self
            .client
            .append_values(&params.spreadsheet_id, &params.range, params.values)
            .await
            .map_err(sheets_err)?;

        let summary = json!({
            "tableRange": resp.table_range,
            "updatedRows": resp.updates.as_ref().and_then(|u| u.updated_rows),
            "updatedCells": resp.updates.as_ref().and_then(|u| u.updated_cells),
        });
        Ok(text_result(
            &serde_json::to_string_pretty(&summary).unwrap_or_default(),
        ))
    }

    // -----------------------------------------------------------------------
    // Tool: metadata
    // -----------------------------------------------------------------------

    #[tool(
        description = "Get spreadsheet metadata including title, locale, timezone, and sheet list."
    )]
    async fn gsheets_metadata(
        &self,
        Parameters(params): Parameters<SpreadsheetIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let meta = self
            .client
            .get_spreadsheet(&params.spreadsheet_id)
            .await
            .map_err(sheets_err)?;

        let payload = json!({
            "spreadsheetId": meta.spreadsheet_id,
            "title": meta.properties.title,
            "locale": meta.properties.locale,
            "timeZone": meta.properties.time_zone,
            "sheetCount": meta.sheets.len(),
            "sheets": meta.sheets.iter().map(|s| json!({
                "title": s.properties.title,
                "sheetId": s.properties.sheet_id,
                "index": s.properties.index,
                "type": s.properties.sheet_type,
            })).collect::<Vec<_>>(),
        });
        Ok(text_result(
            &serde_json::to_string_pretty(&payload).unwrap_or_default(),
        ))
    }

    // -----------------------------------------------------------------------
    // Tool: search
    // -----------------------------------------------------------------------

    #[tool(
        description = "Search for a substring across all cells in a range. Returns matching cell locations and values."
    )]
    async fn gsheets_search(
        &self,
        Parameters(params): Parameters<SearchParam>,
    ) -> Result<CallToolResult, McpError> {
        // If no range specified, get all sheet names and search each.
        let ranges_to_search = if let Some(ref range) = params.range {
            vec![range.clone()]
        } else {
            // Fetch sheet metadata to get all tab names.
            let meta = self
                .client
                .get_spreadsheet(&params.spreadsheet_id)
                .await
                .map_err(sheets_err)?;
            meta.sheets
                .iter()
                .map(|s| s.properties.title.clone())
                .collect()
        };

        let query_lower = params.query.to_lowercase();
        let mut matches = Vec::new();

        for range in &ranges_to_search {
            let vr = self.client.read_range(&params.spreadsheet_id, range).await;

            let vr = match vr {
                Ok(v) => v,
                Err(e) => {
                    // Skip tabs that error (e.g., chart-only tabs).
                    tracing::debug!(range = %range, error = %e, "Skipping range in search");
                    continue;
                }
            };

            for (row_idx, row) in vr.values.iter().enumerate() {
                for (col_idx, cell) in row.iter().enumerate() {
                    let cell_str = cell_to_string(cell);
                    if cell_str.to_lowercase().contains(&query_lower) {
                        let col_letter = col_index_to_letter(col_idx);
                        matches.push(format!("{range}!{col_letter}{}: {}", row_idx + 1, cell_str));
                    }
                }
            }
        }

        if matches.is_empty() {
            Ok(text_result(&format!(
                "No matches found for '{}'",
                params.query
            )))
        } else {
            let header = format!(
                "Found {} match(es) for '{}':\n",
                matches.len(),
                params.query
            );
            Ok(text_result(&format!("{header}{}", matches.join("\n"))))
        }
    }
}

// ---------------------------------------------------------------------------
// ServerHandler impl (exact pattern from claude-fs-mcp)
// ---------------------------------------------------------------------------

impl ServerHandler for GSheetsMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Google Sheets MCP Server\n\nRead, write, search, and manage Google Spreadsheets via service account auth."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "gsheets-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: Some("Google Sheets MCP Server".into()),
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

/// Format a `ValueRange` as a human-readable table.
fn format_value_range(vr: &crate::types::ValueRange) -> String {
    if vr.values.is_empty() {
        return "(empty range)".to_string();
    }

    let mut lines = Vec::new();
    if let Some(ref range) = vr.range {
        lines.push(format!("Range: {range}"));
    }
    lines.push(format!("Rows: {}", vr.values.len()));

    for (i, row) in vr.values.iter().enumerate() {
        let cells: Vec<String> = row.iter().map(cell_to_string).collect();
        lines.push(format!("  [{}] {}", i + 1, cells.join(" | ")));
    }
    lines.join("\n")
}

/// Convert a JSON cell value to a display string.
fn cell_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// Convert a 0-based column index to A1 notation letter(s).
/// 0 → A, 1 → B, ..., 25 → Z, 26 → AA, etc.
fn col_index_to_letter(idx: usize) -> String {
    let mut result = String::new();
    let mut n = idx;
    loop {
        result.insert(0, (b'A' + (n % 26) as u8) as char);
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }
    result
}

/// Convert a `ClientError` to an MCP error.
fn sheets_err(e: crate::client::ClientError) -> McpError {
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
    fn col_index_to_letter_basic() {
        assert_eq!(col_index_to_letter(0), "A");
        assert_eq!(col_index_to_letter(1), "B");
        assert_eq!(col_index_to_letter(25), "Z");
        assert_eq!(col_index_to_letter(26), "AA");
        assert_eq!(col_index_to_letter(27), "AB");
        assert_eq!(col_index_to_letter(51), "AZ");
        assert_eq!(col_index_to_letter(52), "BA");
        assert_eq!(col_index_to_letter(701), "ZZ");
        assert_eq!(col_index_to_letter(702), "AAA");
    }

    #[test]
    fn cell_to_string_variants() {
        assert_eq!(
            cell_to_string(&serde_json::Value::String("hello".into())),
            "hello"
        );
        assert_eq!(cell_to_string(&serde_json::json!(42)), "42");
        assert_eq!(cell_to_string(&serde_json::json!(true)), "true");
        assert_eq!(cell_to_string(&serde_json::Value::Null), "");
    }

    #[test]
    fn format_empty_value_range() {
        let vr = crate::types::ValueRange {
            range: Some("Sheet1!A1:B2".into()),
            major_dimension: None,
            values: vec![],
        };
        assert_eq!(format_value_range(&vr), "(empty range)");
    }

    #[test]
    fn format_value_range_with_data() {
        let vr = crate::types::ValueRange {
            range: Some("Sheet1!A1:B2".into()),
            major_dimension: None,
            values: vec![
                vec![serde_json::json!("Name"), serde_json::json!("Score")],
                vec![serde_json::json!("Alice"), serde_json::json!(95)],
            ],
        };
        let formatted = format_value_range(&vr);
        assert!(formatted.contains("Range: Sheet1!A1:B2"));
        assert!(formatted.contains("Rows: 2"));
        assert!(formatted.contains("Name | Score"));
        assert!(formatted.contains("Alice | 95"));
    }
}
