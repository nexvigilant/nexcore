//! Perplexity AI tools: Search-grounded research with citations
//!
//! Provides access to the Perplexity Sonar API for three research use cases:
//! - General web research
//! - Competitive intelligence (domain-filtered)
//! - Regulatory intelligence (FDA/EMA/ICH/WHO filtered)
//!
//! ## Primitive Grounding
//!
//! Dominant: μ (Mapping) — query maps to search-grounded response with citations.

use crate::params::{
    PerplexityCompetitiveParams, PerplexityRegulatoryParams, PerplexityResearchParams,
    PerplexitySearchParams,
};
use nexcore_perplexity::client::PerplexityClient;
use nexcore_perplexity::research::{
    ResearchUseCase, research_competitive, research_general, research_regulatory,
};
use nexcore_perplexity::types::{ChatRequest, Model, SearchRecency};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

// ============================================================================
// Helper Functions
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
// Tool Implementations
// ============================================================================

/// Search Perplexity AI with optional model, recency, and domain filters.
pub async fn search(params: PerplexitySearchParams) -> Result<CallToolResult, McpError> {
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

/// High-level research routing by use case (general, competitive, regulatory).
pub async fn research(params: PerplexityResearchParams) -> Result<CallToolResult, McpError> {
    let client = match get_client() {
        Ok(c) => c,
        Err(e) => return format_error(&e),
    };

    let use_case =
        ResearchUseCase::from_str_opt(&params.use_case).unwrap_or(ResearchUseCase::General);

    let result = match use_case {
        ResearchUseCase::General => research_general(&client, &params.query, None).await,
        ResearchUseCase::Competitive => {
            // No specific competitors in this route — use general Pro model
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

/// Competitive intelligence with domain-filtered search.
pub async fn competitive(params: PerplexityCompetitiveParams) -> Result<CallToolResult, McpError> {
    let client = match get_client() {
        Ok(c) => c,
        Err(e) => return format_error(&e),
    };

    match research_competitive(&client, &params.query, &params.competitors, None).await {
        Ok(r) => format_result(r.formatted()),
        Err(e) => format_error(&format!("{e}")),
    }
}

/// Regulatory intelligence pre-filtered to FDA/EMA/ICH/WHO domains.
pub async fn regulatory(params: PerplexityRegulatoryParams) -> Result<CallToolResult, McpError> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
