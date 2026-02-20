//! Tool Routing Engine — Deterministic tool selection and DAG execution planning.
//!
//! Dominant primitives: σ(Sequence) + →(Causality) + μ(Mapping) + ∂(Boundary)
//!
//! Provides 4 MCP tools:
//! - `tool_route`: stimulus → deterministic tool selection
//! - `tool_dag`: tool set → dependency DAG + topological execution plan
//! - `tool_deps`: single tool → dependencies and dependents
//! - `tool_chain`: named workflow → full execution plan with DAG
//!
//! All routing decisions are grounded in static metadata:
//! - Tool dependencies (what must run before)
//! - Data types produced/consumed (for automatic chaining)
//! - Highway class (I-IV SLA tiers)
//! - Routing rules (stimulus → tool mappings)

use crate::params::{ToolChainParams, ToolDagParams, ToolDepsParams, ToolRouteParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

// ============================================================================
// Static Metadata: Tool Dependency Registry
// ============================================================================

/// A tool's metadata for routing and DAG construction.
pub struct ToolMeta {
    pub name: &'static str,
    pub category: &'static str,
    pub highway_class: u8,
    /// Data types this tool produces (output schema keys)
    pub outputs: &'static [&'static str],
    /// Data types this tool requires as input
    pub inputs: &'static [&'static str],
    /// Hard dependencies: tools that MUST run before this one
    pub depends_on: &'static [&'static str],
    /// Tools that can substitute for this one
    pub alternatives: &'static [&'static str],
}

/// A routing rule: stimulus pattern → tool sequence
pub struct RoutingRule {
    /// Keywords/patterns to match against
    pub patterns: &'static [&'static str],
    /// Tools to call (ordered)
    pub tools: &'static [&'static str],
    /// Execution mode
    pub mode: &'static str,
    /// Routing confidence
    pub confidence: f64,
    /// Human-readable description
    pub description: &'static str,
}

/// A named workflow chain
pub struct WorkflowChain {
    pub name: &'static str,
    pub description: &'static str,
    /// Steps in order; each step may contain parallel tools
    pub steps: &'static [ChainStep],
}

pub struct ChainStep {
    /// Tools in this step (>1 means parallel within step)
    pub tools: &'static [&'static str],
    /// What data flows from previous step
    pub data_flow: &'static str,
    /// Mode: "sequential" or "parallel"
    pub mode: &'static str,
}

// ============================================================================
// Static Data: Tool Registry (core tools with dependency metadata)
// ============================================================================

pub fn tool_registry() -> Vec<ToolMeta> {
    vec![
        // Foundation (Class I)
        ToolMeta {
            name: "foundation_levenshtein",
            category: "foundation",
            highway_class: 1,
            outputs: &["distance", "similarity"],
            inputs: &["source_string", "target_string"],
            depends_on: &[],
            alternatives: &["edit_distance_compute", "foundation_levenshtein_bounded"],
        },
        ToolMeta {
            name: "foundation_levenshtein_bounded",
            category: "foundation",
            highway_class: 1,
            outputs: &["distance", "within_bound"],
            inputs: &["source_string", "target_string", "max_distance"],
            depends_on: &[],
            alternatives: &["foundation_levenshtein"],
        },
        ToolMeta {
            name: "foundation_fuzzy_search",
            category: "foundation",
            highway_class: 1,
            outputs: &["ranked_matches"],
            inputs: &["query", "candidates"],
            depends_on: &[],
            alternatives: &["edit_distance_batch"],
        },
        ToolMeta {
            name: "foundation_sha256",
            category: "foundation",
            highway_class: 1,
            outputs: &["hash"],
            inputs: &["input_string"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "foundation_graph_topsort",
            category: "foundation",
            highway_class: 1,
            outputs: &["sorted_nodes", "has_cycle"],
            inputs: &["edges"],
            depends_on: &[],
            alternatives: &["domain_primitives_topo_sort"],
        },
        ToolMeta {
            name: "foundation_graph_levels",
            category: "foundation",
            highway_class: 1,
            outputs: &["parallel_levels"],
            inputs: &["edges"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "foundation_concept_grep",
            category: "foundation",
            highway_class: 1,
            outputs: &["expanded_patterns"],
            inputs: &["concept"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "foundation_domain_distance",
            category: "foundation",
            highway_class: 1,
            outputs: &["distance", "confidence"],
            inputs: &["source_domain", "target_domain"],
            depends_on: &[],
            alternatives: &["edit_distance_transfer"],
        },
        // Edit Distance (Class I)
        ToolMeta {
            name: "edit_distance_compute",
            category: "edit_distance",
            highway_class: 1,
            outputs: &["distance", "algorithm"],
            inputs: &["source_string", "target_string"],
            depends_on: &[],
            alternatives: &["foundation_levenshtein"],
        },
        ToolMeta {
            name: "edit_distance_similarity",
            category: "edit_distance",
            highway_class: 1,
            outputs: &["similarity", "above_threshold"],
            inputs: &["source_string", "target_string"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "edit_distance_traceback",
            category: "edit_distance",
            highway_class: 1,
            outputs: &["operations"],
            inputs: &["source_string", "target_string"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "edit_distance_batch",
            category: "edit_distance",
            highway_class: 1,
            outputs: &["ranked_matches"],
            inputs: &["query", "candidates"],
            depends_on: &[],
            alternatives: &["foundation_fuzzy_search"],
        },
        ToolMeta {
            name: "edit_distance_transfer",
            category: "edit_distance",
            highway_class: 1,
            outputs: &["transfer_confidence"],
            inputs: &["source_domain", "target_domain"],
            depends_on: &[],
            alternatives: &["foundation_domain_distance"],
        },
        // PV Signal Detection (Class II)
        ToolMeta {
            name: "pv_signal_complete",
            category: "pv",
            highway_class: 2,
            outputs: &["prr", "ror", "ic", "ebgm", "chi_square", "signal_detected"],
            inputs: &["a", "b", "c", "d"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "pv_signal_prr",
            category: "pv",
            highway_class: 2,
            outputs: &["prr", "signal_detected"],
            inputs: &["a", "b", "c", "d"],
            depends_on: &[],
            alternatives: &["pv_signal_complete"],
        },
        ToolMeta {
            name: "pv_signal_ror",
            category: "pv",
            highway_class: 2,
            outputs: &["ror", "signal_detected"],
            inputs: &["a", "b", "c", "d"],
            depends_on: &[],
            alternatives: &["pv_signal_complete"],
        },
        ToolMeta {
            name: "pv_naranjo_quick",
            category: "pv",
            highway_class: 2,
            outputs: &["naranjo_score", "causality_level"],
            inputs: &["answers"],
            depends_on: &[],
            alternatives: &["pv_who_umc_quick"],
        },
        ToolMeta {
            name: "pv_who_umc_quick",
            category: "pv",
            highway_class: 2,
            outputs: &["who_umc_level"],
            inputs: &["answers"],
            depends_on: &[],
            alternatives: &["pv_naranjo_quick"],
        },
        ToolMeta {
            name: "pv_chi_square",
            category: "pv",
            highway_class: 2,
            outputs: &["chi_square", "significant"],
            inputs: &["a", "b", "c", "d"],
            depends_on: &[],
            alternatives: &[],
        },
        // Signal Theory (Class II)
        ToolMeta {
            name: "signal_theory_detect",
            category: "signal_theory",
            highway_class: 2,
            outputs: &["detected", "ratio", "strength"],
            inputs: &["observed", "expected"],
            depends_on: &[],
            alternatives: &["signal_detect"],
        },
        ToolMeta {
            name: "signal_theory_decision_matrix",
            category: "signal_theory",
            highway_class: 2,
            outputs: &["sensitivity", "specificity", "d_prime", "metrics"],
            inputs: &["hits", "misses", "false_alarms", "correct_rejections"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "signal_theory_pipeline",
            category: "signal_theory",
            highway_class: 2,
            outputs: &["outcome", "stages_passed"],
            inputs: &["stages", "value"],
            depends_on: &[],
            alternatives: &["signal_theory_cascade"],
        },
        ToolMeta {
            name: "signal_theory_cascade",
            category: "signal_theory",
            highway_class: 2,
            outputs: &["outcome", "stages_passed"],
            inputs: &["thresholds", "value"],
            depends_on: &[],
            alternatives: &["signal_theory_pipeline"],
        },
        ToolMeta {
            name: "signal_theory_parallel",
            category: "signal_theory",
            highway_class: 2,
            outputs: &["outcome", "detector_results"],
            inputs: &["threshold_1", "threshold_2", "value"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "signal_theory_conservation_check",
            category: "signal_theory",
            highway_class: 2,
            outputs: &["all_satisfied", "violations", "d_prime"],
            inputs: &["hits", "misses", "false_alarms", "correct_rejections"],
            depends_on: &["signal_theory_decision_matrix"],
            alternatives: &[],
        },
        // Kellnr PK (Class II — pharmacokinetic modeling)
        ToolMeta {
            name: "kellnr_compute_pk_michaelis_menten",
            category: "kellnr",
            highway_class: 2,
            outputs: &["velocity", "saturation_fraction"],
            inputs: &["substrate", "km", "vmax"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "kellnr_compute_pk_half_life",
            category: "kellnr",
            highway_class: 2,
            outputs: &["half_life", "elimination_rate"],
            inputs: &["concentration_initial", "concentration_final", "time"],
            depends_on: &[],
            alternatives: &[],
        },
        // STEM (Class II)
        ToolMeta {
            name: "stem_taxonomy",
            category: "stem",
            highway_class: 2,
            outputs: &["taxonomy", "trait_count"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "stem_confidence_combine",
            category: "stem",
            highway_class: 2,
            outputs: &["combined_confidence"],
            inputs: &["confidences"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "stem_phys_fma",
            category: "stem",
            highway_class: 2,
            outputs: &["force"],
            inputs: &["mass", "acceleration"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "stem_phys_conservation",
            category: "stem",
            highway_class: 2,
            outputs: &["conserved"],
            inputs: &["before", "after"],
            depends_on: &[],
            alternatives: &[],
        },
        // Chemistry (Class II)
        ToolMeta {
            name: "chemistry_pv_mappings",
            category: "chemistry",
            highway_class: 2,
            outputs: &["mappings"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "chemistry_threshold_rate",
            category: "chemistry",
            highway_class: 2,
            outputs: &["rate", "pv_interpretation"],
            inputs: &["pre_exponential", "activation_energy", "temperature"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "chemistry_saturation_rate",
            category: "chemistry",
            highway_class: 2,
            outputs: &["rate", "pv_interpretation"],
            inputs: &["max_rate", "half_max", "substrate"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "chemistry_feasibility",
            category: "chemistry",
            highway_class: 2,
            outputs: &["delta_g", "spontaneous"],
            inputs: &["delta_h", "temperature", "delta_s"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "chemistry_decay_remaining",
            category: "chemistry",
            highway_class: 2,
            outputs: &["remaining", "fraction_remaining", "half_lives_elapsed"],
            inputs: &["initial", "half_life", "time"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "chemistry_dependency_rate",
            category: "chemistry",
            highway_class: 2,
            outputs: &["rate", "overall_order"],
            inputs: &["k", "reactants"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "chemistry_buffer_capacity",
            category: "chemistry",
            highway_class: 2,
            outputs: &["buffer_capacity", "is_optimal_range"],
            inputs: &["total_conc", "ratio"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "chemistry_signal_absorbance",
            category: "chemistry",
            highway_class: 2,
            outputs: &["absorbance", "transmittance"],
            inputs: &["absorptivity", "path_length", "concentration"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "chemistry_equilibrium",
            category: "chemistry",
            highway_class: 2,
            outputs: &["product_fraction", "substrate_fraction"],
            inputs: &["k_eq"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "chemistry_threshold_exceeded",
            category: "chemistry",
            highway_class: 2,
            outputs: &["exceeded", "margin"],
            inputs: &["signal", "threshold"],
            depends_on: &[],
            alternatives: &["chemistry_threshold_rate"],
        },
        ToolMeta {
            name: "chemistry_hill_response",
            category: "chemistry",
            highway_class: 2,
            outputs: &["response", "cooperativity_class"],
            inputs: &["input", "k_half", "n_hill"],
            depends_on: &[],
            alternatives: &["chemistry_saturation_rate"],
        },
        ToolMeta {
            name: "chemistry_nernst_potential",
            category: "chemistry",
            highway_class: 2,
            outputs: &["potential_mv", "threshold_shift"],
            inputs: &[
                "standard_potential_mv",
                "n_electrons",
                "temperature_k",
                "reaction_quotient",
            ],
            depends_on: &[],
            alternatives: &["chemistry_threshold_rate"],
        },
        ToolMeta {
            name: "chemistry_inhibition_rate",
            category: "chemistry",
            highway_class: 2,
            outputs: &["rate", "apparent_km", "inhibition_factor"],
            inputs: &["substrate", "v_max", "k_m", "inhibitor", "k_i"],
            depends_on: &[],
            alternatives: &["chemistry_saturation_rate"],
        },
        ToolMeta {
            name: "chemistry_eyring_rate",
            category: "chemistry",
            highway_class: 2,
            outputs: &["rate_constant", "delta_g_activation_kj"],
            inputs: &[
                "delta_h_activation_kj",
                "delta_s_activation",
                "temperature_k",
            ],
            depends_on: &[],
            alternatives: &["chemistry_threshold_rate"],
        },
        ToolMeta {
            name: "chemistry_langmuir_coverage",
            category: "chemistry",
            highway_class: 2,
            outputs: &["coverage_fraction", "occupied_sites"],
            inputs: &["equilibrium_constant", "adsorbate_conc", "total_sites"],
            depends_on: &[],
            alternatives: &["chemistry_saturation_rate"],
        },
        ToolMeta {
            name: "chemistry_first_law_closed",
            category: "chemistry",
            highway_class: 2,
            outputs: &["delta_u", "u_final", "is_balanced"],
            inputs: &["u_initial", "heat_in", "work_out"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "chemistry_first_law_open",
            category: "chemistry",
            highway_class: 2,
            outputs: &["net_energy_change", "enthalpy_inflow", "enthalpy_outflow"],
            inputs: &[
                "heat_rate",
                "work_rate",
                "inflow_mass_rates",
                "inflow_enthalpies",
                "outflow_mass_rates",
                "outflow_enthalpies",
            ],
            depends_on: &[],
            alternatives: &["chemistry_first_law_closed"],
        },
        // Vigilance (Class II)
        ToolMeta {
            name: "vigilance_risk_score",
            category: "vigilance",
            highway_class: 2,
            outputs: &["risk_score", "risk_level"],
            inputs: &["drug", "event", "prr", "ror_lower", "ic025", "eb05", "n"],
            depends_on: &["pv_signal_complete"],
            alternatives: &[],
        },
        ToolMeta {
            name: "vigilance_safety_margin",
            category: "vigilance",
            highway_class: 2,
            outputs: &["safety_distance", "action", "interpretation"],
            inputs: &["prr", "ror_lower", "ic025", "eb05", "n"],
            depends_on: &["pv_signal_complete"],
            alternatives: &[],
        },
        // Epidemiology (Class II — measures of association, impact, survival)
        ToolMeta {
            name: "epi_relative_risk",
            category: "epidemiology",
            highway_class: 2,
            outputs: &["rr", "ci_lower", "ci_upper", "interpretation"],
            inputs: &["a", "b", "c", "d"],
            depends_on: &[],
            alternatives: &["pv_signal_complete"],
        },
        ToolMeta {
            name: "epi_odds_ratio",
            category: "epidemiology",
            highway_class: 2,
            outputs: &["or", "ci_lower", "ci_upper", "interpretation"],
            inputs: &["a", "b", "c", "d"],
            depends_on: &[],
            alternatives: &["pv_signal_complete"],
        },
        ToolMeta {
            name: "epi_attributable_risk",
            category: "epidemiology",
            highway_class: 2,
            outputs: &["ar", "ci_lower", "ci_upper"],
            inputs: &["a", "b", "c", "d"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "epi_nnt_nnh",
            category: "epidemiology",
            highway_class: 2,
            outputs: &["value", "metric_type", "interpretation"],
            inputs: &["a", "b", "c", "d"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "epi_attributable_fraction",
            category: "epidemiology",
            highway_class: 2,
            outputs: &["af", "interpretation"],
            inputs: &["a", "b", "c", "d"],
            depends_on: &["epi_relative_risk"],
            alternatives: &[],
        },
        ToolMeta {
            name: "epi_population_af",
            category: "epidemiology",
            highway_class: 2,
            outputs: &["paf", "exposed_proportion"],
            inputs: &["a", "b", "c", "d"],
            depends_on: &["epi_relative_risk"],
            alternatives: &[],
        },
        ToolMeta {
            name: "epi_incidence_rate",
            category: "epidemiology",
            highway_class: 2,
            outputs: &["rate", "ci_lower", "ci_upper"],
            inputs: &["events", "person_time"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "epi_prevalence",
            category: "epidemiology",
            highway_class: 2,
            outputs: &["prevalence", "ci_lower", "ci_upper"],
            inputs: &["cases", "population"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "epi_kaplan_meier",
            category: "epidemiology",
            highway_class: 2,
            outputs: &["survival_table", "median_survival"],
            inputs: &["intervals"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "epi_smr",
            category: "epidemiology",
            highway_class: 2,
            outputs: &["smr", "ci_lower", "ci_upper"],
            inputs: &["observed", "expected"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "epi_pv_mappings",
            category: "epidemiology",
            highway_class: 2,
            outputs: &["mappings", "overall_transfer_confidence"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["chemistry_pv_mappings"],
        },
        // Lex Primitiva (Class II)
        ToolMeta {
            name: "lex_primitiva_composition",
            category: "lex_primitiva",
            highway_class: 2,
            outputs: &["composition", "primitives"],
            inputs: &["concept"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "lex_primitiva_tier",
            category: "lex_primitiva",
            highway_class: 2,
            outputs: &["tier", "confidence"],
            inputs: &["concept"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "lex_primitiva_reverse_compose",
            category: "lex_primitiva",
            highway_class: 2,
            outputs: &["composed_types"],
            inputs: &["primitives"],
            depends_on: &[],
            alternatives: &[],
        },
        // Guardian (Class III)
        ToolMeta {
            name: "guardian_homeostasis_tick",
            category: "guardian",
            highway_class: 3,
            outputs: &["iteration", "signals", "actions", "threat_level"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "guardian_evaluate_pv",
            category: "guardian",
            highway_class: 3,
            outputs: &["risk_level", "recommended_actions"],
            inputs: &["signal_data"],
            depends_on: &["pv_signal_complete"],
            alternatives: &[],
        },
        ToolMeta {
            name: "guardian_status",
            category: "guardian",
            highway_class: 3,
            outputs: &["threat_level", "sensors", "actuators"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        // Brain (Class III)
        ToolMeta {
            name: "brain_artifact_save",
            category: "brain",
            highway_class: 3,
            outputs: &["artifact_id"],
            inputs: &["content"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "brain_artifact_resolve",
            category: "brain",
            highway_class: 3,
            outputs: &["resolved_version"],
            inputs: &["artifact_id"],
            depends_on: &["brain_artifact_save"],
            alternatives: &[],
        },
        ToolMeta {
            name: "implicit_get",
            category: "brain",
            highway_class: 3,
            outputs: &["value"],
            inputs: &["key"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "implicit_set",
            category: "brain",
            highway_class: 3,
            outputs: &["stored"],
            inputs: &["key", "value"],
            depends_on: &[],
            alternatives: &[],
        },
        // Grounded (Class III)
        ToolMeta {
            name: "grounded_uncertain",
            category: "grounded",
            highway_class: 3,
            outputs: &["grounded_value"],
            inputs: &["value", "confidence"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "grounded_compose",
            category: "grounded",
            highway_class: 3,
            outputs: &["composed_confidence"],
            inputs: &["grounded_values"],
            depends_on: &["grounded_uncertain"],
            alternatives: &[],
        },
        ToolMeta {
            name: "grounded_require",
            category: "grounded",
            highway_class: 3,
            outputs: &["passed", "value"],
            inputs: &["grounded_value", "min_confidence"],
            depends_on: &["grounded_uncertain"],
            alternatives: &[],
        },
        ToolMeta {
            name: "grounded_evidence_new",
            category: "grounded",
            highway_class: 3,
            outputs: &["evidence_chain_id"],
            inputs: &["hypothesis"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "grounded_evidence_step",
            category: "grounded",
            highway_class: 3,
            outputs: &["chain_state"],
            inputs: &["evidence_chain_id", "observation"],
            depends_on: &["grounded_evidence_new"],
            alternatives: &[],
        },
        ToolMeta {
            name: "grounded_evidence_get",
            category: "grounded",
            highway_class: 3,
            outputs: &["chain_summary"],
            inputs: &["evidence_chain_id"],
            depends_on: &["grounded_evidence_new"],
            alternatives: &[],
        },
        // Highway (Class III)
        ToolMeta {
            name: "highway_classify",
            category: "highway",
            highway_class: 3,
            outputs: &["class", "sla", "layer"],
            inputs: &["tool_name"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "highway_quality",
            category: "highway",
            highway_class: 3,
            outputs: &["quality_score", "grade"],
            inputs: &["tool_name"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "highway_parallel_plan",
            category: "highway",
            highway_class: 3,
            outputs: &["lanes", "execution_plan"],
            inputs: &["tools"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "highway_grade_separate",
            category: "highway",
            highway_class: 3,
            outputs: &["class_groups"],
            inputs: &["tools"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "highway_interchange",
            category: "highway",
            highway_class: 3,
            outputs: &["merged_result", "confidence"],
            inputs: &["results"],
            depends_on: &[],
            alternatives: &[],
        },
        // Validation (Class III)
        ToolMeta {
            name: "validation_run",
            category: "validation",
            highway_class: 3,
            outputs: &["validation_result", "levels_passed"],
            inputs: &["target"],
            depends_on: &[],
            alternatives: &["validation_check"],
        },
        ToolMeta {
            name: "validation_check",
            category: "validation",
            highway_class: 3,
            outputs: &["quick_result"],
            inputs: &["target"],
            depends_on: &[],
            alternatives: &["validation_run"],
        },
        ToolMeta {
            name: "primitive_validate",
            category: "primitive_validation",
            highway_class: 3,
            outputs: &["validated", "sources"],
            inputs: &["term"],
            depends_on: &[],
            alternatives: &[],
        },
        // FAERS (Class IV)
        ToolMeta {
            name: "faers_search",
            category: "faers",
            highway_class: 4,
            outputs: &["cases", "count"],
            inputs: &["query"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "faers_drug_events",
            category: "faers",
            highway_class: 4,
            outputs: &["events", "a_b_c_d"],
            inputs: &["drug_name"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "faers_signal_check",
            category: "faers",
            highway_class: 4,
            outputs: &["signal_detected", "metrics"],
            inputs: &["drug", "event"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "faers_disproportionality",
            category: "faers",
            highway_class: 4,
            outputs: &["prr", "ror", "chi_square"],
            inputs: &["drug", "event"],
            depends_on: &[],
            alternatives: &[],
        },
        // Guidelines (Class IV)
        ToolMeta {
            name: "guidelines_search",
            category: "guidelines",
            highway_class: 4,
            outputs: &["guidelines"],
            inputs: &["query"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "ich_lookup",
            category: "guidelines",
            highway_class: 4,
            outputs: &["term_definition"],
            inputs: &["term"],
            depends_on: &[],
            alternatives: &[],
        },
        // Wolfram (Class IV)
        ToolMeta {
            name: "wolfram_calculate",
            category: "wolfram",
            highway_class: 4,
            outputs: &["result"],
            inputs: &["expression"],
            depends_on: &[],
            alternatives: &[],
        },
        // Perplexity (Class IV)
        ToolMeta {
            name: "perplexity_search",
            category: "perplexity",
            highway_class: 4,
            outputs: &["answer", "sources"],
            inputs: &["query"],
            depends_on: &[],
            alternatives: &[],
        },
        // Cytokine (Class III)
        ToolMeta {
            name: "cytokine_emit",
            category: "cytokine",
            highway_class: 3,
            outputs: &["emitted"],
            inputs: &["signal_type", "payload"],
            depends_on: &[],
            alternatives: &[],
        },
        // Immunity (Class III)
        ToolMeta {
            name: "immunity_propose",
            category: "immunity",
            highway_class: 3,
            outputs: &["antibody_id"],
            inputs: &["pattern", "description"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "immunity_scan",
            category: "immunity",
            highway_class: 3,
            outputs: &["threats_found"],
            inputs: &["code"],
            depends_on: &[],
            alternatives: &[],
        },
        // Synapse (Class III)
        ToolMeta {
            name: "synapse_observe",
            category: "synapse",
            highway_class: 3,
            outputs: &["amplitude", "saturation"],
            inputs: &["name", "score"],
            depends_on: &[],
            alternatives: &[],
        },
        // Principles (Class III)
        ToolMeta {
            name: "principles_search",
            category: "principles",
            highway_class: 3,
            outputs: &["principles"],
            inputs: &["keyword"],
            depends_on: &[],
            alternatives: &[],
        },
        // ====================================================================
        // Auto-registered tools (315 tools) — enriched with data-flow metadata
        // inputs/outputs: semantic data types flowing between tools
        // depends_on: tools that must run before this one (hard dependencies)
        // alternatives: tools that serve the same purpose (substitutable)
        // ====================================================================
        // algovigil (6 tools) — ICSR triage and deduplication
        ToolMeta {
            name: "algovigil_status",
            category: "algovigil",
            highway_class: 2,
            outputs: &["triage_status", "queue_depth"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "algovigil_triage_decay",
            category: "algovigil",
            highway_class: 2,
            outputs: &["decayed_score"],
            inputs: &["icsr_id", "decay_rate"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "algovigil_triage_queue",
            category: "algovigil",
            highway_class: 2,
            outputs: &["queue_items", "priority_order"],
            inputs: &["limit"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "algovigil_triage_reinforce",
            category: "algovigil",
            highway_class: 2,
            outputs: &["reinforced_score"],
            inputs: &["icsr_id", "reinforcement_signal"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "algovigil_dedup_batch",
            category: "algovigil",
            highway_class: 2,
            outputs: &["dedup_results", "duplicate_pairs"],
            inputs: &["icsr_batch"],
            depends_on: &["algovigil_dedup_pair"],
            alternatives: &[],
        },
        ToolMeta {
            name: "algovigil_dedup_pair",
            category: "algovigil",
            highway_class: 2,
            outputs: &["similarity_score", "is_duplicate"],
            inputs: &["icsr_a", "icsr_b"],
            depends_on: &[],
            alternatives: &["edit_distance_similarity"],
        },
        // brain (19 tools) — working memory, sessions, artifacts, implicit knowledge
        ToolMeta {
            name: "implicit_patterns_by_relevance",
            category: "brain",
            highway_class: 3,
            outputs: &["patterns", "relevance_scores"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["implicit_patterns_by_grounding"],
        },
        ToolMeta {
            name: "brain_session_load",
            category: "brain",
            highway_class: 3,
            outputs: &["session", "artifacts"],
            inputs: &["session_id"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "brain_recovery_check",
            category: "brain",
            highway_class: 3,
            outputs: &["recovery_status", "issues"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["brain_recovery_auto"],
        },
        ToolMeta {
            name: "brain_coordination_status",
            category: "brain",
            highway_class: 3,
            outputs: &["lock_status", "holder", "timeout_remaining"],
            inputs: &["lock_id"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "brain_artifact_diff",
            category: "brain",
            highway_class: 3,
            outputs: &["diff_lines", "diff_stats"],
            inputs: &["artifact_id", "version_a", "version_b"],
            depends_on: &["brain_artifact_get"],
            alternatives: &[],
        },
        ToolMeta {
            name: "brain_session_create",
            category: "brain",
            highway_class: 3,
            outputs: &["session_id", "created_at"],
            inputs: &["project", "description"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "brain_artifact_get",
            category: "brain",
            highway_class: 3,
            outputs: &["artifact", "versions", "current_version"],
            inputs: &["artifact_id"],
            depends_on: &[],
            alternatives: &["brain_artifact_resolve"],
        },
        ToolMeta {
            name: "brain_recovery_repair",
            category: "brain",
            highway_class: 3,
            outputs: &["repair_status", "records_fixed"],
            inputs: &["session_id"],
            depends_on: &["brain_recovery_check"],
            alternatives: &["brain_recovery_auto"],
        },
        ToolMeta {
            name: "implicit_patterns_by_grounding",
            category: "brain",
            highway_class: 3,
            outputs: &["patterns", "grounding_confidence"],
            inputs: &["primitive"],
            depends_on: &[],
            alternatives: &["implicit_patterns_by_relevance"],
        },
        ToolMeta {
            name: "brain_sessions_list",
            category: "brain",
            highway_class: 3,
            outputs: &["sessions"],
            inputs: &["limit", "offset"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "code_tracker_original",
            category: "brain",
            highway_class: 3,
            outputs: &["original_content"],
            inputs: &["file_path"],
            depends_on: &["code_tracker_track"],
            alternatives: &[],
        },
        ToolMeta {
            name: "brain_coordination_acquire",
            category: "brain",
            highway_class: 3,
            outputs: &["lock_id", "acquired"],
            inputs: &["resource_id", "timeout_ms"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "code_tracker_track",
            category: "brain",
            highway_class: 3,
            outputs: &["track_id", "initial_checksum"],
            inputs: &["file_path"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "implicit_find_corrections",
            category: "brain",
            highway_class: 3,
            outputs: &["corrections"],
            inputs: &["error_pattern"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "code_tracker_changed",
            category: "brain",
            highway_class: 3,
            outputs: &["changed", "current_checksum"],
            inputs: &["file_path"],
            depends_on: &["code_tracker_track"],
            alternatives: &[],
        },
        ToolMeta {
            name: "implicit_stats",
            category: "brain",
            highway_class: 3,
            outputs: &["pattern_count", "belief_count", "correction_count"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "brain_recovery_rebuild_index",
            category: "brain",
            highway_class: 3,
            outputs: &["index_status", "entries_reindexed"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "brain_coordination_release",
            category: "brain",
            highway_class: 3,
            outputs: &["released"],
            inputs: &["lock_id"],
            depends_on: &["brain_coordination_acquire"],
            alternatives: &[],
        },
        ToolMeta {
            name: "brain_recovery_auto",
            category: "brain",
            highway_class: 3,
            outputs: &["auto_repair_status", "issues_resolved"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["brain_recovery_check"],
        },
        // brand (4 tools)
        ToolMeta {
            name: "brand_semantic_tiers",
            category: "brand",
            highway_class: 1,
            outputs: &["tier_map"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "brand_decomposition_nexvigilant",
            category: "brand",
            highway_class: 1,
            outputs: &["brand_primitives", "semantic_layers"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["brand_decomposition_get"],
        },
        ToolMeta {
            name: "brand_primitive_test",
            category: "brand",
            highway_class: 1,
            outputs: &["test_result", "primitive_coverage"],
            inputs: &["brand_name"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "brand_decomposition_get",
            category: "brand",
            highway_class: 1,
            outputs: &["brand_primitives"],
            inputs: &["brand_name"],
            depends_on: &[],
            alternatives: &["brand_decomposition_nexvigilant"],
        },
        // cargo (6 tools) — Rust build toolchain
        ToolMeta {
            name: "cargo_clippy",
            category: "cargo",
            highway_class: 3,
            outputs: &["diagnostics", "warning_count"],
            inputs: &["crate_name"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "cargo_check",
            category: "cargo",
            highway_class: 3,
            outputs: &["diagnostics", "success"],
            inputs: &["crate_name"],
            depends_on: &[],
            alternatives: &["cargo_build"],
        },
        ToolMeta {
            name: "cargo_build",
            category: "cargo",
            highway_class: 3,
            outputs: &["binary_path", "success"],
            inputs: &["crate_name", "release"],
            depends_on: &["cargo_check"],
            alternatives: &[],
        },
        ToolMeta {
            name: "cargo_test",
            category: "cargo",
            highway_class: 3,
            outputs: &["test_results", "pass_count", "fail_count"],
            inputs: &["crate_name", "filter"],
            depends_on: &["cargo_check"],
            alternatives: &[],
        },
        ToolMeta {
            name: "cargo_tree",
            category: "cargo",
            highway_class: 3,
            outputs: &["dependency_tree"],
            inputs: &["crate_name"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "cargo_fmt",
            category: "cargo",
            highway_class: 3,
            outputs: &["formatted", "changes"],
            inputs: &["crate_name"],
            depends_on: &[],
            alternatives: &[],
        },
        // cep (6 tools) — Cognitive Evolution Pipeline
        ToolMeta {
            name: "cep_execute_stage",
            category: "cep",
            highway_class: 2,
            outputs: &["stage_result", "extracted_knowledge"],
            inputs: &["stage_id", "domain_context"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "cep_validate_extraction",
            category: "cep",
            highway_class: 2,
            outputs: &["validation_status", "confidence"],
            inputs: &["extracted_knowledge"],
            depends_on: &["cep_extract_primitives"],
            alternatives: &[],
        },
        ToolMeta {
            name: "cep_domain_translate",
            category: "cep",
            highway_class: 2,
            outputs: &["translated_concept", "transfer_confidence"],
            inputs: &["concept", "source_domain", "target_domain"],
            depends_on: &[],
            alternatives: &["stem_taxonomy"],
        },
        ToolMeta {
            name: "cep_pipeline_stages",
            category: "cep",
            highway_class: 2,
            outputs: &["stages"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "cep_extract_primitives",
            category: "cep",
            highway_class: 2,
            outputs: &["primitives", "confidence"],
            inputs: &["concept", "domain"],
            depends_on: &[],
            alternatives: &["lex_primitiva_composition"],
        },
        ToolMeta {
            name: "cep_classify_primitive",
            category: "cep",
            highway_class: 2,
            outputs: &["tier", "classification"],
            inputs: &["primitive"],
            depends_on: &[],
            alternatives: &["lex_primitiva_tier"],
        },
        // chemistry (4 tools) — lab experiment tools
        ToolMeta {
            name: "lab_batch",
            category: "chemistry",
            highway_class: 2,
            outputs: &["batch_results"],
            inputs: &["experiments"],
            depends_on: &["lab_experiment"],
            alternatives: &[],
        },
        ToolMeta {
            name: "lab_compare",
            category: "chemistry",
            highway_class: 2,
            outputs: &["comparison", "delta"],
            inputs: &["result_a", "result_b"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "lab_experiment",
            category: "chemistry",
            highway_class: 2,
            outputs: &["experiment_result", "measurements"],
            inputs: &["experiment_config"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "lab_react",
            category: "chemistry",
            highway_class: 2,
            outputs: &["reaction_products", "energy_change"],
            inputs: &["reactants", "conditions"],
            depends_on: &[],
            alternatives: &["wolfram_chemistry"],
        },
        // comm (2 tools) — communication governance
        ToolMeta {
            name: "comm_recommend_protocol",
            category: "comm",
            highway_class: 3,
            outputs: &["protocol", "implementation_guide"],
            inputs: &["use_case"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "comm_route_message",
            category: "comm",
            highway_class: 3,
            outputs: &["delivery_status", "routing_trace"],
            inputs: &["message", "recipient_type"],
            depends_on: &[],
            alternatives: &[],
        },
        // compliance (5 tools) — regulatory compliance
        ToolMeta {
            name: "compliance_catalog_ich",
            category: "compliance",
            highway_class: 2,
            outputs: &["catalog_entry"],
            inputs: &["guideline_code"],
            depends_on: &[],
            alternatives: &["ich_guideline"],
        },
        ToolMeta {
            name: "compliance_sec_pharma",
            category: "compliance",
            highway_class: 2,
            outputs: &["pharma_filings", "relevance"],
            inputs: &["drug_name"],
            depends_on: &["compliance_sec_filings"],
            alternatives: &["faers_search"],
        },
        ToolMeta {
            name: "compliance_assess",
            category: "compliance",
            highway_class: 2,
            outputs: &["compliance_status", "issues"],
            inputs: &["assessment_type", "target"],
            depends_on: &[],
            alternatives: &["validation_run"],
        },
        ToolMeta {
            name: "compliance_check_exclusion",
            category: "compliance",
            highway_class: 2,
            outputs: &["is_excluded", "reason"],
            inputs: &["entity_id", "exclusion_type"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "compliance_sec_filings",
            category: "compliance",
            highway_class: 2,
            outputs: &["filings"],
            inputs: &["company_ticker"],
            depends_on: &[],
            alternatives: &[],
        },
        // config (1 tool)
        ToolMeta {
            name: "config_validate",
            category: "config",
            highway_class: 3,
            outputs: &["validation_status", "errors"],
            inputs: &["config_path"],
            depends_on: &[],
            alternatives: &[],
        },
        // crate_dev (5 tools) — crate development and inspection
        ToolMeta {
            name: "crate_dev_scaffold",
            category: "crate_dev",
            highway_class: 3,
            outputs: &["scaffold_path", "files_created"],
            inputs: &["crate_name", "layer"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "crate_xray_trial",
            category: "crate_dev",
            highway_class: 3,
            outputs: &["trial_report", "test_phases"],
            inputs: &["crate_name"],
            depends_on: &["crate_xray"],
            alternatives: &[],
        },
        ToolMeta {
            name: "crate_xray_goals",
            category: "crate_dev",
            highway_class: 3,
            outputs: &["goals", "progress"],
            inputs: &["crate_name"],
            depends_on: &["crate_xray"],
            alternatives: &[],
        },
        ToolMeta {
            name: "crate_xray",
            category: "crate_dev",
            highway_class: 3,
            outputs: &["crate_analysis", "structure", "dependencies"],
            inputs: &["crate_name"],
            depends_on: &[],
            alternatives: &["measure_crate"],
        },
        ToolMeta {
            name: "crate_dev_audit",
            category: "crate_dev",
            highway_class: 3,
            outputs: &["audit_report", "issues"],
            inputs: &["crate_name"],
            depends_on: &[],
            alternatives: &["validation_run"],
        },
        // cytokine (2 tools) — inter-crate signaling
        ToolMeta {
            name: "cytokine_families",
            category: "cytokine",
            highway_class: 2,
            outputs: &["families"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "cytokine_status",
            category: "cytokine",
            highway_class: 2,
            outputs: &["active_signals", "pending_count"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        // dhs (1 tool)
        ToolMeta {
            name: "dhs_verify_boundary",
            category: "dhs",
            highway_class: 3,
            outputs: &["boundary_secure", "threats"],
            inputs: &["boundary_params"],
            depends_on: &[],
            alternatives: &["guardian_status"],
        },
        // docs (1 tool)
        ToolMeta {
            name: "docs_generate_claude_md",
            category: "docs",
            highway_class: 3,
            outputs: &["claude_md_content"],
            inputs: &["project_path"],
            depends_on: &[],
            alternatives: &[],
        },
        // dot (2 tools) — logistics dispatch
        ToolMeta {
            name: "dot_dispatch_manifest",
            category: "dot",
            highway_class: 3,
            outputs: &["dispatch_id", "routing_confirmed"],
            inputs: &["shipment_manifest"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "dot_verify_highway",
            category: "dot",
            highway_class: 3,
            outputs: &["highway_status", "conditions"],
            inputs: &["route"],
            depends_on: &[],
            alternatives: &[],
        },
        // dtree (6 tools) — decision tree operations
        ToolMeta {
            name: "dtree_train",
            category: "dtree",
            highway_class: 2,
            outputs: &["tree_id", "accuracy", "depth"],
            inputs: &["training_data", "target_column"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "dtree_predict",
            category: "dtree",
            highway_class: 2,
            outputs: &["prediction", "confidence", "path"],
            inputs: &["tree_id", "input_row"],
            depends_on: &["dtree_train"],
            alternatives: &[],
        },
        ToolMeta {
            name: "dtree_prune",
            category: "dtree",
            highway_class: 2,
            outputs: &["pruned_tree", "nodes_removed"],
            inputs: &["tree_id", "alpha"],
            depends_on: &["dtree_train"],
            alternatives: &[],
        },
        ToolMeta {
            name: "dtree_info",
            category: "dtree",
            highway_class: 2,
            outputs: &["tree_structure", "node_count", "depth"],
            inputs: &["tree_id"],
            depends_on: &["dtree_train"],
            alternatives: &[],
        },
        ToolMeta {
            name: "dtree_importance",
            category: "dtree",
            highway_class: 2,
            outputs: &["feature_importance_scores"],
            inputs: &["tree_id"],
            depends_on: &["dtree_train"],
            alternatives: &[],
        },
        ToolMeta {
            name: "dtree_export",
            category: "dtree",
            highway_class: 2,
            outputs: &["rules", "export_format"],
            inputs: &["tree_id"],
            depends_on: &["dtree_train"],
            alternatives: &[],
        },
        // edu (2 tools) — agent training
        ToolMeta {
            name: "edu_train_agent",
            category: "edu",
            highway_class: 3,
            outputs: &["training_status", "competency_achieved"],
            inputs: &["agent_id", "curriculum"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "edu_evaluate",
            category: "edu",
            highway_class: 3,
            outputs: &["competency_score", "certifications"],
            inputs: &["agent_id"],
            depends_on: &["edu_train_agent"],
            alternatives: &[],
        },
        // explore (3 tools) — frontier exploration
        ToolMeta {
            name: "explore_get_frontier",
            category: "explore",
            highway_class: 3,
            outputs: &["frontier", "opportunity_vectors"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "explore_launch_mission",
            category: "explore",
            highway_class: 3,
            outputs: &["mission_id", "launch_status"],
            inputs: &["mission_params"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "explore_record_discovery",
            category: "explore",
            highway_class: 3,
            outputs: &["discovery_id", "cataloging_status"],
            inputs: &["discovery_data"],
            depends_on: &[],
            alternatives: &[],
        },
        // faers (11 tools) — FDA Adverse Event Reporting System
        ToolMeta {
            name: "faers_etl_known_pairs",
            category: "faers",
            highway_class: 4,
            outputs: &["known_pairs", "pair_count"],
            inputs: &["drug_name"],
            depends_on: &[],
            alternatives: &["faers_search"],
        },
        ToolMeta {
            name: "faers_compare_drugs",
            category: "faers",
            highway_class: 4,
            outputs: &["comparison", "relative_risk"],
            inputs: &["drugs", "event"],
            depends_on: &["faers_drug_events"],
            alternatives: &[],
        },
        ToolMeta {
            name: "faers_etl_status",
            category: "faers",
            highway_class: 4,
            outputs: &["etl_status", "last_run"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "faers_outcome_conditioned",
            category: "faers",
            highway_class: 4,
            outputs: &["conditioned_events", "outcome_distribution"],
            inputs: &["drug_name", "outcome"],
            depends_on: &["faers_search"],
            alternatives: &[],
        },
        ToolMeta {
            name: "faers_polypharmacy",
            category: "faers",
            highway_class: 4,
            outputs: &["interaction_signals", "drug_combinations"],
            inputs: &["drugs"],
            depends_on: &["faers_search"],
            alternatives: &[],
        },
        ToolMeta {
            name: "faers_seriousness_cascade",
            category: "faers",
            highway_class: 4,
            outputs: &["seriousness_levels", "cascade_analysis"],
            inputs: &["drug_name", "event"],
            depends_on: &["faers_drug_events"],
            alternatives: &[],
        },
        ToolMeta {
            name: "faers_reporter_weighted",
            category: "faers",
            highway_class: 4,
            outputs: &["weighted_signal", "reporter_breakdown"],
            inputs: &["drug_name", "event"],
            depends_on: &["faers_search"],
            alternatives: &[],
        },
        ToolMeta {
            name: "faers_etl_run",
            category: "faers",
            highway_class: 4,
            outputs: &["etl_result", "records_processed"],
            inputs: &["quarter"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "faers_etl_signals",
            category: "faers",
            highway_class: 4,
            outputs: &["signal_scores", "2x2_table"],
            inputs: &["drug_name", "event"],
            depends_on: &["faers_etl_run"],
            alternatives: &["pv_signal_complete"],
        },
        ToolMeta {
            name: "faers_signal_velocity",
            category: "faers",
            highway_class: 4,
            outputs: &["velocity", "trend", "acceleration"],
            inputs: &["drug_name", "event", "time_window"],
            depends_on: &["faers_search"],
            alternatives: &[],
        },
        ToolMeta {
            name: "faers_geographic_divergence",
            category: "faers",
            highway_class: 4,
            outputs: &["geographic_signals", "divergence_score"],
            inputs: &["drug_name", "event"],
            depends_on: &["faers_search"],
            alternatives: &[],
        },
        // fda (2 tools) — FDA bridge evaluation
        ToolMeta {
            name: "fda_bridge_evaluate",
            category: "fda",
            highway_class: 2,
            outputs: &["signal_classification", "compliance_status"],
            inputs: &["fda_data"],
            depends_on: &["pv_signal_complete"],
            alternatives: &[],
        },
        ToolMeta {
            name: "fda_bridge_batch",
            category: "fda",
            highway_class: 2,
            outputs: &["batch_results", "summary"],
            inputs: &["batch"],
            depends_on: &["fda_bridge_evaluate"],
            alternatives: &[],
        },
        // fed (2 tools) — budget governance
        ToolMeta {
            name: "fed_budget_report",
            category: "fed",
            highway_class: 3,
            outputs: &["budget_summary"],
            inputs: &["fiscal_period"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "fed_recommend_model",
            category: "fed",
            highway_class: 3,
            outputs: &["recommended_model", "rationale"],
            inputs: &["budget_constraint", "project_type"],
            depends_on: &["fed_budget_report"],
            alternatives: &[],
        },
        // forge (5 tools) — game theory + code generation
        ToolMeta {
            name: "forge_code_generate",
            category: "forge",
            highway_class: 3,
            outputs: &["generated_code", "file_path"],
            inputs: &["specification"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "forge_nash_solve",
            category: "forge",
            highway_class: 3,
            outputs: &["equilibria", "strategies"],
            inputs: &["payoff_matrix"],
            depends_on: &["forge_payoff_matrix"],
            alternatives: &["game_theory_nash_2x2"],
        },
        ToolMeta {
            name: "forge_payoff_matrix",
            category: "forge",
            highway_class: 3,
            outputs: &["payoff_matrix"],
            inputs: &["strategies_a", "strategies_b", "payoffs"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "game_theory_nash_2x2",
            category: "forge",
            highway_class: 3,
            outputs: &["equilibria", "dominant_strategy"],
            inputs: &["payoff_matrix_2x2"],
            depends_on: &[],
            alternatives: &["forge_nash_solve"],
        },
        ToolMeta {
            name: "forge_quality_score",
            category: "forge",
            highway_class: 3,
            outputs: &["quality_score", "factors"],
            inputs: &["target"],
            depends_on: &[],
            alternatives: &["validation_run"],
        },
        // foundation (5 tools) — core utilities
        ToolMeta {
            name: "foundation_fsrs_review",
            category: "foundation",
            highway_class: 1,
            outputs: &["updated_card", "ease_factor"],
            inputs: &["card", "rating", "timestamp"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "foundation_yaml_parse",
            category: "foundation",
            highway_class: 1,
            outputs: &["json_value"],
            inputs: &["yaml_content"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "foundation_token_ratio",
            category: "foundation",
            highway_class: 1,
            outputs: &["ratio", "urgency_flag"],
            inputs: &["current_tokens", "budget_tokens"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "foundation_spectral_overlap",
            category: "foundation",
            highway_class: 1,
            outputs: &["overlap_coefficient", "intersection"],
            inputs: &["spectrum_a", "spectrum_b"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "foundation_flywheel_velocity",
            category: "foundation",
            highway_class: 1,
            outputs: &["steady_state_velocity", "convergence_time"],
            inputs: &["input_rate", "feedback_gain", "friction"],
            depends_on: &[],
            alternatives: &[],
        },
        // frontend (6 tools) — UI/UX analysis
        ToolMeta {
            name: "frontend_color_blend",
            category: "frontend",
            highway_class: 4,
            outputs: &["blended_color"],
            inputs: &["color_a", "color_b", "ratio"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "frontend_type_scale_audit",
            category: "frontend",
            highway_class: 4,
            outputs: &["audit_result", "violations"],
            inputs: &["component_tree"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "frontend_spacing_audit",
            category: "frontend",
            highway_class: 4,
            outputs: &["audit_result", "violations"],
            inputs: &["component_tree"],
            depends_on: &[],
            alternatives: &["frontend_type_scale_audit"],
        },
        ToolMeta {
            name: "frontend_touch_target",
            category: "frontend",
            highway_class: 4,
            outputs: &["compliant", "target_size"],
            inputs: &["element"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "frontend_wcag_contrast",
            category: "frontend",
            highway_class: 4,
            outputs: &["contrast_ratio", "wcag_level"],
            inputs: &["foreground", "background"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "frontend_a11y_summary",
            category: "frontend",
            highway_class: 4,
            outputs: &["a11y_score", "issues"],
            inputs: &["page_url"],
            depends_on: &[],
            alternatives: &[],
        },
        // gcloud (19 tools) — Google Cloud Platform
        ToolMeta {
            name: "gcloud_projects_describe",
            category: "gcloud",
            highway_class: 4,
            outputs: &["project_details"],
            inputs: &["project_id"],
            depends_on: &[],
            alternatives: &["gcloud_projects_list"],
        },
        ToolMeta {
            name: "gcloud_functions_list",
            category: "gcloud",
            highway_class: 4,
            outputs: &["functions"],
            inputs: &["project", "region"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "gcloud_run_services_list",
            category: "gcloud",
            highway_class: 4,
            outputs: &["services"],
            inputs: &["project", "region"],
            depends_on: &[],
            alternatives: &["gcloud_run_services_describe"],
        },
        ToolMeta {
            name: "gcloud_config_set",
            category: "gcloud",
            highway_class: 4,
            outputs: &["success"],
            inputs: &["property", "value"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "gcloud_logging_read",
            category: "gcloud",
            highway_class: 4,
            outputs: &["logs"],
            inputs: &["filter", "limit", "project"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "gcloud_compute_instances_list",
            category: "gcloud",
            highway_class: 4,
            outputs: &["instances"],
            inputs: &["project", "zone"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "gcloud_storage_buckets_list",
            category: "gcloud",
            highway_class: 4,
            outputs: &["buckets"],
            inputs: &["project"],
            depends_on: &[],
            alternatives: &["gcloud_storage_ls"],
        },
        ToolMeta {
            name: "gcloud_iam_service_accounts_list",
            category: "gcloud",
            highway_class: 4,
            outputs: &["accounts"],
            inputs: &["project"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "gcloud_run_services_describe",
            category: "gcloud",
            highway_class: 4,
            outputs: &["service_details"],
            inputs: &["name", "region", "project"],
            depends_on: &[],
            alternatives: &["gcloud_run_services_list"],
        },
        ToolMeta {
            name: "gcloud_secrets_versions_access",
            category: "gcloud",
            highway_class: 4,
            outputs: &["secret_value"],
            inputs: &["secret_name", "version", "project"],
            depends_on: &[],
            alternatives: &["gcloud_secrets_list"],
        },
        ToolMeta {
            name: "gcloud_auth_list",
            category: "gcloud",
            highway_class: 4,
            outputs: &["accounts"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "gcloud_config_list",
            category: "gcloud",
            highway_class: 4,
            outputs: &["properties"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["gcloud_config_get"],
        },
        ToolMeta {
            name: "gcloud_projects_list",
            category: "gcloud",
            highway_class: 4,
            outputs: &["projects"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["gcloud_projects_describe"],
        },
        ToolMeta {
            name: "gcloud_secrets_list",
            category: "gcloud",
            highway_class: 4,
            outputs: &["secrets"],
            inputs: &["project"],
            depends_on: &[],
            alternatives: &["gcloud_secrets_versions_access"],
        },
        ToolMeta {
            name: "gcloud_run_command",
            category: "gcloud",
            highway_class: 4,
            outputs: &["output", "exit_code"],
            inputs: &["command"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "gcloud_config_get",
            category: "gcloud",
            highway_class: 4,
            outputs: &["value"],
            inputs: &["property"],
            depends_on: &[],
            alternatives: &["gcloud_config_list"],
        },
        ToolMeta {
            name: "gcloud_storage_cp",
            category: "gcloud",
            highway_class: 4,
            outputs: &["success", "bytes_transferred"],
            inputs: &["source", "destination"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "gcloud_projects_get_iam_policy",
            category: "gcloud",
            highway_class: 4,
            outputs: &["bindings"],
            inputs: &["project_id"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "gcloud_storage_ls",
            category: "gcloud",
            highway_class: 4,
            outputs: &["objects"],
            inputs: &["path"],
            depends_on: &[],
            alternatives: &["gcloud_storage_buckets_list"],
        },
        // git (8 tools) — version control
        ToolMeta {
            name: "git_status",
            category: "git",
            highway_class: 3,
            outputs: &["branch", "staged", "modified", "untracked", "files"],
            inputs: &["path"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "git_diff",
            category: "git",
            highway_class: 3,
            outputs: &["diff", "stat"],
            inputs: &["path", "staged", "file"],
            depends_on: &[],
            alternatives: &["git_status"],
        },
        ToolMeta {
            name: "git_log",
            category: "git",
            highway_class: 3,
            outputs: &["commits"],
            inputs: &["path", "count"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "git_commit",
            category: "git",
            highway_class: 3,
            outputs: &["commit_hash", "success"],
            inputs: &["message", "files"],
            depends_on: &["git_status"],
            alternatives: &[],
        },
        ToolMeta {
            name: "git_branch",
            category: "git",
            highway_class: 3,
            outputs: &["branches"],
            inputs: &["path"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "git_checkout",
            category: "git",
            highway_class: 3,
            outputs: &["success"],
            inputs: &["target"],
            depends_on: &[],
            alternatives: &["git_branch"],
        },
        ToolMeta {
            name: "git_push",
            category: "git",
            highway_class: 3,
            outputs: &["success"],
            inputs: &["remote", "branch"],
            depends_on: &["git_commit"],
            alternatives: &[],
        },
        ToolMeta {
            name: "git_stash",
            category: "git",
            highway_class: 3,
            outputs: &["success", "stash_list"],
            inputs: &["action"],
            depends_on: &[],
            alternatives: &[],
        },
        // gh (5 tools) — GitHub CLI
        ToolMeta {
            name: "gh_pr_create",
            category: "gh",
            highway_class: 4,
            outputs: &["pr_url"],
            inputs: &["title", "body"],
            depends_on: &["git_push"],
            alternatives: &[],
        },
        ToolMeta {
            name: "gh_pr_view",
            category: "gh",
            highway_class: 4,
            outputs: &["pr_data"],
            inputs: &["number"],
            depends_on: &[],
            alternatives: &["gh_pr_list"],
        },
        ToolMeta {
            name: "gh_pr_list",
            category: "gh",
            highway_class: 4,
            outputs: &["prs"],
            inputs: &["state"],
            depends_on: &[],
            alternatives: &["gh_pr_view"],
        },
        ToolMeta {
            name: "gh_issue_view",
            category: "gh",
            highway_class: 4,
            outputs: &["issue_data"],
            inputs: &["number"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "gh_api",
            category: "gh",
            highway_class: 4,
            outputs: &["api_response"],
            inputs: &["endpoint", "method"],
            depends_on: &[],
            alternatives: &[],
        },
        // systemctl (4 tools) — service management
        ToolMeta {
            name: "systemctl_status",
            category: "systemctl",
            highway_class: 3,
            outputs: &["active_state", "sub_state", "pid"],
            inputs: &["unit"],
            depends_on: &[],
            alternatives: &["systemctl_list"],
        },
        ToolMeta {
            name: "systemctl_restart",
            category: "systemctl",
            highway_class: 3,
            outputs: &["success"],
            inputs: &["unit"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "systemctl_start",
            category: "systemctl",
            highway_class: 3,
            outputs: &["success"],
            inputs: &["unit"],
            depends_on: &[],
            alternatives: &["systemctl_restart"],
        },
        ToolMeta {
            name: "systemctl_list",
            category: "systemctl",
            highway_class: 3,
            outputs: &["units"],
            inputs: &["state"],
            depends_on: &[],
            alternatives: &["systemctl_status"],
        },
        // npm (4 tools) — package management
        ToolMeta {
            name: "npm_run",
            category: "npm",
            highway_class: 3,
            outputs: &["output", "success"],
            inputs: &["script", "path"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "npm_install",
            category: "npm",
            highway_class: 3,
            outputs: &["success"],
            inputs: &["packages", "dev"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "npm_list",
            category: "npm",
            highway_class: 3,
            outputs: &["dependencies"],
            inputs: &["depth"],
            depends_on: &[],
            alternatives: &["npm_outdated"],
        },
        ToolMeta {
            name: "npm_outdated",
            category: "npm",
            highway_class: 3,
            outputs: &["outdated_packages"],
            inputs: &["path"],
            depends_on: &[],
            alternatives: &["npm_list"],
        },
        // fs (4 tools) — filesystem operations
        ToolMeta {
            name: "fs_mkdir",
            category: "fs",
            highway_class: 1,
            outputs: &["success"],
            inputs: &["path", "parents"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "fs_copy",
            category: "fs",
            highway_class: 1,
            outputs: &["success"],
            inputs: &["source", "dest"],
            depends_on: &[],
            alternatives: &["fs_move"],
        },
        ToolMeta {
            name: "fs_move",
            category: "fs",
            highway_class: 1,
            outputs: &["success"],
            inputs: &["source", "dest"],
            depends_on: &[],
            alternatives: &["fs_copy"],
        },
        ToolMeta {
            name: "fs_chmod",
            category: "fs",
            highway_class: 1,
            outputs: &["success"],
            inputs: &["path", "mode"],
            depends_on: &[],
            alternatives: &[],
        },
        // gsa (2 tools) — procurement
        ToolMeta {
            name: "gsa_audit_value",
            category: "gsa",
            highway_class: 3,
            outputs: &["value_assessment", "cost_efficiency"],
            inputs: &["procurement_id"],
            depends_on: &["gsa_procure"],
            alternatives: &[],
        },
        ToolMeta {
            name: "gsa_procure",
            category: "gsa",
            highway_class: 3,
            outputs: &["procurement_id", "vendor_assigned"],
            inputs: &["procurement_spec"],
            depends_on: &[],
            alternatives: &[],
        },
        // guardian (8 tools) — homeostasis control loop
        ToolMeta {
            name: "guardian_sensors_list",
            category: "guardian",
            highway_class: 3,
            outputs: &["sensors"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["guardian_actuators_list"],
        },
        ToolMeta {
            name: "guardian_actuators_list",
            category: "guardian",
            highway_class: 3,
            outputs: &["actuators"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["guardian_sensors_list"],
        },
        ToolMeta {
            name: "guardian_ceiling_for_originator",
            category: "guardian",
            highway_class: 3,
            outputs: &["risk_ceiling", "factors"],
            inputs: &["originator_type", "context"],
            depends_on: &["guardian_originator_classify"],
            alternatives: &[],
        },
        ToolMeta {
            name: "guardian_originator_classify",
            category: "guardian",
            highway_class: 3,
            outputs: &["originator_type", "risk_profile"],
            inputs: &["entity_id", "entity_type"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "guardian_space3d_compute",
            category: "guardian",
            highway_class: 3,
            outputs: &["point_3d", "safety_distance"],
            inputs: &["temporal", "scope", "mechanism"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "guardian_reset",
            category: "guardian",
            highway_class: 3,
            outputs: &["success"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "guardian_inject_signal",
            category: "guardian",
            highway_class: 3,
            outputs: &["signal_id", "processed"],
            inputs: &["signal_type", "severity", "source"],
            depends_on: &[],
            alternatives: &["guardian_homeostasis_tick"],
        },
        ToolMeta {
            name: "guardian_history",
            category: "guardian",
            highway_class: 3,
            outputs: &["events"],
            inputs: &["limit"],
            depends_on: &[],
            alternatives: &["guardian_status"],
        },
        // guidelines (7 tools) — regulatory guideline lookup
        ToolMeta {
            name: "ich_stats",
            category: "guidelines",
            highway_class: 2,
            outputs: &["term_count", "guideline_count"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "ich_search",
            category: "guidelines",
            highway_class: 2,
            outputs: &["matches"],
            inputs: &["query", "limit"],
            depends_on: &[],
            alternatives: &["ich_lookup", "guidelines_search"],
        },
        ToolMeta {
            name: "guidelines_categories",
            category: "guidelines",
            highway_class: 2,
            outputs: &["categories"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "guidelines_url",
            category: "guidelines",
            highway_class: 2,
            outputs: &["url", "access_type"],
            inputs: &["guideline_id"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "guidelines_get",
            category: "guidelines",
            highway_class: 2,
            outputs: &["full_text", "metadata", "sections"],
            inputs: &["guideline_id"],
            depends_on: &[],
            alternatives: &["guidelines_search"],
        },
        ToolMeta {
            name: "guidelines_pv_all",
            category: "guidelines",
            highway_class: 2,
            outputs: &["pv_guidelines"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["guidelines_search"],
        },
        // fda_guidance (5 tools) — FDA guidance document search
        ToolMeta {
            name: "fda_guidance_search",
            category: "fda_guidance",
            highway_class: 2,
            outputs: &["guidance_documents"],
            inputs: &["query", "center", "product", "status"],
            depends_on: &[],
            alternatives: &["guidelines_search"],
        },
        ToolMeta {
            name: "fda_guidance_get",
            category: "fda_guidance",
            highway_class: 2,
            outputs: &["guidance_detail"],
            inputs: &["slug"],
            depends_on: &[],
            alternatives: &["fda_guidance_search"],
        },
        ToolMeta {
            name: "fda_guidance_categories",
            category: "fda_guidance",
            highway_class: 2,
            outputs: &["categories"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "fda_guidance_url",
            category: "fda_guidance",
            highway_class: 2,
            outputs: &["url"],
            inputs: &["slug"],
            depends_on: &[],
            alternatives: &["fda_guidance_get"],
        },
        ToolMeta {
            name: "fda_guidance_status",
            category: "fda_guidance",
            highway_class: 2,
            outputs: &["guidance_documents"],
            inputs: &["status"],
            depends_on: &[],
            alternatives: &["fda_guidance_search"],
        },
        ToolMeta {
            name: "ich_guideline",
            category: "guidelines",
            highway_class: 2,
            outputs: &["full_text", "sections"],
            inputs: &["guideline_code"],
            depends_on: &[],
            alternatives: &["guidelines_get"],
        },
        // health (2 tools)
        ToolMeta {
            name: "health_measure_impact",
            category: "health",
            highway_class: 2,
            outputs: &["impact_score", "confidence"],
            inputs: &["intervention", "target_metric"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "health_validate_signal",
            category: "health",
            highway_class: 2,
            outputs: &["validation_status", "health_assessment"],
            inputs: &["health_signal"],
            depends_on: &[],
            alternatives: &["signal_detect"],
        },
        // hooks (6 tools) — hook introspection
        ToolMeta {
            name: "hook_list_nested",
            category: "hooks",
            highway_class: 3,
            outputs: &["hooks"],
            inputs: &["parent_id"],
            depends_on: &[],
            alternatives: &["hooks_stats"],
        },
        ToolMeta {
            name: "hooks_for_event",
            category: "hooks",
            highway_class: 3,
            outputs: &["hooks"],
            inputs: &["event"],
            depends_on: &[],
            alternatives: &["hooks_for_tier"],
        },
        ToolMeta {
            name: "hooks_metrics_by_event",
            category: "hooks",
            highway_class: 3,
            outputs: &["event_metrics"],
            inputs: &["event"],
            depends_on: &[],
            alternatives: &["hooks_metrics_summary"],
        },
        ToolMeta {
            name: "hooks_for_tier",
            category: "hooks",
            highway_class: 3,
            outputs: &["hooks"],
            inputs: &["tier"],
            depends_on: &[],
            alternatives: &["hooks_for_event"],
        },
        ToolMeta {
            name: "hooks_stats",
            category: "hooks",
            highway_class: 3,
            outputs: &["total_hooks", "by_event", "executions_total"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["hooks_metrics_summary"],
        },
        ToolMeta {
            name: "hooks_metrics_summary",
            category: "hooks",
            highway_class: 3,
            outputs: &["summary", "success_rates"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["hooks_stats"],
        },
        // hormones (4 tools) — system-wide signaling
        ToolMeta {
            name: "hormone_modifiers",
            category: "hormones",
            highway_class: 2,
            outputs: &["modifiers"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "hormone_get",
            category: "hormones",
            highway_class: 2,
            outputs: &["hormone_level", "baseline", "half_life"],
            inputs: &["hormone_name"],
            depends_on: &[],
            alternatives: &["hormone_status"],
        },
        ToolMeta {
            name: "hormone_stimulus",
            category: "hormones",
            highway_class: 2,
            outputs: &["new_level", "peak_time_ms"],
            inputs: &["hormone_name", "stimulus_strength"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "hormone_status",
            category: "hormones",
            highway_class: 2,
            outputs: &["hormones"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["hormone_get"],
        },
        // immunity (4 tools) — antipattern detection
        ToolMeta {
            name: "immunity_status",
            category: "immunity",
            highway_class: 2,
            outputs: &["antibody_count", "scan_coverage"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "immunity_scan_errors",
            category: "immunity",
            highway_class: 2,
            outputs: &["matched_patterns"],
            inputs: &["error_stack"],
            depends_on: &[],
            alternatives: &["immunity_scan"],
        },
        ToolMeta {
            name: "immunity_get",
            category: "immunity",
            highway_class: 2,
            outputs: &["antibody"],
            inputs: &["antibody_id"],
            depends_on: &[],
            alternatives: &["immunity_list"],
        },
        ToolMeta {
            name: "immunity_list",
            category: "immunity",
            highway_class: 2,
            outputs: &["antibodies"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["immunity_get"],
        },
        // integrity (3 tools)
        ToolMeta {
            name: "integrity_calibration",
            category: "integrity",
            highway_class: 2,
            outputs: &["calibration_status"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "integrity_analyze",
            category: "integrity",
            highway_class: 2,
            outputs: &["analysis_report", "integrity_score"],
            inputs: &["target"],
            depends_on: &[],
            alternatives: &["validation_run"],
        },
        ToolMeta {
            name: "integrity_assess_ksb",
            category: "integrity",
            highway_class: 2,
            outputs: &["ksb_assessment"],
            inputs: &["ksb_id"],
            depends_on: &[],
            alternatives: &[],
        },
        // lex_primitiva (4 tools) — primitive analysis
        ToolMeta {
            name: "lex_primitiva_molecular_weight",
            category: "lex_primitiva",
            highway_class: 1,
            outputs: &["information_weight", "complexity_score"],
            inputs: &["concept"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "lex_primitiva_get",
            category: "lex_primitiva",
            highway_class: 1,
            outputs: &["primitive"],
            inputs: &["name_or_symbol"],
            depends_on: &[],
            alternatives: &["lex_primitiva_list"],
        },
        ToolMeta {
            name: "lex_primitiva_list",
            category: "lex_primitiva",
            highway_class: 1,
            outputs: &["primitives"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["lex_primitiva_get"],
        },
        ToolMeta {
            name: "lex_primitiva_reverse_lookup",
            category: "lex_primitiva",
            highway_class: 1,
            outputs: &["matching_types"],
            inputs: &["primitives"],
            depends_on: &[],
            alternatives: &["lex_primitiva_reverse_compose"],
        },
        // mcp (2 tools)
        ToolMeta {
            name: "mcp_server_get",
            category: "mcp",
            highway_class: 3,
            outputs: &["server_details"],
            inputs: &["server_name"],
            depends_on: &[],
            alternatives: &["mcp_servers_list"],
        },
        ToolMeta {
            name: "mcp_servers_list",
            category: "mcp",
            highway_class: 3,
            outputs: &["servers"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["mcp_server_get"],
        },
        // mesh (6 tools) — MeSH medical terminology
        ToolMeta {
            name: "mesh_crossref",
            category: "mesh",
            highway_class: 1,
            outputs: &["crossrefs"],
            inputs: &["mesh_id"],
            depends_on: &["mesh_lookup"],
            alternatives: &[],
        },
        ToolMeta {
            name: "mesh_lookup",
            category: "mesh",
            highway_class: 1,
            outputs: &["mesh_term", "definition"],
            inputs: &["term"],
            depends_on: &[],
            alternatives: &["mesh_search"],
        },
        ToolMeta {
            name: "mesh_search",
            category: "mesh",
            highway_class: 1,
            outputs: &["matches"],
            inputs: &["query"],
            depends_on: &[],
            alternatives: &["mesh_lookup"],
        },
        ToolMeta {
            name: "mesh_tree",
            category: "mesh",
            highway_class: 1,
            outputs: &["tree_hierarchy"],
            inputs: &["mesh_id"],
            depends_on: &["mesh_lookup"],
            alternatives: &[],
        },
        ToolMeta {
            name: "mesh_enrich_pubmed",
            category: "mesh",
            highway_class: 1,
            outputs: &["enriched_terms", "pubmed_links"],
            inputs: &["mesh_id"],
            depends_on: &["mesh_lookup"],
            alternatives: &[],
        },
        ToolMeta {
            name: "mesh_consistency",
            category: "mesh",
            highway_class: 1,
            outputs: &["consistency_score", "issues"],
            inputs: &["terms"],
            depends_on: &[],
            alternatives: &[],
        },
        // nexcore (3 tools) — system-level
        ToolMeta {
            name: "nexcore_assist",
            category: "nexcore",
            highway_class: 3,
            outputs: &["response"],
            inputs: &["query"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "nexcore",
            category: "nexcore",
            highway_class: 3,
            outputs: &["info"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "nexcore_health_probe",
            category: "nexcore",
            highway_class: 3,
            outputs: &["health_status", "subsystems"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["guardian_status"],
        },
        // node_hunt (2 tools)
        ToolMeta {
            name: "node_hunt_isolate",
            category: "node_hunt",
            highway_class: 3,
            outputs: &["isolated_node", "connections"],
            inputs: &["node_id"],
            depends_on: &["node_hunt_scan"],
            alternatives: &[],
        },
        ToolMeta {
            name: "node_hunt_scan",
            category: "node_hunt",
            highway_class: 3,
            outputs: &["nodes_found", "graph"],
            inputs: &["search_pattern"],
            depends_on: &[],
            alternatives: &[],
        },
        // nsf (1 tool)
        ToolMeta {
            name: "nsf_fund_research",
            category: "nsf",
            highway_class: 3,
            outputs: &["funding_status", "award_amount"],
            inputs: &["research_proposal"],
            depends_on: &[],
            alternatives: &[],
        },
        // perplexity (3 tools) — AI-powered web research
        ToolMeta {
            name: "perplexity_regulatory",
            category: "perplexity",
            highway_class: 4,
            outputs: &["research_result", "sources"],
            inputs: &["query"],
            depends_on: &[],
            alternatives: &["perplexity_research", "guidelines_search"],
        },
        ToolMeta {
            name: "perplexity_competitive",
            category: "perplexity",
            highway_class: 4,
            outputs: &["research_result", "sources"],
            inputs: &["query"],
            depends_on: &[],
            alternatives: &["perplexity_research"],
        },
        ToolMeta {
            name: "perplexity_research",
            category: "perplexity",
            highway_class: 4,
            outputs: &["research_result", "sources"],
            inputs: &["query"],
            depends_on: &[],
            alternatives: &["perplexity_regulatory", "perplexity_competitive"],
        },
        // primitive_validation (6 tools) — corpus-backed validation
        ToolMeta {
            name: "primitive_cite",
            category: "primitive_validation",
            highway_class: 3,
            outputs: &["citation"],
            inputs: &["pmid_or_doi"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "primitive_scan",
            category: "primitive_validation",
            highway_class: 3,
            outputs: &["primitives_found", "coverage"],
            inputs: &["domain"],
            depends_on: &[],
            alternatives: &["primitive_batch_test"],
        },
        ToolMeta {
            name: "primitive_validation_tiers",
            category: "primitive_validation",
            highway_class: 3,
            outputs: &["tiers"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "primitive_validate_batch",
            category: "primitive_validation",
            highway_class: 3,
            outputs: &["validations"],
            inputs: &["terms"],
            depends_on: &["primitive_validate"],
            alternatives: &[],
        },
        ToolMeta {
            name: "primitive_batch_test",
            category: "primitive_validation",
            highway_class: 3,
            outputs: &["test_results"],
            inputs: &["primitives"],
            depends_on: &[],
            alternatives: &["primitive_scan"],
        },
        ToolMeta {
            name: "primitive_skill_lookup",
            category: "primitive_validation",
            highway_class: 3,
            outputs: &["matching_skills"],
            inputs: &["primitive"],
            depends_on: &[],
            alternatives: &["skill_search_by_tag"],
        },
        // principles (2 tools) — decision frameworks
        ToolMeta {
            name: "principles_get",
            category: "principles",
            highway_class: 3,
            outputs: &["principle"],
            inputs: &["principle_id"],
            depends_on: &[],
            alternatives: &["principles_list"],
        },
        ToolMeta {
            name: "principles_list",
            category: "principles",
            highway_class: 3,
            outputs: &["principles"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["principles_get"],
        },
        // pv (21 tools) — pharmacovigilance signal detection, DSL, control loop
        ToolMeta {
            name: "pvdsl_compile",
            category: "pv",
            highway_class: 2,
            outputs: &["compiled_ast"],
            inputs: &["dsl_code"],
            depends_on: &[],
            alternatives: &["pvdsl_execute"],
        },
        ToolMeta {
            name: "pv_axioms_regulation_search",
            category: "pv",
            highway_class: 2,
            outputs: &["regulations"],
            inputs: &["query"],
            depends_on: &[],
            alternatives: &["guidelines_search"],
        },
        ToolMeta {
            name: "pv_signal_ebgm",
            category: "pv",
            highway_class: 2,
            outputs: &["signal_score", "eb05", "ebgm"],
            inputs: &["2x2_table"],
            depends_on: &[],
            alternatives: &["pv_signal_prr", "pv_signal_ror", "pv_signal_ic"],
        },
        ToolMeta {
            name: "pv_signal_strength",
            category: "pv",
            highway_class: 2,
            outputs: &["strength_label"],
            inputs: &["signal_metrics"],
            depends_on: &["signal_detect"],
            alternatives: &[],
        },
        ToolMeta {
            name: "value_signal_detect",
            category: "pv",
            highway_class: 2,
            outputs: &["signal_detected", "value_metrics"],
            inputs: &["value_data"],
            depends_on: &[],
            alternatives: &["signal_detect"],
        },
        ToolMeta {
            name: "pvdsl_functions",
            category: "pv",
            highway_class: 2,
            outputs: &["functions"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "signal_detect",
            category: "pv",
            highway_class: 2,
            outputs: &["signal_score", "strength", "signal_detected"],
            inputs: &["drug", "event", "2x2_table"],
            depends_on: &["pv_signal_complete"],
            alternatives: &["pv_signal_complete"],
        },
        ToolMeta {
            name: "signal_thresholds",
            category: "pv",
            highway_class: 2,
            outputs: &["thresholds"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "pv_axioms_query",
            category: "pv",
            highway_class: 2,
            outputs: &["axiom_result"],
            inputs: &["query"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "pvdsl_eval",
            category: "pv",
            highway_class: 2,
            outputs: &["value"],
            inputs: &["expression", "context"],
            depends_on: &[],
            alternatives: &["pvdsl_execute"],
        },
        ToolMeta {
            name: "signal_batch",
            category: "pv",
            highway_class: 2,
            outputs: &["batch_results", "signals_found"],
            inputs: &["items"],
            depends_on: &["signal_detect"],
            alternatives: &[],
        },
        ToolMeta {
            name: "pv_axioms_traceability_chain",
            category: "pv",
            highway_class: 2,
            outputs: &["traceability_chain"],
            inputs: &["axiom_id"],
            depends_on: &["pv_axioms_query"],
            alternatives: &[],
        },
        ToolMeta {
            name: "pv_signal_ic",
            category: "pv",
            highway_class: 2,
            outputs: &["signal_score", "ic025"],
            inputs: &["2x2_table"],
            depends_on: &[],
            alternatives: &["pv_signal_prr", "pv_signal_ror", "pv_signal_ebgm"],
        },
        ToolMeta {
            name: "value_baseline_create",
            category: "pv",
            highway_class: 2,
            outputs: &["baseline_id", "baseline_metrics"],
            inputs: &["data_source"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "value_signal_types",
            category: "pv",
            highway_class: 2,
            outputs: &["signal_types"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "value_pv_mapping",
            category: "pv",
            highway_class: 2,
            outputs: &["pv_mapping", "confidence"],
            inputs: &["value_concept"],
            depends_on: &[],
            alternatives: &["chemistry_pv_mappings"],
        },
        ToolMeta {
            name: "pvdsl_execute",
            category: "pv",
            highway_class: 2,
            outputs: &["result", "execution_trace"],
            inputs: &["dsl_code_or_ast"],
            depends_on: &[],
            alternatives: &["pvdsl_eval"],
        },
        ToolMeta {
            name: "pv_axioms_ksb_lookup",
            category: "pv",
            highway_class: 2,
            outputs: &["ksb_data"],
            inputs: &["ksb_id"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "pv_control_loop_tick",
            category: "pv",
            highway_class: 2,
            outputs: &["actions", "next_state"],
            inputs: &["pv_state"],
            depends_on: &["pv_signal_complete", "guardian_evaluate_pv"],
            alternatives: &["guardian_homeostasis_tick"],
        },
        ToolMeta {
            name: "pv_axioms_domain_dashboard",
            category: "pv",
            highway_class: 2,
            outputs: &["dashboard"],
            inputs: &["domain"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "pv_pipeline",
            category: "pv",
            highway_class: 2,
            outputs: &["pipeline_result"],
            inputs: &["drug", "event", "2x2_table"],
            depends_on: &["pv_signal_complete", "signal_detect"],
            alternatives: &[],
        },
        // regulatory (3 tools) — regulatory primitive analysis
        ToolMeta {
            name: "regulatory_primitives_audit",
            category: "regulatory",
            highway_class: 2,
            outputs: &["audit_report", "compliance_gaps"],
            inputs: &["concept"],
            depends_on: &["primitive_validate", "regulatory_primitives_extract"],
            alternatives: &[],
        },
        ToolMeta {
            name: "regulatory_primitives_extract",
            category: "regulatory",
            highway_class: 2,
            outputs: &["extracted_terms", "confidence"],
            inputs: &["text", "domain"],
            depends_on: &[],
            alternatives: &["primitive_validate"],
        },
        ToolMeta {
            name: "regulatory_primitives_compare",
            category: "regulatory",
            highway_class: 2,
            outputs: &["similarity", "alignment_assessment"],
            inputs: &["term_a", "term_b", "domain"],
            depends_on: &["lex_primitiva_composition"],
            alternatives: &["edit_distance_similarity"],
        },
        // sba (2 tools) — agent allocation
        ToolMeta {
            name: "sba_allocate_agent",
            category: "sba",
            highway_class: 3,
            outputs: &["agent_id", "allocation_plan"],
            inputs: &["task_description", "resource_constraints"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "sba_chain_next",
            category: "sba",
            highway_class: 3,
            outputs: &["next_agent_id", "handoff_context"],
            inputs: &["current_agent_id", "task_result"],
            depends_on: &["sba_allocate_agent"],
            alternatives: &[],
        },
        // sec (1 tool)
        ToolMeta {
            name: "sec_audit_market",
            category: "sec",
            highway_class: 3,
            outputs: &["audit_report", "violations"],
            inputs: &["market_sector"],
            depends_on: &[],
            alternatives: &[],
        },
        // skill (16 tools) — skill management and execution
        ToolMeta {
            name: "skill_get",
            category: "skill",
            highway_class: 3,
            outputs: &["skill_definition"],
            inputs: &["skill_name"],
            depends_on: &[],
            alternatives: &["skill_list"],
        },
        ToolMeta {
            name: "skill_taxonomy_query",
            category: "skill",
            highway_class: 3,
            outputs: &["taxonomy_results"],
            inputs: &["query"],
            depends_on: &[],
            alternatives: &["skill_taxonomy_list"],
        },
        ToolMeta {
            name: "skill_search_by_tag",
            category: "skill",
            highway_class: 3,
            outputs: &["matching_skills"],
            inputs: &["tag"],
            depends_on: &[],
            alternatives: &["skill_list"],
        },
        ToolMeta {
            name: "skill_list_nested",
            category: "skill",
            highway_class: 3,
            outputs: &["skills_tree"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["skill_list"],
        },
        ToolMeta {
            name: "skill_token_analyze",
            category: "skill",
            highway_class: 3,
            outputs: &["token_analysis"],
            inputs: &["skill_name"],
            depends_on: &["skill_get"],
            alternatives: &[],
        },
        ToolMeta {
            name: "skill_execute",
            category: "skill",
            highway_class: 3,
            outputs: &["execution_result"],
            inputs: &["skill_name", "arguments"],
            depends_on: &["skill_get"],
            alternatives: &[],
        },
        ToolMeta {
            name: "skill_orchestration_analyze",
            category: "skill",
            highway_class: 3,
            outputs: &["orchestration_report"],
            inputs: &["skill_name"],
            depends_on: &["skill_get"],
            alternatives: &[],
        },
        ToolMeta {
            name: "skill_chain_lookup",
            category: "skill",
            highway_class: 3,
            outputs: &["chain"],
            inputs: &["trigger"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "skill_taxonomy_list",
            category: "skill",
            highway_class: 3,
            outputs: &["taxonomy"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["skill_taxonomy_query"],
        },
        ToolMeta {
            name: "skill_compile_check",
            category: "skill",
            highway_class: 3,
            outputs: &["compilation_status", "errors"],
            inputs: &["skill_name"],
            depends_on: &["skill_get"],
            alternatives: &["skill_validate"],
        },
        ToolMeta {
            name: "skill_scan",
            category: "skill",
            highway_class: 3,
            outputs: &["scan_results"],
            inputs: &["directory"],
            depends_on: &[],
            alternatives: &["skill_list"],
        },
        ToolMeta {
            name: "skill_list",
            category: "skill",
            highway_class: 3,
            outputs: &["skills"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["skill_get"],
        },
        ToolMeta {
            name: "skill_compile",
            category: "skill",
            highway_class: 3,
            outputs: &["compiled_skill"],
            inputs: &["skill_name"],
            depends_on: &["skill_get"],
            alternatives: &[],
        },
        ToolMeta {
            name: "skill_validate",
            category: "skill",
            highway_class: 3,
            outputs: &["validation_result"],
            inputs: &["skill_name"],
            depends_on: &["skill_get"],
            alternatives: &["skill_compile_check"],
        },
        ToolMeta {
            name: "skill_categories_compute_intensive",
            category: "skill",
            highway_class: 3,
            outputs: &["categories"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "skill_schema",
            category: "skill",
            highway_class: 3,
            outputs: &["schema"],
            inputs: &["skill_name"],
            depends_on: &["skill_get"],
            alternatives: &[],
        },
        // ssa (2 tools) — state persistence
        ToolMeta {
            name: "ssa_verify_integrity",
            category: "ssa",
            highway_class: 3,
            outputs: &["integrity_valid", "mismatches"],
            inputs: &["persistence_id"],
            depends_on: &["ssa_persist_state"],
            alternatives: &[],
        },
        ToolMeta {
            name: "ssa_persist_state",
            category: "ssa",
            highway_class: 3,
            outputs: &["persistence_id", "checksum"],
            inputs: &["state_snapshot"],
            depends_on: &[],
            alternatives: &[],
        },
        // stem (20 tools) — cross-domain STEM computation
        ToolMeta {
            name: "stem_version",
            category: "stem",
            highway_class: 2,
            outputs: &["version"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "stem_chem_fraction",
            category: "stem",
            highway_class: 2,
            outputs: &["fraction", "ionized_fraction"],
            inputs: &["pka", "ph"],
            depends_on: &[],
            alternatives: &["wolfram_chemistry"],
        },
        ToolMeta {
            name: "stem_tier_info",
            category: "stem",
            highway_class: 2,
            outputs: &["tier_info"],
            inputs: &["tier"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "stem_math_relation_invert",
            category: "stem",
            highway_class: 2,
            outputs: &["inverted_relation"],
            inputs: &["relation"],
            depends_on: &[],
            alternatives: &["wolfram_calculate"],
        },
        ToolMeta {
            name: "stem_chem_rate",
            category: "stem",
            highway_class: 2,
            outputs: &["rate"],
            inputs: &["concentration", "rate_constant"],
            depends_on: &[],
            alternatives: &["chemistry_threshold_rate"],
        },
        ToolMeta {
            name: "stem_spatial_neighborhood",
            category: "stem",
            highway_class: 2,
            outputs: &["neighbors"],
            inputs: &["point", "radius"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "stem_math_identity",
            category: "stem",
            highway_class: 2,
            outputs: &["identity_type"],
            inputs: &["expression"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "stem_spatial_orientation",
            category: "stem",
            highway_class: 2,
            outputs: &["orientation"],
            inputs: &["vector"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "stem_phys_period",
            category: "stem",
            highway_class: 2,
            outputs: &["period", "frequency"],
            inputs: &["parameters"],
            depends_on: &[],
            alternatives: &["wolfram_physics"],
        },
        ToolMeta {
            name: "stem_phys_inertia",
            category: "stem",
            highway_class: 2,
            outputs: &["inertia"],
            inputs: &["mass", "geometry"],
            depends_on: &[],
            alternatives: &["stem_phys_fma"],
        },
        ToolMeta {
            name: "stem_phys_scale",
            category: "stem",
            highway_class: 2,
            outputs: &["scaled_value"],
            inputs: &["value", "scale_factor"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "stem_chem_affinity",
            category: "stem",
            highway_class: 2,
            outputs: &["affinity_score"],
            inputs: &["ligand", "receptor"],
            depends_on: &[],
            alternatives: &["chemistry_langmuir_coverage"],
        },
        ToolMeta {
            name: "stem_math_bounds_check",
            category: "stem",
            highway_class: 2,
            outputs: &["in_bounds", "violations"],
            inputs: &["value", "lower", "upper"],
            depends_on: &[],
            alternatives: &["chemistry_threshold_exceeded"],
        },
        ToolMeta {
            name: "stem_phys_amplitude",
            category: "stem",
            highway_class: 2,
            outputs: &["amplitude"],
            inputs: &["signal"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "stem_math_proof",
            category: "stem",
            highway_class: 2,
            outputs: &["proof_steps", "valid"],
            inputs: &["proposition"],
            depends_on: &[],
            alternatives: &["wolfram_step_by_step"],
        },
        ToolMeta {
            name: "stem_spatial_distance",
            category: "stem",
            highway_class: 2,
            outputs: &["distance"],
            inputs: &["point_a", "point_b"],
            depends_on: &[],
            alternatives: &["foundation_levenshtein"],
        },
        ToolMeta {
            name: "stem_chem_balance",
            category: "stem",
            highway_class: 2,
            outputs: &["balanced_equation"],
            inputs: &["equation"],
            depends_on: &[],
            alternatives: &["wolfram_chemistry"],
        },
        ToolMeta {
            name: "stem_spatial_triangle",
            category: "stem",
            highway_class: 2,
            outputs: &["triangle_properties"],
            inputs: &["vertices"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "stem_spatial_dimension",
            category: "stem",
            highway_class: 2,
            outputs: &["dimensionality"],
            inputs: &["data"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "stem_chem_ratio",
            category: "stem",
            highway_class: 2,
            outputs: &["ratio"],
            inputs: &["numerator", "denominator"],
            depends_on: &[],
            alternatives: &[],
        },
        // telemetry (6 tools) — session telemetry
        ToolMeta {
            name: "telemetry_sources_list",
            category: "telemetry",
            highway_class: 3,
            outputs: &["sources"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "telemetry_recent",
            category: "telemetry",
            highway_class: 3,
            outputs: &["events"],
            inputs: &["limit"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "telemetry_source_analyze",
            category: "telemetry",
            highway_class: 3,
            outputs: &["analysis"],
            inputs: &["source_id"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "telemetry_governance_crossref",
            category: "telemetry",
            highway_class: 3,
            outputs: &["crossref_report"],
            inputs: &["governance_domain"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "telemetry_intel_report",
            category: "telemetry",
            highway_class: 3,
            outputs: &["intel_report"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "telemetry_snapshot_evolution",
            category: "telemetry",
            highway_class: 3,
            outputs: &["evolution_data", "trend"],
            inputs: &["time_range"],
            depends_on: &[],
            alternatives: &[],
        },
        // treasury (2 tools) — financial governance
        ToolMeta {
            name: "treasury_audit",
            category: "treasury",
            highway_class: 3,
            outputs: &["audit_status", "reconciliations_needed"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "treasury_convert_asymmetry",
            category: "treasury",
            highway_class: 3,
            outputs: &["conversion_result"],
            inputs: &["asymmetry_data"],
            depends_on: &[],
            alternatives: &[],
        },
        // validation (2 tools) — validation framework
        ToolMeta {
            name: "validation_domains",
            category: "validation",
            highway_class: 3,
            outputs: &["domains"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "validation_classify_tests",
            category: "validation",
            highway_class: 3,
            outputs: &["classified_tests"],
            inputs: &["test_names"],
            depends_on: &[],
            alternatives: &[],
        },
        // vigil (13 tools) — FRIDAY orchestrator
        ToolMeta {
            name: "vigil_llm_stats",
            category: "vigil",
            highway_class: 4,
            outputs: &["llm_stats"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "vigil_authority_config",
            category: "vigil",
            highway_class: 4,
            outputs: &["authority_config"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "vigil_source_control",
            category: "vigil",
            highway_class: 4,
            outputs: &["control_status"],
            inputs: &["source_id", "action"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "vigil_status",
            category: "vigil",
            highway_class: 4,
            outputs: &["vigil_status", "running_tasks"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["vigil_health"],
        },
        ToolMeta {
            name: "vigil_health",
            category: "vigil",
            highway_class: 4,
            outputs: &["health_status", "subsystems"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["vigil_status"],
        },
        ToolMeta {
            name: "vigil_executor_control",
            category: "vigil",
            highway_class: 4,
            outputs: &["executor_status"],
            inputs: &["action"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "vigil_authority_verify",
            category: "vigil",
            highway_class: 4,
            outputs: &["verified", "authority_level"],
            inputs: &["action", "context"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "vigil_memory_search",
            category: "vigil",
            highway_class: 4,
            outputs: &["results"],
            inputs: &["query"],
            depends_on: &[],
            alternatives: &["vigil_memory_stats"],
        },
        ToolMeta {
            name: "vigil_context_assemble",
            category: "vigil",
            highway_class: 4,
            outputs: &["assembled_context"],
            inputs: &["task_id"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "vigil_emit_event",
            category: "vigil",
            highway_class: 4,
            outputs: &["event_id"],
            inputs: &["event_type", "payload"],
            depends_on: &[],
            alternatives: &["cytokine_emit"],
        },
        ToolMeta {
            name: "vigil_webhook_test",
            category: "vigil",
            highway_class: 4,
            outputs: &["test_result"],
            inputs: &["webhook_url"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "vigil_source_config",
            category: "vigil",
            highway_class: 4,
            outputs: &["source_config"],
            inputs: &["source_id"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "vigil_memory_stats",
            category: "vigil",
            highway_class: 4,
            outputs: &["memory_stats"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["vigil_memory_search"],
        },
        // vigilance (2 tools) — theory of vigilance
        ToolMeta {
            name: "vigilance_harm_types",
            category: "vigilance",
            highway_class: 2,
            outputs: &["harm_types"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "vigilance_map_to_tov",
            category: "vigilance",
            highway_class: 2,
            outputs: &["tov_level", "hierarchy_metadata"],
            inputs: &["level"],
            depends_on: &[],
            alternatives: &[],
        },
        // viz (6 tools) — visualization
        ToolMeta {
            name: "viz_bounds",
            category: "viz",
            highway_class: 1,
            outputs: &["bounds_diagram"],
            inputs: &["type_name"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "viz_type_composition",
            category: "viz",
            highway_class: 1,
            outputs: &["composition_diagram"],
            inputs: &["type_name"],
            depends_on: &["lex_primitiva_composition"],
            alternatives: &[],
        },
        ToolMeta {
            name: "viz_confidence_chain",
            category: "viz",
            highway_class: 1,
            outputs: &["chain_diagram"],
            inputs: &["chain_steps"],
            depends_on: &["stem_confidence_combine"],
            alternatives: &[],
        },
        ToolMeta {
            name: "viz_method_loop",
            category: "viz",
            highway_class: 1,
            outputs: &["loop_diagram"],
            inputs: &["method_name"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "viz_stem_taxonomy",
            category: "viz",
            highway_class: 1,
            outputs: &["taxonomy_diagram"],
            inputs: &["domain"],
            depends_on: &["stem_taxonomy"],
            alternatives: &[],
        },
        ToolMeta {
            name: "viz_dag",
            category: "viz",
            highway_class: 1,
            outputs: &["dag_diagram"],
            inputs: &["edges"],
            depends_on: &["foundation_graph_topsort"],
            alternatives: &[],
        },
        // vocab (2 tools) — vocabulary programs
        ToolMeta {
            name: "vocab_list",
            category: "vocab",
            highway_class: 3,
            outputs: &["vocabularies"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "vocab_skill_lookup",
            category: "vocab",
            highway_class: 3,
            outputs: &["matching_skill"],
            inputs: &["vocab_name"],
            depends_on: &[],
            alternatives: &["skill_search_by_tag"],
        },
        // watchtower (9 tools) — session monitoring
        ToolMeta {
            name: "watchtower_gemini_stats",
            category: "watchtower",
            highway_class: 3,
            outputs: &["gemini_stats"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "watchtower_unified",
            category: "watchtower",
            highway_class: 3,
            outputs: &["unified_report"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["watchtower_analyze"],
        },
        ToolMeta {
            name: "watchtower_telemetry_stats",
            category: "watchtower",
            highway_class: 3,
            outputs: &["telemetry_stats"],
            inputs: &[],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "watchtower_gemini_recent",
            category: "watchtower",
            highway_class: 3,
            outputs: &["recent_gemini_events"],
            inputs: &["limit"],
            depends_on: &[],
            alternatives: &["watchtower_recent"],
        },
        ToolMeta {
            name: "watchtower_recent",
            category: "watchtower",
            highway_class: 3,
            outputs: &["recent_events"],
            inputs: &["limit"],
            depends_on: &[],
            alternatives: &["watchtower_gemini_recent"],
        },
        ToolMeta {
            name: "watchtower_sessions_list",
            category: "watchtower",
            highway_class: 3,
            outputs: &["sessions"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["watchtower_active_sessions"],
        },
        ToolMeta {
            name: "watchtower_analyze",
            category: "watchtower",
            highway_class: 3,
            outputs: &["analysis_report"],
            inputs: &["session_id"],
            depends_on: &[],
            alternatives: &["watchtower_unified"],
        },
        ToolMeta {
            name: "watchtower_symbol_audit",
            category: "watchtower",
            highway_class: 3,
            outputs: &["symbol_audit"],
            inputs: &["session_id"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "watchtower_active_sessions",
            category: "watchtower",
            highway_class: 3,
            outputs: &["active_sessions"],
            inputs: &[],
            depends_on: &[],
            alternatives: &["watchtower_sessions_list"],
        },
        // wolfram (18 tools) — Wolfram Alpha computation API
        ToolMeta {
            name: "wolfram_finance",
            category: "wolfram",
            highway_class: 4,
            outputs: &["financial_data"],
            inputs: &["ticker_or_indicator"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "wolfram_query_with_assumption",
            category: "wolfram",
            highway_class: 4,
            outputs: &["result"],
            inputs: &["query", "assumption"],
            depends_on: &["wolfram_query"],
            alternatives: &[],
        },
        ToolMeta {
            name: "wolfram_chemistry",
            category: "wolfram",
            highway_class: 4,
            outputs: &["molecular_data"],
            inputs: &["compound_or_reaction"],
            depends_on: &[],
            alternatives: &["stem_chem_balance", "lab_react"],
        },
        ToolMeta {
            name: "wolfram_step_by_step",
            category: "wolfram",
            highway_class: 4,
            outputs: &["steps", "final_result"],
            inputs: &["expression"],
            depends_on: &[],
            alternatives: &["wolfram_query"],
        },
        ToolMeta {
            name: "wolfram_short_answer",
            category: "wolfram",
            highway_class: 4,
            outputs: &["answer"],
            inputs: &["input"],
            depends_on: &[],
            alternatives: &["wolfram_query", "wolfram_spoken_answer"],
        },
        ToolMeta {
            name: "wolfram_astronomy",
            category: "wolfram",
            highway_class: 4,
            outputs: &["astronomical_data"],
            inputs: &["object_or_calculation"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "wolfram_plot",
            category: "wolfram",
            highway_class: 4,
            outputs: &["plot_url", "data_points"],
            inputs: &["expression", "range"],
            depends_on: &[],
            alternatives: &["wolfram_image_result"],
        },
        ToolMeta {
            name: "wolfram_convert",
            category: "wolfram",
            highway_class: 4,
            outputs: &["converted_value"],
            inputs: &["value", "from_unit", "to_unit"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "wolfram_statistics",
            category: "wolfram",
            highway_class: 4,
            outputs: &["statistics_result"],
            inputs: &["operation", "data"],
            depends_on: &[],
            alternatives: &["measure_stats"],
        },
        ToolMeta {
            name: "wolfram_nutrition",
            category: "wolfram",
            highway_class: 4,
            outputs: &["nutrition_data"],
            inputs: &["food_item"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "wolfram_query_filtered",
            category: "wolfram",
            highway_class: 4,
            outputs: &["filtered_results"],
            inputs: &["query", "filter_type"],
            depends_on: &[],
            alternatives: &["wolfram_query"],
        },
        ToolMeta {
            name: "wolfram_image_result",
            category: "wolfram",
            highway_class: 4,
            outputs: &["image_urls"],
            inputs: &["query"],
            depends_on: &[],
            alternatives: &["wolfram_plot"],
        },
        ToolMeta {
            name: "wolfram_physics",
            category: "wolfram",
            highway_class: 4,
            outputs: &["physics_result"],
            inputs: &["phenomenon", "parameters"],
            depends_on: &[],
            alternatives: &["stem_phys_fma", "stem_phys_conservation"],
        },
        ToolMeta {
            name: "wolfram_spoken_answer",
            category: "wolfram",
            highway_class: 4,
            outputs: &["spoken_answer"],
            inputs: &["input"],
            depends_on: &[],
            alternatives: &["wolfram_short_answer"],
        },
        ToolMeta {
            name: "wolfram_datetime",
            category: "wolfram",
            highway_class: 4,
            outputs: &["parsed_datetime"],
            inputs: &["datetime_expression"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "wolfram_linguistics",
            category: "wolfram",
            highway_class: 4,
            outputs: &["linguistic_data"],
            inputs: &["word_or_phrase"],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "wolfram_query",
            category: "wolfram",
            highway_class: 4,
            outputs: &["result"],
            inputs: &["input"],
            depends_on: &[],
            alternatives: &["wolfram_short_answer", "wolfram_step_by_step"],
        },
        ToolMeta {
            name: "wolfram_data_lookup",
            category: "wolfram",
            highway_class: 4,
            outputs: &["data_value"],
            inputs: &["entity", "property"],
            depends_on: &[],
            alternatives: &[],
        },
        // Observatory Personalization (4 tools)
        ToolMeta {
            name: "observatory_personalize_get",
            category: "observatory",
            highway_class: 4,
            outputs: &["preferences"],
            inputs: &["profile"],
            depends_on: &[],
            alternatives: &["observatory_personalize_detect"],
        },
        ToolMeta {
            name: "observatory_personalize_set",
            category: "observatory",
            highway_class: 4,
            outputs: &["validated_preferences", "errors"],
            inputs: &[
                "quality",
                "theme",
                "cvd_mode",
                "explorer",
                "layout",
                "post_processing",
            ],
            depends_on: &[],
            alternatives: &[],
        },
        ToolMeta {
            name: "observatory_personalize_detect",
            category: "observatory",
            highway_class: 4,
            outputs: &["detected_preferences", "score", "reasoning"],
            inputs: &[
                "gpu_renderer",
                "device_pixel_ratio",
                "hardware_concurrency",
                "device_memory",
            ],
            depends_on: &[],
            alternatives: &["observatory_personalize_get"],
        },
        ToolMeta {
            name: "observatory_personalize_validate",
            category: "observatory",
            highway_class: 4,
            outputs: &["valid", "warnings", "errors"],
            inputs: &[
                "explorer",
                "quality",
                "theme",
                "cvd_mode",
                "layout",
                "post_processing",
            ],
            depends_on: &[],
            alternatives: &[],
        },
    ]
}

// ============================================================================
// Static Data: Routing Rules (stimulus → tool)
// ============================================================================

pub fn routing_rules() -> Vec<RoutingRule> {
    vec![
        // Computation reflexes
        RoutingRule {
            patterns: &[
                "compare strings",
                "string similarity",
                "edit distance",
                "levenshtein",
                "fuzzy match",
                "compare names",
                "name similarity",
                "compare similarity",
            ],
            tools: &["foundation_levenshtein"],
            mode: "single",
            confidence: 0.95,
            description: "String comparison via edit distance",
        },
        RoutingRule {
            patterns: &[
                "fuzzy search",
                "fuzzy match",
                "approximate match",
                "find similar",
            ],
            tools: &["foundation_fuzzy_search"],
            mode: "single",
            confidence: 0.92,
            description: "Fuzzy string matching against candidates",
        },
        RoutingRule {
            patterns: &["hash", "checksum", "sha256", "digest"],
            tools: &["foundation_sha256"],
            mode: "single",
            confidence: 0.95,
            description: "SHA-256 cryptographic hashing",
        },
        RoutingRule {
            patterns: &[
                "dependency graph",
                "topological sort",
                "dag sort",
                "execution order",
            ],
            tools: &["foundation_graph_topsort"],
            mode: "single",
            confidence: 0.93,
            description: "Topological sorting of directed acyclic graph",
        },
        RoutingRule {
            patterns: &["concept search", "naming variants", "search all forms"],
            tools: &["foundation_concept_grep"],
            mode: "single",
            confidence: 0.90,
            description: "Multi-variant concept expansion for search",
        },
        RoutingRule {
            patterns: &["calculate", "compute", "numeric", "math", "equation"],
            tools: &["wolfram_calculate"],
            mode: "single",
            confidence: 0.85,
            description: "Numeric computation via Wolfram Alpha",
        },
        RoutingRule {
            patterns: &[
                "t-test",
                "regression",
                "entropy",
                "statistical test",
                "bayesian",
            ],
            tools: &["wolfram_calculate"],
            mode: "single",
            confidence: 0.82,
            description: "Statistical analysis",
        },
        // Domain reflexes
        RoutingRule {
            patterns: &[
                "drug safety",
                "adverse event",
                "signal detection",
                "pharmacovigilance",
                "drug event",
            ],
            tools: &[
                "faers_drug_events",
                "pv_signal_complete",
                "guardian_evaluate_pv",
                "vigilance_risk_score",
            ],
            mode: "sequential",
            confidence: 0.90,
            description: "Full PV signal analysis pipeline",
        },
        RoutingRule {
            patterns: &["fda data", "faers", "adverse event database"],
            tools: &["faers_search"],
            mode: "single",
            confidence: 0.92,
            description: "FDA FAERS adverse event database search",
        },
        RoutingRule {
            patterns: &["regulatory", "ich guideline", "cioms", "ema", "compliance"],
            tools: &["guidelines_search", "ich_lookup"],
            mode: "parallel",
            confidence: 0.88,
            description: "Regulatory guideline search",
        },
        RoutingRule {
            patterns: &[
                "fda guidance",
                "fda draft",
                "guidance document",
                "fda recommendation",
            ],
            tools: &["fda_guidance_search"],
            mode: "parallel",
            confidence: 0.90,
            description: "FDA guidance document search",
        },
        RoutingRule {
            patterns: &["causality", "naranjo", "who-umc", "causal assessment"],
            tools: &["pv_naranjo_quick"],
            mode: "single",
            confidence: 0.90,
            description: "Causality assessment scoring",
        },
        RoutingRule {
            patterns: &["cross-domain", "transfer confidence", "domain mapping"],
            tools: &["stem_taxonomy", "stem_confidence_combine"],
            mode: "sequential",
            confidence: 0.85,
            description: "Cross-domain concept transfer with confidence",
        },
        RoutingRule {
            patterns: &[
                "chemistry model",
                "arrhenius",
                "michaelis",
                "gibbs",
                "dose-response",
            ],
            tools: &["chemistry_pv_mappings"],
            mode: "single",
            confidence: 0.87,
            description: "Chemistry-to-PV equation mapping",
        },
        RoutingRule {
            patterns: &[
                "epidemiology",
                "relative risk",
                "odds ratio",
                "incidence rate",
                "prevalence",
                "attributable risk",
                "nnt",
                "nnh",
                "kaplan meier",
                "survival analysis",
                "smr",
                "standardized mortality",
            ],
            tools: &["epi_pv_mappings"],
            mode: "single",
            confidence: 0.88,
            description: "Epidemiology-to-PV equation mapping catalog",
        },
        RoutingRule {
            patterns: &["2x2 table", "contingency table", "exposed unexposed"],
            tools: &[
                "epi_relative_risk",
                "epi_odds_ratio",
                "epi_attributable_risk",
                "epi_nnt_nnh",
            ],
            mode: "parallel",
            confidence: 0.92,
            description: "Full 2x2 contingency table analysis",
        },
        RoutingRule {
            patterns: &["survival curve", "kaplan meier", "time to event"],
            tools: &["epi_kaplan_meier"],
            mode: "single",
            confidence: 0.90,
            description: "Product-limit survival estimation with Greenwood CI",
        },
        RoutingRule {
            patterns: &[
                "web search",
                "current information",
                "recent news",
                "look up",
            ],
            tools: &["perplexity_search"],
            mode: "single",
            confidence: 0.88,
            description: "AI-powered web search",
        },
        RoutingRule {
            patterns: &["validate", "quality check", "l1-l5", "verification"],
            tools: &["validation_check"],
            mode: "single",
            confidence: 0.85,
            description: "Quick L1-L2 validation",
        },
        // Primitive reflexes
        RoutingRule {
            patterns: &[
                "decompose",
                "primitive composition",
                "t1 grounding",
                "what primitives",
            ],
            tools: &["lex_primitiva_composition"],
            mode: "single",
            confidence: 0.92,
            description: "T1 primitive decomposition of a concept",
        },
        RoutingRule {
            patterns: &["classify tier", "what tier", "t1 t2 t3"],
            tools: &["lex_primitiva_tier"],
            mode: "single",
            confidence: 0.90,
            description: "Tier classification (T1/T2-P/T2-C/T3)",
        },
        RoutingRule {
            patterns: &[
                "validate term",
                "literature check",
                "ich glossary",
                "pubmed",
            ],
            tools: &["primitive_validate"],
            mode: "single",
            confidence: 0.88,
            description: "Term validation against ICH, BioOntology, PubMed",
        },
        // Session lifecycle
        RoutingRule {
            patterns: &[
                "health check",
                "session health",
                "system status",
                "homeostasis",
            ],
            tools: &["guardian_homeostasis_tick"],
            mode: "single",
            confidence: 0.90,
            description: "Guardian homeostasis control loop tick",
        },
        RoutingRule {
            patterns: &["save artifact", "persist work", "checkpoint"],
            tools: &["brain_artifact_save"],
            mode: "single",
            confidence: 0.88,
            description: "Save work as brain artifact",
        },
        RoutingRule {
            patterns: &["remember", "persist learning", "save knowledge"],
            tools: &["implicit_set"],
            mode: "single",
            confidence: 0.85,
            description: "Persist reusable knowledge",
        },
        // Workflow chains
        RoutingRule {
            patterns: &["drug safety review", "full safety analysis"],
            tools: &[
                "faers_drug_events",
                "faers_disproportionality",
                "pv_naranjo_quick",
                "guidelines_search",
            ],
            mode: "sequential",
            confidence: 0.88,
            description: "Complete drug safety review chain",
        },
        RoutingRule {
            patterns: &["concept grounding", "ground concept", "formal grounding"],
            tools: &[
                "lex_primitiva_composition",
                "primitive_validate",
                "stem_taxonomy",
                "chemistry_pv_mappings",
            ],
            mode: "sequential",
            confidence: 0.85,
            description: "Full concept grounding chain",
        },
        RoutingRule {
            patterns: &["signal detection pipeline", "multi-stage detection"],
            tools: &["signal_theory_pipeline"],
            mode: "single",
            confidence: 0.90,
            description: "Multi-stage signal detection pipeline",
        },
        RoutingRule {
            patterns: &["parallel detection", "dual detector", "both and either"],
            tools: &["signal_theory_parallel"],
            mode: "single",
            confidence: 0.88,
            description: "Parallel detector evaluation (AND/OR)",
        },
        // AI Engineering Bible tools
        RoutingRule {
            patterns: &[
                "drift detection",
                "distribution drift",
                "data drift",
                "compare distributions",
                "assess drift",
                "detect drift",
            ],
            tools: &["drift_detect"],
            mode: "single",
            confidence: 0.90,
            description: "Composite drift detection (KS + PSI + JSD)",
        },
        RoutingRule {
            patterns: &["rate limit", "throttle", "token bucket", "sliding window"],
            tools: &["rate_limit_token_bucket"],
            mode: "single",
            confidence: 0.88,
            description: "Token bucket rate limiting",
        },
        RoutingRule {
            patterns: &[
                "rank fusion",
                "merge rankings",
                "combine results",
                "merge results",
            ],
            tools: &["rank_fusion_rrf"],
            mode: "single",
            confidence: 0.88,
            description: "Reciprocal rank fusion (merge multiple ranked lists)",
        },
        RoutingRule {
            patterns: &[
                "security posture",
                "compliance check",
                "soc2",
                "hipaa",
                "gdpr",
            ],
            tools: &["security_posture_assess"],
            mode: "single",
            confidence: 0.85,
            description: "Security compliance scorecard",
        },
        RoutingRule {
            patterns: &["latency", "throughput", "observability", "p99"],
            tools: &["observability_record_latency"],
            mode: "single",
            confidence: 0.82,
            description: "Inference latency and throughput tracking",
        },
        // Routing self-reference (meta)
        RoutingRule {
            patterns: &["routing", "tool selection", "which tool", "what tool"],
            tools: &["tool_route"],
            mode: "single",
            confidence: 0.90,
            description: "Deterministic tool routing engine",
        },
        RoutingRule {
            patterns: &["workflow", "pipeline", "chain", "multi-step"],
            tools: &["tool_chain"],
            mode: "single",
            confidence: 0.85,
            description: "Named workflow chain lookup",
        },
        RoutingRule {
            patterns: &["dependency order", "execution plan", "dag plan"],
            tools: &["tool_dag"],
            mode: "single",
            confidence: 0.87,
            description: "DAG execution planning with topological sort",
        },
    ]
}

// ============================================================================
// Static Data: Named Workflow Chains
// ============================================================================

pub fn workflow_chains() -> Vec<WorkflowChain> {
    vec![
        WorkflowChain {
            name: "pv_signal_analysis",
            description: "Full pharmacovigilance signal detection pipeline",
            steps: &[
                ChainStep {
                    tools: &["faers_search"],
                    data_flow: "query → cases",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["pv_signal_complete"],
                    data_flow: "cases → 2x2 table → 5 algorithms",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["guardian_evaluate_pv"],
                    data_flow: "signal data → risk assessment",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["vigilance_risk_score"],
                    data_flow: "risk factors → scored outcome",
                    mode: "sequential",
                },
            ],
        },
        WorkflowChain {
            name: "concept_grounding",
            description: "Ground a concept to T1 primitives with validation",
            steps: &[
                ChainStep {
                    tools: &["lex_primitiva_composition"],
                    data_flow: "concept → T1 primitives",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["primitive_validate"],
                    data_flow: "term → ICH/BioOntology/PubMed check",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["stem_taxonomy"],
                    data_flow: "→ cross-domain trait lookup",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["chemistry_pv_mappings"],
                    data_flow: "→ applicable chemistry models",
                    mode: "sequential",
                },
            ],
        },
        WorkflowChain {
            name: "drug_safety_review",
            description: "Complete drug safety review with causality assessment",
            steps: &[
                ChainStep {
                    tools: &["faers_drug_events"],
                    data_flow: "drug name → top events",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["faers_disproportionality"],
                    data_flow: "drug+event → PRR/ROR analysis",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["pv_naranjo_quick"],
                    data_flow: "case data → causality score",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["guidelines_search"],
                    data_flow: "drug/event → regulatory guidance",
                    mode: "sequential",
                },
            ],
        },
        WorkflowChain {
            name: "crate_quality_check",
            description: "Quality validation pipeline for a Rust crate",
            steps: &[
                ChainStep {
                    tools: &["validation_run"],
                    data_flow: "crate path → L1-L5 results",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["skill_validate"],
                    data_flow: "target → Diamond v2 compliance",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["regulatory_primitives_audit"],
                    data_flow: "→ FDA/ICH consistency check",
                    mode: "sequential",
                },
            ],
        },
        WorkflowChain {
            name: "distribution_drift_analysis",
            description: "Statistical drift detection with health monitoring",
            steps: &[
                ChainStep {
                    tools: &["drift_detect"],
                    data_flow: "two distributions → composite KS+PSI+JSD",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["guardian_homeostasis_tick"],
                    data_flow: "→ feed signal to control loop",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["cytokine_emit"],
                    data_flow: "if drift → emit IFN signal",
                    mode: "sequential",
                },
            ],
        },
        WorkflowChain {
            name: "parallel_evidence_gathering",
            description: "Parallel tool calls with grounded confidence merging",
            steps: &[
                ChainStep {
                    tools: &["grounded_evidence_new"],
                    data_flow: "hypothesis → evidence chain",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["faers_search", "guidelines_search", "perplexity_search"],
                    data_flow: "→ parallel evidence sources",
                    mode: "parallel",
                },
                ChainStep {
                    tools: &["grounded_evidence_step"],
                    data_flow: "each result → chain step (3x)",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["grounded_evidence_get"],
                    data_flow: "→ chain summary with confidence",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["grounded_require"],
                    data_flow: "→ gate: confidence >= 0.8",
                    mode: "sequential",
                },
            ],
        },
        WorkflowChain {
            name: "signal_detection_algebra",
            description: "Multi-stage signal detection with algebra composition",
            steps: &[
                ChainStep {
                    tools: &["signal_theory_detect"],
                    data_flow: "observed/expected → ratio + strength",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["signal_theory_pipeline"],
                    data_flow: "→ multi-stage screening + confirmation",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["signal_theory_decision_matrix"],
                    data_flow: "→ SDT metrics (d', bias)",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["signal_theory_conservation_check"],
                    data_flow: "→ conservation law verification",
                    mode: "sequential",
                },
            ],
        },
        WorkflowChain {
            name: "epidemiological_impact_assessment",
            description: "Full epidemiological analysis: measures of association → impact → PV transfer",
            steps: &[
                ChainStep {
                    tools: &["epi_relative_risk", "epi_odds_ratio"],
                    data_flow: "2x2 table → RR + OR with CIs",
                    mode: "parallel",
                },
                ChainStep {
                    tools: &[
                        "epi_attributable_risk",
                        "epi_attributable_fraction",
                        "epi_population_af",
                    ],
                    data_flow: "→ impact measures (AR, AF, PAF)",
                    mode: "parallel",
                },
                ChainStep {
                    tools: &["epi_nnt_nnh"],
                    data_flow: "→ clinical significance (NNT/NNH)",
                    mode: "sequential",
                },
                ChainStep {
                    tools: &["epi_pv_mappings"],
                    data_flow: "→ PV transfer confidence catalog",
                    mode: "sequential",
                },
            ],
        },
    ]
}

// ============================================================================
// Export: Route Index for Hooks
// ============================================================================

/// Export routing rules as a JSON lookup file for hook consumption.
/// Single source of truth: this function + `routing_rules()` drive everything.
/// Called by `nexcore-mcp --export-route-index`.
pub fn export_route_index() -> serde_json::Value {
    let rules = routing_rules();
    let chains = workflow_chains();

    // Build keyword → suggestion map (what the hook needs)
    let mut keyword_map: Vec<serde_json::Value> = Vec::new();
    for rule in &rules {
        for pattern in rule.patterns {
            keyword_map.push(json!({
                "keyword": pattern,
                "tools": rule.tools,
                "mode": rule.mode,
                "confidence": rule.confidence,
                "description": rule.description,
            }));
        }
    }

    // Build chain summary
    let chain_names: Vec<&str> = chains.iter().map(|c| c.name).collect();

    json!({
        "version": 1,
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "rules_count": rules.len(),
        "keywords_count": keyword_map.len(),
        "keywords": keyword_map,
        "chains": chain_names,
    })
}

// ============================================================================
// Tool Implementation: tool_route
// ============================================================================

/// Route a stimulus to deterministic tool selection.
pub fn tool_route(params: ToolRouteParams) -> Result<CallToolResult, McpError> {
    let rules = routing_rules();
    let limit = params.limit.unwrap_or(3);
    let stimulus_lower = params.stimulus.to_lowercase();
    let stimulus_words: HashSet<&str> = stimulus_lower.split_whitespace().collect();

    // Score each rule: two-tier matching
    // 1. Exact phrase match (substring) → full weight
    // 2. Word overlap (any pattern word in stimulus) → partial weight
    let mut scored: Vec<(f64, &RoutingRule)> = rules
        .iter()
        .filter_map(|rule| {
            let mut total_score = 0.0_f64;
            let mut any_match = false;

            for pattern in rule.patterns {
                let pattern_lower = pattern.to_lowercase();
                if stimulus_lower.contains(&pattern_lower) {
                    // Exact phrase match: full credit
                    total_score += 1.0;
                    any_match = true;
                } else {
                    // Word overlap: partial credit based on word intersection
                    let pattern_words: HashSet<&str> = pattern_lower.split_whitespace().collect();
                    let overlap = stimulus_words.intersection(&pattern_words).count();
                    if overlap > 0 {
                        let word_score = overlap as f64 / pattern_words.len() as f64;
                        if word_score >= 0.5 {
                            // At least half the pattern words match
                            total_score += word_score * 0.7; // Discount for partial
                            any_match = true;
                        }
                    }
                }
            }

            if any_match {
                let normalized = total_score / rule.patterns.len() as f64;
                let final_score = rule.confidence * normalized;
                Some((final_score, rule))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);

    let matches: Vec<_> = scored
        .iter()
        .enumerate()
        .map(|(i, (score, rule))| {
            // Collect matched patterns (both exact and word-overlap)
            let matched_patterns: Vec<&&str> = rule
                .patterns
                .iter()
                .filter(|p| {
                    let pl = p.to_lowercase();
                    if stimulus_lower.contains(&pl) {
                        return true;
                    }
                    let pw: HashSet<&str> = pl.split_whitespace().collect();
                    let overlap = stimulus_words.intersection(&pw).count();
                    overlap as f64 / pw.len() as f64 >= 0.5
                })
                .collect();
            json!({
                "rank": i + 1,
                "score": format!("{:.3}", score),
                "confidence": rule.confidence,
                "tools": rule.tools,
                "mode": rule.mode,
                "description": rule.description,
                "patterns_matched": matched_patterns,
            })
        })
        .collect();

    let result = json!({
        "stimulus": params.stimulus,
        "matches_found": matches.len(),
        "routes": matches,
        "deterministic": matches.len() == 1 || (!matches.is_empty() && scored[0].0 > 0.8),
        "routing_primitive": "σ(Sequence) + μ(Mapping) + κ(Comparison)",
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Tool Implementation: tool_dag
// ============================================================================

/// Build a dependency DAG for a set of tools and return topological execution plan.
pub fn tool_dag(params: ToolDagParams) -> Result<CallToolResult, McpError> {
    let registry = tool_registry();
    let reg_map: HashMap<&str, &ToolMeta> = registry.iter().map(|t| (t.name, t)).collect();
    let include_transitive = params.transitive.unwrap_or(true);

    // Collect all tools needed (including transitive deps)
    let mut needed: BTreeSet<String> = params.tools.iter().cloned().collect();
    if include_transitive {
        let mut queue: VecDeque<String> = params.tools.iter().cloned().collect();
        while let Some(tool) = queue.pop_front() {
            if let Some(meta) = reg_map.get(tool.as_str()) {
                for dep in meta.depends_on {
                    let dep_s = dep.to_string();
                    if needed.insert(dep_s.clone()) {
                        queue.push_back(dep_s);
                    }
                }
            }
        }
    }

    // Build edges from dependency declarations
    let mut edges: Vec<(String, String)> = Vec::new();
    for tool_name in &needed {
        if let Some(meta) = reg_map.get(tool_name.as_str()) {
            for dep in meta.depends_on {
                if needed.contains(*dep) {
                    edges.push((dep.to_string(), tool_name.clone()));
                }
            }
        }
    }

    // Also infer edges from data flow: if tool A outputs type X and tool B inputs type X
    let mut output_producers: HashMap<&str, Vec<&str>> = HashMap::new();
    for tool_name in &needed {
        if let Some(meta) = reg_map.get(tool_name.as_str()) {
            for out in meta.outputs {
                output_producers.entry(out).or_default().push(meta.name);
            }
        }
    }

    // Topological sort using Kahn's algorithm
    let mut in_degree: BTreeMap<String, usize> = BTreeMap::new();
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();

    for tool in &needed {
        in_degree.entry(tool.clone()).or_insert(0);
    }
    for (from, to) in &edges {
        *in_degree.entry(to.clone()).or_insert(0) += 1;
        adj.entry(from.clone()).or_default().push(to.clone());
    }

    // BFS for topsort + level assignment
    let mut queue: VecDeque<String> = in_degree
        .iter()
        .filter(|&(_, deg)| *deg == 0)
        .map(|(name, _)| name.clone())
        .collect();

    let mut sorted: Vec<String> = Vec::new();
    let mut levels: Vec<Vec<String>> = Vec::new();
    let mut level_map: HashMap<String, usize> = HashMap::new();

    // Assign levels
    let mut current_level: Vec<String> = queue.iter().cloned().collect();
    while !current_level.is_empty() {
        let level_idx = levels.len();
        for tool in &current_level {
            level_map.insert(tool.clone(), level_idx);
        }
        levels.push(current_level.clone());
        sorted.extend(current_level.iter().cloned());

        let mut next_level = Vec::new();
        for tool in &current_level {
            if let Some(neighbors) = adj.get(tool) {
                for neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            next_level.push(neighbor.clone());
                        }
                    }
                }
            }
        }
        current_level = next_level;
    }

    let has_cycle = sorted.len() < needed.len();

    // Build node details
    let nodes: Vec<_> = needed
        .iter()
        .map(|name| {
            let meta = reg_map.get(name.as_str());
            json!({
                "name": name,
                "category": meta.map(|m| m.category).unwrap_or("unknown"),
                "highway_class": meta.map(|m| m.highway_class).unwrap_or(0),
                "depends_on": meta.map(|m| m.depends_on).unwrap_or(&[]),
                "execution_level": level_map.get(name).unwrap_or(&0),
                "in_registry": meta.is_some(),
            })
        })
        .collect();

    let level_plan: Vec<_> = levels
        .iter()
        .enumerate()
        .map(|(i, tools)| {
            let classes: Vec<u8> = tools
                .iter()
                .filter_map(|t| reg_map.get(t.as_str()).map(|m| m.highway_class))
                .collect();
            let max_class = classes.iter().max().copied().unwrap_or(0);
            json!({
                "level": i,
                "tools": tools,
                "mode": if tools.len() > 1 { "parallel" } else { "sequential" },
                "max_highway_class": max_class,
                "estimated_sla_ms": match max_class { 1 => 10, 2 => 100, 3 => 500, 4 => 5000, _ => 0 },
            })
        })
        .collect();

    let result = json!({
        "tools_requested": params.tools.len(),
        "tools_total": needed.len(),
        "transitive_deps_added": needed.len() - params.tools.len(),
        "edges": edges.iter().map(|(a, b)| json!({"from": a, "to": b})).collect::<Vec<_>>(),
        "topological_order": sorted,
        "execution_levels": level_plan,
        "total_levels": levels.len(),
        "has_cycle": has_cycle,
        "nodes": nodes,
        "dag_primitive": "σ(Sequence) + →(Causality) + ∂(Boundary)",
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Tool Implementation: tool_deps
// ============================================================================

/// Look up dependencies and dependents for a specific tool.
pub fn tool_deps(params: ToolDepsParams) -> Result<CallToolResult, McpError> {
    let registry = tool_registry();
    let reg_map: HashMap<&str, &ToolMeta> = registry.iter().map(|t| (t.name, t)).collect();

    let tool_name = params.tool.as_str();

    let meta = reg_map.get(tool_name);

    // Find dependents (tools that depend on this one)
    let dependents: Vec<&str> = registry
        .iter()
        .filter(|t| t.depends_on.contains(&tool_name))
        .map(|t| t.name)
        .collect();

    // Find tools that produce the same outputs (alternatives)
    let alternatives: Vec<&str> = meta.map(|m| m.alternatives.to_vec()).unwrap_or_default();

    // Find tools that consume our outputs (data flow dependents)
    let data_dependents: Vec<&str> = if let Some(m) = meta {
        registry
            .iter()
            .filter(|other| {
                other.name != tool_name
                    && other.inputs.iter().any(|input| m.outputs.contains(input))
            })
            .map(|t| t.name)
            .collect()
    } else {
        vec![]
    };

    let result = if let Some(m) = meta {
        json!({
            "tool": tool_name,
            "found": true,
            "category": m.category,
            "highway_class": m.highway_class,
            "highway_sla_ms": match m.highway_class { 1 => 10, 2 => 100, 3 => 500, 4 => 5000, _ => 0 },
            "inputs": m.inputs,
            "outputs": m.outputs,
            "depends_on": m.depends_on,
            "hard_dependents": dependents,
            "data_dependents": data_dependents,
            "alternatives": alternatives,
            "total_upstream": m.depends_on.len(),
            "total_downstream": dependents.len() + data_dependents.len(),
        })
    } else {
        json!({
            "tool": tool_name,
            "found": false,
            "note": "Tool not in routing registry. Use tool_route with a stimulus to discover tools.",
        })
    };

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Tool Implementation: tool_chain
// ============================================================================

/// Get a named workflow chain with full execution plan.
pub fn tool_chain(params: ToolChainParams) -> Result<CallToolResult, McpError> {
    let chains = workflow_chains();

    if params.chain == "list" {
        let list: Vec<_> = chains
            .iter()
            .map(|c| {
                let tool_count: usize = c.steps.iter().map(|s| s.tools.len()).sum();
                json!({
                    "name": c.name,
                    "description": c.description,
                    "steps": c.steps.len(),
                    "total_tools": tool_count,
                })
            })
            .collect();

        let result = json!({
            "chains_available": list.len(),
            "chains": list,
        });

        return Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]));
    }

    let chain = chains.iter().find(|c| c.name == params.chain);

    if let Some(c) = chain {
        let registry = tool_registry();
        let reg_map: HashMap<&str, &ToolMeta> = registry.iter().map(|t| (t.name, t)).collect();

        let steps: Vec<_> = c
            .steps
            .iter()
            .enumerate()
            .map(|(i, step)| {
                let tools_detail: Vec<_> = step
                    .tools
                    .iter()
                    .map(|t| {
                        let meta = reg_map.get(t);
                        json!({
                            "name": t,
                            "highway_class": meta.map(|m| m.highway_class).unwrap_or(0),
                            "in_registry": meta.is_some(),
                        })
                    })
                    .collect();

                let max_class = step
                    .tools
                    .iter()
                    .filter_map(|t| reg_map.get(t).map(|m| m.highway_class))
                    .max()
                    .unwrap_or(0);

                json!({
                    "step": i + 1,
                    "tools": tools_detail,
                    "data_flow": step.data_flow,
                    "mode": step.mode,
                    "max_highway_class": max_class,
                    "estimated_sla_ms": match max_class { 1 => 10, 2 => 100, 3 => 500, 4 => 5000, _ => 0 },
                })
            })
            .collect();

        let total_tools: usize = c.steps.iter().map(|s| s.tools.len()).sum();
        let has_parallel = c.steps.iter().any(|s| s.mode == "parallel");
        let max_sla: u32 = c
            .steps
            .iter()
            .map(|s| {
                s.tools
                    .iter()
                    .filter_map(|t| reg_map.get(t).map(|m| m.highway_class))
                    .max()
                    .unwrap_or(0)
            })
            .map(|class| match class {
                1 => 10,
                2 => 100,
                3 => 500,
                4 => 5000,
                _ => 0,
            })
            .sum();

        let result = json!({
            "chain": c.name,
            "description": c.description,
            "steps": steps,
            "total_steps": c.steps.len(),
            "total_tools": total_tools,
            "has_parallel_steps": has_parallel,
            "estimated_total_sla_ms": max_sla,
            "execution_primitive": "σ(Sequence) + →(Causality)",
        });

        Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]))
    } else {
        let available: Vec<&str> = chains.iter().map(|c| c.name).collect();
        Err(McpError::invalid_params(
            format!(
                "Unknown chain '{}'. Available: {:?}. Use chain='list' to see all.",
                params.chain, available
            ),
            None,
        ))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn extract_json(r: &CallToolResult) -> serde_json::Value {
        let content_json = serde_json::to_value(&r.content[0]).expect("serialize content");
        let text = content_json["text"].as_str().expect("text field");
        serde_json::from_str(text).expect("parse tool output JSON")
    }

    #[test]
    fn test_route_string_comparison() {
        let result = tool_route(ToolRouteParams {
            stimulus: "string similarity edit distance".to_string(),
            limit: None,
        })
        .unwrap();
        let j = extract_json(&result);
        assert!(j["matches_found"].as_u64().unwrap_or(0) > 0);
        let first_tools = &j["routes"][0]["tools"];
        assert!(first_tools.as_array().unwrap().iter().any(|t| {
            let s = t.as_str().unwrap_or("");
            s.contains("levenshtein") || s.contains("edit_distance") || s.contains("fuzzy")
        }));
    }

    #[test]
    fn test_route_drug_safety() {
        let result = tool_route(ToolRouteParams {
            stimulus: "drug safety signal detection".to_string(),
            limit: None,
        })
        .unwrap();
        let j = extract_json(&result);
        assert!(j["matches_found"].as_u64().unwrap_or(0) > 0);
        // Should find PV signal analysis chain
        let has_pv = j["routes"].as_array().unwrap_or(&vec![]).iter().any(|r| {
            r["tools"].as_array().unwrap_or(&vec![]).iter().any(|t| {
                let s = t.as_str().unwrap_or("");
                s.contains("pv_signal") || s.contains("faers")
            })
        });
        assert!(has_pv, "Should route to PV tools");
    }

    #[test]
    fn test_route_no_match() {
        let result = tool_route(ToolRouteParams {
            stimulus: "xyzzy plugh nothing matches this".to_string(),
            limit: None,
        })
        .unwrap();
        let j = extract_json(&result);
        assert_eq!(j["matches_found"].as_u64().unwrap_or(99), 0);
    }

    #[test]
    fn test_dag_simple_chain() {
        let result = tool_dag(ToolDagParams {
            tools: vec![
                "grounded_compose".to_string(),
                "grounded_uncertain".to_string(),
            ],
            transitive: Some(false),
        })
        .unwrap();
        let j = extract_json(&result);
        assert_eq!(j["tools_total"].as_u64().unwrap(), 2);
        // grounded_compose depends on grounded_uncertain
        let edges = j["edges"].as_array().unwrap();
        assert!(
            edges.iter().any(|e| {
                e["from"].as_str() == Some("grounded_uncertain")
                    && e["to"].as_str() == Some("grounded_compose")
            }),
            "Should have edge from uncertain to compose"
        );
        assert!(!j["has_cycle"].as_bool().unwrap());
    }

    #[test]
    fn test_dag_transitive() {
        let result = tool_dag(ToolDagParams {
            tools: vec!["grounded_compose".to_string()],
            transitive: Some(true),
        })
        .unwrap();
        let j = extract_json(&result);
        // Should pull in grounded_uncertain transitively
        assert!(
            j["tools_total"].as_u64().unwrap() >= 2,
            "Should include transitive dep grounded_uncertain"
        );
        assert!(j["transitive_deps_added"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn test_dag_parallel_levels() {
        let result = tool_dag(ToolDagParams {
            tools: vec![
                "foundation_levenshtein".to_string(),
                "foundation_sha256".to_string(),
                "pv_signal_complete".to_string(),
            ],
            transitive: Some(false),
        })
        .unwrap();
        let j = extract_json(&result);
        // All independent — should be 1 level with 3 parallel tools
        let levels = j["execution_levels"].as_array().unwrap();
        assert_eq!(
            levels.len(),
            1,
            "All independent tools should be in 1 level"
        );
        let level_0_tools = levels[0]["tools"].as_array().unwrap();
        assert_eq!(level_0_tools.len(), 3);
        assert_eq!(levels[0]["mode"].as_str().unwrap(), "parallel");
    }

    #[test]
    fn test_dag_sequential_chain() {
        let result = tool_dag(ToolDagParams {
            tools: vec![
                "grounded_evidence_new".to_string(),
                "grounded_evidence_step".to_string(),
                "grounded_evidence_get".to_string(),
            ],
            transitive: Some(false),
        })
        .unwrap();
        let j = extract_json(&result);
        let levels = j["execution_levels"].as_array().unwrap();
        // evidence_new has no deps, evidence_step and evidence_get both depend on evidence_new
        assert!(
            levels.len() >= 2,
            "Should have at least 2 levels for evidence chain"
        );
        // First level should contain evidence_new
        let l0_tools: Vec<&str> = levels[0]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|t| t.as_str())
            .collect();
        assert!(l0_tools.contains(&"grounded_evidence_new"));
    }

    #[test]
    fn test_deps_known_tool() {
        let result = tool_deps(ToolDepsParams {
            tool: "grounded_uncertain".to_string(),
        })
        .unwrap();
        let j = extract_json(&result);
        assert!(j["found"].as_bool().unwrap());
        assert_eq!(j["category"].as_str().unwrap(), "grounded");
        assert_eq!(j["highway_class"].as_u64().unwrap(), 3);
        // grounded_compose and grounded_require depend on us
        let deps = j["hard_dependents"].as_array().unwrap();
        let dep_names: Vec<&str> = deps.iter().filter_map(|d| d.as_str()).collect();
        assert!(dep_names.contains(&"grounded_compose"));
        assert!(dep_names.contains(&"grounded_require"));
    }

    #[test]
    fn test_deps_unknown_tool() {
        let result = tool_deps(ToolDepsParams {
            tool: "nonexistent_tool".to_string(),
        })
        .unwrap();
        let j = extract_json(&result);
        assert!(!j["found"].as_bool().unwrap());
    }

    #[test]
    fn test_chain_list() {
        let result = tool_chain(ToolChainParams {
            chain: "list".to_string(),
        })
        .unwrap();
        let j = extract_json(&result);
        assert!(j["chains_available"].as_u64().unwrap() >= 7);
    }

    #[test]
    fn test_chain_pv_signal_analysis() {
        let result = tool_chain(ToolChainParams {
            chain: "pv_signal_analysis".to_string(),
        })
        .unwrap();
        let j = extract_json(&result);
        assert_eq!(j["chain"].as_str().unwrap(), "pv_signal_analysis");
        assert!(j["total_steps"].as_u64().unwrap() >= 4);
        assert!(j["total_tools"].as_u64().unwrap() >= 4);
        // Check data flow is described
        let steps = j["steps"].as_array().unwrap();
        assert!(!steps[0]["data_flow"].as_str().unwrap().is_empty());
    }

    #[test]
    fn test_chain_parallel_evidence() {
        let result = tool_chain(ToolChainParams {
            chain: "parallel_evidence_gathering".to_string(),
        })
        .unwrap();
        let j = extract_json(&result);
        assert!(j["has_parallel_steps"].as_bool().unwrap());
        // Step 2 should be parallel with 3 tools
        let steps = j["steps"].as_array().unwrap();
        let parallel_step = steps
            .iter()
            .find(|s| s["mode"].as_str() == Some("parallel"));
        assert!(parallel_step.is_some(), "Should have a parallel step");
        let p = parallel_step.unwrap();
        assert!(p["tools"].as_array().unwrap().len() >= 3);
    }

    #[test]
    fn test_chain_unknown() {
        let result = tool_chain(ToolChainParams {
            chain: "nonexistent_chain".to_string(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_route_deterministic_flag() {
        let result = tool_route(ToolRouteParams {
            stimulus: "sha256 hash".to_string(),
            limit: None,
        })
        .unwrap();
        let j = extract_json(&result);
        // High confidence single match should be deterministic
        assert!(j["deterministic"].as_bool().unwrap_or(false));
    }

    #[test]
    fn test_deps_data_dependents() {
        let result = tool_deps(ToolDepsParams {
            tool: "pv_signal_complete".to_string(),
        })
        .unwrap();
        let j = extract_json(&result);
        assert!(j["found"].as_bool().unwrap());
        // guardian_evaluate_pv has a hard dependency on pv_signal_complete
        let hard_deps = j["hard_dependents"].as_array().unwrap();
        let dep_names: Vec<&str> = hard_deps.iter().filter_map(|d| d.as_str()).collect();
        assert!(dep_names.contains(&"guardian_evaluate_pv"));
    }

    #[test]
    fn test_registry_completeness() {
        let registry = tool_registry();
        // Verify we have tools across all 4 highway classes
        let classes: HashSet<u8> = registry.iter().map(|t| t.highway_class).collect();
        assert!(classes.contains(&1), "Should have Class I tools");
        assert!(classes.contains(&2), "Should have Class II tools");
        assert!(classes.contains(&3), "Should have Class III tools");
        assert!(classes.contains(&4), "Should have Class IV tools");
    }

    #[test]
    fn test_route_concept_grounding() {
        let result = tool_route(ToolRouteParams {
            stimulus: "decompose a concept to T1 primitives".to_string(),
            limit: None,
        })
        .unwrap();
        let j = extract_json(&result);
        assert!(j["matches_found"].as_u64().unwrap_or(0) > 0);
        let has_lex = j["routes"].as_array().unwrap_or(&vec![]).iter().any(|r| {
            r["tools"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .any(|t| t.as_str().unwrap_or("").contains("lex_primitiva"))
        });
        assert!(has_lex, "Should route to lex_primitiva tools");
    }

    #[test]
    fn test_route_drift_detection() {
        let result = tool_route(ToolRouteParams {
            stimulus: "assess drift between two distributions".to_string(),
            limit: None,
        })
        .unwrap();
        let j = extract_json(&result);
        assert!(j["matches_found"].as_u64().unwrap_or(0) > 0);
        let has_drift = j["routes"].as_array().unwrap_or(&vec![]).iter().any(|r| {
            r["tools"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .any(|t| t.as_str().unwrap_or("").contains("drift"))
        });
        assert!(has_drift, "Should route to drift detection tools");
    }

    #[test]
    fn test_route_rate_limiting() {
        let result = tool_route(ToolRouteParams {
            stimulus: "I need to rate limit API calls".to_string(),
            limit: None,
        })
        .unwrap();
        let j = extract_json(&result);
        assert!(j["matches_found"].as_u64().unwrap_or(0) > 0);
    }

    #[test]
    fn test_route_rank_fusion() {
        let result = tool_route(ToolRouteParams {
            stimulus: "merge results from multiple search sources".to_string(),
            limit: None,
        })
        .unwrap();
        let j = extract_json(&result);
        assert!(j["matches_found"].as_u64().unwrap_or(0) > 0);
    }

    #[test]
    fn test_route_meta_routing() {
        let result = tool_route(ToolRouteParams {
            stimulus: "which routing tool should I use".to_string(),
            limit: None,
        })
        .unwrap();
        let j = extract_json(&result);
        assert!(j["matches_found"].as_u64().unwrap_or(0) > 0);
        let has_route = j["routes"].as_array().unwrap_or(&vec![]).iter().any(|r| {
            r["tools"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .any(|t| t.as_str().unwrap_or("").contains("tool_route"))
        });
        assert!(has_route, "Should route to tool_route itself");
    }

    #[test]
    fn test_export_route_index() {
        let index = export_route_index();
        assert!(index["version"].as_u64().unwrap() >= 1);
        assert!(index["keywords_count"].as_u64().unwrap() > 90);
        assert!(index["chains"].as_array().unwrap().len() >= 7);
        // Check a known keyword exists
        let has_levenshtein = index["keywords"]
            .as_array()
            .unwrap()
            .iter()
            .any(|k| k["keyword"].as_str() == Some("levenshtein"));
        assert!(has_levenshtein, "Should export levenshtein keyword");
    }
}
