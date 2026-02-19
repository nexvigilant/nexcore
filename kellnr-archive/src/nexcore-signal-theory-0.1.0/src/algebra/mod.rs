// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Detection Algebra
//!
//! Compositional operators for combining signal detection methods.
//!
//! ## Composition Operators
//!
//! | Operator | Symbol | Behavior |
//! |----------|--------|----------|
//! | Parallel | D₁ ∥ D₂ | Both run independently, results combined |
//! | Sequential | D₁ ; D₂ | D₂ runs only if D₁ detects |
//! | Cascaded | D₁ → D₂ → D₃ | Progressively stricter thresholds |
//! | Iteration | D* | Repeated application (monitoring) |
//!
//! These compose detection methods into pipelines, mirroring how
//! real pharmacovigilance systems chain screening → confirmation.

use alloc::vec::Vec;
use core::marker::PhantomData;

use crate::detection::DetectionOutcome;
use crate::threshold::DetectionPhase;

// ═══════════════════════════════════════════════════════════
// DETECTOR TRAIT
// ═══════════════════════════════════════════════════════════

/// A signal detector that evaluates a value against some criterion.
///
/// This is the fundamental abstraction: any detection method that
/// takes a numeric input and produces an outcome.
pub trait Detector {
    /// Evaluate the detector on a given value.
    fn detect(&self, value: f64) -> DetectionOutcome;

    /// Human-readable name of this detector.
    fn name(&self) -> &str;
}

// ═══════════════════════════════════════════════════════════
// SIMPLE THRESHOLD DETECTOR
// ═══════════════════════════════════════════════════════════

/// A simple threshold-based detector.
///
/// Detects if value >= threshold.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThresholdDetector {
    /// The threshold.
    pub threshold: f64,
    /// Label.
    pub label: alloc::string::String,
}

impl ThresholdDetector {
    /// Create a new threshold detector.
    #[must_use]
    pub fn new(threshold: f64, label: impl Into<alloc::string::String>) -> Self {
        Self {
            threshold,
            label: label.into(),
        }
    }
}

impl Detector for ThresholdDetector {
    fn detect(&self, value: f64) -> DetectionOutcome {
        if !value.is_finite() {
            DetectionOutcome::Indeterminate
        } else if value >= self.threshold {
            DetectionOutcome::Detected
        } else {
            DetectionOutcome::NotDetected
        }
    }

    fn name(&self) -> &str {
        &self.label
    }
}

// ═══════════════════════════════════════════════════════════
// PARALLEL DETECTION
// ═══════════════════════════════════════════════════════════

/// Parallel composition: D₁ ∥ D₂
///
/// Both detectors run independently. The result is combined:
/// - If both detect → Detected
/// - If either detects (depending on mode) → Detected
/// - Otherwise → NotDetected
///
/// ## Tier: T2-C (∂ + κ + Σ + σ)
#[derive(Debug)]
pub struct ParallelDetection<D1: Detector, D2: Detector> {
    /// First detector.
    pub d1: D1,
    /// Second detector.
    pub d2: D2,
    /// Whether both must agree (AND) or either suffices (OR).
    pub require_both: bool,
}

impl<D1: Detector, D2: Detector> ParallelDetection<D1, D2> {
    /// Create parallel detection requiring both to agree.
    #[must_use]
    pub fn both(d1: D1, d2: D2) -> Self {
        Self {
            d1,
            d2,
            require_both: true,
        }
    }

    /// Create parallel detection where either suffices.
    #[must_use]
    pub fn either(d1: D1, d2: D2) -> Self {
        Self {
            d1,
            d2,
            require_both: false,
        }
    }

    /// Evaluate both detectors in parallel.
    #[must_use]
    pub fn evaluate(&self, value: f64) -> DetectionOutcome {
        let r1 = self.d1.detect(value);
        let r2 = self.d2.detect(value);

        match (r1, r2) {
            (DetectionOutcome::Detected, DetectionOutcome::Detected) => DetectionOutcome::Detected,
            (DetectionOutcome::Detected, _) | (_, DetectionOutcome::Detected) => {
                if self.require_both {
                    DetectionOutcome::NotDetected
                } else {
                    DetectionOutcome::Detected
                }
            }
            (DetectionOutcome::Indeterminate, _) | (_, DetectionOutcome::Indeterminate) => {
                DetectionOutcome::Indeterminate
            }
            _ => DetectionOutcome::NotDetected,
        }
    }
}

// ═══════════════════════════════════════════════════════════
// SEQUENTIAL DETECTION
// ═══════════════════════════════════════════════════════════

/// Sequential composition: D₁ ; D₂
///
/// D₂ runs only if D₁ detects. This models the screening→confirmation pattern.
///
/// ## Tier: T2-C (∂ + σ + κ + ∃)
#[derive(Debug)]
pub struct SequentialDetection<D1: Detector, D2: Detector> {
    /// Screening detector.
    pub screening: D1,
    /// Confirmation detector.
    pub confirmation: D2,
}

impl<D1: Detector, D2: Detector> SequentialDetection<D1, D2> {
    /// Create a sequential screening→confirmation pipeline.
    #[must_use]
    pub fn new(screening: D1, confirmation: D2) -> Self {
        Self {
            screening,
            confirmation,
        }
    }

    /// Evaluate sequentially: screening first, then confirmation.
    #[must_use]
    pub fn evaluate(&self, value: f64) -> DetectionOutcome {
        match self.screening.detect(value) {
            DetectionOutcome::Detected => self.confirmation.detect(value),
            other => other,
        }
    }

    /// Get the phase that would execute for a given value.
    #[must_use]
    pub fn active_phase(&self, value: f64) -> DetectionPhase {
        if self.screening.detect(value).is_detected() {
            DetectionPhase::Confirmation
        } else {
            DetectionPhase::Screening
        }
    }
}

// ═══════════════════════════════════════════════════════════
// CASCADED THRESHOLD
// ═══════════════════════════════════════════════════════════

/// Cascaded thresholds: progressively stricter boundaries.
///
/// Models the typical PV pipeline where initial screening is sensitive,
/// and each subsequent stage is more specific.
///
/// ## Tier: T2-C (∂ + σ + κ + N)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CascadedThreshold {
    /// Thresholds in order (each must be >= the previous).
    pub thresholds: Vec<f64>,
    /// Labels for each stage.
    pub labels: Vec<alloc::string::String>,
}

impl CascadedThreshold {
    /// Create a new cascaded threshold.
    ///
    /// Returns `None` if thresholds aren't monotonically non-decreasing.
    #[must_use]
    pub fn try_new(thresholds: Vec<f64>, labels: Vec<alloc::string::String>) -> Option<Self> {
        if thresholds.len() != labels.len() || thresholds.is_empty() {
            return None;
        }
        // Check monotonicity
        for window in thresholds.windows(2) {
            if window[0] > window[1] {
                return None;
            }
        }
        Some(Self { thresholds, labels })
    }

    /// How many stages a value passes.
    #[must_use]
    pub fn stages_passed(&self, value: f64) -> usize {
        self.thresholds.iter().take_while(|&&t| value >= t).count()
    }

    /// Whether the value passes all stages.
    #[must_use]
    pub fn passes_all(&self, value: f64) -> bool {
        self.stages_passed(value) == self.thresholds.len()
    }

    /// Number of cascade stages.
    #[must_use]
    pub fn stage_count(&self) -> usize {
        self.thresholds.len()
    }

    /// The outcome based on how many stages are passed.
    #[must_use]
    pub fn evaluate(&self, value: f64) -> DetectionOutcome {
        if self.passes_all(value) {
            DetectionOutcome::Detected
        } else if self.stages_passed(value) > 0 {
            DetectionOutcome::Indeterminate
        } else {
            DetectionOutcome::NotDetected
        }
    }
}

// ═══════════════════════════════════════════════════════════
// DETECTION ITERATION (KLEENE STAR)
// ═══════════════════════════════════════════════════════════

/// Iterated detection: D* (monitoring over time).
///
/// Applies a detector repeatedly, tracking detection history.
///
/// ## Tier: T2-C (∂ + σ + ν + π)
#[derive(Debug)]
pub struct DetectionIteration<D: Detector> {
    /// The underlying detector.
    pub detector: D,
    /// History of outcomes.
    pub history: Vec<DetectionOutcome>,
    /// Minimum consecutive detections for confirmation.
    pub min_consecutive: usize,
}

impl<D: Detector> DetectionIteration<D> {
    /// Create a new detection iteration.
    #[must_use]
    pub fn new(detector: D, min_consecutive: usize) -> Self {
        Self {
            detector,
            history: Vec::new(),
            min_consecutive,
        }
    }

    /// Apply the detector to a new observation.
    pub fn observe(&mut self, value: f64) -> DetectionOutcome {
        let outcome = self.detector.detect(value);
        self.history.push(outcome);
        outcome
    }

    /// Whether the minimum consecutive detection threshold is met.
    #[must_use]
    pub fn is_confirmed(&self) -> bool {
        if self.history.len() < self.min_consecutive {
            return false;
        }
        self.history
            .iter()
            .rev()
            .take(self.min_consecutive)
            .all(|o| o.is_detected())
    }

    /// Total number of observations.
    #[must_use]
    pub fn observation_count(&self) -> usize {
        self.history.len()
    }

    /// Total number of detections.
    #[must_use]
    pub fn detection_count(&self) -> usize {
        self.history.iter().filter(|o| o.is_detected()).count()
    }

    /// Detection rate (detections / observations).
    #[must_use]
    pub fn detection_rate(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        self.detection_count() as f64 / self.history.len() as f64
    }

    /// Reset the history.
    pub fn reset(&mut self) {
        self.history.clear();
    }
}

// ═══════════════════════════════════════════════════════════
// DETECTION PIPELINE
// ═══════════════════════════════════════════════════════════

/// A named detection pipeline stage for building dynamic pipelines.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineStage {
    /// Stage name.
    pub name: alloc::string::String,
    /// Threshold for this stage.
    pub threshold: f64,
    /// Phase classification.
    pub phase: DetectionPhase,
}

/// A dynamic detection pipeline composed of named stages.
///
/// Unlike the generic composition types, this is fully serializable.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DetectionPipeline {
    /// Pipeline stages in execution order.
    pub stages: Vec<PipelineStage>,
    /// Pipeline label.
    pub label: alloc::string::String,
}

impl DetectionPipeline {
    /// Create a new pipeline.
    #[must_use]
    pub fn new(label: impl Into<alloc::string::String>) -> Self {
        Self {
            stages: Vec::new(),
            label: label.into(),
        }
    }

    /// Add a stage.
    pub fn add_stage(
        &mut self,
        name: impl Into<alloc::string::String>,
        threshold: f64,
        phase: DetectionPhase,
    ) {
        self.stages.push(PipelineStage {
            name: name.into(),
            threshold,
            phase,
        });
    }

    /// Evaluate the pipeline: returns the first failing stage index, or None if all pass.
    #[must_use]
    pub fn evaluate(&self, value: f64) -> (DetectionOutcome, usize) {
        for (i, stage) in self.stages.iter().enumerate() {
            if value < stage.threshold {
                return (DetectionOutcome::NotDetected, i);
            }
        }
        (DetectionOutcome::Detected, self.stages.len())
    }

    /// Number of stages.
    #[must_use]
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_detector() {
        let d = ThresholdDetector::new(2.0, "PRR");
        assert_eq!(d.detect(3.0), DetectionOutcome::Detected);
        assert_eq!(d.detect(1.5), DetectionOutcome::NotDetected);
        assert_eq!(d.detect(f64::NAN), DetectionOutcome::Indeterminate);
        assert_eq!(d.name(), "PRR");
    }

    #[test]
    fn test_parallel_both() {
        let d1 = ThresholdDetector::new(2.0, "PRR");
        let d2 = ThresholdDetector::new(3.84, "Chi²");
        let parallel = ParallelDetection::both(d1, d2);

        // 5.0 passes both
        assert_eq!(parallel.evaluate(5.0), DetectionOutcome::Detected);
        // 3.0 passes PRR but not Chi²
        assert_eq!(parallel.evaluate(3.0), DetectionOutcome::NotDetected);
    }

    #[test]
    fn test_parallel_either() {
        let d1 = ThresholdDetector::new(2.0, "PRR");
        let d2 = ThresholdDetector::new(3.84, "Chi²");
        let parallel = ParallelDetection::either(d1, d2);

        // 3.0 passes PRR only
        assert_eq!(parallel.evaluate(3.0), DetectionOutcome::Detected);
        // 1.0 passes neither
        assert_eq!(parallel.evaluate(1.0), DetectionOutcome::NotDetected);
    }

    #[test]
    fn test_sequential_detection() {
        let screening = ThresholdDetector::new(1.5, "Screening");
        let confirmation = ThresholdDetector::new(3.0, "Confirmation");
        let seq = SequentialDetection::new(screening, confirmation);

        // 4.0 passes both → Detected
        assert_eq!(seq.evaluate(4.0), DetectionOutcome::Detected);
        // 2.0 passes screening but not confirmation → NotDetected
        assert_eq!(seq.evaluate(2.0), DetectionOutcome::NotDetected);
        // 1.0 doesn't pass screening → NotDetected
        assert_eq!(seq.evaluate(1.0), DetectionOutcome::NotDetected);
    }

    #[test]
    fn test_sequential_phase() {
        let screening = ThresholdDetector::new(1.5, "S");
        let confirmation = ThresholdDetector::new(3.0, "C");
        let seq = SequentialDetection::new(screening, confirmation);

        assert_eq!(seq.active_phase(2.0), DetectionPhase::Confirmation);
        assert_eq!(seq.active_phase(1.0), DetectionPhase::Screening);
    }

    #[test]
    fn test_cascaded_threshold() {
        let ct = CascadedThreshold::try_new(
            alloc::vec![1.5, 2.0, 3.0],
            alloc::vec!["Sensitive".into(), "Default".into(), "Strict".into()],
        );
        assert!(ct.is_some());
        let ct = ct.unwrap_or_else(|| CascadedThreshold {
            thresholds: alloc::vec![1.0],
            labels: alloc::vec!["fallback".into()],
        });

        assert_eq!(ct.stages_passed(2.5), 2);
        assert!(!ct.passes_all(2.5));
        assert!(ct.passes_all(3.0));
        assert_eq!(ct.evaluate(2.5), DetectionOutcome::Indeterminate);
        assert_eq!(ct.evaluate(3.0), DetectionOutcome::Detected);
        assert_eq!(ct.evaluate(1.0), DetectionOutcome::NotDetected);
    }

    #[test]
    fn test_cascaded_not_monotonic() {
        let ct =
            CascadedThreshold::try_new(alloc::vec![3.0, 2.0], alloc::vec!["A".into(), "B".into()]);
        assert!(ct.is_none());
    }

    #[test]
    fn test_detection_iteration() {
        let detector = ThresholdDetector::new(2.0, "PRR");
        let mut iter = DetectionIteration::new(detector, 3);

        iter.observe(3.0); // detected
        iter.observe(2.5); // detected
        assert!(!iter.is_confirmed()); // only 2 consecutive

        iter.observe(3.0); // detected
        assert!(iter.is_confirmed()); // 3 consecutive

        iter.observe(1.0); // not detected
        assert!(!iter.is_confirmed()); // broken streak

        assert_eq!(iter.observation_count(), 4);
        assert_eq!(iter.detection_count(), 3);
        assert!((iter.detection_rate() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_detection_pipeline() {
        let mut pipeline = DetectionPipeline::new("PV standard");
        pipeline.add_stage("Screening", 1.5, DetectionPhase::Screening);
        pipeline.add_stage("Confirmation", 3.0, DetectionPhase::Confirmation);

        let (outcome, stages) = pipeline.evaluate(4.0);
        assert_eq!(outcome, DetectionOutcome::Detected);
        assert_eq!(stages, 2);

        let (outcome, stages) = pipeline.evaluate(2.0);
        assert_eq!(outcome, DetectionOutcome::NotDetected);
        assert_eq!(stages, 1); // failed at confirmation

        let (outcome, stages) = pipeline.evaluate(1.0);
        assert_eq!(outcome, DetectionOutcome::NotDetected);
        assert_eq!(stages, 0); // failed at screening
    }
}
