//! Theory of Vigilance (Grounded) MCP tools.
//!
//! Runtime ToV primitives: signal strength calculation, safety margin,
//! stability shell analysis, harm classification, meta-vigilance health.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::tov_grounded::{
    TovGroundedEkaIntelligenceParams, TovGroundedHarmTypeParams, TovGroundedMagicNumbersParams,
    TovGroundedMetaVigilanceParams, TovGroundedSafetyMarginParams, TovGroundedSignalStrengthParams,
    TovGroundedStabilityShellParams,
};

// ── Helpers ──────────────────────────────────────────────────────────────

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

// ── Tools ────────────────────────────────────────────────────────────────

/// Calculate signal strength S = U × R × T.
pub fn tov_signal_strength(p: TovGroundedSignalStrengthParams) -> Result<CallToolResult, McpError> {
    let u = nexcore_tov_grounded::UniquenessU(nexcore_tov_grounded::Bits(p.uniqueness));
    let r = nexcore_tov_grounded::RecognitionR(p.recognition);
    let t = nexcore_tov_grounded::TemporalT(p.temporal);
    let s = nexcore_tov_grounded::SignalStrengthS::calculate(u, r, t);
    ok_json(json!({
        "uniqueness_bits": p.uniqueness,
        "recognition": p.recognition,
        "temporal": p.temporal,
        "signal_strength": (s.0).0,
        "formula": "S = U × R × T",
    }))
}

/// Calculate safety margin d(s) = (threshold - s) / threshold.
pub fn tov_safety_margin(p: TovGroundedSafetyMarginParams) -> Result<CallToolResult, McpError> {
    let sys = nexcore_tov_grounded::VigilanceSystem {
        id: String::new(),
        state_space_dim: 0,
        elements: Vec::new(),
        constraints: std::collections::HashMap::new(),
    };
    let margin = sys.calculate_safety_margin(p.signal, p.threshold);
    let safe = margin.0 > 0.0;
    ok_json(json!({
        "signal": p.signal,
        "threshold": p.threshold,
        "safety_margin": margin.0,
        "is_safe": safe,
        "formula": "d(s) = (threshold - s) / threshold",
    }))
}

/// Check if a complexity value sits on a stability shell (magic number).
pub fn tov_stability_shell(p: TovGroundedStabilityShellParams) -> Result<CallToolResult, McpError> {
    use nexcore_tov_grounded::{
        COMPLEXITY_MAGIC_NUMBERS, CONNECTION_MAGIC_NUMBERS, ComplexityChi,
        ConfigurableStabilityShell, QuantityUnit, ShellConfig, StabilityShell,
    };

    let chi = ComplexityChi(QuantityUnit(p.complexity));
    let shell_type = p.shell_type.as_deref().unwrap_or("complexity");

    let (is_closed, distance, magic_numbers) = match shell_type {
        "connection" => {
            let config = ShellConfig {
                magic_numbers: CONNECTION_MAGIC_NUMBERS.to_vec(),
            };
            (
                chi.is_closed_shell_with(&config),
                chi.distance_to_stability_with(&config),
                CONNECTION_MAGIC_NUMBERS.to_vec(),
            )
        }
        _ => (
            chi.is_closed_shell(),
            chi.distance_to_stability(),
            COMPLEXITY_MAGIC_NUMBERS.to_vec(),
        ),
    };

    ok_json(json!({
        "complexity": p.complexity,
        "shell_type": shell_type,
        "is_closed_shell": is_closed,
        "distance_to_stability": distance,
        "magic_numbers": magic_numbers,
    }))
}

/// Classify a harm type and return its properties.
pub fn tov_harm_type(p: TovGroundedHarmTypeParams) -> Result<CallToolResult, McpError> {
    let (harm_type, letter, description) = match p.harm_type.to_lowercase().as_str() {
        "acute" | "a" => ("Acute", "A", "Immediate direct harm"),
        "chronic" | "b" => ("Chronic", "B", "Slow-building cumulative harm"),
        "cascading" | "c" => (
            "Cascading",
            "C",
            "Harm that propagates through dependencies",
        ),
        "dormant" | "d" => ("Dormant", "D", "Latent harm awaiting trigger"),
        "emergent" | "e" => ("Emergent", "E", "Harm from unexpected interactions"),
        "feedback" | "f" => ("Feedback", "F", "Self-amplifying harm loop"),
        "gateway" | "g" => ("Gateway", "G", "Harm that enables further harm"),
        "hidden" | "h" => ("Hidden", "H", "Undetectable or obscured harm"),
        _ => {
            return err_result(
                "Unknown harm type. Use: Acute, Chronic, Cascading, Dormant, Emergent, Feedback, Gateway, Hidden",
            );
        }
    };

    ok_json(json!({
        "harm_type": harm_type,
        "letter": letter,
        "description": description,
        "all_types": ["A: Acute", "B: Chronic", "C: Cascading", "D: Dormant",
                      "E: Emergent", "F: Feedback", "G: Gateway", "H: Hidden"],
    }))
}

/// Check meta-vigilance health of the vigilance loop itself.
pub fn tov_meta_vigilance(p: TovGroundedMetaVigilanceParams) -> Result<CallToolResult, McpError> {
    let mv = nexcore_tov_grounded::MetaVigilance {
        loop_latency_ms: p.loop_latency_ms,
        calibration_overhead_ms: p.calibration_overhead_ms,
        detection_drift: p.detection_drift,
        apparatus_integrity: nexcore_tov_grounded::RecognitionR(p.apparatus_integrity),
    };
    let healthy = mv.is_healthy();
    let net_latency = p.loop_latency_ms.saturating_sub(p.calibration_overhead_ms);
    ok_json(json!({
        "is_healthy": healthy,
        "net_latency_ms": net_latency,
        "latency_ok": net_latency < 100,
        "integrity_ok": p.apparatus_integrity > 0.95,
        "detection_drift": p.detection_drift,
        "thresholds": {
            "max_net_latency_ms": 100,
            "min_integrity": 0.95,
        },
    }))
}

/// Check EkaIntelligence emergence threshold.
pub fn tov_eka_intelligence(
    p: TovGroundedEkaIntelligenceParams,
) -> Result<CallToolResult, McpError> {
    use nexcore_tov_grounded::{ComplexityChi, QuantityUnit, StabilityShell};

    let eka = nexcore_tov_grounded::EkaIntelligence {
        complexity: ComplexityChi(QuantityUnit(p.complexity)),
        stability: p.stability,
    };
    ok_json(json!({
        "complexity": p.complexity,
        "stability": p.stability,
        "is_emergent": eka.is_emergent(),
        "emergence_threshold": 320,
        "is_closed_shell": eka.is_closed_shell(),
        "distance_to_stability": eka.distance_to_stability(),
    }))
}

/// List all stability shell magic numbers for a given shell type.
pub fn tov_magic_numbers(p: TovGroundedMagicNumbersParams) -> Result<CallToolResult, McpError> {
    let shell_type = p.shell_type.as_deref().unwrap_or("complexity");
    let numbers = match shell_type {
        "connection" => nexcore_tov_grounded::CONNECTION_MAGIC_NUMBERS.to_vec(),
        _ => nexcore_tov_grounded::COMPLEXITY_MAGIC_NUMBERS.to_vec(),
    };
    ok_json(json!({
        "shell_type": shell_type,
        "magic_numbers": numbers,
        "count": numbers.len(),
    }))
}
