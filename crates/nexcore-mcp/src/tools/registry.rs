//! Registry tools: skill ecosystem compliance assessment, promotion, and ToV monitoring.
//!
//! Exposes nexcore-registry capabilities as MCP tools for Claude Code.

use crate::params::{
    RegistryAssessAllParams, RegistryAssessSkillParams, RegistryGapReportParams,
    RegistryPromotableParams, RegistryPromotionPlanParams, RegistryTovHarmParams,
    RegistryTovIsSafeParams, RegistryTovSafetyParams,
};
use nexcore_registry::assess::{self, ComplianceTier};
use nexcore_registry::pool::RegistryPool;
use nexcore_registry::{promote, reports, tov};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Open the default registry pool, returning an MCP error result on failure.
fn open_pool() -> Result<RegistryPool, CallToolResult> {
    RegistryPool::open_default().map_err(|e| {
        CallToolResult::success(vec![Content::text(
            json!({ "error": format!("Cannot open registry: {e}") }).to_string(),
        )])
    })
}

/// Assess a single skill's compliance tier and SMST v2 score.
pub fn assess_skill(params: RegistryAssessSkillParams) -> Result<CallToolResult, McpError> {
    let pool = match open_pool() {
        Ok(p) => p,
        Err(r) => return Ok(r),
    };

    let result = pool.with_conn(|conn| assess::assess_skill(conn, &params.name));

    match result {
        Ok(assessment) => {
            let json = json!({
                "name": params.name,
                "compliance": assessment.compliance.as_str(),
                "smst_v2": assessment.smst_v2,
                "components": {
                    "input": assessment.components.input,
                    "output": assessment.components.output,
                    "logic": assessment.components.logic,
                    "error_handling": assessment.components.error_handling,
                    "examples": assessment.components.examples,
                    "references": assessment.components.references,
                    "total": assessment.components.total(),
                },
                "gaps": assessment.gaps.iter().map(|g| json!({
                    "field": g.field,
                    "target_tier": g.target_tier,
                    "reason": g.reason,
                })).collect::<Vec<_>>(),
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({ "error": format!("{e}") });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Assess all skills in the registry.
pub fn assess_all(params: RegistryAssessAllParams) -> Result<CallToolResult, McpError> {
    let pool = match open_pool() {
        Ok(p) => p,
        Err(r) => return Ok(r),
    };

    let result: Result<(Vec<assess::Assessment>, Option<usize>), _> = pool.with_conn(|conn| {
        let assessments = assess::assess_all(conn)?;

        if params.apply {
            let applied = assess::apply_assessments(conn, &assessments)?;
            Ok((assessments, Some(applied)))
        } else {
            Ok((assessments, None))
        }
    });

    match result {
        Ok((assessments, applied)) => {
            // Summarize tier distribution
            let mut tier_counts = std::collections::HashMap::<String, u32>::new();
            for a in &assessments {
                *tier_counts
                    .entry(a.compliance.as_str().to_string())
                    .or_insert(0) += 1;
            }

            let skills: Vec<_> = assessments
                .iter()
                .map(|a| {
                    json!({
                        "name": a.name,
                        "compliance": a.compliance.as_str(),
                        "smst_v2": a.smst_v2,
                        "gap_count": a.gaps.len(),
                    })
                })
                .collect();

            let mut json = json!({
                "total": assessments.len(),
                "tier_distribution": tier_counts,
                "skills": skills,
            });

            if let Some(count) = applied {
                json["applied"] = json!(count);
            }

            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({ "error": format!("{e}") });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Generate a gap report for the entire skill ecosystem.
pub fn gap_report(_params: RegistryGapReportParams) -> Result<CallToolResult, McpError> {
    let pool = match open_pool() {
        Ok(p) => p,
        Err(r) => return Ok(r),
    };

    let result = pool.with_conn(|conn| reports::generate_gap_report(conn));

    match result {
        Ok(report) => {
            let json = json!({
                "total_skills": report.total_skills,
                "coverage_pct": report.coverage_pct(),
                "total_gaps": report.total_gaps(),
                "missing_description": report.missing_description,
                "missing_tags": report.missing_tags,
                "missing_argument_hint": report.missing_argument_hint,
                "missing_allowed_tools": report.missing_allowed_tools,
                "missing_version": report.missing_version,
                "over_budget": report.over_budget.iter().map(|(n, c)| json!({"name": n, "chars": c})).collect::<Vec<_>>(),
                "over_line_limit": report.over_line_limit.iter().map(|(n, c)| json!({"name": n, "lines": c})).collect::<Vec<_>>(),
                "missing_compliance": report.missing_compliance,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({ "error": format!("{e}") });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Get skills eligible for promotion to a target tier.
pub fn promotable(params: RegistryPromotableParams) -> Result<CallToolResult, McpError> {
    let tier = match ComplianceTier::from_str_opt(&params.target_tier) {
        Some(t) => t,
        None => {
            let json = json!({
                "error": format!("Invalid tier: {}. Use: Invalid, Bronze, Silver, Gold, Platinum, Diamond", params.target_tier),
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };

    let pool = match open_pool() {
        Ok(p) => p,
        Err(r) => return Ok(r),
    };

    let result = pool.with_conn(|conn| promote::promotable_to(conn, tier));

    match result {
        Ok(names) => {
            let json = json!({
                "target_tier": params.target_tier,
                "promotable_count": names.len(),
                "skills": names,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({ "error": format!("{e}") });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Generate a promotion plan for a skill to reach a target tier.
pub fn promotion_plan(params: RegistryPromotionPlanParams) -> Result<CallToolResult, McpError> {
    let tier = match ComplianceTier::from_str_opt(&params.target_tier) {
        Some(t) => t,
        None => {
            let json = json!({
                "error": format!("Invalid tier: {}. Use: Invalid, Bronze, Silver, Gold, Platinum, Diamond", params.target_tier),
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };

    let pool = match open_pool() {
        Ok(p) => p,
        Err(r) => return Ok(r),
    };

    let result = pool.with_conn(|conn| promote::promotion_plan(conn, &params.name, tier));

    match result {
        Ok(plan) => {
            let actions: Vec<_> = plan
                .actions
                .iter()
                .map(|a| match a {
                    promote::PromotionAction::AddFrontmatterField { key, suggested_value } => {
                        json!({ "action": "add_field", "key": key, "suggested_value": suggested_value })
                    }
                    promote::PromotionAction::ReduceLineCount { current, target } => {
                        json!({ "action": "reduce_lines", "current": current, "target": target })
                    }
                    promote::PromotionAction::ReduceContentChars { current, target } => {
                        json!({ "action": "reduce_chars", "current": current, "target": target })
                    }
                    promote::PromotionAction::AddAgentPairing => {
                        json!({ "action": "add_agent_pairing" })
                    }
                    promote::PromotionAction::AddHooksDefinition => {
                        json!({ "action": "add_hooks" })
                    }
                    promote::PromotionAction::AddAllowedTools { suggested } => {
                        json!({ "action": "add_allowed_tools", "suggested": suggested })
                    }
                })
                .collect();

            let json = json!({
                "name": plan.name,
                "current_tier": plan.current_tier,
                "target_tier": plan.target_tier,
                "action_count": actions.len(),
                "actions": actions,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({ "error": format!("{e}") });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Compute the ToV safety distance d(s) for the ecosystem.
pub fn tov_safety(_params: RegistryTovSafetyParams) -> Result<CallToolResult, McpError> {
    let pool = match open_pool() {
        Ok(p) => p,
        Err(r) => return Ok(r),
    };

    let result = pool.with_conn(|conn| tov::safety_distance(conn));

    match result {
        Ok(ds) => {
            let json = json!({
                "safety_distance": ds,
                "axiom": "A4 (Safety Manifold)",
                "requirement": "d(s) > 0",
                "status": if ds > 0.0 { "SAFE" } else { "VIOLATION" },
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({ "error": format!("{e}") });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Compute all 8 harm type indicators (A-H).
pub fn tov_harm(_params: RegistryTovHarmParams) -> Result<CallToolResult, McpError> {
    let pool = match open_pool() {
        Ok(p) => p,
        Err(r) => return Ok(r),
    };

    let result = pool.with_conn(|conn| tov::harm_indicators(conn));

    match result {
        Ok(report) => {
            let json = json!({
                "harm_a_unguarded": report.harm_a_count,
                "harm_b_no_disable": report.harm_b_count,
                "harm_c_no_description": report.harm_c_count,
                "harm_d_over_budget": report.harm_d_count,
                "harm_e_stale": report.harm_e_count,
                "harm_f_broken_chains": report.harm_f_count,
                "harm_g_unrestricted_fork": report.harm_g_count,
                "harm_h_complexity": report.harm_h_count,
                "total_harm_score": report.total_harm_score,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({ "error": format!("{e}") });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}

/// Check if the ecosystem is in the safe manifold.
pub fn tov_is_safe(params: RegistryTovIsSafeParams) -> Result<CallToolResult, McpError> {
    let pool = match open_pool() {
        Ok(p) => p,
        Err(r) => return Ok(r),
    };

    let result = pool.with_conn(|conn| {
        let ds = tov::safety_distance(conn)?;
        let safe = tov::is_safe(conn, params.threshold)?;
        Ok((ds, safe))
    });

    match result {
        Ok((ds, safe)) => {
            let json = json!({
                "is_safe": safe,
                "safety_distance": ds,
                "threshold": params.threshold,
                "verdict": if safe { "Ecosystem is within safe manifold" } else { "WARNING: Ecosystem safety distance below threshold" },
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({ "error": format!("{e}") });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}
