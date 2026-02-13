//! Stage 9: Theory of Vigilance Safety Margin.
//!
//! Computes d(s) — signed distance from the safety boundary.
//! Negative values = operating in dangerous territory.
//!
//! Tier: T2-C | Dominant: ∂ (Boundary) + ∝ (Irreversibility).

use crate::projection::Point3D;
use crate::thermodynamics::Tension;
use serde::{Deserialize, Serialize};

/// Safety level classification based on d(s).
///
/// Tier: T2-P | Grounds to: ∂ (Boundary) + κ (Comparison).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SafetyLevel {
    /// d(s) < -0.5 — deep in dangerous territory.
    Critical,
    /// -0.5 ≤ d(s) < 0.0 — near the boundary, caution required.
    Warning,
    /// 0.0 ≤ d(s) < 0.5 — safely within normal operating range.
    Normal,
    /// d(s) ≥ 0.5 — well-insulated from risk.
    Safe,
}

impl SafetyLevel {
    /// Whether this level requires immediate attention.
    #[must_use]
    pub const fn is_actionable(&self) -> bool {
        matches!(self, Self::Critical | Self::Warning)
    }
}

impl std::fmt::Display for SafetyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "CRITICAL"),
            Self::Warning => write!(f, "WARNING"),
            Self::Normal => write!(f, "NORMAL"),
            Self::Safe => write!(f, "SAFE"),
        }
    }
}

/// Safety margin: signed distance from the vigilance boundary.
///
/// d(s) = boundary - risk_score
///
/// Where risk is derived from:
/// - Spectral entropy (z-axis): harmonic complexity → activation potential
/// - Tension (normalized): thermodynamic stress → instability
/// - AT-richness (1 - GC ratio): promoter-like character → activation signal
///
/// Tier: T3 | Grounds to: ∂ (Boundary) + ∝ (Irreversibility) + κ (Comparison)
///         + N (Quantity) + λ (Location) + → (Causality).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyMargin {
    /// Signed distance from safety boundary.
    /// Negative = operating in dangerous territory.
    pub distance: f64,
    /// Classified safety level.
    pub level: SafetyLevel,
    /// Raw risk score before boundary subtraction.
    pub risk_score: f64,
}

/// Neutral safety boundary — below this is "safe", above is "at risk".
const SAFETY_BOUNDARY: f64 = 0.5;

/// Weight for spectral entropy contribution to risk.
const SPECTRAL_WEIGHT: f64 = 0.4;
/// Weight for tension contribution to risk.
const TENSION_WEIGHT: f64 = 0.3;
/// Weight for AT-richness (promoter character) contribution to risk.
const AT_RICH_WEIGHT: f64 = 0.3;

impl SafetyMargin {
    /// Compute safety margin from 3D position and tension.
    ///
    /// Risk formula:
    /// risk = spectral_entropy × 0.4 + tension × 0.3 + (1 - GC_ratio) × 0.3
    ///
    /// d(s) = boundary - risk
    #[must_use]
    pub fn compute(point: &Point3D, tension: &Tension) -> Self {
        // Normalize spectral entropy to [0, 1] range
        // Typical range: 0-5 nats → divide by 5 and clamp
        let normalized_z = (point.z / 5.0).clamp(0.0, 1.0);

        let risk_score = normalized_z * SPECTRAL_WEIGHT
            + tension.normalized * TENSION_WEIGHT
            + (1.0 - point.y) * AT_RICH_WEIGHT;

        let distance = SAFETY_BOUNDARY - risk_score;

        let level = if distance < -0.5 {
            SafetyLevel::Critical
        } else if distance < 0.0 {
            SafetyLevel::Warning
        } else if distance < 0.5 {
            SafetyLevel::Normal
        } else {
            SafetyLevel::Safe
        };

        Self {
            distance,
            level,
            risk_score,
        }
    }

    /// Whether d(s) is negative (in dangerous territory).
    #[must_use]
    pub fn is_negative(&self) -> bool {
        self.distance < 0.0
    }
}

impl std::fmt::Display for SafetyMargin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "d(s) = {:.4}  Level: {}", self.distance, self.level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_risk_is_safe() {
        let point = Point3D::new(0.0, 1.0, 0.0); // zero entropy, all GC, zero spectral
        let tension = Tension {
            total_delta_h: 0.0,
            total_delta_s: 0.0,
            delta_g_37: 0.0,
            mean_step_energy: 0.0,
            peak_step_energy: 0.0,
            normalized: 0.0,
            step_count: 0,
        };
        let sm = SafetyMargin::compute(&point, &tension);
        assert_eq!(sm.level, SafetyLevel::Safe);
        assert!(sm.distance > 0.0);
    }

    #[test]
    fn high_risk_is_critical_or_warning() {
        let point = Point3D::new(2.0, 0.0, 5.0); // high entropy, no GC, high spectral
        let tension = Tension {
            total_delta_h: -100.0,
            total_delta_s: -300.0,
            delta_g_37: -10.0,
            mean_step_energy: 3.0,
            peak_step_energy: 3.5,
            normalized: 0.9,
            step_count: 50,
        };
        let sm = SafetyMargin::compute(&point, &tension);
        assert!(sm.level.is_actionable());
        assert!(sm.distance < 0.0);
    }

    #[test]
    fn distance_is_boundary_minus_risk() {
        let point = Point3D::new(1.0, 0.5, 2.0);
        let tension = Tension {
            total_delta_h: -50.0,
            total_delta_s: -150.0,
            delta_g_37: -5.0,
            mean_step_energy: 1.5,
            peak_step_energy: 2.0,
            normalized: 0.5,
            step_count: 20,
        };
        let sm = SafetyMargin::compute(&point, &tension);
        assert!((sm.distance - (SAFETY_BOUNDARY - sm.risk_score)).abs() < 1e-10);
    }
}
