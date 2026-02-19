//! PV Core tools — IVF axiom assessment, severity classification
//!
//! Unique modules from nexcore-pv-core not already exposed through pv.rs/vigilance.rs.

use crate::params::{IvfAssessParams, IvfAxiomsParams, SeverityAssessParams};
use nexcore_pv_core::classification::{SeverityCriteria, full_assessment};
use nexcore_pv_core::ivf::{InterventionCharacteristics, IvfAxiom, assess_ivf_axioms};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Assess all 5 IVF axioms for an intervention (ToV §35)
pub fn ivf_assess(params: IvfAssessParams) -> Result<CallToolResult, McpError> {
    let characteristics = InterventionCharacteristics::new()
        .with_potency(params.potency)
        .with_emergence_uncertainty(params.emergence_uncertainty)
        .with_vulnerability_exposure(params.vulnerability_exposure)
        .with_deployment_scale(params.deployment_scale)
        .with_testing_completeness(params.testing_completeness);

    let assessment = assess_ivf_axioms(&characteristics);

    let axiom_results: Vec<_> = assessment
        .axiom_results
        .iter()
        .map(|r| {
            json!({
                "axiom": format!("{}", r.axiom),
                "level": format!("{}", r.level),
                "risk_score": (r.risk_score * 100.0).round() / 100.0,
                "rationale": r.rationale,
                "requires_vigilance": r.level.requires_vigilance(),
            })
        })
        .collect();

    let json = json!({
        "overall_risk": (assessment.overall_risk * 100.0).round() / 100.0,
        "vigilance_required": assessment.vigilance_required,
        "monitoring_intensity": format!("{}", assessment.monitoring_intensity),
        "axiom_results": axiom_results,
        "inputs": {
            "potency": params.potency,
            "emergence_uncertainty": params.emergence_uncertainty,
            "vulnerability_exposure": params.vulnerability_exposure,
            "deployment_scale": params.deployment_scale,
            "testing_completeness": params.testing_completeness,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// List the 5 IVF axioms with ToV mappings and formal statements
pub fn ivf_axioms(_params: IvfAxiomsParams) -> Result<CallToolResult, McpError> {
    let axioms: Vec<_> = IvfAxiom::all()
        .iter()
        .map(|a| {
            json!({
                "number": a.number(),
                "name": format!("{a}"),
                "statement": a.statement(),
                "tov_mapping": a.tov_mapping(),
            })
        })
        .collect();

    let json = json!({
        "axioms": axioms,
        "count": 5,
        "framework": "Intervention Vigilance Framework (ToV §35)",
        "note": "Generalizes pharmacovigilance methodology to all intervention domains",
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Assess adverse event severity using Hartwig-Siegel scale (levels 1-7)
pub fn severity_assess(params: SeverityAssessParams) -> Result<CallToolResult, McpError> {
    let mut criteria = SeverityCriteria::new();

    if params.treatment_changed {
        criteria = criteria.with_treatment_change();
    }
    if params.antidote_required {
        criteria = criteria.with_antidote();
    }
    if params.hospitalization_required {
        criteria = criteria.with_hospitalization();
    }
    if params.icu_required {
        criteria = criteria.with_icu();
    }
    if params.permanent_harm {
        criteria = criteria.with_permanent_harm();
    }
    if params.death {
        criteria = criteria.with_death();
    }

    let result = full_assessment(&criteria);

    let json = json!({
        "level": result.level.level(),
        "level_name": format!("{:?}", result.level),
        "category": format!("{:?}", result.category),
        "is_serious": result.is_serious,
        "priority_weight": result.priority_weight,
        "description": result.level.description(),
        "clinical_action": result.level.clinical_action(),
        "criteria_met": {
            "treatment_changed": params.treatment_changed,
            "antidote_required": params.antidote_required,
            "hospitalization_required": params.hospitalization_required,
            "icu_required": params.icu_required,
            "permanent_harm": params.permanent_harm,
            "death": params.death,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{IvfAssessParams, IvfAxiomsParams, SeverityAssessParams};

    fn extract_json(result: &CallToolResult) -> serde_json::Value {
        let content = &result.content[0];
        let text = content.as_text().expect("expected text content");
        serde_json::from_str(&text.text).expect("valid JSON")
    }

    #[test]
    fn ivf_assess_high_risk() {
        let r = ivf_assess(IvfAssessParams {
            potency: 0.95,
            emergence_uncertainty: 0.9,
            vulnerability_exposure: 0.9,
            deployment_scale: 0.9,
            testing_completeness: 0.1,
        })
        .expect("ok");
        let j = extract_json(&r);
        let risk = j["overall_risk"].as_f64().expect("f64");
        assert!(risk > 0.5, "high-risk → high overall_risk: {risk}");
        assert!(j["vigilance_required"].as_bool().expect("bool"));
    }

    #[test]
    fn ivf_assess_low_risk() {
        let r = ivf_assess(IvfAssessParams {
            potency: 0.1,
            emergence_uncertainty: 0.1,
            vulnerability_exposure: 0.1,
            deployment_scale: 0.1,
            testing_completeness: 0.9,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["overall_risk"].as_f64().expect("f64") < 0.5);
    }

    #[test]
    fn ivf_assess_all_zero() {
        let r = ivf_assess(IvfAssessParams {
            potency: 0.0,
            emergence_uncertainty: 0.0,
            vulnerability_exposure: 0.0,
            deployment_scale: 0.0,
            testing_completeness: 0.0,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["overall_risk"].as_f64().is_some());
        assert!(j["axiom_results"].as_array().is_some());
    }

    #[test]
    fn ivf_assess_all_one() {
        let r = ivf_assess(IvfAssessParams {
            potency: 1.0,
            emergence_uncertainty: 1.0,
            vulnerability_exposure: 1.0,
            deployment_scale: 1.0,
            testing_completeness: 1.0,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["axiom_results"].as_array().expect("arr").len(), 5);
    }

    #[test]
    fn ivf_assess_returns_five_axioms() {
        let r = ivf_assess(IvfAssessParams {
            potency: 0.5,
            emergence_uncertainty: 0.5,
            vulnerability_exposure: 0.5,
            deployment_scale: 0.5,
            testing_completeness: 0.5,
        })
        .expect("ok");
        let j = extract_json(&r);
        let axioms = j["axiom_results"].as_array().expect("arr");
        assert_eq!(axioms.len(), 5);
        for a in axioms {
            assert!(a["axiom"].as_str().is_some());
            assert!(a["level"].as_str().is_some());
            assert!(a["risk_score"].as_f64().is_some());
        }
    }

    #[test]
    fn ivf_axioms_lists_five() {
        let r = ivf_axioms(IvfAxiomsParams {}).expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["count"], 5);
        let axioms = j["axioms"].as_array().expect("arr");
        assert_eq!(axioms.len(), 5);
        for a in axioms {
            assert!(a["number"].as_u64().is_some());
            assert!(a["name"].as_str().is_some());
            assert!(a["statement"].as_str().is_some());
            assert!(a["tov_mapping"].as_str().is_some());
        }
    }

    #[test]
    fn ivf_axioms_framework() {
        let r = ivf_axioms(IvfAxiomsParams {}).expect("ok");
        let j = extract_json(&r);
        assert!(
            j["framework"]
                .as_str()
                .expect("str")
                .contains("Intervention Vigilance")
        );
    }

    #[test]
    fn severity_all_false_is_mild() {
        let r = severity_assess(SeverityAssessParams {
            treatment_changed: false,
            antidote_required: false,
            hospitalization_required: false,
            icu_required: false,
            permanent_harm: false,
            death: false,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["level"].as_u64().expect("u64"), 1);
        assert!(!j["is_serious"].as_bool().expect("bool"));
    }

    #[test]
    fn severity_death_is_lethal() {
        let r = severity_assess(SeverityAssessParams {
            treatment_changed: false,
            antidote_required: false,
            hospitalization_required: false,
            icu_required: false,
            permanent_harm: false,
            death: true,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["level"].as_u64().expect("u64"), 7);
        assert!(j["is_serious"].as_bool().expect("bool"));
    }

    #[test]
    fn severity_hospitalization() {
        let r = severity_assess(SeverityAssessParams {
            treatment_changed: false,
            antidote_required: false,
            hospitalization_required: true,
            icu_required: false,
            permanent_harm: false,
            death: false,
        })
        .expect("ok");
        let j = extract_json(&r);
        let lvl = j["level"].as_u64().expect("u64");
        assert!(lvl >= 3 && lvl <= 4, "hospitalization → 3-4: {lvl}");
        assert!(j["is_serious"].as_bool().expect("bool"));
    }

    #[test]
    fn severity_icu() {
        let r = severity_assess(SeverityAssessParams {
            treatment_changed: false,
            antidote_required: false,
            hospitalization_required: false,
            icu_required: true,
            permanent_harm: false,
            death: false,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["level"].as_u64().expect("u64") >= 5);
    }

    #[test]
    fn severity_permanent_harm() {
        let r = severity_assess(SeverityAssessParams {
            treatment_changed: false,
            antidote_required: false,
            hospitalization_required: false,
            icu_required: false,
            permanent_harm: true,
            death: false,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["level"].as_u64().expect("u64") >= 6);
    }

    #[test]
    fn severity_all_true_death_wins() {
        let r = severity_assess(SeverityAssessParams {
            treatment_changed: true,
            antidote_required: true,
            hospitalization_required: true,
            icu_required: true,
            permanent_harm: true,
            death: true,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["level"].as_u64().expect("u64"), 7);
    }

    #[test]
    fn severity_treatment_change_only() {
        let r = severity_assess(SeverityAssessParams {
            treatment_changed: true,
            antidote_required: false,
            hospitalization_required: false,
            icu_required: false,
            permanent_harm: false,
            death: false,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert_eq!(j["level"].as_u64().expect("u64"), 2);
    }

    #[test]
    fn severity_output_fields_complete() {
        let r = severity_assess(SeverityAssessParams {
            treatment_changed: true,
            antidote_required: false,
            hospitalization_required: true,
            icu_required: false,
            permanent_harm: false,
            death: false,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["level"].as_u64().is_some());
        assert!(j["level_name"].as_str().is_some());
        assert!(j["category"].as_str().is_some());
        assert!(j["is_serious"].is_boolean());
        assert!(j["priority_weight"].as_f64().is_some());
        assert!(j["description"].as_str().is_some());
        assert!(j["clinical_action"].as_str().is_some());
        assert!(j["criteria_met"].is_object());
    }
}
