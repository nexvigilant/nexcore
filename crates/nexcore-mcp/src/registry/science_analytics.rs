// ========================================================================
// Universal Validation Tools
// ========================================================================

#[tool(
    description = "Run L1-L5 validation on a target. Validates coherence (L1), structure (L2), function (L3), operations (L4), and impact (L5). Auto-detects domain or specify explicitly. Returns detailed check results per level."
)]
pub async fn validation_run(
    &self,
    Parameters(params): Parameters<params::validation::ValidationRunParams>
) -> Result<CallToolResult, McpError> {
    tools::validation::run(params)
}

#[tool(
    description = "Quick L1-L2 validation check. Fast coherence and structural validation only. Use for rapid sanity checks before full validation."
)]
pub async fn validation_check(
    &self,
    Parameters(params): Parameters<params::validation::ValidationCheckParams>
) -> Result<CallToolResult, McpError> {
    tools::validation::check(params)
}

#[tool(
    description = "List available validation domains and L1-L5 level definitions. Shows detection patterns for each domain."
)]
pub async fn validation_domains(
    &self,
    Parameters(params): Parameters<params::validation::ValidationDomainsParams>
) -> Result<CallToolResult, McpError> {
    tools::validation::domains(params)
}

#[tool(
    description = "Classify tests in Rust source into 5 categories: Positive (happy path), Negative (error handling), Edge (boundary conditions), Stress (performance), Adversarial (security). Returns distribution, coverage gaps, and recommendations."
)]
pub async fn validation_classify_tests(
    &self,
    Parameters(params): Parameters<params::validation::ValidationClassifyTestsParams>
) -> Result<CallToolResult, McpError> {
    tools::validation::classify_tests_tool(params)
}

// ========================================================================
// Brain Tools (12) - Antigravity-style working memory
// ========================================================================

#[tool(
    description = "Create a new brain session for working memory. Returns session ID. Sessions store artifacts (task.md, plan.md) with versioning."
)]
pub async fn brain_session_create(
    &self,
    Parameters(params): Parameters<params::brain::BrainSessionCreateParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::session_create(params)
}

#[tool(
    description = "Load an existing brain session by ID. Returns session details and artifact list."
)]
pub async fn brain_session_load(
    &self,
    Parameters(params): Parameters<params::brain::BrainSessionLoadParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::session_load(params)
}

#[tool(
    description = "List all brain sessions, most recent first. Returns session IDs, dates, and projects."
)]
pub async fn brain_sessions_list(
    &self,
    Parameters(params): Parameters<params::brain::BrainSessionsListParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::sessions_list(params)
}

#[tool(
    description = "Save an artifact to a brain session. Artifact types: task, plan, walkthrough, review, research, decision, custom."
)]
pub async fn brain_artifact_save(
    &self,
    Parameters(params): Parameters<params::brain::BrainArtifactSaveParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::artifact_save(params)
}

#[tool(
    description = "Resolve an artifact, creating an immutable snapshot. Creates .resolved and .resolved.N files. Returns version number."
)]
pub async fn brain_artifact_resolve(
    &self,
    Parameters(params): Parameters<params::brain::BrainArtifactResolveParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::artifact_resolve(params)
}

#[tool(
    description = "Get an artifact's content. Specify version for a specific resolved snapshot, or omit for current mutable state."
)]
pub async fn brain_artifact_get(
    &self,
    Parameters(params): Parameters<params::brain::BrainArtifactGetParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::artifact_get(params)
}

#[tool(
    description = "Diff two resolved versions of an artifact. Returns line-based diff showing additions and removals."
)]
pub async fn brain_artifact_diff(
    &self,
    Parameters(params): Parameters<params::brain::BrainArtifactDiffParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::artifact_diff(params)
}

#[tool(
    description = "Track a file for content-addressable change detection. Stores SHA-256 hash and file copy."
)]
pub async fn code_tracker_track(
    &self,
    Parameters(params): Parameters<params::brain::BrainCodeTrackerTrackParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::code_tracker_track(params)
}

#[tool(
    description = "Check if a tracked file has changed since it was tracked. Returns changed: true/false."
)]
pub async fn code_tracker_changed(
    &self,
    Parameters(params): Parameters<params::brain::BrainCodeTrackerChangedParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::code_tracker_changed(params)
}

#[tool(description = "Get the original content of a tracked file when it was first tracked.")]
pub async fn code_tracker_original(
    &self,
    Parameters(params): Parameters<params::brain::BrainCodeTrackerOriginalParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::code_tracker_original(params)
}

#[tool(
    description = "Get a learned preference from implicit knowledge. Returns value and confidence score."
)]
pub async fn implicit_get(
    &self,
    Parameters(params): Parameters<params::brain::BrainImplicitGetParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::implicit_get(params)
}

#[tool(
    description = "Set a learned preference in implicit knowledge. Value should be JSON. Preferences are reinforced with repeated use."
)]
pub async fn implicit_set(
    &self,
    Parameters(params): Parameters<params::brain::BrainImplicitSetParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::implicit_set(params)
}

#[tool(
    description = "Get implicit knowledge statistics including total patterns, corrections, preferences, decayed patterns (confidence < 0.5 after time-decay), and ungrounded patterns (no T1 primitive assigned)."
)]
pub async fn implicit_stats(&self) -> Result<CallToolResult, McpError> {
    tools::brain::implicit_stats()
}

#[tool(
    description = "Find corrections using fuzzy token matching (Jaccard similarity). Returns corrections above the similarity threshold. Default threshold: 0.3."
)]
pub async fn implicit_find_corrections(
    &self,
    Parameters(params): Parameters<params::brain::BrainImplicitFindCorrectionsParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::implicit_find_corrections(params)
}

#[tool(
    description = "List patterns filtered by T1 primitive grounding (sequence, mapping, recursion, state, void). Returns patterns with their decay-adjusted effective confidence."
)]
pub async fn implicit_patterns_by_grounding(
    &self,
    Parameters(params): Parameters<params::brain::BrainImplicitPatternsByGroundingParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::implicit_patterns_by_grounding(params)
}

#[tool(
    description = "List all patterns sorted by decay-adjusted relevance (effective confidence descending). Shows which patterns are still strong vs fading."
)]
pub async fn implicit_patterns_by_relevance(&self) -> Result<CallToolResult, McpError> {
    tools::brain::implicit_patterns_by_relevance()
}

#[tool(
    description = "Check brain health status. Returns overall status (healthy/degraded), index health, and partial writes."
)]
pub async fn brain_recovery_check(&self) -> Result<CallToolResult, McpError> {
    tools::brain::recovery_check()
}

#[tool(
    description = "Repair partial writes by creating missing metadata for artifacts. Operates on specified session or latest."
)]
pub async fn brain_recovery_repair(
    &self,
    Parameters(params): Parameters<params::brain::BrainRecoveryRepairParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::recovery_repair(params.session_id.as_deref())
}

#[tool(
    description = "Rebuild session index from session directories. Use when index is corrupted or missing."
)]
pub async fn brain_recovery_rebuild_index(&self) -> Result<CallToolResult, McpError> {
    tools::brain::recovery_rebuild_index()
}

#[tool(
    description = "Attempt automatic brain recovery. Checks health and repairs if auto_recovery is enabled."
)]
pub async fn brain_recovery_auto(&self) -> Result<CallToolResult, McpError> {
    tools::brain::recovery_auto()
}

#[tool(
    description = "Acquire a file lock for agent coordination. Prevents race conditions in multi-agent environments. Returns success status and lock info. Idempotent."
)]
pub async fn brain_coordination_acquire(
    &self,
    Parameters(params): Parameters<params::brain::BrainCoordinationAcquireParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::coordination_acquire(params)
}

#[tool(
    description = "Release a file lock held by an agent. Allows other agents to access the file. Returns success status."
)]
pub async fn brain_coordination_release(
    &self,
    Parameters(params): Parameters<params::brain::BrainCoordinationReleaseParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::coordination_release(params)
}

#[tool(
    description = "Check the lock status of a file. Returns Vacant or Occupied with agent info."
)]
pub async fn brain_coordination_status(
    &self,
    Parameters(params): Parameters<params::brain::BrainCoordinationStatusParams>
) -> Result<CallToolResult, McpError> {
    tools::brain::coordination_status(params)
}

// ========================================================================
// Immunity Tools (6) - Antipattern Detection and Self-Regulation
// ========================================================================

#[tool(
    description = "Scan code content for known antipatterns. Returns threats with antibody IDs, severity, confidence, and response actions. Use before writing code to detect PAMPs/DAMPs."
)]
pub async fn immunity_scan(
    &self,
    Parameters(params): Parameters<params::immunity::ImmunityScanParams>
) -> Result<CallToolResult, McpError> {
    tools::immunity::immunity_scan(params)
}

#[tool(
    description = "Scan stderr output for known error patterns. Use after build/test failures to identify recurring antipatterns from compiler or tool output."
)]
pub async fn immunity_scan_errors(
    &self,
    Parameters(params): Parameters<params::immunity::ImmunityScanErrorsParams>
) -> Result<CallToolResult, McpError> {
    tools::immunity::immunity_scan_errors(params)
}

#[tool(
    description = "List all registered antibodies with optional filtering by threat type (PAMP/DAMP) and minimum severity (low/medium/high/critical)."
)]
pub async fn immunity_list(
    &self,
    Parameters(params): Parameters<params::immunity::ImmunityListParams>
) -> Result<CallToolResult, McpError> {
    tools::immunity::immunity_list(params)
}

#[tool(
    description = "Get detailed antibody by ID. Returns detection patterns, response strategy, Rust template, confidence, and learned_from context."
)]
pub async fn immunity_get(
    &self,
    Parameters(params): Parameters<params::immunity::ImmunityGetParams>
) -> Result<CallToolResult, McpError> {
    tools::immunity::immunity_get(params)
}

#[tool(
    description = "Propose new antibody from observed error/fix pair. Creates proposal for review at ~/.claude/immunity/proposals.yaml. Use after fixing a recurring bug."
)]
pub async fn immunity_propose(
    &self,
    Parameters(params): Parameters<params::immunity::ImmunityProposeParams>
) -> Result<CallToolResult, McpError> {
    tools::immunity::immunity_propose(params)
}

#[tool(
    description = "Get immunity system status. Returns registry version, antibody counts by type/severity, and homeostasis loop status."
)]
pub async fn immunity_status(&self) -> Result<CallToolResult, McpError> {
    tools::immunity::immunity_status()
}

// ========================================================================
// Regulatory Primitives Tools (3)
// ========================================================================

#[tool(
    description = "Extract regulatory primitives from FDA/ICH/CIOMS sources. Classifies terms into T1 (Universal), T2-P (Cross-Domain Primitive), T2-C (Cross-Domain Composite), or T3 (Domain-Specific). Returns primitive inventory with dependency graphs and source citations."
)]
pub async fn regulatory_primitives_extract(
    &self,
    Parameters(params): Parameters<params::regulatory::RegulatoryExtractParams>
) -> Result<CallToolResult, McpError> {
    tools::regulatory::extract(params)
}

#[tool(
    description = "Audit FDA vs CIOMS/ICH term consistency. Computes structural match (word overlap), semantic match (concept alignment), identifies discrepancies, and returns verdict: Aligned, MinorDeviation, MajorDeviation, or NoCorrespondence."
)]
pub async fn regulatory_primitives_audit(
    &self,
    Parameters(params): Parameters<params::regulatory::RegulatoryAuditParams>
) -> Result<CallToolResult, McpError> {
    tools::regulatory::audit(params)
}

#[tool(
    description = "Cross-domain transfer analysis between regulatory domains. Maps primitives from one domain (PV, Cloud, AI Safety, Finance) to another, computing transfer confidence scores."
)]
pub async fn regulatory_primitives_compare(
    &self,
    Parameters(params): Parameters<params::regulatory::RegulatoryCompareParams>
) -> Result<CallToolResult, McpError> {
    tools::regulatory::compare(params)
}

// ========================================================================
// Brand Semantics Tools (3) - Primitive Extraction for Brand Names
// ========================================================================

#[tool(
    description = "Get the pre-computed NexVigilant brand decomposition. Returns etymology roots, tier-classified primitives (T1/T2-P/T2-C/T3), primitive tests, transfer mappings to aviation domain, and semantic synthesis."
)]
pub async fn brand_decomposition_nexvigilant(&self) -> Result<CallToolResult, McpError> {
    tools::brand_semantics::brand_decomposition_nexvigilant()
}

#[tool(
    description = "Get a brand decomposition by name. Currently supports 'nexvigilant'. Returns full primitive extraction with tier classification and cross-domain transfer mappings."
)]
pub async fn brand_decomposition_get(
    &self,
    Parameters(params): Parameters<params::BrandDecompositionGetParams>
) -> Result<CallToolResult, McpError> {
    tools::brand_semantics::brand_decomposition_get(params)
}

#[tool(
    description = "Test if a term is a primitive using the 3-part test: (1) No domain-internal dependencies, (2) Grounds to external concepts, (3) Not merely a synonym. Returns verdict (Primitive/Composite) and tier classification (T1/T2-P/T2-C/T3)."
)]
pub async fn brand_primitive_test(
    &self,
    Parameters(params): Parameters<params::BrandPrimitiveTestParams>
) -> Result<CallToolResult, McpError> {
    tools::brand_semantics::brand_primitive_test(params)
}

#[tool(
    description = "List all semantic tiers (T1-Universal, T2-P-CrossDomainPrimitive, T2-C-CrossDomainComposite, T3-DomainSpecific) with definitions, examples, and transfer guidance."
)]
pub async fn brand_semantic_tiers(&self) -> Result<CallToolResult, McpError> {
    tools::brand_semantics::brand_semantic_tiers()
}

// ========================================================================
// Primitive Validation Tools (4) - Corpus-Backed with Professional Citations
// ========================================================================

#[tool(
    description = "Validate a primitive term against external corpus (ICH glossary, PubMed). Returns validation tier (1=Authoritative/2=Peer-Reviewed/3=Web/4=Expert), confidence score, definition, and professional citations in Vancouver format. Essential for regulatory compliance."
)]
pub async fn primitive_validate(
    &self,
    Parameters(params): Parameters<params::PrimitiveValidateParams>
) -> Result<CallToolResult, McpError> {
    tools::primitive_validation::validate(params).await
}

#[tool(
    description = "Generate a professional Vancouver-format citation for a PubMed ID (PMID) or DOI. Returns formatted citation with all metadata for regulatory documentation."
)]
pub async fn primitive_cite(
    &self,
    Parameters(params): Parameters<params::PrimitiveCiteParams>
) -> Result<CallToolResult, McpError> {
    tools::primitive_validation::cite(params).await
}

#[tool(
    description = "Batch validate multiple primitive terms. Returns validation summary with tier distribution, confidence scores, and aggregate statistics."
)]
pub async fn primitive_validate_batch(
    &self,
    Parameters(params): Parameters<params::PrimitiveValidateBatchParams>
) -> Result<CallToolResult, McpError> {
    tools::primitive_validation::validate_batch(params).await
}

#[tool(
    description = "List validation tiers and their confidence levels. Shows Tier 1 (Authoritative: ICH/FDA/EMA), Tier 2 (Peer-Reviewed: PubMed), Tier 3 (Validated Web), and Tier 4 (Expert Generation). Includes regulatory compliance notes."
)]
pub async fn primitive_validation_tiers(&self) -> Result<CallToolResult, McpError> {
    tools::primitive_validation::validation_tiers()
}

// ========================================================================
// Chemistry Primitives Tools (10) - Cross-Domain Transfer
// ========================================================================

#[tool(
    description = "Calculate Arrhenius rate constant (threshold gating). Maps to signal_detection_threshold in PV. PV transfer confidence: 0.92. k = A × e^(-Ea/RT)"
)]
pub async fn chemistry_threshold_rate(
    &self,
    Parameters(params): Parameters<params::ChemistryThresholdRateParams>
) -> Result<CallToolResult, McpError> {
    tools::chemistry::threshold_rate(params)
}

#[tool(
    description = "Calculate remaining amount after decay (half-life kinetics). Maps to signal_persistence in PV. PV transfer confidence: 0.90. N(t) = N₀ × e^(-kt)"
)]
pub async fn chemistry_decay_remaining(
    &self,
    Parameters(params): Parameters<params::ChemistryDecayRemainingParams>
) -> Result<CallToolResult, McpError> {
    tools::chemistry::decay_remaining(params)
}

#[tool(
    description = "Calculate Michaelis-Menten saturation rate. Maps to case_processing_capacity in PV. PV transfer confidence: 0.88. v = Vmax × [S] / (Km + [S])"
)]
pub async fn chemistry_saturation_rate(
    &self,
    Parameters(params): Parameters<params::ChemistrySaturationRateParams>
) -> Result<CallToolResult, McpError> {
    tools::chemistry::saturation_rate(params)
}

#[tool(
    description = "Calculate Gibbs free energy feasibility. Maps to causality_likelihood in PV. PV transfer confidence: 0.85. ΔG = ΔH - TΔS"
)]
pub async fn chemistry_feasibility(
    &self,
    Parameters(params): Parameters<params::ChemistryFeasibilityParams>
) -> Result<CallToolResult, McpError> {
    tools::chemistry::feasibility(params)
}

#[tool(
    description = "Calculate rate law dependency. Maps to signal_dependency in PV. PV transfer confidence: 0.82. rate = k[A]^n[B]^m"
)]
pub async fn chemistry_dependency_rate(
    &self,
    Parameters(params): Parameters<params::ChemistryDependencyRateParams>
) -> Result<CallToolResult, McpError> {
    tools::chemistry::dependency_rate(params)
}

#[tool(
    description = "Calculate buffer capacity (Henderson-Hasselbalch). Maps to baseline_stability in PV. PV transfer confidence: 0.78. β = 2.303 × C × ratio / (1+ratio)²"
)]
pub async fn chemistry_buffer_capacity(
    &self,
    Parameters(params): Parameters<params::ChemistryBufferCapacityParams>
) -> Result<CallToolResult, McpError> {
    tools::chemistry::buffer_cap(params)
}

#[tool(
    description = "Calculate Beer-Lambert absorbance. Maps to dose_response_linearity in PV. PV transfer confidence: 0.75. A = εlc"
)]
pub async fn chemistry_signal_absorbance(
    &self,
    Parameters(params): Parameters<params::ChemistrySignalAbsorbanceParams>
) -> Result<CallToolResult, McpError> {
    tools::chemistry::signal_absorbance(params)
}

#[tool(
    description = "Calculate equilibrium steady-state fractions. Maps to reporting_baseline in PV. PV transfer confidence: 0.72. Returns product and substrate fractions at equilibrium."
)]
pub async fn chemistry_equilibrium(
    &self,
    Parameters(params): Parameters<params::ChemistryEquilibriumParams>
) -> Result<CallToolResult, McpError> {
    tools::chemistry::equilibrium(params)
}

#[tool(
    description = "Get all chemistry → PV mappings. Returns 8 cross-domain transfer mappings with confidence scores (0.72-0.92) and rationale for each mapping."
)]
pub async fn chemistry_pv_mappings(
    &self,
    Parameters(params): Parameters<params::ChemistryPvMappingsParams>
) -> Result<CallToolResult, McpError> {
    tools::chemistry::get_pv_mappings(params)
}

#[tool(
    description = "Simple threshold exceeded check. Returns boolean: signal > threshold. Useful as a gate for signal detection workflows."
)]
pub async fn chemistry_threshold_exceeded(
    &self,
    Parameters(params): Parameters<params::ChemistryThresholdExceededParams>
) -> Result<CallToolResult, McpError> {
    tools::chemistry::check_threshold_exceeded(params)
}

#[tool(
    description = "Calculate Hill equation response (cooperative binding). Maps to signal_cascade_amplification in PV. PV confidence: 0.85. Y = I^nH / (K₀.₅^nH + I^nH). nH>1 amplifies, nH<1 dampens."
)]
pub async fn chemistry_hill_response(
    &self,
    Parameters(params): Parameters<params::ChemistryHillResponseParams>
) -> Result<CallToolResult, McpError> {
    tools::chemistry::hill_cooperative(params)
}

#[tool(
    description = "Calculate Nernst potential (dynamic threshold). Maps to dynamic_decision_threshold in PV. PV confidence: 0.80. E = E⁰ - (RT/nF)ln(Q). Threshold shifts with concentration."
)]
pub async fn chemistry_nernst_potential(
    &self,
    Parameters(params): Parameters<params::ChemistryNernstParams>
) -> Result<CallToolResult, McpError> {
    tools::chemistry::nernst_dynamic(params)
}

#[tool(
    description = "Calculate competitive inhibition rate. Maps to signal_interference_factor in PV. PV confidence: 0.78. v = Vmax[S] / (Km(1+[I]/Ki) + [S]). Competing signals raise threshold."
)]
pub async fn chemistry_inhibition_rate(
    &self,
    Parameters(params): Parameters<params::ChemistryInhibitionParams>
) -> Result<CallToolResult, McpError> {
    tools::chemistry::inhibition_rate(params)
}

#[tool(
    description = "Calculate Eyring rate (transition state theory). Maps to signal_escalation_rate in PV. PV confidence: 0.82. k = κ(kB*T/h)exp(-ΔG‡/RT). More accurate than Arrhenius."
)]
pub async fn chemistry_eyring_rate(
    &self,
    Parameters(params): Parameters<params::ChemistryEyringRateParams>
) -> Result<CallToolResult, McpError> {
    tools::chemistry::eyring_transition(params)
}

#[tool(
    description = "Calculate Langmuir coverage (resource binding). Maps to case_slot_occupancy in PV. PV confidence: 0.88. θ = K[A]/(1+K[A]). Finite slots, saturation behavior."
)]
pub async fn chemistry_langmuir_coverage(
    &self,
    Parameters(params): Parameters<params::ChemistryLangmuirParams>
) -> Result<CallToolResult, McpError> {
    tools::chemistry::langmuir_binding(params)
}

// ========================================================================
// Cytokine Signaling Tools (3) - Typed Event Bus (Immune System Patterns)
// ========================================================================

#[tool(
    description = "Emit a cytokine signal to the global event bus. Families: il1 (alarm), il2 (growth), il6 (acute), il10 (suppress), tnf_alpha (terminate), ifn_gamma (activate), tgf_beta (regulate), csf (spawn). Severities: trace, low, medium, high, critical. Scopes: autocrine, paracrine, endocrine, systemic."
)]
pub async fn cytokine_emit(
    &self,
    Parameters(params): Parameters<params::CytokineEmitParams>
) -> Result<CallToolResult, McpError> {
    tools::cytokine::emit(params)
}

#[tool(
    description = "Get cytokine bus status: signals emitted/delivered/dropped, cascades triggered, counts by family and severity. Shows current event bus health."
)]
pub async fn cytokine_status(&self) -> Result<CallToolResult, McpError> {
    tools::cytokine::status()
}

#[tool(
    description = "List all cytokine families with descriptions. Each family maps to an immune function: IL-1 (alarm), IL-6 (acute response), TNF-alpha (destroy threats), etc. Optional filter by family name."
)]
pub async fn cytokine_families(
    &self,
    Parameters(params): Parameters<params::CytokineListParams>
) -> Result<CallToolResult, McpError> {
    tools::cytokine::families(params)
}

// ========================================================================
// STEM Primitives Tools (11) - Cross-Domain T2-P Traits
// ========================================================================

#[tool(
    description = "Get STEM system version, trait counts, domain summary, and T1 distribution. 32 traits across 4 domains (Science, Chemistry, Physics, Mathematics)."
)]
pub async fn stem_version(&self) -> Result<CallToolResult, McpError> {
    tools::stem::version()
}

#[tool(
    description = "Get the full 32-trait STEM taxonomy with T1 groundings. Shows every trait's domain, T1 primitive (SEQUENCE/MAPPING/RECURSION/STATE), and cross-domain transfer description."
)]
pub async fn stem_taxonomy(&self) -> Result<CallToolResult, McpError> {
    tools::stem::taxonomy()
}

#[tool(
    description = "Combine two confidence values via multiplicative composition. Confidence (T2-P) is the universal uncertainty quantifier shared across all STEM domains. Result clamped to [0.0, 1.0]."
)]
pub async fn stem_confidence_combine(
    &self,
    Parameters(params): Parameters<params::stem::StemConfidenceCombineParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::confidence_combine(params)
}

#[tool(
    description = "Get tier classification info and transfer multiplier. Tiers: T1 (1.0), T2-P (0.9), T2-C (0.7), T3 (0.4). Higher tier = more transferable across domains."
)]
pub async fn stem_tier_info(
    &self,
    Parameters(params): Parameters<params::stem::StemTierInfoParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::tier_info(params)
}

#[tool(
    description = "Calculate chemistry equilibrium balance from forward/reverse rates. Returns K (equilibrium constant), whether system is at equilibrium, and whether products are favored. Grounds to STATE (Harmonize trait)."
)]
pub async fn stem_chem_balance(
    &self,
    Parameters(params): Parameters<params::stem::StemChemBalanceParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::chem_balance(params)
}

#[tool(
    description = "Create a Fraction [0.0, 1.0] and check saturation status. Saturated when ≥ 0.99. Grounds to STATE (Saturate trait). Cross-domain: capacity utilization."
)]
pub async fn stem_chem_fraction(
    &self,
    Parameters(params): Parameters<params::stem::StemChemFractionParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::chem_fraction(params)
}

#[tool(
    description = "Calculate acceleration from force and mass (Newton's F=ma → a=F/m). Grounds to MAPPING (YieldForce trait). Cross-domain: dose-response relationship."
)]
pub async fn stem_phys_fma(
    &self,
    Parameters(params): Parameters<params::stem::StemPhysFmaParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::phys_fma(params)
}

#[tool(
    description = "Check quantity conservation (before vs after within tolerance). Grounds to STATE (Preserve trait). Cross-domain: case count conservation in reporting pipelines."
)]
pub async fn stem_phys_conservation(
    &self,
    Parameters(params): Parameters<params::stem::StemPhysConservationParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::phys_conservation(params)
}

#[tool(
    description = "Convert frequency to period (T = 1/f). Grounds to RECURSION (Harmonics trait). Cross-domain: seasonal reporting pattern cycle detection."
)]
pub async fn stem_phys_period(
    &self,
    Parameters(params): Parameters<params::stem::StemPhysPeriodParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::phys_period(params)
}

#[tool(
    description = "Check if a value is within optional lower/upper bounds and compute clamped value. Grounds to STATE (Bound trait). Cross-domain: confidence interval boundary checking."
)]
pub async fn stem_math_bounds_check(
    &self,
    Parameters(params): Parameters<params::stem::StemMathBoundsCheckParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::math_bounds_check(params)
}

#[tool(
    description = "Invert a mathematical relation (LessThan↔GreaterThan, Equal↔Equal, Incomparable↔Incomparable). Checks symmetry. Grounds to MAPPING (Symmetric trait)."
)]
pub async fn stem_math_relation_invert(
    &self,
    Parameters(params): Parameters<params::stem::StemMathRelationInvertParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::math_relation_invert(params)
}

// ========================================================================
// STEM Chemistry Tools (3) - Extended
// ========================================================================

#[tool(
    description = "Create a concentration ratio and optionally compare two ratios with fold-change. Grounds to MAPPING + QUANTITY (Concentrate trait). Cross-domain: dose-per-volume."
)]
pub async fn stem_chem_ratio(
    &self,
    Parameters(params): Parameters<params::stem::StemChemRatioParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::chem_ratio(params)
}

#[tool(
    description = "Create a rate value and optionally compare two rates with ratio. Grounds to MAPPING + QUANTITY (Energize trait). Cross-domain: reaction rate, signal velocity."
)]
pub async fn stem_chem_rate(
    &self,
    Parameters(params): Parameters<params::stem::StemChemRateParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::chem_rate(params)
}

#[tool(
    description = "Create a binding affinity value and classify strength (weak/moderate/strong). Grounds to MAPPING (Interact trait). Cross-domain: drug-target binding."
)]
pub async fn stem_chem_affinity(
    &self,
    Parameters(params): Parameters<params::stem::StemChemAffinityParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::chem_affinity(params)
}

// ========================================================================
// STEM Physics Tools (3) - Extended
// ========================================================================

#[tool(
    description = "Create an amplitude and optionally superpose with another (a + b). Grounds to QUANTITY (Harmonics trait). Cross-domain: signal strength, effect magnitude."
)]
pub async fn stem_phys_amplitude(
    &self,
    Parameters(params): Parameters<params::stem::StemPhysAmplitudeParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::phys_amplitude(params)
}

#[tool(
    description = "Apply a scale factor to a value (output = factor × input). Grounds to MAPPING (Scale trait). Cross-domain: dose scaling, proportional adjustment."
)]
pub async fn stem_phys_scale(
    &self,
    Parameters(params): Parameters<params::stem::StemPhysScaleParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::phys_scale(params)
}

#[tool(
    description = "Calculate resistance force from inertial mass and proposed change magnitude. Grounds to PERSISTENCE (Inertia trait). Cross-domain: organizational resistance."
)]
pub async fn stem_phys_inertia(
    &self,
    Parameters(params): Parameters<params::stem::StemPhysInertiaParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::phys_inertia(params)
}

// ========================================================================
// STEM Mathematics Tools (2) - Extended
// ========================================================================

#[tool(
    description = "Construct a logical proof from premises and conclusion. Mark as valid or invalid (counterexample). Grounds to SEQUENCE + EXISTENCE (Prove trait)."
)]
pub async fn stem_math_proof(
    &self,
    Parameters(params): Parameters<params::stem::StemMathProofParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::math_proof(params)
}

#[tool(
    description = "Check if a value is the identity element for an operation (add→0, multiply→1). Grounds to STATE (Identify trait). Cross-domain: neutral element, no-op."
)]
pub async fn stem_math_identity(
    &self,
    Parameters(params): Parameters<params::stem::StemMathIdentityParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::math_identity(params)
}

// ========================================================================
// STEM Spatial Tools (5)
// ========================================================================

#[tool(
    description = "Create a distance value and optionally compare for approximate equality within tolerance. Cross-domain: edit distance, concept similarity, drift magnitude."
)]
pub async fn stem_spatial_distance(
    &self,
    Parameters(params): Parameters<params::stem::StemSpatialDistanceParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::spatial_distance(params)
}

#[tool(
    description = "Check the triangle inequality: d(a,c) ≤ d(a,b) + d(b,c). Returns slack and degenerate status. Cross-domain: metric consistency, embedding validity."
)]
pub async fn stem_spatial_triangle(
    &self,
    Parameters(params): Parameters<params::stem::StemSpatialTriangleParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::spatial_triangle(params)
}

#[tool(
    description = "Check if a point (at test_distance) is contained within a neighborhood of given radius. Open or closed boundary. Cross-domain: fuzzy match radius."
)]
pub async fn stem_spatial_neighborhood(
    &self,
    Parameters(params): Parameters<params::stem::StemSpatialNeighborhoodParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::spatial_neighborhood(params)
}

#[tool(
    description = "Dimension rank and optional subspace check. Returns codimension if subspace_of is provided. Cross-domain: feature dimensionality."
)]
pub async fn stem_spatial_dimension(
    &self,
    Parameters(params): Parameters<params::stem::StemSpatialDimensionParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::spatial_dimension(params)
}

#[tool(
    description = "Orientation composition (positive × negative = negative, etc.). Cross-domain: signal direction, trend polarity."
)]
pub async fn stem_spatial_orientation(
    &self,
    Parameters(params): Parameters<params::stem::StemSpatialOrientationParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::spatial_orientation(params)
}

// ========================================================================
// STEM Core Tools (4) - Transfer, Integrity, Retry, Determinism
// ========================================================================

#[tool(
    description = "Compute cross-domain transfer confidence: structural×0.4 + functional×0.4 + contextual×0.2. Returns combined score and limiting factor. Use for any cross-domain mapping."
)]
pub async fn stem_transfer_confidence(
    &self,
    Parameters(params): Parameters<params::stem::StemTransferConfidenceParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::transfer_confidence(params)
}

#[tool(
    description = "Integrity gate: validate a value against min/max bounds. Returns pass/fail, clamped value, and violation details. Pattern: parse-don't-validate at boundaries."
)]
pub async fn stem_integrity_check(
    &self,
    Parameters(params): Parameters<params::stem::StemIntegrityCheckParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::integrity_check(params)
}

#[tool(
    description = "Calculate retry budget from max_attempts and current_attempt. Returns attempts remaining, budget fraction, and recommendation (retry/alternative/escalate)."
)]
pub async fn stem_retry_budget(
    &self,
    Parameters(params): Parameters<params::stem::StemRetryBudgetParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::retry_budget(params)
}

#[tool(
    description = "Classify a determinism score [0,1]. Deterministic (≥0.95) = pure function. Stochastic (<0.05) = random. Cross-domain: pipeline repeatability, test flakiness."
)]
pub async fn stem_determinism_score(
    &self,
    Parameters(params): Parameters<params::stem::StemDeterminismScoreParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::determinism_score(params)
}

// ========================================================================
// STEM Bio Tools (2) - Behavior and Tone Profiles
// ========================================================================

#[tool(
    description = "Load endocrine state and codify to BehaviorModulation profile (risk_tolerance, validation_depth, exploration_rate, warmth, urgency). Optional stimulus: stress/reward/social/urgency."
)]
pub async fn stem_bio_behavior_profile(
    &self,
    Parameters(params): Parameters<params::stem::StemBioBehaviorProfileParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::bio_behavior_profile(params)
}

#[tool(
    description = "Load endocrine state, codify behavior, and extend to ToneProfile (directness, hedging, warmth, precision, verbosity). AI output style calibration from biological state."
)]
pub async fn stem_bio_tone_profile(
    &self,
    Parameters(params): Parameters<params::stem::StemBioToneProfileParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::bio_tone_profile(params)
}

// ========================================================================
// STEM Finance Tools (8) - Cross-Domain Transfer from Finance
// ========================================================================

#[tool(
    description = "Compute present value via discounting: PV = FV / (1+r)^n. Cross-domain: signal decay over time, data depreciation, cache staleness."
)]
pub async fn stem_finance_discount(
    &self,
    Parameters(params): Parameters<params::stem::StemFinanceDiscountParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::finance_discount(params)
}

#[tool(
    description = "Compute compounded growth (discrete or continuous). FV = P×(1+r)^n or P×e^(rt). Cross-domain: tech debt compounding, signal accumulation."
)]
pub async fn stem_finance_compound(
    &self,
    Parameters(params): Parameters<params::stem::StemFinanceCompoundParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::finance_compound(params)
}

#[tool(
    description = "Compute bid-ask spread and percentage of mid price. Cross-domain: source disagreement, signal discrepancy, calibration offset."
)]
pub async fn stem_finance_spread(
    &self,
    Parameters(params): Parameters<params::stem::StemFinanceSpreadParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::finance_spread(params)
}

#[tool(
    description = "Check maturity/expiry status from years to maturity. Returns expired/near_maturity/active. Cross-domain: regulatory deadline, token expiry, lease TTL."
)]
pub async fn stem_finance_maturity(
    &self,
    Parameters(params): Parameters<params::stem::StemFinanceMaturityParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::finance_maturity(params)
}

#[tool(
    description = "Compute exposure from position value. Returns long/short/flat status and absolute exposure. Cross-domain: total signal weight, cumulative risk."
)]
pub async fn stem_finance_exposure(
    &self,
    Parameters(params): Parameters<params::stem::StemFinanceExposureParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::finance_exposure(params)
}

#[tool(
    description = "Detect arbitrage opportunity: is spread > transaction cost? Returns exploitability and net profit. Cross-domain: cross-source signal discrepancy, A/B test divergence."
)]
pub async fn stem_finance_arbitrage(
    &self,
    Parameters(params): Parameters<params::stem::StemFinanceArbitrageParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::finance_arbitrage(params)
}

#[tool(
    description = "Aggregate positions and compute diversification benefit (1 - portfolio_risk/sum_of_risks). Cross-domain: multi-source confirmation, redundancy benefit."
)]
pub async fn stem_finance_diversify(
    &self,
    Parameters(params): Parameters<params::stem::StemFinanceDiversifyParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::finance_diversify(params)
}

#[tool(
    description = "Calculate return (simple or log) from P₀ and P₁. Simple: (P₁-P₀)/P₀, Log: ln(P₁/P₀). Cross-domain: ROI, signal improvement ratio."
)]
pub async fn stem_finance_return(
    &self,
    Parameters(params): Parameters<params::stem::StemFinanceReturnParams>
) -> Result<CallToolResult, McpError> {
    tools::stem::finance_return(params)
}

// ========================================================================
// Game Theory Tools (5)
// ========================================================================

#[tool(description = "Analyze a 2x2 normal-form game and return pure/mixed Nash equilibria.")]
pub async fn game_theory_nash_2x2(
    &self,
    Parameters(params): Parameters<params::game_theory::GameTheoryNash2x2Params>
) -> Result<CallToolResult, McpError> {
    tools::game_theory::nash_2x2(params)
}

#[tool(
    description = "Build and analyze an N×M payoff matrix. Returns best responses, dominant strategies, minimax/maximin values, and expected payoffs per row."
)]
pub async fn forge_payoff_matrix(
    &self,
    Parameters(params): Parameters<params::forge::ForgePayoffMatrixParams>
) -> Result<CallToolResult, McpError> {
    tools::game_theory::forge_payoff_matrix(params)
}

#[tool(
    description = "Solve N×M mixed strategy Nash equilibrium via fictitious play. Returns mixed strategy weights, expected payoff, and convergence info."
)]
pub async fn forge_nash_solve(
    &self,
    Parameters(params): Parameters<params::forge::ForgeNashSolveParams>
) -> Result<CallToolResult, McpError> {
    tools::game_theory::forge_nash_solve(params)
}

#[tool(
    description = "Compute forge quality score Q = 0.40×prim + 0.25×combat + 0.20×turn + 0.15×survival. Returns total, component breakdown, letter grade, and code completeness."
)]
pub async fn forge_quality_score(
    &self,
    Parameters(params): Parameters<params::forge::ForgeQualityScoreParams>
) -> Result<CallToolResult, McpError> {
    tools::game_theory::forge_quality_score(params)
}

#[tool(
    description = "Generate Rust code from collected Lex Primitiva (0-15) and defeated safety enemies. Returns complete forge_output.rs with GroundsTo-annotated types."
)]
pub async fn forge_code_generate(
    &self,
    Parameters(params): Parameters<params::forge::ForgeCodeGenerateParams>
) -> Result<CallToolResult, McpError> {
    tools::game_theory::forge_code_generate(params)
}

// ========================================================================
// Visualization Tools (6)
// ========================================================================

#[tool(
    description = "Generate STEM taxonomy sunburst SVG showing all 32 traits across 4 domains (Science, Chemistry, Physics, Mathematics), color-coded by T1 grounding."
)]
pub async fn viz_stem_taxonomy(
    &self,
    Parameters(params): Parameters<params::viz::VizTaxonomyParams>
) -> Result<CallToolResult, McpError> {
    tools::viz::taxonomy(params)
}

#[tool(
    description = "Generate type composition SVG showing how a type decomposes to T1 Lex Primitiva."
)]
pub async fn viz_type_composition(
    &self,
    Parameters(params): Parameters<params::viz::VizCompositionParams>
) -> Result<CallToolResult, McpError> {
    tools::viz::composition(params)
}

#[tool(description = "Generate circular flow SVG for STEM method loops.")]
pub async fn viz_method_loop(
    &self,
    Parameters(params): Parameters<params::viz::VizLoopParams>
) -> Result<CallToolResult, McpError> {
    tools::viz::method_loop(params)
}

#[tool(description = "Generate confidence propagation waterfall SVG.")]
pub async fn viz_confidence_chain(
    &self,
    Parameters(params): Parameters<params::viz::VizConfidenceParams>
) -> Result<CallToolResult, McpError> {
    tools::viz::confidence(params)
}

#[tool(description = "Generate bounds visualization SVG.")]
pub async fn viz_bounds(
    &self,
    Parameters(params): Parameters<params::viz::VizBoundsParams>
) -> Result<CallToolResult, McpError> {
    tools::viz::bounds(params)
}

#[tool(description = "Generate DAG topology SVG with parallel execution levels.")]
pub async fn viz_dag(
    &self,
    Parameters(params): Parameters<params::viz::VizDagParams>
) -> Result<CallToolResult, McpError> {
    tools::viz::dag(params)
}

// ═══════════════════════════════════════════════════════════════════════════
// PRIMITIVE SCANNER
// ═══════════════════════════════════════════════════════════════════════════

#[tool(
    description = "Scan sources for primitives in a domain. Returns T1/T2/T3 tier classification."
)]
pub async fn primitive_scan(
    &self,
    Parameters(params): Parameters<params::primitive_scanner::PrimitiveScanParams>
) -> Result<CallToolResult, McpError> {
    tools::primitive_scanner::primitive_scan(params)
}

#[tool(
    description = "Batch test terms for primitiveness using 3-test protocol. Returns verdict and tier for each term."
)]
pub async fn primitive_batch_test(
    &self,
    Parameters(params): Parameters<params::primitive_scanner::PrimitiveBatchTestParams>
) -> Result<CallToolResult, McpError> {
    tools::primitive_scanner::primitive_batch_test(params)
}

// ========================================================================
// Integrity Assessment (3 tools)
// ========================================================================

#[tool(
    description = "Analyze text for AI-generation indicators using 5 statistical features (Zipf, entropy, burstiness, perplexity, TTR) aggregated via chemistry primitives (Beer-Lambert + Hill + Arrhenius)."
)]
pub async fn integrity_analyze(
    &self,
    Parameters(params): Parameters<params::integrity::IntegrityAnalyzeParams>
) -> Result<CallToolResult, McpError> {
    tools::integrity::integrity_analyze(params)
}

#[tool(
    description = "Assess KSB response integrity. Convenience endpoint requiring Bloom level (1-7)."
)]
pub async fn integrity_assess_ksb(
    &self,
    Parameters(params): Parameters<params::integrity::IntegrityAssessKsbParams>
) -> Result<CallToolResult, McpError> {
    tools::integrity::integrity_assess_ksb(params)
}

#[tool(
    description = "Get domain calibration profile with baseline feature expectations for PV domains (D02-D12)."
)]
pub async fn integrity_calibration(
    &self,
    Parameters(params): Parameters<params::integrity::IntegrityCalibrationParams>
) -> Result<CallToolResult, McpError> {
    tools::integrity::integrity_calibration(params)
}

// ========================================================================
// Decision Tree Engine (6 tools)
// ========================================================================

#[tool(description = "Train a CART decision tree on feature matrix and labels. Returns tree_id.")]
pub async fn dtree_train(
    &self,
    Parameters(params): Parameters<params::dtree::DtreeTrainParams>
) -> Result<CallToolResult, McpError> {
    tools::dtree::dtree_train(params)
}

#[tool(
    description = "Predict class label (or regression value) for a single sample using a trained tree."
)]
pub async fn dtree_predict(
    &self,
    Parameters(params): Parameters<params::dtree::DtreePredictParams>
) -> Result<CallToolResult, McpError> {
    tools::dtree::dtree_predict(params)
}

#[tool(description = "Get feature importance scores from a trained tree.")]
pub async fn dtree_importance(
    &self,
    Parameters(params): Parameters<params::dtree::DtreeImportanceParams>
) -> Result<CallToolResult, McpError> {
    tools::dtree::dtree_importance(params)
}

#[tool(description = "Prune a trained tree using cost-complexity pruning (CCP).")]
pub async fn dtree_prune(
    &self,
    Parameters(params): Parameters<params::dtree::DtreePruneParams>
) -> Result<CallToolResult, McpError> {
    tools::dtree::dtree_prune(params)
}

#[tool(description = "Export a trained tree in specified format.")]
pub async fn dtree_export(
    &self,
    Parameters(params): Parameters<params::dtree::DtreeExportParams>
) -> Result<CallToolResult, McpError> {
    tools::dtree::dtree_export(params)
}

#[tool(description = "Get metadata and statistics for a trained tree.")]
pub async fn dtree_info(
    &self,
    Parameters(params): Parameters<params::dtree::DtreeInfoParams>
) -> Result<CallToolResult, McpError> {
    tools::dtree::dtree_info(params)
}

// ========================================================================
// Disney Loop Tools (4) - Forward-only compound discovery pipeline
// Pipeline: ρ(t) → ∂(¬σ⁻¹) → ∃(ν) → ρ(t+1)
// ========================================================================

#[tool(
    description = "Run the full Disney Loop pipeline: state assessment → anti-regression gate → curiosity search → new state. Provide records with domain, direction (forward/backward), novelty_score, and discovery fields. Returns pipeline stages, aggregated output by domain, and T1 primitive grounding."
)]
pub async fn disney_loop_run(
    &self,
    Parameters(params): Parameters<params::disney_loop::DisneyLoopRunParams>
) -> Result<CallToolResult, McpError> {
    tools::disney_loop::run(params)
}

#[tool(
    description = "Disney Loop Stage 2: ∂(¬σ⁻¹) — Anti-Regression Gate. Filters out backward-directed records, keeping only forward momentum. Returns rejected count and surviving records."
)]
pub async fn disney_loop_anti_regression(
    &self,
    Parameters(params): Parameters<params::disney_loop::DisneyAntiRegressionParams>
) -> Result<CallToolResult, McpError> {
    tools::disney_loop::anti_regression(params)
}

#[tool(
    description = "Disney Loop Stage 3: ∃(ν) — Curiosity Search. Aggregates forward records by domain, summing novelty scores and counting discoveries. Use after anti-regression filtering."
)]
pub async fn disney_loop_curiosity_search(
    &self,
    Parameters(params): Parameters<params::disney_loop::DisneyCuriositySearchParams>
) -> Result<CallToolResult, McpError> {
    tools::disney_loop::curiosity_search(params)
}

#[tool(
    description = "Disney Loop Stage 1: ρ(t) — State Assessment. Analyze records without transforming them. Returns forward/backward ratio, health status, domain distribution, novelty statistics, and discovery count."
)]
pub async fn disney_loop_state_assess(
    &self,
    Parameters(params): Parameters<params::disney_loop::DisneyStateAssessParams>
) -> Result<CallToolResult, McpError> {
    tools::disney_loop::state_assess(params)
}

// ========================================================================
// Knowledge Engine Tools (5) - Compression, Compilation, Query
// ========================================================================

#[tool(
    description = "Ingest text as a knowledge fragment. Extracts concepts, primitives, classifies domain, and scores compendious density. Returns fragment_id, domain, concepts, primitives, and compendious_score."
)]
pub async fn knowledge_ingest(
    &self,
    Parameters(params): Parameters<params::knowledge_engine::KnowledgeIngestParams>
) -> Result<CallToolResult, McpError> {
    tools::knowledge_engine::ingest(params)
}

#[tool(
    description = "Compress text structurally using pattern replacement (45 verbose patterns), dedup (token Jaccard >0.8), and hierarchy flattening. Returns original and compressed scores, text, and compression ratio."
)]
pub async fn knowledge_compress(
    &self,
    Parameters(params): Parameters<params::knowledge_engine::KnowledgeCompressParams>
) -> Result<CallToolResult, McpError> {
    tools::knowledge_engine::compress(params)
}

#[tool(
    description = "Compile knowledge from Brain distillations, artifacts, implicit knowledge, and/or free text into an immutable versioned knowledge pack. Persists to ~/.claude/brain/knowledge_packs/. Returns pack stats."
)]
pub async fn knowledge_compile(
    &self,
    Parameters(params): Parameters<params::knowledge_engine::KnowledgeCompileParams>
) -> Result<CallToolResult, McpError> {
    tools::knowledge_engine::compile(params)
}

#[tool(
    description = "Query compiled knowledge packs. Modes: keyword (token similarity), concept (concept-term matching), domain (domain filter). Returns ranked results with content, concepts, domain, and relevance."
)]
pub async fn knowledge_query(
    &self,
    Parameters(params): Parameters<params::knowledge_engine::KnowledgeQueryParams>
) -> Result<CallToolResult, McpError> {
    tools::knowledge_engine::query(params)
}

#[tool(
    description = "Get knowledge engine statistics. Returns pack inventory with fragment counts, concept counts, average compendious scores, and totals across all packs."
)]
pub async fn knowledge_stats(
    &self,
    Parameters(params): Parameters<params::knowledge_engine::KnowledgeStatsParams>
) -> Result<CallToolResult, McpError> {
    tools::knowledge_engine::stats(params)
}
