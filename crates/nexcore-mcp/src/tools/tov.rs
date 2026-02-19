//! Theory of Vigilance (ToV) direct tools
//!
//! Signal strength S = U × R × T, stability shells, epistemic trust.
//! Unique types from nexcore-tov not already exposed through vigilance.rs.

use crate::params::{TovEpistemicTrustParams, TovSignalStrengthParams, TovStabilityShellParams};
use nexcore_pv_core::SafetyMargin;
use nexcore_tov::{
    Bits, ComplexityChi, QuantityUnit, RecognitionR, SignalStrengthS, StabilityShell, TemporalT,
    UniquenessU,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Calculate signal strength S = U × R × T (ToV Core Equation §20)
pub fn signal_strength(params: TovSignalStrengthParams) -> Result<CallToolResult, McpError> {
    let u = UniquenessU(Bits(params.uniqueness_bits));
    let r = RecognitionR(params.recognition.clamp(0.0, 1.0));
    let t = TemporalT(params.temporal.clamp(0.0, 1.0));

    let s = SignalStrengthS::calculate(u, r, t);

    let json = json!({
        "signal_strength_bits": s.0.0,
        "components": {
            "uniqueness_U": params.uniqueness_bits,
            "recognition_R": r.0,
            "temporal_T": t.0,
        },
        "equation": "S = U × R × T",
        "interpretation": if s.0.0 > 5.0 {
            "Strong signal — high uniqueness, recognition, and recency"
        } else if s.0.0 > 2.0 {
            "Moderate signal — warrants investigation"
        } else if s.0.0 > 0.5 {
            "Weak signal — monitor but low priority"
        } else {
            "Negligible signal — no action needed"
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Check architectural stability shell (ToV §66.2)
///
/// Uses complexity magic numbers [2, 8, 20, 28, 50, 82, 126, 184, 258, 350]
/// to determine if a system's complexity count is at a stability point.
pub fn stability_shell(params: TovStabilityShellParams) -> Result<CallToolResult, McpError> {
    let chi = ComplexityChi(QuantityUnit(params.complexity));
    let is_stable = chi.is_closed_shell();
    let distance = chi.distance_to_stability();

    let json = json!({
        "complexity": params.complexity,
        "is_closed_shell": is_stable,
        "distance_to_stability": distance,
        "magic_numbers": [2, 8, 20, 28, 50, 82, 126, 184, 258, 350],
        "interpretation": if is_stable {
            "At a magic number — system is at a stability point"
        } else if distance <= 2 {
            "Near a magic number — close to stability"
        } else if distance <= 5 {
            "Moderate distance from stability — consider refactoring"
        } else {
            "Far from stability — structural risk"
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Score epistemic trust based on ToV hierarchy coverage and evidence sources
pub fn epistemic_trust(params: TovEpistemicTrustParams) -> Result<CallToolResult, McpError> {
    let score = SafetyMargin::score_epistemic_trust(&params.levels_covered, params.sources);

    let json = json!({
        "epistemic_trust": (score * 1000.0).round() / 1000.0,
        "levels_covered": params.levels_covered,
        "level_count": params.levels_covered.len(),
        "total_levels": 8,
        "coverage_ratio": (params.levels_covered.len() as f64 / 8.0 * 100.0).round() / 100.0,
        "sources": params.sources,
        "interpretation": if score > 0.8 {
            "High epistemic trust — comprehensive evidence across hierarchy"
        } else if score > 0.5 {
            "Moderate epistemic trust — reasonable coverage"
        } else if score > 0.2 {
            "Low epistemic trust — limited evidence coverage"
        } else {
            "Very low epistemic trust — insufficient evidence"
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{
        TovEpistemicTrustParams, TovSignalStrengthParams, TovStabilityShellParams,
    };

    fn extract_json(result: &CallToolResult) -> serde_json::Value {
        let content = &result.content[0];
        let text = content.as_text().expect("expected text content");
        serde_json::from_str(&text.text).expect("valid JSON")
    }

    #[test]
    fn signal_strength_normal() {
        let r = signal_strength(TovSignalStrengthParams {
            uniqueness_bits: 10.0,
            recognition: 0.8,
            temporal: 0.9,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["signal_strength_bits"].as_f64().expect("f64") > 0.0);
        assert_eq!(j["equation"], "S = U × R × T");
    }

    #[test]
    fn signal_strength_zero_inputs() {
        let r = signal_strength(TovSignalStrengthParams {
            uniqueness_bits: 0.0,
            recognition: 0.0,
            temporal: 0.0,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["signal_strength_bits"].as_f64().expect("f64") <= 0.01);
    }

    #[test]
    fn signal_strength_clamps_recognition() {
        let r = signal_strength(TovSignalStrengthParams {
            uniqueness_bits: 5.0,
            recognition: 2.0,
            temporal: 0.5,
        })
        .expect("ok");
        let j = extract_json(&r);
        let v = j["components"]["recognition_R"].as_f64().expect("f64");
        assert!((v - 1.0).abs() < f64::EPSILON, "clamp to 1.0: {v}");
    }

    #[test]
    fn signal_strength_clamps_temporal_negative() {
        let r = signal_strength(TovSignalStrengthParams {
            uniqueness_bits: 5.0,
            recognition: 0.5,
            temporal: -1.0,
        })
        .expect("ok");
        let j = extract_json(&r);
        let v = j["components"]["temporal_T"].as_f64().expect("f64");
        assert!(v.abs() < f64::EPSILON, "clamp to 0.0: {v}");
    }

    #[test]
    fn signal_strength_strong_interpretation() {
        let r = signal_strength(TovSignalStrengthParams {
            uniqueness_bits: 20.0,
            recognition: 1.0,
            temporal: 1.0,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(
            j["interpretation"]
                .as_str()
                .expect("str")
                .contains("Strong")
        );
    }

    #[test]
    fn stability_shell_magic_numbers() {
        for magic in [2, 8, 20, 28, 50, 82, 126] {
            let r = stability_shell(TovStabilityShellParams { complexity: magic }).expect("ok");
            let j = extract_json(&r);
            assert!(
                j["is_closed_shell"].as_bool().expect("bool"),
                "{magic} closed"
            );
            assert_eq!(j["distance_to_stability"].as_u64().expect("u64"), 0);
        }
    }

    #[test]
    fn stability_shell_near_miss() {
        let r = stability_shell(TovStabilityShellParams { complexity: 19 }).expect("ok");
        let j = extract_json(&r);
        assert!(!j["is_closed_shell"].as_bool().expect("bool"));
        assert!(j["distance_to_stability"].as_u64().expect("u64") <= 2);
    }

    #[test]
    fn stability_shell_zero() {
        let r = stability_shell(TovStabilityShellParams { complexity: 0 }).expect("ok");
        let j = extract_json(&r);
        assert!(!j["is_closed_shell"].as_bool().expect("bool"));
        assert_eq!(j["distance_to_stability"].as_u64().expect("u64"), 2);
    }

    #[test]
    fn stability_shell_very_large() {
        let r = stability_shell(TovStabilityShellParams { complexity: 1000 }).expect("ok");
        let j = extract_json(&r);
        assert!(!j["is_closed_shell"].as_bool().expect("bool"));
    }

    #[test]
    fn epistemic_trust_full_coverage() {
        let r = epistemic_trust(TovEpistemicTrustParams {
            levels_covered: vec![1, 2, 3, 4, 5, 6, 7, 8],
            sources: 10,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["epistemic_trust"].as_f64().expect("f64") > 0.5);
        assert_eq!(j["level_count"], 8);
    }

    #[test]
    fn epistemic_trust_empty() {
        let r = epistemic_trust(TovEpistemicTrustParams {
            levels_covered: vec![],
            sources: 0,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["epistemic_trust"].as_f64().expect("f64") < 0.3);
    }

    #[test]
    fn epistemic_trust_single_level() {
        let r = epistemic_trust(TovEpistemicTrustParams {
            levels_covered: vec![3],
            sources: 100,
        })
        .expect("ok");
        let j = extract_json(&r);
        assert!(j["epistemic_trust"].as_f64().expect("f64") < 1.0);
    }
}
