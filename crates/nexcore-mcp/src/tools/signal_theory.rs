//! Signal Theory tools — Universal Theory of Signals (axioms, theorems, detection, SDT)
//!
//! Wraps `nexcore-signal-theory` crate for MCP access.
//! Dominant primitive: ∂ (Boundary). Core thesis: "All detection is boundary drawing."

use crate::params::{
    SignalTheoryCascadeParams, SignalTheoryConservationCheckParams,
    SignalTheoryDecisionMatrixParams, SignalTheoryDetectParams, SignalTheoryParallelParams,
    SignalTheoryPipelineParams,
};
use nexcore_signal_theory::prelude::*;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// List all 6 axioms of signal theory.
pub fn axioms() -> Result<CallToolResult, McpError> {
    let axiom_data: [(_, _, _, _); 6] = [
        (
            "A1",
            "Data Generation",
            "ν (Frequency)",
            <A1DataGeneration<1000> as Axiom>::statement(),
        ),
        (
            "A2",
            "Noise Dominance",
            "∅ (Void)",
            <A2NoiseDominance as Axiom>::statement(),
        ),
        (
            "A3",
            "Signal Existence",
            "∃ (Existence)",
            <A3SignalExistence as Axiom>::statement(),
        ),
        (
            "A4",
            "Boundary Requirement",
            "∂ (Boundary) [DOMINANT]",
            <A4BoundaryRequirement as Axiom>::statement(),
        ),
        (
            "A5",
            "Disproportionality",
            "κ (Comparison)",
            <A5Disproportionality as Axiom>::statement(),
        ),
        (
            "A6",
            "Causal Inference",
            "→ (Causality)",
            <A6CausalInference as Axiom>::statement(),
        ),
    ];

    let axioms: Vec<_> = axiom_data
        .iter()
        .map(|(id, name, prim, stmt)| {
            json!({
                "id": id, "name": name,
                "primitive": prim,
                "statement": stmt,
            })
        })
        .collect();

    let result = json!({
        "crate": "nexcore-signal-theory",
        "thesis": "All detection is boundary drawing",
        "dominant_primitive": "∂ (Boundary)",
        "axiom_count": 6,
        "axioms": axioms,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// List all 5 theorems of signal theory.
pub fn theorems() -> Result<CallToolResult, McpError> {
    let registry = TheoremRegistry::build();
    let theorems: Vec<_> = registry
        .theorems
        .iter()
        .map(|t| {
            json!({
                "id": t.id,
                "name": t.name,
                "statement": t.statement,
                "prerequisites": t.prerequisites,
            })
        })
        .collect();

    let result = json!({
        "theorem_count": theorems.len(),
        "theorems": theorems,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Run signal detection: compare observed vs expected against a threshold.
pub fn detect(params: SignalTheoryDetectParams) -> Result<CallToolResult, McpError> {
    let threshold = params.threshold.unwrap_or(2.0);
    let boundary = FixedBoundary::above(threshold, "detection");

    let ratio = Ratio::from_counts(params.observed, params.expected);
    let (ratio_value, detected, strength) = match ratio {
        Some(r) => {
            let det = boundary.evaluate(r.0);
            let strength = SignalStrengthLevel::from_ratio(r.0);
            (Some(r.0), det, strength)
        }
        None => (None, false, SignalStrengthLevel::None),
    };

    let difference = Difference::from_counts(params.observed, params.expected);

    let result = json!({
        "observed": params.observed,
        "expected": params.expected,
        "threshold": threshold,
        "ratio": ratio_value,
        "difference": difference.0,
        "detected": detected,
        "strength": format!("{:?}", strength),
        "outcome": if detected { "Detected" } else { "NotDetected" },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Compute SDT decision matrix metrics from a 2×2 table.
pub fn decision_matrix(
    params: SignalTheoryDecisionMatrixParams,
) -> Result<CallToolResult, McpError> {
    let m = DecisionMatrix::new(
        params.hits,
        params.misses,
        params.false_alarms,
        params.correct_rejections,
    );

    let dprime = DPrime::from_matrix(&m);
    let bias = ResponseBias::from_matrix(&m);

    let result = json!({
        "matrix": {
            "hits": m.hits,
            "misses": m.misses,
            "false_alarms": m.false_alarms,
            "correct_rejections": m.correct_rejections,
            "total": m.total(),
        },
        "metrics": {
            "sensitivity": m.sensitivity(),
            "specificity": m.specificity(),
            "ppv": m.ppv(),
            "npv": m.npv(),
            "accuracy": m.accuracy(),
            "fpr": m.false_positive_rate(),
            "fnr": m.false_negative_rate(),
            "prevalence": m.prevalence(),
            "f1_score": m.f1_score(),
            "mcc": m.mcc(),
        },
        "sdt": {
            "d_prime": dprime.0,
            "d_prime_level": dprime.level(),
            "response_bias": bias.0,
            "bias_description": bias.description(),
        },
        "signal_present": m.signal_present(),
        "signal_absent": m.signal_absent(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Verify conservation laws on a decision matrix.
pub fn conservation_check(
    params: SignalTheoryConservationCheckParams,
) -> Result<CallToolResult, McpError> {
    let m = DecisionMatrix::new(
        params.hits,
        params.misses,
        params.false_alarms,
        params.correct_rejections,
    );

    let mut report = ConservationReport::new();

    // L1: Total count conservation
    let expected_total = params
        .expected_total
        .unwrap_or(m.hits + m.misses + m.false_alarms + m.correct_rejections);
    let l1 = L1TotalCountConservation;
    report.add("L1", l1.verify(&m, expected_total));

    // L4: Information conservation (if max_dprime provided)
    if let Some(max_dp) = params.max_dprime {
        let l4 = L4InformationConservation;
        let observed_dp = DPrime::from_matrix(&m).0;
        report.add("L4", l4.verify(observed_dp, max_dp));
    }

    let violations: Vec<_> = report
        .violations()
        .iter()
        .map(|(id, msg)| json!({"law": id, "violation": msg}))
        .collect();

    let result = json!({
        "all_satisfied": report.all_satisfied(),
        "laws_checked": report.results.len(),
        "violations": violations,
        "conservation_laws": [
            {"id": "L1", "name": "Total Count Conservation", "statement": "The 2x2 matrix is exhaustive"},
            {"id": "L2", "name": "Base Rate Invariance", "statement": "Prevalence independent of threshold"},
            {"id": "L3", "name": "Sensitivity-Specificity Tradeoff", "statement": "Improving one degrades the other"},
            {"id": "L4", "name": "Information Conservation", "statement": "Detection cannot create signal info"},
        ],
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Run a multi-stage signal detection pipeline.
///
/// Passes a value through sequential stages; each stage has its own threshold.
/// Reports which stages passed and the overall verdict.
pub fn pipeline(params: SignalTheoryPipelineParams) -> Result<CallToolResult, McpError> {
    let mut stages_result = Vec::new();
    let mut all_passed = true;
    let mut first_failure: Option<String> = None;

    for (i, stage) in params.stages.iter().enumerate() {
        let boundary = FixedBoundary::above(stage.threshold, "detection");
        let passed = boundary.evaluate(params.value);

        if !passed && first_failure.is_none() {
            first_failure = Some(stage.name.clone());
            all_passed = false;
        }

        stages_result.push(json!({
            "stage": i + 1,
            "name": stage.name,
            "phase": stage.phase,
            "threshold": stage.threshold,
            "value": params.value,
            "passed": passed,
        }));
    }

    let result = json!({
        "label": params.label,
        "value": params.value,
        "stage_count": params.stages.len(),
        "all_passed": all_passed,
        "first_failure": first_failure,
        "stages": stages_result,
        "verdict": if all_passed { "SIGNAL_DETECTED" } else { "NOT_DETECTED" },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Cascading threshold evaluation — find the highest severity level exceeded.
///
/// Thresholds must be in ascending order. Reports the highest level exceeded.
pub fn cascade(params: SignalTheoryCascadeParams) -> Result<CallToolResult, McpError> {
    if params.thresholds.len() != params.labels.len() {
        let err = json!({
            "error": "thresholds and labels must have equal length",
            "thresholds_len": params.thresholds.len(),
            "labels_len": params.labels.len(),
        });
        return Ok(CallToolResult::success(vec![Content::text(
            err.to_string(),
        )]));
    }

    let mut levels = Vec::new();
    let mut highest_level: Option<usize> = None;

    for (i, (threshold, label)) in params
        .thresholds
        .iter()
        .zip(params.labels.iter())
        .enumerate()
    {
        let boundary = FixedBoundary::above(*threshold, "cascade");
        let exceeded = boundary.evaluate(params.value);
        if exceeded {
            highest_level = Some(i);
        }
        levels.push(json!({
            "level": i + 1,
            "label": label,
            "threshold": threshold,
            "exceeded": exceeded,
        }));
    }

    let result = json!({
        "value": params.value,
        "levels": levels,
        "highest_level_exceeded": highest_level.map(|l| l + 1),
        "highest_label": highest_level.and_then(|l| params.labels.get(l)),
        "verdict": if highest_level.is_some() { "SIGNAL_DETECTED" } else { "NOT_DETECTED" },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Parallel signal detection across two independent detectors.
///
/// Mode "both" (AND): signal only if both detectors fire.
/// Mode "either" (OR): signal if at least one detector fires.
pub fn parallel(params: SignalTheoryParallelParams) -> Result<CallToolResult, McpError> {
    let b1 = FixedBoundary::above(params.threshold_1, "detector_1");
    let b2 = FixedBoundary::above(params.threshold_2, "detector_2");

    let d1 = b1.evaluate(params.value);
    let d2 = b2.evaluate(params.value);

    let combined = if params.mode == "either" {
        d1 || d2
    } else {
        d1 && d2
    };

    let result = json!({
        "value": params.value,
        "mode": params.mode,
        "detector_1": { "label": params.label_1, "threshold": params.threshold_1, "detected": d1 },
        "detector_2": { "label": params.label_2, "threshold": params.threshold_2, "detected": d2 },
        "combined_result": combined,
        "verdict": if combined { "SIGNAL_DETECTED" } else { "NOT_DETECTED" },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}
