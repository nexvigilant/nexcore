//! MCP Bridge Routes — REST gateway to nexcore-mcp tools
//!
//! Exposes `POST /{tool_name}` that dispatches in-process via `unified::dispatch()`.
//! Only tools on the allowlist are accessible; everything else returns 403.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::common::ApiError;
use crate::mcp_bridge;

// ── Allowlist ────────────────────────────────
// Read-only computation, FAERS search, PV signals, kellnr compute,
// guidelines, text scoring. NO gcloud, NO guardian mutators.

const ALLOWED_TOOLS: &[&str] = &[
    // Foundation
    "foundation_levenshtein",
    "foundation_levenshtein_bounded",
    "foundation_fuzzy_search",
    "foundation_sha256",
    "foundation_yaml_parse",
    "foundation_graph_topsort",
    "foundation_graph_levels",
    "foundation_fsrs_review",
    "foundation_concept_grep",
    "foundation_domain_distance",
    "foundation_spectral_overlap",
    "foundation_token_ratio",
    "foundation_flywheel_velocity",
    // PV Signal Detection
    "pv_signal_complete",
    "pv_signal_prr",
    "pv_signal_ror",
    "pv_signal_ic",
    "pv_signal_ebgm",
    "pv_chi_square",
    "pv_naranjo_quick",
    "pv_who_umc_quick",
    "pv_control_loop_tick",
    "pv_pipeline",
    // Vigilance
    "vigilance_safety_margin",
    "vigilance_risk_score",
    "vigilance_harm_types",
    "vigilance_map_to_tov",
    // FAERS
    "faers_search",
    "faers_drug_events",
    "faers_signal_check",
    "faers_disproportionality",
    "faers_compare_drugs",
    "faers_polypharmacy",
    "faers_reporter_weighted",
    "faers_outcome_conditioned",
    "faers_geographic_divergence",
    "faers_seriousness_cascade",
    "faers_signal_velocity",
    // Kellnr Compute — PK
    "kellnr_compute_pk_auc",
    "kellnr_compute_pk_steady_state",
    "kellnr_compute_pk_ionization",
    "kellnr_compute_pk_clearance",
    "kellnr_compute_pk_volume_distribution",
    "kellnr_compute_pk_michaelis_menten",
    // Kellnr Compute — Thermo
    "kellnr_compute_thermo_gibbs",
    "kellnr_compute_thermo_kd",
    "kellnr_compute_thermo_binding_affinity",
    "kellnr_compute_thermo_arrhenius",
    // Kellnr Compute — Stats
    "kellnr_compute_stats_welch_ttest",
    "kellnr_compute_stats_ols_regression",
    "kellnr_compute_stats_poisson_ci",
    "kellnr_compute_stats_bayesian_posterior",
    "kellnr_compute_stats_entropy",
    // Kellnr Compute — Graph
    "kellnr_compute_graph_betweenness",
    "kellnr_compute_graph_mutual_info",
    "kellnr_compute_graph_tarjan_scc",
    "kellnr_compute_graph_topsort",
    // Kellnr Compute — Decision Trees
    "kellnr_compute_dtree_feature_importance",
    "kellnr_compute_dtree_prune",
    "kellnr_compute_dtree_to_rules",
    // Kellnr Compute — Sequential Surveillance
    "kellnr_compute_signal_sprt",
    "kellnr_compute_signal_cusum",
    "kellnr_compute_signal_weibull_tto",
    // Kellnr Registry (read-only)
    "kellnr_search_crates",
    "kellnr_get_crate_metadata",
    "kellnr_list_crate_versions",
    "kellnr_get_version_details",
    "kellnr_list_all_crates",
    "kellnr_get_dependencies",
    "kellnr_get_dependents",
    "kellnr_health_check",
    "kellnr_registry_stats",
    // Guidelines
    "guidelines_search",
    "guidelines_get",
    "guidelines_categories",
    "guidelines_pv_all",
    "guidelines_url",
    // ICH
    "ich_lookup",
    "ich_search",
    "ich_guideline",
    "ich_stats",
    // MeSH
    "mesh_search",
    "mesh_lookup",
    "mesh_tree",
    "mesh_crossref",
    "mesh_consistency",
    "mesh_enrich_pubmed",
    // STEM (read-only computation)
    "stem_version",
    "stem_taxonomy",
    "stem_confidence_combine",
    "stem_tier_info",
    "stem_chem_balance",
    "stem_chem_fraction",
    "stem_chem_ratio",
    "stem_chem_rate",
    "stem_chem_affinity",
    "stem_phys_fma",
    "stem_phys_conservation",
    "stem_phys_period",
    "stem_phys_amplitude",
    "stem_phys_scale",
    "stem_phys_inertia",
    "stem_math_bounds_check",
    "stem_math_relation_invert",
    "stem_math_proof",
    "stem_math_identity",
    "stem_spatial_distance",
    "stem_spatial_triangle",
    "stem_spatial_neighborhood",
    "stem_spatial_dimension",
    "stem_spatial_orientation",
    // Chemistry (cross-domain transfer)
    "chemistry_threshold_rate",
    "chemistry_decay_remaining",
    "chemistry_saturation_rate",
    "chemistry_feasibility",
    "chemistry_dependency_rate",
    "chemistry_buffer_capacity",
    "chemistry_signal_absorbance",
    "chemistry_equilibrium",
    "chemistry_pv_mappings",
    "chemistry_threshold_exceeded",
    "chemistry_hill_response",
    "chemistry_nernst_potential",
    "chemistry_inhibition_rate",
    "chemistry_eyring_rate",
    "chemistry_langmuir_coverage",
    // Edit distance
    "edit_distance_compute",
    "edit_distance_similarity",
    "edit_distance_traceback",
    "edit_distance_transfer",
    "edit_distance_batch",
    // Algovigilance (read-only queries)
    "algovigil_status",
    "algovigil_triage_decay",
    "algovigil_triage_queue",
    // Regulatory primitives (read-only)
    "regulatory_primitives_extract",
    "regulatory_primitives_audit",
    "regulatory_primitives_compare",
    // Primitive validation (read-only)
    "primitive_validate",
    "primitive_cite",
    "primitive_validate_batch",
    "primitive_validation_tiers",
    // Signal detection
    "signal_detect",
    "signal_batch",
    "signal_thresholds",
    // Drift / Rate / Rank / Security / Observability
    "drift_ks_test",
    "drift_psi",
    "drift_jsd",
    "drift_detect",
    "rank_fusion_rrf",
    "rank_fusion_hybrid",
    "rank_fusion_borda",
    "observability_query",
    "observability_freshness",
    // Observatory Phase 9 (career, learning, graph layout)
    "career_transitions",
    "learning_dag_resolve",
    "graph_layout_converge",
    // Observatory personalization (detect, get, set, validate)
    "observatory_personalize_detect",
    "observatory_personalize_get",
    "observatory_personalize_set",
    "observatory_personalize_validate",
    // Academy Forge (extract IR + validate content + scaffold + schema)
    "forge_extract",
    "forge_validate",
    "forge_scaffold",
    "forge_schema",
    // Stoichiometry (balanced primitive equations, Jeopardy decode, sisters)
    "stoichiometry_encode",
    "stoichiometry_decode",
    "stoichiometry_sisters",
    "stoichiometry_mass_state",
    "stoichiometry_dictionary",
];

// ── Telemetry ─────────────────────────────────
// Per-tool call counters, keyed by tool name.  Poisoned mutexes are silently
// skipped so telemetry never blocks the happy path.

static TOOL_CALL_COUNTS: LazyLock<Mutex<HashMap<String, u64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn is_tool_allowed(name: &str) -> bool {
    ALLOWED_TOOLS.contains(&name)
}

// ── DTOs ─────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct McpCallRequest {
    /// Tool parameters (passed as JSON-RPC arguments)
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct McpToolResponse {
    /// Tool name that was called
    pub tool: String,
    /// Whether the MCP call succeeded
    pub success: bool,
    /// Result payload from the MCP tool
    pub result: serde_json::Value,
}

// ── Router ───────────────────────────────────

pub fn router() -> Router<crate::ApiState> {
    Router::new()
        .route("/{tool_name}", post(call_mcp_tool))
        .route("/stats/usage", get(get_usage_stats))
}

// ── Handler ──────────────────────────────────

/// Call an MCP tool via in-process dispatch
#[utoipa::path(
    post,
    path = "/mcp/{tool_name}",
    tag = "mcp",
    params(
        ("tool_name" = String, Path, description = "MCP tool name")
    ),
    request_body = McpCallRequest,
    responses(
        (status = 200, description = "Tool executed successfully", body = McpToolResponse),
        (status = 403, description = "Tool not in allowlist", body = ApiError),
        (status = 502, description = "MCP dispatch error", body = ApiError),
    )
)]
async fn call_mcp_tool(
    State(state): State<crate::ApiState>,
    Path(tool_name): Path<String>,
    Json(body): Json<McpCallRequest>,
) -> Result<Json<McpToolResponse>, ApiError> {
    // Allowlist check
    if !is_tool_allowed(&tool_name) {
        return Err(ApiError {
            code: "FORBIDDEN".to_string(),
            message: format!("Tool '{tool_name}' is not in the Studio allowlist"),
            details: None,
        });
    }

    // Increment per-tool call counter (best-effort; poisoned mutex is skipped)
    if let Ok(mut counts) = TOOL_CALL_COUNTS.lock() {
        *counts.entry(tool_name.clone()).or_insert(0) += 1;
    }

    // Dispatch in-process
    let result = mcp_bridge::call_tool(&tool_name, body.params, &state.mcp_server)
        .await
        .map_err(|e| ApiError {
            code: "MCP_ERROR".to_string(),
            message: format!("MCP bridge error: {e}"),
            details: None,
        })?;

    // Check if the tool itself reported failure
    let success = result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
        .and_then(|parsed| parsed.get("success").and_then(|s| s.as_bool()))
        .unwrap_or(true);

    Ok(Json(McpToolResponse {
        tool: tool_name,
        success,
        result,
    }))
}

// ── Usage stats ───────────────────────────────

/// Snapshot of per-tool call counts since process start.
#[derive(Debug, Serialize, ToSchema)]
pub struct UsageStatsResponse {
    /// Sum of all individual tool call counts
    pub total_calls: u64,
    /// Per-tool counts in alphabetical order
    pub tools: std::collections::BTreeMap<String, u64>,
}

/// Return accumulated per-tool call counters
#[utoipa::path(
    get,
    path = "/mcp/stats/usage",
    tag = "mcp",
    responses(
        (status = 200, description = "Tool usage statistics", body = UsageStatsResponse),
    )
)]
async fn get_usage_stats() -> Json<UsageStatsResponse> {
    let (total_calls, tools) = match TOOL_CALL_COUNTS.lock() {
        Ok(counts) => {
            let total_calls: u64 = counts.values().sum();
            let tools: std::collections::BTreeMap<String, u64> =
                counts.iter().map(|(k, v)| (k.clone(), *v)).collect();
            (total_calls, tools)
        }
        Err(_) => (0, std::collections::BTreeMap::new()),
    };
    Json(UsageStatsResponse { total_calls, tools })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_tools() {
        assert!(is_tool_allowed("pv_signal_complete"));
        assert!(is_tool_allowed("kellnr_compute_thermo_gibbs"));
        assert!(is_tool_allowed("faers_search"));
        assert!(is_tool_allowed("foundation_levenshtein"));
        // Academy Forge tools
        assert!(is_tool_allowed("forge_extract"));
        assert!(is_tool_allowed("forge_validate"));
        assert!(is_tool_allowed("forge_schema"));
    }

    #[test]
    fn test_blocked_tools() {
        assert!(!is_tool_allowed("gcloud_run_command"));
        assert!(!is_tool_allowed("gcloud_secrets_versions_access"));
        assert!(!is_tool_allowed("guardian_reset"));
        assert!(!is_tool_allowed("brain_session_create"));
        assert!(!is_tool_allowed("vigil_emit_event"));
    }
}
