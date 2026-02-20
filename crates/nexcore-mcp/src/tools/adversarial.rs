//! Adversarial prompt sensor tools
//!
//! Tier: T3 (μ Mapping + ∂ Boundary)

use crate::params::{AdversarialDecisionProbeParams, AdversarialSensorInputParams};
use crate::tools::guardian::get_loop;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Set the input text for the AdversarialPromptSensor to analyze on next tick.
pub async fn guardian_adversarial_input(
    params: AdversarialSensorInputParams,
) -> Result<CallToolResult, McpError> {
    // If text looks like code, use a stricter threshold
    let is_code =
        params.text.contains('{') || params.text.contains(';') || params.text.contains("pub fn");

    let config = antitransformer::pipeline::AnalysisConfig {
        threshold: if is_code { 0.65 } else { 0.50 },
        window_size: 50,
    };

    let result = antitransformer::pipeline::analyze(&params.text, &config);

    let mut control_loop = get_loop().lock().await;

    // For code, we want to detect if it's purely generated boilerplate
    let suspicion_threshold = if is_code { 0.80 } else { 0.40 };

    if result.verdict == "generated" || result.probability > suspicion_threshold {
        use nexcore_guardian_engine::sensing::{SignalSource, ThreatLevel, ThreatSignal};
        use nexcore_primitives::measurement::Measured;

        let severity = if result.probability >= 0.9 {
            ThreatLevel::Critical
        } else if result.probability >= 0.75 {
            ThreatLevel::High
        } else {
            ThreatLevel::Medium
        };

        let signal = ThreatSignal::new(
            result.clone().to_string(),
            severity,
            SignalSource::Pamp {
                source_id: "mcp_adversarial_input".to_string(),
                vector: if is_code {
                    "pathogenic_code_pattern"
                } else {
                    "statistical_anomaly"
                }
                .to_string(),
            },
        )
        .with_confidence(Measured::certain(result.confidence))
        .with_metadata("verdict", &result.verdict)
        .with_metadata("type", if is_code { "code" } else { "text" })
        .with_metadata("probability", format!("{:.3}", result.probability));

        control_loop.inject_signal(signal);
    }

    let response = json!({
        "status": "analyzed",
        "type": if is_code { "code" } else { "text" },
        "verdict": result.verdict,
        "probability": result.probability,
        "confidence": result.confidence,
        "action": if result.probability > suspicion_threshold { "signal_injected" } else { "none" }
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Probe a PV signal decision for adversarial robustness.
///
/// Given baseline PV metrics (PRR, ROR-LCI, IC025, EB05), generates systematic
/// perturbations near each metric's threshold to test decision stability.
/// Returns a robustness score, per-metric vulnerabilities, and recommendation.
pub fn adversarial_decision_probe(
    params: AdversarialDecisionProbeParams,
) -> Result<CallToolResult, McpError> {
    let steps = params.perturbation_steps.unwrap_or(5);

    // PV signal thresholds (ToV invariants)
    let metrics: [(&str, f64, f64); 4] = [
        ("PRR", params.prr, 2.0),
        ("ROR_LCI", params.ror_lower, 1.0),
        ("IC025", params.ic025, 0.0),
        ("EB05", params.eb05, 2.0),
    ];

    let mut vulnerabilities = Vec::new();
    let mut total_flips: usize = 0;
    let mut total_probes: usize = 0;

    for (name, value, threshold) in metrics {
        let signal_detected = value > threshold;
        let margin = (value - threshold).abs();
        // step size: spread perturbations over ±50% of margin (minimum 0.05)
        let step_size = (margin * 0.5 / steps as f64).max(0.05);

        let mut flips: usize = 0;
        for i in 1..=steps {
            let perturbation = step_size * i as f64;
            for &sign in &[1.0_f64, -1.0] {
                let perturbed = value + sign * perturbation;
                let perturbed_detected = perturbed > threshold;
                total_probes += 1;
                if perturbed_detected != signal_detected {
                    flips += 1;
                }
            }
        }
        total_flips += flips;

        if flips > 0 {
            vulnerabilities.push(json!({
                "metric": name,
                "baseline_value": value,
                "threshold": threshold,
                "signal_detected": signal_detected,
                "margin": margin,
                "step_size": step_size,
                "flips": flips,
                "flip_rate": flips as f64 / (steps * 2) as f64,
            }));
        }
    }

    let signal_detected =
        params.prr > 2.0 && params.ror_lower > 1.0 && params.ic025 > 0.0 && params.eb05 >= 2.0;

    let robustness_score = if total_probes > 0 {
        1.0 - (total_flips as f64 / total_probes as f64)
    } else {
        1.0
    };

    let recommendation = if robustness_score >= 0.90 {
        "ROBUST: Decision stable across perturbations"
    } else if robustness_score >= 0.70 {
        "BORDERLINE: Decision near threshold — verify with additional data"
    } else {
        "FRAGILE: Decision flips under small perturbations — insufficient evidence"
    };

    let response = json!({
        "signal_detected": signal_detected,
        "robustness_score": robustness_score,
        "n": params.n,
        "vulnerabilities": vulnerabilities,
        "recommendation": recommendation,
        "probe_summary": {
            "total_probes": total_probes,
            "total_flips": total_flips,
            "perturbation_steps": steps,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}
