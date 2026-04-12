//! Autonomous web MCP tools — fetch, search, extract, crawl.

use crate::params::web::*;
use nexcore_web::crawl::{CrawlConfig, crawl};
use nexcore_web::extract;
use nexcore_web::fetch::{FetchConfig, fetch_page};
use nexcore_web::metadata::extract_metadata;
use nexcore_web::search::{SearchConfig, web_search};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Fetch a URL and return clean text.
pub async fn web_fetch_tool(params: WebFetchParams) -> Result<CallToolResult, McpError> {
    let config = FetchConfig {
        timeout_secs: params.timeout_secs.unwrap_or(30),
        ..FetchConfig::default()
    };

    match fetch_page(&params.url, &config).await {
        Ok(result) => {
            // Truncate to 8000 chars for MCP response
            let text = if result.text.len() > 8000 {
                format!(
                    "{}... [truncated, {} total chars]",
                    &result.text[..8000],
                    result.text_length
                )
            } else {
                result.text
            };

            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&json!({
                    "success": true,
                    "url": result.url,
                    "status": result.status,
                    "title": result.title,
                    "text": text,
                    "text_length": result.text_length,
                    "content_type": result.content_type,
                    "elapsed_ms": result.elapsed_ms,
                }))
                .unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Fetch failed: {e}"
        ))])),
    }
}

/// Search the web and return results.
pub async fn web_search_tool(params: WebSearchToolParams) -> Result<CallToolResult, McpError> {
    let config = SearchConfig {
        max_results: params.max_results.unwrap_or(10),
        ..SearchConfig::default()
    };

    match web_search(&params.query, &config).await {
        Ok(results) => {
            let items: Vec<serde_json::Value> = results
                .iter()
                .map(|r| {
                    json!({
                        "title": r.title,
                        "url": r.url,
                        "snippet": r.snippet,
                    })
                })
                .collect();

            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&json!({
                    "success": true,
                    "query": params.query,
                    "count": items.len(),
                    "results": items,
                }))
                .unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Search failed: {e}"
        ))])),
    }
}

/// Extract content using CSS selector.
pub async fn web_extract_tool(params: WebExtractParams) -> Result<CallToolResult, McpError> {
    let config = FetchConfig::default();

    let page = fetch_page(&params.url, &config)
        .await
        .map_err(|e| McpError::internal_error(format!("fetch: {e}"), None))?;

    // Re-fetch raw HTML for selector extraction
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("NexVigilant-Web/0.1")
        .build()
        .map_err(|e| McpError::internal_error(format!("client: {e}"), None))?;

    let html = client
        .get(&params.url)
        .send()
        .await
        .map_err(|e| McpError::internal_error(format!("fetch: {e}"), None))?
        .text()
        .await
        .map_err(|e| McpError::internal_error(format!("body: {e}"), None))?;

    match extract::extract_css(&html, &params.selector) {
        Ok(result) => Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&json!({
                "success": true,
                "url": page.url,
                "selector": result.selector,
                "count": result.count,
                "matches": result.matches,
            }))
            .unwrap_or_default(),
        )])),
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Extract failed: {e}"
        ))])),
    }
}

/// Extract all links from a page.
pub async fn web_links_tool(params: WebLinksParams) -> Result<CallToolResult, McpError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("NexVigilant-Web/0.1")
        .build()
        .map_err(|e| McpError::internal_error(format!("client: {e}"), None))?;

    let html = client
        .get(&params.url)
        .send()
        .await
        .map_err(|e| McpError::internal_error(format!("fetch: {e}"), None))?
        .text()
        .await
        .map_err(|e| McpError::internal_error(format!("body: {e}"), None))?;

    let mut links = extract::extract_links(&html, &params.url);

    if params.external_only.unwrap_or(false) {
        links.retain(|l| l.is_external);
    }

    let items: Vec<serde_json::Value> = links
        .iter()
        .map(|l| {
            json!({
                "url": l.url,
                "text": l.text,
                "external": l.is_external,
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json!({
            "success": true,
            "url": params.url,
            "count": items.len(),
            "links": items,
        }))
        .unwrap_or_default(),
    )]))
}

/// Extract page metadata (title, description, OpenGraph).
pub async fn web_metadata_tool(params: WebMetadataParams) -> Result<CallToolResult, McpError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("NexVigilant-Web/0.1")
        .build()
        .map_err(|e| McpError::internal_error(format!("client: {e}"), None))?;

    let html = client
        .get(&params.url)
        .send()
        .await
        .map_err(|e| McpError::internal_error(format!("fetch: {e}"), None))?
        .text()
        .await
        .map_err(|e| McpError::internal_error(format!("body: {e}"), None))?;

    let meta = extract_metadata(&html);

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json!({
            "success": true,
            "url": params.url,
            "title": meta.title,
            "description": meta.description,
            "canonical": meta.canonical,
            "og_title": meta.og_title,
            "og_description": meta.og_description,
            "og_image": meta.og_image,
            "og_type": meta.og_type,
            "language": meta.language,
            "keywords": meta.keywords,
        }))
        .unwrap_or_default(),
    )]))
}

/// Crawl a website from a seed URL.
pub async fn web_crawl_tool(params: WebCrawlParams) -> Result<CallToolResult, McpError> {
    let config = CrawlConfig {
        max_depth: params.max_depth.unwrap_or(1),
        max_pages: params.max_pages.unwrap_or(10),
        same_domain_only: params.same_domain_only.unwrap_or(true),
        ..CrawlConfig::default()
    };

    match crawl(&params.url, &config).await {
        Ok(result) => {
            let pages: Vec<serde_json::Value> = result
                .pages
                .iter()
                .map(|p| {
                    json!({
                        "url": p.url,
                        "depth": p.depth,
                        "title": p.title,
                        "text_preview": if p.text.len() > 200 { format!("{}...", &p.text[..200]) } else { p.text.clone() },
                        "link_count": p.link_count,
                        "status": p.status,
                    })
                })
                .collect();

            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&json!({
                    "success": true,
                    "seed": result.seed,
                    "total_pages": result.total_pages,
                    "max_depth_reached": result.max_depth_reached,
                    "elapsed_ms": result.elapsed_ms,
                    "pages": pages,
                }))
                .unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Crawl failed: {e}"
        ))])),
    }
}
