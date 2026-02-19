//! FDA AI Credibility Assessment MCP Tools
//!
//! ## T1 Primitive Foundation
//!
//! | Tool | T1 Grounding | FDA Step |
//! |------|--------------|----------|
//! | fda_define_cou | λ+μ (Location+Mapping) | Step 2 |
//! | fda_assess_risk | κ×N (Comparison×Quantity) | Step 3 |
//! | fda_create_plan | σ (Sequence) | Step 4 |
//! | fda_validate_evidence | ∃+κ (Existence+Comparison) | Step 5-6 |
//! | fda_decide_adequacy | κ (Comparison) | Step 7 |

use nexcore_vigilance::fda::{
    AdequacyDecision, AssessmentStep, ContextOfUse, CredibilityEvidence, CredibilityPlan,
    DecisionConsequence, DecisionQuestion, EvidenceIntegration, EvidenceQuality, EvidenceType,
    FitForUse, ModelInfluence, ModelPurpose, ModelRisk, PlanStatus, RegulatoryContext, Relevance,
    Reliability, RiskLevel,
};
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Step 2: Define Context of Use
///
/// T1 Grounding: λ (Location) + μ (Mapping)
pub fn fda_define_cou(
    question: &str,
    input_domain: &str,
    output_domain: &str,
    purpose_description: &str,
    integration: &str,
    confirmatory_sources: Option<Vec<String>>,
    regulatory_context: &str,
) -> CallToolResult {
    // Validate question
    let decision_question = match DecisionQuestion::new(question) {
        Ok(q) => q,
        Err(e) => {
            return error_result(&format!("Invalid question: {}", e));
        }
    };

    // Create model purpose
    let purpose = match ModelPurpose::new(input_domain, output_domain, purpose_description) {
        Ok(p) => p,
        Err(e) => {
            return error_result(&format!("Invalid purpose: {}", e));
        }
    };

    // Parse evidence integration
    let evidence_integration = match integration.to_lowercase().as_str() {
        "sole" => EvidenceIntegration::Sole,
        "primary" => EvidenceIntegration::Primary {
            confirmatory: confirmatory_sources.clone().unwrap_or_default(),
        },
        "contributory" => EvidenceIntegration::Contributory {
            other_sources: confirmatory_sources.clone().unwrap_or_default(),
        },
        "supplementary" => EvidenceIntegration::Supplementary {
            primary_source: confirmatory_sources
                .and_then(|v| v.into_iter().next())
                .unwrap_or_else(|| "Not specified".to_string()),
        },
        _ => {
            return error_result(
                "Invalid integration type. Use: sole, primary, contributory, supplementary",
            );
        }
    };

    // Parse regulatory context
    let reg_context = match regulatory_context.to_lowercase().as_str() {
        "ind" => RegulatoryContext::Ind,
        "nda" => RegulatoryContext::Nda,
        "bla" => RegulatoryContext::Bla,
        "postmarket" => RegulatoryContext::Postmarket,
        "manufacturing" => RegulatoryContext::Manufacturing,
        _ => RegulatoryContext::Other,
    };

    // Create COU
    let cou = ContextOfUse::new(
        decision_question,
        purpose,
        evidence_integration,
        reg_context,
    );

    // Validate
    if let Err(e) = cou.validate() {
        return error_result(&format!("COU validation failed: {}", e));
    }

    success_json(json!({
        "status": "success",
        "step": "Step 2: Define Context of Use",
        "t1_grounding": "λ (Location) + μ (Mapping)",
        "cou": {
            "question": cou.question().as_str(),
            "purpose": {
                "input_domain": cou.purpose().input_domain(),
                "output_domain": cou.purpose().output_domain(),
                "description": cou.purpose().description()
            },
            "integration": format!("{:?}", cou.integration()),
            "evidence_count": cou.integration().evidence_count(),
            "is_primary": cou.integration().is_primary(),
            "regulatory_context": cou.regulatory_context().to_string()
        }
    }))
}

/// Step 3: Assess AI Model Risk
///
/// T1 Grounding: κ (Comparison) × N (Quantity)
pub fn fda_assess_risk(influence: &str, consequence: &str) -> CallToolResult {
    // Parse model influence
    let model_influence = match influence.to_lowercase().as_str() {
        "high" => ModelInfluence::High,
        "medium" => ModelInfluence::Medium,
        "low" => ModelInfluence::Low,
        _ => {
            return error_result("Invalid influence. Use: high, medium, low");
        }
    };

    // Parse decision consequence
    let decision_consequence = match consequence.to_lowercase().as_str() {
        "high" => DecisionConsequence::High,
        "medium" => DecisionConsequence::Medium,
        "low" => DecisionConsequence::Low,
        _ => {
            return error_result("Invalid consequence. Use: high, medium, low");
        }
    };

    // Compute risk via matrix
    let risk = ModelRisk::new(model_influence, decision_consequence);
    let level = risk.level();
    let min_evidence = level.min_evidence_count();

    success_json(json!({
        "status": "success",
        "step": "Step 3: Assess Risk",
        "t1_grounding": "κ (Comparison) × N (Quantity)",
        "risk_assessment": {
            "model_influence": risk.influence().to_string(),
            "decision_consequence": risk.consequence().to_string(),
            "risk_level": level.to_string(),
            "minimum_evidence_required": min_evidence,
            "matrix_formula": "Risk = f(ModelInfluence, DecisionConsequence)"
        },
        "thresholds": {
            "LOW": "AUC ≥0.65, 2 evidence items",
            "MEDIUM": "AUC ≥0.75, 4 evidence items",
            "HIGH": "AUC ≥0.85, 8 evidence items + external validation"
        }
    }))
}

/// Step 4: Create Credibility Plan
///
/// T1 Grounding: σ (Sequence)
pub fn fda_create_plan(
    question: &str,
    input_domain: &str,
    output_domain: &str,
    influence: &str,
    consequence: &str,
    regulatory_context: &str,
) -> CallToolResult {
    // Build COU
    let decision_question = match DecisionQuestion::new(question) {
        Ok(q) => q,
        Err(e) => {
            return error_result(&format!("Invalid question: {}", e));
        }
    };

    let purpose = match ModelPurpose::new(input_domain, output_domain, "AI model transformation") {
        Ok(p) => p,
        Err(e) => {
            return error_result(&format!("Invalid purpose: {}", e));
        }
    };

    let reg_context = match regulatory_context.to_lowercase().as_str() {
        "ind" => RegulatoryContext::Ind,
        "nda" => RegulatoryContext::Nda,
        "bla" => RegulatoryContext::Bla,
        "postmarket" => RegulatoryContext::Postmarket,
        "manufacturing" => RegulatoryContext::Manufacturing,
        _ => RegulatoryContext::Other,
    };

    let cou = ContextOfUse::new(
        decision_question,
        purpose,
        EvidenceIntegration::Sole, // Default for planning
        reg_context,
    );

    // Build risk
    let model_influence = match influence.to_lowercase().as_str() {
        "high" => ModelInfluence::High,
        "medium" => ModelInfluence::Medium,
        _ => ModelInfluence::Low,
    };

    let decision_consequence = match consequence.to_lowercase().as_str() {
        "high" => DecisionConsequence::High,
        "medium" => DecisionConsequence::Medium,
        _ => DecisionConsequence::Low,
    };

    let risk = ModelRisk::new(model_influence, decision_consequence);

    // Create plan
    let plan = CredibilityPlan::new(cou, risk);
    let steps = AssessmentStep::all_steps();

    success_json(json!({
        "status": "success",
        "step": "Step 4: Develop Plan",
        "t1_grounding": "σ (Sequence)",
        "plan": {
            "status": plan.status().to_string(),
            "current_step": plan.current_step().to_string(),
            "risk_level": plan.model_risk().level().to_string(),
            "evidence_required": plan.model_risk().level().min_evidence_count(),
            "completion_percentage": plan.completion_percentage()
        },
        "assessment_steps": steps.iter().map(|s| {
            json!({
                "number": s.number(),
                "name": s.to_string()
            })
        }).collect::<Vec<_>>(),
        "required_activities": get_required_activities(plan.model_risk().level())
    }))
}

/// Step 5-6: Validate Evidence
///
/// T1 Grounding: ∃ (Existence) + κ (Comparison)
pub fn fda_validate_evidence(
    evidence_type: &str,
    quality: &str,
    description: &str,
    relevant: bool,
    reliable: bool,
    representative: bool,
) -> CallToolResult {
    // Parse evidence type
    let ev_type = match evidence_type.to_lowercase().as_str() {
        "validation_metrics" | "metrics" => EvidenceType::ValidationMetrics,
        "test_results" | "test" => EvidenceType::TestResults,
        "training_data" | "data" => EvidenceType::TrainingData,
        "architecture" | "arch" => EvidenceType::Architecture,
        "bias_analysis" | "bias" => EvidenceType::BiasAnalysis,
        "explainability" | "explain" => EvidenceType::Explainability,
        "prior_knowledge" | "literature" => EvidenceType::PriorKnowledge,
        "precedent" => EvidenceType::Precedent,
        other => EvidenceType::Other(other.to_string()),
    };

    // Parse quality
    let ev_quality = match quality.to_lowercase().as_str() {
        "high" => EvidenceQuality::High,
        "medium" => EvidenceQuality::Medium,
        _ => EvidenceQuality::Low,
    };

    // Create evidence
    let evidence = CredibilityEvidence::new(ev_type, ev_quality, description);

    // Create fit-for-use assessment using the actual Relevance/Reliability types
    // Map simple bools to detailed criteria (assume complete coverage if relevant/reliable)
    let relevance = Relevance::new(relevant, representative, true);
    let reliability = Reliability::new(reliable, reliable, reliable);
    let fit = FitForUse::new(relevance, reliability);
    let is_adequate = fit.is_adequate();

    success_json(json!({
        "status": "success",
        "step": "Step 5-6: Execute & Document",
        "t1_grounding": "∃ (Existence) + κ (Comparison)",
        "evidence": {
            "type": evidence.evidence_type().to_string(),
            "quality": format!("{:?}", evidence.quality()),
            "quality_score": evidence.quality().score(),
            "description": evidence.description()
        },
        "fit_for_use": {
            "relevance": {
                "has_key_elements": relevant,
                "is_representative": representative,
                "is_temporally_appropriate": true
            },
            "reliability": {
                "is_accurate": reliable,
                "is_complete": reliable,
                "is_traceable": reliable
            },
            "total_score": fit.total_score(),
            "is_adequate": is_adequate
        },
        "validation_result": if is_adequate { "PASS" } else { "FAIL" }
    }))
}

/// Step 7: Determine Adequacy
///
/// T1 Grounding: κ (Comparison)
pub fn fda_decide_adequacy(
    risk_level: &str,
    high_quality_evidence_count: usize,
    fit_for_use_passed: bool,
    critical_drift_detected: bool,
) -> CallToolResult {
    // Parse risk level
    let level = match risk_level.to_lowercase().as_str() {
        "high" => RiskLevel::High,
        "medium" => RiskLevel::Medium,
        _ => RiskLevel::Low,
    };

    let min_required = level.min_evidence_count();
    let has_sufficient_evidence = high_quality_evidence_count >= min_required;

    // Determine adequacy
    let decision = if !has_sufficient_evidence {
        AdequacyDecision::Inadequate {
            reason: format!(
                "Insufficient high-quality evidence: {} < {}",
                high_quality_evidence_count, min_required
            ),
        }
    } else if !fit_for_use_passed {
        AdequacyDecision::Inadequate {
            reason: "Data failed fit-for-use assessment".to_string(),
        }
    } else if critical_drift_detected {
        AdequacyDecision::Inadequate {
            reason: "Critical drift detected — revalidation required".to_string(),
        }
    } else {
        AdequacyDecision::Adequate
    };

    let status = if decision.is_adequate() {
        PlanStatus::Approved
    } else {
        PlanStatus::NeedsRevision
    };

    success_json(json!({
        "status": "success",
        "step": "Step 7: Determine Adequacy",
        "t1_grounding": "κ (Comparison)",
        "inputs": {
            "risk_level": level.to_string(),
            "high_quality_evidence_count": high_quality_evidence_count,
            "minimum_required": min_required,
            "fit_for_use_passed": fit_for_use_passed,
            "critical_drift_detected": critical_drift_detected
        },
        "decision": decision.to_string(),
        "is_adequate": decision.is_adequate(),
        "plan_status": status.to_string(),
        "next_action": if decision.is_adequate() {
            "Proceed with regulatory submission"
        } else {
            "Address gaps and resubmit for assessment"
        }
    }))
}

/// Helper: Get required activities by risk level
fn get_required_activities(level: RiskLevel) -> serde_json::Value {
    match level {
        RiskLevel::High => json!([
            "Independent test set validation",
            "Prospective validation study",
            "External audit",
            "Real-time drift monitoring",
            "Subgroup analysis (age, sex, race)",
            "Bias assessment"
        ]),
        RiskLevel::Medium => json!([
            "Independent test set validation",
            "Internal validation",
            "Periodic review",
            "Subgroup analysis"
        ]),
        RiskLevel::Low => json!(["Hold-out test set", "Documentation", "Annual review"]),
    }
}

/// Helper: Create success result from JSON
fn success_json(value: serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string()),
    )])
}

/// Helper: Create error result
fn error_result(message: &str) -> CallToolResult {
    CallToolResult::error(vec![Content::text(message)])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to extract text from CallToolResult content
    fn get_text(result: &CallToolResult) -> String {
        result
            .content
            .first()
            .map(|c| match &c.raw {
                rmcp::model::RawContent::Text(t) => t.text.clone(),
                _ => String::new(),
            })
            .unwrap_or_default()
    }

    #[test]
    fn test_fda_define_cou() {
        let result = fda_define_cou(
            "Is drug X safe for elderly patients?",
            "Patient demographics + AE reports",
            "Safety signal scores",
            "PRR/ROR signal detection",
            "primary",
            Some(vec!["Clinical trials".into(), "Literature".into()]),
            "postmarket",
        );
        let text = get_text(&result);
        assert!(text.contains("success"), "Should contain success: {}", text);
        assert!(text.contains("Step 2"), "Should contain Step 2: {}", text);
    }

    #[test]
    fn test_fda_assess_risk_high() {
        let result = fda_assess_risk("high", "high");
        let text = get_text(&result);
        assert!(text.contains("High"), "Should contain High: {}", text);
        assert!(
            text.contains("8"),
            "Should contain 8 (min evidence): {}",
            text
        );
    }

    #[test]
    fn test_fda_assess_risk_low() {
        let result = fda_assess_risk("low", "low");
        let text = get_text(&result);
        assert!(text.contains("Low"), "Should contain Low: {}", text);
    }

    #[test]
    fn test_fda_create_plan() {
        let result = fda_create_plan(
            "Should we approve this AI model?",
            "MRI images",
            "Tumor classification",
            "medium",
            "high",
            "nda",
        );
        let text = get_text(&result);
        assert!(text.contains("Step 4"), "Should contain Step 4: {}", text);
        assert!(
            text.contains("assessment_steps"),
            "Should contain assessment_steps: {}",
            text
        );
    }

    #[test]
    fn test_fda_validate_evidence_pass() {
        let result = fda_validate_evidence(
            "validation_metrics",
            "high",
            "ROC AUC = 0.92 (95% CI: 0.89-0.95)",
            true,
            true,
            true,
        );
        let text = get_text(&result);
        assert!(text.contains("PASS"), "Should contain PASS: {}", text);
    }

    #[test]
    fn test_fda_validate_evidence_fail() {
        let result = fda_validate_evidence(
            "test_results",
            "low",
            "Basic unit tests",
            false, // not relevant
            true,
            true,
        );
        let text = get_text(&result);
        assert!(text.contains("FAIL"), "Should contain FAIL: {}", text);
    }

    #[test]
    fn test_fda_decide_adequacy_pass() {
        let result = fda_decide_adequacy("high", 10, true, false);
        let text = get_text(&result);
        assert!(
            text.contains("ADEQUATE"),
            "Should contain ADEQUATE: {}",
            text
        );
        assert!(
            text.contains("Approved"),
            "Should contain Approved: {}",
            text
        );
    }

    #[test]
    fn test_fda_decide_adequacy_insufficient_evidence() {
        let result = fda_decide_adequacy("high", 3, true, false);
        let text = get_text(&result);
        assert!(
            text.contains("INADEQUATE"),
            "Should contain INADEQUATE: {}",
            text
        );
        assert!(
            text.contains("Insufficient"),
            "Should contain Insufficient: {}",
            text
        );
    }

    #[test]
    fn test_fda_decide_adequacy_drift() {
        let result = fda_decide_adequacy("medium", 6, true, true);
        let text = get_text(&result);
        assert!(
            text.contains("INADEQUATE"),
            "Should contain INADEQUATE: {}",
            text
        );
        assert!(text.contains("drift"), "Should contain drift: {}", text);
    }
}
