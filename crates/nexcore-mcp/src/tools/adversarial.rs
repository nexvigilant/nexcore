//! Adversarial prompt sensor tools
//!
//! Tier: T3 (μ Mapping + ∂ Boundary)

use crate::params::AdversarialSensorInputParams;
use crate::tools::guardian::get_loop;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Set the input text for the AdversarialPromptSensor to analyze on next tick.
pub async fn guardian_adversarial_input(
    params: AdversarialSensorInputParams,
) -> Result<CallToolResult, McpError> {
    // If text looks like code, use a stricter threshold
    let is_code = params.text.contains('{') || params.text.contains(';') || params.text.contains("pub fn");
    
    let config = antitransformer::pipeline::AnalysisConfig {
        threshold: if is_code { 0.65 } else { 0.50 },
        window_size: 50,
    };
    
    let result = antitransformer::pipeline::analyze(&params.text, &config);
    
    let mut control_loop = get_loop().lock().await;
    
    // For code, we want to detect if it's purely generated boilerplate
    let suspicion_threshold = if is_code { 0.80 } else { 0.40 };
    
    if result.verdict == "generated" || result.probability > suspicion_threshold {
        use nexcore_guardian_engine::sensing::{ThreatSignal, ThreatLevel, SignalSource};
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
                vector: if is_code { "pathogenic_code_pattern" } else { "statistical_anomaly" }.to_string(),
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
