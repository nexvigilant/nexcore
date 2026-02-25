//! Entropy computation MCP tools
//!
//! Unified interface for Shannon entropy, cross-entropy, KL divergence,
//! mutual information, normalized and conditional entropy.

use crate::params::EntropyComputeParams;
use nexcore_primitives::entropy::{self, LogBase};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Parse log base string to LogBase enum.
fn parse_base(s: &str) -> Result<LogBase, McpError> {
    match s.to_lowercase().as_str() {
        "bits" | "bit" | "log2" => Ok(LogBase::Bits),
        "nats" | "nat" | "ln" | "natural" => Ok(LogBase::Nats),
        "hartleys" | "hartley" | "bans" | "log10" => Ok(LogBase::Hartleys),
        other => Err(McpError::invalid_params(
            format!("Unknown log base '{other}'. Use: bits, nats, or hartleys"),
            None,
        )),
    }
}

/// Build a flat joint distribution matrix from a flat vector + row count.
fn build_joint_matrix(flat: &[f64], rows: usize) -> Result<Vec<Vec<f64>>, McpError> {
    if rows == 0 || flat.len() % rows != 0 {
        return Err(McpError::invalid_params(
            format!(
                "Joint matrix: {} elements not divisible by {} rows",
                flat.len(),
                rows
            ),
            None,
        ));
    }
    let cols = flat.len() / rows;
    Ok(flat.chunks(cols).map(|c| c.to_vec()).collect())
}

/// Generate human-readable interpretation of entropy value.
fn interpret_entropy(value: f64, base: LogBase) -> String {
    let unit = base.unit_name();
    if value < 1e-10 {
        format!(
            "Entropy of 0.0 {unit} indicates a deterministic (perfectly predictable) distribution"
        )
    } else {
        // Approximate equivalent number of equally likely outcomes
        let equiv_outcomes = match base {
            LogBase::Bits => 2.0_f64.powf(value),
            LogBase::Nats => value.exp(),
            LogBase::Hartleys => 10.0_f64.powf(value),
        };
        format!(
            "Entropy of {value:.4} {unit} corresponds to approximately {equiv_outcomes:.1} equally likely outcomes"
        )
    }
}

/// Compute entropy, cross-entropy, KL divergence, mutual information,
/// normalized or conditional entropy from probability distributions or counts.
pub fn entropy_compute(params: EntropyComputeParams) -> Result<CallToolResult, McpError> {
    let base = parse_base(&params.base)?;

    let mode = params.mode.to_lowercase();

    match mode.as_str() {
        "shannon" => compute_shannon(params, base),
        "cross" => compute_cross_entropy(params, base),
        "kl" => compute_kl(params, base),
        "mutual" => compute_mutual_info(params, base),
        "normalized" => compute_normalized(params, base),
        "conditional" => compute_conditional(params, base),
        other => Err(McpError::invalid_params(
            format!(
                "Unknown mode '{other}'. Use: shannon, cross, kl, mutual, normalized, conditional"
            ),
            None,
        )),
    }
}

fn compute_shannon(
    params: EntropyComputeParams,
    base: LogBase,
) -> Result<CallToolResult, McpError> {
    let result = if params.from_counts {
        let counts: Vec<u64> = params.distribution_p.iter().map(|&v| v as u64).collect();
        let m = entropy::shannon_entropy_measured_with_base(&counts, base)
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        json!({
            "value": (m.value.bits * 1_000_000.0).round() / 1_000_000.0,
            "normalized": (m.value.normalized * 1_000_000.0).round() / 1_000_000.0,
            "sample_count": m.value.sample_count,
            "confidence": (m.confidence.value() * 1_000_000.0).round() / 1_000_000.0,
            "base": base.unit_name(),
            "interpretation": interpret_entropy(m.value.bits, base),
            "grounding": "N(Quantity) + ∝(Irreversibility) + κ(Comparison)",
        })
    } else {
        let r = entropy::shannon_entropy_with_base(&params.distribution_p, base)
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        json!({
            "value": (r.bits * 1_000_000.0).round() / 1_000_000.0,
            "normalized": (r.normalized * 1_000_000.0).round() / 1_000_000.0,
            "sample_count": r.sample_count,
            "base": base.unit_name(),
            "interpretation": interpret_entropy(r.bits, base),
            "grounding": "N(Quantity) + ∝(Irreversibility) + κ(Comparison)",
        })
    };
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn compute_cross_entropy(
    params: EntropyComputeParams,
    base: LogBase,
) -> Result<CallToolResult, McpError> {
    let q = params
        .distribution_q
        .ok_or_else(|| McpError::invalid_params("Cross-entropy requires distribution_q", None))?;
    let ce = entropy::cross_entropy(&params.distribution_p, &q, base)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
    let result = json!({
        "value": (ce * 1_000_000.0).round() / 1_000_000.0,
        "base": base.unit_name(),
        "interpretation": format!(
            "Cross-entropy of {:.4} {} means encoding P using Q's code requires {:.4} {} per symbol",
            ce, base.unit_name(), ce, base.unit_name()
        ),
        "grounding": "N(Quantity) + ∝(Irreversibility) + κ(Comparison)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn compute_kl(params: EntropyComputeParams, base: LogBase) -> Result<CallToolResult, McpError> {
    let q = params
        .distribution_q
        .ok_or_else(|| McpError::invalid_params("KL divergence requires distribution_q", None))?;
    let kl = entropy::kl_divergence_with_base(&params.distribution_p, &q, base)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
    let result = json!({
        "value": (kl * 1_000_000.0).round() / 1_000_000.0,
        "base": base.unit_name(),
        "interpretation": format!(
            "KL divergence of {:.4} {} means P requires {:.4} extra {} when encoded using Q",
            kl, base.unit_name(), kl, base.unit_name()
        ),
        "grounding": "N(Quantity) + ∝(Irreversibility) + κ(Comparison)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn compute_mutual_info(
    params: EntropyComputeParams,
    base: LogBase,
) -> Result<CallToolResult, McpError> {
    let rows = params
        .joint_rows
        .ok_or_else(|| McpError::invalid_params("Mutual information requires joint_rows", None))?;
    let joint = build_joint_matrix(&params.distribution_p, rows)?;
    let mi = entropy::mutual_information_with_base(&joint, base)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
    let result = json!({
        "value": (mi * 1_000_000.0).round() / 1_000_000.0,
        "base": base.unit_name(),
        "interpretation": format!(
            "Mutual information of {:.4} {} — shared information between X and Y",
            mi, base.unit_name()
        ),
        "grounding": "N(Quantity) + ∝(Irreversibility) + κ(Comparison)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn compute_normalized(
    params: EntropyComputeParams,
    base: LogBase,
) -> Result<CallToolResult, McpError> {
    let n = entropy::normalized_entropy(&params.distribution_p, base)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
    let result = json!({
        "value": (n * 1_000_000.0).round() / 1_000_000.0,
        "range": "[0, 1]",
        "interpretation": format!(
            "Normalized entropy {:.4} — 0 = deterministic, 1 = maximum disorder",
            n
        ),
        "grounding": "N(Quantity) + ∝(Irreversibility) + κ(Comparison)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn compute_conditional(
    params: EntropyComputeParams,
    base: LogBase,
) -> Result<CallToolResult, McpError> {
    let rows = params
        .joint_rows
        .ok_or_else(|| McpError::invalid_params("Conditional entropy requires joint_rows", None))?;
    let joint = build_joint_matrix(&params.distribution_p, rows)?;
    let h_yx = entropy::conditional_entropy(&joint, base)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
    let result = json!({
        "value": (h_yx * 1_000_000.0).round() / 1_000_000.0,
        "base": base.unit_name(),
        "interpretation": format!(
            "Conditional entropy H(Y|X) = {:.4} {} — remaining uncertainty about Y given X",
            h_yx, base.unit_name()
        ),
        "grounding": "N(Quantity) + ∝(Irreversibility) + κ(Comparison)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shannon_from_probs() {
        let params = EntropyComputeParams {
            mode: "shannon".to_string(),
            distribution_p: vec![0.5, 0.5],
            distribution_q: None,
            joint_rows: None,
            base: "bits".to_string(),
            from_counts: false,
        };
        let result = entropy_compute(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_shannon_from_counts() {
        let params = EntropyComputeParams {
            mode: "shannon".to_string(),
            distribution_p: vec![50.0, 50.0],
            distribution_q: None,
            joint_rows: None,
            base: "nats".to_string(),
            from_counts: true,
        };
        let result = entropy_compute(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cross_entropy_mode() {
        let params = EntropyComputeParams {
            mode: "cross".to_string(),
            distribution_p: vec![0.9, 0.1],
            distribution_q: Some(vec![0.5, 0.5]),
            joint_rows: None,
            base: "bits".to_string(),
            from_counts: false,
        };
        let result = entropy_compute(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_kl_mode() {
        let params = EntropyComputeParams {
            mode: "kl".to_string(),
            distribution_p: vec![0.9, 0.1],
            distribution_q: Some(vec![0.5, 0.5]),
            joint_rows: None,
            base: "bits".to_string(),
            from_counts: false,
        };
        let result = entropy_compute(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mutual_info_mode() {
        let params = EntropyComputeParams {
            mode: "mutual".to_string(),
            distribution_p: vec![0.25, 0.25, 0.25, 0.25],
            distribution_q: None,
            joint_rows: Some(2),
            base: "bits".to_string(),
            from_counts: false,
        };
        let result = entropy_compute(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_normalized_mode() {
        let params = EntropyComputeParams {
            mode: "normalized".to_string(),
            distribution_p: vec![0.25, 0.25, 0.25, 0.25],
            distribution_q: None,
            joint_rows: None,
            base: "bits".to_string(),
            from_counts: false,
        };
        let result = entropy_compute(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_conditional_mode() {
        let params = EntropyComputeParams {
            mode: "conditional".to_string(),
            distribution_p: vec![0.25, 0.25, 0.25, 0.25],
            distribution_q: None,
            joint_rows: Some(2),
            base: "bits".to_string(),
            from_counts: false,
        };
        let result = entropy_compute(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_mode() {
        let params = EntropyComputeParams {
            mode: "invalid".to_string(),
            distribution_p: vec![0.5, 0.5],
            distribution_q: None,
            joint_rows: None,
            base: "bits".to_string(),
            from_counts: false,
        };
        let result = entropy_compute(params);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_base() {
        let params = EntropyComputeParams {
            mode: "shannon".to_string(),
            distribution_p: vec![0.5, 0.5],
            distribution_q: None,
            joint_rows: None,
            base: "invalid_base".to_string(),
            from_counts: false,
        };
        let result = entropy_compute(params);
        assert!(result.is_err());
    }

    #[test]
    fn test_cross_missing_q() {
        let params = EntropyComputeParams {
            mode: "cross".to_string(),
            distribution_p: vec![0.5, 0.5],
            distribution_q: None,
            joint_rows: None,
            base: "bits".to_string(),
            from_counts: false,
        };
        let result = entropy_compute(params);
        assert!(result.is_err());
    }
}
