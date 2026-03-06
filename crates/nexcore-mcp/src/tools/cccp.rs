//! CCCP (Consultant's Client Care Process) MCP tools.
//!
//! Exposes the nexcore-cccp typed pipeline as MCP-callable operations.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use nexcore_cccp::assess::GapAnalysis;
use nexcore_cccp::follow_up::{Achievement, ObjectiveEvaluation, OutcomeEvaluation};
use nexcore_cccp::plan::EngagementPlan;
use nexcore_vigilance::caba::{DomainCategory, DomainStateVector, ProficiencyLevel};

use crate::params::cccp::{
    CccpEpaReadinessParams, CccpEvaluateParams, CccpGapAnalysisParams, CccpPhaseInfoParams,
    CccpPlanParams,
};

// ── Helpers ──────────────────────────────────────────────────────────────

fn level_from_u8(v: u8) -> ProficiencyLevel {
    match v {
        1 => ProficiencyLevel::L1Novice,
        2 => ProficiencyLevel::L2AdvancedBeginner,
        3 => ProficiencyLevel::L3Competent,
        4 => ProficiencyLevel::L4Proficient,
        5 => ProficiencyLevel::L5Expert,
        _ => ProficiencyLevel::L1Novice,
    }
}

fn levels_from_array(arr: &[u8; 15]) -> [ProficiencyLevel; 15] {
    let mut out = [ProficiencyLevel::L1Novice; 15];
    for (i, &v) in arr.iter().enumerate() {
        out[i] = level_from_u8(v);
    }
    out
}

fn ok_json(val: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&val).unwrap_or_else(|_| val.to_string()),
    )]))
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Compute a full gap analysis from current and desired proficiency levels.
pub fn cccp_gap_analysis(params: CccpGapAnalysisParams) -> Result<CallToolResult, McpError> {
    let current = DomainStateVector::new(levels_from_array(&params.current));
    let desired = DomainStateVector::new(levels_from_array(&params.desired));
    let analysis = GapAnalysis::compute(current, desired);

    let gaps: Vec<_> = analysis
        .domain_gaps
        .iter()
        .filter(|g| g.gap > 0)
        .map(|g| {
            json!({
                "domain": g.domain.as_str(),
                "current": g.current.as_str(),
                "desired": g.desired.as_str(),
                "gap": g.gap,
            })
        })
        .collect();

    let blocked_epas: Vec<_> = analysis
        .blocked_epas()
        .iter()
        .map(|e| {
            json!({
                "epa": format!("{:?}", e.epa),
                "threshold": e.threshold.as_str(),
                "blocking_domains": e.blocking_domains.iter().map(|d| d.as_str()).collect::<Vec<_>>(),
            })
        })
        .collect();

    let immature_cpas: Vec<_> = analysis
        .immature_cpas(1.0)
        .iter()
        .map(|c| {
            json!({
                "cpa": format!("{:?}", c.cpa),
                "readiness_score": format!("{:.0}%", c.readiness_score * 100.0),
                "blocking_epas": c.blocking_epas.iter().map(|e| format!("{:?}", e)).collect::<Vec<_>>(),
            })
        })
        .collect();

    ok_json(json!({
        "overall_readiness": format!("{:.0}%", analysis.overall_readiness * 100.0),
        "domains_with_gaps": gaps.len(),
        "priority_gaps": gaps,
        "blocked_epas_count": blocked_epas.len(),
        "blocked_epas": blocked_epas,
        "immature_cpas": immature_cpas,
    }))
}

/// Generate an engagement plan from current/desired proficiency levels.
pub fn cccp_plan(params: CccpPlanParams) -> Result<CallToolResult, McpError> {
    let current = DomainStateVector::new(levels_from_array(&params.current));
    let desired = DomainStateVector::new(levels_from_array(&params.desired));
    let analysis = GapAnalysis::compute(current, desired);
    let gaps = analysis.priority_gaps();
    let gap_refs: Vec<&nexcore_cccp::assess::DomainGap> = gaps;
    let plan = EngagementPlan::from_gaps(&gap_refs);

    let interventions: Vec<_> = plan
        .interventions
        .iter()
        .map(|i| {
            json!({
                "id": i.id,
                "description": i.description,
                "target_domains": i.target_domains.iter().map(|d| d.as_str()).collect::<Vec<_>>(),
                "target_level": i.target_level.as_str(),
                "priority": format!("{:?}", i.priority),
                "estimated_sessions": i.estimated_sessions,
            })
        })
        .collect();

    ok_json(json!({
        "scope": plan.scope.iter().map(|d| d.as_str()).collect::<Vec<_>>(),
        "total_interventions": plan.interventions.len(),
        "total_estimated_sessions": plan.total_estimated_sessions,
        "interventions": interventions,
    }))
}

/// Check EPA readiness given current proficiency levels.
pub fn cccp_epa_readiness(params: CccpEpaReadinessParams) -> Result<CallToolResult, McpError> {
    let current = DomainStateVector::new(levels_from_array(&params.current));
    // Use L5 Expert as desired to get full EPA readiness assessment
    let desired = DomainStateVector::new([ProficiencyLevel::L5Expert; 15]);
    let analysis = GapAnalysis::compute(current, desired);

    let ready: Vec<_> = analysis
        .epa_readiness
        .iter()
        .filter(|e| e.ready)
        .map(|e| {
            json!({
                "epa": format!("{:?}", e.epa),
                "threshold": e.threshold.as_str(),
            })
        })
        .collect();

    let blocked: Vec<_> = analysis
        .epa_readiness
        .iter()
        .filter(|e| !e.ready)
        .map(|e| {
            json!({
                "epa": format!("{:?}", e.epa),
                "threshold": e.threshold.as_str(),
                "blocking_domains": e.blocking_domains.iter().map(|d| d.as_str()).collect::<Vec<_>>(),
            })
        })
        .collect();

    ok_json(json!({
        "total_epas": analysis.epa_readiness.len(),
        "ready_count": ready.len(),
        "blocked_count": blocked.len(),
        "ready_epas": ready,
        "blocked_epas": blocked,
    }))
}

/// Compute outcome evaluation comparing initial, final, and desired states.
pub fn cccp_evaluate(params: CccpEvaluateParams) -> Result<CallToolResult, McpError> {
    let initial_current = DomainStateVector::new(levels_from_array(&params.initial));
    let desired = DomainStateVector::new(levels_from_array(&params.desired));
    let final_state = DomainStateVector::new(levels_from_array(&params.final_state));

    let initial_analysis = GapAnalysis::compute(initial_current, desired.clone());

    let objectives: Vec<ObjectiveEvaluation> = params
        .objectives
        .iter()
        .map(|o| {
            let achievement = match o.achievement {
                4 => Achievement::FullyAchieved,
                3 => Achievement::SubstantiallyAchieved,
                2 => Achievement::PartiallyAchieved,
                _ => Achievement::NotAchieved,
            };
            let domains: Vec<DomainCategory> = o
                .domains
                .iter()
                .filter_map(|&idx| DomainCategory::ALL.get(idx).copied())
                .collect();
            ObjectiveEvaluation {
                objective: o.objective.clone(),
                achievement,
                evidence: o.evidence.clone(),
                domains,
            }
        })
        .collect();

    let eval = OutcomeEvaluation::evaluate(initial_analysis, final_state, desired, objectives);

    let residual: Vec<_> = eval.residual_gaps.iter().map(|d| d.as_str()).collect();

    let obj_results: Vec<_> = eval
        .objectives
        .iter()
        .map(|o| {
            json!({
                "objective": o.objective,
                "achievement": format!("{:?}", o.achievement),
                "evidence": o.evidence,
            })
        })
        .collect();

    ok_json(json!({
        "gap_closure_rate": format!("{:.1}%", eval.gap_closure_rate * 100.0),
        "disposition": format!("{:?}", eval.disposition),
        "residual_gaps": residual,
        "objectives": obj_results,
    }))
}

/// Get CCCP phase information.
pub fn cccp_phase_info(params: CccpPhaseInfoParams) -> Result<CallToolResult, McpError> {
    use nexcore_cccp::engagement::Phase;

    let phase = match params.phase.to_lowercase().as_str() {
        "1" | "collect" => Phase::Collect,
        "2" | "assess" => Phase::Assess,
        "3" | "plan" => Phase::Plan,
        "4" | "implement" => Phase::Implement,
        "5" | "follow_up" | "followup" => Phase::FollowUp,
        other => {
            return Err(McpError::invalid_params(
                format!(
                    "Unknown phase: '{other}'. Use 1-5 or collect/assess/plan/implement/follow_up."
                ),
                None,
            ));
        }
    };

    ok_json(json!({
        "phase": format!("{:?}", phase),
        "number": phase.number(),
        "primary_algorithm": phase.primary_algorithm(),
        "typical_sessions": phase.typical_sessions(),
        "next_phase": phase.next().map(|p| format!("{:?}", p)),
        "templates": match phase {
            Phase::Collect => vec!["NV-COR-SOP-001 Diagnostic Assessment Tool"],
            Phase::Assess => vec!["NV-COR-SOP-002 Gap Analysis Matrix", "NV-COR-SOP-002 Strategic Assessment Report"],
            Phase::Plan => vec!["NV-COR-SOP-003 Strategic Engagement Plan", "NV-COR-SOP-003 Implementation Roadmap"],
            Phase::Implement => vec!["NV-COR-TRK-001 Implementation Tracker", "NV-COR-RPT-001 Status Report", "NV-COR-LOG-001 Issue Log"],
            Phase::FollowUp => vec!["NV-COR-EVL-001 Outcome Evaluation", "NV-COR-CLO-001 Transition/Closure Package"],
        },
    }))
}
