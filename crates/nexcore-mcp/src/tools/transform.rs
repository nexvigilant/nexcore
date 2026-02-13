//! Text transformation MCP tools (5 tools).
//!
//! Exposes the nexcore-transform crate's pipeline as MCP tools:
//! list profiles, get profile, segment text, compile plan, score fidelity.

use crate::params::{
    TransformCompilePlanParams, TransformGetProfileParams, TransformScoreFidelityParams,
    TransformSegmentParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

use nexcore_transform::prelude::*;

/// List all available domain profiles.
pub fn transform_list_profiles() -> Result<CallToolResult, McpError> {
    let registry = DomainProfileRegistry::new();
    let profiles: Vec<serde_json::Value> = registry
        .list()
        .iter()
        .map(|name| {
            let profile = registry.get(name);
            match profile {
                Some(p) => {
                    let source_domains = p.known_source_domains();
                    json!({
                        "name": p.name,
                        "display_name": p.display_name,
                        "description": p.description,
                        "vocabulary_count": p.vocabulary.len(),
                        "bridge_count": p.bridges.len(),
                        "known_source_domains": source_domains,
                    })
                }
                None => json!({ "name": name }),
            }
        })
        .collect();

    let result = json!({
        "total": profiles.len(),
        "profiles": profiles,
        "note": "Use transform_get_profile for full details. Custom profiles can be registered at runtime.",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Get detailed information about a specific domain profile.
pub fn transform_get_profile(
    params: TransformGetProfileParams,
) -> Result<CallToolResult, McpError> {
    let registry = DomainProfileRegistry::new();
    let profile = registry.get(&params.name).ok_or_else(|| {
        McpError::invalid_params(
            format!(
                "Profile '{}' not found. Available: {:?}",
                params.name,
                registry.list()
            ),
            None,
        )
    })?;

    let bridges: Vec<serde_json::Value> = profile
        .bridges
        .iter()
        .map(|b| {
            let mut entry = json!({
                "generic": b.generic,
                "specific": b.specific,
                "confidence": b.confidence,
            });
            if let Some(ref sd) = b.source_domain {
                entry
                    .as_object_mut()
                    .map(|o| o.insert("source_domain".to_string(), json!(sd)));
            }
            entry
        })
        .collect();

    let source_domains = profile.known_source_domains();

    let result = json!({
        "name": profile.name,
        "display_name": profile.display_name,
        "description": profile.description,
        "vocabulary": profile.vocabulary,
        "vocabulary_count": profile.vocabulary.len(),
        "bridges": bridges,
        "bridge_count": bridges.len(),
        "universal_bridge_count": profile.bridges.iter().filter(|b| b.source_domain.is_none()).count(),
        "source_specific_bridge_count": profile.bridges.iter().filter(|b| b.source_domain.is_some()).count(),
        "known_source_domains": source_domains,
        "rhetorical_notes": profile.rhetorical_notes,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Segment raw text into indexed paragraphs.
pub fn transform_segment(params: TransformSegmentParams) -> Result<CallToolResult, McpError> {
    let source = segment(&params.title, &params.text);

    let paragraphs: Vec<serde_json::Value> = source
        .paragraphs
        .iter()
        .map(|p| {
            json!({
                "index": p.index,
                "word_count": p.word_count,
                "preview": if p.text.len() > 100 {
                    format!("{}...", &p.text[..97])
                } else {
                    p.text.clone()
                },
            })
        })
        .collect();

    let result = json!({
        "title": source.title,
        "paragraph_count": source.paragraphs.len(),
        "total_words": source.total_words,
        "paragraphs": paragraphs,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Compile a full transformation plan.
pub fn transform_compile_plan(
    params: TransformCompilePlanParams,
) -> Result<CallToolResult, McpError> {
    let registry = DomainProfileRegistry::new();
    let profile = registry.get(&params.target_profile).ok_or_else(|| {
        McpError::invalid_params(
            format!(
                "Profile '{}' not found. Available: {:?}",
                params.target_profile,
                registry.list()
            ),
            None,
        )
    })?;

    let plan = compile_plan(&params.title, &params.text, &params.source_domain, profile);

    let instructions: Vec<serde_json::Value> = plan
        .instructions
        .iter()
        .map(|inst| {
            json!({
                "paragraph_index": inst.paragraph_index,
                "rhetorical_role": format!("{:?}", inst.rhetorical_role),
                "replacements": inst.replacements.iter().map(|(s, t)| {
                    json!({ "source": s, "target": t })
                }).collect::<Vec<_>>(),
                "guidance": inst.guidance,
                "original_word_count": inst.original_word_count,
                "original_preview": if inst.original_text.len() > 120 {
                    format!("{}...", &inst.original_text[..117])
                } else {
                    inst.original_text.clone()
                },
            })
        })
        .collect();

    // Build ledger
    let ledger = build_ledger(&plan.mapping_table);

    let result = json!({
        "plan_id": plan.plan_id,
        "source_domain": plan.source_domain,
        "target_profile": plan.target_profile,
        "paragraph_count": plan.source.paragraphs.len(),
        "total_words": plan.source.total_words,
        "total_replacements": plan.total_replacements(),
        "passthrough_paragraphs": plan.passthrough_count(),
        "unmapped_concepts": plan.mapping_table.unmapped_count,
        "aggregate_confidence": plan.mapping_table.aggregate_confidence,
        "instructions": instructions,
        "ledger": {
            "total": ledger.summary.total,
            "bridged": ledger.summary.bridged,
            "llm_assisted": ledger.summary.llm_assisted,
            "unmapped": ledger.summary.unmapped,
            "entries": ledger.entries.iter().map(|e| {
                json!({
                    "source": e.source,
                    "target": e.target,
                    "confidence": e.confidence,
                    "method": e.method,
                })
            }).collect::<Vec<_>>(),
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Score the fidelity of a transformation output.
pub fn transform_score_fidelity(
    params: TransformScoreFidelityParams,
) -> Result<CallToolResult, McpError> {
    let registry = DomainProfileRegistry::new();
    let profile = registry.get(&params.target_profile).ok_or_else(|| {
        McpError::invalid_params(
            format!(
                "Profile '{}' not found. Available: {:?}",
                params.target_profile,
                registry.list()
            ),
            None,
        )
    })?;

    let plan = compile_plan(&params.title, &params.text, &params.source_domain, profile);
    let report = score_fidelity(&plan, params.output_paragraph_count, &params.concept_hits);

    let grade = if report.fidelity_score >= 0.85 {
        "Excellent"
    } else if report.fidelity_score >= 0.70 {
        "Good"
    } else if report.fidelity_score >= 0.50 {
        "Acceptable"
    } else {
        "Poor"
    };

    let result = json!({
        "plan_id": report.plan_id,
        "source_domain": params.source_domain,
        "target_profile": params.target_profile,
        "source_paragraphs": report.source_paragraphs,
        "output_paragraphs": report.output_paragraphs,
        "paragraph_match": report.paragraph_match,
        "mean_coverage": report.mean_coverage,
        "aggregate_confidence": report.aggregate_confidence,
        "fidelity_score": report.fidelity_score,
        "grade": grade,
        "coverage_per_paragraph": report.coverage_per_paragraph,
        "formula": "score = paragraph_match * 0.3 + mean_coverage * 0.4 + aggregate_confidence * 0.3",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
