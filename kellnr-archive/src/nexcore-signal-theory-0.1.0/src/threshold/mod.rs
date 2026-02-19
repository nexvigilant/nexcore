// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Boundary Theory
//!
//! Types and strategies for drawing detection boundaries (thresholds).
//!
//! "All detection is boundary drawing." — This module formalizes the
//! different ways boundaries can be defined and composed.
//!
//! ## Boundary Kinds
//!
//! | Kind | Behavior | Example |
//! |------|----------|---------|
//! | Fixed | Static threshold | PRR ≥ 2.0 |
//! | Adaptive | Adjusts based on data | Bayesian shrinkage |
//! | Composite | Multiple thresholds combined | PRR ≥ 2.0 AND χ² ≥ 3.84 |

use alloc::string::String;
use alloc::vec::Vec;

use crate::detection::DetectionOutcome;

// ═══════════════════════════════════════════════════════════
// BOUNDARY KIND
// ═══════════════════════════════════════════════════════════

/// Classification of boundary types.
///
/// ## Tier: T1 (κ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BoundaryKind {
    /// Static, predetermined threshold.
    Fixed,
    /// Threshold that adjusts based on accumulating data.
    Adaptive,
    /// Bayesian shrinkage boundary (e.g., EBGM).
    Bayesian,
    /// Multiple boundaries combined via conjunction/disjunction.
    Composite,
}

impl BoundaryKind {
    /// All boundary kinds.
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [Self::Fixed, Self::Adaptive, Self::Bayesian, Self::Composite]
    }
}

// ═══════════════════════════════════════════════════════════
// CONJUNCTION MODE
// ═══════════════════════════════════════════════════════════

/// How multiple boundaries are combined in composite detection.
///
/// ## Tier: T1 (κ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ConjunctionMode {
    /// All boundaries must be exceeded (AND).
    All,
    /// At least one boundary must be exceeded (OR).
    Any,
    /// At least K out of N boundaries must be exceeded.
    AtLeast(u32),
}

impl ConjunctionMode {
    /// Evaluate the conjunction over a set of boolean results.
    #[must_use]
    pub fn evaluate(&self, results: &[bool]) -> bool {
        match self {
            Self::All => results.iter().all(|&r| r),
            Self::Any => results.iter().any(|&r| r),
            Self::AtLeast(k) => {
                let count = results.iter().filter(|&&r| r).count();
                count >= (*k as usize)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════
// THRESHOLD PRESET
// ═══════════════════════════════════════════════════════════

/// Standard threshold presets for signal detection.
///
/// ## Tier: T1 (κ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ThresholdPreset {
    /// Default thresholds (balanced sensitivity/specificity).
    Default,
    /// Strict thresholds (high specificity, fewer false positives).
    Strict,
    /// Sensitive thresholds (high sensitivity, fewer misses).
    Sensitive,
}

impl ThresholdPreset {
    /// PRR threshold for this preset.
    #[must_use]
    pub const fn prr_threshold(&self) -> u32 {
        // Stored as fixed-point * 10
        match self {
            Self::Default => 20,   // 2.0
            Self::Strict => 30,    // 3.0
            Self::Sensitive => 15, // 1.5
        }
    }

    /// Chi-squared threshold for this preset (fixed-point * 1000).
    #[must_use]
    pub const fn chi_sq_threshold(&self) -> u32 {
        match self {
            Self::Default => 3841,   // 3.841
            Self::Strict => 6635,    // 6.635
            Self::Sensitive => 2706, // 2.706
        }
    }

    /// Minimum case count.
    #[must_use]
    pub const fn min_cases(&self) -> u32 {
        match self {
            Self::Default => 3,
            Self::Strict => 5,
            Self::Sensitive => 2,
        }
    }

    /// All presets.
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [Self::Default, Self::Strict, Self::Sensitive]
    }
}

// ═══════════════════════════════════════════════════════════
// FIXED BOUNDARY
// ═══════════════════════════════════════════════════════════

/// A fixed (static) boundary for signal detection.
///
/// ## Tier: T2-P (∂ + N)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FixedBoundary {
    /// The threshold value.
    pub threshold: f64,
    /// Label describing what is being thresholded.
    pub label: String,
    /// Direction: true = value must exceed threshold, false = must be below.
    pub upper_bound: bool,
}

impl FixedBoundary {
    /// Create a new fixed boundary (value must exceed threshold).
    #[must_use]
    pub fn above(threshold: f64, label: impl Into<String>) -> Self {
        Self {
            threshold,
            label: label.into(),
            upper_bound: true,
        }
    }

    /// Create a new fixed boundary (value must be below threshold).
    #[must_use]
    pub fn below(threshold: f64, label: impl Into<String>) -> Self {
        Self {
            threshold,
            label: label.into(),
            upper_bound: false,
        }
    }

    /// Evaluate whether a value satisfies this boundary.
    #[must_use]
    pub fn evaluate(&self, value: f64) -> bool {
        if self.upper_bound {
            value >= self.threshold
        } else {
            value <= self.threshold
        }
    }

    /// Distance from the boundary (positive = satisfies, negative = violates).
    #[must_use]
    pub fn distance(&self, value: f64) -> f64 {
        if self.upper_bound {
            value - self.threshold
        } else {
            self.threshold - value
        }
    }
}

// ═══════════════════════════════════════════════════════════
// ADAPTIVE BOUNDARY
// ═══════════════════════════════════════════════════════════

/// An adaptive boundary that adjusts based on accumulated data.
///
/// ## Tier: T2-C (∂ + N + ν + ς)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AdaptiveBoundary {
    /// Current threshold value.
    pub current_threshold: f64,
    /// Initial threshold value.
    pub initial_threshold: f64,
    /// Number of updates applied.
    pub update_count: u64,
    /// Learning rate for adaptation.
    pub learning_rate: f64,
    /// Label.
    pub label: String,
}

impl AdaptiveBoundary {
    /// Create a new adaptive boundary.
    #[must_use]
    pub fn new(initial_threshold: f64, learning_rate: f64, label: impl Into<String>) -> Self {
        Self {
            current_threshold: initial_threshold,
            initial_threshold,
            update_count: 0,
            learning_rate,
            label: label.into(),
        }
    }

    /// Update the threshold based on new evidence.
    ///
    /// The threshold moves toward the new evidence value at the learning rate.
    pub fn update(&mut self, new_evidence: f64) {
        self.current_threshold += self.learning_rate * (new_evidence - self.current_threshold);
        self.update_count += 1;
    }

    /// Evaluate whether a value exceeds the current threshold.
    #[must_use]
    pub fn evaluate(&self, value: f64) -> bool {
        value >= self.current_threshold
    }

    /// How far the threshold has drifted from its initial value.
    #[must_use]
    pub fn drift(&self) -> f64 {
        (self.current_threshold - self.initial_threshold).abs()
    }

    /// Reset to initial threshold.
    pub fn reset(&mut self) {
        self.current_threshold = self.initial_threshold;
        self.update_count = 0;
    }
}

// ═══════════════════════════════════════════════════════════
// COMPOSITE BOUNDARY
// ═══════════════════════════════════════════════════════════

/// A composite boundary combining multiple fixed boundaries.
///
/// ## Tier: T2-C (∂ + κ + N + Σ)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CompositeBoundary {
    /// The individual boundaries.
    pub boundaries: Vec<FixedBoundary>,
    /// How boundaries are combined.
    pub mode: ConjunctionMode,
    /// Label for the composite.
    pub label: String,
}

impl CompositeBoundary {
    /// Create a new composite boundary.
    #[must_use]
    pub fn new(
        boundaries: Vec<FixedBoundary>,
        mode: ConjunctionMode,
        label: impl Into<String>,
    ) -> Self {
        Self {
            boundaries,
            mode,
            label: label.into(),
        }
    }

    /// Evaluate the composite boundary against a set of values.
    ///
    /// Values must be in the same order as boundaries.
    #[must_use]
    pub fn evaluate(&self, values: &[f64]) -> DetectionOutcome {
        if values.len() != self.boundaries.len() {
            return DetectionOutcome::Indeterminate;
        }

        let results: Vec<bool> = self
            .boundaries
            .iter()
            .zip(values.iter())
            .map(|(b, &v)| b.evaluate(v))
            .collect();

        if self.mode.evaluate(&results) {
            DetectionOutcome::Detected
        } else {
            DetectionOutcome::NotDetected
        }
    }

    /// Number of individual boundaries.
    #[must_use]
    pub fn boundary_count(&self) -> usize {
        self.boundaries.len()
    }

    /// Create a standard PV composite (PRR + Chi-squared + min cases).
    #[must_use]
    pub fn standard_pv(preset: ThresholdPreset) -> Self {
        let prr_thresh = preset.prr_threshold() as f64 / 10.0;
        let chi_thresh = preset.chi_sq_threshold() as f64 / 1000.0;
        let min_n = preset.min_cases() as f64;

        Self::new(
            alloc::vec![
                FixedBoundary::above(prr_thresh, "PRR"),
                FixedBoundary::above(chi_thresh, "Chi-squared"),
                FixedBoundary::above(min_n, "Min cases"),
            ],
            ConjunctionMode::All,
            "Standard PV composite",
        )
    }
}

// ═══════════════════════════════════════════════════════════
// DETECTION PHASE
// ═══════════════════════════════════════════════════════════

/// Phase of detection in a multi-stage pipeline.
///
/// ## Tier: T1 (κ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DetectionPhase {
    /// Initial screening — sensitive, many false positives.
    Screening,
    /// Confirmation — specific, fewer false positives.
    Confirmation,
}

impl DetectionPhase {
    /// Whether this is the initial screening phase.
    #[must_use]
    pub const fn is_screening(&self) -> bool {
        matches!(self, Self::Screening)
    }

    /// Recommended preset for this phase.
    #[must_use]
    pub const fn recommended_preset(&self) -> ThresholdPreset {
        match self {
            Self::Screening => ThresholdPreset::Sensitive,
            Self::Confirmation => ThresholdPreset::Strict,
        }
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boundary_kind_all() {
        assert_eq!(BoundaryKind::all().len(), 4);
    }

    #[test]
    fn test_conjunction_mode_all() {
        assert!(ConjunctionMode::All.evaluate(&[true, true, true]));
        assert!(!ConjunctionMode::All.evaluate(&[true, false, true]));
    }

    #[test]
    fn test_conjunction_mode_any() {
        assert!(ConjunctionMode::Any.evaluate(&[false, true, false]));
        assert!(!ConjunctionMode::Any.evaluate(&[false, false, false]));
    }

    #[test]
    fn test_conjunction_mode_at_least() {
        assert!(ConjunctionMode::AtLeast(2).evaluate(&[true, true, false]));
        assert!(!ConjunctionMode::AtLeast(2).evaluate(&[true, false, false]));
    }

    #[test]
    fn test_threshold_presets() {
        assert_eq!(ThresholdPreset::Default.prr_threshold(), 20);
        assert_eq!(ThresholdPreset::Strict.min_cases(), 5);
        assert_eq!(ThresholdPreset::Sensitive.chi_sq_threshold(), 2706);
        assert_eq!(ThresholdPreset::all().len(), 3);
    }

    #[test]
    fn test_fixed_boundary_above() {
        let b = FixedBoundary::above(2.0, "PRR");
        assert!(b.evaluate(3.0));
        assert!(b.evaluate(2.0));
        assert!(!b.evaluate(1.5));
        assert!((b.distance(3.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fixed_boundary_below() {
        let b = FixedBoundary::below(0.05, "p-value");
        assert!(b.evaluate(0.01));
        assert!(!b.evaluate(0.10));
    }

    #[test]
    fn test_adaptive_boundary() {
        let mut ab = AdaptiveBoundary::new(2.0, 0.1, "adaptive PRR");
        assert!(ab.evaluate(3.0));
        assert!(!ab.evaluate(1.0));

        ab.update(4.0);
        assert_eq!(ab.update_count, 1);
        // threshold moved: 2.0 + 0.1 * (4.0 - 2.0) = 2.2
        assert!((ab.current_threshold - 2.2).abs() < f64::EPSILON);

        ab.reset();
        assert!((ab.current_threshold - 2.0).abs() < f64::EPSILON);
        assert_eq!(ab.update_count, 0);
    }

    #[test]
    fn test_composite_boundary() {
        let composite = CompositeBoundary::standard_pv(ThresholdPreset::Default);
        assert_eq!(composite.boundary_count(), 3);

        // PRR=3.0, Chi²=5.0, n=5 → all pass
        assert_eq!(
            composite.evaluate(&[3.0, 5.0, 5.0]),
            DetectionOutcome::Detected
        );

        // PRR=1.5 → fails PRR threshold (2.0)
        assert_eq!(
            composite.evaluate(&[1.5, 5.0, 5.0]),
            DetectionOutcome::NotDetected
        );
    }

    #[test]
    fn test_composite_boundary_wrong_length() {
        let composite = CompositeBoundary::standard_pv(ThresholdPreset::Default);
        assert_eq!(
            composite.evaluate(&[3.0, 5.0]),
            DetectionOutcome::Indeterminate
        );
    }

    #[test]
    fn test_detection_phase() {
        assert!(DetectionPhase::Screening.is_screening());
        assert!(!DetectionPhase::Confirmation.is_screening());
        assert_eq!(
            DetectionPhase::Screening.recommended_preset(),
            ThresholdPreset::Sensitive
        );
        assert_eq!(
            DetectionPhase::Confirmation.recommended_preset(),
            ThresholdPreset::Strict
        );
    }
}
