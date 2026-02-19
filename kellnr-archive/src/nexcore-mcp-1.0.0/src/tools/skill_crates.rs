//! MCP tools for Rust skill crates

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Execute primitive extractor skill via MCP
pub fn skill_extract_primitives(input: &str) -> Result<CallToolResult, McpError> {
    use skills::primitive_extractor::PrimitiveExtractor;

    let extractor = PrimitiveExtractor::new();
    let primitives = extractor.extract(input);

    let rows: Vec<_> = primitives
        .iter()
        .map(|p| {
            json!({
                "term": p.term,
                "tier": format!("{:?}", p.tier),
                "grounding": p.grounding,
                "confidence": p.transfer_confidence
            })
        })
        .collect();

    let output = json!({
        "primitives": rows,
        "count": primitives.len()
    });

    Ok(CallToolResult::success(vec![Content::text(
        output.to_string(),
    )]))
}

/// Compute transfer confidence via MCP
pub fn skill_transfer_confidence(
    structural: f64,
    functional: f64,
    contextual: f64,
) -> Result<CallToolResult, McpError> {
    use skills::transfer_confidence::compute_confidence;

    let result = compute_confidence(structural, functional, contextual);

    let output = json!({
        "confidence": result.confidence,
        "tier": format!("{:?}", result.tier),
        "components": {
            "structural": result.structural,
            "functional": result.functional,
            "contextual": result.contextual
        },
        "formula": "confidence = (S × 0.4) + (F × 0.4) + (C × 0.2)"
    });

    Ok(CallToolResult::success(vec![Content::text(
        output.to_string(),
    )]))
}

/// List available skill crates
pub fn skill_crates_list() -> Result<CallToolResult, McpError> {
    let skills = vec![
        json!({
            "name": "primitive-extractor",
            "version": "0.1.0",
            "description": "Extract T1/T2/T3 primitives from text",
            "crate": "skill-primitive-extractor"
        }),
        json!({
            "name": "transfer-confidence",
            "version": "0.1.0",
            "description": "Compute cross-domain transfer confidence",
            "crate": "skill-transfer-confidence"
        }),
    ];

    let output = json!({
        "skills": skills,
        "count": skills.len()
    });

    Ok(CallToolResult::success(vec![Content::text(
        output.to_string(),
    )]))
}
