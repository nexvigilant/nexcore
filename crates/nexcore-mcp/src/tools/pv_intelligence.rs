//! PV Intelligence MCP tools — competitive pharmacovigilance analysis.
//!
//! Six read-only tools backed by `nexcore_pv_intelligence::build_top10_graph()`:
//! - `pv_intelligence_head_to_head`: Drug vs drug safety signal comparison
//! - `pv_intelligence_class_effects`: Class-wide adverse event detection
//! - `pv_intelligence_safety_gaps`: Unlabelled / elevated signal gaps for a disease
//! - `pv_intelligence_safest_company`: Company safety ranking for a disease
//! - `pv_intelligence_therapeutic_landscape`: Competitive landscape for a therapeutic area
//! - `pv_intelligence_pipeline_overlap`: Companies competing in the same indication

use crate::params::{
    PvIntelligenceClassEffectsParams, PvIntelligenceHeadToHeadParams,
    PvIntelligencePipelineOverlapParams, PvIntelligenceSafestCompanyParams,
    PvIntelligenceSafetyGapsParams, PvIntelligenceTherapeuticLandscapeParams,
};
use nexcore_pv_intelligence::build_top10_graph;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

// ---------------------------------------------------------------------------
// pv_intelligence_head_to_head
// ---------------------------------------------------------------------------

/// Head-to-head safety comparison between two drugs.
///
/// Compares shared adverse event signals (via ClassEffect edges), drug-unique
/// events, and returns an overall safety advantage where one exists.
pub fn pv_intelligence_head_to_head(
    params: PvIntelligenceHeadToHeadParams,
) -> Result<CallToolResult, McpError> {
    let graph = build_top10_graph();
    let result = graph.head_to_head(&params.drug_a, &params.drug_b);

    let json = serde_json::to_string_pretty(&result)
        .map_err(|e| McpError::internal_error(format!("Serialization failed: {e}"), None))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

// ---------------------------------------------------------------------------
// pv_intelligence_class_effects
// ---------------------------------------------------------------------------

/// Find adverse event signals shared across a drug class.
///
/// Scans `ClassEffect` edges for the given `drug_class` label and returns
/// one result per distinct adverse event, sorted by number of affected drugs.
pub fn pv_intelligence_class_effects(
    params: PvIntelligenceClassEffectsParams,
) -> Result<CallToolResult, McpError> {
    let graph = build_top10_graph();
    let results = graph.class_effects(&params.drug_class);

    let json = serde_json::to_string_pretty(&results)
        .map_err(|e| McpError::internal_error(format!("Serialization failed: {e}"), None))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

// ---------------------------------------------------------------------------
// pv_intelligence_safety_gaps
// ---------------------------------------------------------------------------

/// Find off-label or weak signals exploitable by a better molecule.
///
/// Returns drugs treating `disease_id` with `signal_count > 0` and
/// `strongest_prr > 2.0`, sorted descending by PRR.
pub fn pv_intelligence_safety_gaps(
    params: PvIntelligenceSafetyGapsParams,
) -> Result<CallToolResult, McpError> {
    let graph = build_top10_graph();
    let results = graph.safety_gaps(&params.disease_id);

    let json = serde_json::to_string_pretty(&results)
        .map_err(|e| McpError::internal_error(format!("Serialization failed: {e}"), None))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

// ---------------------------------------------------------------------------
// pv_intelligence_safest_company
// ---------------------------------------------------------------------------

/// Rank companies by safety portfolio for a disease.
///
/// Only companies that own at least one drug treating `disease_id` are
/// included. Rank 1 is the safest (lowest average PRR).
pub fn pv_intelligence_safest_company(
    params: PvIntelligenceSafestCompanyParams,
) -> Result<CallToolResult, McpError> {
    let graph = build_top10_graph();
    let rankings = graph.safest_company_for_disease(&params.disease_id);

    let json = serde_json::to_string_pretty(&rankings)
        .map_err(|e| McpError::internal_error(format!("Serialization failed: {e}"), None))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

// ---------------------------------------------------------------------------
// pv_intelligence_therapeutic_landscape
// ---------------------------------------------------------------------------

/// Map the competitive landscape for a therapeutic area.
///
/// Returns all companies with drugs in the area, ranked by safety, along
/// with aggregate signal totals and the dominant company (most drugs).
pub fn pv_intelligence_therapeutic_landscape(
    params: PvIntelligenceTherapeuticLandscapeParams,
) -> Result<CallToolResult, McpError> {
    let graph = build_top10_graph();
    let landscape = graph.therapeutic_landscape(&params.therapeutic_area);

    let json = serde_json::to_string_pretty(&landscape)
        .map_err(|e| McpError::internal_error(format!("Serialization failed: {e}"), None))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

// ---------------------------------------------------------------------------
// pv_intelligence_pipeline_overlap
// ---------------------------------------------------------------------------

/// Find companies competing in the same indication.
///
/// Returns one `PipelineOverlap` per pair of companies that both own drugs
/// treating `disease_id`. `competition_phase` is `"head-to-head"` when both
/// companies have signal data, `"emerging"` otherwise.
pub fn pv_intelligence_pipeline_overlap(
    params: PvIntelligencePipelineOverlapParams,
) -> Result<CallToolResult, McpError> {
    let graph = build_top10_graph();
    let overlaps = graph.pipeline_overlap(&params.disease_id);

    let json = serde_json::to_string_pretty(&overlaps)
        .map_err(|e| McpError::internal_error(format!("Serialization failed: {e}"), None))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}
