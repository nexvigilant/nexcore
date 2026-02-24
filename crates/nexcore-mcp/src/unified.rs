//! Unified command dispatcher for nexcore MCP Server.
//!
//! Routes `nexcore(command="CMD", params={...})` to the appropriate domain function.
//! Handles 286 commands via a single match table.
//!
//! Two dispatch patterns:
//! - `typed(params, f)`: Deserialize `Value` → `T`, then call `f(T)`
//! - `typed_async(params, f)`: Same but for async functions
//! - Direct server method calls for stateful tools (registry, config)
//!
//! When the `telemetry` feature is enabled, all dispatches are instrumented
//! with timing and size metrics.

use crate::{NexCoreMcpServer, params, tools};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::Value;

/// Dispatch with telemetry + audit instrumentation (when feature enabled).
///
/// This is the primary entry point that wraps `dispatch_inner` with metrics collection
/// and full audit trail recording (params + response content).
#[cfg(feature = "telemetry")]
pub async fn dispatch(
    command: &str,
    params: Value,
    server: &NexCoreMcpServer,
) -> Result<CallToolResult, McpError> {
    use crate::telemetry::{
        CallMeasurement, build_audit_record, init_telemetry_writer, record_audit,
    };
    use std::time::Instant;

    // Ensure telemetry + audit writers are initialized
    init_telemetry_writer();

    // Capture serialized input for audit trail
    let input_json = serde_json::to_string(&params).unwrap_or_else(|_| "{}".to_string());
    let input_bytes = input_json.len();

    let measurement = CallMeasurement::start(command, input_bytes);
    let audit_start = Instant::now();

    // Execute the actual dispatch
    let result = dispatch_inner(command, params, server).await;

    // Estimate output size and record telemetry
    let (success, output_bytes, output_json, error_msg) = match &result {
        Ok(r) => {
            let size = estimate_result_size(r);
            let json = serialize_result_for_audit(r);
            (true, size, json, None)
        }
        Err(e) => {
            let err_str = format!("{:?}", e);
            (
                false,
                0,
                format!("{{\"error\":\"{}\"}}", err_str.replace('"', "\\\"")),
                Some(err_str),
            )
        }
    };

    measurement.finish(success, output_bytes);

    // Record full audit trail (params + response)
    let audit_record = build_audit_record(
        command,
        &input_json,
        &output_json,
        audit_start.elapsed(),
        success,
        error_msg,
    );
    record_audit(audit_record);

    result
}

/// Serialize a CallToolResult's text content for audit logging.
#[cfg(feature = "telemetry")]
fn serialize_result_for_audit(result: &CallToolResult) -> String {
    let mut parts = Vec::new();
    for content in &result.content {
        if let Some(text) = content.as_text() {
            parts.push(text.text.clone());
        }
    }
    if parts.is_empty() {
        "{}".to_string()
    } else if parts.len() == 1 {
        parts.into_iter().next().unwrap_or_default()
    } else {
        serde_json::to_string(&parts).unwrap_or_else(|_| "[]".to_string())
    }
}

/// Estimate the size of a CallToolResult for telemetry.
#[cfg(feature = "telemetry")]
fn estimate_result_size(result: &CallToolResult) -> usize {
    let mut size = 0;
    for content in &result.content {
        if let Some(text) = content.as_text() {
            size += text.text.len();
        }
    }
    if let Some(structured) = &result.structured_content {
        size += serde_json::to_string(structured)
            .map(|s| s.len())
            .unwrap_or(0);
    }
    size
}

/// Dispatch without telemetry (when feature disabled).
#[cfg(not(feature = "telemetry"))]
pub async fn dispatch(
    command: &str,
    params: Value,
    server: &NexCoreMcpServer,
) -> Result<CallToolResult, McpError> {
    dispatch_inner(command, params, server).await
}

/// Inner dispatch function (command routing logic).
///
/// Tier: T2-C (Cross-domain composite dispatcher)
/// Grounds to: T1 Mapping (command->function) + T1 Sequence (param pipeline)
async fn dispatch_inner(
    command: &str,
    params: Value,
    server: &NexCoreMcpServer,
) -> Result<CallToolResult, McpError> {
    match command {
        "help" => help_catalog(),
        "toolbox" => typed(params, toolbox_search),

        // ====================================================================
        // System Tools (4)
        // ====================================================================
        "nexcore_health" => server.unified_health(),
        "config_validate" => server.unified_config_validate(),
        "mcp_servers_list" => typed(params, |p: params::McpServersListParams| {
            server.unified_mcp_servers_list(p)
        }),
        "mcp_server_get" => typed(params, |p: params::McpServerGetParams| {
            server.unified_mcp_server_get(p)
        }),

        // ====================================================================
        // API Tools (2)
        // ====================================================================
        "api_health" => typed_async(params, tools::api::health).await,
        "api_list_routes" => typed_async(params, tools::api::list_routes).await,

        // ====================================================================
        // Foundation Tools (9)
        // ====================================================================
        "foundation_levenshtein" => typed(params, tools::foundation::calc_levenshtein),
        "foundation_levenshtein_bounded" => {
            typed(params, tools::foundation::calc_levenshtein_bounded)
        }
        "foundation_fuzzy_search" => typed(params, tools::foundation::fuzzy_search),
        "foundation_sha256" => typed(params, tools::foundation::sha256),
        "foundation_yaml_parse" => typed(params, tools::foundation::yaml_parse),
        "foundation_graph_topsort" => typed(params, tools::foundation::graph_topsort),
        "foundation_graph_levels" => typed(params, tools::foundation::graph_levels),
        "foundation_fsrs_review" => typed(params, tools::foundation::fsrs_review),
        "foundation_concept_grep" => typed(params, tools::foundation::concept_grep),

        // ====================================================================
        // Topology — TDA + Graph Analysis (6 tools)
        // ====================================================================
        "topo_vietoris_rips" => typed(params, tools::topology::topo_vietoris_rips),
        "topo_persistence" => typed(params, tools::topology::topo_persistence),
        "topo_betti" => typed(params, tools::topology::topo_betti),
        "graph_centrality" => typed(params, tools::topology::graph_centrality),
        "graph_components" => typed(params, tools::topology::graph_components),
        "graph_shortest_path" => typed(params, tools::topology::graph_shortest_path),

        // ====================================================================
        // Formula-Derived Tools (5) — KU extraction → MCP tools
        // ====================================================================
        "pv_signal_strength" => typed(params, tools::formula::signal_strength),
        "foundation_domain_distance" => typed(params, tools::formula::domain_distance),
        "foundation_flywheel_velocity" => typed(params, tools::formula::flywheel_velocity),
        "foundation_token_ratio" => typed(params, tools::formula::token_ratio),
        "foundation_spectral_overlap" => typed(params, tools::formula::spectral_overlap),

        // ====================================================================
        // PV Signal Detection Tools (8)
        // ====================================================================
        "pv_signal_complete" => typed(params, tools::pv::signal_complete),
        "pv_signal_prr" => typed(params, tools::pv::signal_prr),
        "pv_signal_ror" => typed(params, tools::pv::signal_ror),
        "pv_signal_ic" => typed(params, tools::pv::signal_ic),
        "pv_signal_ebgm" => typed(params, tools::pv::signal_ebgm),
        "pv_chi_square" => typed(params, tools::pv::chi_square),
        "pv_signal_cooperative" => typed(params, tools::pv::signal_cooperative),
        "pv_naranjo_quick" => typed(params, tools::pv::naranjo_quick),
        "pv_who_umc_quick" => typed(params, tools::pv::who_umc_quick),

        // ====================================================================
        // Benefit-Risk QBRI Tools (3)
        // ====================================================================
        "pv_qbri_compute" => typed(params, tools::benefit_risk::qbri_compute),
        "pv_qbri_derive" => typed(params, tools::benefit_risk::qbri_derive),
        "pv_qbri_equation" => tools::benefit_risk::qbri_equation(),

        // ====================================================================
        // Benefit-Risk QBR Tools (3) — Statistical-Evidence Engine
        // ====================================================================
        "qbr_compute" => typed(params, tools::benefit_risk::qbr_compute),
        "qbr_simple" => typed(params, tools::benefit_risk::qbr_simple),
        "qbr_therapeutic_window" => typed(params, tools::benefit_risk::qbr_therapeutic_window),

        // ====================================================================
        // Signal Pipeline Tools (3)
        // ====================================================================
        "signal_detect" => typed(params, tools::signal::signal_detect),
        "signal_batch" => typed(params, tools::signal::signal_batch),
        "signal_thresholds" => tools::signal::signal_thresholds(),

        // ====================================================================
        // PVDSL Tools (4)
        // ====================================================================
        "pvdsl_compile" => typed(params, tools::pvdsl::pvdsl_compile),
        "pvdsl_execute" => typed(params, tools::pvdsl::pvdsl_execute),
        "pvdsl_eval" => typed(params, tools::pvdsl::pvdsl_eval),
        "pvdsl_functions" => tools::pvdsl::pvdsl_functions(),

        // ====================================================================
        // Vigilance Tools (5)
        // ====================================================================
        "vigilance_safety_margin" => typed(params, tools::vigilance::safety_margin),
        "vigilance_risk_score" => typed(params, tools::vigilance::risk_score),
        "vigilance_harm_types" => tools::vigilance::harm_types(),
        "vigilance_map_to_tov" => typed(params, tools::vigilance::map_to_tov),
        "pv_signal_chart" => typed(params, tools::vigilance::pv_signal_chart),

        // ====================================================================
        // Compliance Tools (5)
        // ====================================================================
        "compliance_check_exclusion" => {
            typed_async(params, tools::compliance::check_exclusion).await
        }
        "compliance_assess" => typed(params, tools::compliance::assess),
        "compliance_catalog_ich" => typed(params, tools::compliance::catalog_ich),
        "compliance_sec_filings" => typed_async(params, tools::compliance::sec_filings).await,
        "compliance_sec_pharma" => typed_async(params, tools::compliance::sec_pharma).await,

        // ====================================================================
        // Hormone Tools (4)
        // ====================================================================
        "hormone_status" => tools::hormones::status(),
        "hormone_get" => typed(params, tools::hormones::get),
        "hormone_stimulus" => typed(params, tools::hormones::stimulus),
        "hormone_modifiers" => tools::hormones::modifiers(),

        // ====================================================================
        // Guardian Tools (14)
        // ====================================================================
        "guardian_homeostasis_tick" => typed_async(params, tools::guardian::homeostasis_tick).await,
        "guardian_evaluate_pv" => typed(params, tools::guardian::evaluate_pv),
        "guardian_status" => typed_async(params, tools::guardian::status).await,
        "guardian_reset" => tools::guardian::reset().await,
        "guardian_inject_signal" => typed(params, tools::guardian::inject_signal),
        "guardian_sensors_list" => typed_async(params, tools::guardian::sensors_list).await,
        "guardian_actuators_list" => typed_async(params, tools::guardian::actuators_list).await,
        "guardian_history" => typed(params, tools::guardian::history),
        "guardian_subscribe" => typed(params, tools::guardian::subscribe),
        "guardian_originator_classify" => typed(params, tools::guardian::originator_classify),
        "guardian_ceiling_for_originator" => typed(params, tools::guardian::ceiling_for_originator),
        "guardian_space3d_compute" => typed(params, tools::guardian::space3d_compute),
        "guardian_adversarial_input" => {
            typed_async(params, tools::adversarial::guardian_adversarial_input).await
        }
        "adversarial_decision_probe" => {
            typed(params, tools::adversarial::adversarial_decision_probe)
        }
        "pv_control_loop_tick" => typed(params, tools::guardian::pv_control_loop_tick),
        "fda_bridge_evaluate" => typed(params, tools::guardian::fda_bridge_evaluate),
        "fda_bridge_batch" => typed(params, tools::guardian::fda_bridge_batch),

        // ====================================================================
        // MCP Lock Tools (3)
        // ====================================================================
        "mcp_lock" => typed(params, tools::mcp_lock::mcp_lock),
        "mcp_unlock" => typed(params, tools::mcp_lock::mcp_unlock),
        "mcp_lock_status" => typed(params, tools::mcp_lock::mcp_lock_status),

        // ====================================================================
        // HUD Capability Tools (24)
        // ====================================================================
        "sba_allocate_agent" => typed(params, tools::hud::sba_allocate_agent),
        "sba_chain_next" => typed(params, tools::hud::sba_chain_next),
        "ssa_persist_state" => typed(params, tools::hud::ssa_persist_state),
        "ssa_verify_integrity" => typed(params, tools::hud::ssa_verify_integrity),
        "fed_budget_report" => typed(params, tools::hud::fed_budget_report),
        "fed_recommend_model" => typed(params, tools::hud::fed_recommend_model),
        "sec_audit_market" => typed(params, tools::hud::sec_audit_market),
        "comm_recommend_protocol" => typed(params, tools::hud::comm_recommend_protocol),
        "comm_route_message" => typed(params, tools::hud::comm_route_message),
        "explore_launch_mission" => typed(params, tools::hud::explore_launch_mission),
        "explore_record_discovery" => typed(params, tools::hud::explore_record_discovery),
        "explore_get_frontier" => typed(params, tools::hud::explore_get_frontier),
        "health_validate_signal" => typed(params, tools::hud::health_validate_signal),
        "health_measure_impact" => typed(params, tools::hud::health_measure_impact),
        "treasury_convert_asymmetry" => typed(params, tools::hud::treasury_convert_asymmetry),
        "treasury_audit" => typed(params, tools::hud::treasury_audit),
        "dot_dispatch_manifest" => typed(params, tools::hud::dot_dispatch_manifest),
        "dot_verify_highway" => typed(params, tools::hud::dot_verify_highway),
        "dhs_verify_boundary" => typed(params, tools::hud::dhs_verify_boundary),
        "edu_train_agent" => typed(params, tools::hud::edu_train_agent),
        "edu_evaluate" => typed(params, tools::hud::edu_evaluate),
        "nsf_fund_research" => typed(params, tools::hud::nsf_fund_research),
        "gsa_procure" => typed(params, tools::hud::gsa_procure),
        "gsa_audit_value" => typed(params, tools::hud::gsa_audit_value),

        // ====================================================================
        // Commandment Tools (4) - The 15 Human Commandments
        // ====================================================================
        "commandment_verify" => typed(params, tools::commandments::commandment_verify),
        "commandment_info" => typed(params, tools::commandments::commandment_info),
        "commandment_list" => typed(params, tools::commandments::commandment_list),
        "commandment_audit" => typed(params, tools::commandments::commandment_audit),

        // ====================================================================
        // Documentation Generation Tools (1)
        // ====================================================================
        "docs_generate_claude_md" => typed(params, tools::docs::docs_generate_claude_md),

        // ====================================================================
        // Crate X-Ray (3)
        // ====================================================================
        "crate_xray" => typed(params, tools::crate_xray::xray),
        "crate_xray_trial" => typed(params, tools::crate_xray::trial),
        "crate_xray_goals" => typed(params, tools::crate_xray::goals),

        // ====================================================================
        // Crate Development Framework (2)
        // ====================================================================
        "crate_dev_scaffold" => typed(params, tools::crate_dev::scaffold_crate),
        "crate_dev_audit" => typed(params, tools::crate_dev::audit_crate),

        // ====================================================================
        // Vigil Orchestrator Tools (13)
        // ====================================================================
        "vigil_status" => typed(params, tools::vigil::status),
        "vigil_health" => typed_async(params, tools::vigil::health).await,
        "vigil_emit_event" => typed_async(params, tools::vigil::emit_event).await,
        "vigil_memory_search" => typed_async(params, tools::vigil::memory_search).await,
        "vigil_memory_stats" => typed_async(params, tools::vigil::memory_stats).await,
        "vigil_llm_stats" => typed_async(params, tools::vigil::llm_stats).await,
        "vigil_source_control" => typed_async(params, tools::vigil::source_control).await,
        "vigil_executor_control" => typed_async(params, tools::vigil::executor_control).await,
        "vigil_authority_config" => typed_async(params, tools::vigil::authority_config).await,
        "vigil_context_assemble" => typed_async(params, tools::vigil::context_assemble).await,
        "vigil_authority_verify" => typed_async(params, tools::vigil::authority_verify).await,
        "vigil_webhook_test" => typed_async(params, tools::vigil::webhook_test).await,
        "vigil_source_config" => typed_async(params, tools::vigil::source_config).await,

        // ====================================================================
        // End-to-End PV Pipeline (1)
        // ====================================================================
        "pv_pipeline" => typed_async(params, tools::pv_pipeline::run_pipeline).await,

        // ====================================================================
        // PV Axioms Database (5)
        // ====================================================================
        "pv_axioms_ksb_lookup" => typed(params, tools::pv_axioms::ksb_lookup),
        "pv_axioms_regulation_search" => typed(params, tools::pv_axioms::regulation_search),
        "pv_axioms_traceability_chain" => typed(params, tools::pv_axioms::traceability_chain),
        "pv_axioms_domain_dashboard" => typed(params, tools::pv_axioms::domain_dashboard),
        "pv_axioms_query" => typed(params, tools::pv_axioms::query),

        // ====================================================================
        // Skills Tools (22)
        // ====================================================================
        "skill_scan" => typed(params, |p| tools::skills::scan(&server.registry, p)),
        "skill_list" => tools::skills::list(&server.registry),
        "skill_get" => typed(params, |p| tools::skills::get(&server.registry, p)),
        "skill_validate" => typed(params, tools::skills::validate),
        "skill_search_by_tag" => typed(params, |p| {
            tools::skills::search_by_tag(&server.registry, p)
        }),
        "skill_list_nested" => typed(params, |p| tools::skills::list_nested(&server.registry, p)),
        "skill_taxonomy_query" => typed(params, tools::skills::taxonomy_query),
        "skill_taxonomy_list" => typed(params, tools::skills::taxonomy_list),
        "skill_categories_compute_intensive" => tools::skills::categories_compute_intensive(),
        "skill_orchestration_analyze" => typed(params, tools::skills::orchestration_analyze),
        "skill_execute" => typed(params, tools::skills::execute),
        "skill_schema" => typed(params, tools::skills::schema),
        "skill_compile" => typed(params, tools::skills::compile),
        "skill_compile_check" => typed(params, tools::skills::compile_check),
        "vocab_skill_lookup" => typed(params, tools::skills::vocab_skill_lookup),
        "primitive_skill_lookup" => typed(params, tools::skills::primitive_skill_lookup),
        "skill_chain_lookup" => typed(params, tools::skills::skill_chain_lookup),
        "skill_route" => typed(params, |p| tools::skills::route(&server.registry, p)),
        "vocab_list" => tools::skills::vocab_list(),
        "nexcore_assist" => typed(params, |p| tools::assist::search(&server.assist_index, p)),

        // ====================================================================
        // Registry Tools (8) — Compliance assessment, promotion, ToV monitoring
        // ====================================================================
        "registry_assess_skill" => typed(params, tools::registry::assess_skill),
        "registry_assess_all" => typed(params, tools::registry::assess_all),
        "registry_gap_report" => typed(params, tools::registry::gap_report),
        "registry_promotable" => typed(params, tools::registry::promotable),
        "registry_promotion_plan" => typed(params, tools::registry::promotion_plan),
        "registry_tov_safety" => typed(params, tools::registry::tov_safety),
        "registry_tov_harm" => typed(params, tools::registry::tov_harm),
        "registry_tov_is_safe" => typed(params, tools::registry::tov_is_safe),

        // ====================================================================
        // Guidelines Tools (9)
        // ====================================================================
        "guidelines_search" => typed(params, tools::guidelines::search),
        "guidelines_get" => typed(params, tools::guidelines::get),
        "guidelines_categories" => tools::guidelines::categories(),
        "guidelines_pv_all" => tools::guidelines::pv_all(),
        "guidelines_url" => typed(params, tools::guidelines::url),

        // FDA Guidance Tools (5)
        // ====================================================================
        "fda_guidance_search" => typed(params, tools::fda_guidance::search),
        "fda_guidance_get" => typed(params, tools::fda_guidance::get),
        "fda_guidance_categories" => tools::fda_guidance::categories(),
        "fda_guidance_url" => typed(params, tools::fda_guidance::url),
        "fda_guidance_status" => typed(params, tools::fda_guidance::status),

        "ich_lookup" => typed(params, tools::ich_glossary::ich_lookup),
        "ich_search" => typed(params, tools::ich_glossary::ich_search),
        "ich_guideline" => typed(params, tools::ich_glossary::ich_guideline),
        "ich_stats" => tools::ich_glossary::ich_stats(),

        // ====================================================================
        // MESH Tools (6)
        // ====================================================================
        "mesh_lookup" => typed_async(params, tools::mesh::lookup).await,
        "mesh_search" => typed_async(params, tools::mesh::search).await,
        "mesh_tree" => typed_async(params, tools::mesh::tree).await,
        "mesh_crossref" => typed_async(params, tools::mesh::crossref).await,
        "mesh_enrich_pubmed" => typed_async(params, tools::mesh::enrich_pubmed).await,
        "mesh_consistency" => typed_async(params, tools::mesh::consistency).await,

        // ====================================================================
        // FAERS Tools (5)
        // ====================================================================
        "faers_search" => typed_async(params, tools::faers::search).await,
        "faers_drug_events" => typed_async(params, tools::faers::drug_events).await,
        "faers_signal_check" => typed_async(params, tools::faers::signal_check).await,
        "faers_disproportionality" => typed_async(params, tools::faers::disproportionality).await,
        "faers_compare_drugs" => typed_async(params, tools::faers::compare_drugs).await,

        // ====================================================================
        // FAERS ETL Tools (4) — Local bulk data pipeline
        // ====================================================================
        "faers_etl_run" => typed_async(params, tools::faers_etl::run).await,
        "faers_etl_signals" => typed_async(params, tools::faers_etl::signals).await,
        "faers_etl_known_pairs" => typed_async(params, tools::faers_etl::known_pairs).await,
        "faers_etl_status" => typed(params, tools::faers_etl::status),

        // ====================================================================
        // PHAROS Tools (3) — Autonomous Signal Surveillance
        // ====================================================================
        "pharos_run" => typed_async(params, tools::pharos::run).await,
        "pharos_status" => typed(params, tools::pharos::status),
        "pharos_report" => typed(params, tools::pharos::report),

        // ====================================================================
        // FAERS Analytics Tools (6) — P0 Patient Safety Algorithms
        // ====================================================================
        "faers_outcome_conditioned" => typed(params, tools::faers_analytics::outcome_conditioned),
        "faers_signal_velocity" => typed(params, tools::faers_analytics::signal_velocity),
        "faers_seriousness_cascade" => typed(params, tools::faers_analytics::seriousness_cascade),
        "faers_polypharmacy" => typed(params, tools::faers_analytics::polypharmacy),
        "faers_reporter_weighted" => typed(params, tools::faers_analytics::reporter_weighted),
        "faers_geographic_divergence" => {
            typed(params, tools::faers_analytics::geographic_divergence)
        }

        // ====================================================================
        // NCBI Entrez Tools (3)
        // ====================================================================
        // ncbi tools disabled — ncbi module needs feature gate in nexcore-dna

        // ====================================================================
        // Lex Primitiva Tools (7) — T1 Symbolic Foundation
        // ====================================================================
        "lex_primitiva_list" => typed(params, tools::lex_primitiva::list_primitives),
        "lex_primitiva_get" => typed(params, tools::lex_primitiva::get_primitive),
        "lex_primitiva_tier" => typed(params, tools::lex_primitiva::classify_tier),
        "lex_primitiva_composition" => typed(params, tools::lex_primitiva::get_composition),
        "lex_primitiva_reverse_compose" => typed(params, tools::lex_primitiva::reverse_compose),
        "lex_primitiva_reverse_lookup" => typed(params, tools::lex_primitiva::reverse_lookup),
        "lex_primitiva_molecular_weight" => typed(params, tools::lex_primitiva::molecular_weight),
        "lex_primitiva_dominant_shift" => typed(params, tools::lex_primitiva::dominant_shift),
        "lex_primitiva_synth" => typed(params, tools::synth::lex_primitiva_synth),
        "lex_primitiva_state_mode" => typed(params, tools::lex_primitiva::get_state_mode),
        "lex_primitiva_audit" => return tools::lex_primitiva::audit(),

        // ====================================================================
        // Laboratory Tools (4) — Concept Experiment Engine
        // ====================================================================
        "lab_experiment" => typed(params, tools::laboratory::lab_experiment),
        "lab_compare" => typed(params, tools::laboratory::lab_compare),
        "lab_react" => typed(params, tools::laboratory::lab_react),
        "lab_batch" => typed(params, tools::laboratory::lab_batch),

        // ====================================================================
        // Skill Token Analysis (1)
        // ====================================================================
        "skill_token_analyze" => typed(params, tools::skill_tokens::analyze),

        // ====================================================================
        // CEP Tools (6) — Cognitive Evolution Pipeline
        // ====================================================================
        "cep_execute_stage" => typed(params, tools::cep::execute_stage),
        "cep_pipeline_stages" => tools::cep::pipeline_stages(),
        "cep_validate_extraction" => typed(params, tools::cep::validate_extraction),
        "cep_extract_primitives" => typed(params, tools::cep::extract_primitives),
        "cep_domain_translate" => typed(params, tools::cep::domain_translate),
        "cep_classify_primitive" => {
            let p: params::PrimitiveTierClassifyParams = deser(params)?;
            tools::cep::classify_primitive(p.domain_count)
        }

        // ====================================================================
        // GCloud Tools (19)
        // ====================================================================
        "gcloud_auth_list" => tools::gcloud::auth_list().await,
        "gcloud_config_list" => tools::gcloud::config_list().await,
        "gcloud_config_get" => {
            typed_async(params, |p: params::GcloudConfigGetParams| async move {
                tools::gcloud::config_get(&p.property).await
            })
            .await
        }
        "gcloud_config_set" => {
            typed_async(params, |p: params::GcloudConfigSetParams| async move {
                tools::gcloud::config_set(&p.property, &p.value).await
            })
            .await
        }
        "gcloud_projects_list" => tools::gcloud::projects_list().await,
        "gcloud_projects_describe" => {
            typed_async(params, |p: params::GcloudProjectParams| async move {
                tools::gcloud::projects_describe(&p.project_id).await
            })
            .await
        }
        "gcloud_projects_get_iam_policy" => {
            typed_async(params, |p: params::GcloudProjectParams| async move {
                tools::gcloud::projects_get_iam_policy(&p.project_id).await
            })
            .await
        }
        "gcloud_secrets_list" => {
            typed_async(
                params,
                |p: params::GcloudOptionalProjectParams| async move {
                    tools::gcloud::secrets_list(p.project.as_deref()).await
                },
            )
            .await
        }
        "gcloud_secrets_versions_access" => {
            typed_async(params, |p: params::GcloudSecretsAccessParams| async move {
                tools::gcloud::secrets_versions_access(
                    &p.secret_name,
                    &p.version,
                    p.project.as_deref(),
                )
                .await
            })
            .await
        }
        "gcloud_storage_buckets_list" => {
            typed_async(
                params,
                |p: params::GcloudOptionalProjectParams| async move {
                    tools::gcloud::storage_buckets_list(p.project.as_deref()).await
                },
            )
            .await
        }
        "gcloud_storage_ls" => {
            typed_async(params, |p: params::GcloudStoragePathParams| async move {
                tools::gcloud::storage_ls(&p.path).await
            })
            .await
        }
        "gcloud_storage_cp" => {
            typed_async(params, |p: params::GcloudStorageCpParams| async move {
                tools::gcloud::storage_cp(&p.source, &p.destination, p.recursive).await
            })
            .await
        }
        "gcloud_compute_instances_list" => {
            typed_async(
                params,
                |p: params::GcloudComputeInstancesParams| async move {
                    tools::gcloud::compute_instances_list(p.project.as_deref(), p.zone.as_deref())
                        .await
                },
            )
            .await
        }
        "gcloud_run_services_list" => {
            typed_async(params, |p: params::GcloudServiceListParams| async move {
                tools::gcloud::run_services_list(p.project.as_deref(), p.region.as_deref()).await
            })
            .await
        }
        "gcloud_run_services_describe" => {
            typed_async(
                params,
                |p: params::GcloudServiceDescribeParams| async move {
                    tools::gcloud::run_services_describe(&p.name, &p.region, p.project.as_deref())
                        .await
                },
            )
            .await
        }
        "gcloud_functions_list" => {
            typed_async(params, |p: params::GcloudServiceListParams| async move {
                tools::gcloud::functions_list(p.project.as_deref(), p.region.as_deref()).await
            })
            .await
        }
        "gcloud_iam_service_accounts_list" => {
            typed_async(
                params,
                |p: params::GcloudOptionalProjectParams| async move {
                    tools::gcloud::iam_service_accounts_list(p.project.as_deref()).await
                },
            )
            .await
        }
        "gcloud_logging_read" => {
            typed_async(params, |p: params::GcloudLoggingReadParams| async move {
                tools::gcloud::logging_read(&p.filter, p.limit, p.project.as_deref()).await
            })
            .await
        }
        "gcloud_run_command" => {
            typed_async(params, |p: params::GcloudRunCommandParams| async move {
                tools::gcloud::run_command(&p.command, p.timeout).await
            })
            .await
        }

        // ====================================================================
        // Wolfram Alpha Tools (19)
        // ====================================================================
        "wolfram_query" => typed_async(params, tools::wolfram::query).await,
        "wolfram_short_answer" => typed_async(params, tools::wolfram::short_answer).await,
        "wolfram_spoken_answer" => typed_async(params, tools::wolfram::spoken_answer).await,
        "wolfram_calculate" => typed_async(params, tools::wolfram::calculate).await,
        "wolfram_step_by_step" => typed_async(params, tools::wolfram::step_by_step).await,
        "wolfram_plot" => typed_async(params, tools::wolfram::plot).await,
        "wolfram_convert" => typed_async(params, tools::wolfram::convert).await,
        "wolfram_chemistry" => typed_async(params, tools::wolfram::chemistry).await,
        "wolfram_physics" => typed_async(params, tools::wolfram::physics).await,
        "wolfram_astronomy" => typed_async(params, tools::wolfram::astronomy).await,
        "wolfram_statistics" => typed_async(params, tools::wolfram::statistics).await,
        "wolfram_data_lookup" => typed_async(params, tools::wolfram::data_lookup).await,
        "wolfram_query_with_assumption" => {
            typed_async(params, tools::wolfram::query_with_assumption).await
        }
        "wolfram_query_filtered" => typed_async(params, tools::wolfram::query_filtered).await,
        "wolfram_image_result" => typed_async(params, tools::wolfram::image_result).await,
        "wolfram_datetime" => typed_async(params, tools::wolfram::datetime).await,
        "wolfram_nutrition" => typed_async(params, tools::wolfram::nutrition).await,
        "wolfram_finance" => typed_async(params, tools::wolfram::finance).await,
        "wolfram_linguistics" => typed_async(params, tools::wolfram::linguistics).await,

        // ====================================================================
        // Perplexity AI Search Tools (4)
        // T1 Grounding: μ (Mapping) — query → search-grounded response
        // ====================================================================
        "perplexity_search" => typed_async(params, tools::perplexity::search).await,
        "perplexity_research" => typed_async(params, tools::perplexity::research).await,
        "perplexity_competitive" => typed_async(params, tools::perplexity::competitive).await,
        "perplexity_regulatory" => typed_async(params, tools::perplexity::regulatory).await,

        // ====================================================================
        // Principles Knowledge Base Tools (3)
        // ====================================================================
        "principles_list" => typed(params, tools::principles::list_principles),
        "principles_get" => typed(params, tools::principles::get_principle),
        "principles_search" => typed(params, tools::principles::search_principles),

        // ====================================================================
        // Universal Validation Tools (4)
        // ====================================================================
        "validation_run" => typed(params, tools::validation::run),
        "validation_check" => typed(params, tools::validation::check),
        "validation_domains" => typed(params, tools::validation::domains),
        "validation_classify_tests" => typed(params, tools::validation::classify_tests_tool),

        // ====================================================================
        // Brain Tools (25)
        // ====================================================================
        "brain_session_create" => typed(params, tools::brain::session_create),
        "brain_session_load" => typed(params, tools::brain::session_load),
        "brain_sessions_list" => typed(params, tools::brain::sessions_list),
        "brain_artifact_save" => typed(params, tools::brain::artifact_save),
        "brain_artifact_resolve" => typed(params, tools::brain::artifact_resolve),
        "brain_artifact_get" => typed(params, tools::brain::artifact_get),
        "brain_artifact_diff" => typed(params, tools::brain::artifact_diff),
        "code_tracker_track" => typed(params, tools::brain::code_tracker_track),
        "code_tracker_changed" => typed(params, tools::brain::code_tracker_changed),
        "code_tracker_original" => typed(params, tools::brain::code_tracker_original),
        "implicit_get" => typed(params, tools::brain::implicit_get),
        "implicit_set" => typed(params, tools::brain::implicit_set),
        "implicit_stats" => tools::brain::implicit_stats(),
        "implicit_find_corrections" => typed(params, tools::brain::implicit_find_corrections),
        "implicit_patterns_by_grounding" => {
            typed(params, tools::brain::implicit_patterns_by_grounding)
        }
        "implicit_patterns_by_relevance" => tools::brain::implicit_patterns_by_relevance(),
        "brain_recovery_check" => tools::brain::recovery_check(),
        "brain_recovery_repair" => typed(params, |p: params::BrainRecoveryRepairParams| {
            tools::brain::recovery_repair(p.session_id.as_deref())
        }),
        "brain_recovery_rebuild_index" => tools::brain::recovery_rebuild_index(),
        "brain_recovery_auto" => tools::brain::recovery_auto(),
        "brain_coordination_acquire" => typed(params, tools::brain::coordination_acquire),
        "brain_coordination_release" => typed(params, tools::brain::coordination_release),
        "brain_coordination_status" => typed(params, tools::brain::coordination_status),
        "brain_verify_engrams" => typed(params, tools::brain_verify::brain_verify_engrams),

        // ====================================================================
        // Brain Database Tools (8) - SQLite Knowledge Store
        // ====================================================================
        "brain_db_summary" => tools::brain_db::summary(),
        "brain_db_decisions_stats" => tools::brain_db::decisions_stats(),
        "brain_db_tool_stats" => tools::brain_db::tool_stats(),
        "brain_db_antibodies" => tools::brain_db::antibodies(),
        "brain_db_handoffs" => typed(params, tools::brain_db::handoffs),
        "brain_db_tasks" => tools::brain_db::tasks(),
        "brain_db_efficiency" => tools::brain_db::efficiency(),
        "brain_db_sync" => tools::brain_db::sync(),
        "brain_db_query" => typed(params, tools::brain_db::query),

        // ====================================================================
        // Anatomy DB Tools (12) - Persistent Organ State
        // ====================================================================
        "anatomy_query" => typed(params, tools::anatomy_db::anatomy_query),
        "anatomy_status" => {
            tools::anatomy_db::anatomy_status(crate::params::anatomy_db::AnatomyStatusParams {})
        }
        "anatomy_record_cytokine" => typed(params, tools::anatomy_db::record_cytokine),
        "anatomy_record_hormones" => typed(params, tools::anatomy_db::record_hormones),
        "anatomy_record_guardian_tick" => typed(params, tools::anatomy_db::record_guardian_tick),
        "anatomy_record_immunity_event" => typed(params, tools::anatomy_db::record_immunity_event),
        "anatomy_record_synapse" => typed(params, tools::anatomy_db::record_synapse),
        "anatomy_record_energy" => typed(params, tools::anatomy_db::record_energy),
        "anatomy_record_transcriptase" => typed(params, tools::anatomy_db::record_transcriptase),
        "anatomy_record_ribosome" => typed(params, tools::anatomy_db::record_ribosome),
        "anatomy_record_phenotype" => typed(params, tools::anatomy_db::record_phenotype),
        "anatomy_record_organ_signal" => typed(params, tools::anatomy_db::record_organ_signal),

        // ====================================================================
        // Learning Daemon Tools (5) - Compounding Infrastructure
        // ====================================================================
        "learning_daemon_status" => tools::learning::status(),
        "learning_daemon_trends" => tools::learning::trends(),
        "learning_daemon_beliefs" => tools::learning::beliefs(),
        "learning_daemon_corrections" => tools::learning::corrections(),
        "learning_daemon_velocity" => tools::learning::velocity(),

        // LEARN Vocabulary Program (6) - Feedback Loop Pipeline
        "learn_landscape" => tools::learning::learn_landscape(),
        "learn_extract" => tools::learning::learn_extract(),
        "learn_assimilate" => tools::learning::learn_assimilate(),
        "learn_recall" => tools::learning::learn_recall(),
        "learn_normalize" => tools::learning::learn_normalize(),
        "learn_pipeline" => tools::learning::learn_pipeline(),

        // ====================================================================
        // Synapse Tools (6) - Amplitude Growth Learning
        // ====================================================================
        "synapse_get_or_create" => typed(params, tools::synapse::synapse_get_or_create),
        "synapse_get" => typed(params, tools::synapse::synapse_get),
        "synapse_observe" => typed(params, tools::synapse::synapse_observe),
        "synapse_list" => typed(params, tools::synapse::synapse_list),
        "synapse_stats" => tools::synapse::synapse_stats(),
        "synapse_prune" => tools::synapse::synapse_prune(),

        // ====================================================================
        // Oracle Tools (7) — Bayesian Event Prediction
        // ====================================================================
        "oracle_ingest" => typed(params, tools::oracle::oracle_ingest),
        "oracle_predict" => typed(params, tools::oracle::oracle_predict),
        "oracle_observe" => typed(params, tools::oracle::oracle_observe),
        "oracle_report" => typed(params, tools::oracle::oracle_report),
        "oracle_status" => typed(params, tools::oracle::oracle_status),
        "oracle_reset" => typed(params, tools::oracle::oracle_reset),
        "oracle_top_predictions" => typed(params, tools::oracle::oracle_top_predictions),

        // ====================================================================
        // Immunity Tools (6)
        // ====================================================================
        "immunity_scan" => typed(params, tools::immunity::immunity_scan),
        "immunity_scan_errors" => typed(params, tools::immunity::immunity_scan_errors),
        "immunity_list" => typed(params, tools::immunity::immunity_list),
        "immunity_get" => typed(params, tools::immunity::immunity_get),
        "immunity_propose" => typed(params, tools::immunity::immunity_propose),
        "immunity_status" => tools::immunity::immunity_status(),

        // NMD Surveillance (anti-hallucination pipeline)
        "nmd_check" => typed(params, tools::nmd::nmd_check),
        "nmd_upf_evaluate" => typed(params, tools::nmd::nmd_upf_evaluate),
        "nmd_smg_process" => typed(params, tools::nmd::nmd_smg_process),
        "nmd_adaptive_stats" => typed(params, tools::nmd::nmd_adaptive_stats),
        "nmd_thymic_status" => typed(params, tools::nmd::nmd_thymic_status),
        "nmd_status" => tools::nmd::nmd_status(),

        // ====================================================================
        // Regulatory Primitives Tools (4)
        // ====================================================================
        "regulatory_primitives_extract" => typed(params, tools::regulatory::extract),
        "regulatory_primitives_audit" => typed(params, tools::regulatory::audit),
        "regulatory_primitives_compare" => typed(params, tools::regulatory::compare),
        "regulatory_effectiveness_assess" => typed(params, tools::regulatory::effectiveness_assess),

        // ====================================================================
        // Brand Semantics Tools (4)
        // ====================================================================
        "brand_decomposition_nexvigilant" => {
            tools::brand_semantics::brand_decomposition_nexvigilant()
        }
        "brand_decomposition_get" => typed(params, tools::brand_semantics::brand_decomposition_get),
        "brand_primitive_test" => typed(params, tools::brand_semantics::brand_primitive_test),
        "brand_semantic_tiers" => tools::brand_semantics::brand_semantic_tiers(),

        // ====================================================================
        // Forge Tools (6) — Primitive-first technology construction
        // ====================================================================
        "forge_init" => typed(params, tools::forge::forge_init),
        "forge_reference" => typed(params, tools::forge::forge_reference),
        "forge_mine" => typed(params, tools::forge::forge_mine),
        "forge_prompt" => typed(params, tools::forge::forge_prompt),
        "forge_summary" => typed(params, tools::forge::forge_summary),
        "forge_suggest" => typed(params, tools::forge::forge_suggest),
        "forge_system_prompt" => tools::forge::forge_system_prompt(),
        "forge_tier" => typed(params, |p: params::ForgeTierParams| {
            tools::forge::forge_tier_classify(p.count)
        }),

        // ====================================================================
        // Academy Forge Tools (8)
        // ====================================================================
        "forge_extract" => typed(params, tools::academy_forge::forge_extract),
        "forge_validate" => typed(params, tools::academy_forge::forge_validate),
        "forge_scaffold" => typed(params, tools::academy_forge::forge_scaffold),
        "forge_schema" => tools::academy_forge::forge_schema(),
        "forge_compile" => typed(params, tools::academy_forge::forge_compile),
        "forge_atomize" => typed(params, tools::academy_forge::forge_atomize),
        "forge_graph" => typed(params, tools::academy_forge::forge_graph),
        "forge_shortest_path" => typed(params, tools::academy_forge::forge_shortest_path),

        // ====================================================================
        // Primitive Validation Tools (4)
        // ====================================================================
        "primitive_validate" => typed_async(params, tools::primitive_validation::validate).await,
        "primitive_cite" => typed_async(params, tools::primitive_validation::cite).await,
        "primitive_validate_batch" => {
            typed_async(params, tools::primitive_validation::validate_batch).await
        }
        "primitive_validation_tiers" => tools::primitive_validation::validation_tiers(),

        // ====================================================================
        // Chemistry Primitives Tools (15)
        // ====================================================================
        "chemistry_threshold_rate" => typed(params, tools::chemistry::threshold_rate),
        "chemistry_decay_remaining" => typed(params, tools::chemistry::decay_remaining),
        "chemistry_saturation_rate" => typed(params, tools::chemistry::saturation_rate),
        "chemistry_feasibility" => typed(params, tools::chemistry::feasibility),
        "chemistry_dependency_rate" => typed(params, tools::chemistry::dependency_rate),
        "chemistry_buffer_capacity" => typed(params, tools::chemistry::buffer_cap),
        "chemistry_signal_absorbance" => typed(params, tools::chemistry::signal_absorbance),
        "chemistry_equilibrium" => typed(params, tools::chemistry::equilibrium),
        "chemistry_pv_mappings" => typed(params, tools::chemistry::get_pv_mappings),
        "chemistry_threshold_exceeded" => typed(params, tools::chemistry::check_threshold_exceeded),
        "chemistry_hill_response" => typed(params, tools::chemistry::hill_cooperative),
        "chemistry_nernst_potential" => typed(params, tools::chemistry::nernst_dynamic),
        "chemistry_inhibition_rate" => typed(params, tools::chemistry::inhibition_rate),
        "chemistry_eyring_rate" => typed(params, tools::chemistry::eyring_transition),
        "chemistry_langmuir_coverage" => typed(params, tools::chemistry::langmuir_binding),
        "chemistry_first_law_closed" => typed(params, tools::chemistry::first_law_closed),
        "chemistry_first_law_open" => typed(params, tools::chemistry::first_law_open),
        "chemistry_gaussian_overlap" => typed(params, tools::chemistry::gaussian_overlap),

        // ====================================================================
        // Molecular Biology Tools (4) — Central Dogma, ADME, Codon Translation
        // ====================================================================
        "molecular_translate_codon" => typed(params, tools::molecular::translate_codon),
        "molecular_translate_mrna" => typed(params, tools::molecular::translate_mrna),
        "molecular_central_dogma" => typed(params, tools::molecular::central_dogma),
        "molecular_adme_phase" => typed(params, tools::molecular::adme_phase),

        // ====================================================================
        // Visual Primitives Tools (3) — Shape/Color Pattern Matching for Prima
        // ====================================================================
        "visual_shape_classify" => typed(params, tools::visual::classify_shape),
        "visual_color_analyze" => typed(params, tools::visual::analyze_color),
        "visual_shape_list" => typed(params, tools::visual::list_shapes),

        // ====================================================================
        // Cytokine Signaling Tools (4) — Typed Event Bus + File Telemetry
        // ====================================================================
        "cytokine_emit" => typed(params, tools::cytokine::emit),
        "cytokine_status" => tools::cytokine::telemetry_status(),
        "cytokine_families" => typed(params, tools::cytokine::families),
        "cytokine_recent" => typed(params, tools::cytokine::recent),

        // ====================================================================
        // Biology Primitive Tools (2) — Chemotaxis (→+λ) + Endocytosis (∂+ρ)
        // ====================================================================
        "chemotaxis_gradient" => typed(params, tools::cytokine::chemotaxis_gradient),
        "endocytosis_internalize" => typed(params, tools::cytokine::endocytosis_internalize),

        // ====================================================================
        // Value Mining Tools (4) — Economic Signal Detection (PV Algorithms)
        // ====================================================================
        "value_signal_types" => typed(params, tools::value_mining::list_signal_types),
        "value_signal_detect" => typed(params, tools::value_mining::detect_signal),
        "value_baseline_create" => typed(params, tools::value_mining::create_baseline),
        "value_pv_mapping" => typed(params, tools::value_mining::get_pv_mapping),

        // ====================================================================
        // Signal Theory Tools (5) — Universal Theory of Signals
        // T1 Grounding: ∂(Boundary) + κ(Comparison) + N(Quantity) + ∅(Void)
        // ====================================================================
        "signal_theory_axioms" => tools::signal_theory::axioms(),
        "signal_theory_theorems" => tools::signal_theory::theorems(),
        "signal_theory_detect" => typed(params, tools::signal_theory::detect),
        "signal_theory_decision_matrix" => typed(params, tools::signal_theory::decision_matrix),
        "signal_theory_conservation_check" => {
            typed(params, tools::signal_theory::conservation_check)
        }
        "signal_theory_pipeline" => typed(params, tools::signal_theory::pipeline),
        "signal_theory_cascade" => typed(params, tools::signal_theory::cascade),
        "signal_theory_parallel" => typed(params, tools::signal_theory::parallel),

        // ====================================================================
        // Signal Fence Tools (3) — Process-Level Network Signal Container
        // T1 Grounding: ∂(Boundary) + ∅(Void) + κ(Comparison) + ς(State)
        // ====================================================================
        "fence_status" => tools::signal_fence::fence_status(),
        "fence_scan" => tools::signal_fence::fence_scan(),
        "fence_evaluate" => typed(params, tools::signal_fence::fence_evaluate),

        // ====================================================================
        // Game Theory Tools (5) — 2x2 + N×M Equilibrium + Forge Pipeline
        // ====================================================================
        "game_theory_nash_2x2" => typed(params, tools::game_theory::nash_2x2),
        "forge_payoff_matrix" => typed(params, tools::game_theory::forge_payoff_matrix),
        "forge_nash_solve" => typed(params, tools::game_theory::forge_nash_solve),
        "forge_quality_score" => typed(params, tools::game_theory::forge_quality_score),
        "forge_code_generate" => typed(params, tools::game_theory::forge_code_generate),

        // ====================================================================
        // Mesh Network Tools (4) — Runtime Mesh Networking
        // T1 Grounding: λ(addresses) μ(routing) σ(paths) ∂(TTL) ρ(relay) ν(heartbeats)
        // ====================================================================
        "mesh_network_simulate" => typed(params, tools::mesh_network::simulate),
        "mesh_network_route_quality" => typed(params, tools::mesh_network::route_quality),
        "mesh_network_grounding" => tools::mesh_network::grounding(),
        "mesh_network_node_info" => typed(params, tools::mesh_network::node_info),

        // ====================================================================
        // Prima Language Tools (5) — Primitive-First Programming (.true)
        // T1 Grounding: μ(parse/eval) → σ(AST) → Σ(target) → κ(primitives)
        // ====================================================================
        "prima_parse" => typed(params, tools::prima::prima_parse),
        "prima_eval" => typed(params, tools::prima::prima_eval),
        "prima_codegen" => typed(params, tools::prima::prima_codegen),
        "prima_primitives" => typed(params, tools::prima::prima_primitives),
        "prima_targets" => tools::prima::prima_targets(),

        // ====================================================================
        // Aggregate Tools (5) — Σ + ρ + κ Primitive Gap Fillers
        // T1 Grounding: Σ (Sum) + ρ (Recursion) + κ (Comparison)
        // ====================================================================
        "aggregate_fold" => typed(params, tools::aggregate::aggregate_fold_all),
        "aggregate_tree_fold" => typed(params, tools::aggregate::aggregate_tree_fold),
        "aggregate_rank" => typed(params, tools::aggregate::aggregate_rank),
        "aggregate_percentile" => typed(params, tools::aggregate::aggregate_percentile),
        "aggregate_outliers" => typed(params, tools::aggregate::aggregate_outliers),

        // ====================================================================
        // Compound Growth Tool (1) — Primitive Basis Velocity Tracking
        // T1 Grounding: N (Quantity) + ∂ (Derivative) + ∝ (Proportion)
        // ====================================================================
        "compound_growth" => typed(params, tools::compound::compound_growth),
        "compound_detect" => typed(params, tools::compound_detector::compound_detect),

        // ====================================================================
        // Claude Care Process Tools (6) — Pharmacokinetic Engine for AI Support
        // T1 Grounding: σ (sequence) + ∝ (proportionality) + κ (comparison)
        // ====================================================================
        "ccp_episode_start" => typed(params, tools::ccp::episode_start),
        "ccp_dose_compute" => typed(params, tools::ccp::dose_compute),
        "ccp_episode_advance" => typed(params, tools::ccp::episode_advance),
        "ccp_interaction_check" => typed(params, tools::ccp::interaction_check),
        "ccp_quality_score" => typed(params, tools::ccp::quality_score),
        "ccp_phase_transition" => typed(params, tools::ccp::phase_transition),

        // ====================================================================
        // Education Machine Tools (15) — Primitive-Based STEM Learning Engine
        // T1 Grounding: σ (sequence) + μ (mapping) + ρ (recursion) + ς (state) + N (quantity) + κ (comparison)
        // ====================================================================
        "edu_subject_create" => typed(params, tools::education::subject_create),
        "edu_subject_list" => tools::education::subject_list(),
        "edu_lesson_create" => typed(params, tools::education::lesson_create),
        "edu_lesson_add_step" => typed(params, tools::education::lesson_add_step),
        "edu_learner_create" => typed(params, tools::education::learner_create),
        "edu_enroll" => typed(params, tools::education::enroll),
        "edu_assess" => typed(params, tools::education::assess),
        "edu_mastery" => typed(params, tools::education::mastery),
        "edu_phase_transition" => typed(params, tools::education::phase_transition),
        "edu_phase_info" => typed(params, tools::education::phase_info),
        "edu_review_create" => typed(params, tools::education::review_create),
        "edu_review_schedule" => typed(params, tools::education::review_schedule),
        "edu_review_status" => typed(params, tools::education::review_status),
        "edu_bayesian_update" => typed(params, tools::education::bayesian_update),
        "edu_primitive_map" => typed(params, tools::education::primitive_map),

        // ====================================================================
        // Antitransformer Tools (2) — AI Text Detection via Statistical Fingerprints
        // T1 Grounding: σ (sequence) + κ (comparison) + ∂ (boundary)
        // ====================================================================
        "antitransformer_analyze" => typed(params, tools::antitransformer::antitransformer_analyze),
        "antitransformer_batch" => typed(params, tools::antitransformer::antitransformer_batch),

        // ====================================================================
        // Epidemiology Tools (11)
        // ====================================================================
        "epidemiology_relative_risk" => typed(params, tools::epidemiology::relative_risk),
        "epidemiology_odds_ratio" => typed(params, tools::epidemiology::odds_ratio),
        "epidemiology_attributable_risk" => typed(params, tools::epidemiology::attributable_risk),
        "epidemiology_nnt_nnh" => typed(params, tools::epidemiology::nnt_nnh),
        "epidemiology_attributable_fraction" => {
            typed(params, tools::epidemiology::attributable_fraction)
        }
        "epidemiology_population_attributable_fraction" => typed(
            params,
            tools::epidemiology::population_attributable_fraction,
        ),
        "epidemiology_incidence_rate" => typed(params, tools::epidemiology::incidence_rate),
        "epidemiology_prevalence" => typed(params, tools::epidemiology::prevalence),
        "epidemiology_kaplan_meier" => typed(params, tools::epidemiology::kaplan_meier),
        "epidemiology_smr" => typed(params, tools::epidemiology::smr),
        "epidemiology_mappings" => typed(params, tools::epidemiology::epi_pv_mappings),

        // ====================================================================
        // Token-as-Energy Tools (2) — ATP/ADP Biochemistry for Token Budgets
        // T1 Grounding: N (quantity) + κ (comparison) + ∝ (proportionality)
        // ====================================================================
        "energy_charge" => typed(params, tools::energy::energy_charge),
        "energy_decide" => typed(params, tools::energy::energy_decide),

        // ====================================================================
        // Reverse Transcriptase Tools (4) — Schema Inference + Data Generation
        // T1 Grounding: κ (comparison) + σ (sequence) + μ (function) + ∂ (conditional)
        // ====================================================================
        "transcriptase_process" => typed(params, tools::transcriptase::transcriptase_process),
        "transcriptase_infer" => typed(params, tools::transcriptase::transcriptase_infer),
        "transcriptase_violations" => typed(params, tools::transcriptase::transcriptase_violations),
        "transcriptase_generate" => typed(params, tools::transcriptase::transcriptase_generate),

        // ====================================================================
        // Ribosome Tools (5) — Schema Contract Registry + Drift Detection
        // T1 Grounding: κ (comparison) + σ (sequence) + μ (function) + ∂ (conditional) + N (quantity)
        // ====================================================================
        "ribosome_store" => typed(params, tools::ribosome::ribosome_store),
        "ribosome_list" => Ok(tools::ribosome::ribosome_list()?),
        "ribosome_validate" => typed(params, tools::ribosome::ribosome_validate),
        "ribosome_generate" => typed(params, tools::ribosome::ribosome_generate),
        "ribosome_drift" => typed(params, tools::ribosome::ribosome_drift),

        // ====================================================================
        // Domain Primitives Tools (8) — Tier Taxonomy + Transfer Confidence + Analysis
        // T1 Grounding: σ (sequence) + κ (comparison) + ρ (recursion) + ∃ (existence)
        // ====================================================================
        "domain_primitives_list" => typed(params, tools::domain_primitives::domain_primitives_list),
        "domain_primitives_transfer" => {
            typed(params, tools::domain_primitives::domain_primitives_transfer)
        }
        "domain_primitives_decompose" => typed(
            params,
            tools::domain_primitives::domain_primitives_decompose,
        ),
        "domain_primitives_bottlenecks" => typed(
            params,
            tools::domain_primitives::domain_primitives_bottlenecks,
        ),
        "domain_primitives_compare" => {
            typed(params, tools::domain_primitives::domain_primitives_compare)
        }
        "domain_primitives_topo_sort" => typed(
            params,
            tools::domain_primitives::domain_primitives_topo_sort,
        ),
        "domain_primitives_critical_paths" => typed(
            params,
            tools::domain_primitives::domain_primitives_critical_paths,
        ),
        "domain_primitives_registry" => {
            typed(params, tools::domain_primitives::domain_primitives_registry)
        }
        "domain_primitives_save" => typed(params, tools::domain_primitives::domain_primitives_save),
        "domain_primitives_load" => typed(params, tools::domain_primitives::domain_primitives_load),
        "domain_primitives_transfer_matrix" => typed(
            params,
            tools::domain_primitives::domain_primitives_transfer_matrix,
        ),

        // ====================================================================
        // FDA AI Credibility Assessment Tools (5) — 7-Step Regulatory Framework
        // T1 Grounding: λ+μ (COU) → κ×N (Risk) → σ (Plan) → ∃+κ (Evidence) → κ (Adequacy)
        // ====================================================================
        "fda_define_cou" => {
            let p: params::FdaDefineCouParams = deser(params)?;
            Ok(tools::fda::fda_define_cou(
                &p.question,
                &p.input_domain,
                &p.output_domain,
                &p.purpose_description,
                &p.integration,
                p.confirmatory_sources,
                &p.regulatory_context,
            ))
        }
        "fda_assess_risk" => {
            let p: params::FdaAssessRiskParams = deser(params)?;
            Ok(tools::fda::fda_assess_risk(&p.influence, &p.consequence))
        }
        "fda_create_plan" => {
            let p: params::FdaCreatePlanParams = deser(params)?;
            Ok(tools::fda::fda_create_plan(
                &p.question,
                &p.input_domain,
                &p.output_domain,
                &p.influence,
                &p.consequence,
                &p.regulatory_context,
            ))
        }
        "fda_validate_evidence" => {
            let p: params::FdaValidateEvidenceParams = deser(params)?;
            Ok(tools::fda::fda_validate_evidence(
                &p.evidence_type,
                &p.quality,
                &p.description,
                p.relevant,
                p.reliable,
                p.representative,
            ))
        }
        "fda_decide_adequacy" => {
            let p: params::FdaDecideAdequacyParams = deser(params)?;
            Ok(tools::fda::fda_decide_adequacy(
                &p.risk_level,
                p.high_quality_evidence_count,
                p.fit_for_use_passed,
                p.critical_drift_detected,
            ))
        }

        // ====================================================================
        // FDA Credibility Metrics Tools (6)
        // ====================================================================
        "fda_calculate_score" => {
            let p: params::FdaCalculateScoreParams = deser(params)?;
            Ok(tools::fda_metrics::fda_calculate_score(
                p.evidence_quality,
                p.fit_for_use,
                p.risk_mitigation,
                p.documentation,
            ))
        }
        "fda_metrics_summary" => {
            let p: params::FdaMetricsSummaryParams = deser(params)?;
            Ok(tools::fda_metrics::fda_metrics_summary(
                p.started,
                p.completed,
                p.approved,
                p.rejected,
                p.revision,
                p.drift_alerts,
            ))
        }
        "fda_evidence_distribution" => {
            let p: params::FdaEvidenceDistributionParams = deser(params)?;
            Ok(tools::fda_metrics::fda_evidence_distribution(
                p.evidence_items,
            ))
        }
        "fda_risk_distribution" => {
            let p: params::FdaRiskDistributionParams = deser(params)?;
            Ok(tools::fda_metrics::fda_risk_distribution(p.risk_levels))
        }
        "fda_drift_trend" => {
            let p: params::FdaDriftTrendParams = deser(params)?;
            Ok(tools::fda_metrics::fda_drift_trend(
                p.measurements,
                p.trend_threshold,
            ))
        }
        "fda_rating_thresholds" => Ok(tools::fda_metrics::fda_rating_thresholds()),

        // ====================================================================
        // STEM Primitives Tools (11)
        // ====================================================================
        "stem_version" => tools::stem::version(),
        "stem_taxonomy" => tools::stem::taxonomy(),
        "stem_confidence_combine" => typed(params, tools::stem::confidence_combine),
        "stem_tier_info" => typed(params, tools::stem::tier_info),
        "stem_chem_balance" => typed(params, tools::stem::chem_balance),
        "stem_chem_fraction" => typed(params, tools::stem::chem_fraction),
        "stem_phys_fma" => typed(params, tools::stem::phys_fma),
        "stem_phys_conservation" => typed(params, tools::stem::phys_conservation),
        "stem_phys_period" => typed(params, tools::stem::phys_period),
        "stem_math_bounds_check" => typed(params, tools::stem::math_bounds_check),
        "stem_math_relation_invert" => typed(params, tools::stem::math_relation_invert),
        "stem_chem_ratio" => typed(params, tools::stem::chem_ratio),
        "stem_chem_rate" => typed(params, tools::stem::chem_rate),
        "stem_chem_affinity" => typed(params, tools::stem::chem_affinity),
        "stem_phys_amplitude" => typed(params, tools::stem::phys_amplitude),
        "stem_phys_scale" => typed(params, tools::stem::phys_scale),
        "stem_phys_inertia" => typed(params, tools::stem::phys_inertia),
        "stem_math_proof" => typed(params, tools::stem::math_proof),
        "stem_math_identity" => typed(params, tools::stem::math_identity),
        "stem_spatial_distance" => typed(params, tools::stem::spatial_distance),
        "stem_spatial_triangle" => typed(params, tools::stem::spatial_triangle),
        "stem_spatial_neighborhood" => typed(params, tools::stem::spatial_neighborhood),
        "stem_spatial_dimension" => typed(params, tools::stem::spatial_dimension),
        "stem_spatial_orientation" => typed(params, tools::stem::spatial_orientation),
        // Core tools
        "stem_transfer_confidence" => typed(params, tools::stem::transfer_confidence),
        "stem_integrity_check" => typed(params, tools::stem::integrity_check),
        "stem_retry_budget" => typed(params, tools::stem::retry_budget),
        "stem_determinism_score" => typed(params, tools::stem::determinism_score),
        // Bio tools
        "stem_bio_behavior_profile" => typed(params, tools::stem::bio_behavior_profile),
        "stem_bio_tone_profile" => typed(params, tools::stem::bio_tone_profile),
        // Finance tools
        "stem_finance_discount" => typed(params, tools::stem::finance_discount),
        "stem_finance_compound" => typed(params, tools::stem::finance_compound),
        "stem_finance_spread" => typed(params, tools::stem::finance_spread),
        "stem_finance_maturity" => typed(params, tools::stem::finance_maturity),
        "stem_finance_exposure" => typed(params, tools::stem::finance_exposure),
        "stem_finance_arbitrage" => typed(params, tools::stem::finance_arbitrage),
        "stem_finance_diversify" => typed(params, tools::stem::finance_diversify),
        "stem_finance_return" => typed(params, tools::stem::finance_return),

        // ====================================================================
        // Disney Loop Tools (4) — forward-only compound discovery
        // ====================================================================
        "disney_loop_run" => typed(params, tools::disney_loop::run),
        "disney_loop_anti_regression" => typed(params, tools::disney_loop::anti_regression),
        "disney_loop_curiosity_search" => typed(params, tools::disney_loop::curiosity_search),
        "disney_loop_state_assess" => typed(params, tools::disney_loop::state_assess),

        // ====================================================================
        // KSB Knowledge Tools (3) — 628 PV articles across 15 domains
        // ====================================================================
        "ksb_get" => typed(params, tools::knowledge::get),
        "ksb_search" => typed(params, tools::knowledge::search),
        "ksb_stats" => typed(params, tools::knowledge::stats),

        // ====================================================================
        // Theory of Vigilance (ToV direct — signal strength, stability, epistemic trust)
        // ====================================================================
        "tov_signal_strength" => typed(params, tools::tov::signal_strength),
        "tov_stability_shell" => typed(params, tools::tov::stability_shell),
        "tov_epistemic_trust" => typed(params, tools::tov::epistemic_trust),

        // ====================================================================
        // Knowledge Engine Tools (3)
        // ====================================================================
        "knowledge_engine_compress" => typed(params, tools::knowledge_engine::compress),
        "knowledge_engine_compile" => typed(params, tools::knowledge_engine::compile),
        "knowledge_engine_query" => typed(params, tools::knowledge_engine::query),

        // ====================================================================
        // Relay Fidelity Tools (4)
        // ====================================================================
        "relay_chain_verify" => typed(params, tools::relay::relay_chain_verify),
        "relay_pv_pipeline" => tools::relay::relay_pv_pipeline(),
        "relay_core_detection" => tools::relay::relay_core_detection(),
        "relay_fidelity_compose" => typed(params, tools::relay::relay_fidelity_compose),

        // ====================================================================
        // PV Core Tools (3) — IVF axioms, severity classification
        // ====================================================================
        "pv_core_ivf_assess" => typed(params, tools::pv_core::ivf_assess),
        "pv_core_ivf_axioms" => typed(params, tools::pv_core::ivf_axioms),
        "pv_core_severity_assess" => typed(params, tools::pv_core::severity_assess),

        // ====================================================================
        // Visualization Tools (31)
        // ====================================================================
        "viz_stem_taxonomy" => typed(params, tools::viz::taxonomy),
        "viz_type_composition" => typed(params, tools::viz::composition),
        "viz_method_loop" => typed(params, tools::viz::method_loop),
        "viz_confidence_chain" => typed(params, tools::viz::confidence),
        "viz_bounds" => typed(params, tools::viz::bounds),
        "viz_dag" => typed(params, tools::viz::dag),
        // Foundation (pre-phase)
        "viz_molecular_info" => typed(params, tools::viz_foundation::molecular_info),
        "viz_surface_mesh" => typed(params, tools::viz_foundation::surface_mesh),
        "viz_spectral_analysis" => typed(params, tools::viz_foundation::spectral_analysis),
        "viz_community_detect" => typed(params, tools::viz_foundation::community_detect),
        "viz_centrality" => typed(params, tools::viz_foundation::centrality),
        "viz_vdag_overlay" => typed(params, tools::viz_foundation::vdag_overlay),
        // Biologics (phase 2)
        "viz_antibody_structure" => typed(params, tools::viz_biologics::antibody_structure),
        "viz_interaction_map" => typed(params, tools::viz_biologics::interaction_map),
        "viz_projection" => typed(params, tools::viz_biologics::projection),
        "viz_protein_structure" => typed(params, tools::viz_biologics::protein_structure),
        "viz_topology_analysis" => typed(params, tools::viz_biologics::topology_analysis),
        // Physics (phase 3+4)
        "viz_dynamics_step" => typed(params, tools::viz_physics::dynamics_step),
        "viz_force_field_energy" => typed(params, tools::viz_physics::force_field_energy),
        "viz_gpu_layout" => typed(params, tools::viz_physics::gpu_layout),
        "viz_hypergraph" => typed(params, tools::viz_physics::hypergraph),
        "viz_lod_select" => typed(params, tools::viz_physics::lod_select),
        "viz_minimize_energy" => typed(params, tools::viz_physics::minimize_energy),
        "viz_particle_preset" => typed(params, tools::viz_physics::particle_preset),
        "viz_ae_overlay" => typed(params, tools::viz_physics::ae_overlay),
        "viz_coord_gen" => typed(params, tools::viz_physics::coord_gen),
        "viz_bipartite_layout" => typed(params, tools::viz_physics::bipartite_layout),
        // Advanced (phase 5)
        "viz_manifold_sample" => typed(params, tools::viz_advanced::manifold_sample),
        "viz_string_modes" => typed(params, tools::viz_advanced::string_modes),
        "viz_render_pipeline" => typed(params, tools::viz_advanced::render_pipeline),
        "viz_orbital_density" => typed(params, tools::viz_advanced::orbital_density),

        // ====================================================================
        // Watchtower Tools (9)
        // ====================================================================
        "watchtower_sessions_list" => {
            watchtower_wrap(tools::watchtower::watchtower_sessions_list())
        }
        "watchtower_active_sessions" => {
            watchtower_wrap(tools::watchtower::watchtower_active_sessions())
        }
        "watchtower_analyze" => {
            let p: params::WatchtowerAnalyzeParams = deser(params)?;
            watchtower_wrap(tools::watchtower::watchtower_analyze(
                p.session_path.as_deref(),
            ))
        }
        "watchtower_telemetry_stats" => {
            watchtower_wrap(tools::watchtower::watchtower_telemetry_stats())
        }
        "watchtower_recent" => {
            let p: params::WatchtowerRecentParams = deser(params)?;
            watchtower_wrap(tools::watchtower::watchtower_recent(
                p.count,
                p.session_filter.as_deref(),
            ))
        }
        "watchtower_symbol_audit" => {
            let p: params::WatchtowerSymbolAuditParams = deser(params)?;
            watchtower_wrap(tools::watchtower::watchtower_symbol_audit(&p.path))
        }
        "watchtower_gemini_stats" => watchtower_wrap(tools::watchtower::watchtower_gemini_stats()),
        "watchtower_gemini_recent" => {
            let p: params::WatchtowerGeminiRecentParams = deser(params)?;
            watchtower_wrap(tools::watchtower::watchtower_gemini_recent(p.count))
        }
        "watchtower_unified" => {
            let p: params::WatchtowerUnifiedParams = deser(params)?;
            watchtower_wrap(tools::watchtower::watchtower_unified(
                p.include_claude,
                p.include_gemini,
            ))
        }

        // ====================================================================
        // Node Hunter Tools (2)
        // ====================================================================
        "node_hunt_scan" => typed(params, tools::node_hunter::scan),
        "node_hunt_isolate" => typed(params, tools::node_hunter::isolate),

        // ====================================================================
        // Telemetry Intelligence Tools (6)
        // ====================================================================
        "telemetry_sources_list" => typed(params, tools::telemetry_intel::sources_list),
        "telemetry_source_analyze" => typed(params, tools::telemetry_intel::source_analyze),
        "telemetry_governance_crossref" => {
            typed(params, tools::telemetry_intel::governance_crossref)
        }
        "telemetry_snapshot_evolution" => typed(params, tools::telemetry_intel::snapshot_evolution),
        "telemetry_intel_report" => typed(params, tools::telemetry_intel::intel_report),
        "telemetry_recent" => typed(params, tools::telemetry_intel::recent_activity),

        // ====================================================================
        // Primitive Scanner (2)
        // ====================================================================
        "primitive_scan" => typed(params, tools::primitive_scanner::primitive_scan),
        "primitive_batch_test" => typed(params, tools::primitive_scanner::primitive_batch_test),

        // ====================================================================
        // Algovigilance (6)
        // ====================================================================
        "algovigil_dedup_pair" => typed(params, tools::algovigilance::dedup_pair),
        "algovigil_dedup_batch" => typed(params, tools::algovigilance::dedup_batch),
        "algovigil_triage_decay" => typed(params, tools::algovigilance::triage_decay),
        "algovigil_triage_reinforce" => typed(params, tools::algovigilance::triage_reinforce),
        "algovigil_triage_queue" => typed(params, tools::algovigilance::triage_queue),
        "algovigil_status" => tools::algovigilance::status(),

        // ====================================================================
        // Decision Tree Engine (6)
        // ====================================================================
        "dtree_train" => typed(params, tools::dtree::dtree_train),
        "dtree_predict" => typed(params, tools::dtree::dtree_predict),
        "dtree_importance" => typed(params, tools::dtree::dtree_importance),
        "dtree_prune" => typed(params, tools::dtree::dtree_prune),
        "dtree_export" => typed(params, tools::dtree::dtree_export),
        "dtree_info" => typed(params, tools::dtree::dtree_info),

        // ====================================================================
        // Edit Distance Framework (5)
        // ====================================================================
        "edit_distance_compute" => typed(params, tools::edit_distance::edit_distance_compute),
        "edit_distance_similarity" => typed(params, tools::edit_distance::edit_distance_similarity),
        "edit_distance_traceback" => typed(params, tools::edit_distance::edit_distance_traceback),
        "edit_distance_transfer" => typed(params, tools::edit_distance::edit_distance_transfer),
        "edit_distance_batch" => typed(params, tools::edit_distance::edit_distance_batch),

        // ====================================================================
        // Integrity Assessment (3)
        // ====================================================================
        "integrity_analyze" => typed(params, tools::integrity::integrity_analyze),
        "integrity_assess_ksb" => typed(params, tools::integrity::integrity_assess_ksb),
        "integrity_calibration" => typed(params, tools::integrity::integrity_calibration),

        // ====================================================================
        // MCP Telemetry Tools (4) - feature-gated
        // ====================================================================
        #[cfg(feature = "telemetry")]
        "telemetry_summary" => typed_async(params, tools::telemetry::telemetry_summary).await,
        #[cfg(feature = "telemetry")]
        "telemetry_by_tool" => typed_async(params, tools::telemetry::telemetry_by_tool).await,
        #[cfg(feature = "telemetry")]
        "telemetry_slow_calls" => typed_async(params, tools::telemetry::telemetry_slow_calls).await,
        #[cfg(feature = "telemetry")]
        "audit_trail" => typed_async(params, tools::telemetry::audit_trail).await,

        // ====================================================================
        // Sentinel — SSH brute-force protection (4)
        // ====================================================================
        "sentinel_status" => tools::sentinel::sentinel_status(),
        "sentinel_check_ip" => typed(params, tools::sentinel::sentinel_check_ip),
        "sentinel_parse_line" => typed(params, tools::sentinel::sentinel_parse_line),
        "sentinel_config_defaults" => typed(params, tools::sentinel::sentinel_config_defaults),

        // ====================================================================
        // Organize — 8-step file organization pipeline (6)
        // ====================================================================
        "organize_analyze" => typed(params, tools::organize::organize_analyze),
        "organize_config_default" => typed(params, tools::organize::organize_config_default),
        "organize_report_markdown" => typed(params, tools::organize::organize_report_markdown),
        "organize_report_json" => typed(params, tools::organize::organize_report_json),
        "organize_observe" => typed(params, tools::organize::organize_observe),
        "organize_rank" => typed(params, tools::organize::organize_rank),

        // ====================================================================
        // Measure — Workspace quality measurement (7)
        // ====================================================================
        "measure_crate" => typed(params, tools::measure::measure_crate_tool),
        "measure_workspace" => tools::measure::measure_workspace_tool(),
        "measure_entropy" => typed(params, tools::measure::measure_entropy_tool),
        "measure_graph" => tools::measure::measure_graph_tool(),
        "measure_drift" => typed(params, tools::measure::measure_drift_tool),
        "measure_compare" => typed(params, tools::measure::measure_compare_tool),
        "measure_stats" => typed(params, tools::measure::measure_stats_tool),
        "quality_gradient" => typed(params, tools::measure::quality_gradient_tool),

        // ====================================================================
        // Anatomy — Workspace structural analysis (4)
        // ====================================================================
        "anatomy_health" => tools::anatomy::anatomy_health_tool(),
        "anatomy_blast_radius" => typed(params, tools::anatomy::anatomy_blast_radius_tool),
        "anatomy_chomsky" => typed(params, tools::anatomy::anatomy_chomsky_tool),
        "anatomy_violations" => tools::anatomy::anatomy_violations_tool(),

        // ====================================================================
        // Cargo — Structured build/check/test/lint/fmt/tree (6)
        // ====================================================================
        "cargo_check" => typed(params, tools::cargo::cargo_check),
        "cargo_build" => typed(params, tools::cargo::cargo_build),
        "cargo_test" => typed(params, tools::cargo::cargo_test),
        "cargo_clippy" => typed(params, tools::cargo::cargo_clippy),
        "cargo_fmt" => typed(params, tools::cargo::cargo_fmt),
        "cargo_tree" => typed(params, tools::cargo::cargo_tree),

        // ====================================================================
        // Rust Development — Error types, derives, match gen, borrow explain (4)
        // ====================================================================
        "rust_dev_error_type" => typed(params, tools::rust_dev::error_type),
        "rust_dev_derive_advisor" => typed(params, tools::rust_dev::derive_advisor),
        "rust_dev_match_generate" => typed(params, tools::rust_dev::match_generate),
        "rust_dev_borrow_explain" => typed(params, tools::rust_dev::borrow_explain),
        "rust_dev_clippy_explain" => typed(params, tools::rust_dev::clippy_explain),
        "rust_dev_rustc_explain" => typed(params, tools::rust_dev::rustc_explain),
        "rust_dev_unsafe_audit" => typed(params, tools::rust_dev::unsafe_audit),
        "rust_dev_cargo_expand" => typed(params, tools::rust_dev::cargo_expand),
        "rust_dev_cargo_bloat" => typed(params, tools::rust_dev::cargo_bloat),
        "rust_dev_cargo_miri" => typed(params, tools::rust_dev::cargo_miri),
        "rust_dev_edition_migrate" => typed(params, tools::rust_dev::edition_migrate),
        "rust_dev_invocations" => typed(params, tools::rust_dev::invocations),

        // ====================================================================
        // SQI — Skill Quality Index (2)
        // ====================================================================
        "sqi_score" => typed(params, tools::sqi::sqi_score),
        "sqi_ecosystem" => typed(params, tools::sqi::sqi_ecosystem),

        // ====================================================================
        // State Operating System (SOS) Tools (10)
        // ====================================================================
        "sos_create" => typed(params, tools::sos::sos_create),
        "sos_transition" => typed(params, tools::sos::sos_transition),
        "sos_state" => typed(params, tools::sos::sos_state),
        "sos_history" => typed(params, tools::sos::sos_history),
        "sos_validate" => typed(params, tools::sos::sos_validate),
        "sos_list" => typed(params, tools::sos::sos_list),
        "sos_cycles" => typed(params, tools::sos::sos_cycles),
        "sos_audit" => typed(params, tools::sos::sos_audit),
        "sos_schedule" => typed(params, tools::sos::sos_schedule),
        "sos_route" => typed(params, tools::sos::sos_route),

        // ====================================================================
        // Cortex Tools (7) — Local LLM Inference
        // ====================================================================
        "cortex_download_model" => typed(params, tools::cortex::cortex_download_model),
        "cortex_list_models" => typed(params, tools::cortex::cortex_list_models),
        "cortex_model_info" => typed(params, tools::cortex::cortex_model_info),
        "cortex_generate" => typed(params, tools::cortex::cortex_generate),
        "cortex_embed" => typed(params, tools::cortex::cortex_embed),
        "cortex_fine_tune_status" => typed(params, tools::cortex::cortex_fine_tune_status),

        // ====================================================================
        // Monitoring Tools (5) — Development system health + Phase 4 surveillance
        // ====================================================================
        "monitoring_health_check" => tools::monitoring::health_check(),
        "monitoring_alerts" => typed(params, tools::monitoring::alerts),
        "monitoring_hook_health" => typed(params, tools::monitoring::hook_health),
        "monitoring_signal_digest" => typed(params, tools::monitoring::signal_digest),
        "phase4_surveillance_tick" => typed(params, tools::monitoring::phase4_surveillance_tick),

        // ====================================================================
        // Security Classification Tools (5) — 5-Level Clearance System
        // T1 Grounding: ∂ (boundary) + ς (state) + κ (comparison) + π (persistence)
        // ====================================================================
        "clearance_evaluate" => typed(params, tools::clearance::clearance_evaluate),
        "clearance_policy_for" => typed(params, tools::clearance::clearance_policy_for),
        "clearance_validate_change" => typed(params, tools::clearance::clearance_validate_change),
        "clearance_level_info" => typed(params, tools::clearance::clearance_level_info),
        "clearance_config" => tools::clearance::clearance_config(),

        // ====================================================================
        // Secure Boot Tools (3) — TPM-style Measured Boot Chain
        // T1 Grounding: σ (sequence) + → (causality) + ∂ (boundary) + ∝ (irreversibility) + κ (comparison)
        // ====================================================================
        "secure_boot_status" => typed(params, tools::secure_boot::secure_boot_status),
        "secure_boot_verify" => typed(params, tools::secure_boot::secure_boot_verify),
        "secure_boot_quote" => typed(params, tools::secure_boot::secure_boot_quote),

        // ====================================================================
        // User Management Tools (8) — Authentication & Access Control
        // T1 Grounding: ∂ (boundary) + κ (comparison) + ς (state) + μ (mapping) + π (persistence)
        // ====================================================================
        "user_create" => typed(params, tools::user::user_create),
        "user_login" => typed(params, tools::user::user_login),
        "user_logout" => typed(params, tools::user::user_logout),
        "user_list" => Ok(tools::user::user_list()),
        "user_lock" => typed(params, tools::user::user_lock),
        "user_unlock" => typed(params, tools::user::user_unlock),
        "user_status" => Ok(tools::user::user_status()),
        "user_change_password" => typed(params, tools::user::user_change_password),

        // ====================================================================
        // Claude FS Tools (9) — Filesystem CRUD on ~/.claude/
        // T3 Grounding: π (persistence) + σ (sequence) + ∂ (boundary) + μ (mapping)
        // ====================================================================
        "claude_fs_list" => typed_async(params, tools::claude_fs::claude_fs_list).await,
        "claude_fs_read" => typed_async(params, tools::claude_fs::claude_fs_read).await,
        "claude_fs_write" => typed_async(params, tools::claude_fs::claude_fs_write).await,
        "claude_fs_delete" => typed_async(params, tools::claude_fs::claude_fs_delete).await,
        "claude_fs_search" => typed(params, tools::claude_fs::claude_fs_search),
        "claude_fs_tail" => typed_async(params, tools::claude_fs::claude_fs_tail).await,
        "claude_fs_diff" => typed_async(params, tools::claude_fs::claude_fs_diff).await,
        "claude_fs_stat" => typed_async(params, tools::claude_fs::claude_fs_stat).await,
        "claude_fs_backup_now" => tools::claude_fs::claude_fs_backup_now(),

        // ====================================================================
        // Compendious Tools (5) — Text Density Optimization
        // T3 Grounding: N (quantity) + κ (comparison) + σ (sequence) + μ (mapping)
        // ====================================================================
        "score_text" => typed(params, tools::compendious::score_text),
        "compress_text" => typed(params, tools::compendious::compress_text),
        "compare_texts" => typed(params, tools::compendious::compare_texts),
        "analyze_patterns" => typed(params, tools::compendious::analyze_patterns),
        "get_domain_target" => typed(params, tools::compendious::get_domain_target),

        // ====================================================================
        // Docs Claude Tools (4) — Claude Code Documentation
        // T3 Grounding: π (persistence) + μ (mapping) + σ (sequence) + ∂ (boundary)
        // ====================================================================
        "docs_claude_list_pages" => typed(params, tools::docs_claude::docs_claude_list_pages),
        "docs_claude_get_page" => {
            typed_async(params, tools::docs_claude::docs_claude_get_page).await
        }
        "docs_claude_search" => typed(params, tools::docs_claude::docs_claude_search),
        "docs_claude_index" => tools::docs_claude::docs_claude_index().await,

        // ====================================================================
        // Google Sheets Tools (7) — Sheets API v4
        // T3 Grounding: π (persistence) + μ (mapping) + σ (sequence) + ∂ (boundary)
        // ====================================================================
        "gsheets_list_sheets" => typed_async(params, tools::gsheets::gsheets_list_sheets).await,
        "gsheets_read_range" => typed_async(params, tools::gsheets::gsheets_read_range).await,
        "gsheets_batch_read" => typed_async(params, tools::gsheets::gsheets_batch_read).await,
        "gsheets_write_range" => typed_async(params, tools::gsheets::gsheets_write_range).await,
        "gsheets_append" => typed_async(params, tools::gsheets::gsheets_append).await,
        "gsheets_metadata" => typed_async(params, tools::gsheets::gsheets_metadata).await,
        "gsheets_search" => typed_async(params, tools::gsheets::gsheets_search).await,

        // ====================================================================
        // Reddit Tools (7) — Reddit API + Value Signal Detection
        // T3 Grounding: μ (mapping) + σ (sequence) + ∂ (boundary) + ς (state) + ν (frequency)
        // ====================================================================
        "reddit_status" => tools::reddit::reddit_status(),
        "reddit_authenticate" => tools::reddit::reddit_authenticate().await,
        "reddit_hot_posts" => typed_async(params, tools::reddit::reddit_hot_posts).await,
        "reddit_new_posts" => typed_async(params, tools::reddit::reddit_new_posts).await,
        "reddit_subreddit_info" => typed_async(params, tools::reddit::reddit_subreddit_info).await,
        "reddit_detect_signals" => typed_async(params, tools::reddit::reddit_detect_signals).await,
        "reddit_search_entity" => typed_async(params, tools::reddit::reddit_search_entity).await,

        // ====================================================================
        // Trust Tools (8) — Bayesian Trust Engine + Patient Safety
        // T3 Grounding: ς (state) + ∂ (boundary) + N (quantity) + → (causality) + κ (comparison)
        // ====================================================================
        "trust_score" => typed(params, tools::trust::trust_score),
        "trust_record" => typed(params, tools::trust::trust_record),
        "trust_snapshot" => typed(params, tools::trust::trust_snapshot),
        "trust_decide" => typed(params, tools::trust::trust_decide),
        "trust_harm_weight" => typed(params, tools::trust::trust_harm_weight),
        "trust_velocity" => typed(params, tools::trust::trust_velocity),
        "trust_multi_score" => typed(params, tools::trust::trust_multi_score),
        "trust_network_chain" => typed(params, tools::trust::trust_network_chain),

        // --- Molecular Weight (4 tools) ---
        "mw_compute" => typed(params, tools::molecular_weight::mw_compute),
        "mw_periodic_table" => tools::molecular_weight::mw_periodic_table(),
        "mw_compare" => typed(params, tools::molecular_weight::mw_compare),
        "mw_predict_transfer" => typed(params, tools::molecular_weight::mw_predict_transfer),

        // Primitive Trace (1 tool — concept interaction mapping)
        "primitive_trace" => typed(params, tools::primitive_trace::primitive_trace),

        // --- Text Transform (5 tools) ---
        "transform_list_profiles" => tools::transform::transform_list_profiles(),
        "transform_get_profile" => typed(params, tools::transform::transform_get_profile),
        "transform_segment" => typed(params, tools::transform::transform_segment),
        "transform_compile_plan" => typed(params, tools::transform::transform_compile_plan),
        "transform_score_fidelity" => typed(params, tools::transform::transform_score_fidelity),

        // ====================================================================
        // Insight Engine Tools (14) — Persistent Pattern Detection, Novelty, Connection, Compression
        // T1 Grounding: INSIGHT ≡ ⟨σ, κ, μ, ∃, ς, ∅, N, ∂⟩
        // State: ς-acc (accumulated, persisted to ~/.claude/brain/insight/)
        // Domain-level: engine.json | System-level: system.json
        // ====================================================================
        "insight_ingest" => typed(params, tools::insight::insight_ingest),
        "insight_status" => typed(params, tools::insight::insight_status),
        "insight_config" => typed(params, tools::insight::insight_config),
        "insight_connect" => typed(params, tools::insight::insight_connect),
        "insight_compress" => typed(params, tools::insight::insight_compress),
        "insight_compress_auto" => typed(params, tools::insight::insight_compress_auto),
        "insight_patterns" => typed(params, tools::insight::insight_patterns),
        "insight_reset" => typed(params, tools::insight::insight_reset),
        // System-level (NexCoreInsight compositor — multi-domain broadcast)
        "insight_system_status" => typed(params, tools::insight::insight_system_status),
        "insight_system_ingest" => typed(params, tools::insight::insight_system_ingest),
        "insight_system_register" => typed(params, tools::insight::insight_system_register),
        "insight_system_reset" => typed(params, tools::insight::insight_system_reset),
        // Read-side query tools
        "insight_query" => typed(params, tools::insight::insight_query),
        "insight_novelties" => typed(params, tools::insight::insight_novelties),

        // ====================================================================
        // Caesura Detection (3) — Structural seam detection in codebases
        // T1 Grounding: ∂(boundary) ς(state) ∝(irreversibility) ν(frequency)
        // ====================================================================
        "caesura_scan" => typed(params, tools::caesura::caesura_scan_tool),
        "caesura_metrics" => typed(params, tools::caesura::caesura_metrics_tool),
        "caesura_report" => typed(params, tools::caesura::caesura_report_tool),

        // --- Declension System (4 tools) ---
        // T1 Grounding: ∂(boundary) ς(state) μ(mapping) ∅(void) ×(product)
        // ====================================================================
        "declension_classify" => typed(params, tools::declension::declension_classify_tool),
        "declension_inflect" => typed(params, tools::declension::declension_inflect_tool),
        "declension_agree" => typed(params, tools::declension::declension_agree_tool),
        "declension_prodrop" => typed(params, tools::declension::declension_prodrop_tool),

        // ====================================================================
        // Vigil System Tools (8) — π(∂·ν)|∝ Vigilance Engine
        // T1 Grounding: π (Persistence) + ∂ (Boundary) + ν (Frequency) + ∝ (Irreversibility)
        // ====================================================================
        "vigil_sys_start" => typed(params, tools::vigil_system::vigil_sys_start),
        "vigil_sys_stop" => tools::vigil_system::vigil_sys_stop(),
        "vigil_sys_status" => tools::vigil_system::vigil_sys_status(),
        "vigil_sys_boundaries" => tools::vigil_system::vigil_sys_boundaries(),
        "vigil_sys_add_boundary" => typed(params, tools::vigil_system::vigil_sys_add_boundary),
        "vigil_sys_ledger_query" => typed(params, tools::vigil_system::vigil_sys_ledger_query),
        "vigil_sys_ledger_verify" => tools::vigil_system::vigil_sys_ledger_verify(),
        "vigil_sys_stats" => tools::vigil_system::vigil_sys_stats(),

        // --- Counter-Awareness (5 tools) ---
        "ca_detect" => typed(params, tools::counter_awareness::detect),
        "ca_fusion" => typed(params, tools::counter_awareness::fusion),
        "ca_optimize" => typed(params, tools::counter_awareness::optimize),
        "ca_matrix" => typed(params, tools::counter_awareness::matrix),
        "ca_catalog" => typed(params, tools::counter_awareness::catalog_list),

        // ====================================================================
        // DNA — DNA-based Computation (9 tools)
        // encode/decode text, eval on Codon VM, tile/voxel viz, PV signals
        // T1 Grounding: DNA ≡ ⟨σ, μ, ∂, ς, N, κ, →, ∃⟩
        // ====================================================================
        "dna_encode" => typed(params, tools::dna::dna_encode),
        "dna_decode" => typed(params, tools::dna::dna_decode),
        "dna_eval" => typed(params, tools::dna::dna_eval),
        "dna_compile_asm" => typed(params, tools::dna::dna_compile_asm),
        "dna_tile" => typed(params, tools::dna::dna_tile),
        "dna_voxel" => typed(params, tools::dna::dna_voxel),
        "dna_pv_signal" => typed(params, tools::dna::dna_pv_signal),
        "dna_profile_drug" => typed(params, tools::dna::dna_profile_drug),
        "dna_catalog" => tools::dna::dna_catalog(),
        "dna_nexcore_genome" => tools::dna::dna_nexcore_genome(),

        // ====================================================================
        // Lessons Learned Tools (6) — Persistent Lesson Storage
        // T3 Grounding: π (persistence) + μ (mapping) + σ (sequence) + ∃ (existence)
        // ====================================================================
        "lesson_add" => typed(params, tools::lessons::lesson_add),
        "lesson_get" => typed(params, tools::lessons::lesson_get),
        "lesson_search" => typed(params, tools::lessons::lesson_search),
        "lesson_by_context" => typed(params, tools::lessons::lesson_by_context),
        "lesson_by_tag" => typed(params, tools::lessons::lesson_by_tag),
        "primitives_summary" => tools::lessons::primitives_summary(),

        // ====================================================================
        // Claude REPL Tool (1) — CLI Bridge
        // T3 Grounding: σ (sequence) + ∂ (boundary) + ς (state) + → (causality)
        // ====================================================================
        "claude_repl" => typed_async(params, tools::claude_repl::claude_repl).await,

        // ====================================================================
        // Adventure HUD Tools (6) — Session Tracking
        // T3 Grounding: ς (state) + σ (sequence) + μ (mapping) + N (quantity)
        // ====================================================================
        "adventure_start" => typed(params, tools::adventure::adventure_start),
        "adventure_task" => typed(params, tools::adventure::adventure_task),
        "adventure_skill" => typed(params, tools::adventure::adventure_skill),
        "adventure_measure" => typed(params, tools::adventure::adventure_measure),
        "adventure_milestone" => typed(params, tools::adventure::adventure_milestone),
        "adventure_status" => tools::adventure::adventure_status(),

        // ====================================================================
        // Borrow Miner Tools (4) — Ore Mining Game
        // T3 Grounding: ς (state) + N (quantity) + κ (comparison) + ∂ (boundary)
        // ====================================================================
        "mine" => tools::borrow_miner::mine(),
        "drop_ore" => tools::borrow_miner::drop_ore(),
        "get_state" => tools::borrow_miner::get_state(),
        "signal_check" => typed(params, tools::borrow_miner::signal_check),

        // ====================================================================
        // Reproductive System Tools (3)
        // ====================================================================
        "reproductive_guard_mutation" => typed(params, tools::reproductive::guard_mutation),
        "reproductive_specialize_agent" => typed(params, tools::reproductive::specialize_agent),
        "reproductive_start_mitosis" => typed(params, tools::reproductive::start_mitosis),

        // ====================================================================
        // Crew-Mode Orchestration Tools (3) — Multi-Agent Task Decomposition
        // T1 Grounding: σ (sequence) + μ (mapping) + Σ (sum) + × (product)
        // ====================================================================
        "crew_assign" => typed(params, tools::crew::crew_assign),
        "crew_task_status" => typed(params, tools::crew::crew_task_status),
        "crew_fuse_decisions" => typed(params, tools::crew::crew_fuse_decisions),

        // ====================================================================
        // Persistent Retrieval Pipeline Tools (3) — Unified Multi-Source Search
        // T1 Grounding: μ (mapping) + σ (sequence) + κ (comparison) + π (persistence) + ν (frequency)
        // ====================================================================
        "retrieval_query" => typed(params, tools::retrieval::retrieval_query),
        "retrieval_ingest" => typed(params, tools::retrieval::retrieval_ingest),
        "retrieval_stats" => typed(params, tools::retrieval::retrieval_stats),

        // ====================================================================
        // Human-in-the-Loop Pipeline Tools (4) — Decision Approval Queue
        // T1 Grounding: ∂ (boundary) + κ (comparison) + ς (state) + π (persistence) + → (causality)
        // ====================================================================
        "hitl_submit" => typed(params, tools::hitl::hitl_submit),
        "hitl_queue" => typed(params, tools::hitl::hitl_queue),
        "hitl_review" => typed(params, tools::hitl::hitl_review),
        "hitl_stats" => typed(params, tools::hitl::hitl_stats),

        // ====================================================================
        // PV Domain Embeddings Tools (3) — TF-IDF + Graph Similarity
        // T1 Grounding: μ (mapping) + κ (comparison) + N (quantity) + σ (sequence)
        // ====================================================================
        "pv_embedding_similarity" => typed(params, tools::pv_embeddings::pv_embedding_similarity),
        "pv_embedding_get" => typed(params, tools::pv_embeddings::pv_embedding_get),
        "pv_embedding_stats" => typed(params, tools::pv_embeddings::pv_embedding_stats),

        // ====================================================================
        // Proof of Meaning Tools (5)
        // ====================================================================
        "pom_distill" => typed(params, tools::proof_of_meaning::pom_distill),
        "pom_chromatograph" => typed(params, tools::proof_of_meaning::pom_chromatograph),
        "pom_titrate" => typed(params, tools::proof_of_meaning::pom_titrate),
        "pom_prove_equivalence" => typed(params, tools::proof_of_meaning::pom_prove_equivalence),
        "pom_registry_stats" => typed(params, tools::proof_of_meaning::pom_registry_stats),

        // ====================================================================
        // Statistical Drift Detection Tools (4) — KS, PSI, JSD, Composite
        // T1 Grounding: ν (frequency) + κ (comparison) + ∂ (boundary) + N (quantity)
        // Source: AI Engineering Bible Section 32 (Model Monitoring & Drift Detection)
        // ====================================================================
        "drift_ks_test" => typed(params, tools::drift_detection::drift_ks_test),
        "drift_psi" => typed(params, tools::drift_detection::drift_psi),
        "drift_jsd" => typed(params, tools::drift_detection::drift_jsd),
        "drift_detect" => typed(params, tools::drift_detection::drift_detect),

        // ====================================================================
        // Rate Limiter Tools (3) — Token Bucket, Sliding Window, Status
        // T1 Grounding: ν (frequency) + ∂ (boundary) + ς (state) + N (quantity)
        // Source: AI Engineering Bible Section 14 (API Development & Integration)
        // ====================================================================
        "rate_limit_token_bucket" => typed(params, tools::rate_limiter::rate_limit_token_bucket),
        "rate_limit_sliding_window" => {
            typed(params, tools::rate_limiter::rate_limit_sliding_window)
        }
        "rate_limit_status" => typed(params, tools::rate_limiter::rate_limit_status),

        // ====================================================================
        // Rank Fusion Tools (3) — RRF, Hybrid Interpolation, Borda Count
        // T1 Grounding: σ (sequence) + μ (mapping) + κ (comparison) + N (quantity)
        // Source: AI Engineering Bible Section 31 (Context & Retrieval Refinements)
        // ====================================================================
        "rank_fusion_rrf" => typed(params, tools::rank_fusion::rank_fusion_rrf),
        "rank_fusion_hybrid" => typed(params, tools::rank_fusion::rank_fusion_hybrid),
        "rank_fusion_borda" => typed(params, tools::rank_fusion::rank_fusion_borda),

        // Security Posture Assessment Tools (3)
        "security_posture_assess" => {
            typed(params, tools::security_posture::security_posture_assess)
        }
        "security_threat_readiness" => {
            typed(params, tools::security_posture::security_threat_readiness)
        }
        "security_compliance_gap" => {
            typed(params, tools::security_posture::security_compliance_gap)
        }

        // AI Observability Metrics Tools (3)
        "observability_record_latency" => {
            typed(params, tools::observability::observability_record_latency)
        }
        "observability_query" => typed(params, tools::observability::observability_query),
        "observability_freshness" => typed(params, tools::observability::observability_freshness),

        // ====================================================================
        // GROUNDED Tools (7) — Epistemological substrate
        // T1 Grounding: ×(Product) + N(Quantity) + ∂(Boundary) + σ(Sequence) + →(Causality)
        // Uncertainty tracking, evidence chains, confidence gating
        // ====================================================================
        "grounded_uncertain" => typed(params, tools::grounded::grounded_uncertain),
        "grounded_require" => typed(params, tools::grounded::grounded_require),
        "grounded_compose" => typed(params, tools::grounded::grounded_compose),
        "grounded_evidence_new" => typed(params, tools::grounded::grounded_evidence_new),
        "grounded_evidence_step" => typed(params, tools::grounded::grounded_evidence_step),
        "grounded_evidence_get" => typed(params, tools::grounded::grounded_evidence_get),
        "grounded_skill_assess" => typed(params, tools::grounded::grounded_skill_assess),

        // ====================================================================
        // Digital Highway Tools (8) — Infrastructure acceleration (Chatburn 1923)
        // T1 Grounding: κ(Comparison) + ∂(Boundary) + N(Quantity) + ν(Frequency) + Σ(Sum)
        // Classification, quality scoring, stress analysis, field doctrine,
        // traffic census, parallel planning, interchange merging, grade separation
        // ====================================================================
        "highway_classify" => typed(params, tools::highway::highway_classify),
        "highway_quality" => typed(params, tools::highway::highway_quality),
        "highway_destructive" => typed(params, tools::highway::highway_destructive),
        "highway_legitimate_field" => typed(params, tools::highway::highway_legitimate_field),
        "highway_traffic_census" => typed(params, tools::highway::highway_traffic_census),
        "highway_parallel_plan" => typed(params, tools::highway::highway_parallel_plan),
        "highway_interchange" => typed(params, tools::highway::highway_interchange),
        "highway_grade_separate" => typed(params, tools::highway::highway_grade_separate),

        // ====================================================================
        // Tool Routing (deterministic dispatch + DAG execution planning)
        // ====================================================================
        "tool_route" => typed(params, tools::routing::tool_route),
        "tool_dag" => typed(params, tools::routing::tool_dag),
        "tool_deps" => typed(params, tools::routing::tool_deps),
        "tool_chain" => typed(params, tools::routing::tool_chain),

        // ====================================================================
        // Validify Tools (3) — 8-gate crate validation (V-A-L-I-D-I-F-Y)
        // ====================================================================
        "validify_run" => typed(params, tools::validify::run),
        "validify_gate" => typed(params, tools::validify::gate),
        "validify_gates_list" => typed(params, tools::validify::gates_list),

        // ====================================================================
        // CTVP Tools (3) — Clinical Trial Validation Paradigm
        // ====================================================================
        "ctvp_score" => typed(params, tools::ctvp::score),
        "ctvp_five_problems" => typed(params, tools::ctvp::five_problems),
        "ctvp_phases_list" => typed(params, tools::ctvp::phases_list),

        // ====================================================================
        // Code Inspection Tools (3) — FDA-inspired audit
        // ====================================================================
        "code_inspect_audit" => typed(params, tools::code_inspect::audit),
        "code_inspect_score" => typed(params, tools::code_inspect::score),
        "code_inspect_criteria" => typed(params, tools::code_inspect::criteria),

        // ====================================================================
        // Primitive Coverage Tools (2) — T1 Lex Primitiva coverage
        // ====================================================================
        "primitive_coverage_check" => typed(params, tools::primitive_coverage::check),
        "primitive_coverage_rules" => typed(params, tools::primitive_coverage::rules),

        // ====================================================================
        // Model Delegation Tools (3) — task→model routing
        // ====================================================================
        "model_route" => typed(params, tools::model_delegation::route),
        "model_compare" => typed(params, tools::model_delegation::compare),
        "model_list" => typed(params, tools::model_delegation::list),

        // ====================================================================
        // Prompt Kinetics Tools (3) — ADME PK model for prompts
        // ====================================================================
        "prompt_kinetics_analyze" => typed(params, tools::prompt_kinetics::analyze),
        "prompt_bioavailability" => typed(params, tools::prompt_kinetics::bioavailability),
        "prompt_kinetics_model" => typed(params, tools::prompt_kinetics::model),

        // ====================================================================
        // Compounding Engine Tools (3) — learning velocity metrics
        // ====================================================================
        "compounding_velocity" => typed(params, tools::compounding_engine::velocity),
        "compounding_loop_health" => typed(params, tools::compounding_engine::loop_health),
        "compounding_metrics" => typed(params, tools::compounding_engine::metrics),

        // ====================================================================
        // Polymer Tools (3) — hook pipeline composition
        // ====================================================================
        "polymer_compose" => typed(params, tools::polymer::compose),
        "polymer_validate" => typed(params, tools::polymer::validate),
        "polymer_analyze" => typed(params, tools::polymer::analyze),

        // ====================================================================
        // BAS Nervous System (4) — signal routing, reflex arcs, myelination
        // ====================================================================
        "nervous_reflex" => typed(params, tools::nervous::reflex),
        "nervous_latency" => typed(params, tools::nervous::latency),
        "nervous_myelination" => typed(params, tools::nervous::myelination),
        "nervous_health" => typed(params, tools::nervous::health),

        // ====================================================================
        // BAS Cardiovascular System (4) — data transport, pressure, flow
        // ====================================================================
        "cardio_blood_pressure" => typed(params, tools::cardiovascular::blood_pressure),
        "cardio_blood_health" => typed(params, tools::cardiovascular::blood_health),
        "cardio_diagnose" => typed(params, tools::cardiovascular::diagnose),
        "cardio_vitals" => typed(params, tools::cardiovascular::vitals),

        // ====================================================================
        // BAS Lymphatic System (4) — overflow, quality, inspection
        // ====================================================================
        "lymphatic_drainage" => typed(params, tools::lymphatic::drainage),
        "lymphatic_thymic" => typed(params, tools::lymphatic::thymic_selection),
        "lymphatic_inspect" => typed(params, tools::lymphatic::inspect),
        "lymphatic_health" => typed(params, tools::lymphatic::health),

        // ====================================================================
        // BAS Respiratory System (4) — context window, gas exchange
        // ====================================================================
        "respiratory_exchange" => typed(params, tools::respiratory::exchange),
        "respiratory_dead_space" => typed(params, tools::respiratory::dead_space),
        "respiratory_tidal" => typed(params, tools::respiratory::tidal_volume),
        "respiratory_health" => typed(params, tools::respiratory::health),

        // ====================================================================
        // BAS Urinary System (4) — waste management, pruning, retention
        // ====================================================================
        "urinary_pruning" => typed(params, tools::urinary::pruning),
        "urinary_expiry" => typed(params, tools::urinary::expiry),
        "urinary_retention" => typed(params, tools::urinary::retention),
        "urinary_health" => typed(params, tools::urinary::health),

        // ====================================================================
        // BAS Integumentary System (5) — boundary, permissions, scarring
        // ====================================================================
        "integumentary_permission" => typed(params, tools::integumentary::permission),
        "integumentary_settings" => typed(params, tools::integumentary::settings),
        "integumentary_sandbox" => typed(params, tools::integumentary::sandbox),
        "integumentary_scarring" => typed(params, tools::integumentary::scarring),
        "integumentary_health" => typed(params, tools::integumentary::health),

        // ====================================================================
        // BAS Digestive System (3) — data pipeline processing
        // ====================================================================
        "digestive_process" => typed(params, tools::digestive::process),
        "digestive_taste" => typed(params, tools::digestive::taste),
        "digestive_health" => typed(params, tools::digestive::health),

        // ====================================================================
        // BAS Circulatory System (3) — data transport and routing
        // ====================================================================
        "circulatory_pump" => typed(params, tools::circulatory::pump),
        "circulatory_pressure" => typed(params, tools::circulatory::pressure),
        "circulatory_health" => typed(params, tools::circulatory::health),

        // ====================================================================
        // BAS Skeletal System (3) — structural knowledge framework
        // ====================================================================
        "skeletal_health" => typed(params, tools::skeletal::health),
        "skeletal_wolffs_law" => typed(params, tools::skeletal::wolffs_law),
        "skeletal_structure" => typed(params, tools::skeletal::structure),

        // ====================================================================
        // BAS Muscular System (3) — tool execution patterns
        // ====================================================================
        "muscular_classify" => typed(params, tools::muscular::classify),
        "muscular_fatigue" => typed(params, tools::muscular::fatigue),
        "muscular_health" => typed(params, tools::muscular::health),

        // ====================================================================
        // BAS Phenotype (2) — adversarial test generation
        // ====================================================================
        "phenotype_mutate" => typed(params, tools::phenotype::phenotype_mutate),
        "phenotype_verify" => typed(params, tools::phenotype::phenotype_verify),

        // ====================================================================
        // Kellnr PK (6 — pharmacokinetics)
        // ====================================================================
        "kellnr_compute_pk_auc" => typed(params, tools::kellnr_pk::compute_pk_auc),
        "kellnr_compute_pk_steady_state" => {
            typed(params, tools::kellnr_pk::compute_pk_steady_state)
        }
        "kellnr_compute_pk_ionization" => typed(params, tools::kellnr_pk::compute_pk_ionization),
        "kellnr_compute_pk_clearance" => typed(params, tools::kellnr_pk::compute_pk_clearance),
        "kellnr_compute_pk_volume_distribution" => {
            typed(params, tools::kellnr_pk::compute_pk_volume_distribution)
        }
        "kellnr_compute_pk_michaelis_menten" => {
            typed(params, tools::kellnr_pk::compute_pk_michaelis_menten)
        }

        // ====================================================================
        // Kellnr Thermo (4 — thermodynamics)
        // ====================================================================
        "kellnr_compute_thermo_gibbs" => typed(params, tools::kellnr_thermo::compute_thermo_gibbs),
        "kellnr_compute_thermo_kd" => typed(params, tools::kellnr_thermo::compute_thermo_kd),
        "kellnr_compute_thermo_binding_affinity" => typed(
            params,
            tools::kellnr_thermo::compute_thermo_binding_affinity,
        ),
        "kellnr_compute_thermo_arrhenius" => {
            typed(params, tools::kellnr_thermo::compute_thermo_arrhenius)
        }

        // ====================================================================
        // Kellnr Stats (5 — advanced statistics)
        // ====================================================================
        "kellnr_compute_stats_welch_ttest" => {
            typed(params, tools::kellnr_stats::compute_stats_welch_ttest)
        }
        "kellnr_compute_stats_ols_regression" => {
            typed(params, tools::kellnr_stats::compute_stats_ols_regression)
        }
        "kellnr_compute_stats_poisson_ci" => {
            typed(params, tools::kellnr_stats::compute_stats_poisson_ci)
        }
        "kellnr_compute_stats_bayesian_posterior" => typed(
            params,
            tools::kellnr_stats::compute_stats_bayesian_posterior,
        ),
        "kellnr_compute_stats_entropy" => typed(params, tools::kellnr_stats::compute_stats_entropy),

        // ====================================================================
        // Kellnr Graph (4 — graph theory)
        // ====================================================================
        "kellnr_compute_graph_betweenness" => {
            typed(params, tools::kellnr_graph::compute_graph_betweenness)
        }
        "kellnr_compute_graph_mutual_info" => {
            typed(params, tools::kellnr_graph::compute_graph_mutual_info)
        }
        "kellnr_compute_graph_tarjan_scc" => {
            typed(params, tools::kellnr_graph::compute_graph_tarjan_scc)
        }
        "kellnr_compute_graph_topsort" => typed(params, tools::kellnr_graph::compute_graph_topsort),

        // ====================================================================
        // Kellnr Decision Trees (3)
        // ====================================================================
        "kellnr_compute_dtree_feature_importance" => typed(
            params,
            tools::kellnr_dtree::compute_dtree_feature_importance,
        ),
        "kellnr_compute_dtree_prune" => typed(params, tools::kellnr_dtree::compute_dtree_prune),
        "kellnr_compute_dtree_to_rules" => {
            typed(params, tools::kellnr_dtree::compute_dtree_to_rules)
        }

        // ====================================================================
        // Kellnr Surveillance (3 — sequential signal detection)
        // ====================================================================
        "kellnr_compute_signal_sprt" => typed(params, tools::kellnr_signal::compute_signal_sprt),
        "kellnr_compute_signal_cusum" => typed(params, tools::kellnr_signal::compute_signal_cusum),
        "kellnr_compute_signal_weibull_tto" => {
            typed(params, tools::kellnr_signal::compute_signal_weibull_tto)
        }

        // ====================================================================
        // Kellnr Registry (15 — async HTTP)
        // ====================================================================
        "kellnr_search_crates" => typed_async(params, tools::kellnr_registry::search_crates).await,
        "kellnr_get_crate_metadata" => {
            typed_async(params, tools::kellnr_registry::get_crate_metadata).await
        }
        "kellnr_list_crate_versions" => {
            typed_async(params, tools::kellnr_registry::list_crate_versions).await
        }
        "kellnr_get_version_details" => {
            typed_async(params, tools::kellnr_registry::get_version_details).await
        }
        "kellnr_list_owners" => typed_async(params, tools::kellnr_registry::list_owners).await,
        "kellnr_add_owner" => typed_async(params, tools::kellnr_registry::add_owner).await,
        "kellnr_remove_owner" => typed_async(params, tools::kellnr_registry::remove_owner).await,
        "kellnr_yank_version" => typed_async(params, tools::kellnr_registry::yank_version).await,
        "kellnr_unyank_version" => {
            typed_async(params, tools::kellnr_registry::unyank_version).await
        }
        "kellnr_list_all_crates" => {
            typed_async(params, tools::kellnr_registry::list_all_crates).await
        }
        "kellnr_get_dependencies" => {
            typed_async(params, tools::kellnr_registry::get_dependencies).await
        }
        "kellnr_get_dependents" => {
            typed_async(params, tools::kellnr_registry::get_dependents).await
        }
        "kellnr_health_check" => {
            // No params — just call directly
            tools::kellnr_registry::health_check().await
        }
        "kellnr_download_crate" => {
            typed_async(params, tools::kellnr_registry::download_crate).await
        }
        "kellnr_registry_stats" => {
            // No params — just call directly
            tools::kellnr_registry::registry_stats().await
        }

        // ====================================================================
        // Stoichiometry Tools (5) — encode/decode concepts as balanced equations
        // ====================================================================
        "stoichiometry_encode" => typed(params, tools::stoichiometry::encode),
        "stoichiometry_decode" => typed(params, tools::stoichiometry::decode),
        "stoichiometry_sisters" => typed(params, tools::stoichiometry::sisters),
        "stoichiometry_mass_state" => typed(params, tools::stoichiometry::mass_state),
        "stoichiometry_dictionary" => typed(params, tools::stoichiometry::dictionary),

        // ====================================================================
        // Observatory Phase 9 — Graph Layout, Career Transitions, Learning DAG
        // ====================================================================
        "graph_layout_converge" => typed(params, tools::graph_layout::converge),
        "career_transitions" => typed(params, tools::career::transitions),
        "learning_dag_resolve" => typed(params, tools::learning_dag::resolve),

        // ====================================================================
        // TRIAL Framework (10) — universal experimentation (FDA clinical trial methodology)
        // ====================================================================
        "trial_protocol_register" => typed(params, tools::trial::protocol_register),
        "trial_power_analysis" => typed(params, tools::trial::power_analysis),
        "trial_randomize" => typed(params, tools::trial::randomize),
        "trial_blind_verify" => typed(params, tools::trial::blind_verify),
        "trial_interim_analyze" => typed(params, tools::trial::interim_analyze),
        "trial_safety_check" => typed(params, tools::trial::safety_check),
        "trial_endpoint_evaluate" => typed(params, tools::trial::endpoint_evaluate),
        "trial_multiplicity_adjust" => typed(params, tools::trial::multiplicity_adjust),
        "trial_adapt_decide" => typed(params, tools::trial::adapt_decide),
        "trial_report_generate" => typed(params, tools::trial::report_generate),

        // ====================================================================
        // Cognition Tools (8) — Transformer Algorithm as Strict Rust
        // ====================================================================
        "cognition_process" => typed(params, tools::cognition::cognition_process),
        "cognition_analyze" => typed(params, tools::cognition::cognition_analyze),
        "cognition_forward" => typed(params, tools::cognition::cognition_forward),
        "cognition_entropy" => typed(params, tools::cognition::cognition_entropy),
        "cognition_perplexity" => typed(params, tools::cognition::cognition_perplexity),
        "cognition_embed" => typed(params, tools::cognition::cognition_embed),
        "cognition_sample" => typed(params, tools::cognition::cognition_sample),
        "cognition_confidence" => typed(params, tools::cognition::cognition_confidence),

        // ====================================================================
        // Foundry Tools (5)
        // ====================================================================
        "foundry_validate_artifact" => typed(params, tools::foundry::foundry_validate_artifact),
        "foundry_cascade_validate" => typed(params, tools::foundry::foundry_cascade_validate),
        "foundry_render_intelligence" => typed(params, tools::foundry::foundry_render_intelligence),
        "foundry_vdag_order" => typed(params, tools::foundry::foundry_vdag_order),
        "foundry_infer" => typed(params, tools::foundry::foundry_infer),

        // ====================================================================
        // Chemivigilance Tools (15) — SMILES, descriptors, QSAR, SafetyBrief
        // ====================================================================
        "chem_parse_smiles" => typed(params, tools::chemivigilance::chem_parse_smiles),
        "chem_descriptors" => typed(params, tools::chemivigilance::chem_descriptors),
        "chem_fingerprint" => typed(params, tools::chemivigilance::chem_fingerprint),
        "chem_similarity" => typed(params, tools::chemivigilance::chem_similarity),
        "chem_structural_alerts" => typed(params, tools::chemivigilance::chem_structural_alerts),
        "chem_predict_toxicity" => typed(params, tools::chemivigilance::chem_predict_toxicity),
        "chem_predict_metabolites" => {
            typed(params, tools::chemivigilance::chem_predict_metabolites)
        }
        "chem_predict_degradants" => typed(params, tools::chemivigilance::chem_predict_degradants),
        "chem_safety_brief" => typed(params, tools::chemivigilance::chem_safety_brief),
        "chem_substructure" => typed(params, tools::chemivigilance::chem_substructure),
        "chem_watchlist" => typed(params, tools::chemivigilance::chem_watchlist),
        "chem_alert_library" => typed(params, tools::chemivigilance::chem_alert_library),
        "chem_ring_scan" => typed(params, tools::chemivigilance::chem_ring_scan),
        "chem_aromaticity" => typed(params, tools::chemivigilance::chem_aromaticity),
        "chem_molecular_formula" => typed(params, tools::chemivigilance::chem_molecular_formula),

        // ====================================================================
        // QSAR Granular Predictions (4) — per-endpoint toxicity + domain
        // ====================================================================
        "chem_predict_mutagenicity" => typed(params, tools::qsar::chem_predict_mutagenicity),
        "chem_predict_hepatotoxicity" => typed(params, tools::qsar::chem_predict_hepatotoxicity),
        "chem_predict_cardiotoxicity" => typed(params, tools::qsar::chem_predict_cardiotoxicity),
        "chem_assess_applicability_domain" => {
            typed(params, tools::qsar::chem_assess_applicability_domain)
        }

        // ====================================================================
        // PV Pharmacokinetics (6) — AUC, clearance, half-life, steady-state, ionization, Michaelis-Menten
        // ====================================================================
        "pk_auc" => typed(params, tools::pk::pk_auc),
        "pk_clearance" => typed(params, tools::pk::pk_clearance),
        "pk_half_life" => typed(params, tools::pk::pk_half_life),
        "pk_steady_state" => typed(params, tools::pk::pk_steady_state),
        "pk_ionization" => typed(params, tools::pk::pk_ionization),
        "pk_michaelis_menten" => typed(params, tools::pk::pk_michaelis_menten),

        // ====================================================================
        // PV Causality Assessment (2) — RUCAM hepatotoxicity, UCAS universal
        // ====================================================================
        "causality_rucam" => typed(params, tools::causality::causality_rucam),
        "causality_ucas" => typed(params, tools::causality::causality_ucas),

        // ====================================================================
        // PV Temporal Analysis (3) — TTO, challenge, plausibility
        // ====================================================================
        "temporal_tto" => typed(params, tools::temporal::temporal_tto),
        "temporal_challenge" => typed(params, tools::temporal::temporal_challenge),
        "temporal_plausibility" => typed(params, tools::temporal::temporal_plausibility_tool),

        // ====================================================================
        // Knowledge Engine Extended (3) — scoring, primitives, concepts
        // ====================================================================
        "knowledge_engine_score" => typed(params, tools::knowledge_engine::score_compendious),
        "knowledge_engine_extract_primitives" => {
            typed(params, tools::knowledge_engine::extract_primitives)
        }
        "knowledge_engine_extract_concepts" => {
            typed(params, tools::knowledge_engine::extract_concepts)
        }

        // ====================================================================
        // Stoichiometry Extended (3) — balance check, proof, isomer
        // ====================================================================
        "stoichiometry_is_balanced" => typed(params, tools::stoichiometry::is_balanced),
        "stoichiometry_prove" => typed(params, tools::stoichiometry::prove),
        "stoichiometry_is_isomer" => typed(params, tools::stoichiometry::is_isomer),

        // ====================================================================
        // NotebookLM Tools (16) — Library, Sessions, Auth, Query
        // T1 Grounding: μ (mapping) + π (persistence) + ∂ (boundary)
        // ====================================================================
        "nlm_add_notebook" => typed(params, tools::notebooklm::add_notebook),
        "nlm_list_notebooks" => typed(params, tools::notebooklm::list_notebooks),
        "nlm_get_notebook" => typed(params, tools::notebooklm::get_notebook),
        "nlm_select_notebook" => typed(params, tools::notebooklm::select_notebook),
        "nlm_update_notebook" => typed(params, tools::notebooklm::update_notebook),
        "nlm_remove_notebook" => typed(params, tools::notebooklm::remove_notebook),
        "nlm_search_notebooks" => typed(params, tools::notebooklm::search_notebooks),
        "nlm_get_library_stats" => typed(params, tools::notebooklm::get_library_stats),
        "nlm_list_sessions" => typed(params, tools::notebooklm::list_sessions),
        "nlm_close_session" => typed(params, tools::notebooklm::close_session),
        "nlm_reset_session" => typed(params, tools::notebooklm::reset_session),
        "nlm_get_health" => typed(params, tools::notebooklm::get_health),
        "nlm_setup_auth" => typed_async(params, tools::notebooklm::setup_auth).await,
        "nlm_re_auth" => typed_async(params, tools::notebooklm::re_auth).await,
        "nlm_ask_question" => typed_async(params, tools::notebooklm::ask_question).await,
        "nlm_cleanup_data" => typed_async(params, tools::notebooklm::cleanup_data).await,

        // ====================================================================
        // Cloud Intelligence (17) — 35-type taxonomy activation
        // T1 Grounding: μ (mapping) + → (causality) + ∂ (boundary) + N (quantity)
        // ====================================================================
        "cloud_primitive_composition" => typed(params, tools::cloud::primitive_composition),
        "cloud_transfer_confidence" => typed(params, tools::cloud::transfer_confidence),
        "cloud_tier_classify" => typed(params, tools::cloud::tier_classify),
        "cloud_compare_types" => typed(params, tools::cloud::compare_types),
        "cloud_reverse_synthesize" => typed(params, tools::cloud::reverse_synthesize),
        "cloud_list_types" => typed(params, tools::cloud::list_types),
        "cloud_molecular_weight" => typed(params, tools::cloud::molecular_weight),
        "cloud_dominant_shift" => typed(params, tools::cloud::dominant_shift),
        // Infrastructure Awareness (4)
        "cloud_infra_status" => typed_async(params, tools::cloud::infra_status).await,
        "cloud_infra_map" => typed_async(params, tools::cloud::infra_map).await,
        "cloud_capacity_project" => typed(params, tools::cloud::capacity_project),
        "cloud_supervisor_health" => typed(params, tools::cloud::supervisor_health),
        // Cross-Domain Reasoning (5)
        "cloud_reverse_transfer" => typed(params, tools::cloud::reverse_transfer),
        "cloud_transfer_chain" => typed(params, tools::cloud::transfer_chain),
        "cloud_architecture_advisor" => typed(params, tools::cloud::architecture_advisor),
        "cloud_anomaly_detect" => typed(params, tools::cloud::anomaly_detect),
        "cloud_transfer_matrix" => typed(params, tools::cloud::transfer_matrix),

        // Zeta — Riemann zeta function telescope pipeline (13)
        "zeta_compute" => typed(params, tools::zeta::zeta_compute),
        "zeta_find_zeros" => typed(params, tools::zeta::zeta_find_zeros),
        "zeta_verify_rh" => typed(params, tools::zeta::zeta_verify_rh),
        "zeta_embedded_zeros" => typed(params, tools::zeta::zeta_embedded_zeros),
        "zeta_lmfdb_parse" => typed(params, tools::zeta::zeta_lmfdb_parse),
        "zeta_telescope_run" => typed(params, tools::zeta::zeta_telescope_run),
        "zeta_batch_run" => typed(params, tools::zeta::zeta_batch_run),
        "zeta_scaling_fit" => typed(params, tools::zeta::zeta_scaling_fit),
        "zeta_scaling_predict" => typed(params, tools::zeta::zeta_scaling_predict),
        "zeta_cayley" => typed(params, tools::zeta::zeta_cayley),
        "zeta_operator_hunt" => typed(params, tools::zeta::zeta_operator_hunt),
        "zeta_operator_candidate" => typed(params, tools::zeta::zeta_operator_candidate),
        "zeta_gue_compare" => typed(params, tools::zeta::zeta_gue_compare),

        // ── Signal Detection Pipeline ──────────────────────────────────
        "pipeline_compute_all" => typed(params, tools::signal_pipeline::pipeline_compute_all),
        "pipeline_batch_compute" => typed(params, tools::signal_pipeline::pipeline_batch_compute),
        "pipeline_detect" => typed(params, tools::signal_pipeline::pipeline_detect),
        "pipeline_validate" => typed(params, tools::signal_pipeline::pipeline_validate),
        "pipeline_thresholds" => typed(params, tools::signal_pipeline::pipeline_thresholds),
        "pipeline_report" => typed(params, tools::signal_pipeline::pipeline_report),
        "pipeline_relay_chain" => typed(params, tools::signal_pipeline::pipeline_relay_chain),
        "pipeline_transfer" => typed(params, tools::signal_pipeline::pipeline_transfer),
        "pipeline_primitives" => typed(params, tools::signal_pipeline::pipeline_primitives),

        // ── Preemptive Pharmacovigilance ───────────────────────────────
        "preemptive_reactive" => typed(params, tools::preemptive_pv::preemptive_reactive),
        "preemptive_gibbs" => typed(params, tools::preemptive_pv::preemptive_gibbs),
        "preemptive_trajectory" => typed(params, tools::preemptive_pv::preemptive_trajectory),
        "preemptive_severity" => typed(params, tools::preemptive_pv::preemptive_severity),
        "preemptive_noise" => typed(params, tools::preemptive_pv::preemptive_noise),
        "preemptive_predictive" => typed(params, tools::preemptive_pv::preemptive_predictive),
        "preemptive_evaluate" => typed(params, tools::preemptive_pv::preemptive_evaluate),
        "preemptive_intervention" => typed(params, tools::preemptive_pv::preemptive_intervention),
        "preemptive_required_strength" => {
            typed(params, tools::preemptive_pv::preemptive_required_strength)
        }
        "preemptive_omega_table" => typed(params, tools::preemptive_pv::preemptive_omega_table),

        // ── OpenFDA ─────────────────────────────────────────────────────
        "openfda_drug_events" => typed_async(params, tools::openfda::openfda_drug_events).await,
        "openfda_drug_labels" => typed_async(params, tools::openfda::openfda_drug_labels).await,
        "openfda_drug_recalls" => typed_async(params, tools::openfda::openfda_drug_recalls).await,
        "openfda_drug_ndc" => typed_async(params, tools::openfda::openfda_drug_ndc).await,
        "openfda_drugs_at_fda" => typed_async(params, tools::openfda::openfda_drugs_at_fda).await,
        "openfda_device_events" => typed_async(params, tools::openfda::openfda_device_events).await,
        "openfda_device_recalls" => {
            typed_async(params, tools::openfda::openfda_device_recalls).await
        }
        "openfda_food_recalls" => typed_async(params, tools::openfda::openfda_food_recalls).await,
        "openfda_food_events" => typed_async(params, tools::openfda::openfda_food_events).await,
        "openfda_substances" => typed_async(params, tools::openfda::openfda_substances).await,
        "openfda_fan_out" => typed_async(params, tools::openfda::openfda_fan_out).await,

        // ── Compound Registry ───────────────────────────────────────────
        "compound_resolve" => typed(params, tools::compound_registry::compound_resolve),
        "compound_resolve_batch" => typed(params, tools::compound_registry::compound_resolve_batch),
        "compound_cache_search" => typed(params, tools::compound_registry::compound_cache_search),
        "compound_cache_get" => typed(params, tools::compound_registry::compound_cache_get),
        "compound_cache_count" => typed(params, tools::compound_registry::compound_cache_count),

        // ── FHIR R4 ─────────────────────────────────────────────────────
        "fhir_adverse_event_to_signal" => typed(params, tools::fhir::fhir_adverse_event_to_signal),
        "fhir_batch_to_signals" => typed(params, tools::fhir::fhir_batch_to_signals),
        "fhir_parse_bundle" => typed(params, tools::fhir::fhir_parse_bundle),
        "fhir_validate_resource" => typed(params, tools::fhir::fhir_validate_resource),

        // ── Retrocasting ─────────────────────────────────────────────────
        "retro_structural_similarity" => {
            typed(params, tools::retrocasting::retro_structural_similarity)
        }
        "retro_signal_significance" => {
            typed(params, tools::retrocasting::retro_signal_significance)
        }
        "retro_cluster_signals" => typed(params, tools::retrocasting::retro_cluster_signals),
        "retro_correlate_alerts" => typed(params, tools::retrocasting::retro_correlate_alerts),
        "retro_extract_features" => typed(params, tools::retrocasting::retro_extract_features),
        "retro_dataset_stats" => typed(params, tools::retrocasting::retro_dataset_stats),

        // ── Engram Knowledge Store ───────────────────────────────────────
        "engram_search" => typed(params, tools::engram::engram_search),
        "engram_search_decay" => typed(params, tools::engram::engram_search_decay),
        "engram_peek" => typed(params, tools::engram::engram_peek),
        "engram_stats" => typed(params, tools::engram::engram_stats),
        "engram_find_duplicates" => typed(params, tools::engram::engram_find_duplicates),
        "engram_decay_score" => typed(params, tools::engram::engram_decay_score),
        "engram_ingest" => typed(params, tools::engram::engram_ingest),
        "engram_by_source" => typed(params, tools::engram::engram_by_source),

        // ── Ghost Privacy ────────────────────────────────────────────────
        "ghost_boundary_check" => typed(params, tools::ghost::ghost_boundary_check),
        "ghost_mode_info" => typed(params, tools::ghost::ghost_mode_info),
        "ghost_category_policy" => typed(params, tools::ghost::ghost_category_policy),
        "ghost_scan_pii" => typed(params, tools::ghost::ghost_scan_pii),
        "ghost_scrub_fields" => typed(params, tools::ghost::ghost_scrub_fields),

        // ── Pharma R&D Taxonomy ──────────────────────────────────────────
        "pharma_taxonomy_summary" => typed(params, tools::pharma_rd::pharma_taxonomy_summary),
        "pharma_lookup_transfer" => typed(params, tools::pharma_rd::pharma_lookup_transfer),
        "pharma_transfer_matrix" => typed(params, tools::pharma_rd::pharma_transfer_matrix),
        "pharma_strongest_transfers" => typed(params, tools::pharma_rd::pharma_strongest_transfers),
        "pharma_weakest_transfers" => typed(params, tools::pharma_rd::pharma_weakest_transfers),
        "pharma_symbol_coverage" => typed(params, tools::pharma_rd::pharma_symbol_coverage),
        "pharma_pipeline_stage" => typed(params, tools::pharma_rd::pharma_pipeline_stage),
        "pharma_classify_generators" => typed(params, tools::pharma_rd::pharma_classify_generators),

        // ── Combinatorics ────────────────────────────────────────────────
        "comb_catalan" => typed(params, tools::combinatorics::comb_catalan),
        "comb_catalan_table" => typed(params, tools::combinatorics::comb_catalan_table),
        "comb_cycle_decomposition" => typed(params, tools::combinatorics::comb_cycle_decomposition),
        "comb_min_transpositions" => typed(params, tools::combinatorics::comb_min_transpositions),
        "comb_derangement" => typed(params, tools::combinatorics::comb_derangement),
        "comb_derangement_probability" => {
            typed(params, tools::combinatorics::comb_derangement_probability)
        }
        "comb_grid_paths" => typed(params, tools::combinatorics::comb_grid_paths),
        "comb_binomial" => typed(params, tools::combinatorics::comb_binomial),
        "comb_multinomial" => typed(params, tools::combinatorics::comb_multinomial),
        "comb_josephus" => typed(params, tools::combinatorics::comb_josephus),
        "comb_elimination_order" => typed(params, tools::combinatorics::comb_elimination_order),
        "comb_linear_extensions" => typed(params, tools::combinatorics::comb_linear_extensions),

        // ── Theory of Vigilance (Grounded) ───────────────────────────────
        "tov_grounded_signal_strength" => typed(params, tools::tov_grounded::tov_signal_strength),
        "tov_grounded_safety_margin" => typed(params, tools::tov_grounded::tov_safety_margin),
        "tov_grounded_stability_shell" => typed(params, tools::tov_grounded::tov_stability_shell),
        "tov_grounded_harm_type" => typed(params, tools::tov_grounded::tov_harm_type),
        "tov_grounded_meta_vigilance" => typed(params, tools::tov_grounded::tov_meta_vigilance),
        "tov_grounded_eka_intelligence" => typed(params, tools::tov_grounded::tov_eka_intelligence),
        "tov_grounded_magic_numbers" => typed(params, tools::tov_grounded::tov_magic_numbers),

        // ====================================================================
        // Unknown command
        // ── Statemind (DNA Pipeline) ──────────────────────────────────────
        "statemind_analyze_word" => typed(params, tools::statemind::statemind_analyze_word),
        "statemind_constellation" => typed(params, tools::statemind::statemind_constellation),

        // ── Reason (Causal Inference) ────────────────────────────────────
        "reason_infer" => typed(params, tools::reason::reason_infer),
        "reason_counterfactual" => typed(params, tools::reason::reason_counterfactual),

        // ── Word (Binary Algebra) ─────────────────────────────────────────
        "word_analyze" => typed(params, tools::word::word_analyze),
        "word_popcount" => typed(params, tools::word::word_popcount),
        "word_hamming_distance" => typed(params, tools::word::word_hamming_distance),
        "word_parity" => typed(params, tools::word::word_parity),
        "word_rotate" => typed(params, tools::word::word_rotate),
        "word_log2" => typed(params, tools::word::word_log2),
        "word_isqrt" => typed(params, tools::word::word_isqrt),
        "word_binary_gcd" => typed(params, tools::word::word_binary_gcd),
        "word_bit_test" => typed(params, tools::word::word_bit_test),
        "word_align_up" => typed(params, tools::word::word_align_up),

        // ── Harm Taxonomy (ToV §9) ───────────────────────────────────────
        "harm_classify" => typed(params, tools::harm_taxonomy::harm_classify),
        "harm_definition" => typed(params, tools::harm_taxonomy::harm_definition),
        "harm_catalog" => typed(params, tools::harm_taxonomy::harm_catalog),
        "harm_exhaustiveness" => typed(params, tools::harm_taxonomy::harm_exhaustiveness),
        "harm_axiom_connection" => typed(params, tools::harm_taxonomy::harm_axiom_connection),
        "harm_axiom_catalog" => typed(params, tools::harm_taxonomy::harm_axiom_catalog),
        "harm_combinations" => typed(params, tools::harm_taxonomy::harm_combinations),
        "harm_manifestation_derive" => {
            typed(params, tools::harm_taxonomy::harm_manifestation_derive)
        }

        // ── Antibodies (Adaptive Immune) ──────────────────────────────────
        "antibody_compute_affinity" => typed(params, tools::antibodies::antibody_compute_affinity),
        "antibody_classify_response" => {
            typed(params, tools::antibodies::antibody_classify_response)
        }
        "antibody_ig_info" => typed(params, tools::antibodies::antibody_ig_info),
        "antibody_ig_catalog" => typed(params, tools::antibodies::antibody_ig_catalog),

        // ── Jeopardy (Game Theory Strategy) ────────────────────────────────
        "jeopardy_clue_values" => typed(params, tools::jeopardy::jeopardy_clue_values),
        "jeopardy_categories" => typed(params, tools::jeopardy::jeopardy_categories),
        "jeopardy_score_board" => typed(params, tools::jeopardy::jeopardy_score_board),
        "jeopardy_should_buzz" => typed(params, tools::jeopardy::jeopardy_should_buzz),
        "jeopardy_optimal_dd_wager" => typed(params, tools::jeopardy::jeopardy_optimal_dd_wager),
        "jeopardy_optimal_final_wager" => {
            typed(params, tools::jeopardy::jeopardy_optimal_final_wager)
        }
        "jeopardy_board_control_value" => {
            typed(params, tools::jeopardy::jeopardy_board_control_value)
        }
        "jeopardy_compound_velocity" => typed(params, tools::jeopardy::jeopardy_compound_velocity),

        // ── Audio (Sample Conversion & Spec) ───────────────────────────────
        "audio_spec_compute" => typed(params, tools::audio::audio_spec_compute),
        "audio_spec_presets" => typed(params, tools::audio::audio_spec_presets),
        "audio_format_info" => typed(params, tools::audio::audio_format_info),
        "audio_rate_info" => typed(params, tools::audio::audio_rate_info),
        "audio_convert_sample" => typed(params, tools::audio::audio_convert_sample),
        "audio_resample" => typed(params, tools::audio::audio_resample),
        "audio_codec_catalog" => typed(params, tools::audio::audio_codec_catalog),
        "audio_device_capabilities" => typed(params, tools::audio::audio_device_capabilities),
        "audio_mixer_pan" => typed(params, tools::audio::audio_mixer_pan),
        "audio_stream_transitions" => typed(params, tools::audio::audio_stream_transitions),

        // ── Compilation Space (7D Transform Algebra) ───────────────────────
        "compilation_point_compare" => {
            typed(params, tools::compilation_space::compilation_point_compare)
        }
        "compilation_point_summary" => {
            typed(params, tools::compilation_space::compilation_point_summary)
        }
        "compilation_point_presets" => {
            typed(params, tools::compilation_space::compilation_point_presets)
        }
        "compilation_catalog_lookup" => {
            typed(params, tools::compilation_space::compilation_catalog_lookup)
        }
        "compilation_chain_validate" => {
            typed(params, tools::compilation_space::compilation_chain_validate)
        }
        "compilation_chain_presets" => {
            typed(params, tools::compilation_space::compilation_chain_presets)
        }
        "compilation_axes_catalog" => {
            typed(params, tools::compilation_space::compilation_axes_catalog)
        }
        "compilation_abstraction_levels" => typed(
            params,
            tools::compilation_space::compilation_abstraction_levels,
        ),
        "compilation_distance" => typed(params, tools::compilation_space::compilation_distance),

        // ── Pharmacovigilance Taxonomy (WHO-grounded PV concept encoder) ────
        "pv_taxonomy_summary" => typed(params, tools::pharmacovigilance::pv_taxonomy_summary),
        "pv_taxonomy_primitive" => typed(params, tools::pharmacovigilance::pv_taxonomy_primitive),
        "pv_taxonomy_composite" => typed(params, tools::pharmacovigilance::pv_taxonomy_composite),
        "pv_taxonomy_concept" => typed(params, tools::pharmacovigilance::pv_taxonomy_concept),
        "pv_taxonomy_chomsky" => typed(params, tools::pharmacovigilance::pv_taxonomy_chomsky),
        "pv_taxonomy_who_pillars" => {
            typed(params, tools::pharmacovigilance::pv_taxonomy_who_pillars)
        }
        "pv_taxonomy_transfer" => typed(params, tools::pharmacovigilance::pv_taxonomy_transfer),
        "pv_taxonomy_transfer_matrix" => typed(
            params,
            tools::pharmacovigilance::pv_taxonomy_transfer_matrix,
        ),
        "pv_taxonomy_lex_symbols" => {
            typed(params, tools::pharmacovigilance::pv_taxonomy_lex_symbols)
        }

        // ── Vault (AES-256-GCM + PBKDF2 crypto primitives) ────────────────
        "vault_derive_key" => typed(params, tools::vault::vault_derive_key),
        "vault_encrypt" => typed(params, tools::vault::vault_encrypt),
        "vault_decrypt" => typed(params, tools::vault::vault_decrypt),
        "vault_generate_salt" => typed(params, tools::vault::vault_generate_salt),
        "vault_config_sample" => typed(params, tools::vault::vault_config_sample),

        // ── Build Orchestrator (CI/CD pipeline management) ────────────────
        "build_orchestrator_dry_run" => typed(
            params,
            tools::build_orchestrator::build_orchestrator_dry_run,
        ),
        "build_orchestrator_stages" => {
            typed(params, tools::build_orchestrator::build_orchestrator_stages)
        }
        "build_orchestrator_workspace" => typed(
            params,
            tools::build_orchestrator::build_orchestrator_workspace,
        ),
        "build_orchestrator_history" => typed(
            params,
            tools::build_orchestrator::build_orchestrator_history,
        ),
        "build_orchestrator_metrics" => typed(
            params,
            tools::build_orchestrator::build_orchestrator_metrics,
        ),

        // ── Skills Engine (advanced skill analysis) ───────────────────────
        "skill_quality_index" => typed(params, tools::skills_engine::skill_quality_index),
        "skill_maturity" => typed(params, tools::skills_engine::skill_maturity),
        "skill_ksb_verify" => typed(params, tools::skills_engine::skill_ksb_verify),
        "skill_ecosystem_score" => typed(params, tools::skills_engine::skill_ecosystem_score),
        "skill_dependency_graph" => typed(params, tools::skills_engine::skill_dependency_graph),
        "skill_gap_analysis" => typed(params, tools::skills_engine::skill_gap_analysis),
        "skill_evolution_track" => typed(params, tools::skills_engine::skill_evolution_track),

        // ── NCBI (National Center for Biotechnology Information) ───────────
        "ncbi_esearch" => typed_async(params, tools::ncbi::esearch).await,
        "ncbi_esummary" => typed_async(params, tools::ncbi::esummary).await,
        "ncbi_efetch" => typed_async(params, tools::ncbi::efetch).await,
        "ncbi_elink" => typed_async(params, tools::ncbi::elink).await,
        "ncbi_search_and_fetch" => typed_async(params, tools::ncbi::search_and_fetch).await,
        "ncbi_search_and_summarize" => typed_async(params, tools::ncbi::search_and_summarize).await,

        // ====================================================================
        _ => Err(McpError::invalid_params(
            format!("Unknown command: {command}. Use command='help' for catalog."),
            None,
        )),
    }
}

// ============================================================================
// Dispatch Helpers
// ============================================================================

/// Deserialize `Value` → `T`, then call sync handler `f(T)`.
fn typed<T, F>(params: Value, f: F) -> Result<CallToolResult, McpError>
where
    T: serde::de::DeserializeOwned,
    F: FnOnce(T) -> Result<CallToolResult, McpError>,
{
    let p: T = deser(params)?;
    f(p)
}

/// Deserialize `Value` → `T`, then call async handler `f(T)`.
async fn typed_async<T, F, Fut>(params: Value, f: F) -> Result<CallToolResult, McpError>
where
    T: serde::de::DeserializeOwned,
    F: FnOnce(T) -> Fut,
    Fut: std::future::Future<Output = Result<CallToolResult, McpError>>,
{
    let p: T = deser(params)?;
    f(p).await
}

/// Deserialize `Value` → `T` with standard error.
/// Converts `null` to `{}` so tools with all-optional params work without explicit `{}`.
fn deser<T: serde::de::DeserializeOwned>(params: Value) -> Result<T, McpError> {
    let params = if params.is_null() {
        Value::Object(serde_json::Map::new())
    } else {
        params
    };
    serde_json::from_value(params)
        .map_err(|e| McpError::invalid_params(format!("Bad params: {e}"), None))
}

/// Wrap a watchtower `serde_json::Value` result into `CallToolResult`.
fn watchtower_wrap(result: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ============================================================================
// Help Catalog
// ============================================================================

/// Return JSON catalog of all commands grouped by category.
/// Toolbox: keyword search across the tool catalog.
///
/// Returns matching categories and their tools for discovery.
fn toolbox_search(params: params::system::ToolboxParams) -> Result<CallToolResult, McpError> {
    // Static catalog: category -> tool names
    let catalog = toolbox_catalog();

    let mut matches: Vec<serde_json::Value> = Vec::new();

    if let Some(ref cat) = params.category {
        // Direct category lookup
        let key = cat.to_lowercase();
        for (category, tools) in &catalog {
            if category.to_lowercase() == key {
                matches.push(serde_json::json!({
                    "category": category,
                    "tools": tools,
                    "count": tools.len()
                }));
            }
        }
    } else if let Some(ref query) = params.query {
        let q = query.to_lowercase();
        let terms: Vec<&str> = q.split_whitespace().collect();

        for (category, tools) in &catalog {
            let cat_lower = category.to_lowercase();
            let cat_matches = terms.iter().all(|t| cat_lower.contains(t));

            if cat_matches {
                // Whole category matches
                matches.push(serde_json::json!({
                    "category": category,
                    "tools": tools,
                    "count": tools.len()
                }));
            } else {
                // Check individual tool names
                let tool_hits: Vec<&str> = tools
                    .iter()
                    .filter(|t| {
                        let tl = t.to_lowercase();
                        terms.iter().any(|term| tl.contains(term))
                    })
                    .map(|t| t.as_str())
                    .collect();

                if !tool_hits.is_empty() {
                    matches.push(serde_json::json!({
                        "category": category,
                        "tools": tool_hits,
                        "count": tool_hits.len()
                    }));
                }
            }
        }
    } else {
        // No query — list all categories with counts
        let summary: Vec<serde_json::Value> = catalog
            .iter()
            .map(|(cat, tools)| {
                serde_json::json!({
                    "category": cat,
                    "count": tools.len()
                })
            })
            .collect();

        let result = serde_json::json!({
            "total_categories": summary.len(),
            "categories": summary,
            "hint": "Use query or category param to search"
        });

        return Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        )]));
    }

    let total_tools: usize = matches
        .iter()
        .map(|m| m.get("count").and_then(|c| c.as_u64()).unwrap_or(0) as usize)
        .sum();

    let result = serde_json::json!({
        "query": params.query,
        "category": params.category,
        "total_tools": total_tools,
        "matches": matches
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Static tool catalog for toolbox search.
fn toolbox_catalog() -> Vec<(&'static str, Vec<String>)> {
    // Reuse the same data as help_catalog but structured for search
    let catalog_json = help_catalog_json();
    let categories = catalog_json
        .get("categories")
        .and_then(|c| c.as_object())
        .cloned()
        .unwrap_or_default();

    categories
        .into_iter()
        .map(|(cat, tools)| {
            let tool_names: Vec<String> = tools
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            (Box::leak(cat.into_boxed_str()) as &'static str, tool_names)
        })
        .collect()
}

/// Shared catalog data as JSON Value.
fn help_catalog_json() -> serde_json::Value {
    serde_json::json!({
        "categories": {
            "system": ["nexcore_health", "config_validate", "mcp_servers_list", "mcp_server_get"],
            "foundation": ["foundation_levenshtein", "foundation_levenshtein_bounded", "foundation_fuzzy_search", "foundation_sha256", "foundation_yaml_parse", "foundation_graph_topsort", "foundation_graph_levels", "foundation_fsrs_review", "foundation_concept_grep", "foundation_domain_distance", "foundation_flywheel_velocity", "foundation_token_ratio", "foundation_spectral_overlap"],
            "topology": ["topo_vietoris_rips", "topo_persistence", "topo_betti", "graph_centrality", "graph_components", "graph_shortest_path"],
            "pv": ["pv_signal_complete", "pv_signal_prr", "pv_signal_ror", "pv_signal_ic", "pv_signal_ebgm", "pv_chi_square", "pv_signal_cooperative", "pv_naranjo_quick", "pv_who_umc_quick", "pv_signal_strength"],
            "benefit_risk": ["pv_qbri_compute", "pv_qbri_derive", "pv_qbri_equation", "qbr_compute", "qbr_simple", "qbr_therapeutic_window"],
            "signal": ["signal_detect", "signal_batch", "signal_thresholds"],
            "vigilance": ["vigilance_safety_margin", "vigilance_risk_score", "vigilance_harm_types", "vigilance_map_to_tov", "pv_signal_chart"],
            "guardian": ["guardian_homeostasis_tick", "guardian_evaluate_pv", "guardian_status", "guardian_reset", "guardian_inject_signal", "guardian_sensors_list", "guardian_actuators_list", "guardian_history", "guardian_originator_classify", "guardian_ceiling_for_originator", "guardian_space3d_compute"],
            "vigil": ["vigil_status", "vigil_health", "vigil_emit_event", "vigil_memory_search", "vigil_memory_stats", "vigil_llm_stats", "vigil_source_control", "vigil_executor_control", "vigil_authority_config", "vigil_context_assemble", "vigil_authority_verify", "vigil_webhook_test", "vigil_source_config"],
            "skills": ["skill_scan", "skill_list", "skill_get", "skill_validate", "skill_search_by_tag", "skill_list_nested", "skill_taxonomy_query", "skill_taxonomy_list", "skill_categories_compute_intensive", "skill_orchestration_analyze", "skill_execute", "skill_schema", "skill_compile", "skill_compile_check", "vocab_skill_lookup", "primitive_skill_lookup", "skill_chain_lookup", "skill_route", "vocab_list", "nexcore_assist"],
            "guidelines": ["guidelines_search", "guidelines_get", "guidelines_categories", "guidelines_pv_all", "guidelines_url", "ich_lookup", "ich_search", "ich_guideline", "ich_stats"],
            "fda_guidance": ["fda_guidance_search", "fda_guidance_get", "fda_guidance_categories", "fda_guidance_url", "fda_guidance_status"],
            "faers": ["faers_search", "faers_drug_events", "faers_signal_check", "faers_disproportionality", "faers_compare_drugs"],
            "faers_analytics": ["faers_outcome_conditioned", "faers_signal_velocity", "faers_seriousness_cascade", "faers_polypharmacy", "faers_reporter_weighted", "faers_geographic_divergence"],
            "faers_etl": ["faers_etl_run", "faers_etl_signals", "faers_etl_known_pairs", "faers_etl_status"],
            "gcloud": ["gcloud_auth_list", "gcloud_config_list", "gcloud_config_get", "gcloud_config_set", "gcloud_projects_list", "gcloud_projects_describe", "gcloud_projects_get_iam_policy", "gcloud_secrets_list", "gcloud_secrets_versions_access", "gcloud_storage_buckets_list", "gcloud_storage_ls", "gcloud_storage_cp", "gcloud_compute_instances_list", "gcloud_run_services_list", "gcloud_run_services_describe", "gcloud_functions_list", "gcloud_iam_service_accounts_list", "gcloud_logging_read", "gcloud_run_command"],
            "wolfram": ["wolfram_query", "wolfram_short_answer", "wolfram_spoken_answer", "wolfram_calculate", "wolfram_step_by_step", "wolfram_plot", "wolfram_convert", "wolfram_chemistry", "wolfram_physics", "wolfram_astronomy", "wolfram_statistics", "wolfram_data_lookup", "wolfram_datetime", "wolfram_nutrition", "wolfram_finance", "wolfram_linguistics"],
            "principles": ["principles_list", "principles_get", "principles_search"],
            "brain": ["brain_session_create", "brain_session_load", "brain_sessions_list", "brain_artifact_save", "brain_artifact_resolve", "brain_artifact_get", "brain_artifact_diff", "code_tracker_track", "code_tracker_changed", "code_tracker_original", "implicit_get", "implicit_set", "implicit_stats", "implicit_find_corrections", "implicit_patterns_by_grounding", "implicit_patterns_by_relevance", "brain_recovery_check", "brain_recovery_repair", "brain_recovery_rebuild_index", "brain_recovery_auto", "brain_coordination_acquire", "brain_coordination_release", "brain_coordination_status"],
            "regulatory": ["regulatory_primitives_extract", "regulatory_primitives_audit", "regulatory_primitives_compare"],
            "chemistry": ["chemistry_threshold_rate", "chemistry_decay_remaining", "chemistry_saturation_rate", "chemistry_feasibility", "chemistry_dependency_rate", "chemistry_buffer_capacity", "chemistry_signal_absorbance", "chemistry_equilibrium", "chemistry_pv_mappings", "chemistry_threshold_exceeded", "chemistry_hill_response", "chemistry_nernst_potential", "chemistry_inhibition_rate", "chemistry_eyring_rate", "chemistry_langmuir_coverage", "chemistry_gaussian_overlap"],
            "stem": ["stem_version", "stem_taxonomy", "stem_confidence_combine", "stem_tier_info", "stem_chem_balance", "stem_chem_fraction", "stem_chem_ratio", "stem_chem_rate", "stem_chem_affinity", "stem_phys_fma", "stem_phys_conservation", "stem_phys_period", "stem_phys_amplitude", "stem_phys_scale", "stem_phys_inertia", "stem_math_bounds_check", "stem_math_relation_invert", "stem_math_proof", "stem_math_identity", "stem_spatial_distance", "stem_spatial_triangle", "stem_spatial_neighborhood", "stem_spatial_dimension", "stem_spatial_orientation"],
            "algovigilance": ["algovigil_dedup_pair", "algovigil_dedup_batch", "algovigil_triage_decay", "algovigil_triage_reinforce", "algovigil_triage_queue", "algovigil_status"],
            "edit_distance": ["edit_distance_compute", "edit_distance_similarity", "edit_distance_traceback", "edit_distance_transfer", "edit_distance_batch"],
            "cargo": ["cargo_check", "cargo_build", "cargo_test", "cargo_clippy", "cargo_fmt", "cargo_tree"],
            "rust_dev": ["rust_dev_error_type", "rust_dev_derive_advisor", "rust_dev_match_generate", "rust_dev_borrow_explain", "rust_dev_clippy_explain", "rust_dev_rustc_explain", "rust_dev_unsafe_audit", "rust_dev_cargo_expand", "rust_dev_cargo_bloat", "rust_dev_cargo_miri", "rust_dev_edition_migrate", "rust_dev_invocations"],
            "validation": ["validation_run", "validation_check", "validation_domains", "validation_classify_tests"],
            "compliance": ["compliance_check_exclusion", "compliance_assess", "compliance_catalog_ich", "compliance_sec_filings", "compliance_sec_pharma"],
            "watchtower": ["watchtower_sessions_list", "watchtower_active_sessions", "watchtower_analyze", "watchtower_telemetry_stats", "watchtower_recent", "watchtower_symbol_audit", "watchtower_gemini_stats", "watchtower_gemini_recent", "watchtower_unified"],
            "telemetry": ["telemetry_sources_list", "telemetry_source_analyze", "telemetry_governance_crossref", "telemetry_snapshot_evolution", "telemetry_intel_report", "telemetry_recent"],
            "immunity": ["immunity_scan", "immunity_scan_errors", "immunity_list", "immunity_get", "immunity_propose", "immunity_status"],
            "hud": ["sba_allocate_agent", "sba_chain_next", "ssa_persist_state", "ssa_verify_integrity", "fed_budget_report", "fed_recommend_model", "sec_audit_market", "comm_recommend_protocol", "comm_route_message", "explore_launch_mission", "explore_record_discovery", "explore_get_frontier", "health_validate_signal", "health_measure_impact", "treasury_convert_asymmetry", "treasury_audit", "dot_dispatch_manifest", "dot_verify_highway", "dhs_verify_boundary", "edu_train_agent", "edu_evaluate", "nsf_fund_research", "gsa_procure", "gsa_audit_value"],
            "primitive_validation": ["primitive_validate", "primitive_cite", "primitive_validate_batch", "primitive_validation_tiers"],
            "brand_semantics": ["brand_decomposition_nexvigilant", "brand_decomposition_get", "brand_primitive_test", "brand_semantic_tiers"],
            "mesh": ["mesh_lookup", "mesh_search", "mesh_tree", "mesh_crossref", "mesh_enrich_pubmed", "mesh_consistency"],
            "hormone": ["hormone_status", "hormone_get", "hormone_stimulus", "hormone_modifiers"],
            "cytokine": ["cytokine_emit", "cytokine_status", "cytokine_families", "cytokine_recent", "chemotaxis_gradient", "endocytosis_internalize"],
            "synapse": ["synapse_get_or_create", "synapse_get", "synapse_observe", "synapse_list", "synapse_stats", "synapse_prune"],
            "energy": ["energy_charge", "energy_decide"],
            "transcriptase": ["transcriptase_process", "transcriptase_infer", "transcriptase_violations", "transcriptase_generate"],
            "ribosome": ["ribosome_store", "ribosome_list", "ribosome_validate", "ribosome_generate", "ribosome_drift"],
            "phenotype": ["phenotype_mutate", "phenotype_verify"],
            "pvdsl": ["pvdsl_compile", "pvdsl_execute", "pvdsl_eval", "pvdsl_functions"],
            "dtree": ["dtree_train", "dtree_predict", "dtree_importance", "dtree_prune", "dtree_export", "dtree_info"],
            "game_theory": ["game_theory_nash_2x2", "forge_payoff_matrix", "forge_nash_solve", "forge_quality_score", "forge_code_generate"],
            "epidemiology": ["epidemiology_relative_risk", "epidemiology_odds_ratio", "epidemiology_attributable_risk", "epidemiology_nnt_nnh", "epidemiology_attributable_fraction", "epidemiology_population_attributable_fraction", "epidemiology_incidence_rate", "epidemiology_prevalence", "epidemiology_kaplan_meier", "epidemiology_smr", "epidemiology_mappings"],
            "chemivigilance": ["chem_parse_smiles", "chem_descriptors", "chem_fingerprint", "chem_similarity", "chem_structural_alerts", "chem_predict_toxicity", "chem_predict_metabolites", "chem_predict_degradants", "chem_safety_brief", "chem_substructure", "chem_watchlist", "chem_alert_library", "chem_ring_scan", "chem_aromaticity", "chem_molecular_formula", "chem_predict_mutagenicity", "chem_predict_hepatotoxicity", "chem_predict_cardiotoxicity", "chem_assess_applicability_domain"],
            "pk": ["pk_auc", "pk_clearance", "pk_half_life", "pk_steady_state", "pk_ionization", "pk_michaelis_menten"],
            "causality": ["causality_rucam", "causality_ucas"],
            "temporal": ["temporal_tto", "temporal_challenge", "temporal_plausibility"],
            "knowledge_engine": ["knowledge_engine_compress", "knowledge_engine_compile", "knowledge_engine_query", "knowledge_engine_score", "knowledge_engine_extract_primitives", "knowledge_engine_extract_concepts"],
            "notebooklm": ["nlm_add_notebook", "nlm_list_notebooks", "nlm_get_notebook", "nlm_select_notebook", "nlm_update_notebook", "nlm_remove_notebook", "nlm_search_notebooks", "nlm_get_library_stats", "nlm_list_sessions", "nlm_close_session", "nlm_reset_session", "nlm_get_health", "nlm_setup_auth", "nlm_re_auth", "nlm_ask_question", "nlm_cleanup_data"],
            "cloud": ["cloud_primitive_composition", "cloud_transfer_confidence", "cloud_tier_classify", "cloud_compare_types", "cloud_reverse_synthesize", "cloud_list_types", "cloud_molecular_weight", "cloud_dominant_shift", "cloud_infra_status", "cloud_infra_map", "cloud_capacity_project", "cloud_supervisor_health", "cloud_reverse_transfer", "cloud_transfer_chain", "cloud_architecture_advisor", "cloud_anomaly_detect", "cloud_transfer_matrix"],
            "kellnr_pk": ["kellnr_compute_pk_auc", "kellnr_compute_pk_steady_state", "kellnr_compute_pk_ionization", "kellnr_compute_pk_clearance", "kellnr_compute_pk_volume_distribution", "kellnr_compute_pk_michaelis_menten"],
            "kellnr_thermo": ["kellnr_compute_thermo_gibbs", "kellnr_compute_thermo_kd", "kellnr_compute_thermo_binding_affinity", "kellnr_compute_thermo_arrhenius"],
            "kellnr_stats": ["kellnr_compute_stats_welch_ttest", "kellnr_compute_stats_ols_regression", "kellnr_compute_stats_poisson_ci", "kellnr_compute_stats_bayesian_posterior", "kellnr_compute_stats_entropy"],
            "kellnr_graph": ["kellnr_compute_graph_betweenness", "kellnr_compute_graph_mutual_info", "kellnr_compute_graph_tarjan_scc", "kellnr_compute_graph_topsort"],
            "kellnr_dtree": ["kellnr_compute_dtree_feature_importance", "kellnr_compute_dtree_prune", "kellnr_compute_dtree_to_rules"],
            "kellnr_signal": ["kellnr_compute_signal_sprt", "kellnr_compute_signal_cusum", "kellnr_compute_signal_weibull_tto"],
            "kellnr_registry": ["kellnr_search_crates", "kellnr_get_crate_metadata", "kellnr_list_crate_versions", "kellnr_get_version_details", "kellnr_list_owners", "kellnr_add_owner", "kellnr_remove_owner", "kellnr_yank_version", "kellnr_unyank_version", "kellnr_list_all_crates", "kellnr_get_dependencies", "kellnr_get_dependents", "kellnr_health_check", "kellnr_download_crate", "kellnr_registry_stats"],
            "graph_layout": ["graph_layout_converge"],
            "career": ["career_transitions"],
            "learning_dag": ["learning_dag_resolve"],
            "stoichiometry": ["stoichiometry_encode", "stoichiometry_decode", "stoichiometry_sisters", "stoichiometry_mass_state", "stoichiometry_dictionary", "stoichiometry_is_balanced", "stoichiometry_prove", "stoichiometry_is_isomer"],
            "drift": ["drift_ks_test", "drift_psi", "drift_jsd", "drift_detect"],
            "rate_limit": ["rate_limit_token_bucket", "rate_limit_sliding_window", "rate_limit_status"],
            "rank_fusion": ["rank_fusion_rrf", "rank_fusion_hybrid", "rank_fusion_borda"],
            "security_posture": ["security_posture_assess", "security_threat_readiness", "security_compliance_gap"],
            "observability": ["observability_record_latency", "observability_query", "observability_freshness"]
        }
    })
}

fn help_catalog() -> Result<CallToolResult, McpError> {
    let catalog = serde_json::json!({
        "total": 496,
        "usage": "nexcore(command=\"CMD\", params={...})",
        "categories": {
            "system": ["nexcore_health", "config_validate", "mcp_servers_list", "mcp_server_get"],
            "forge": ["forge_init", "forge_reference", "forge_mine", "forge_prompt", "forge_suggest", "forge_summary", "forge_system_prompt", "forge_tier"],
            "academy_forge": ["forge_extract", "forge_validate", "forge_scaffold", "forge_schema", "forge_compile", "forge_atomize", "forge_graph", "forge_shortest_path"],
            "foundation": ["foundation_levenshtein", "foundation_levenshtein_bounded", "foundation_fuzzy_search", "foundation_sha256", "foundation_yaml_parse", "foundation_graph_topsort", "foundation_graph_levels", "foundation_fsrs_review", "foundation_concept_grep", "foundation_domain_distance", "foundation_flywheel_velocity", "foundation_token_ratio", "foundation_spectral_overlap"],
            "topology": ["topo_vietoris_rips", "topo_persistence", "topo_betti", "graph_centrality", "graph_components", "graph_shortest_path"],
            "pv": ["pv_signal_complete", "pv_signal_prr", "pv_signal_ror", "pv_signal_ic", "pv_signal_ebgm", "pv_chi_square", "pv_signal_cooperative", "pv_naranjo_quick", "pv_who_umc_quick", "pv_signal_strength"],
            "benefit_risk": ["pv_qbri_compute", "pv_qbri_derive", "pv_qbri_equation", "qbr_compute", "qbr_simple", "qbr_therapeutic_window"],
            "signal": ["signal_detect", "signal_batch", "signal_thresholds"],
            "pvdsl": ["pvdsl_compile", "pvdsl_execute", "pvdsl_eval", "pvdsl_functions"],
            "mcp_lock": ["mcp_lock", "mcp_unlock", "mcp_lock_status"],
            "vigilance": ["vigilance_safety_margin", "vigilance_risk_score", "vigilance_harm_types", "vigilance_map_to_tov", "pv_signal_chart"],
            "compliance": ["compliance_check_exclusion", "compliance_assess", "compliance_catalog_ich", "compliance_sec_filings", "compliance_sec_pharma"],
            "hormone": ["hormone_status", "hormone_get", "hormone_stimulus", "hormone_modifiers"],
            "guardian": ["guardian_homeostasis_tick", "guardian_evaluate_pv", "guardian_status", "guardian_reset", "guardian_inject_signal", "guardian_sensors_list", "guardian_actuators_list", "guardian_history", "guardian_subscribe", "guardian_originator_classify", "guardian_ceiling_for_originator", "guardian_space3d_compute", "guardian_adversarial_input", "adversarial_decision_probe", "pv_control_loop_tick", "fda_bridge_evaluate", "fda_bridge_batch"],
            "hud": ["sba_allocate_agent", "sba_chain_next", "ssa_persist_state", "ssa_verify_integrity", "fed_budget_report", "fed_recommend_model", "sec_audit_market", "comm_recommend_protocol", "comm_route_message", "explore_launch_mission", "explore_record_discovery", "explore_get_frontier", "health_validate_signal", "health_measure_impact", "treasury_convert_asymmetry", "treasury_audit", "dot_dispatch_manifest", "dot_verify_highway", "dhs_verify_boundary", "edu_train_agent", "edu_evaluate", "nsf_fund_research", "gsa_procure", "gsa_audit_value"],
            "commandments": ["commandment_verify", "commandment_info", "commandment_list", "commandment_audit"],
            "docs": ["docs_generate_claude_md"],
            "vigil": ["vigil_status", "vigil_health", "vigil_emit_event", "vigil_memory_search", "vigil_memory_stats", "vigil_llm_stats", "vigil_source_control", "vigil_executor_control", "vigil_authority_config", "vigil_context_assemble", "vigil_authority_verify", "vigil_webhook_test", "vigil_source_config"],
            "pv_pipeline": ["pv_pipeline"],
            "pv_axioms": ["pv_axioms_ksb_lookup", "pv_axioms_regulation_search", "pv_axioms_traceability_chain", "pv_axioms_domain_dashboard", "pv_axioms_query"],
            "skills": ["skill_scan", "skill_list", "skill_get", "skill_validate", "skill_search_by_tag", "skill_list_nested", "skill_taxonomy_query", "skill_taxonomy_list", "skill_categories_compute_intensive", "skill_orchestration_analyze", "skill_execute", "skill_schema", "skill_compile", "skill_compile_check", "vocab_skill_lookup", "primitive_skill_lookup", "skill_chain_lookup", "skill_route", "vocab_list", "nexcore_assist"],
            "guidelines": ["guidelines_search", "guidelines_get", "guidelines_categories", "guidelines_pv_all", "guidelines_url", "ich_lookup", "ich_search", "ich_guideline", "ich_stats"],
            "fda_guidance": ["fda_guidance_search", "fda_guidance_get", "fda_guidance_categories", "fda_guidance_url", "fda_guidance_status"],
            "mesh": ["mesh_lookup", "mesh_search", "mesh_tree", "mesh_crossref", "mesh_enrich_pubmed", "mesh_consistency"],
            "faers": ["faers_search", "faers_drug_events", "faers_signal_check", "faers_disproportionality", "faers_compare_drugs"],
            // "ncbi": disabled — needs feature gate in nexcore-dna
            "faers_etl": ["faers_etl_run", "faers_etl_signals", "faers_etl_known_pairs", "faers_etl_status"],
            "pharos": ["pharos_run", "pharos_status", "pharos_report"],
            "faers_analytics": ["faers_outcome_conditioned", "faers_signal_velocity", "faers_seriousness_cascade", "faers_polypharmacy", "faers_reporter_weighted", "faers_geographic_divergence"],
            "lex_primitiva": ["lex_primitiva_list", "lex_primitiva_get", "lex_primitiva_tier", "lex_primitiva_composition", "lex_primitiva_reverse_compose", "lex_primitiva_reverse_lookup", "lex_primitiva_molecular_weight", "lex_primitiva_dominant_shift", "lex_primitiva_state_mode", "lex_primitiva_audit", "lex_primitiva_synth"],
            "laboratory": ["lab_experiment", "lab_compare", "lab_react", "lab_batch"],
            "skill_tokens": ["skill_token_analyze"],
            "cep": ["cep_execute_stage", "cep_pipeline_stages", "cep_validate_extraction", "cep_extract_primitives", "cep_domain_translate", "cep_classify_primitive"],
            "gcloud": ["gcloud_auth_list", "gcloud_config_list", "gcloud_config_get", "gcloud_config_set", "gcloud_projects_list", "gcloud_projects_describe", "gcloud_projects_get_iam_policy", "gcloud_secrets_list", "gcloud_secrets_versions_access", "gcloud_storage_buckets_list", "gcloud_storage_ls", "gcloud_storage_cp", "gcloud_compute_instances_list", "gcloud_run_services_list", "gcloud_run_services_describe", "gcloud_functions_list", "gcloud_iam_service_accounts_list", "gcloud_logging_read", "gcloud_run_command"],
            "wolfram": ["wolfram_query", "wolfram_short_answer", "wolfram_spoken_answer", "wolfram_calculate", "wolfram_step_by_step", "wolfram_plot", "wolfram_convert", "wolfram_chemistry", "wolfram_physics", "wolfram_astronomy", "wolfram_statistics", "wolfram_data_lookup", "wolfram_query_with_assumption", "wolfram_query_filtered", "wolfram_image_result", "wolfram_datetime", "wolfram_nutrition", "wolfram_finance", "wolfram_linguistics"],
            "perplexity": ["perplexity_search", "perplexity_research", "perplexity_competitive", "perplexity_regulatory"],
            "principles": ["principles_list", "principles_get", "principles_search"],
            "validation": ["validation_run", "validation_check", "validation_domains", "validation_classify_tests"],
            "brain": ["brain_session_create", "brain_session_load", "brain_sessions_list", "brain_artifact_save", "brain_artifact_resolve", "brain_artifact_get", "brain_artifact_diff", "code_tracker_track", "code_tracker_changed", "code_tracker_original", "implicit_get", "implicit_set", "implicit_stats", "implicit_find_corrections", "implicit_patterns_by_grounding", "implicit_patterns_by_relevance", "brain_recovery_check", "brain_recovery_repair", "brain_recovery_rebuild_index", "brain_recovery_auto", "brain_coordination_acquire", "brain_coordination_release", "brain_coordination_status", "brain_verify_engrams"],
            "brain_db": ["brain_db_summary", "brain_db_decisions_stats", "brain_db_tool_stats", "brain_db_antibodies", "brain_db_handoffs", "brain_db_tasks", "brain_db_efficiency", "brain_db_sync", "brain_db_query"],
            "anatomy_db": ["anatomy_query", "anatomy_status", "anatomy_record_cytokine", "anatomy_record_hormones", "anatomy_record_guardian_tick", "anatomy_record_immunity_event", "anatomy_record_synapse", "anatomy_record_energy", "anatomy_record_transcriptase", "anatomy_record_ribosome", "anatomy_record_phenotype", "anatomy_record_organ_signal"],
            "learning": ["learning_daemon_status", "learning_daemon_trends", "learning_daemon_beliefs", "learning_daemon_corrections", "learning_daemon_velocity", "learn_landscape", "learn_extract", "learn_assimilate", "learn_recall", "learn_normalize", "learn_pipeline"],
            "oracle": ["oracle_ingest", "oracle_predict", "oracle_observe", "oracle_report", "oracle_status", "oracle_reset", "oracle_top_predictions"],
            "synapse": ["synapse_get_or_create", "synapse_get", "synapse_observe", "synapse_list", "synapse_stats", "synapse_prune"],
            "immunity": ["immunity_scan", "immunity_scan_errors", "immunity_list", "immunity_get", "immunity_propose", "immunity_status"],
            "nmd": ["nmd_check", "nmd_upf_evaluate", "nmd_smg_process", "nmd_adaptive_stats", "nmd_thymic_status", "nmd_status"],
            "regulatory": ["regulatory_primitives_extract", "regulatory_primitives_audit", "regulatory_primitives_compare", "regulatory_effectiveness_assess"],
            "brand_semantics": ["brand_decomposition_nexvigilant", "brand_decomposition_get", "brand_primitive_test", "brand_semantic_tiers"],
            "primitive_validation": ["primitive_validate", "primitive_cite", "primitive_validate_batch", "primitive_validation_tiers"],
            "chemistry": ["chemistry_threshold_rate", "chemistry_decay_remaining", "chemistry_saturation_rate", "chemistry_feasibility", "chemistry_dependency_rate", "chemistry_buffer_capacity", "chemistry_signal_absorbance", "chemistry_equilibrium", "chemistry_pv_mappings", "chemistry_threshold_exceeded", "chemistry_hill_response", "chemistry_nernst_potential", "chemistry_inhibition_rate", "chemistry_eyring_rate", "chemistry_langmuir_coverage", "chemistry_first_law_closed", "chemistry_first_law_open"],
            "molecular": ["molecular_translate_codon", "molecular_translate_mrna", "molecular_central_dogma", "molecular_adme_phase"],
            "visual": ["visual_shape_classify", "visual_color_analyze", "visual_shape_list"],
            "cytokine": ["cytokine_emit", "cytokine_status", "cytokine_families", "cytokine_recent", "chemotaxis_gradient", "endocytosis_internalize"],
            "value_mining": ["value_signal_types", "value_signal_detect", "value_baseline_create", "value_pv_mapping"],
            "signal_theory": ["signal_theory_axioms", "signal_theory_theorems", "signal_theory_detect", "signal_theory_decision_matrix", "signal_theory_conservation_check", "signal_theory_pipeline", "signal_theory_cascade", "signal_theory_parallel"],
            "signal_fence": ["fence_status", "fence_scan", "fence_evaluate"],
            "game_theory": ["game_theory_nash_2x2", "forge_payoff_matrix", "forge_nash_solve", "forge_quality_score", "forge_code_generate"],
            "mesh_network": ["mesh_network_simulate", "mesh_network_route_quality", "mesh_network_grounding", "mesh_network_node_info"],
            "prima": ["prima_parse", "prima_eval", "prima_codegen", "prima_primitives", "prima_targets"],
            "aggregate": ["aggregate_fold", "aggregate_tree_fold", "aggregate_rank", "aggregate_percentile", "aggregate_outliers"],
            "compound": ["compound_growth", "compound_detect"],
            "ccp": ["ccp_episode_start", "ccp_dose_compute", "ccp_episode_advance", "ccp_interaction_check", "ccp_quality_score", "ccp_phase_transition"],
            "education": ["edu_subject_create", "edu_subject_list", "edu_lesson_create", "edu_lesson_add_step", "edu_learner_create", "edu_enroll", "edu_assess", "edu_mastery", "edu_phase_transition", "edu_phase_info", "edu_review_create", "edu_review_schedule", "edu_review_status", "edu_bayesian_update", "edu_primitive_map"],
            "antitransformer": ["antitransformer_analyze", "antitransformer_batch"],
            "energy": ["energy_charge", "energy_decide"],
            "transcriptase": ["transcriptase_process", "transcriptase_infer", "transcriptase_violations", "transcriptase_generate"],
            "ribosome": ["ribosome_store", "ribosome_list", "ribosome_validate", "ribosome_generate", "ribosome_drift"],
            "phenotype": ["phenotype_mutate", "phenotype_verify"],
            "domain_primitives": ["domain_primitives_list", "domain_primitives_transfer", "domain_primitives_decompose", "domain_primitives_bottlenecks", "domain_primitives_compare", "domain_primitives_topo_sort", "domain_primitives_critical_paths", "domain_primitives_registry", "domain_primitives_save", "domain_primitives_load", "domain_primitives_transfer_matrix"],
            "fda_credibility": ["fda_define_cou", "fda_assess_risk", "fda_create_plan", "fda_validate_evidence", "fda_decide_adequacy", "fda_calculate_score", "fda_metrics_summary", "fda_evidence_distribution", "fda_risk_distribution", "fda_drift_trend", "fda_rating_thresholds"],
            "stem": ["stem_version", "stem_taxonomy", "stem_confidence_combine", "stem_tier_info", "stem_chem_balance", "stem_chem_fraction", "stem_chem_ratio", "stem_chem_rate", "stem_chem_affinity", "stem_phys_fma", "stem_phys_conservation", "stem_phys_period", "stem_phys_amplitude", "stem_phys_scale", "stem_phys_inertia", "stem_math_bounds_check", "stem_math_relation_invert", "stem_math_proof", "stem_math_identity", "stem_spatial_distance", "stem_spatial_triangle", "stem_spatial_neighborhood", "stem_spatial_dimension", "stem_spatial_orientation"],
            "viz": ["viz_stem_taxonomy", "viz_type_composition", "viz_method_loop", "viz_confidence_chain", "viz_bounds", "viz_dag", "viz_molecular_info", "viz_surface_mesh", "viz_spectral_analysis", "viz_community_detect", "viz_centrality", "viz_vdag_overlay", "viz_antibody_structure", "viz_interaction_map", "viz_projection", "viz_protein_structure", "viz_topology_analysis", "viz_dynamics_step", "viz_force_field_energy", "viz_gpu_layout", "viz_hypergraph", "viz_lod_select", "viz_minimize_energy", "viz_particle_preset", "viz_ae_overlay", "viz_coord_gen", "viz_bipartite_layout", "viz_manifold_sample", "viz_string_modes", "viz_render_pipeline", "viz_orbital_density"],
            "watchtower": ["watchtower_sessions_list", "watchtower_active_sessions", "watchtower_analyze", "watchtower_telemetry_stats", "watchtower_recent", "watchtower_symbol_audit", "watchtower_gemini_stats", "watchtower_gemini_recent", "watchtower_unified"],
            "node_hunter": ["node_hunt_scan", "node_hunt_isolate"],
            "telemetry_intel": ["telemetry_sources_list", "telemetry_source_analyze", "telemetry_governance_crossref", "telemetry_snapshot_evolution", "telemetry_intel_report", "telemetry_recent"],
            "primitive_scanner": ["primitive_scan", "primitive_batch_test"],
            "algovigilance": ["algovigil_dedup_pair", "algovigil_dedup_batch", "algovigil_triage_decay", "algovigil_triage_reinforce", "algovigil_triage_queue", "algovigil_status"],
            "dtree": ["dtree_train", "dtree_predict", "dtree_importance", "dtree_prune", "dtree_export", "dtree_info"],
            "edit_distance": ["edit_distance_compute", "edit_distance_similarity", "edit_distance_traceback", "edit_distance_transfer", "edit_distance_batch"],
            "integrity": ["integrity_analyze", "integrity_assess_ksb", "integrity_calibration"],
            "organize": ["organize_analyze", "organize_config_default", "organize_report_markdown", "organize_report_json", "organize_observe", "organize_rank"],
            "sentinel": ["sentinel_status", "sentinel_check_ip", "sentinel_parse_line", "sentinel_config_defaults"],
            "measure": ["measure_crate", "measure_workspace", "measure_entropy", "measure_graph", "measure_drift", "measure_compare", "measure_stats", "quality_gradient"],
            "anatomy": ["anatomy_health", "anatomy_blast_radius", "anatomy_chomsky", "anatomy_violations"],
            "sqi": ["sqi_score", "sqi_ecosystem"],
            "sos": ["sos_create", "sos_transition", "sos_state", "sos_history", "sos_validate", "sos_list", "sos_cycles", "sos_audit", "sos_schedule", "sos_route"],
            "cortex": ["cortex_download_model", "cortex_list_models", "cortex_model_info", "cortex_generate", "cortex_embed", "cortex_fine_tune_status"],
            "mcp_telemetry": ["telemetry_summary", "telemetry_by_tool", "telemetry_slow_calls", "audit_trail"],
            "monitoring": ["monitoring_health_check", "monitoring_alerts", "monitoring_hook_health", "monitoring_signal_digest", "phase4_surveillance_tick"],
            "clearance": ["clearance_evaluate", "clearance_policy_for", "clearance_validate_change", "clearance_level_info", "clearance_config"],
            "secure_boot": ["secure_boot_status", "secure_boot_verify", "secure_boot_quote"],
            "user": ["user_create", "user_login", "user_logout", "user_list", "user_lock", "user_unlock", "user_status", "user_change_password"],
            "claude_fs": ["claude_fs_list", "claude_fs_read", "claude_fs_write", "claude_fs_delete", "claude_fs_search", "claude_fs_tail", "claude_fs_diff", "claude_fs_stat", "claude_fs_backup_now"],
            "compendious": ["compendious_score_text", "compendious_compress_text", "compendious_compare_texts", "compendious_analyze_patterns", "compendious_get_domain_target"],
            "docs_claude": ["docs_claude_list_pages", "docs_claude_get_page", "docs_claude_search_docs", "docs_claude_get_docs_index"],
            "gsheets": ["gsheets_list_sheets", "gsheets_read_range", "gsheets_batch_read", "gsheets_write_range", "gsheets_append_rows", "gsheets_metadata", "gsheets_search"],
            "reddit": ["reddit_status", "reddit_authenticate", "reddit_hot_posts", "reddit_new_posts", "reddit_subreddit_info", "reddit_detect_signals", "reddit_search_entity"],
            "trust": ["trust_score", "trust_record", "trust_snapshot", "trust_decide", "trust_harm_weight", "trust_velocity", "trust_multi_score", "trust_network_chain"],
            "molecular_weight": ["mw_compute", "mw_periodic_table", "mw_compare", "mw_predict_transfer"],
            "primitive_trace": ["primitive_trace"],
            "transform": ["transform_list_profiles", "transform_get_profile", "transform_segment", "transform_compile_plan", "transform_score_fidelity"],
            "insight": ["insight_ingest", "insight_status", "insight_config", "insight_connect", "insight_compress", "insight_compress_auto", "insight_patterns", "insight_reset", "insight_system_status", "insight_system_ingest", "insight_system_register", "insight_system_reset", "insight_query", "insight_novelties"],
            "counter_awareness": ["ca_detect", "ca_fusion", "ca_optimize", "ca_matrix", "ca_catalog"],
            "caesura": ["caesura_scan", "caesura_metrics", "caesura_report"],
            "declension": ["declension_classify", "declension_inflect", "declension_agree", "declension_prodrop"],
            "vigil_system": ["vigil_sys_start", "vigil_sys_stop", "vigil_sys_status", "vigil_sys_boundaries", "vigil_sys_add_boundary", "vigil_sys_ledger_query", "vigil_sys_ledger_verify", "vigil_sys_stats"],
            "dna": ["dna_encode", "dna_decode", "dna_eval", "dna_compile_asm", "dna_tile", "dna_voxel", "dna_pv_signal", "dna_profile_drug", "dna_catalog", "dna_nexcore_genome"],
            "lessons": ["lesson_add", "lesson_get", "lesson_search", "lesson_by_context", "lesson_by_tag", "primitives_summary"],
            "claude_repl": ["claude_repl"],
            "adventure": ["adventure_start", "adventure_task", "adventure_skill", "adventure_measure", "adventure_milestone", "adventure_status"],
            "borrow_miner": ["mine", "drop_ore", "get_state", "signal_check"],
            "reproductive": ["reproductive_guard_mutation", "reproductive_specialize_agent", "reproductive_start_mitosis"],
            "proof_of_meaning": ["pom_distill", "pom_chromatograph", "pom_titrate", "pom_prove_equivalence", "pom_registry_stats"],
            "crew": ["crew_assign", "crew_task_status", "crew_fuse_decisions"],
            "retrieval": ["retrieval_query", "retrieval_ingest", "retrieval_stats"],
            "hitl": ["hitl_submit", "hitl_queue", "hitl_review", "hitl_stats"],
            "pv_embeddings": ["pv_embedding_similarity", "pv_embedding_get", "pv_embedding_stats"],
            "grounded": ["grounded_uncertain", "grounded_require", "grounded_compose", "grounded_evidence_new", "grounded_evidence_step", "grounded_evidence_get", "grounded_skill_assess"],
            "highway": ["highway_classify", "highway_quality", "highway_destructive", "highway_legitimate_field", "highway_traffic_census", "highway_parallel_plan", "highway_interchange", "highway_grade_separate"],
            "routing": ["tool_route", "tool_dag", "tool_deps", "tool_chain"],
            "validify": ["validify_run", "validify_gate", "validify_gates_list"],
            "ctvp": ["ctvp_score", "ctvp_five_problems", "ctvp_phases_list"],
            "code_inspect": ["code_inspect_audit", "code_inspect_score", "code_inspect_criteria"],
            "primitive_coverage": ["primitive_coverage_check", "primitive_coverage_rules"],
            "model_delegation": ["model_route", "model_compare", "model_list"],
            "prompt_kinetics": ["prompt_kinetics_analyze", "prompt_bioavailability", "prompt_kinetics_model"],
            "compounding": ["compounding_velocity", "compounding_loop_health", "compounding_metrics"],
            "polymer": ["polymer_compose", "polymer_validate", "polymer_analyze"],
            "nervous": ["nervous_reflex", "nervous_latency", "nervous_myelination", "nervous_health"],
            "cardiovascular": ["cardio_blood_pressure", "cardio_blood_health", "cardio_diagnose", "cardio_vitals"],
            "lymphatic": ["lymphatic_drainage", "lymphatic_thymic", "lymphatic_inspect", "lymphatic_health"],
            "respiratory": ["respiratory_exchange", "respiratory_dead_space", "respiratory_tidal", "respiratory_health"],
            "urinary": ["urinary_pruning", "urinary_expiry", "urinary_retention", "urinary_health"],
            "integumentary": ["integumentary_permission", "integumentary_settings", "integumentary_sandbox", "integumentary_scarring", "integumentary_health"],
            "kellnr_pk": ["kellnr_compute_pk_auc", "kellnr_compute_pk_steady_state", "kellnr_compute_pk_ionization", "kellnr_compute_pk_clearance", "kellnr_compute_pk_volume_distribution", "kellnr_compute_pk_michaelis_menten"],
            "kellnr_thermo": ["kellnr_compute_thermo_gibbs", "kellnr_compute_thermo_kd", "kellnr_compute_thermo_binding_affinity", "kellnr_compute_thermo_arrhenius"],
            "kellnr_stats": ["kellnr_compute_stats_welch_ttest", "kellnr_compute_stats_ols_regression", "kellnr_compute_stats_poisson_ci", "kellnr_compute_stats_bayesian_posterior", "kellnr_compute_stats_entropy"],
            "kellnr_graph": ["kellnr_compute_graph_betweenness", "kellnr_compute_graph_mutual_info", "kellnr_compute_graph_tarjan_scc", "kellnr_compute_graph_topsort"],
            "kellnr_dtree": ["kellnr_compute_dtree_feature_importance", "kellnr_compute_dtree_prune", "kellnr_compute_dtree_to_rules"],
            "kellnr_signal": ["kellnr_compute_signal_sprt", "kellnr_compute_signal_cusum", "kellnr_compute_signal_weibull_tto"],
            "kellnr_registry": ["kellnr_search_crates", "kellnr_get_crate_metadata", "kellnr_list_crate_versions", "kellnr_get_version_details", "kellnr_list_owners", "kellnr_add_owner", "kellnr_remove_owner", "kellnr_yank_version", "kellnr_unyank_version", "kellnr_list_all_crates", "kellnr_get_dependencies", "kellnr_get_dependents", "kellnr_health_check", "kellnr_download_crate", "kellnr_registry_stats"],
            "registry": ["registry_assess_skill", "registry_assess_all", "registry_gap_report", "registry_promotable", "registry_promotion_plan", "registry_tov_safety", "registry_tov_harm", "registry_tov_is_safe"],
            "stoichiometry": ["stoichiometry_encode", "stoichiometry_decode", "stoichiometry_sisters", "stoichiometry_mass_state", "stoichiometry_dictionary", "stoichiometry_is_balanced", "stoichiometry_prove", "stoichiometry_is_isomer"],
            "graph_layout": ["graph_layout_converge"],
            "career": ["career_transitions"],
            "learning_dag": ["learning_dag_resolve"],
            "drift": ["drift_ks_test", "drift_psi", "drift_jsd", "drift_detect"],
            "rate_limit": ["rate_limit_token_bucket", "rate_limit_sliding_window", "rate_limit_status"],
            "rank_fusion": ["rank_fusion_rrf", "rank_fusion_hybrid", "rank_fusion_borda"],
            "security_posture": ["security_posture_assess", "security_threat_readiness", "security_compliance_gap"],
            "observability": ["observability_record_latency", "observability_query", "observability_freshness"],
            "epidemiology": ["epidemiology_relative_risk", "epidemiology_odds_ratio", "epidemiology_attributable_risk", "epidemiology_nnt_nnh", "epidemiology_attributable_fraction", "epidemiology_population_attributable_fraction", "epidemiology_incidence_rate", "epidemiology_prevalence", "epidemiology_kaplan_meier", "epidemiology_smr", "epidemiology_mappings"],
            "trial": ["trial_protocol_register", "trial_power_analysis", "trial_randomize", "trial_blind_verify", "trial_interim_analyze", "trial_safety_check", "trial_endpoint_evaluate", "trial_multiplicity_adjust", "trial_adapt_decide", "trial_report_generate"],
            "cognition": ["cognition_process", "cognition_analyze", "cognition_forward", "cognition_entropy", "cognition_perplexity", "cognition_embed", "cognition_sample", "cognition_confidence"],
            "notebooklm": ["nlm_add_notebook", "nlm_list_notebooks", "nlm_get_notebook", "nlm_select_notebook", "nlm_update_notebook", "nlm_remove_notebook", "nlm_search_notebooks", "nlm_get_library_stats", "nlm_list_sessions", "nlm_close_session", "nlm_reset_session", "nlm_get_health", "nlm_setup_auth", "nlm_re_auth", "nlm_ask_question", "nlm_cleanup_data"],
            "cloud": ["cloud_primitive_composition", "cloud_transfer_confidence", "cloud_tier_classify", "cloud_compare_types", "cloud_reverse_synthesize", "cloud_list_types", "cloud_molecular_weight", "cloud_dominant_shift", "cloud_infra_status", "cloud_infra_map", "cloud_capacity_project", "cloud_supervisor_health", "cloud_reverse_transfer", "cloud_transfer_chain", "cloud_architecture_advisor", "cloud_anomaly_detect", "cloud_transfer_matrix"],
            "zeta": ["zeta_compute", "zeta_find_zeros", "zeta_verify_rh", "zeta_embedded_zeros", "zeta_lmfdb_parse", "zeta_telescope_run", "zeta_batch_run", "zeta_scaling_fit", "zeta_scaling_predict", "zeta_cayley", "zeta_operator_hunt", "zeta_operator_candidate", "zeta_gue_compare"],
            "signal_pipeline": ["pipeline_compute_all", "pipeline_batch_compute", "pipeline_detect", "pipeline_validate", "pipeline_thresholds", "pipeline_report", "pipeline_relay_chain", "pipeline_transfer", "pipeline_primitives"],
            "preemptive_pv": ["preemptive_reactive", "preemptive_gibbs", "preemptive_trajectory", "preemptive_severity", "preemptive_noise", "preemptive_predictive", "preemptive_evaluate", "preemptive_intervention", "preemptive_required_strength", "preemptive_omega_table"],
            "openfda": ["openfda_drug_events", "openfda_drug_labels", "openfda_drug_recalls", "openfda_drug_ndc", "openfda_drugs_at_fda", "openfda_device_events", "openfda_device_recalls", "openfda_food_recalls", "openfda_food_events", "openfda_substances", "openfda_fan_out"],
            "compound_registry": ["compound_resolve", "compound_resolve_batch", "compound_cache_search", "compound_cache_get", "compound_cache_count"],
            "fhir": ["fhir_adverse_event_to_signal", "fhir_batch_to_signals", "fhir_parse_bundle", "fhir_validate_resource"],
            "retrocasting": ["retro_structural_similarity", "retro_signal_significance", "retro_cluster_signals", "retro_correlate_alerts", "retro_extract_features", "retro_dataset_stats"],
            "engram": ["engram_search", "engram_search_decay", "engram_peek", "engram_stats", "engram_find_duplicates", "engram_decay_score", "engram_ingest", "engram_by_source"],
            "ghost": ["ghost_boundary_check", "ghost_mode_info", "ghost_category_policy", "ghost_scan_pii", "ghost_scrub_fields"],
            "pharma_rd": ["pharma_taxonomy_summary", "pharma_lookup_transfer", "pharma_transfer_matrix", "pharma_strongest_transfers", "pharma_weakest_transfers", "pharma_symbol_coverage", "pharma_pipeline_stage", "pharma_classify_generators"],
            "combinatorics": ["comb_catalan", "comb_catalan_table", "comb_cycle_decomposition", "comb_min_transpositions", "comb_derangement", "comb_derangement_probability", "comb_grid_paths", "comb_binomial", "comb_multinomial", "comb_josephus", "comb_elimination_order", "comb_linear_extensions"],
            "tov_grounded": ["tov_grounded_signal_strength", "tov_grounded_safety_margin", "tov_grounded_stability_shell", "tov_grounded_harm_type", "tov_grounded_meta_vigilance", "tov_grounded_eka_intelligence", "tov_grounded_magic_numbers"],
            "statemind": ["statemind_analyze_word", "statemind_constellation"],
            "reason": ["reason_infer", "reason_counterfactual"],
            "word": ["word_analyze", "word_popcount", "word_hamming_distance", "word_parity", "word_rotate", "word_log2", "word_isqrt", "word_binary_gcd", "word_bit_test", "word_align_up"],
            "harm_taxonomy": ["harm_classify", "harm_definition", "harm_catalog", "harm_exhaustiveness", "harm_axiom_connection", "harm_axiom_catalog", "harm_combinations", "harm_manifestation_derive"],
            "antibodies": ["antibody_compute_affinity", "antibody_classify_response", "antibody_ig_info", "antibody_ig_catalog"],
            "jeopardy": ["jeopardy_clue_values", "jeopardy_categories", "jeopardy_score_board", "jeopardy_should_buzz", "jeopardy_optimal_dd_wager", "jeopardy_optimal_final_wager", "jeopardy_board_control_value", "jeopardy_compound_velocity"],
            "audio": ["audio_spec_compute", "audio_spec_presets", "audio_format_info", "audio_rate_info", "audio_convert_sample", "audio_resample", "audio_codec_catalog", "audio_device_capabilities", "audio_mixer_pan", "audio_stream_transitions"],
            "compilation_space": ["compilation_point_compare", "compilation_point_summary", "compilation_point_presets", "compilation_catalog_lookup", "compilation_chain_validate", "compilation_chain_presets", "compilation_axes_catalog", "compilation_abstraction_levels", "compilation_distance"],
            "pharmacovigilance": ["pv_taxonomy_summary", "pv_taxonomy_primitive", "pv_taxonomy_composite", "pv_taxonomy_concept", "pv_taxonomy_chomsky", "pv_taxonomy_who_pillars", "pv_taxonomy_transfer", "pv_taxonomy_transfer_matrix", "pv_taxonomy_lex_symbols"],
            "vault": ["vault_derive_key", "vault_encrypt", "vault_decrypt", "vault_generate_salt", "vault_config_sample"],
            "build_orchestrator": ["build_orchestrator_dry_run", "build_orchestrator_stages", "build_orchestrator_workspace", "build_orchestrator_history", "build_orchestrator_metrics"],
            "skills_engine": ["skill_quality_index", "skill_maturity", "skill_ksb_verify", "skill_ecosystem_score", "skill_dependency_graph", "skill_gap_analysis", "skill_evolution_track"],
            "ncbi": ["ncbi_esearch", "ncbi_esummary", "ncbi_efetch", "ncbi_elink", "ncbi_search_and_fetch", "ncbi_search_and_summarize"],
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&catalog).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
