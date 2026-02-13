//! Assist tool: intent-based skill knowledge search.
//!
//! Searches the pre-populated `SkillKnowledgeIndex` and returns
//! ranked results with guidance sections and MCP tool references.

use std::sync::Arc;

use crate::params::AssistParams;
use nexcore_vigilance::skills::{SkillKnowledgeIndex, search_skills};
use parking_lot::RwLock;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Search skill knowledge index by intent.
///
/// Returns ranked results with relevant sections and MCP tools for chaining.
pub fn search(
    index: &Arc<RwLock<SkillKnowledgeIndex>>,
    params: AssistParams,
) -> Result<CallToolResult, McpError> {
    let idx = index.read();

    if idx.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({
                "error": "Skill knowledge index is empty",
                "hint": "Index is populated at server startup from ~/.claude/skills/",
            })
            .to_string(),
        )]));
    }

    let tag_filter = params.context.as_deref();
    let limit = if params.limit == 0 { 5 } else { params.limit };
    let results = search_skills(&idx, &params.intent, tag_filter, limit);

    let result_json: Vec<_> = results.iter().map(format_result).collect();

    let response = json!({
        "query": params.intent,
        "results": result_json,
        "count": result_json.len(),
        "index_size": idx.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Format a single search result as JSON value.
fn format_result(result: &nexcore_vigilance::skills::SkillSearchResult) -> serde_json::Value {
    json!({
        "name": result.name,
        "intent": result.intent,
        "score": result.score,
        "matches": result.matches,
        "mcp_tools": result.mcp_tools,
        "relevant_section": result.relevant_section,
        "related_skills": result.related_skills,
    })
}
