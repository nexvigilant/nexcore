//! Anti-vector MCP tools: classify, compute, and report countermeasures.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::antivector::{
    AntivectorClassifyParams, AntivectorComputeParams, AntivectorReportParams,
};

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn err_result(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

fn parse_harm_type(s: &str) -> Option<nexcore_harm_taxonomy::HarmTypeId> {
    use nexcore_harm_taxonomy::HarmTypeId;
    match s.to_uppercase().trim() {
        "A" => Some(HarmTypeId::A),
        "B" => Some(HarmTypeId::B),
        "C" => Some(HarmTypeId::C),
        "D" => Some(HarmTypeId::D),
        "E" => Some(HarmTypeId::E),
        "F" => Some(HarmTypeId::F),
        "G" => Some(HarmTypeId::G),
        "H" => Some(HarmTypeId::H),
        "I" => Some(HarmTypeId::I),
        _ => None,
    }
}

fn parse_bias_type(s: &str) -> Option<nexcore_antivector::BiasType> {
    use nexcore_antivector::BiasType;
    match s.to_lowercase().trim() {
        "indication" | "indication_bias" => Some(BiasType::IndicationBias),
        "notoriety" | "notoriety_bias" => Some(BiasType::NotorietyBias),
        "weber" | "weber_effect" => Some(BiasType::WeberEffect),
        "stimulated" | "stimulated_reporting" => Some(BiasType::StimulatedReporting),
        "channeling" | "channeling_bias" => Some(BiasType::ChannelingBias),
        "protopathic" | "protopathic_bias" => Some(BiasType::ProtopathicBias),
        "depletion" | "depletion_of_susceptibles" => Some(BiasType::DepletionOfSusceptibles),
        "duplicate" | "duplicate_reporting" => Some(BiasType::DuplicateReporting),
        _ => None,
    }
}

/// Classify a harm type into its anti-vector strategy.
pub fn antivector_classify(p: AntivectorClassifyParams) -> Result<CallToolResult, McpError> {
    let harm_type = match parse_harm_type(&p.harm_type) {
        Some(ht) => ht,
        None => return err_result("harm_type must be A-I"),
    };

    let strategy = nexcore_antivector::classify_anti_vector(harm_type);

    ok_json(json!({
        "harm_type": p.harm_type.to_uppercase(),
        "primary_class": format!("{:?}", strategy.primary_class),
        "secondary_class": strategy.secondary_class.map(|c| format!("{c:?}")),
        "description": strategy.description,
        "measures": strategy.measures.iter().map(|m| format!("{m:?}")).collect::<Vec<_>>(),
        "common_biases": strategy.common_biases.iter().map(|b| format!("{b:?}")).collect::<Vec<_>>(),
    }))
}

/// Compute a complete anti-vector for a harm vector.
pub fn antivector_compute(p: AntivectorComputeParams) -> Result<CallToolResult, McpError> {
    let harm_type = match parse_harm_type(&p.harm_type) {
        Some(ht) => ht,
        None => return err_result("harm_type must be A-I"),
    };

    let harm = nexcore_antivector::HarmVector {
        source: p.drug,
        target: p.event,
        harm_type,
        magnitude: p.magnitude.clamp(0.0, 1.0),
        confidence: p.confidence.clamp(0.0, 1.0),
        pathway: p.pathway,
    };

    let biases = match (p.bias_type.as_deref(), p.bias_magnitude) {
        (Some(bt_str), Some(mag)) => match parse_bias_type(bt_str) {
            Some(bt) => vec![nexcore_antivector::BiasAssessment {
                bias_type: bt,
                magnitude: mag.clamp(0.0, 1.0),
                evidence_source: nexcore_antivector::EvidenceSource::DatabaseAnalysis,
                description: format!("{bt_str} bias assessment"),
            }],
            None => {
                return err_result(
                    "Unknown bias_type. Use: indication, notoriety, weber, channeling, protopathic, depletion, stimulated, duplicate",
                );
            }
        },
        _ => vec![],
    };

    let av = nexcore_antivector::compute_anti_vector(&harm, &biases, None);

    ok_json(json!({
        "harm_vector": {
            "source": av.harm_vector.source,
            "target": av.harm_vector.target,
            "harm_type": format!("{:?}", av.harm_vector.harm_type),
            "magnitude": av.harm_vector.magnitude,
        },
        "anti_vector_magnitude": av.magnitude,
        "has_mechanistic": av.mechanistic.is_some(),
        "has_epistemic": av.epistemic.is_some(),
        "has_architectural": av.architectural.is_some(),
        "epistemic_verdict": av.epistemic.as_ref().map(|e| format!("{:?}", e.verdict)),
        "epistemic_residual": av.epistemic.as_ref().map(|e| e.residual_signal),
        "architectural_measure": av.architectural.as_ref().map(|a| format!("{:?}", a.measure)),
        "architectural_delta_ds": av.architectural.as_ref().map(|a| a.delta_safety_distance),
        "annihilation_result": match &av.annihilation_result {
            nexcore_antivector::AnnihilationResult::ResidualHarm { residual, gap } => json!({
                "outcome": "residual_harm",
                "residual": residual,
                "gap": gap,
            }),
            nexcore_antivector::AnnihilationResult::Annihilated { knowledge } => json!({
                "outcome": "annihilated",
                "knowledge": knowledge,
            }),
            nexcore_antivector::AnnihilationResult::SurplusProtection { surplus, disproportionate } => json!({
                "outcome": "surplus_protection",
                "surplus": surplus,
                "disproportionate": disproportionate,
            }),
        },
    }))
}

/// Generate an annihilation report for a drug-event pair.
pub fn antivector_report(p: AntivectorReportParams) -> Result<CallToolResult, McpError> {
    let harm_type = match parse_harm_type(&p.harm_type) {
        Some(ht) => ht,
        None => return err_result("harm_type must be A-I"),
    };

    let harm = nexcore_antivector::HarmVector {
        source: p.drug,
        target: p.event,
        harm_type,
        magnitude: p.magnitude.clamp(0.0, 1.0),
        confidence: p.confidence.clamp(0.0, 1.0),
        pathway: p.pathway,
    };

    let biases = match (p.bias_type.as_deref(), p.bias_magnitude) {
        (Some(bt_str), Some(mag)) => match parse_bias_type(bt_str) {
            Some(bt) => vec![nexcore_antivector::BiasAssessment {
                bias_type: bt,
                magnitude: mag.clamp(0.0, 1.0),
                evidence_source: nexcore_antivector::EvidenceSource::DatabaseAnalysis,
                description: format!("{bt_str} bias assessment"),
            }],
            None => return err_result("Unknown bias_type"),
        },
        _ => vec![],
    };

    let mechanistic = match (p.intervention.as_deref(), p.expected_attenuation) {
        (Some(intervention), Some(attenuation)) => {
            Some(nexcore_antivector::MechanisticAntiVector {
                pathway_target: harm
                    .pathway
                    .clone()
                    .unwrap_or_else(|| "unknown pathway".to_string()),
                intervention: intervention.to_string(),
                mechanism_of_action: format!("Targeted intervention: {intervention}"),
                expected_attenuation: attenuation.clamp(0.0, 1.0),
                evidence: vec![],
            })
        }
        _ => None,
    };

    let av = nexcore_antivector::compute_anti_vector(&harm, &biases, mechanistic);
    let report = nexcore_antivector::annihilation_report(&av);

    ok_json(json!({
        "drug": report.drug,
        "event": report.event,
        "harm_type": format!("{:?}", report.harm_type),
        "harm_magnitude": report.harm_magnitude,
        "anti_vector_magnitude": report.anti_vector_magnitude,
        "mechanistic": report.mechanistic_summary,
        "epistemic": report.epistemic_summary,
        "architectural": report.architectural_summary,
        "outcome": report.outcome,
    }))
}

/// Check whether an anti-vector is already deployed in the drug label.
pub fn antivector_label_check(
    p: crate::params::antivector::AntivectorLabelCheckParams,
) -> Result<CallToolResult, McpError> {
    let result = nexcore_antivector::check_label_deployment(
        &p.drug,
        &p.event,
        p.adr_section.as_deref(),
        p.warnings_section.as_deref(),
        p.boxed_warning.as_deref(),
    );

    ok_json(json!({
        "drug": result.drug,
        "event": result.event,
        "status": format!("{:?}", result.status),
        "event_in_adr_section": result.event_in_adr_section,
        "event_in_warnings": result.event_in_warnings,
        "event_in_boxed_warning": result.event_in_boxed_warning,
        "has_dose_guidance": result.has_dose_guidance,
        "has_monitoring": result.has_monitoring,
        "has_contraindication": result.has_contraindication,
        "has_rems": result.has_rems,
        "has_medication_guide": result.has_medication_guide,
        "deployed_measures": result.deployed_measures.iter().map(|m| format!("{m:?}")).collect::<Vec<_>>(),
        "recommendation": result.recommendation,
    }))
}
