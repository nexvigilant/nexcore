// ========================================================================
// Guardian Tools (17) - Homeostasis Control Loop + Actuator Inspection
// ========================================================================

#[tool(
    description = "Run one iteration of the Guardian homeostasis control loop. Collects signals from sensors, evaluates via decision engine, and executes responses through actuators."
)]
pub async fn guardian_homeostasis_tick(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianTickParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::homeostasis_tick(params).await
}

#[tool(
    description = "Evaluate PV risk context and get recommended responses. Takes signal metrics (PRR, ROR, IC, EBGM) and returns risk score with suggested actions."
)]
pub async fn guardian_evaluate_pv(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianEvaluatePvParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::evaluate_pv(params)
}

#[tool(
    description = "Get Guardian homeostasis loop status. Returns iteration count, registered sensors, and actuators."
)]
pub async fn guardian_status(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianStatusParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::status(params).await
}

#[tool(
    description = "Reset the Guardian homeostasis loop state. Clears iteration count and resets decision engine amplification."
)]
pub async fn guardian_reset(&self) -> Result<CallToolResult, McpError> {
    tools::guardian::reset().await
}

#[tool(
    description = "Inject a test signal into Guardian for simulation/testing. Creates synthetic PAMP/DAMP/PV signal processed on next tick."
)]
pub async fn guardian_inject_signal(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianInjectSignalParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::inject_signal(params).await
}

#[tool(
    description = "List all registered Guardian sensors (PAMPs, DAMPs, PV) with their status and detector capabilities."
)]
pub async fn guardian_sensors_list(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianSensorsListParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::sensors_list(params).await
}

#[tool(
    description = "List all registered Guardian actuators (Alert, AuditLog, Block, RateLimit, Escalation) with status and priority."
)]
pub async fn guardian_actuators_list(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianActuatorsListParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::actuators_list(params).await
}

#[tool(
    description = "Get Guardian event history. Returns recent signals and actions for monitoring and debugging."
)]
pub async fn guardian_history(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianHistoryParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::history(params)
}

#[tool(
    description = "Classify an entity by {G,V,R} autonomy capabilities. G=Goal-selection, V=Value-evaluation, R=Refusal-capacity. Entities with \u00acG\u2227\u00acV\u2227\u00acR have symmetric harm capability. Returns originator type, ceiling multiplier, and interpretation."
)]
pub async fn guardian_originator_classify(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianOriginatorClassifyParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::originator_classify(params)
}

#[tool(
    description = "Get autonomy-aware ceiling limits for an originator type. Higher autonomy \u2192 lower limits (entity can self-regulate). Types: tool (1.0x), agent_with_r (0.8x), agent_with_vr (0.5x), agent_with_gr (0.6x), agent_with_gvr (0.2x)."
)]
pub async fn guardian_ceiling_for_originator(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianCeilingForOriginatorParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::ceiling_for_originator(params)
}

#[tool(
    description = "Compute 3D safety space point for visualization. Maps PV metrics + ToV + GVR to: X=Severity (harm magnitude), Y=Likelihood (boundary crossing probability), Z=Detectability (early detection capability). Returns RPN=S\u00d7L\u00d7(1-D) and zone (Green/Yellow/Orange/Red)."
)]
pub async fn guardian_space3d_compute(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianSpace3DComputeParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::space3d_compute(params)
}

#[tool(
    description = "Run one PV control loop iteration (Aerospace\u2192PV domain translation). Executes SENSE\u2192COMPARE\u2192CONTROL\u2192ACTUATE\u2192FEEDBACK cycle. Returns safety state (PRR/ROR/IC/EBGM), signal strength, and recommended action (ContinueMonitoring\u2192EmergencyWithdrawal). Transfer confidence: 0.92."
)]
pub async fn pv_control_loop_tick(
    &self,
    Parameters(params): Parameters<params::guardian::PvControlLoopTickParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::pv_control_loop_tick(params)
}

// ========================================================================
// Guardian Actuator Inspection Tools (6)
// ========================================================================

#[tool(
    description = "List all currently blocked targets in the Guardian BlockActuator. Returns target identifiers that have been blocked by the decision engine."
)]
pub async fn guardian_blocked_list(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianBlockedListParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::blocked_list(params)
}

#[tool(
    description = "Unblock a target that was blocked by the Guardian BlockActuator. Returns whether the target was actually blocked."
)]
pub async fn guardian_unblock(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianUnblockParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::unblock(params)
}

#[tool(
    description = "List all open escalations from the Guardian EscalationActuator. Returns escalation records with level (L1-L4), description, timestamp, and status."
)]
pub async fn guardian_escalations(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianEscalationsParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::escalations(params)
}

#[tool(
    description = "Acknowledge an open escalation by ID. Changes escalation status from Open to Acknowledged."
)]
pub async fn guardian_acknowledge_escalation(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianAcknowledgeEscalationParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::acknowledge_escalation(params)
}

#[tool(
    description = "List all quarantined items from the Guardian QuarantineActuator. Returns targets with their quarantine type."
)]
pub async fn guardian_quarantined_list(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianQuarantinedListParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::quarantined_list(params)
}

#[tool(
    description = "Adjust the Guardian decision engine threshold (0.0-100.0). Lower values make the engine more sensitive, triggering actions on weaker signals. Default is 50.0."
)]
pub async fn guardian_set_threshold(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianSetThresholdParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::set_threshold(params).await
}

// ========================================================================
// Guardian Governance Tools (5) - Phase 4: Head of State
// ========================================================================

#[tool(
    description = "Grant governance consent for a scope. Creates a consent record (Pending→Granted→Active) that authorizes Guardian to operate within the specified scope. Scopes: patient-safety, system-health, hud-governance, access-control, data-integrity, global."
)]
pub async fn guardian_consent_grant(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianConsentGrantParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::consent_grant(params).await
}

#[tool(
    description = "Revoke governance consent by ID. Authority dissolves immediately upon revocation. Records reason in audit trail."
)]
pub async fn guardian_consent_revoke(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianConsentRevokeParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::consent_revoke(params).await
}

#[tool(
    description = "View active governance consents, delegations, and legitimacy metrics. Returns consent count, delegation count, checks performed, failures detected, and legitimacy rate."
)]
pub async fn guardian_consent_status(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianConsentStatusParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::consent_status(params).await
}

#[tool(
    description = "Create an authority delegation chain. Delegates authority from user to an entity within a governance scope. Root delegations have depth=0. Max chain depth: 8 levels."
)]
pub async fn guardian_delegation_create(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianDelegationCreateParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::delegation_create(params).await
}

#[tool(
    description = "Run full legitimacy check for a proposed action. Validates: (1) consent exists and is active, (2) authority delegation valid, (3) scope containment, (4) evidence sufficient, (5) originator capability. Returns verdict: Legitimate, Illegitimate, or P0Override."
)]
pub async fn guardian_legitimacy_check(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianLegitimacyCheckParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::legitimacy_check(params).await
}

// ========================================================================
// Guardian Forensics Tools (3) - Phase 5: Journal & Audit
// ========================================================================

#[tool(
    description = "Query governance journal by scope, actor, or time. Returns evidenced actions with legitimacy verdicts. Every Guardian action is recorded with full evidence trail."
)]
pub async fn guardian_journal_query(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianJournalQueryParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::journal_query(params).await
}

#[tool(
    description = "Get governance journal statistics. Returns: total entries, unchecked count, illegitimate count, governance coverage rate, and legitimacy rate."
)]
pub async fn guardian_journal_stats(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianJournalStatsParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::journal_stats(params).await
}

#[tool(
    description = "Full governance audit: expired consents, delegation depth, tyranny detection, journal gaps. Returns audit summary with governance health indicators."
)]
pub async fn guardian_governance_audit(
    &self,
    Parameters(params): Parameters<params::guardian::GuardianGovernanceAuditParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::governance_audit(params).await
}

// ========================================================================
// FDA Data Bridge Tools (2)
// ========================================================================

#[tool(
    description = "Evaluate drug-event pair through FDA Data Bridge. Connects FAERS contingency data (a,b,c,d) to PV Control Loop. Returns signal detection (PRR/ROR/IC/EBGM), severity (None\u2192Critical), and recommended action (ContinueMonitoring\u2192EmergencyWithdrawal)."
)]
pub async fn fda_bridge_evaluate(
    &self,
    Parameters(params): Parameters<params::guardian::FdaBridgeEvaluateParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::fda_bridge_evaluate(params)
}

#[tool(
    description = "Batch evaluate multiple drug-event pairs through FDA Data Bridge. Processes multiple contingency tables [[a,b,c,d],...] efficiently. Returns signals detected count, max priority action, and per-pair results."
)]
pub async fn fda_bridge_batch(
    &self,
    Parameters(params): Parameters<params::guardian::FdaBridgeBatchParams>
) -> Result<CallToolResult, McpError> {
    tools::guardian::fda_bridge_batch(params)
}

// ========================================================================
// HUD Capability Tools (6) - CAP-025, CAP-026, CAP-027
// ========================================================================

#[tool(
    description = "Allocate agent for task (CAP-025 Small Business Act). Analyzes task complexity, recommends model tier (Haiku/Sonnet/Opus), identifies matching skills, and allocates compute quota. Returns primary agent, alternatives, and confidence."
)]
pub async fn sba_allocate_agent(
    &self,
    Parameters(params): Parameters<params::hud::SbaAllocateAgentParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::sba_allocate_agent(params)
}

#[tool(
    description = "Get next step in agent chain (CAP-025). After completing a step, returns the next agent to trigger based on chain configuration (Always/WithErrors/Stable conditions)."
)]
pub async fn sba_chain_next(
    &self,
    Parameters(params): Parameters<params::hud::SbaChainNextParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::sba_chain_next(params)
}

#[tool(
    description = "Persist state with SHA-256 integrity (CAP-026 Social Security Act). Stores state with cryptographic hash for verification. Returns version number, hash, and persistence level (Session/Local/Distributed/Resolved)."
)]
pub async fn ssa_persist_state(
    &self,
    Parameters(params): Parameters<params::hud::SsaPersistStateParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::ssa_persist_state(params)
}

#[tool(
    description = "Verify state integrity (CAP-026). Compares expected SHA-256 hash against computed hash to detect tampering or corruption. Returns match status and recommendation."
)]
pub async fn ssa_verify_integrity(
    &self,
    Parameters(params): Parameters<params::hud::SsaVerifyIntegrityParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::ssa_verify_integrity(params)
}

#[tool(
    description = "Get token budget report (CAP-027 Federal Reserve Act). Returns daily/session usage, remaining budget, stability level (Stable/Cautious/Restricted/Emergency), and estimated cost."
)]
pub async fn fed_budget_report(
    &self,
    Parameters(params): Parameters<params::hud::FedBudgetReportParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::fed_budget_report(params)
}

#[tool(
    description = "Recommend model tier (CAP-027). Based on task complexity, budget utilization, and accuracy requirements, recommends Economy/Standard/Premium tier with cost comparison."
)]
pub async fn fed_recommend_model(
    &self,
    Parameters(params): Parameters<params::hud::FedRecommendModelParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::fed_recommend_model(params)
}

#[tool(
    description = "Audit market compliance (CAP-028 Securities Act). Analyzes trade volume against liquidity thresholds, detects concentration risk, and returns compliance verdicts."
)]
pub async fn sec_audit_market(
    &self,
    Parameters(params): Parameters<params::hud::SecAuditMarketParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::sec_audit_market(params)
}

#[tool(
    description = "Recommend communication protocol (CAP-029 Communications Act). Selects optimal protocol based on delivery guarantee, latency, and broadcast requirements."
)]
pub async fn comm_recommend_protocol(
    &self,
    Parameters(params): Parameters<params::hud::CommRecommendProtocolParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::comm_recommend_protocol(params)
}

#[tool(
    description = "Route message (CAP-029). Dispatches message from source to destination with optional protocol selection, TTL, and routing confirmation."
)]
pub async fn comm_route_message(
    &self,
    Parameters(params): Parameters<params::hud::CommRouteMessageParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::comm_route_message(params)
}

#[tool(
    description = "Launch exploration mission (CAP-030 Exploration Act). Creates mission manifest with target, objective, scope (Wide/Focused/Deep), and discovery patterns."
)]
pub async fn explore_launch_mission(
    &self,
    Parameters(params): Parameters<params::hud::ExploreLaunchMissionParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::explore_launch_mission(params)
}

#[tool(
    description = "Record discovery (CAP-030). Logs finding with location and significance score, adds to discovery index."
)]
pub async fn explore_record_discovery(
    &self,
    Parameters(params): Parameters<params::hud::ExploreRecordDiscoveryParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::explore_record_discovery(params)
}

#[tool(
    description = "Get exploration frontier (CAP-030). Returns current frontier bounds (min/max explored), unexplored regions, and discovery count."
)]
pub async fn explore_get_frontier(
    &self,
    Parameters(params): Parameters<params::hud::ExploreGetFrontierParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::explore_get_frontier(params)
}

#[tool(
    description = "Validate signal efficacy (CAP-014 Public Health Act). FDA-style validation with accuracy, FPR, and community value."
)]
pub async fn health_validate_signal(
    &self,
    Parameters(params): Parameters<params::hud::HealthValidateSignalParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::health_validate_signal(params)
}

#[tool(
    description = "Measure public health impact (CAP-014). Returns efficacy score based on valid/total signals."
)]
pub async fn health_measure_impact(
    &self,
    Parameters(params): Parameters<params::hud::HealthMeasureImpactParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::health_measure_impact(params)
}

#[tool(
    description = "Convert signal asymmetry (CAP-018 Treasury Act). Converts informational edge to token rewards via arbitrage."
)]
pub async fn treasury_convert_asymmetry(
    &self,
    Parameters(params): Parameters<params::hud::TreasuryConvertAsymmetryParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::treasury_convert_asymmetry(params)
}

#[tool(
    description = "Audit treasury status (CAP-018). Checks compute/memory quotas for solvency."
)]
pub async fn treasury_audit(
    &self,
    Parameters(params): Parameters<params::hud::TreasuryAuditParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::treasury_audit(params)
}

#[tool(
    description = "Dispatch transit manifest (CAP-019 Transportation Act). Routes signal batch between domains with priority."
)]
pub async fn dot_dispatch_manifest(
    &self,
    Parameters(params): Parameters<params::hud::DotDispatchManifestParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::dot_dispatch_manifest(params)
}

#[tool(description = "Verify highway safety (CAP-019). NHTSA-style route integrity check.")]
pub async fn dot_verify_highway(
    &self,
    Parameters(params): Parameters<params::hud::DotVerifyHighwayParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::dot_verify_highway(params)
}

#[tool(
    description = "Verify boundary entry (CAP-020 Homeland Security Act). CBP-style authenticity check for incoming data."
)]
pub async fn dhs_verify_boundary(
    &self,
    Parameters(params): Parameters<params::hud::DhsVerifyBoundaryParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::dhs_verify_boundary(params)
}

#[tool(
    description = "Train agent in curriculum (CAP-022 Education Act). Returns mastery level based on completion."
)]
pub async fn edu_train_agent(
    &self,
    Parameters(params): Parameters<params::hud::EduTrainAgentParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::edu_train_agent(params)
}

#[tool(
    description = "Evaluate comprehension (CAP-022). Returns average score and letter grade from score array."
)]
pub async fn edu_evaluate(
    &self,
    Parameters(params): Parameters<params::hud::EduEvaluateParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::edu_evaluate(params)
}

#[tool(
    description = "Fund research project (CAP-031 Science Foundation Act). NSF-style grant for capability enhancement."
)]
pub async fn nsf_fund_research(
    &self,
    Parameters(params): Parameters<params::hud::NsfFundResearchParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::nsf_fund_research(params)
}

#[tool(
    description = "Procure resource (CAP-037 General Services Act). GSA procurement with priority and fulfillment status."
)]
pub async fn gsa_procure(
    &self,
    Parameters(params): Parameters<params::hud::GsaProcureParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::gsa_procure(params)
}

#[tool(
    description = "Audit service value (CAP-037). Cost/benefit analysis with ROI and rating."
)]
pub async fn gsa_audit_value(
    &self,
    Parameters(params): Parameters<params::hud::GsaAuditValueParams>
) -> Result<CallToolResult, McpError> {
    tools::hud::gsa_audit_value(params)
}

// ========================================================================
// Skills Tools (8)
// ========================================================================

#[tool(
    description = "Scan a directory for skills and populate the registry. Returns count of skills found."
)]
pub async fn skill_scan(
    &self,
    Parameters(params): Parameters<params::skills::SkillScanParams>
) -> Result<CallToolResult, McpError> {
    tools::skills::scan(&self.registry, params)
}

#[tool(
    description = "List all registered skills with their names, intents, and compliance levels."
)]
pub async fn skill_list(&self) -> Result<CallToolResult, McpError> {
    tools::skills::list(&self.registry)
}

#[tool(description = "Get detailed information about a specific skill by name.")]
pub async fn skill_get(
    &self,
    Parameters(params): Parameters<params::skills::SkillGetParams>
) -> Result<CallToolResult, McpError> {
    tools::skills::get(&self.registry, params)
}

#[tool(
    description = "Validate a skill for Diamond v2 compliance. Returns level, SMST score, issues, and suggestions."
)]
pub async fn skill_validate(
    &self,
    Parameters(params): Parameters<params::skills::SkillValidateParams>
) -> Result<CallToolResult, McpError> {
    tools::skills::validate(params)
}

#[tool(description = "Search skills by tag. Returns matching skills from the registry.")]
pub async fn skill_search_by_tag(
    &self,
    Parameters(params): Parameters<params::skills::SkillSearchByTagParams>
) -> Result<CallToolResult, McpError> {
    tools::skills::search_by_tag(&self.registry, params)
}

#[tool(
    description = "List nested sub-skills for a compound/parent skill. Discovers skills declared in the parent's nested-skills frontmatter."
)]
pub async fn skill_list_nested(
    &self,
    Parameters(params): Parameters<params::skills::SkillListNestedParams>
) -> Result<CallToolResult, McpError> {
    tools::skills::list_nested(&self.registry, params)
}

#[tool(
    description = "Query taxonomy by type and key. Types: compliance, smst, category, node."
)]
pub async fn skill_taxonomy_query(
    &self,
    Parameters(params): Parameters<params::skills::TaxonomyQueryParams>
) -> Result<CallToolResult, McpError> {
    tools::skills::taxonomy_query(params)
}

#[tool(
    description = "List all entries in a taxonomy. Types: compliance, smst, category, node."
)]
pub async fn skill_taxonomy_list(
    &self,
    Parameters(params): Parameters<params::skills::TaxonomyListParams>
) -> Result<CallToolResult, McpError> {
    tools::skills::taxonomy_list(params)
}

#[tool(
    description = "Get skill categories that are compute-intensive (candidates for Rust/NexCore delegation)."
)]
pub async fn skill_categories_compute_intensive(&self) -> Result<CallToolResult, McpError> {
    tools::skills::categories_compute_intensive()
}

#[tool(
    description = "Search skill knowledge by intent. Returns ranked skills with guidance sections, MCP tools for chaining, and related skills. Pre-populated index of all SKILL.md files. Use instead of reading individual skill files."
)]
pub async fn nexcore_assist(
    &self,
    Parameters(params): Parameters<params::skills::AssistParams>
) -> Result<CallToolResult, McpError> {
    tools::assist::search(&self.assist_index, params)
}

#[tool(
    description = "Analyze skills for orchestration patterns. Detects orchestrators (skills that spawn/delegate work), extracts triggers, and recommends subagent types with rationale. Accepts a path or glob pattern (e.g., '~/.claude/skills/forge' or '~/.claude/skills/*')."
)]
pub async fn skill_orchestration_analyze(
    &self,
    Parameters(params): Parameters<params::skills::SkillOrchestrationAnalyzeParams>
) -> Result<CallToolResult, McpError> {
    tools::skills::orchestration_analyze(params)
}

#[tool(
    description = "Execute a skill by name with parameters. Runs the skill's scripts/binaries via nexcore-skill-exec. Returns output, status, duration, exit code, and stdout/stderr."
)]
pub async fn skill_execute(
    &self,
    Parameters(params): Parameters<params::skills::SkillExecuteParams>
) -> Result<CallToolResult, McpError> {
    tools::skills::execute(params)
}

#[tool(
    description = "Get a skill's input/output schema and execution methods. Returns JSON Schema for inputs/outputs, whether the skill is executable, and available execution methods (shell, binary, library)."
)]
pub async fn skill_schema(
    &self,
    Parameters(params): Parameters<params::skills::SkillSchemaParams>
) -> Result<CallToolResult, McpError> {
    tools::skills::schema(params)
}

// ========================================================================
// Watchtower Tools (5) - Session monitoring and primitive extraction
// ========================================================================

#[tool(
    description = "List saved Watchtower sessions. Returns session files with metadata (size, modified time) from ~/.claude/logs/sessions/."
)]
pub async fn watchtower_sessions_list(
    &self,
    Parameters(_params): Parameters<params::watchtower::WatchtowerSessionsListParams>
) -> Result<CallToolResult, McpError> {
    let result = tools::watchtower::watchtower_sessions_list();
    Ok(
        CallToolResult::success(
            vec![
                rmcp::model::Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
                )
            ]
        )
    )
}

#[tool(
    description = "Get active Claude Code sessions from recent log entries. Returns session IDs and their working directories."
)]
pub async fn watchtower_active_sessions(
    &self,
    Parameters(_params): Parameters<params::watchtower::WatchtowerActiveSessionsParams>
) -> Result<CallToolResult, McpError> {
    let result = tools::watchtower::watchtower_active_sessions();
    Ok(
        CallToolResult::success(
            vec![
                rmcp::model::Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
                )
            ]
        )
    )
}

#[tool(
    description = "Analyze a session log and extract behavioral primitives. Returns statistics, T1/T2/T3 primitives, anti-patterns, and transfer confidence scores."
)]
pub async fn watchtower_analyze(
    &self,
    Parameters(params): Parameters<params::watchtower::WatchtowerAnalyzeParams>
) -> Result<CallToolResult, McpError> {
    let result = tools::watchtower::watchtower_analyze(params.session_path.as_deref());
    Ok(
        CallToolResult::success(
            vec![
                rmcp::model::Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
                )
            ]
        )
    )
}

#[tool(
    description = "Get hook telemetry statistics. Returns tool distribution, average hook timing, and session activity from hook_telemetry.jsonl."
)]
pub async fn watchtower_telemetry_stats(
    &self,
    Parameters(_params): Parameters<params::watchtower::WatchtowerTelemetryStatsParams>
) -> Result<CallToolResult, McpError> {
    let result = tools::watchtower::watchtower_telemetry_stats();
    Ok(
        CallToolResult::success(
            vec![
                rmcp::model::Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
                )
            ]
        )
    )
}

#[tool(
    description = "Get recent log entries from commands.log. Optionally filter by session ID and limit count."
)]
pub async fn watchtower_recent(
    &self,
    Parameters(params): Parameters<params::watchtower::WatchtowerRecentParams>
) -> Result<CallToolResult, McpError> {
    let result = tools::watchtower::watchtower_recent(
        params.count,
        params.session_filter.as_deref()
    );
    Ok(
        CallToolResult::success(
            vec![
                rmcp::model::Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
                )
            ]
        )
    )
}

#[tool(
    description = "Audit symbols in files for potential collisions. Scans for single-letter and short symbols, groups by context (definition/equation/reference), and flags where same symbol may have different meanings."
)]
pub async fn watchtower_symbol_audit(
    &self,
    Parameters(params): Parameters<params::watchtower::WatchtowerSymbolAuditParams>
) -> Result<CallToolResult, McpError> {
    let result = tools::watchtower::watchtower_symbol_audit(&params.path);
    Ok(
        CallToolResult::success(
            vec![
                rmcp::model::Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
                )
            ]
        )
    )
}

#[tool(
    description = "Get Gemini API call statistics. Returns total calls, token usage, latency, and breakdown by session/flow from gemini_telemetry.jsonl."
)]
pub async fn watchtower_gemini_stats(
    &self,
    Parameters(_params): Parameters<params::watchtower::WatchtowerGeminiStatsParams>
) -> Result<CallToolResult, McpError> {
    let result = tools::watchtower::watchtower_gemini_stats();
    Ok(
        CallToolResult::success(
            vec![
                rmcp::model::Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
                )
            ]
        )
    )
}

#[tool(
    description = "Get recent Gemini API calls. Returns the latest N entries with timestamps, models, tokens, and latency."
)]
pub async fn watchtower_gemini_recent(
    &self,
    Parameters(params): Parameters<params::watchtower::WatchtowerGeminiRecentParams>
) -> Result<CallToolResult, McpError> {
    let result = tools::watchtower::watchtower_gemini_recent(params.count);
    Ok(
        CallToolResult::success(
            vec![
                rmcp::model::Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
                )
            ]
        )
    )
}

#[tool(
    description = "Get unified Claude + Gemini telemetry view. Combines hook telemetry and Gemini API stats into a single dashboard."
)]
pub async fn watchtower_unified(
    &self,
    Parameters(params): Parameters<params::watchtower::WatchtowerUnifiedParams>
) -> Result<CallToolResult, McpError> {
    let result = tools::watchtower::watchtower_unified(
        params.include_claude,
        params.include_gemini
    );
    Ok(
        CallToolResult::success(
            vec![
                rmcp::model::Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
                )
            ]
        )
    )
}

// ========================================================================
// Telemetry Intelligence Tools (6) - External source monitoring
// ========================================================================

#[tool(
    description = "List all discovered telemetry sources (sessions). Scans telemetry directories and returns session metadata including project hash, filename, and path."
)]
pub async fn telemetry_sources_list(
    &self,
    Parameters(params): Parameters<params::telemetry::TelemetrySourcesListParams>
) -> Result<CallToolResult, McpError> {
    tools::telemetry_intel::sources_list(params)
}

#[tool(
    description = "Deep analysis of a specific telemetry source session. Returns operation counts, file access patterns, and token usage. Provide session_path or project_hash."
)]
pub async fn telemetry_source_analyze(
    &self,
    Parameters(params): Parameters<params::telemetry::TelemetrySourceAnalyzeParams>
) -> Result<CallToolResult, McpError> {
    tools::telemetry_intel::source_analyze(params)
}

#[tool(
    description = "Cross-reference telemetry with governance module changes. Identifies file accesses to governance-related paths (primitives, governance, capabilities, constitutional) and tracks modifications."
)]
pub async fn telemetry_governance_crossref(
    &self,
    Parameters(params): Parameters<params::telemetry::TelemetryGovernanceCrossrefParams>
) -> Result<CallToolResult, McpError> {
    tools::telemetry_intel::governance_crossref(params)
}

#[tool(
    description = "Track snapshot/artifact version history. Discovers brain sessions and their artifacts, showing version evolution over time. Provide session_id for specific session or omit for overview."
)]
pub async fn telemetry_snapshot_evolution(
    &self,
    Parameters(params): Parameters<params::telemetry::TelemetrySnapshotEvolutionParams>
) -> Result<CallToolResult, McpError> {
    tools::telemetry_intel::snapshot_evolution(params)
}

#[tool(
    description = "Full intelligence report from telemetry and brain. Returns consolidated metrics for session activity, token usage, and cross-domain file access."
)]
pub async fn telemetry_intel_report(
    &self,
    Parameters(params): Parameters<params::telemetry::TelemetryIntelReportParams>
) -> Result<CallToolResult, McpError> {
    tools::telemetry_intel::intel_report(params)
}

#[tool(
    description = "Get recent activity from telemetry sources. Returns last N operations across all sessions."
)]
pub async fn telemetry_recent(
    &self,
    Parameters(params): Parameters<params::telemetry::TelemetryRecentParams>
) -> Result<CallToolResult, McpError> {
    tools::telemetry_intel::recent_activity(params)
}

// ========================================================================
// Skill Compiler Tools (2)
// ========================================================================

#[tool(
    description = "Compile 2+ skills into a single compound skill binary. Generates a Rust crate with SKILL.md, optionally builds it. Params: skills (list), strategy (sequential/parallel/feedback_loop), name, build (bool)."
)]
pub async fn skill_compile(
    &self,
    Parameters(params): Parameters<params::skills::SkillCompileParams>
) -> Result<CallToolResult, McpError> {
    tools::skills::compile(params)
}

#[tool(
    description = "Check if skills can be compiled into a compound skill (dry run). Returns compatibility report with warnings and blockers."
)]
pub async fn skill_compile_check(
    &self,
    Parameters(params): Parameters<params::skills::SkillCompileCheckParams>
) -> Result<CallToolResult, McpError> {
    tools::skills::compile_check(params)
}

#[tool(
    description = "Analyze token usage in a skill directory. Returns per-file metrics (chars, tokens, lines, code blocks), total estimated tokens, and optimization recommendations. Useful for context window optimization."
)]
pub async fn skill_token_analyze(
    &self,
    Parameters(params): Parameters<params::skills::SkillTokenAnalyzeParams>
) -> Result<CallToolResult, McpError> {
    tools::skill_tokens::analyze(params)
}
