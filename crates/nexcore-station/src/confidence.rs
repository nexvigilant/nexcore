//! Confidence computation for station resolution.

/// Weight for schema completeness signal.
pub const SCHEMA_WEIGHT: f64 = 0.35;

/// Weight for CSS selector presence signal.
pub const SELECTOR_WEIGHT: f64 = 0.35;

/// Weight for explicit human verification.
pub const VERIFIED_WEIGHT: f64 = 0.30;

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
}
