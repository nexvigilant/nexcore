//! FDA Guidance Documents MCP tools
//!
//! 5 tools for searching, retrieving, and categorizing 2,794+ FDA guidance documents.

use crate::params::knowledge::{
    FdaGuidanceGetParams, FdaGuidanceSearchParams, FdaGuidanceStatusParams, FdaGuidanceUrlParams,
};
use nexcore_fda_guidance::{format, index};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

/// Search FDA guidance documents by keyword with optional filters.
pub fn search(params: FdaGuidanceSearchParams) -> Result<CallToolResult, McpError> {
    let limit = params.limit.unwrap_or(10);
    let results = index::search(
        &params.query,
        params.center.as_deref(),
        params.product.as_deref(),
        params.status.as_deref(),
        limit,
    )
    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let output = format::format_search_results(&results, &params.query);
    Ok(CallToolResult::success(vec![Content::text(output)]))
}

/// Get a specific FDA guidance document by slug or partial title.
pub fn get(params: FdaGuidanceGetParams) -> Result<CallToolResult, McpError> {
    let found = index::get(&params.id)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    match found {
        Some(doc) => {
            let output = format::format_detail(&doc);
            Ok(CallToolResult::success(vec![Content::text(output)]))
        }
        None => Ok(CallToolResult::success(vec![Content::text(format!(
            "FDA guidance document '{}' not found. Try fda_guidance_search to find available documents.",
            params.id
        ))])),
    }
}

/// List all FDA guidance categories with document counts.
pub fn categories() -> Result<CallToolResult, McpError> {
    let docs = index::load_all()
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let by_center = index::categories_by_center(&docs);
    let by_product = index::categories_by_product(&docs);
    let by_topic = index::categories_by_topic(&docs);
    let total = docs.len();

    let output = format::format_categories(&by_center, &by_product, &by_topic, total);
    Ok(CallToolResult::success(vec![Content::text(output)]))
}

/// Get the PDF download URL for an FDA guidance document.
pub fn url(params: FdaGuidanceUrlParams) -> Result<CallToolResult, McpError> {
    let found = index::get(&params.id)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    match found {
        Some(doc) => {
            let output = match doc.pdf_url {
                Some(ref pdf) => format!("PDF URL for '{}':\n{pdf}", doc.title),
                None => format!(
                    "No PDF available for '{}'. View online at:\n{}",
                    doc.title, doc.url
                ),
            };
            Ok(CallToolResult::success(vec![Content::text(output)]))
        }
        None => Ok(CallToolResult::success(vec![Content::text(format!(
            "FDA guidance document '{}' not found.",
            params.id
        ))])),
    }
}

/// Filter FDA guidance documents by status (Draft/Final) and comment period.
pub fn status(params: FdaGuidanceStatusParams) -> Result<CallToolResult, McpError> {
    let open_only = params.open_for_comment.unwrap_or(false);
    let docs = index::by_status(&params.status, open_only)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let output = format::format_status_list(&docs, &params.status, open_only);
    Ok(CallToolResult::success(vec![Content::text(output)]))
}
