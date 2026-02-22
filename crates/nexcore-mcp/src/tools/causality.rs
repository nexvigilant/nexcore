//! Causality Assessment Tools
//!
//! RUCAM hepatotoxicity and UCAS universal causality assessment MCP tools.

use crate::params::causality::{
    CriterionResponseParam, RucamParams, RucamReactionType, RechallengeResultParam,
    SerologyResultParam, UcasParams, YesNoNaParam,
};
use crate::tooling::attach_forensic_meta;
use nexcore_pv_core::causality::rucam::{
    self, AlternativeCauses, ConcomitantDrugs, PreviousHepatotoxicity, ReactionType,
    RechallengeResult, RucamInput, SerologyResult, YesNoNa, calculate_rucam,
};
use nexcore_pv_core::causality::ucas::{CriterionResponse, UcasInput, calculate_ucas};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// =============================================================================
// Conversion helpers
// =============================================================================

fn to_reaction_type(rt: &RucamReactionType) -> ReactionType {
    match rt {
        RucamReactionType::Hepatocellular => ReactionType::Hepatocellular,
        RucamReactionType::Cholestatic => ReactionType::Cholestatic,
        RucamReactionType::Mixed => ReactionType::Mixed,
    }
}

fn to_serology(s: &SerologyResultParam) -> SerologyResult {
    match s {
        SerologyResultParam::Positive => SerologyResult::Positive,
        SerologyResultParam::Negative => SerologyResult::Negative,
        SerologyResultParam::NotDone => SerologyResult::NotDone,
    }
}

fn to_yes_no_na(y: &YesNoNaParam) -> YesNoNa {
    match y {
        YesNoNaParam::Yes => YesNoNa::Yes,
        YesNoNaParam::No => YesNoNa::No,
        YesNoNaParam::NotApplicable => YesNoNa::NotApplicable,
    }
}

fn to_rechallenge_result(r: &RechallengeResultParam) -> RechallengeResult {
    match r {
        RechallengeResultParam::Positive => RechallengeResult::Positive,
        RechallengeResultParam::Negative => RechallengeResult::Negative,
        RechallengeResultParam::NotConclusive => RechallengeResult::NotConclusive,
    }
}

fn to_criterion_response(c: &CriterionResponseParam) -> CriterionResponse {
    match c {
        CriterionResponseParam::Yes => CriterionResponse::Yes,
        CriterionResponseParam::No => CriterionResponse::No,
        CriterionResponseParam::Unknown => CriterionResponse::Unknown,
    }
}

// =============================================================================
// RUCAM Tool
// =============================================================================

/// RUCAM hepatotoxicity causality assessment.
///
/// Scores drug-induced liver injury (DILI) causality across 7 areas:
/// temporal relationship, course of reaction, risk factors, concomitant drugs,
/// alternative causes, previous hepatotoxicity info, and rechallenge.
/// Score range: -4 to +14.
pub fn causality_rucam(params: RucamParams) -> Result<CallToolResult, McpError> {
    let input = RucamInput {
        time_to_onset: params.time_to_onset,
        reaction_type: to_reaction_type(&params.reaction_type),
        drug_withdrawn: params.drug_withdrawn,
        time_to_improvement: params.time_to_improvement,
        percentage_decrease: params.percentage_decrease,
        age: params.age,
        alcohol: params.alcohol,
        pregnancy: params.pregnancy,
        concomitant_drugs: ConcomitantDrugs {
            hepatotoxic_count: params.concomitant_drugs.hepatotoxic_count,
            interactions: params.concomitant_drugs.interactions,
        },
        alternative_causes: AlternativeCauses {
            hepatitis_a: to_serology(&params.alternative_causes.hepatitis_a),
            hepatitis_b: to_serology(&params.alternative_causes.hepatitis_b),
            hepatitis_c: to_serology(&params.alternative_causes.hepatitis_c),
            cmv_ebv: to_serology(&params.alternative_causes.cmv_ebv),
            biliary_sonography: to_serology(&params.alternative_causes.biliary_sonography),
            alcoholism: to_yes_no_na(&params.alternative_causes.alcoholism),
            underlying_complications: to_yes_no_na(
                &params.alternative_causes.underlying_complications,
            ),
        },
        previous_hepatotoxicity: PreviousHepatotoxicity {
            labeled_hepatotoxic: params.previous_hepatotoxicity.labeled_hepatotoxic,
            published_reports: params.previous_hepatotoxicity.published_reports,
            reaction_known: params.previous_hepatotoxicity.reaction_known,
        },
        rechallenge_performed: params.rechallenge_performed,
        rechallenge_result: params.rechallenge_result.as_ref().map(to_rechallenge_result),
    };

    let result = calculate_rucam(&input);

    let json_val = json!({
        "total_score": result.total_score,
        "category": result.category.to_string(),
        "confidence": result.confidence,
        "breakdown": {
            "temporal_relationship": result.breakdown.temporal_relationship,
            "course_of_reaction": result.breakdown.course_of_reaction,
            "risk_factors": result.breakdown.risk_factors,
            "concomitant_drugs": result.breakdown.concomitant_drugs,
            "alternative_causes": result.breakdown.alternative_causes,
            "previous_information": result.breakdown.previous_information,
            "rechallenge": result.breakdown.rechallenge,
        },
        "interpretation": match result.category {
            rucam::RucamCategory::HighlyProbable => "Score >=8: Drug causation highly probable",
            rucam::RucamCategory::Probable => "Score 6-7: Drug causation probable",
            rucam::RucamCategory::Possible => "Score 3-5: Drug causation possible",
            rucam::RucamCategory::Unlikely => "Score 1-2: Drug causation unlikely",
            rucam::RucamCategory::Excluded => "Score <=0: Drug causation excluded",
        },
    });

    let confidence = result.confidence;
    let is_causal = result.total_score >= 6;
    let mut res = CallToolResult::success(vec![Content::text(json_val.to_string())]);
    attach_forensic_meta(&mut res, confidence, Some(is_causal), "pv_causality");
    Ok(res)
}

// =============================================================================
// UCAS Tool
// =============================================================================

/// UCAS universal causality assessment (ToV §36).
///
/// Domain-agnostic causality assessment with 8 criteria and sigmoid-calibrated
/// recognition component for ToV integration (R = sigmoid(score, mu=5, sigma=2)).
/// Score range: -3 to +14.
pub fn causality_ucas(params: UcasParams) -> Result<CallToolResult, McpError> {
    let input = UcasInput::new()
        .with_temporal(to_criterion_response(&params.temporal_relationship))
        .with_dechallenge(to_criterion_response(&params.dechallenge))
        .with_rechallenge(to_criterion_response(&params.rechallenge))
        .with_mechanism(to_criterion_response(&params.mechanistic_plausibility))
        .with_alternatives(to_criterion_response(&params.alternative_explanations))
        .with_dose_response(to_criterion_response(&params.dose_response))
        .with_prior_evidence(to_criterion_response(&params.prior_evidence))
        .with_specificity(to_criterion_response(&params.specificity));

    let result = calculate_ucas(&input);

    let breakdown: Vec<_> = result
        .breakdown
        .iter()
        .map(|b| {
            json!({
                "criterion": b.name,
                "response": format!("{:?}", b.response),
                "score": b.score.value(),
                "max_points": b.max_points,
            })
        })
        .collect();

    let json_val = json!({
        "score": result.score.value(),
        "category": result.category.to_string(),
        "recognition_r": (result.recognition_r * 1000.0).round() / 1000.0,
        "confidence": result.confidence,
        "breakdown": breakdown,
        "is_likely_causal": result.is_likely_causal(),
        "is_possibly_causal": result.is_possibly_causal(),
        "recommended_action": result.category.recommended_action(),
    });

    let confidence = result.confidence;
    let is_causal = result.is_likely_causal();
    let mut res = CallToolResult::success(vec![Content::text(json_val.to_string())]);
    attach_forensic_meta(&mut res, confidence, Some(is_causal), "pv_causality");
    Ok(res)
}
