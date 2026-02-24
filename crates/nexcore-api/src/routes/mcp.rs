//! MCP Bridge Routes — REST gateway to nexcore-mcp tools
//!
//! Exposes `POST /{tool_name}` that dispatches in-process via `unified::dispatch()`.
//! Only tools on the allowlist are accessible; everything else returns 403.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use axum::{
    Json, Router,
    extract::Path,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::common::ApiError;
use crate::mcp_bridge;
use nexcore_mcp::NexCoreMcpServer;

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
    // Epidemiology
    "epi_relative_risk",
    "epi_odds_ratio",
    "epi_attributable_risk",
    "epi_nnt_nnh",
    "epi_attributable_fraction",
    "epi_population_af",
    "epi_incidence_rate",
    "epi_prevalence",
    "epi_kaplan_meier",
    "epi_smr",
    "epi_pv_mappings",
    // Foundry
    "foundry_validate_artifact",
    "foundry_cascade_validate",
    "foundry_render_intelligence",
    "foundry_vdag_order",
    "foundry_infer",
    // TRIAL Framework
    "trial_protocol_register",
    "trial_power_analysis",
    "trial_randomize",
    "trial_blind_verify",
    "trial_interim_analyze",
    "trial_safety_check",
    "trial_endpoint_evaluate",
    "trial_multiplicity_adjust",
    "trial_adapt_decide",
    "trial_report_generate",
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
    // Chemivigilance (7-crate platform: molcore, structural-alerts, qsar, metabolite, chemivigilance)
    "chem_parse_smiles",
    "chem_descriptors",
    "chem_fingerprint",
    "chem_similarity",
    "chem_structural_alerts",
    "chem_predict_toxicity",
    "chem_predict_metabolites",
    "chem_predict_degradants",
    "chem_safety_brief",
    "chem_substructure",
    "chem_watchlist",
    "chem_alert_library",
    "chem_ring_scan",
    "chem_aromaticity",
    "chem_molecular_formula",
    // NMD Surveillance (anti-hallucination pipeline, read-only)
    "nmd_check",
    "nmd_upf_evaluate",
    "nmd_smg_process",
    "nmd_adaptive_stats",
    "nmd_thymic_status",
    "nmd_status",
    // Cloud Intelligence (read-only taxonomy, transfer, analysis)
    "cloud_primitive_composition",
    "cloud_transfer_confidence",
    "cloud_tier_classify",
    "cloud_compare_types",
    "cloud_reverse_synthesize",
    "cloud_list_types",
    "cloud_molecular_weight",
    "cloud_dominant_shift",
    "cloud_capacity_project",
    "cloud_supervisor_health",
    "cloud_reverse_transfer",
    "cloud_transfer_chain",
    "cloud_architecture_advisor",
    "cloud_anomaly_detect",
    "cloud_transfer_matrix",
    // ── NEW ADDITIONS (2026-02-23 gateway expansion) ──────────
    // Aggregate (data processing)
    "aggregate_fold",
    "aggregate_tree_fold",
    "aggregate_rank",
    "aggregate_percentile",
    "aggregate_outliers",
    // Benefit-Risk (QBRI)
    "pv_qbri_compute",
    "pv_qbri_derive",
    "pv_qbri_equation",
    // Chemistry (missing thermodynamics)
    "chemistry_first_law_closed",
    "chemistry_first_law_open",
    // Cloud (missing status)
    "cloud_infra_status",
    "cloud_infra_map",
    // Combinatorics (pure math)
    "comb_catalan",
    "comb_catalan_table",
    "comb_cycle_decomposition",
    "comb_min_transpositions",
    "comb_derangement",
    "comb_derangement_probability",
    "comb_grid_paths",
    "comb_binomial",
    "comb_multinomial",
    "comb_josephus",
    "comb_elimination_order",
    "comb_linear_extensions",
    // Compendious (text density analysis)
    "compendious_score_text",
    "compendious_compress_text",
    "compendious_compare_texts",
    "compendious_analyze_patterns",
    "compendious_get_domain_target",
    // Compliance (missing regulatory)
    "compliance_catalog_ich",
    "compliance_sec_filings",
    "compliance_sec_pharma",
    // FDA Credibility Assessment
    "fda_define_cou",
    "fda_assess_risk",
    "fda_create_plan",
    "fda_validate_evidence",
    "fda_decide_adequacy",
    "fda_calculate_score",
    "fda_metrics_summary",
    "fda_evidence_distribution",
    "fda_risk_distribution",
    "fda_drift_trend",
    "fda_rating_thresholds",
    // FDA Guidance (missing status)
    "fda_guidance_status",
    // FHIR (clinical data parsing)
    "fhir_adverse_event_to_signal",
    "fhir_batch_to_signals",
    "fhir_parse_bundle",
    "fhir_validate_resource",
    // Lex Primitiva (type taxonomy)
    "lex_primitiva_list",
    "lex_primitiva_get",
    "lex_primitiva_tier",
    "lex_primitiva_composition",
    "lex_primitiva_reverse_compose",
    "lex_primitiva_reverse_lookup",
    "lex_primitiva_molecular_weight",
    "lex_primitiva_dominant_shift",
    "lex_primitiva_state_mode",
    "lex_primitiva_audit",
    "lex_primitiva_synth",
    // Molecular (computation)
    "molecular_translate_codon",
    "molecular_translate_mrna",
    "molecular_central_dogma",
    "molecular_adme_phase",
    // Molecular Weight
    "mw_compute",
    "mw_periodic_table",
    "mw_compare",
    "mw_predict_transfer",
    // OpenFDA (read-only API queries)
    "openfda_drug_events",
    "openfda_drug_labels",
    "openfda_drug_recalls",
    "openfda_drug_ndc",
    "openfda_drugs_at_fda",
    "openfda_device_events",
    "openfda_device_recalls",
    "openfda_food_recalls",
    "openfda_food_events",
    "openfda_substances",
    "openfda_fan_out",
    // Pharma R&D (read-only taxonomy)
    "pharma_taxonomy_summary",
    "pharma_lookup_transfer",
    "pharma_transfer_matrix",
    "pharma_strongest_transfers",
    "pharma_weakest_transfers",
    "pharma_symbol_coverage",
    "pharma_pipeline_stage",
    "pharma_classify_generators",
    // Polymer (computation)
    "polymer_compose",
    "polymer_validate",
    "polymer_analyze",
    // Preemptive PV (predictive safety)
    "preemptive_reactive",
    "preemptive_gibbs",
    "preemptive_trajectory",
    "preemptive_severity",
    "preemptive_noise",
    "preemptive_predictive",
    "preemptive_evaluate",
    "preemptive_intervention",
    "preemptive_required_strength",
    "preemptive_omega_table",
    // PV (missing signal methods)
    "pv_signal_cooperative",
    "pv_signal_strength",
    // PV Axioms (KSB/regulation lookup)
    "pv_axioms_ksb_lookup",
    "pv_axioms_regulation_search",
    "pv_axioms_traceability_chain",
    "pv_axioms_domain_dashboard",
    "pv_axioms_query",
    // PV DSL (domain-specific language)
    "pvdsl_compile",
    "pvdsl_execute",
    "pvdsl_eval",
    "pvdsl_functions",
    // PV Embeddings (semantic similarity)
    "pv_embedding_similarity",
    "pv_embedding_get",
    "pv_embedding_stats",
    // Reason (inference)
    "reason_infer",
    "reason_counterfactual",
    // Retrocasting (retrospective analysis)
    "retro_structural_similarity",
    "retro_signal_significance",
    "retro_cluster_signals",
    "retro_correlate_alerts",
    "retro_extract_features",
    "retro_dataset_stats",
    // Security Posture (compliance scoring)
    "security_posture_assess",
    "security_threat_readiness",
    "security_compliance_gap",
    // Signal Pipeline (full compute)
    "pipeline_compute_all",
    "pipeline_batch_compute",
    "pipeline_detect",
    "pipeline_validate",
    "pipeline_thresholds",
    "pipeline_report",
    "pipeline_relay_chain",
    "pipeline_transfer",
    "pipeline_primitives",
    // Signal Theory (axiomatic)
    "signal_theory_axioms",
    "signal_theory_theorems",
    "signal_theory_detect",
    "signal_theory_decision_matrix",
    "signal_theory_conservation_check",
    "signal_theory_pipeline",
    "signal_theory_cascade",
    "signal_theory_parallel",
    // Stoichiometry (missing)
    "stoichiometry_is_balanced",
    "stoichiometry_prove",
    "stoichiometry_is_isomer",
    // Topology (graph computation)
    "topo_vietoris_rips",
    "topo_persistence",
    "topo_betti",
    "graph_centrality",
    "graph_components",
    "graph_shortest_path",
    // ToV Grounded (safety analysis)
    "tov_grounded_signal_strength",
    "tov_grounded_safety_margin",
    "tov_grounded_stability_shell",
    "tov_grounded_harm_type",
    "tov_grounded_meta_vigilance",
    "tov_grounded_eka_intelligence",
    "tov_grounded_magic_numbers",
    // Value Mining (economic signal detection)
    "value_signal_types",
    "value_signal_detect",
    "value_baseline_create",
    "value_pv_mapping",
    // Visualization (Observatory — 31 tools)
    "viz_stem_taxonomy",
    "viz_type_composition",
    "viz_method_loop",
    "viz_confidence_chain",
    "viz_bounds",
    "viz_dag",
    "viz_molecular_info",
    "viz_surface_mesh",
    "viz_spectral_analysis",
    "viz_community_detect",
    "viz_centrality",
    "viz_vdag_overlay",
    "viz_antibody_structure",
    "viz_interaction_map",
    "viz_projection",
    "viz_protein_structure",
    "viz_topology_analysis",
    "viz_dynamics_step",
    "viz_force_field_energy",
    "viz_gpu_layout",
    "viz_hypergraph",
    "viz_lod_select",
    "viz_minimize_energy",
    "viz_particle_preset",
    "viz_ae_overlay",
    "viz_coord_gen",
    "viz_bipartite_layout",
    "viz_manifold_sample",
    "viz_string_modes",
    "viz_render_pipeline",
    "viz_orbital_density",
    // Wolfram (external computation)
    "wolfram_query",
    "wolfram_short_answer",
    "wolfram_spoken_answer",
    "wolfram_calculate",
    "wolfram_step_by_step",
    "wolfram_plot",
    "wolfram_convert",
    "wolfram_chemistry",
    "wolfram_physics",
    "wolfram_astronomy",
    "wolfram_statistics",
    "wolfram_data_lookup",
    "wolfram_query_with_assumption",
    "wolfram_query_filtered",
    "wolfram_image_result",
    "wolfram_datetime",
    "wolfram_nutrition",
    "wolfram_finance",
    "wolfram_linguistics",
    // Measure (quality metrics)
    "measure_crate",
    "measure_workspace",
    "measure_entropy",
    "measure_graph",
    "measure_drift",
    "measure_compare",
    "measure_stats",
    "quality_gradient",
    // Visual (shape/color analysis)
    "visual_shape_classify",
    "visual_color_analyze",
    "visual_shape_list",
    // Zeta (pure math)
    "zeta_compute",
    "zeta_find_zeros",
    "zeta_verify_rh",
    "zeta_embedded_zeros",
    "zeta_lmfdb_parse",
    "zeta_telescope_run",
    "zeta_batch_run",
    "zeta_scaling_fit",
    "zeta_scaling_predict",
    "zeta_cayley",
    "zeta_operator_hunt",
    "zeta_operator_candidate",
    "zeta_gue_compare",
    // Docs (read-only Claude docs)
    "docs_claude_list_pages",
    "docs_claude_get_page",
    "docs_claude_search_docs",
    "docs_claude_get_docs_index",
    // Telemetry Intelligence (read-only)
    "telemetry_sources_list",
    "telemetry_source_analyze",
    "telemetry_governance_crossref",
    "telemetry_snapshot_evolution",
    "telemetry_intel_report",
    "telemetry_recent",
    // Watchtower (read-only session analysis)
    "watchtower_sessions_list",
    "watchtower_active_sessions",
    "watchtower_analyze",
    "watchtower_telemetry_stats",
    "watchtower_recent",
    "watchtower_symbol_audit",
    "watchtower_unified",
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
    let server = NexCoreMcpServer::new();
    let result = mcp_bridge::call_tool(&tool_name, body.params, &server)
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
    fn test_new_allowed_tools() {
        // 2026-02-23 expansion: spot-check each new domain
        assert!(is_tool_allowed("pv_qbri_compute"));
        assert!(is_tool_allowed("openfda_drug_events"));
        assert!(is_tool_allowed("viz_centrality"));
        assert!(is_tool_allowed("topo_betti"));
        assert!(is_tool_allowed("wolfram_calculate"));
        assert!(is_tool_allowed("pipeline_compute_all"));
        assert!(is_tool_allowed("signal_theory_axioms"));
        assert!(is_tool_allowed("preemptive_predictive"));
        assert!(is_tool_allowed("fda_calculate_score"));
        assert!(is_tool_allowed("fhir_parse_bundle"));
        assert!(is_tool_allowed("lex_primitiva_list"));
        assert!(is_tool_allowed("comb_binomial"));
        assert!(is_tool_allowed("zeta_compute"));
        assert!(is_tool_allowed("retro_cluster_signals"));
        assert!(is_tool_allowed("pv_axioms_ksb_lookup"));
        assert!(is_tool_allowed("pvdsl_compile"));
        assert!(is_tool_allowed("tov_grounded_harm_type"));
        assert!(is_tool_allowed("watchtower_analyze"));
        assert!(is_tool_allowed("telemetry_sources_list"));
        assert!(is_tool_allowed("chemistry_first_law_closed"));
    }

    #[test]
    fn test_blocked_tools() {
        assert!(!is_tool_allowed("gcloud_run_command"));
        assert!(!is_tool_allowed("gcloud_secrets_versions_access"));
        assert!(!is_tool_allowed("guardian_reset"));
        assert!(!is_tool_allowed("brain_session_create"));
        assert!(!is_tool_allowed("vigil_emit_event"));
        // Mutation/admin tools remain blocked
        assert!(!is_tool_allowed("cytokine_emit"));
        assert!(!is_tool_allowed("user_create"));
        assert!(!is_tool_allowed("claude_fs_write"));
        assert!(!is_tool_allowed("mcp_lock"));
        assert!(!is_tool_allowed("faers_etl_run"));
    }

    #[test]
    fn test_allowlist_size() {
        // Verify expansion: was 265, now ~475+
        assert!(
            ALLOWED_TOOLS.len() > 450,
            "Expected 450+ allowed tools, got {}",
            ALLOWED_TOOLS.len()
        );
    }
}
