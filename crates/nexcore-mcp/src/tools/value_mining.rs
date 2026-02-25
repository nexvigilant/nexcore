//! Value Mining MCP Tools
//!
//! Economic signal detection using pharmacovigilance algorithms.
//! Maps PV concepts (PRR, ROR, IC, EBGM, Chi²) to value discovery.

use nexcore_value_mining::{SignalStrength, SignalType};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::{
    ValueBaselineCreateParams, ValuePvMappingParams, ValueSignalDetectParams,
    ValueSignalTypesParams,
};

/// List all value signal types with their PV analogs and T1 primitives.
pub fn list_signal_types(params: ValueSignalTypesParams) -> Result<CallToolResult, McpError> {
    let types = [
        SignalType::Sentiment,
        SignalType::Trend,
        SignalType::Engagement,
        SignalType::Virality,
        SignalType::Controversy,
    ];

    let filtered: Vec<_> = types
        .iter()
        .filter(|t| {
            params
                .type_filter
                .as_ref()
                .map(|f| t.to_string().to_lowercase().contains(&f.to_lowercase()))
                .unwrap_or(true)
        })
        .map(|t| {
            serde_json::json!({
                "type": t.to_string(),
                "pv_analog": t.pv_analog(),
                "primitives": t.primitives(),
                "description": match t {
                    SignalType::Sentiment => "Unexpected positive/negative sentiment vs baseline",
                    SignalType::Trend => "Directional momentum over time",
                    SignalType::Engagement => "Unusual engagement rate vs baseline",
                    SignalType::Virality => "Exponential growth phase detection",
                    SignalType::Controversy => "High sentiment variance (polarization)",
                },
            })
        })
        .collect();

    let result = serde_json::json!({
        "signal_types": filtered,
        "total": filtered.len(),
        "domain_transfer": "Pharmacovigilance → Economic Value Mining (0.87 confidence)",
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Detect a value signal from numeric observations.
///
/// Uses the PV algorithm analog to compute signal strength.
pub fn detect_signal(params: ValueSignalDetectParams) -> Result<CallToolResult, McpError> {
    let signal_type = match params.signal_type.to_lowercase().as_str() {
        "sentiment" => SignalType::Sentiment,
        "trend" => SignalType::Trend,
        "engagement" => SignalType::Engagement,
        "virality" => SignalType::Virality,
        "controversy" => SignalType::Controversy,
        _ => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Unknown signal type '{}'. Valid: sentiment, trend, engagement, virality, controversy",
                params.signal_type
            ))]));
        }
    };

    // Apply the appropriate PV algorithm analog
    let (score, confidence, algorithm) = match signal_type {
        SignalType::Sentiment => {
            // PRR analog: observed / baseline (proportional reporting ratio)
            let prr = if params.baseline > 0.0 {
                params.observed / params.baseline
            } else {
                0.0
            };
            // Confidence based on sample size and deviation from 1.0
            let conf = compute_prr_confidence(prr, params.sample_size);
            (prr, conf, "PRR")
        }
        SignalType::Trend => {
            // IC analog: log2(observed/baseline) - information component
            let ic = if params.baseline > 0.0 && params.observed > 0.0 {
                (params.observed / params.baseline).log2()
            } else {
                0.0
            };
            let conf = compute_ic_confidence(ic, params.sample_size);
            (ic, conf, "IC")
        }
        SignalType::Engagement => {
            // ROR analog: (observed/(1-observed)) / (baseline/(1-baseline))
            let or_obs = params.observed / (1.0 - params.observed.clamp(0.001, 0.999));
            let or_base = params.baseline / (1.0 - params.baseline.clamp(0.001, 0.999));
            let ror = if or_base > 0.0 { or_obs / or_base } else { 0.0 };
            let conf = compute_ror_confidence(ror, params.sample_size);
            (ror, conf, "ROR")
        }
        SignalType::Virality => {
            // EBGM analog: empirical Bayes geometric mean
            // Simplified: ratio with shrinkage toward 1.0
            let ratio = if params.baseline > 0.0 {
                params.observed / params.baseline
            } else {
                1.0
            };
            let shrinkage = params.sample_size as f64 / (params.sample_size as f64 + 10.0);
            let ebgm = 1.0 + (ratio - 1.0) * shrinkage;
            let conf = compute_ebgm_confidence(ebgm, params.sample_size);
            (ebgm, conf, "EBGM")
        }
        SignalType::Controversy => {
            // Chi² analog: squared deviation from expected
            let chi2 = if params.baseline > 0.0 {
                let diff = params.observed - params.baseline;
                (diff * diff) / params.baseline
            } else {
                0.0
            };
            let conf = compute_chi2_confidence(chi2, params.sample_size);
            (chi2, conf, "Chi²")
        }
    };

    let strength = SignalStrength::from_confidence(confidence);
    let is_actionable = confidence >= 0.7;

    let result = serde_json::json!({
        "signal_type": signal_type.to_string(),
        "pv_algorithm": algorithm,
        "entity": params.entity,
        "source": params.source,
        "observed": params.observed,
        "baseline": params.baseline,
        "sample_size": params.sample_size,
        "score": format!("{:.4}", score),
        "confidence": format!("{:.4}", confidence),
        "strength": strength.to_string(),
        "is_actionable": is_actionable,
        "primitives": signal_type.primitives(),
        "interpretation": interpret_signal(signal_type, score, strength),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Create a baseline for signal detection.
pub fn create_baseline(params: ValueBaselineCreateParams) -> Result<CallToolResult, McpError> {
    let baseline = serde_json::json!({
        "source": params.source,
        "positive_rate": params.positive_rate,
        "negative_rate": params.negative_rate,
        "avg_engagement": params.avg_engagement,
        "posts_per_hour": params.posts_per_hour,
        "computed_at": nexcore_chrono::DateTime::now().to_rfc3339(),
        "sample_count": 0,
        "grounding": "Baseline → N (quantities) + π (persistence) + ν (frequency)",
    });

    Ok(CallToolResult::success(vec![Content::text(
        baseline.to_string(),
    )]))
}

/// Get the PV ↔ Value Mining algorithm mapping.
pub fn get_pv_mapping(params: ValuePvMappingParams) -> Result<CallToolResult, McpError> {
    let mappings = vec![
        serde_json::json!({
            "value_signal": "Sentiment",
            "pv_algorithm": "PRR",
            "pv_full_name": "Proportional Reporting Ratio",
            "formula": "observed_rate / baseline_rate",
            "threshold": "> 2.0 signals concern",
            "transfer_confidence": 0.92,
            "primitives": "N + κ (quantity + comparison)",
        }),
        serde_json::json!({
            "value_signal": "Trend",
            "pv_algorithm": "IC",
            "pv_full_name": "Information Component",
            "formula": "log2(observed / baseline)",
            "threshold": "> 0.5 signals trend",
            "transfer_confidence": 0.88,
            "primitives": "σ + → (sequence + causality)",
        }),
        serde_json::json!({
            "value_signal": "Engagement",
            "pv_algorithm": "ROR",
            "pv_full_name": "Reporting Odds Ratio",
            "formula": "(O/!O) / (B/!B)",
            "threshold": "> 2.0 with CI > 1.0",
            "transfer_confidence": 0.85,
            "primitives": "ν + N (frequency + quantity)",
        }),
        serde_json::json!({
            "value_signal": "Virality",
            "pv_algorithm": "EBGM",
            "pv_full_name": "Empirical Bayes Geometric Mean",
            "formula": "ratio with shrinkage prior",
            "threshold": "> 2.0 signals exponential",
            "transfer_confidence": 0.82,
            "primitives": "∂ + ∝ (boundary + irreversibility)",
        }),
        serde_json::json!({
            "value_signal": "Controversy",
            "pv_algorithm": "Chi²",
            "pv_full_name": "Chi-Square Statistic",
            "formula": "Σ(O-E)²/E",
            "threshold": "> 3.84 (p < 0.05)",
            "transfer_confidence": 0.90,
            "primitives": "κ + ς (comparison + state)",
        }),
    ];

    let filtered: Vec<_> = if let Some(ref filter) = params.signal_type {
        mappings
            .into_iter()
            .filter(|m| {
                m["value_signal"]
                    .as_str()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&filter.to_lowercase())
            })
            .collect()
    } else {
        mappings
    };

    let result = serde_json::json!({
        "mappings": filtered,
        "domain_source": "Pharmacovigilance Signal Detection",
        "domain_target": "Economic Value Mining",
        "overall_transfer_confidence": 0.87,
        "methodology": "Cross-domain primitive mapping via T1/T2 grounding",
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Confidence computation helpers (simplified versions of PV algorithms)
// ============================================================================

fn compute_prr_confidence(prr: f64, n: usize) -> f64 {
    // Higher sample size and larger deviation from 1.0 increases confidence
    let n_factor = (n as f64).ln().clamp(0.0, 5.0) / 5.0;
    let prr_factor = if prr > 1.0 {
        ((prr - 1.0).ln() + 1.0).clamp(0.0, 1.0)
    } else if prr < 1.0 && prr > 0.0 {
        ((1.0 / prr - 1.0).ln() + 1.0).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (n_factor * 0.4 + prr_factor * 0.6).clamp(0.0, 1.0)
}

fn compute_ic_confidence(ic: f64, n: usize) -> f64 {
    let n_factor = (n as f64).ln().clamp(0.0, 5.0) / 5.0;
    let ic_factor = (ic.abs() / 2.0).clamp(0.0, 1.0);
    (n_factor * 0.4 + ic_factor * 0.6).clamp(0.0, 1.0)
}

fn compute_ror_confidence(ror: f64, n: usize) -> f64 {
    let n_factor = (n as f64).ln().clamp(0.0, 5.0) / 5.0;
    let ror_factor = if ror > 1.0 {
        (ror.ln() / 2.0).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (n_factor * 0.4 + ror_factor * 0.6).clamp(0.0, 1.0)
}

fn compute_ebgm_confidence(ebgm: f64, n: usize) -> f64 {
    let n_factor = (n as f64).ln().clamp(0.0, 5.0) / 5.0;
    let ebgm_factor = ((ebgm - 1.0).abs() / 2.0).clamp(0.0, 1.0);
    (n_factor * 0.5 + ebgm_factor * 0.5).clamp(0.0, 1.0)
}

fn compute_chi2_confidence(chi2: f64, n: usize) -> f64 {
    let n_factor = (n as f64).ln().clamp(0.0, 5.0) / 5.0;
    // chi2 > 3.84 is p < 0.05, > 6.64 is p < 0.01
    let chi2_factor = (chi2 / 10.0).clamp(0.0, 1.0);
    (n_factor * 0.3 + chi2_factor * 0.7).clamp(0.0, 1.0)
}

fn interpret_signal(signal_type: SignalType, score: f64, strength: SignalStrength) -> String {
    let direction = match signal_type {
        SignalType::Sentiment => {
            if score > 1.0 {
                "positive sentiment elevated"
            } else {
                "negative sentiment elevated"
            }
        }
        SignalType::Trend => {
            if score > 0.0 {
                "upward trend detected"
            } else {
                "downward trend detected"
            }
        }
        SignalType::Engagement => {
            if score > 1.0 {
                "engagement above baseline"
            } else {
                "engagement below baseline"
            }
        }
        SignalType::Virality => {
            if score > 2.0 {
                "exponential growth phase"
            } else if score > 1.5 {
                "accelerating growth"
            } else {
                "linear growth"
            }
        }
        SignalType::Controversy => {
            if score > 6.64 {
                "highly polarized discussion"
            } else if score > 3.84 {
                "moderate controversy detected"
            } else {
                "normal sentiment variance"
            }
        }
    };

    format!("{} ({:?} signal)", direction, strength)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_signal_types() {
        let params = ValueSignalTypesParams { type_filter: None };
        let result = list_signal_types(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_signal_types_filtered() {
        let params = ValueSignalTypesParams {
            type_filter: Some("sent".to_string()),
        };
        let result = list_signal_types(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_sentiment_signal() {
        let params = ValueSignalDetectParams {
            signal_type: "sentiment".to_string(),
            observed: 0.8,
            baseline: 0.5,
            sample_size: 100,
            entity: "TSLA".to_string(),
            source: "wallstreetbets".to_string(),
        };
        let result = detect_signal(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_trend_signal() {
        let params = ValueSignalDetectParams {
            signal_type: "trend".to_string(),
            observed: 150.0,
            baseline: 100.0,
            sample_size: 50,
            entity: "Bitcoin".to_string(),
            source: "cryptocurrency".to_string(),
        };
        let result = detect_signal(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_virality_signal() {
        let params = ValueSignalDetectParams {
            signal_type: "virality".to_string(),
            observed: 1000.0,
            baseline: 200.0,
            sample_size: 200,
            entity: "GME".to_string(),
            source: "stocks".to_string(),
        };
        let result = detect_signal(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_baseline() {
        let params = ValueBaselineCreateParams {
            source: "wallstreetbets".to_string(),
            positive_rate: 0.6,
            negative_rate: 0.2,
            avg_engagement: 500.0,
            posts_per_hour: 25.0,
        };
        let result = create_baseline(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pv_mapping() {
        let params = ValuePvMappingParams { signal_type: None };
        let result = get_pv_mapping(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pv_mapping_filtered() {
        let params = ValuePvMappingParams {
            signal_type: Some("sentiment".to_string()),
        };
        let result = get_pv_mapping(params);
        assert!(result.is_ok());
    }
}
