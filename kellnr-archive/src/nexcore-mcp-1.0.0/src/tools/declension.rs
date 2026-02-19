//! Declension system MCP tools — Latin-inspired architectural primitives.
//!
//! ## Primitive Foundation
//! - ∂ (Boundary): Declension class partitioning
//! - ς (State): Component case role assignment
//! - μ (Mapping): Tool family inflection
//! - ∅ (Void): Pro-drop parameter elision
//! - × (Product): Multi-dimensional agreement checking

use crate::params::{
    DeclensionAgreeParams, DeclensionClassifyParams, DeclensionInflectParams,
    DeclensionProdropParams,
};
use nexcore_declension::{
    Declension, ProDropContext, agreement, case, declension, inflection, prodrop,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ---------------------------------------------------------------------------
// declension_classify — classify a crate into its declension and case
// ---------------------------------------------------------------------------

/// Classify a NexCore crate into its architectural declension (layer) and
/// grammatical case (role). Returns the full classification with constraints.
///
/// Tier: T3 (nexcore-declension × MCP integration)
pub fn declension_classify_tool(
    params: DeclensionClassifyParams,
) -> Result<CallToolResult, McpError> {
    let decl = declension::classify_crate(&params.crate_name);
    let comp_case = case::classify_crate(&params.crate_name);

    let result = json!({
        "crate": params.crate_name,
        "declension": {
            "class": format!("{:?}", decl),
            "layer": decl.layer(),
            "stem_vowel": decl.stem_vowel(),
            "allowed_deps": decl.allowed_deps().iter()
                .map(|d| format!("{:?} ({})", d, d.layer()))
                .collect::<Vec<_>>(),
            "async_permitted": decl.async_permitted(),
            "io_permitted": decl.io_permitted(),
            "grounds_to_required": decl.grounds_to_required(),
        },
        "case": {
            "role": format!("{:?}", comp_case),
            "latin_label": comp_case.latin(),
            "system_role": comp_case.role(),
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| "Failed to serialize result".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// declension_inflect — analyze tool family inflection patterns
// ---------------------------------------------------------------------------

/// Analyze a list of tool names for family groupings (μ-stacking).
/// Identifies tool families, singletons, compression ratio, and μ-power.
///
/// Tier: T3 (nexcore-declension × MCP integration)
pub fn declension_inflect_tool(
    params: DeclensionInflectParams,
) -> Result<CallToolResult, McpError> {
    let names: Vec<&str> = params.tool_names.iter().map(|s| s.as_str()).collect();
    let analysis = inflection::extract_families(&names);

    let families_json: Vec<_> = analysis
        .families
        .iter()
        .map(|f| {
            json!({
                "stem": f.stem,
                "mu_density": f.mu_density,
                "mu_power": inflection::mu_power(f),
                "inflections": f.inflections.iter().map(|i| json!({
                    "mode": i.mode,
                    "replaces": i.replaces,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();

    let result = json!({
        "total_tools": analysis.total_tools,
        "family_count": analysis.family_count,
        "avg_mu_density": analysis.avg_mu_density,
        "compression_ratio": analysis.compression_ratio,
        "families": families_json,
        "singletons": analysis.singletons,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| "Failed to serialize result".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// declension_agree — check multi-dimensional agreement between two crates
// ---------------------------------------------------------------------------

/// Check multi-dimensional agreement (declension, case, async boundary)
/// between two NexCore crates. Returns per-dimension pass/fail with reasons.
///
/// Tier: T3 (nexcore-declension × MCP integration)
pub fn declension_agree_tool(params: DeclensionAgreeParams) -> Result<CallToolResult, McpError> {
    let from_decl = declension::classify_crate(&params.from_crate);
    let from_case = case::classify_crate(&params.from_crate);
    let to_decl = declension::classify_crate(&params.to_crate);
    let to_case = case::classify_crate(&params.to_crate);

    let agreement_result = agreement::check_agreement(
        &params.from_crate,
        from_decl,
        from_case,
        &params.to_crate,
        to_decl,
        to_case,
    );

    let dimensions_json: Vec<_> = agreement_result
        .dimensions
        .iter()
        .map(|d| {
            json!({
                "dimension": format!("{:?}", d.dimension),
                "latin_analog": d.dimension.latin_analog(),
                "passes": d.passes,
                "reason": d.reason,
            })
        })
        .collect();

    let result = json!({
        "from": {
            "crate": params.from_crate,
            "declension": format!("{:?}", from_decl),
            "case": format!("{:?}", from_case),
        },
        "to": {
            "crate": params.to_crate,
            "declension": format!("{:?}", to_decl),
            "case": format!("{:?}", to_case),
        },
        "agrees": agreement_result.agrees,
        "agreement_ratio": agreement_result.agreement_ratio,
        "dimensions": dimensions_json,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| "Failed to serialize result".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// declension_prodrop — analyze pro-drop potential for a tool invocation
// ---------------------------------------------------------------------------

/// Analyze which parameters can be contextually elided (pro-dropped)
/// for a given tool invocation based on the current session context.
///
/// Tier: T3 (nexcore-declension × MCP integration)
pub fn declension_prodrop_tool(
    params: DeclensionProdropParams,
) -> Result<CallToolResult, McpError> {
    let mut context = ProDropContext::new();

    // Set up context from provided info
    if let Some(cwd) = params.cwd {
        context = ProDropContext::with_cwd(cwd);
    }
    if let Some(last_tool) = params.last_tool {
        context.record_invocation(&last_tool, &std::collections::HashMap::new());
    }

    let param_refs: Vec<&str> = params.param_names.iter().map(|s| s.as_str()).collect();
    let analysis = prodrop::analyze_prodrop(&params.tool_name, &param_refs, &context);

    let result = json!({
        "tool": analysis.tool,
        "total_params": analysis.total_params,
        "droppable": analysis.droppable,
        "elision_ratio": analysis.elision_ratio,
        "resolutions": analysis.resolutions,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| "Failed to serialize result".to_string()),
    )]))
}
