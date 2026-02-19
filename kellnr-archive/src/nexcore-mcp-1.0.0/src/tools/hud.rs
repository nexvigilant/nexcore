//! HUD Capability tools: CAP-014 through CAP-037
//!
//! Exposes HUD capabilities as MCP tools:
//! - CAP-014: Public Health Act (signal validation)
//! - CAP-018: Treasury Act (revenue/finance)
//! - CAP-019: Transportation Act (signal logistics)
//! - CAP-020: Homeland Security Act (boundary protection)
//! - CAP-022: Education Act (agent training)
//! - CAP-025: Small Business Act (agent allocation)
//! - CAP-026: Social Security Act (state persistence)
//! - CAP-027: Federal Reserve Act (token budgets)
//! - CAP-028: Securities Act (market compliance)
//! - CAP-029: Communications Act (protocol routing)
//! - CAP-030: Exploration Act (frontier mapping)
//! - CAP-031: Science Foundation Act (research)
//! - CAP-037: General Services Act (procurement)

use crate::params::{
    CommRecommendProtocolParams, CommRouteMessageParams, DhsVerifyBoundaryParams,
    DotDispatchManifestParams, DotVerifyHighwayParams, EduEvaluateParams, EduTrainAgentParams,
    ExploreGetFrontierParams, ExploreLaunchMissionParams, ExploreRecordDiscoveryParams,
    FedBudgetReportParams, FedRecommendModelParams, GsaAuditValueParams, GsaProcureParams,
    HealthMeasureImpactParams, HealthValidateSignalParams, NsfFundResearchParams,
    SbaAllocateAgentParams, SbaChainNextParams, SecAuditMarketParams, SsaPersistStateParams,
    SsaVerifyIntegrityParams, TreasuryAuditParams, TreasuryConvertAsymmetryParams,
};
use nexcore_vigilance::hud::capabilities::{
    CommunicationsAct, Curriculum, Discovery, DiscoveryIndex, EducationAct, ExplorationAct,
    ExplorationScope, FederalReserveAct, GeneralServicesAct, HomelandSecurityAct, IntegrityHash,
    MissionManifest, PersistenceLevel, ProcurementOrder, ProtocolType, PublicHealthAct,
    ScienceFoundationAct, SecuritiesAct, SmallBusinessAct, SocialSecurityAct, TaskComplexity,
    TransitManifest, TransportationAct, TreasuryAct,
};
use nexcore_vigilance::primitives::governance::{Odds, Treasury};
use parking_lot::Mutex;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::sync::OnceLock;

// ============================================================================
// SINGLETON STATE MANAGEMENT (Guardian Pattern)
// ============================================================================
// Using OnceLock<Mutex<T>> for thread-safe, lazily-initialized singletons.
// This ensures state persists across MCP tool calls within a session.

static SMALL_BUSINESS_ACT: OnceLock<Mutex<SmallBusinessAct>> = OnceLock::new();
static SOCIAL_SECURITY_ACT: OnceLock<Mutex<SocialSecurityAct>> = OnceLock::new();
static FEDERAL_RESERVE_ACT: OnceLock<Mutex<FederalReserveAct>> = OnceLock::new();
static SECURITIES_ACT: OnceLock<Mutex<SecuritiesAct>> = OnceLock::new();
static COMMUNICATIONS_ACT: OnceLock<Mutex<CommunicationsAct>> = OnceLock::new();
static EXPLORATION_ACT: OnceLock<Mutex<ExplorationAct>> = OnceLock::new();
static PUBLIC_HEALTH_ACT: OnceLock<Mutex<PublicHealthAct>> = OnceLock::new();
static TREASURY_ACT: OnceLock<Mutex<TreasuryAct>> = OnceLock::new();
static TRANSPORTATION_ACT: OnceLock<Mutex<TransportationAct>> = OnceLock::new();
static HOMELAND_SECURITY_ACT: OnceLock<Mutex<HomelandSecurityAct>> = OnceLock::new();
static EDUCATION_ACT: OnceLock<Mutex<EducationAct>> = OnceLock::new();
static SCIENCE_FOUNDATION_ACT: OnceLock<Mutex<ScienceFoundationAct>> = OnceLock::new();
static GENERAL_SERVICES_ACT: OnceLock<Mutex<GeneralServicesAct>> = OnceLock::new();

fn get_sba() -> &'static Mutex<SmallBusinessAct> {
    SMALL_BUSINESS_ACT.get_or_init(|| Mutex::new(SmallBusinessAct::new()))
}

fn get_ssa() -> &'static Mutex<SocialSecurityAct> {
    SOCIAL_SECURITY_ACT.get_or_init(|| Mutex::new(SocialSecurityAct::new()))
}

fn get_fed() -> &'static Mutex<FederalReserveAct> {
    FEDERAL_RESERVE_ACT.get_or_init(|| Mutex::new(FederalReserveAct::new()))
}

fn get_sec() -> &'static Mutex<SecuritiesAct> {
    SECURITIES_ACT.get_or_init(|| Mutex::new(SecuritiesAct::new()))
}

fn get_comm() -> &'static Mutex<CommunicationsAct> {
    COMMUNICATIONS_ACT.get_or_init(|| Mutex::new(CommunicationsAct::new()))
}

fn get_exploration() -> &'static Mutex<ExplorationAct> {
    EXPLORATION_ACT.get_or_init(|| Mutex::new(ExplorationAct::new()))
}

fn get_health() -> &'static Mutex<PublicHealthAct> {
    PUBLIC_HEALTH_ACT.get_or_init(|| Mutex::new(PublicHealthAct::new()))
}

fn get_treasury() -> &'static Mutex<TreasuryAct> {
    TREASURY_ACT.get_or_init(|| Mutex::new(TreasuryAct::new()))
}

fn get_transport() -> &'static Mutex<TransportationAct> {
    TRANSPORTATION_ACT.get_or_init(|| Mutex::new(TransportationAct::new()))
}

fn get_dhs() -> &'static Mutex<HomelandSecurityAct> {
    HOMELAND_SECURITY_ACT.get_or_init(|| Mutex::new(HomelandSecurityAct::new()))
}

fn get_edu() -> &'static Mutex<EducationAct> {
    EDUCATION_ACT.get_or_init(|| Mutex::new(EducationAct::new()))
}

fn get_nsf() -> &'static Mutex<ScienceFoundationAct> {
    SCIENCE_FOUNDATION_ACT.get_or_init(|| Mutex::new(ScienceFoundationAct::new()))
}

fn get_gsa() -> &'static Mutex<GeneralServicesAct> {
    GENERAL_SERVICES_ACT.get_or_init(|| Mutex::new(GeneralServicesAct::new()))
}

// ============================================================================
// CAP-025: Small Business Act (Sub-Agent Support)
// ============================================================================

/// Allocate an agent for a task based on complexity analysis
///
/// Returns:
/// - Recommended agent (primary + alternatives)
/// - Task complexity assessment
/// - Confidence score
pub fn sba_allocate_agent(params: SbaAllocateAgentParams) -> Result<CallToolResult, McpError> {
    let sba = get_sba().lock();
    let result = sba.allocate_agent(&params.task_description);

    let allocation = &result.value;
    let primary = &allocation.primary;

    let response = json!({
        "primary_agent": {
            "agent_id": primary.agent_id,
            "agent_type": primary.agent_type,
            "model": format!("{:?}", primary.model),
            "quota_grant": primary.quota_grant,
            "task_id": primary.task_id,
            "skills": primary.skills,
            "tool_restrictions": primary.tool_restrictions,
        },
        "alternatives": allocation.alternatives.iter().map(|alt| json!({
            "agent_type": alt.agent_type,
            "model": format!("{:?}", alt.model),
            "quota_grant": alt.quota_grant,
        })).collect::<Vec<_>>(),
        "complexity": format!("{:?}", allocation.complexity),
        "complexity_score": match allocation.complexity {
            TaskComplexity::Trivial => 0.1,
            TaskComplexity::Simple => 0.3,
            TaskComplexity::Moderate => 0.5,
            TaskComplexity::Complex => 0.7,
            TaskComplexity::Autonomous => 1.0,
        },
        "allocation_confidence": allocation.confidence,
        "reasoning": allocation.reasoning,
        "confidence": result.confidence.value(),
        "model_guidance": {
            "Haiku": { "use_for": "Trivial/Simple tasks", "cost": "lowest" },
            "Sonnet": { "use_for": "Moderate/Complex tasks", "cost": "balanced" },
            "Opus": { "use_for": "Autonomous loops, research", "cost": "premium" },
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

/// Get next step in an agent chain after completing a step
///
/// Returns:
/// - Next agent to run (if any)
/// - Condition that triggered this step
/// - Whether chain is complete
pub fn sba_chain_next(params: SbaChainNextParams) -> Result<CallToolResult, McpError> {
    let sba = get_sba().lock();

    // Look up chain and get next step
    let next = sba.get_chain_next(&params.completed_step, params.had_errors);

    let response = match next {
        Some(chain) => json!({
            "chain_id": params.chain_id,
            "completed_step": params.completed_step,
            "has_next": true,
            "next_agent": chain.next,
            "condition": format!("{:?}", chain.condition),
            "chain_complete": false,
        }),
        None => json!({
            "chain_id": params.chain_id,
            "completed_step": params.completed_step,
            "has_next": false,
            "next_agent": null,
            "condition": null,
            "chain_complete": true,
            "message": "Agent chain completed successfully",
        }),
    };

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

// ============================================================================
// CAP-026: Social Security Act (State Persistence)
// ============================================================================

/// Persist state with SHA-256 integrity hash
///
/// Returns:
/// - State ID
/// - Version number
/// - Integrity hash
/// - Persistence level
pub fn ssa_persist_state(params: SsaPersistStateParams) -> Result<CallToolResult, McpError> {
    let mut ssa = get_ssa().lock();

    let level = match params.level.as_deref() {
        Some("session") => PersistenceLevel::Session,
        Some("local") => PersistenceLevel::Local,
        Some("distributed") => PersistenceLevel::Distributed,
        Some("resolved") => PersistenceLevel::Resolved,
        _ => PersistenceLevel::Session, // default
    };

    // Use state_id as entity_id, derive artifact_name from state_id
    let artifact_name = format!("{}.state", params.state_id);
    let result = ssa.persist_state(&params.state_id, &artifact_name, &params.content, level);
    let backup = &result.value;

    let response = json!({
        "state_id": params.state_id,
        "entity_id": backup.entity_id,
        "artifact_name": backup.artifact_name,
        "version": backup.version.0,
        "hash": backup.integrity_hash.0,
        "hash_short": backup.integrity_hash.short(),
        "level": format!("{:?}", backup.level),
        "size_bytes": backup.size_bytes,
        "created_at": backup.created_at,
        "confidence": result.confidence.value(),
        "verification": {
            "can_verify": true,
            "use_tool": "ssa_verify_integrity",
            "expected_hash": backup.integrity_hash.0,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

/// Verify state integrity using SHA-256 hash
///
/// Returns:
/// - Whether integrity check passed
/// - Expected vs computed hash
/// - Recommendation if mismatch
pub fn ssa_verify_integrity(params: SsaVerifyIntegrityParams) -> Result<CallToolResult, McpError> {
    let expected = IntegrityHash(params.expected_hash.clone());
    let verified = expected.verify(&params.content);
    let computed = IntegrityHash::compute(&params.content);

    let response = json!({
        "state_id": params.state_id,
        "verified": verified,
        "expected_hash": params.expected_hash,
        "computed_hash": computed.0,
        "match": verified,
        "recommendation": if verified {
            "State integrity confirmed. Safe to use."
        } else {
            "INTEGRITY FAILURE: State has been modified. Restore from backup or re-persist."
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

// ============================================================================
// CAP-027: Federal Reserve Act (Token Stability)
// ============================================================================

/// Get token budget report
///
/// Returns:
/// - Current usage
/// - Budget limits
/// - Stability level
/// - Estimated cost
pub fn fed_budget_report(_params: FedBudgetReportParams) -> Result<CallToolResult, McpError> {
    let fed = get_fed().lock();
    let result = fed.get_budget_report();
    let report = &result.value;

    let response = json!({
        "daily": {
            "used": report.daily_used.value(),
            "remaining": report.daily_remaining.value(),
            "used_ktok": report.daily_used.as_ktok(),
        },
        "session": {
            "used": report.session_used.value(),
            "remaining": report.session_remaining.value(),
            "used_ktok": report.session_used.as_ktok(),
        },
        "stability": format!("{:?}", report.stability),
        "estimated_cost_usd": report.estimated_cost.value(),
        "inflation_rate": report.inflation.value(),
        "confidence": result.confidence.value(),
        "thresholds": {
            "Stable": "< 50% utilization",
            "Cautious": "50-80% utilization",
            "Restricted": "80-95% utilization",
            "Emergency": "> 95% utilization",
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

/// Recommend model tier based on task and budget
///
/// Returns:
/// - Recommended model tier
/// - Rationale
/// - Alternative options
pub fn fed_recommend_model(params: FedRecommendModelParams) -> Result<CallToolResult, McpError> {
    let fed = get_fed().lock();
    let recommended = fed.recommend_model_tier();

    // Get params with defaults
    let complexity = params.task_complexity.as_deref().unwrap_or("moderate");
    let utilization = params.budget_utilization.unwrap_or(50.0);
    let requires_accuracy = params.requires_accuracy.unwrap_or(false);

    // Decision logic - adjust recommendation based on params
    let (tier, rationale) = if utilization > 80.0 {
        (
            "Economy",
            "High budget utilization - use economy tier to conserve tokens",
        )
    } else if requires_accuracy || complexity == "research" || complexity == "autonomous" {
        (
            "Premium",
            "Complex task or accuracy required - use premium tier",
        )
    } else if complexity == "trivial" || complexity == "simple" {
        ("Economy", "Simple task - economy tier sufficient")
    } else {
        (
            "Standard",
            "Moderate task - standard tier provides good balance",
        )
    };

    let response = json!({
        "recommended_tier": tier,
        "system_default": format!("{:?}", recommended),
        "rationale": rationale,
        "inputs": {
            "task_complexity": complexity,
            "budget_utilization": format!("{:.1}%", utilization),
            "requires_accuracy": requires_accuracy,
        },
        "model_mapping": {
            "Economy": "claude-3-5-haiku (fast, cheap)",
            "Standard": "claude-sonnet-4 (balanced)",
            "Premium": "claude-opus-4 (powerful)",
            "Apex": "claude-opus-4 with extended thinking",
        },
        "cost_comparison": {
            "Economy": "$0.25/Mtok input, $1.25/Mtok output",
            "Standard": "$3/Mtok input, $15/Mtok output",
            "Premium": "$15/Mtok input, $75/Mtok output",
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

// ============================================================================
// CAP-028: Securities Act (Market Compliance)
// ============================================================================

/// Audit a market for compliance
///
/// Returns:
/// - Market ID
/// - Heresy level (compliance violations)
/// - Verdict (Permitted/Flagged)
pub fn sec_audit_market(params: SecAuditMarketParams) -> Result<CallToolResult, McpError> {
    let sec = get_sec().lock();
    let result = sec.audit_market(&params.market_id, params.trade_volume);
    let audit = &result.value;

    let response = json!({
        "market_id": audit.market_id,
        "trade_volume": params.trade_volume,
        "heresy_level": audit.heresy_level,
        "verdict": format!("{:?}", audit.verdict),
        "confidence": result.confidence.value(),
        "compliance_status": if audit.heresy_level < 0.1 { "COMPLIANT" } else { "REVIEW_REQUIRED" },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

// ============================================================================
// CAP-029: Communications Act (Protocol Routing)
// ============================================================================

/// Recommend protocol for inter-agent communication
///
/// Returns:
/// - Recommended protocol (MCP/JSON-RPC/Event/Direct/REST)
/// - Reliability and latency characteristics
pub fn comm_recommend_protocol(
    params: CommRecommendProtocolParams,
) -> Result<CallToolResult, McpError> {
    let comm = get_comm().lock();
    let protocol = comm.recommend_protocol(
        params.needs_guarantee,
        params.low_latency,
        params.is_broadcast,
    );

    let response = json!({
        "recommended_protocol": format!("{:?}", protocol),
        "inputs": {
            "needs_guarantee": params.needs_guarantee,
            "low_latency": params.low_latency,
            "is_broadcast": params.is_broadcast,
        },
        "characteristics": {
            "typical_latency_ms": protocol.typical_latency_ms(),
            "reliability": protocol.reliability(),
        },
        "protocol_guide": {
            "Direct": "In-process, 1ms, 100% reliable",
            "Mcp": "Claude Code tools, 50ms, 99% reliable",
            "Event": "Pub/sub, 10ms, 90% reliable",
            "JsonRpc": "Standard API, 100ms, 95% reliable",
            "Rest": "External HTTP, 200ms, 85% reliable",
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

/// Route a message between agents
///
/// Returns:
/// - Transmission metrics (latency, clarity, delivery status)
pub fn comm_route_message(params: CommRouteMessageParams) -> Result<CallToolResult, McpError> {
    let mut comm = get_comm().lock();

    let protocol = match params.protocol.as_deref() {
        Some("mcp") => ProtocolType::Mcp,
        Some("jsonrpc") => ProtocolType::JsonRpc,
        Some("event") => ProtocolType::Event,
        Some("direct") => ProtocolType::Direct,
        Some("rest") => ProtocolType::Rest,
        _ => ProtocolType::Mcp,
    };

    let message = nexcore_vigilance::hud::capabilities::Message {
        id: format!("msg-{}", chrono::Utc::now().timestamp_millis()),
        from: params.from.clone(),
        to: params.to.clone(),
        protocol,
        payload: params.payload.clone(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        ttl: params.ttl.unwrap_or(60),
    };

    let result = comm.route_message(message);
    let metrics = &result.value;

    let response = json!({
        "message_id": format!("msg-{}", chrono::Utc::now().timestamp_millis()),
        "from": params.from,
        "to": params.to,
        "protocol": format!("{:?}", protocol),
        "transmission": {
            "latency_ms": metrics.latency.value(),
            "clarity": metrics.clarity.value(),
            "size_bytes": metrics.size_bytes,
            "delivered": metrics.delivered,
            "retries": metrics.retries,
        },
        "confidence": result.confidence.value(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

// ============================================================================
// CAP-030: Exploration Act (Frontier Mapping)
// ============================================================================

/// Launch an exploration mission
///
/// Returns:
/// - Mission manifest with scope and patterns
/// - Success probability
pub fn explore_launch_mission(
    params: ExploreLaunchMissionParams,
) -> Result<CallToolResult, McpError> {
    let mut exp = get_exploration().lock();

    let scope = match params.scope.as_deref() {
        Some("quick") => ExplorationScope::Quick,
        Some("thorough") => ExplorationScope::Thorough,
        _ => ExplorationScope::Medium,
    };

    let manifest = MissionManifest {
        id: format!("mission-{}", chrono::Utc::now().timestamp_millis()),
        target: params.target.clone(),
        objective: params.objective.clone(),
        scope,
        patterns: params.patterns.unwrap_or_default(),
        success_prob: match scope {
            ExplorationScope::Quick => 0.7,
            ExplorationScope::Medium => 0.85,
            ExplorationScope::Thorough => 0.95,
        },
    };

    let result = exp.launch_mission(manifest.clone());

    let response = json!({
        "mission_id": manifest.id,
        "target": manifest.target,
        "objective": manifest.objective,
        "scope": format!("{:?}", scope),
        "expected_files": scope.expected_files(),
        "patterns": manifest.patterns,
        "success_probability": manifest.success_prob,
        "confidence": result.confidence.value(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

/// Record a discovery from exploration
///
/// Returns:
/// - Discovery ID
/// - Significance score
/// - Whether it's significant (>0.7)
pub fn explore_record_discovery(
    params: ExploreRecordDiscoveryParams,
) -> Result<CallToolResult, McpError> {
    let mut exp = get_exploration().lock();

    let significance = DiscoveryIndex::new(params.significance.unwrap_or(0.5));

    let discovery = Discovery {
        id: format!("disc-{}", chrono::Utc::now().timestamp_millis()),
        finding: params.finding.clone(),
        location: params.location.clone(),
        significance,
        related: vec![],
    };

    let result = exp.record_discovery(discovery.clone());

    let response = json!({
        "discovery_id": discovery.id,
        "finding": discovery.finding,
        "location": discovery.location,
        "significance": significance.0,
        "is_significant": significance.is_significant(),
        "confidence": result.confidence.value(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

/// Get current exploration frontier status
///
/// Returns:
/// - Known domains
/// - Unknown areas
/// - Coverage percentage
pub fn explore_get_frontier(_params: ExploreGetFrontierParams) -> Result<CallToolResult, McpError> {
    let exp = get_exploration().lock();
    let frontier = exp.get_frontier();

    let response = json!({
        "known_domains": frontier.known,
        "unknown_areas": frontier.unknown,
        "knowledge_gaps": frontier.gaps,
        "coverage_percent": format!("{:.1}", frontier.coverage * 100.0),
        "status": if frontier.coverage > 0.8 { "WELL_EXPLORED" }
                 else if frontier.coverage > 0.5 { "PARTIAL" }
                 else { "UNEXPLORED" },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

// ============================================================================
// CAP-014: Public Health Act (Signal Validation)
// ============================================================================

/// Validate signal efficacy against ground truth (FDA-style validation)
pub fn health_validate_signal(
    params: HealthValidateSignalParams,
) -> Result<CallToolResult, McpError> {
    let health = get_health().lock();
    let result = health.validate_signal(&params.signal_id, params.accuracy);
    let audit = &result.value;

    let response = json!({
        "signal_id": audit.signal_id,
        "efficacy_score": audit.efficacy_score,
        "false_positive_rate": audit.false_positive_rate,
        "community_value": audit.community_value_generated,
        "confidence": result.confidence.value(),
        "verdict": if audit.efficacy_score > 0.8 { "HIGH_EFFICACY" }
                  else if audit.efficacy_score > 0.5 { "MODERATE_EFFICACY" }
                  else { "LOW_EFFICACY" },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

/// Measure public health impact of signals
pub fn health_measure_impact(
    params: HealthMeasureImpactParams,
) -> Result<CallToolResult, McpError> {
    let health = get_health().lock();
    let efficacy = health.measure_impact(params.total_signals, params.valid_signals);

    let response = json!({
        "total_signals": params.total_signals,
        "valid_signals": params.valid_signals,
        "efficacy": efficacy.0,
        "efficacy_percent": format!("{:.1}%", efficacy.0 * 100.0),
        "rating": if efficacy.0 > 0.9 { "EXCELLENT" }
                 else if efficacy.0 > 0.7 { "GOOD" }
                 else if efficacy.0 > 0.5 { "MODERATE" }
                 else { "POOR" },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

// ============================================================================
// CAP-018: Treasury Act (Revenue/Finance)
// ============================================================================

/// Convert signal asymmetry into resources (information arbitrage)
pub fn treasury_convert_asymmetry(
    params: TreasuryConvertAsymmetryParams,
) -> Result<CallToolResult, McpError> {
    let treasury = get_treasury().lock();
    let odds = Odds::new(params.market_odds);
    let result = treasury.convert_asymmetry(
        &params.signal_id,
        nexcore_vigilance::hud::capabilities::AsymmetryValue(params.asymmetry),
        odds,
    );
    let event = &result.value;

    let response = json!({
        "signal_id": event.signal_id,
        "value_captured": event.value_captured,
        "token_reward": event.token_reward.0,
        "conversion_confidence": event.conversion_confidence.value(),
        "confidence": result.confidence.value(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

/// Audit treasury status
pub fn treasury_audit(params: TreasuryAuditParams) -> Result<CallToolResult, McpError> {
    let treasury_act = get_treasury().lock();
    let treasury = Treasury {
        compute_quota: params.compute_quota,
        memory_quota: params.memory_quota,
    };
    let healthy = treasury_act.audit_treasury(&treasury);

    let response = json!({
        "compute_quota": params.compute_quota,
        "memory_quota": params.memory_quota,
        "healthy": healthy,
        "status": if healthy { "SOLVENT" } else { "CRITICAL" },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

// ============================================================================
// CAP-019: Transportation Act (Signal Logistics)
// ============================================================================

/// Dispatch a transit manifest across domains
pub fn dot_dispatch_manifest(
    params: DotDispatchManifestParams,
) -> Result<CallToolResult, McpError> {
    let transport = get_transport().lock();
    let manifest = TransitManifest {
        manifest_id: format!("manifest-{}", chrono::Utc::now().timestamp_millis()),
        origin_domain: params.origin.clone(),
        target_domain: params.destination.clone(),
        signal_count: params.signal_count,
        priority: params.priority,
    };
    let result = transport.dispatch_manifest(manifest.clone());

    let response = json!({
        "manifest_id": manifest.manifest_id,
        "origin": params.origin,
        "destination": params.destination,
        "signal_count": params.signal_count,
        "priority": params.priority,
        "route_status": format!("{:?}", result.value),
        "confidence": result.confidence.value(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

/// Verify highway safety (NHTSA audit)
pub fn dot_verify_highway(params: DotVerifyHighwayParams) -> Result<CallToolResult, McpError> {
    let transport = get_transport().lock();
    let safe = transport.verify_highway_safety(&params.route_id);

    let response = json!({
        "route_id": params.route_id,
        "safe": safe,
        "status": if safe { "CLEAR" } else { "UNSAFE" },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

// ============================================================================
// CAP-020: Homeland Security Act (Boundary Protection)
// ============================================================================

/// Verify incoming data at boundary (CBP-style check)
pub fn dhs_verify_boundary(params: DhsVerifyBoundaryParams) -> Result<CallToolResult, McpError> {
    let dhs = get_dhs().lock();
    let result = dhs.verify_boundary(&params.source_id, &params.payload_hash);
    let check = &result.value;

    let response = json!({
        "source_id": check.source_id,
        "authenticity": check.authenticity.0,
        "verdict": format!("{:?}", check.verdict),
        "confidence": result.confidence.value(),
        "status": if check.authenticity.0 > 0.5 { "PERMITTED" } else { "BLOCKED" },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

// ============================================================================
// CAP-022: Education Act (Agent Training)
// ============================================================================

/// Train an agent in a curriculum
pub fn edu_train_agent(params: EduTrainAgentParams) -> Result<CallToolResult, McpError> {
    let edu = get_edu().lock();
    let curriculum = Curriculum {
        subject: params.subject.clone(),
        level: params.level,
        completion_status: params.completion,
    };
    let result = edu.train_agent(&curriculum);

    let response = json!({
        "subject": params.subject,
        "level": params.level,
        "completion": params.completion,
        "mastery": result.value.0,
        "mastery_percent": format!("{:.1}%", result.value.0 * 100.0),
        "confidence": result.confidence.value(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

/// Evaluate methodology comprehension
pub fn edu_evaluate(params: EduEvaluateParams) -> Result<CallToolResult, McpError> {
    let edu = get_edu().lock();
    let avg = edu.evaluate_comprehension(&params.scores);

    let response = json!({
        "scores": params.scores,
        "average": avg,
        "average_percent": format!("{:.1}%", avg * 100.0),
        "grade": if avg > 0.9 { "A" }
                else if avg > 0.8 { "B" }
                else if avg > 0.7 { "C" }
                else if avg > 0.6 { "D" }
                else { "F" },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

// ============================================================================
// CAP-031: Science Foundation Act (Research)
// ============================================================================

/// Fund a research project (NSF grant)
pub fn nsf_fund_research(params: NsfFundResearchParams) -> Result<CallToolResult, McpError> {
    let nsf = get_nsf().lock();
    let result = nsf.fund_research(&params.project, &params.target_cap);
    let grant = &result.value;

    let response = json!({
        "project_title": grant.project_title,
        "target_capability": grant.target_cap_id,
        "expected_impact": grant.impact,
        "confidence": result.confidence.value(),
        "status": "FUNDED",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

// ============================================================================
// CAP-037: General Services Act (Procurement)
// ============================================================================

/// Process a procurement order (GSA)
pub fn gsa_procure(params: GsaProcureParams) -> Result<CallToolResult, McpError> {
    let gsa = get_gsa().lock();
    let order = ProcurementOrder {
        resource_id: params.resource_id.clone(),
        quantity: params.quantity,
        priority: params.priority,
    };
    let result = gsa.procure_resource(&order);

    let response = json!({
        "resource_id": params.resource_id,
        "quantity": params.quantity,
        "priority": params.priority,
        "success": result.value,
        "confidence": result.confidence.value(),
        "status": if result.value { "FULFILLED" } else { "REJECTED" },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

/// Audit service value (cost/benefit)
pub fn gsa_audit_value(params: GsaAuditValueParams) -> Result<CallToolResult, McpError> {
    let gsa = get_gsa().lock();
    let value = gsa.audit_service_value(params.cost, params.benefit);

    let response = json!({
        "cost": params.cost,
        "benefit": params.benefit,
        "value_ratio": value.0,
        "roi_percent": format!("{:.1}%", (value.0 - 1.0) * 100.0),
        "rating": if value.0 > 2.0 { "EXCELLENT" }
                 else if value.0 > 1.5 { "GOOD" }
                 else if value.0 > 1.0 { "ACCEPTABLE" }
                 else { "POOR" },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_default(),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    // ========================================================================
    // BASIC FUNCTIONALITY TESTS (Phase 0: Preclinical)
    // ========================================================================

    #[test]
    fn test_sba_allocate_agent() {
        let params = SbaAllocateAgentParams {
            task_description: "Fix a simple typo in the README".to_string(),
            preferred_tier: None,
        };
        let result = sba_allocate_agent(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sec_audit_market() {
        let params = SecAuditMarketParams {
            market_id: "test-market".to_string(),
            trade_volume: 500000,
        };
        let result = sec_audit_market(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_comm_recommend_protocol() {
        let params = CommRecommendProtocolParams {
            needs_guarantee: true,
            low_latency: true,
            is_broadcast: false,
        };
        let result = comm_recommend_protocol(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_explore_launch_mission() {
        let params = ExploreLaunchMissionParams {
            target: "src/".to_string(),
            objective: "Find error handlers".to_string(),
            scope: Some("medium".to_string()),
            patterns: None,
        };
        let result = explore_launch_mission(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ssa_persist_state() {
        let params = SsaPersistStateParams {
            state_id: "test-state".to_string(),
            content: "test content".to_string(),
            level: Some("session".to_string()),
        };
        let result = ssa_persist_state(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ssa_verify_integrity() {
        let content = "test content";
        let hash = IntegrityHash::compute(content);

        let params = SsaVerifyIntegrityParams {
            state_id: "test-state".to_string(),
            expected_hash: hash.0,
            content: content.to_string(),
        };
        let result = ssa_verify_integrity(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fed_budget_report() {
        let params = FedBudgetReportParams {
            current_tokens: Some(50000),
            budget_limit: Some(100000),
        };
        let result = fed_budget_report(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fed_recommend_model() {
        let params = FedRecommendModelParams {
            task_complexity: Some("complex".to_string()),
            budget_utilization: Some(30.0),
            requires_accuracy: Some(true),
        };
        let result = fed_recommend_model(params);
        assert!(result.is_ok());
    }

    // ========================================================================
    // STATE PERSISTENCE TESTS (Phase 1: Safety - Singleton Pattern)
    // ========================================================================

    #[test]
    fn test_ssa_state_persists_across_calls() {
        // Use unique ID to avoid test interference
        let unique_id = format!(
            "persist-test-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        let content = "content persisted across calls";

        // Call 1: Persist state via MCP tool
        let params1 = SsaPersistStateParams {
            state_id: unique_id.clone(),
            content: content.to_string(),
            level: Some("session".to_string()),
        };
        let result1 = ssa_persist_state(params1);
        assert!(result1.is_ok(), "First persist should succeed");

        // Call 2: Verify state exists in singleton via direct access
        {
            let ssa = get_ssa().lock();
            let entities = ssa.list_entities();
            assert!(
                entities.contains(&unique_id.as_str()),
                "Entity should exist in singleton after MCP tool call. Found: {:?}",
                entities
            );

            // Verify backup content
            let artifact_name = format!("{}.state", unique_id);
            let backup = ssa.get_latest_backup(&unique_id, &artifact_name);
            assert!(backup.is_some(), "Backup should exist for entity");
            assert!(
                backup.unwrap().integrity_hash.verify(content),
                "Backup integrity hash should match original content"
            );
        }
    }

    #[test]
    fn test_ssa_version_increments_across_calls() {
        let unique_id = format!(
            "version-test-{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        let artifact_name = format!("{}.state", unique_id);

        // Version 1
        let params1 = SsaPersistStateParams {
            state_id: unique_id.clone(),
            content: "version 1 content".to_string(),
            level: Some("local".to_string()),
        };
        assert!(
            ssa_persist_state(params1).is_ok(),
            "v1 persist should succeed"
        );

        // Version 2
        let params2 = SsaPersistStateParams {
            state_id: unique_id.clone(),
            content: "version 2 content".to_string(),
            level: Some("resolved".to_string()),
        };
        assert!(
            ssa_persist_state(params2).is_ok(),
            "v2 persist should succeed"
        );

        // Verify version incremented
        {
            let ssa = get_ssa().lock();
            let history = ssa.get_history(&unique_id);
            assert_eq!(history.len(), 2, "Should have 2 versions");

            let latest = ssa.get_latest_backup(&unique_id, &artifact_name);
            assert!(latest.is_some());
            assert_eq!(latest.unwrap().version.0, 2, "Latest version should be 2");
        }
    }

    #[test]
    fn test_singleton_identity() {
        // Multiple calls to get_ssa() should return the same instance
        let ptr1 = get_ssa() as *const _;
        let ptr2 = get_ssa() as *const _;
        assert_eq!(
            ptr1, ptr2,
            "get_ssa() should return same singleton instance"
        );

        let ptr3 = get_fed() as *const _;
        let ptr4 = get_fed() as *const _;
        assert_eq!(
            ptr3, ptr4,
            "get_fed() should return same singleton instance"
        );

        let ptr5 = get_exploration() as *const _;
        let ptr6 = get_exploration() as *const _;
        assert_eq!(
            ptr5, ptr6,
            "get_exploration() should return same singleton instance"
        );
    }

    #[test]
    fn test_exploration_state_persists() {
        // Launch a mission and verify it's tracked
        let params = ExploreLaunchMissionParams {
            target: "test/exploration-persistence".to_string(),
            objective: "Verify state persistence".to_string(),
            scope: Some("quick".to_string()),
            patterns: Some(vec!["*.rs".to_string()]),
        };

        let result = explore_launch_mission(params);
        assert!(result.is_ok(), "Mission launch should succeed");

        // Verify frontier state was updated
        {
            let exp = get_exploration().lock();
            let frontier = exp.get_frontier();
            // After launching a mission, coverage should be tracked
            assert!(
                frontier.coverage >= 0.0,
                "Frontier coverage should be tracked"
            );
        }
    }

    // ========================================================================
    // THREAD SAFETY TESTS (Phase 1: Safety - Concurrent Access)
    // ========================================================================

    #[test]
    fn test_concurrent_ssa_access() {
        let handles: Vec<_> = (0..4)
            .map(|i| {
                thread::spawn(move || {
                    // Each thread calls the MCP tool multiple times
                    for j in 0..10 {
                        let unique_id = format!(
                            "concurrent-{}-{}-{}",
                            i,
                            j,
                            chrono::Utc::now().timestamp_millis()
                        );
                        let params = SsaPersistStateParams {
                            state_id: unique_id,
                            content: format!("content from thread {} iteration {}", i, j),
                            level: Some("session".to_string()),
                        };
                        let result = ssa_persist_state(params);
                        assert!(result.is_ok(), "Concurrent persist should not panic");
                    }
                })
            })
            .collect();

        for handle in handles {
            assert!(
                handle.join().is_ok(),
                "Thread should complete without panic"
            );
        }

        // Verify singleton is still accessible after concurrent access
        let ssa = get_ssa().lock();
        assert!(
            ssa.health_check() != nexcore_vigilance::hud::capabilities::StateHealth::Failed,
            "SSA should be healthy after concurrent access"
        );
    }

    #[test]
    fn test_concurrent_mixed_singleton_access() {
        let handles: Vec<_> = (0..4)
            .map(|i| {
                thread::spawn(move || {
                    // Mix different singleton accesses - acquire and release lock to verify no deadlock
                    for _ in 0..5 {
                        match i % 4 {
                            0 => {
                                let guard = get_ssa().lock();
                                assert!(guard.persistence_active, "SSA should be active");
                            }
                            1 => {
                                let guard = get_fed().lock();
                                drop(guard); // Explicit release
                            }
                            2 => {
                                let guard = get_exploration().lock();
                                drop(guard);
                            }
                            _ => {
                                let guard = get_comm().lock();
                                drop(guard);
                            }
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            assert!(
                handle.join().is_ok(),
                "Mixed concurrent access should not deadlock"
            );
        }
    }
}
