//! # Safety Manifold (ToV §5)
//!
//! Formal geometric representation of safe state regions with signed distance fields.
//!
//! ## Theory
//!
//! The safety manifold M defines the boundary between safe and harmful states.
//! We formalize d(s) as a proper signed distance function where:
//! - d(s) > 0: State s is inside M (safe)
//! - d(s) = 0: State s is on the boundary ∂M
//! - d(s) < 0: State s is outside M (harmful)
//!
//! ## Modules
//!
//! - **`axiom4`**: ToV §5 Axiom 4 implementation (generic, constraint-based)
//!
//! ## Components (PV Domain-Specific)
//!
//! - **`HarmBoundary`**: Defines thresholds for each signal dimension
//! - **`SignedDistance`**: Result of distance computation with gradient
//! - **`GeometricSafetyManifold`**: PV-specific manifold with stratified regions
//!
//! ## ToV §5 Axiom 4 (Generic)
//!
//! - **`Axiom4SafetyManifold`**: Generic manifold from constraint sets
//! - **`Axiom4Verification`**: Verifies the three Axiom 4 conditions
//! - **`SafetyMarginResult`**: Constraint-specific margins dᵢ = -gᵢ
//! - **`ManifoldPointType`**: Point classification (interior/boundary/corner/exterior)

pub mod axiom4;

use serde::{Deserialize, Serialize};

// Re-export ToV §5 types
pub use axiom4::{
    // Core Axiom 4 types
    Axiom4SafetyManifold,
    Axiom4Verification,
    // Definition 5.6: Constraint Compatibility
    ConstraintCompatibility,
    FirstPassageTime,
    HarmBoundaryInfo,
    ManifoldPointType,
    // Stratified Structure (Axiom 4.2)
    ManifoldRegularityCase,
    RegularityConditionResult,
    // Proposition 5.2: Safe Configuration Openness
    SafeConfigurationOpenness,
    SafetyMarginResult,
    StratifiedStructure,
    // Definition 5.7: Inherently Unsafe Configuration
    UnsafeConfigurationResult,
};

// ═══════════════════════════════════════════════════════════════════════════
// SIGNAL THRESHOLDS
// ═══════════════════════════════════════════════════════════════════════════

/// Signal thresholds defining the harm boundary.
///
/// Based on standard pharmacovigilance signal detection criteria.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalThresholds {
    /// PRR threshold (typically 2.0)
    pub prr: f64,
    /// ROR threshold (typically 2.0)
    pub ror: f64,
    /// IC threshold (typically 0.0, signals when > 0)
    pub ic: f64,
    /// EBGM threshold (typically 2.0)
    pub ebgm: f64,
}

impl Default for SignalThresholds {
    fn default() -> Self {
        Self {
            prr: 2.0,
            ror: 2.0,
            ic: 0.0,
            ebgm: 2.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SIGNAL POINT
// ═══════════════════════════════════════════════════════════════════════════

/// A point in 4D signal space (PRR, ROR, IC, EBGM).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SignalPoint {
    /// Proportional Reporting Ratio
    pub prr: f64,
    /// Reporting Odds Ratio
    pub ror: f64,
    /// Information Component
    pub ic: f64,
    /// Empirical Bayes Geometric Mean
    pub ebgm: f64,
}

impl SignalPoint {
    /// Create a new signal point.
    #[must_use]
    pub const fn new(prr: f64, ror: f64, ic: f64, ebgm: f64) -> Self {
        Self { prr, ror, ic, ebgm }
    }

    /// Convert to array form for vector operations.
    #[must_use]
    pub const fn to_array(&self) -> [f64; 4] {
        [self.prr, self.ror, self.ic, self.ebgm]
    }

    /// Create from array.
    #[must_use]
    pub const fn from_array(arr: [f64; 4]) -> Self {
        Self {
            prr: arr[0],
            ror: arr[1],
            ic: arr[2],
            ebgm: arr[3],
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HARM BOUNDARY
// ═══════════════════════════════════════════════════════════════════════════

/// Harm boundary as a geometric surface in signal space.
///
/// The boundary separates safe states (below thresholds) from harmful states
/// (above thresholds). Uses a metric tensor for proper distance computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmBoundary {
    /// Signal thresholds defining the boundary
    pub thresholds: SignalThresholds,
    /// Metric tensor diagonal (weights for each dimension)
    pub metric_weights: [f64; 4],
}

impl Default for HarmBoundary {
    fn default() -> Self {
        Self {
            thresholds: SignalThresholds::default(),
            // Equal weighting by default
            metric_weights: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl HarmBoundary {
    /// Create boundary with custom thresholds.
    #[must_use]
    pub fn with_thresholds(thresholds: SignalThresholds) -> Self {
        Self {
            thresholds,
            metric_weights: [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Set metric weights for distance computation.
    #[must_use]
    pub const fn with_weights(mut self, weights: [f64; 4]) -> Self {
        self.metric_weights = weights;
        self
    }

    /// Get threshold array for vector operations.
    #[must_use]
    pub fn threshold_array(&self) -> [f64; 4] {
        [
            self.thresholds.prr,
            self.thresholds.ror,
            self.thresholds.ic,
            self.thresholds.ebgm,
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SIGNED DISTANCE
// ═══════════════════════════════════════════════════════════════════════════

/// Signed distance field result.
///
/// Contains the distance value plus geometric information for interpretation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SignedDistance {
    /// Signed distance: positive = safe, negative = harm
    pub value: f64,
    /// Gradient direction (steepest ascent toward safety)
    pub gradient: [f64; 4],
    /// Surface normal at nearest boundary point
    pub normal: [f64; 4],
    /// The dimension with minimum margin (critical dimension)
    pub critical_dimension: usize,
}

impl SignedDistance {
    /// Check if in safe region (positive distance).
    #[must_use]
    pub const fn is_safe(&self) -> bool {
        self.value > 0.0
    }

    /// Check if on boundary (approximately zero).
    #[must_use]
    pub fn is_on_boundary(&self) -> bool {
        self.value.abs() < 1e-6
    }

    /// Check if in harm region (negative distance).
    #[must_use]
    pub const fn is_harmful(&self) -> bool {
        self.value < 0.0
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SAFETY MANIFOLD
// ═══════════════════════════════════════════════════════════════════════════

/// Safety manifold with stratified regions.
///
/// Defines three regions:
/// - **Interior**: d(s) > buffer_width (robustly safe)
/// - **Buffer**: 0 < d(s) ≤ buffer_width (warning zone)
/// - **Exterior**: d(s) ≤ 0 (harmful)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometricSafetyManifold {
    /// Harm boundary definition
    pub boundary: HarmBoundary,
    /// Width of the buffer/warning zone
    pub buffer_width: f64,
}

impl Default for GeometricSafetyManifold {
    fn default() -> Self {
        Self {
            boundary: HarmBoundary::default(),
            buffer_width: 0.5, // 50% margin before threshold
        }
    }
}

impl GeometricSafetyManifold {
    /// Create manifold with custom boundary.
    #[must_use]
    pub fn with_boundary(boundary: HarmBoundary) -> Self {
        Self {
            boundary,
            buffer_width: 0.5,
        }
    }

    /// Set buffer width.
    #[must_use]
    pub const fn with_buffer(mut self, width: f64) -> Self {
        self.buffer_width = width;
        self
    }

    /// Calculate signed distance from a signal point to the harm boundary.
    ///
    /// Uses normalized distance where each dimension is scaled by its threshold.
    /// The result is the minimum margin across all dimensions.
    #[must_use]
    pub fn signed_distance(&self, signals: &SignalPoint) -> SignedDistance {
        let thresholds = self.boundary.threshold_array();
        let weights = self.boundary.metric_weights;
        let values = signals.to_array();

        // Calculate normalized margins for each dimension
        // margin_i = (threshold_i - value_i) / threshold_i
        let mut margins = [0.0; 4];
        let mut min_margin = f64::MAX;
        let mut min_idx = 0;

        for i in 0..4 {
            if thresholds[i].abs() > 1e-10 {
                margins[i] = (thresholds[i] - values[i]) / thresholds[i];
            } else {
                margins[i] = -values[i]; // For IC where threshold is 0
            }

            let weighted_margin = margins[i] * weights[i];
            if weighted_margin < min_margin {
                min_margin = weighted_margin;
                min_idx = i;
            }
        }

        // Gradient points toward increasing distance (away from boundary)
        let mut gradient = [0.0; 4];
        let mut normal = [0.0; 4];
        let norm_factor = if thresholds[min_idx].abs() > 1e-10 {
            thresholds[min_idx]
        } else {
            1.0
        };

        gradient[min_idx] = -weights[min_idx] / norm_factor;
        normal[min_idx] = -1.0;

        SignedDistance {
            value: min_margin,
            gradient,
            normal,
            critical_dimension: min_idx,
        }
    }

    /// Estimate first-passage time to boundary given current drift.
    ///
    /// If the signal is drifting toward the boundary, estimates how long
    /// until it crosses. Returns infinity if drifting away or stationary.
    #[must_use]
    pub fn first_passage_time(&self, signals: &SignalPoint, drift: &[f64; 4]) -> f64 {
        let dist = self.signed_distance(signals);

        if dist.value <= 0.0 {
            return 0.0; // Already in harm region
        }

        // Compute rate of approach to boundary
        // Rate = gradient · drift (negative means approaching)
        let rate: f64 = dist
            .gradient
            .iter()
            .zip(drift.iter())
            .map(|(g, d)| g * d)
            .sum();

        if rate >= 0.0 {
            f64::INFINITY // Moving away or parallel
        } else {
            dist.value / (-rate) // Time = distance / speed
        }
    }

    /// Project a point onto the boundary surface.
    ///
    /// Returns the nearest point on ∂M to the given signal point.
    #[must_use]
    pub fn project_to_boundary(&self, signals: &SignalPoint) -> SignalPoint {
        let thresholds = self.boundary.threshold_array();
        let mut values = signals.to_array();

        // Find the critical dimension and project to threshold
        let dist = self.signed_distance(signals);
        values[dist.critical_dimension] = thresholds[dist.critical_dimension];

        SignalPoint::from_array(values)
    }

    /// Check if point is in the buffer zone (warning region).
    #[must_use]
    pub fn is_in_buffer(&self, signals: &SignalPoint) -> bool {
        let dist = self.signed_distance(signals);
        dist.value > 0.0 && dist.value <= self.buffer_width
    }

    /// Check if point is robustly safe (inside interior).
    #[must_use]
    pub fn is_robustly_safe(&self, signals: &SignalPoint) -> bool {
        let dist = self.signed_distance(signals);
        dist.value > self.buffer_width
    }

    /// Get the safety status as a human-readable string.
    #[must_use]
    pub fn safety_status(&self, signals: &SignalPoint) -> &'static str {
        let dist = self.signed_distance(signals);
        if dist.value > self.buffer_width {
            "SAFE"
        } else if dist.value > 0.0 {
            "WARNING"
        } else {
            "HARM"
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_point() {
        let manifold = GeometricSafetyManifold::default();
        let safe_point = SignalPoint::new(1.0, 1.0, -0.5, 1.0);

        let dist = manifold.signed_distance(&safe_point);
        assert!(dist.is_safe());
        assert!(dist.value > 0.0);
    }

    #[test]
    fn test_harmful_point() {
        let manifold = GeometricSafetyManifold::default();
        let harm_point = SignalPoint::new(3.0, 3.0, 1.0, 3.0);

        let dist = manifold.signed_distance(&harm_point);
        assert!(dist.is_harmful());
        assert!(dist.value < 0.0);
    }

    #[test]
    fn test_buffer_zone() {
        let manifold = GeometricSafetyManifold::default().with_buffer(0.25);
        // Point at 1.5 when threshold is 2.0 -> margin = 0.25
        let buffer_point = SignalPoint::new(1.5, 1.0, -0.5, 1.0);

        assert!(manifold.is_in_buffer(&buffer_point));
        assert!(!manifold.is_robustly_safe(&buffer_point));
    }

    #[test]
    fn test_first_passage_time() {
        let manifold = GeometricSafetyManifold::default();
        let point = SignalPoint::new(1.0, 1.0, -0.5, 1.0);

        // Drifting toward boundary (PRR increasing)
        let drift = [0.1, 0.0, 0.0, 0.0];
        let time = manifold.first_passage_time(&point, &drift);
        assert!(time.is_finite());
        assert!(time > 0.0);

        // Drifting away from boundary
        let drift_away = [-0.1, 0.0, 0.0, 0.0];
        let time_away = manifold.first_passage_time(&point, &drift_away);
        assert!(time_away.is_infinite());
    }

    #[test]
    fn test_project_to_boundary() {
        let manifold = GeometricSafetyManifold::default();
        let point = SignalPoint::new(3.0, 1.0, -0.5, 1.0); // PRR over threshold

        let projected = manifold.project_to_boundary(&point);
        // Should project PRR to threshold (2.0)
        assert!((projected.prr - 2.0).abs() < 0.01);
    }
}
