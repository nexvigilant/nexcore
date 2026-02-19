//! Safety monitoring per ICH E2A/E2C and E20 §3.
//!
//! Tier: T2-P (∂+N — Safety Monitoring)
//!
//! Checks observed safety metrics against pre-specified stopping boundaries.

use crate::types::{SafetyCheckResult, SafetyRule};

/// Check whether the observed safety metric crosses the pre-specified boundary.
///
/// Returns `is_safe = true` if `observed_value` is BELOW the threshold (trial continues).
/// Returns `is_safe = false` if threshold is crossed (trial must stop).
///
/// # Arguments
/// - `rule`: The `SafetyRule` from the registered protocol
/// - `observed_value`: The current observed metric value (e.g., SAE rate)
///
/// # Returns
/// `SafetyCheckResult` with verdict, margin, and context.
pub fn check_safety_boundary(rule: &SafetyRule, observed_value: f64) -> SafetyCheckResult {
    let margin = rule.threshold - observed_value;
    let is_safe = observed_value < rule.threshold;

    SafetyCheckResult {
        is_safe,
        metric: rule.metric.clone(),
        observed: observed_value,
        threshold: rule.threshold,
        margin,
    }
}

/// Compute safety event rate from counts.
///
/// Convenience helper: `events / subjects`.
pub fn safety_event_rate(events: u32, subjects: u32) -> f64 {
    if subjects == 0 { return 0.0; }
    events as f64 / subjects as f64
}

/// Generate a PV-compatible disproportionality context string from a safety check.
///
/// Output format aligns with `pv_signal_complete` input expectations so downstream
/// pharmacovigilance tools can ingest safety signals from trial surveillance.
pub fn format_pv_context(result: &SafetyCheckResult) -> String {
    let status = if result.is_safe { "WITHIN_BOUNDARY" } else { "BOUNDARY_CROSSED" };
    format!(
        "metric={} observed={:.4} threshold={:.4} margin={:.4} status={}",
        result.metric, result.observed, result.threshold, result.margin, status
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SafetyRule;

    fn make_rule(threshold: f64) -> SafetyRule {
        SafetyRule {
            metric: "sae_rate".into(),
            threshold,
            description: format!("Stop if SAE rate > {threshold}"),
        }
    }

    #[test]
    fn test_safety_boundary_not_crossed() {
        let rule = make_rule(0.02);
        let result = check_safety_boundary(&rule, 0.01);
        assert!(result.is_safe, "0.01 < 0.02 — should be safe");
        assert!(result.margin > 0.0, "Margin should be positive");
    }

    #[test]
    fn test_safety_boundary_crossed() {
        let rule = make_rule(0.02);
        let result = check_safety_boundary(&rule, 0.03);
        assert!(!result.is_safe, "0.03 > 0.02 — boundary crossed");
        assert!(result.margin < 0.0, "Margin should be negative when crossed");
    }

    #[test]
    fn test_safety_boundary_at_threshold() {
        let rule = make_rule(0.05);
        // Exactly at threshold: not crossed (strict <)
        let result = check_safety_boundary(&rule, 0.05);
        assert!(!result.is_safe, "Exact threshold should be flagged (not strict <)");
        assert!((result.margin).abs() < 1e-10);
    }

    #[test]
    fn test_safety_event_rate() {
        let rate = safety_event_rate(4, 200);
        assert!((rate - 0.02).abs() < 1e-10);
    }

    #[test]
    fn test_safety_event_rate_zero_subjects() {
        assert_eq!(safety_event_rate(5, 0), 0.0);
    }

    #[test]
    fn test_pv_context_format() {
        let rule = make_rule(0.02);
        let result = check_safety_boundary(&rule, 0.01);
        let ctx = format_pv_context(&result);
        assert!(ctx.contains("WITHIN_BOUNDARY"));
        assert!(ctx.contains("sae_rate"));
    }
}
