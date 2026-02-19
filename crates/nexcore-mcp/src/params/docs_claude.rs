//! Docs Claude Parameters (Documentation Browser)
//!
//! Listing pages, retrieving content, and searching the Claude documentation.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for docs_claude_list_pages.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DocsClaudeListPagesParams {
    /// Optional category filter.
    pub category: Option<String>,
}

/// Parameters for docs_claude_get_page.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DocsClaudeGetPageParams {
    /// Page path.
    pub page: String,
}

/// Parameters for docs_claude_search.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DocsClaudeSearchParams {
    /// Search query.
    pub query: String,
    /// Max results.
    pub limit: Option<usize>,
}
