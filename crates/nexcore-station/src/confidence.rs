//! Confidence computation for station resolution.

/// Weight for schema completeness signal.
pub const SCHEMA_WEIGHT: f64 = 0.35;

/// Weight for CSS selector presence signal.
pub const SELECTOR_WEIGHT: f64 = 0.35;

/// Weight for explicit human verification.
pub const VERIFIED_WEIGHT: f64 = 0.30;

/// Daily decay rate for confidence when no re-verification occurs.
///
/// At this rate, confidence drops below 80% of its original value after ~11 days.
/// Derived from knowledge-hygiene-reflexes: `effective = base * e^(-0.02 * days)`.
// CALIBRATION: λ = 0.02/day. Half-life ≈ 34.7 days (ln(2)/0.02).
// At t=11d: e^(-0.22) ≈ 0.802 → effective drops below 80% threshold.
pub const DAILY_DECAY_RATE: f64 = 0.02;

/// Compute a confidence score from Observatory quality signals.
///
/// Returns a score in `[0.0, 1.0]` as a weighted sum of three boolean inputs.
#[must_use]
pub fn compute_confidence(schema_complete: bool, selector_present: bool, verified: bool) -> f64 {
    let mut score = 0.0_f64;
    if schema_complete {
        score += SCHEMA_WEIGHT;
    }
    if selector_present {
        score += SELECTOR_WEIGHT;
    }
    if verified {
        score += VERIFIED_WEIGHT;
    }
    score
}

/// Apply temporal decay to a base confidence score.
///
/// Uses exponential decay: `effective = base * e^(-λt)` where λ = [`DAILY_DECAY_RATE`].
/// Returns the decayed confidence, clamped to `[0.0, 1.0]`.
///
/// # Arguments
/// - `base_confidence`: The confidence at time of last verification, in `[0.0, 1.0]`.
/// - `days_since_verification`: Elapsed time in fractional days.
#[must_use]
pub fn apply_staleness_decay(base_confidence: f64, days_since_verification: f64) -> f64 {
    if days_since_verification <= 0.0 {
        return base_confidence.clamp(0.0, 1.0);
    }
    let decayed = base_confidence * (-DAILY_DECAY_RATE * days_since_verification).exp();
    decayed.clamp(0.0, 1.0)
}

/// Check whether a confidence score is stale (below 80% of its original value).
///
/// Returns `true` when temporal decay has eroded the score past the 80% threshold.
#[must_use]
pub fn is_stale(base_confidence: f64, days_since_verification: f64) -> bool {
    let effective = apply_staleness_decay(base_confidence, days_since_verification);
    effective < base_confidence * 0.80
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_true_yields_one() {
        let c = compute_confidence(true, true, true);
        assert!((c - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn all_false_yields_zero() {
        let c = compute_confidence(false, false, false);
        assert!((c - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn partial_signals() {
        let c = compute_confidence(true, false, true);
        assert!((c - 0.65).abs() < f64::EPSILON);
    }

    // ---- staleness decay tests ----

    #[test]
    fn zero_days_returns_base() {
        let c = apply_staleness_decay(0.65, 0.0);
        assert!((c - 0.65).abs() < f64::EPSILON);
    }

    #[test]
    fn negative_days_returns_base() {
        let c = apply_staleness_decay(0.65, -5.0);
        assert!((c - 0.65).abs() < f64::EPSILON);
    }

    #[test]
    fn decay_at_11_days_below_80_percent() {
        // e^(-0.02 * 11) ≈ 0.8025 → effective = 0.65 * 0.8025 ≈ 0.5216
        // 80% of 0.65 = 0.52 → effective ≈ 0.5216, borderline
        let base = 0.65;
        let effective = apply_staleness_decay(base, 11.2);
        assert!(effective < base * 0.80, "should be stale at ~11 days");
    }

    #[test]
    fn not_stale_at_5_days() {
        assert!(!is_stale(0.65, 5.0));
    }

    #[test]
    fn stale_at_12_days() {
        assert!(is_stale(0.65, 12.0));
    }

    #[test]
    fn decay_clamps_to_zero_floor() {
        let c = apply_staleness_decay(0.5, 10_000.0);
        assert!(c >= 0.0);
    }
}
