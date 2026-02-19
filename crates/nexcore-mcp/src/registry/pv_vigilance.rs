// ========================================================================
// PV Signal Detection Tools (8)
// ========================================================================

#[tool(
    description = "Complete signal analysis using all 5 algorithms (PRR, ROR, IC, EBGM, Chi-square) on a 2x2 contingency table."
)]
pub async fn pv_signal_complete(
    &self,
    Parameters(params): Parameters<params::pv::SignalCompleteParams>
) -> Result<CallToolResult, McpError> {
    tools::pv::signal_complete(params)
}

#[tool(
    description = "Calculate Proportional Reporting Ratio (PRR) for pharmacovigilance signal detection."
)]
pub async fn pv_signal_prr(
    &self,
    Parameters(params): Parameters<params::pv::SignalAlgorithmParams>
) -> Result<CallToolResult, McpError> {
    tools::pv::signal_prr(params)
}

#[tool(
    description = "Calculate Reporting Odds Ratio (ROR) for pharmacovigilance signal detection."
)]
pub async fn pv_signal_ror(
    &self,
    Parameters(params): Parameters<params::pv::SignalAlgorithmParams>
) -> Result<CallToolResult, McpError> {
    tools::pv::signal_ror(params)
}

#[tool(
    description = "Calculate Information Component (IC) using Bayesian shrinkage for pharmacovigilance signal detection."
)]
pub async fn pv_signal_ic(
    &self,
    Parameters(params): Parameters<params::pv::SignalAlgorithmParams>
) -> Result<CallToolResult, McpError> {
    tools::pv::signal_ic(params)
}

#[tool(
    description = "Calculate Empirical Bayes Geometric Mean (EBGM) for pharmacovigilance signal detection."
)]
pub async fn pv_signal_ebgm(
    &self,
    Parameters(params): Parameters<params::pv::SignalAlgorithmParams>
) -> Result<CallToolResult, McpError> {
    tools::pv::signal_ebgm(params)
}

#[tool(description = "Calculate Chi-square statistic for a 2x2 contingency table.")]
pub async fn pv_chi_square(
    &self,
    Parameters(params): Parameters<params::pv::SignalAlgorithmParams>
) -> Result<CallToolResult, McpError> {
    tools::pv::chi_square(params)
}

#[tool(
    description = "Naranjo causality assessment: quick 5-question assessment returning score (-4 to 13) and category (Definite/Probable/Possible/Doubtful)."
)]
pub async fn pv_naranjo_quick(
    &self,
    Parameters(params): Parameters<params::pv::NaranjoParams>
) -> Result<CallToolResult, McpError> {
    tools::pv::naranjo_quick(params)
}

#[tool(
    description = "WHO-UMC causality assessment: returns category (Certain/Probable/Possible/Unlikely/Conditional/Unassessable) and description."
)]
pub async fn pv_who_umc_quick(
    &self,
    Parameters(params): Parameters<params::pv::WhoUmcParams>
) -> Result<CallToolResult, McpError> {
    tools::pv::who_umc_quick(params)
}

// ========================================================================
// Signal Pipeline Tools (3) - signal-stats / signal-core crates
// ========================================================================

#[tool(
    description = "Detect signal for a drug-event pair using the signal-stats pipeline (PRR, ROR, IC, EBGM, Chi-square, SignalStrength). Returns all metrics with strength classification."
)]
pub async fn signal_detect(
    &self,
    Parameters(params): Parameters<params::pv::SignalDetectParams>
) -> Result<CallToolResult, McpError> {
    tools::signal::signal_detect(params)
}

#[tool(
    description = "Batch signal detection for multiple drug-event pairs. Returns results array with signals_found count."
)]
pub async fn signal_batch(
    &self,
    Parameters(params): Parameters<params::pv::SignalBatchParams>
) -> Result<CallToolResult, McpError> {
    tools::signal::signal_batch(params)
}

#[tool(
    description = "Get signal detection threshold configurations: Evans (default), Strict (fewer false positives), Sensitive (fewer false negatives)."
)]
pub async fn signal_thresholds(&self) -> Result<CallToolResult, McpError> {
    tools::signal::signal_thresholds()
}

// ========================================================================
// PVDSL Tools (4) - Pharmacovigilance Domain-Specific Language
// ========================================================================

#[tool(
    description = "Compile PVDSL source code to bytecode. Validates syntax and returns compilation stats."
)]
pub async fn pvdsl_compile(
    &self,
    Parameters(params): Parameters<params::pvdsl::PvdslCompileParams>
) -> Result<CallToolResult, McpError> {
    tools::pvdsl::pvdsl_compile(params)
}

#[tool(
    description = "Execute PVDSL source code with optional variables. Supports signal detection (PRR, ROR, IC, EBGM, SPRT, MaxSPRT, CuSum, MGPS), causality (Naranjo, WHO-UMC), and math functions."
)]
pub async fn pvdsl_execute(
    &self,
    Parameters(params): Parameters<params::pvdsl::PvdslExecuteParams>
) -> Result<CallToolResult, McpError> {
    tools::pvdsl::pvdsl_execute(params)
}

#[tool(
    description = "Evaluate a single PVDSL expression. Example: signal::prr(10, 90, 100, 9800)"
)]
pub async fn pvdsl_eval(
    &self,
    Parameters(params): Parameters<params::pvdsl::PvdslEvalParams>
) -> Result<CallToolResult, McpError> {
    tools::pvdsl::pvdsl_eval(params)
}

#[tool(
    description = "List all available PVDSL functions organized by namespace (signal, causality, meddra, math, risk, date, classify)."
)]
pub async fn pvdsl_functions(&self) -> Result<CallToolResult, McpError> {
    tools::pvdsl::pvdsl_functions()
}

// ========================================================================
// Vigilance Tools (4)
// ========================================================================

#[tool(
    description = "Calculate safety margin d(s) - signed distance to harm boundary based on signal metrics. ToV axiom implementation."
)]
pub async fn vigilance_safety_margin(
    &self,
    Parameters(params): Parameters<params::vigilance::SafetyMarginParams>
) -> Result<CallToolResult, McpError> {
    tools::vigilance::safety_margin(params)
}

#[tool(
    description = "Guardian-AV risk scoring: calculate overall risk (0-100) with contributing factors."
)]
pub async fn vigilance_risk_score(
    &self,
    Parameters(params): Parameters<params::vigilance::RiskScoreParams>
) -> Result<CallToolResult, McpError> {
    tools::vigilance::risk_score(params)
}

#[tool(
    description = "List all 8 ToV harm types (A-H) with their conservation laws, letters, and affected hierarchy levels."
)]
pub async fn vigilance_harm_types(&self) -> Result<CallToolResult, McpError> {
    tools::vigilance::harm_types()
}

#[tool(
    description = "Map a SafetyLevel (1-8) to its ToV hierarchy level. Levels: Molecular(1-2), Physiological(3-5), Clinical(6), Population(7), Regulatory(8)."
)]
pub async fn vigilance_map_to_tov(
    &self,
    Parameters(params): Parameters<params::vigilance::MapToTovParams>
) -> Result<CallToolResult, McpError> {
    tools::vigilance::map_to_tov(params)
}

// ========================================================================
// Compliance Tools (3) - SAM.gov, OSCAL, ICH Controls
// ========================================================================

#[tool(
    description = "Check SAM.gov for federal entity exclusions (debarment, suspension). Query by UEI, CAGE code, or entity name. Requires SAM_GOV_API_KEY env var."
)]
pub async fn compliance_check_exclusion(
    &self,
    Parameters(params): Parameters<params::compliance::ComplianceCheckExclusionParams>
) -> Result<CallToolResult, McpError> {
    tools::compliance::check_exclusion(params).await
}

#[tool(
    description = "Run compliance assessment on controls. Evaluates control status and findings to determine Compliant/NonCompliant/Inconclusive result."
)]
pub async fn compliance_assess(
    &self,
    Parameters(params): Parameters<params::compliance::ComplianceAssessParams>
) -> Result<CallToolResult, McpError> {
    tools::compliance::assess(params)
}

#[tool(
    description = "Get pre-populated ICH control catalog (E2A-E2E guidelines). Optionally filter by guideline code. Returns 12 pharmacovigilance controls."
)]
pub async fn compliance_catalog_ich(
    &self,
    Parameters(params): Parameters<params::compliance::ComplianceCatalogParams>
) -> Result<CallToolResult, McpError> {
    tools::compliance::catalog_ich(params)
}

#[tool(
    description = "Get SEC EDGAR filings for a company by CIK. Filter by form type (10-K, 10-Q, 8-K). No auth required."
)]
pub async fn compliance_sec_filings(
    &self,
    Parameters(params): Parameters<params::compliance::ComplianceSecFilingsParams>
) -> Result<CallToolResult, McpError> {
    tools::compliance::sec_filings(params).await
}

#[tool(
    description = "Get SEC 10-K filings for major pharma companies. Companies: pfizer, jnj, merck, abbvie, bms, lilly, amgen, gilead, regeneron, moderna."
)]
pub async fn compliance_sec_pharma(
    &self,
    Parameters(params): Parameters<params::compliance::ComplianceSecPharmaParams>
) -> Result<CallToolResult, McpError> {
    tools::compliance::sec_pharma(params).await
}

// ========================================================================
// End-to-End PV Pipeline (1)
// ========================================================================

#[tool(
    description = "End-to-end pharmacovigilance pipeline: FAERS \u2192 Signal Detection \u2192 Guardian Risk. Queries live FDA data, runs all signal algorithms (PRR, ROR, IC, EBGM, \u03c7\u00b2), and returns Guardian risk assessment with recommended actions. Validated capability with 19M+ FAERS reports."
)]
pub async fn pv_pipeline(
    &self,
    Parameters(params): Parameters<params::pv::PvPipelineParams>
) -> Result<CallToolResult, McpError> {
    tools::pv_pipeline::run_pipeline(params).await
}

// ========================================================================
// PV Axioms Database (5)
// ========================================================================

#[tool(
    description = "Look up KSBs (Knowledge, Skill, Behavior, AI Integration items) from the PV axioms database. Filter by ksb_id, domain_id (D01-D15), ksb_type, or keyword search. Returns 1,462 KSBs across 15 PV domains with regulatory refs, EPA/CPA mappings, and Bloom levels."
)]
pub async fn pv_axioms_ksb_lookup(
    &self,
    Parameters(params): Parameters<params::axioms::PvAxiomsKsbLookupParams>
) -> Result<CallToolResult, McpError> {
    tools::pv_axioms::ksb_lookup(params)
}

#[tool(
    description = "Search 341 PV regulations (FDA, EMA, ICH, CIOMS) by text query, jurisdiction, or domain. Returns official identifiers, key requirements, compliance risk levels, and harmonization status."
)]
pub async fn pv_axioms_regulation_search(
    &self,
    Parameters(params): Parameters<params::axioms::PvAxiomsRegulationSearchParams>
) -> Result<CallToolResult, McpError> {
    tools::pv_axioms::regulation_search(params)
}

#[tool(
    description = "Trace regulatory axioms through the full chain: axiom \u2192 parameter \u2192 pipeline stage \u2192 Rust implementation coverage. Filter by axiom_id, source_guideline (e.g. 'ICH E2A'), or primitive symbol."
)]
pub async fn pv_axioms_traceability_chain(
    &self,
    Parameters(params): Parameters<params::axioms::PvAxiomsTraceabilityParams>
) -> Result<CallToolResult, McpError> {
    tools::pv_axioms::traceability_chain(params)
}

#[tool(
    description = "Domain dashboard showing KSB counts by type, regulation counts, EPA mappings, and coverage metrics. Specify domain_id for one domain or omit for all 15."
)]
pub async fn pv_axioms_domain_dashboard(
    &self,
    Parameters(params): Parameters<params::axioms::PvAxiomsDomainDashboardParams>
) -> Result<CallToolResult, McpError> {
    tools::pv_axioms::domain_dashboard(params)
}

#[tool(
    description = "Raw SQL query against the PV axioms database (read-only, max 100 rows). Access 68 tables and 14 views including ksbs, regulations, axioms, domains, epas, primitives, and traceability views."
)]
pub async fn pv_axioms_query(
    &self,
    Parameters(params): Parameters<params::axioms::PvAxiomsQueryParams>
) -> Result<CallToolResult, McpError> {
    tools::pv_axioms::query(params)
}

// ========================================================================
// Value Mining Tools (4) - Economic Signal Detection (PV Algorithms)
// ========================================================================

#[tool(
    description = "List all value signal types with their PV algorithm analogs and T1 primitives. Types: Sentiment (PRR), Trend (IC), Engagement (ROR), Virality (EBGM), Controversy (Chi\u00b2). Each type maps to a pharmacovigilance algorithm for cross-domain transfer."
)]
pub async fn value_signal_types(
    &self,
    Parameters(params): Parameters<params::ValueSignalTypesParams>
) -> Result<CallToolResult, McpError> {
    tools::value_mining::list_signal_types(params)
}

#[tool(
    description = "Detect a value signal from numeric observations. Applies PV algorithm analog (PRR, IC, ROR, EBGM, Chi\u00b2) based on signal type. Returns score, confidence, strength, and interpretation. Params: signal_type, observed, baseline, sample_size, entity, source."
)]
pub async fn value_signal_detect(
    &self,
    Parameters(params): Parameters<params::ValueSignalDetectParams>
) -> Result<CallToolResult, McpError> {
    tools::value_mining::detect_signal(params)
}

#[tool(
    description = "Create a baseline for signal detection. Sets expected rates for sentiment, engagement, and posting frequency. Used as comparison point for detecting signals."
)]
pub async fn value_baseline_create(
    &self,
    Parameters(params): Parameters<params::ValueBaselineCreateParams>
) -> Result<CallToolResult, McpError> {
    tools::value_mining::create_baseline(params)
}

#[tool(
    description = "Get the PV \u2194 Value Mining algorithm mapping. Shows how pharmacovigilance algorithms (PRR, ROR, IC, EBGM, Chi\u00b2) transfer to economic signal detection with confidence scores and T1 primitive groundings."
)]
pub async fn value_pv_mapping(
    &self,
    Parameters(params): Parameters<params::ValuePvMappingParams>
) -> Result<CallToolResult, McpError> {
    tools::value_mining::get_pv_mapping(params)
}

// \u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550
// ALGOVIGILANCE (6 tools) - ICSR deduplication + signal triage
// \u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550

#[tool(
    description = "Compare two ICSR narratives for similarity using Jaccard tokenization with medical stopword removal. Returns similarity score, threshold comparison, and duplicate verdict."
)]
pub async fn algovigil_dedup_pair(
    &self,
    Parameters(params): Parameters<params::algovigilance::AlgovigilDedupPairParams>
) -> Result<CallToolResult, McpError> {
    tools::algovigilance::dedup_pair(params)
}

#[tool(
    description = "Configure batch ICSR deduplication for a drug. Sets up DedupFunction with threshold and limit. Use algovigil_dedup_pair for pairwise comparison."
)]
pub async fn algovigil_dedup_batch(
    &self,
    Parameters(params): Parameters<params::algovigilance::AlgovigilDedupBatchParams>
) -> Result<CallToolResult, McpError> {
    tools::algovigilance::dedup_batch(params)
}

#[tool(
    description = "Get signal with current decay-adjusted relevance. Looks up signal by drug+event in persisted queue, applies exponential decay (half-life configurable)."
)]
pub async fn algovigil_triage_decay(
    &self,
    Parameters(params): Parameters<params::algovigilance::AlgovigilTriageDecayParams>
) -> Result<CallToolResult, McpError> {
    tools::algovigilance::triage_decay(params)
}

#[tool(
    description = "Reinforce a signal with new case evidence. Restores confidence toward original level proportional to new cases."
)]
pub async fn algovigil_triage_reinforce(
    &self,
    Parameters(params): Parameters<params::algovigilance::AlgovigilTriageReinforceParams>
) -> Result<CallToolResult, McpError> {
    tools::algovigilance::triage_reinforce(params)
}

#[tool(
    description = "Get prioritized signal queue for a drug. Returns signals sorted by decay-adjusted relevance with configurable half-life and cutoff."
)]
pub async fn algovigil_triage_queue(
    &self,
    Parameters(params): Parameters<params::algovigilance::AlgovigilTriageQueueParams>
) -> Result<CallToolResult, McpError> {
    tools::algovigilance::triage_queue(params)
}

#[tool(
    description = "Algovigilance system status. Returns store health, synonym count, queue count, and registered function metadata."
)]
pub async fn algovigil_status(&self) -> Result<CallToolResult, McpError> {
    tools::algovigilance::status()
}
