//! Knowledge tools: KSB article access, search, statistics
//!
//! Access to 628 PV articles across 15 domains.

use std::path::Path;

use crate::params::{
    KsbGetParams as KnowledgeGetParams, KsbSearchParams as KnowledgeSearchParams,
    KsbStatsParams as KnowledgeStatsParams,
};
use nexcore_knowledge::{KsbIndex, default_ksb_path, search_articles};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Get a specific article by ID
pub fn get(params: KnowledgeGetParams) -> Result<CallToolResult, McpError> {
    let ksb_path = match default_ksb_path() {
        Some(p) => p,
        None => {
            let json = json!({
                "error": "KSB path not found",
                "hint": "Set NEXCORE_KSB_PATH or create ~/nexcore/knowledge/pv-ksb/",
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };

    let index = match KsbIndex::scan(&ksb_path) {
        Ok(idx) => idx,
        Err(e) => {
            let json = json!({
                "error": format!("Failed to scan KSB: {e}"),
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };

    match index.get(&params.article_id) {
        Some(article) => {
            let json = json!({
                "id": article.id,
                "domain": format!("{:?}", article.domain),
                "title": article.title,
                "content": article.content,
                "proficiency_level": article.proficiency_level.as_ref().map(|p| format!("{p:?}")),
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        None => {
            let json = json!({
                "error": format!("Article not found: {}", params.article_id),
                "hint": "Use knowledge_search to find articles",
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Search articles by query
pub fn search(params: KnowledgeSearchParams) -> Result<CallToolResult, McpError> {
    let ksb_path = match default_ksb_path() {
        Some(p) => p,
        None => {
            let json = json!({
                "error": "KSB path not found",
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };

    let index = match KsbIndex::scan(&ksb_path) {
        Ok(idx) => idx,
        Err(e) => {
            let json = json!({
                "error": format!("Failed to scan KSB: {e}"),
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };

    let limit = params.limit.unwrap_or(10);
    let domain_filter = params
        .domain
        .as_deref()
        .and_then(nexcore_knowledge::KsbDomain::from_str);
    let results = search_articles(&index, &params.query, domain_filter, limit);

    let articles: Vec<_> = results
        .iter()
        .map(|r| {
            json!({
                "id": r.id,
                "domain": r.domain,
                "title": r.title,
                "score": r.score,
                "snippet": r.description,
            })
        })
        .collect();

    let json = json!({
        "query": params.query,
        "domain_filter": params.domain,
        "results": articles,
        "count": articles.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Get KSB statistics
pub fn stats(_params: KnowledgeStatsParams) -> Result<CallToolResult, McpError> {
    let ksb_path = match default_ksb_path() {
        Some(p) => p,
        None => {
            let json = json!({
                "error": "KSB path not found",
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };

    let index = match KsbIndex::scan(&ksb_path) {
        Ok(idx) => idx,
        Err(e) => {
            let json = json!({
                "error": format!("Failed to scan KSB: {e}"),
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };

    let domain_counts = index.domain_counts();
    let domains: Vec<_> = domain_counts
        .iter()
        .map(|(domain, count)| {
            json!({
                "domain": format!("{:?}", domain),
                "count": count,
            })
        })
        .collect();

    let json = json!({
        "total_articles": index.len(),
        "domains": domains,
        "domain_count": domains.len(),
        "path": ksb_path.display().to_string(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}
