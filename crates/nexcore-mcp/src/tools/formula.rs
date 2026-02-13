//! Formula-derived tools from Knowledge Unit extraction.
//!
//! Five formulas extracted from the archive-extractor pipeline (Phase 1, 99 KUs)
//! converted directly to MCP tools via the Formula→Tool causal chain.
//!
//! | Tool | Formula | Source KU |
//! |------|---------|-----------|
//! | pv_signal_strength | S = U × R × T | F-011 / KU-023 |
//! | foundation_domain_distance | d = 1 - overlap(A,B) | F-004 / KU-025 |
//! | foundation_flywheel_velocity | v = 1/avg(Δt) | F-007 / KU-015 |
//! | foundation_token_ratio | r = tokens/ops | F-008 / KU-016 |
//! | foundation_spectral_overlap | cos(a,b) | F-015 / KU-048 |

use crate::params::{
    DomainDistanceParams, FlywheelVelocityParams, SignalStrengthParams, SpectralOverlapParams,
    TokenRatioParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ============================================================================
// F-011: Signal Strength Composite (S = U × R × T)
// Tier: T1 — pure numeric product (N × N × N → N)
// ============================================================================

/// Compute signal strength composite: S = Unexpectedness × Robustness × Therapeutic_importance.
///
/// All inputs should be in [0.0, 1.0]. Output is in [0.0, 1.0].
/// Higher values indicate stronger, more actionable signals.
pub fn signal_strength(params: SignalStrengthParams) -> Result<CallToolResult, McpError> {
    let u = params.unexpectedness;
    let r = params.robustness;
    let t = params.therapeutic_importance;

    // Validate bounds
    for (name, val) in [
        ("unexpectedness", u),
        ("robustness", r),
        ("therapeutic_importance", t),
    ] {
        if !(0.0..=1.0).contains(&val) {
            return Err(McpError::invalid_params(
                format!("{name} must be in [0.0, 1.0], got {val}"),
                None,
            ));
        }
    }

    let strength = u * r * t;

    // Classification based on composite score
    let classification = if strength >= 0.5 {
        "strong"
    } else if strength >= 0.2 {
        "moderate"
    } else if strength >= 0.05 {
        "weak"
    } else {
        "negligible"
    };

    let result = json!({
        "signal_strength": (strength * 10000.0).round() / 10000.0,
        "unexpectedness": u,
        "robustness": r,
        "therapeutic_importance": t,
        "classification": classification,
        "formula": "S = U × R × T",
        "source": "KU F-011 (ToV extraction)"
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// F-004: Domain Distance (d = 1 - weighted_overlap)
// Tier: T2-C — comparison + summation + mapping
// ============================================================================

/// Compute domain distance based on weighted primitive overlap.
///
/// Primitives are classified by tier:
/// - T1 (universal): σ, μ, ς, ρ, ∅, ∂, ν, ∃, π, →, κ, N, λ, ∝, Σ, ×
/// - Unrecognized primitives are treated as T3 (domain-specific)
///
/// distance = 1 - (w1 × T1_overlap + w2 × T2_overlap + w3 × T3_overlap)
pub fn domain_distance(params: DomainDistanceParams) -> Result<CallToolResult, McpError> {
    // T1 primitives (the 16 Lex Primitiva)
    let t1_names: Vec<&str> = vec![
        "sequence",
        "sigma",
        "σ",
        "mapping",
        "mu",
        "μ",
        "state",
        "varsigma",
        "ς",
        "recursion",
        "rho",
        "ρ",
        "void",
        "emptyset",
        "∅",
        "boundary",
        "partial",
        "∂",
        "frequency",
        "nu",
        "ν",
        "existence",
        "exists",
        "∃",
        "persistence",
        "pi",
        "π",
        "causality",
        "rightarrow",
        "→",
        "comparison",
        "kappa",
        "κ",
        "quantity",
        "n",
        "location",
        "lambda",
        "λ",
        "irreversibility",
        "propto",
        "∝",
        "sum",
        "coproduct",
        "Σ",
        "product",
        "times",
        "×",
    ];

    let normalize = |s: &str| s.to_lowercase();

    let a_set: std::collections::HashSet<String> =
        params.primitives_a.iter().map(|s| normalize(s)).collect();
    let b_set: std::collections::HashSet<String> =
        params.primitives_b.iter().map(|s| normalize(s)).collect();

    // Classify each primitive
    let is_t1 = |s: &str| t1_names.contains(&s.to_lowercase().as_str());

    let a_t1: std::collections::HashSet<&String> = a_set.iter().filter(|s| is_t1(s)).collect();
    let b_t1: std::collections::HashSet<&String> = b_set.iter().filter(|s| is_t1(s)).collect();
    let a_t3: std::collections::HashSet<&String> = a_set.iter().filter(|s| !is_t1(s)).collect();
    let b_t3: std::collections::HashSet<&String> = b_set.iter().filter(|s| !is_t1(s)).collect();

    // Jaccard overlap per tier
    let jaccard =
        |a: &std::collections::HashSet<&String>, b: &std::collections::HashSet<&String>| -> f64 {
            let intersection = a.intersection(b).count() as f64;
            let union = a.union(b).count() as f64;
            if union == 0.0 {
                1.0 // Both empty = identical
            } else {
                intersection / union
            }
        };

    let t1_overlap = jaccard(&a_t1, &b_t1);
    // T2 overlap approximated as overall overlap minus T1 and T3
    let all_a: std::collections::HashSet<&String> = a_set.iter().collect();
    let all_b: std::collections::HashSet<&String> = b_set.iter().collect();
    let total_overlap = jaccard(&all_a, &all_b);
    let t3_overlap = jaccard(&a_t3, &b_t3);

    // T2 = residual from total that isn't T1 or T3
    let t2_overlap = total_overlap;

    let weighted = params.w1 * t1_overlap + params.w2 * t2_overlap + params.w3 * t3_overlap;
    let distance = (1.0 - weighted).max(0.0).min(1.0);

    let classification = if distance <= 0.2 {
        "very_close"
    } else if distance <= 0.4 {
        "close"
    } else if distance <= 0.6 {
        "moderate"
    } else if distance <= 0.8 {
        "distant"
    } else {
        "very_distant"
    };

    let result = json!({
        "distance": (distance * 10000.0).round() / 10000.0,
        "t1_overlap": (t1_overlap * 10000.0).round() / 10000.0,
        "t2_overlap": (t2_overlap * 10000.0).round() / 10000.0,
        "t3_overlap": (t3_overlap * 10000.0).round() / 10000.0,
        "weighted_overlap": (weighted * 10000.0).round() / 10000.0,
        "weights": { "w1_t1": params.w1, "w2_t2": params.w2, "w3_t3": params.w3 },
        "classification": classification,
        "primitives_a_count": params.primitives_a.len(),
        "primitives_b_count": params.primitives_b.len(),
        "shared_count": all_a.intersection(&all_b).count(),
        "formula": "distance = 1 - (w1×T1_overlap + w2×T2_overlap + w3×T3_overlap)",
        "source": "KU F-004 (Domain Discovery Book)"
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// F-007: Flywheel Velocity (v = 1 / avg_time(failure → fix))
// Tier: T2-P — frequency (inverse of average duration)
// ============================================================================

/// Compute flywheel velocity from paired failure/fix timestamps.
///
/// Higher velocity = faster improvement cycle = stronger compound growth.
/// velocity = 1 / avg(fix_time - failure_time) in events per millisecond.
pub fn flywheel_velocity(params: FlywheelVelocityParams) -> Result<CallToolResult, McpError> {
    if params.failure_timestamps.len() != params.fix_timestamps.len() {
        return Err(McpError::invalid_params(
            format!(
                "failure_timestamps ({}) and fix_timestamps ({}) must have same length",
                params.failure_timestamps.len(),
                params.fix_timestamps.len()
            ),
            None,
        ));
    }

    if params.failure_timestamps.is_empty() {
        return Err(McpError::invalid_params(
            "Need at least one failure/fix pair".to_string(),
            None,
        ));
    }

    let mut deltas: Vec<f64> = Vec::new();
    let mut invalid_pairs = 0u64;

    for (f, x) in params
        .failure_timestamps
        .iter()
        .zip(params.fix_timestamps.iter())
    {
        if x >= f {
            deltas.push((*x - *f) as f64);
        } else {
            invalid_pairs += 1;
        }
    }

    if deltas.is_empty() {
        return Err(McpError::invalid_params(
            "All pairs have fix_time < failure_time".to_string(),
            None,
        ));
    }

    let avg_delta_ms = deltas.iter().sum::<f64>() / deltas.len() as f64;
    let avg_delta_hours = avg_delta_ms / 3_600_000.0;
    let velocity_per_hour = if avg_delta_hours > 0.0 {
        1.0 / avg_delta_hours
    } else {
        f64::INFINITY
    };

    let min_delta = deltas.iter().copied().fold(f64::INFINITY, f64::min);
    let max_delta = deltas.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    let classification = if avg_delta_hours <= 1.0 {
        "exceptional"
    } else if avg_delta_hours <= 24.0 {
        "target"
    } else if avg_delta_hours <= 168.0 {
        "acceptable"
    } else {
        "slow"
    };

    let result = json!({
        "velocity_per_hour": (velocity_per_hour * 10000.0).round() / 10000.0,
        "avg_delta_ms": avg_delta_ms.round(),
        "avg_delta_hours": (avg_delta_hours * 100.0).round() / 100.0,
        "min_delta_hours": ((min_delta / 3_600_000.0) * 100.0).round() / 100.0,
        "max_delta_hours": ((max_delta / 3_600_000.0) * 100.0).round() / 100.0,
        "valid_pairs": deltas.len(),
        "invalid_pairs": invalid_pairs,
        "classification": classification,
        "target": "< 24 hours (velocity > 0.042/hr)",
        "formula": "velocity = 1 / avg_time(failure → fix)",
        "source": "KU F-007 (KPI Framework)"
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// F-008: Token Ratio (r = LLM_tokens / semantic_operations)
// Tier: T2-P — quantity ratio
// ============================================================================

/// Compute token-to-operation ratio for LLM code generation efficiency.
///
/// Target: ≤ 1.0 (every token carries semantic meaning).
/// Lower is better — 0.33 means 3 operations per token.
pub fn token_ratio(params: TokenRatioParams) -> Result<CallToolResult, McpError> {
    if params.operation_count == 0 {
        return Err(McpError::invalid_params(
            "operation_count must be > 0".to_string(),
            None,
        ));
    }

    let ratio = params.token_count as f64 / params.operation_count as f64;

    let classification = if ratio <= 0.5 {
        "excellent"
    } else if ratio <= 1.0 {
        "target"
    } else if ratio <= 2.0 {
        "verbose"
    } else {
        "wasteful"
    };

    let efficiency = if ratio > 0.0 { 1.0 / ratio } else { 0.0 };

    let result = json!({
        "token_ratio": (ratio * 10000.0).round() / 10000.0,
        "operations_per_token": (efficiency * 10000.0).round() / 10000.0,
        "token_count": params.token_count,
        "operation_count": params.operation_count,
        "classification": classification,
        "target": "≤ 1.0",
        "formula": "ratio = LLM_tokens / semantic_operations",
        "source": "KU F-008 (DNA SPEC-v3)"
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// F-015: Spectral Overlap (cosine similarity)
// Tier: T2-P — comparison via dot product
// ============================================================================

/// Compute spectral overlap (cosine similarity) between two feature vectors.
///
/// overlap = (a · b) / (‖a‖ × ‖b‖) ∈ [-1, 1]
/// For autocorrelation spectra, values are typically in [0, 1].
pub fn spectral_overlap(params: SpectralOverlapParams) -> Result<CallToolResult, McpError> {
    if params.spectrum_a.len() != params.spectrum_b.len() {
        return Err(McpError::invalid_params(
            format!(
                "Spectra must have same dimensionality: {} vs {}",
                params.spectrum_a.len(),
                params.spectrum_b.len()
            ),
            None,
        ));
    }

    if params.spectrum_a.is_empty() {
        return Err(McpError::invalid_params(
            "Spectra must not be empty".to_string(),
            None,
        ));
    }

    let dot: f64 = params
        .spectrum_a
        .iter()
        .zip(params.spectrum_b.iter())
        .map(|(a, b)| a * b)
        .sum();

    let norm_a: f64 = params.spectrum_a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = params.spectrum_b.iter().map(|x| x * x).sum::<f64>().sqrt();

    let denominator = norm_a * norm_b;

    if denominator == 0.0 {
        return Err(McpError::invalid_params(
            "One or both spectra are zero vectors".to_string(),
            None,
        ));
    }

    let overlap = dot / denominator;

    let classification = if overlap >= 0.9 {
        "highly_similar"
    } else if overlap >= 0.7 {
        "similar"
    } else if overlap >= 0.4 {
        "moderate"
    } else if overlap >= 0.0 {
        "dissimilar"
    } else {
        "anti_correlated"
    };

    let result = json!({
        "overlap": (overlap * 10000.0).round() / 10000.0,
        "dot_product": (dot * 10000.0).round() / 10000.0,
        "norm_a": (norm_a * 10000.0).round() / 10000.0,
        "norm_b": (norm_b * 10000.0).round() / 10000.0,
        "dimensionality": params.spectrum_a.len(),
        "classification": classification,
        "formula": "overlap = (a · b) / (‖a‖ × ‖b‖)",
        "source": "KU F-015 (DNA SOP-001)"
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}
