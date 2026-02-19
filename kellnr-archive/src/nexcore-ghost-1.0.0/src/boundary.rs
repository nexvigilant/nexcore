//! # Anonymization Boundary
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | ∂ Boundary | Threshold gates for anonymization quality |
//! | N Quantity | k-anonymity and l-diversity numeric thresholds |
//!
//! ## Tier: T2-C (∂ Boundary + N Quantity)

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::mode::GhostMode;

/// Anonymization quality thresholds.
///
/// Parallel to `SafetyBoundary<T>` but for privacy metrics.
/// Defines the minimum acceptable anonymization quality.
///
/// ## Tier: T2-C (∂ Boundary + N Quantity)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnonymizationBoundary {
    /// Maximum acceptable re-identification risk (0.0 = impossible, 1.0 = certain).
    pub max_risk: f64,
    /// Minimum k-anonymity: each record must be indistinguishable from k-1 others.
    pub k_anonymity: u32,
    /// Minimum l-diversity: each equivalence class must have l distinct sensitive values.
    pub l_diversity: u32,
    /// Human-readable description of this boundary.
    pub description: String,
}

impl AnonymizationBoundary {
    /// Create from a GhostMode with sensible defaults.
    #[must_use]
    pub fn from_mode(mode: GhostMode) -> Self {
        match mode {
            GhostMode::Off => Self {
                max_risk: 1.0,
                k_anonymity: 0,
                l_diversity: 0,
                description: "No anonymization enforcement".to_string(),
            },
            GhostMode::Standard => Self {
                max_risk: 0.2,
                k_anonymity: 3,
                l_diversity: 2,
                description: "Standard: k≥3, l≥2, risk≤20%".to_string(),
            },
            GhostMode::Strict => Self {
                max_risk: 0.05,
                k_anonymity: 5,
                l_diversity: 3,
                description: "Strict: k≥5, l≥3, risk≤5%".to_string(),
            },
            GhostMode::Maximum => Self {
                max_risk: 0.01,
                k_anonymity: 10,
                l_diversity: 5,
                description: "Maximum: k≥10, l≥5, risk≤1%".to_string(),
            },
        }
    }

    /// Check if an observed risk level violates this boundary.
    #[must_use]
    pub fn is_risk_violated(&self, observed_risk: f64) -> bool {
        observed_risk > self.max_risk
    }

    /// Check if observed k-anonymity violates this boundary.
    #[must_use]
    pub fn is_k_violated(&self, observed_k: u32) -> bool {
        observed_k < self.k_anonymity
    }

    /// Check if observed l-diversity violates this boundary.
    #[must_use]
    pub fn is_l_violated(&self, observed_l: u32) -> bool {
        observed_l < self.l_diversity
    }

    /// Risk margin: negative = within bounds, positive = violation.
    #[must_use]
    pub fn risk_margin(&self, observed_risk: f64) -> f64 {
        observed_risk - self.max_risk
    }

    /// Check all thresholds at once.
    #[must_use]
    pub fn check_all(&self, risk: f64, k: u32, l: u32) -> BoundaryCheckResult {
        BoundaryCheckResult {
            risk_ok: !self.is_risk_violated(risk),
            k_ok: !self.is_k_violated(k),
            l_ok: !self.is_l_violated(l),
        }
    }
}

impl fmt::Display for AnonymizationBoundary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "∂[risk≤{:.2}, k≥{}, l≥{}]",
            self.max_risk, self.k_anonymity, self.l_diversity
        )
    }
}

/// Result of checking all boundary thresholds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryCheckResult {
    /// Whether risk is within bounds.
    pub risk_ok: bool,
    /// Whether k-anonymity is met.
    pub k_ok: bool,
    /// Whether l-diversity is met.
    pub l_ok: bool,
}

impl BoundaryCheckResult {
    /// True if all checks pass.
    #[must_use]
    pub const fn all_ok(&self) -> bool {
        self.risk_ok && self.k_ok && self.l_ok
    }

    /// Count of violations.
    #[must_use]
    pub fn violation_count(&self) -> u32 {
        let mut count = 0;
        if !self.risk_ok {
            count += 1;
        }
        if !self.k_ok {
            count += 1;
        }
        if !self.l_ok {
            count += 1;
        }
        count
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_mode_no_enforcement() {
        let b = AnonymizationBoundary::from_mode(GhostMode::Off);
        assert!(!b.is_risk_violated(0.99));
        assert!(!b.is_k_violated(0));
    }

    #[test]
    fn standard_mode_factory() {
        let b = AnonymizationBoundary::from_mode(GhostMode::Standard);
        assert_eq!(b.k_anonymity, 3);
        assert_eq!(b.l_diversity, 2);
        assert!((b.max_risk - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn strict_mode_factory() {
        let b = AnonymizationBoundary::from_mode(GhostMode::Strict);
        assert_eq!(b.k_anonymity, 5);
        assert!((b.max_risk - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn maximum_mode_factory() {
        let b = AnonymizationBoundary::from_mode(GhostMode::Maximum);
        assert_eq!(b.k_anonymity, 10);
        assert_eq!(b.l_diversity, 5);
    }

    #[test]
    fn risk_violation_detected() {
        let b = AnonymizationBoundary::from_mode(GhostMode::Standard);
        assert!(b.is_risk_violated(0.25)); // 25% > 20%
        assert!(!b.is_risk_violated(0.15)); // 15% < 20%
    }

    #[test]
    fn k_violation_detected() {
        let b = AnonymizationBoundary::from_mode(GhostMode::Strict);
        assert!(b.is_k_violated(4)); // 4 < 5
        assert!(!b.is_k_violated(5)); // 5 >= 5
        assert!(!b.is_k_violated(10)); // 10 >= 5
    }

    #[test]
    fn risk_margin_positive_when_violated() {
        let b = AnonymizationBoundary::from_mode(GhostMode::Standard);
        let margin = b.risk_margin(0.3);
        assert!(margin > 0.0);
    }

    #[test]
    fn risk_margin_negative_when_within() {
        let b = AnonymizationBoundary::from_mode(GhostMode::Standard);
        let margin = b.risk_margin(0.1);
        assert!(margin < 0.0);
    }

    #[test]
    fn check_all_passes() {
        let b = AnonymizationBoundary::from_mode(GhostMode::Standard);
        let result = b.check_all(0.1, 5, 3);
        assert!(result.all_ok());
        assert_eq!(result.violation_count(), 0);
    }

    #[test]
    fn check_all_partial_failure() {
        let b = AnonymizationBoundary::from_mode(GhostMode::Strict);
        let result = b.check_all(0.01, 3, 3); // k=3 < 5 required
        assert!(!result.all_ok());
        assert!(!result.k_ok);
        assert!(result.risk_ok);
        assert!(result.l_ok);
        assert_eq!(result.violation_count(), 1);
    }

    #[test]
    fn display_format() {
        let b = AnonymizationBoundary::from_mode(GhostMode::Strict);
        let s = format!("{b}");
        assert!(s.contains("k≥5"));
        assert!(s.contains("l≥3"));
    }

    #[test]
    fn serde_roundtrip() {
        let b = AnonymizationBoundary::from_mode(GhostMode::Standard);
        let json = serde_json::to_string(&b).unwrap_or_default();
        let back: std::result::Result<AnonymizationBoundary, _> = serde_json::from_str(&json);
        assert!(back.is_ok());
    }
}
