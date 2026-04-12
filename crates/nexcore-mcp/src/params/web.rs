//! Parameter structs for autonomous web MCP tools.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for `web_fetch`.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct WebFetchParams {
    /// URL to fetch.
    pub url: String,
    /// Timeout in seconds (default: 30).
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

/// Parameters for `web_search`.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct WebSearchToolParams {
    /// Search query.
    pub query: String,
    /// Maximum results (default: 10).
    #[serde(default)]
    pub max_results: Option<usize>,
}

/// Parameters for `web_extract`.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct WebExtractParams {
    /// URL to fetch and extract from.
    pub url: String,
    /// CSS selector to extract.
    pub selector: String,
}

/// Parameters for `web_links`.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct WebLinksParams {
    /// URL to extract links from.
    pub url: String,
    /// Only return external links (default: false).
    #[serde(default)]
    pub external_only: Option<bool>,
}

/// Parameters for `web_metadata`.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct WebMetadataParams {
    /// URL to extract metadata from.
    pub url: String,
}

/// Parameters for `web_crawl`.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct WebCrawlParams {
    /// Seed URL to start crawling.
    pub url: String,
    /// Maximum depth (default: 1).
    #[serde(default)]
    pub max_depth: Option<usize>,
    /// Maximum pages (default: 10).
    #[serde(default)]
    pub max_pages: Option<usize>,
    /// Only follow same-domain links (default: true).
    #[serde(default)]
    pub same_domain_only: Option<bool>,
}
