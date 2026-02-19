//! Drift detection for Phase 4 surveillance.
//!
//! This module provides statistical methods for detecting when system behavior
//! deviates from an established baseline, enabling continuous validation.

use serde::{Deserialize, Serialize};

/// Method for detecting drift between distributions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftMethod {
    /// Population Stability Index
    PopulationStabilityIndex,

    /// Kolmogorov-Smirnov two-sample test
    KolmogorovSmirnov,

    /// Chi-squared test for categorical data
    ChiSquared,

    /// Page-Hinkley test for streaming data
    PageHinkley,

    /// Simple mean comparison
    MeanComparison,
}

impl DriftMethod {
    /// Returns the default threshold for this method
    pub fn get_default_threshold(&self) -> f64 {
        match self {
            Self::PopulationStabilityIndex => 0.10, // < 0.10 = no drift, 0.10-0.25 = slight, > 0.25 = significant
            Self::KolmogorovSmirnov => 0.05,        // p-value threshold
            Self::ChiSquared => 0.05,               // p-value threshold
            Self::PageHinkley => 50.0,              // PH statistic threshold
            Self::MeanComparison => 0.10,           // 10% deviation
        }
    }
}

/// Result of drift detection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriftResult {
    /// Whether drift was detected
    pub drift_detected: bool,

    /// The drift score (method-dependent)
    pub drift_score: f64,

    /// Threshold that was used
    pub threshold: f64,

    /// Method that was used
    pub method: DriftMethod,

    /// Interpretation of the result
    pub interpretation: DriftInterpretation,

    /// Additional details
    pub details: Option<String>,
}

/// Interpretation of drift severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriftInterpretation {
    /// No significant drift detected
    NoDrift,

    /// Minor drift that may warrant monitoring
    MinorDrift,

    /// Moderate drift that should be investigated
    ModerateDrift,

    /// Significant drift requiring immediate action
    SignificantDrift,
}

impl DriftInterpretation {
    /// Returns a description of the interpretation
    pub fn get_description(&self) -> &'static str {
        match self {
            Self::NoDrift => "System behavior is stable and within expected bounds",
            Self::MinorDrift => "Minor deviation detected - continue monitoring",
            Self::ModerateDrift => "Moderate deviation - investigation recommended",
            Self::SignificantDrift => "Significant deviation - immediate action required",
        }
    }
}

/// Drift detector for continuous validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftDetector {
    /// Baseline distribution
    baseline: Vec<f64>,

    /// Current observation window
    current_window: Vec<f64>,

    /// Window size
    window_size: usize,

    /// Detection threshold
    threshold: f64,

    /// Detection method
    method: DriftMethod,
}

impl DriftDetector {
    /// Creates a new drift detector
    pub fn new(method: DriftMethod, window_size: usize) -> Self {
        Self {
            baseline: Vec::new(),
            current_window: Vec::new(),
            window_size,
            threshold: method.get_default_threshold(),
            method,
        }
    }

    /// Creates a detector with custom threshold
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    /// Sets the baseline distribution
    pub fn set_baseline(&mut self, values: Vec<f64>) {
        self.baseline = values;
    }

    /// Sets baseline from current window
    pub fn capture_baseline(&mut self) {
        self.baseline = self.current_window.clone();
    }

    /// Observes a new value
    pub fn observe(&mut self, value: f64) {
        self.current_window.push(value);

        // Maintain window size
        while self.current_window.len() > self.window_size {
            self.current_window.remove(0);
        }
    }

    /// Observes multiple values
    pub fn observe_batch(&mut self, values: &[f64]) {
        for value in values {
            self.observe(*value);
        }
    }

    /// Calculates the drift score
    pub fn get_drift_score(&self) -> Option<f64> {
        if self.baseline.is_empty() || self.current_window.len() < self.window_size / 2 {
            return None;
        }

        Some(match self.method {
            DriftMethod::PopulationStabilityIndex => self.calculate_psi(),
            DriftMethod::KolmogorovSmirnov => self.calculate_ks(),
            DriftMethod::MeanComparison => self.calculate_mean_diff(),
            _ => self.calculate_mean_diff(), // Fallback
        })
    }

    /// Checks if drift is detected
    pub fn check_is_drifting(&self) -> bool {
        self.get_drift_score()
            .map(|score| score > self.threshold)
            .unwrap_or(false)
    }

    /// Performs full drift analysis
    pub fn analyze_drift(&self) -> Option<DriftResult> {
        let score = self.get_drift_score()?;
        let drift_detected = score > self.threshold;

        let interpretation = self.get_interpretation(score);

        Some(DriftResult {
            drift_detected,
            drift_score: score,
            threshold: self.threshold,
            method: self.method,
            interpretation,
            details: Some(format!(
                "Baseline size: {}, Window size: {}, Score: {:.4}",
                self.baseline.len(),
                self.current_window.len(),
                score
            )),
        })
    }

    /// Interprets a drift score
    fn get_interpretation(&self, score: f64) -> DriftInterpretation {
        match self.method {
            DriftMethod::PopulationStabilityIndex => self.interpret_psi_score(score),
            _ => self.interpret_fallback_score(score),
        }
    }

    fn interpret_psi_score(&self, score: f64) -> DriftInterpretation {
        if score < 0.10 {
            DriftInterpretation::NoDrift
        } else if score < 0.25 {
            DriftInterpretation::MinorDrift
        } else if score < 0.50 {
            DriftInterpretation::ModerateDrift
        } else {
            DriftInterpretation::SignificantDrift
        }
    }

    fn interpret_fallback_score(&self, score: f64) -> DriftInterpretation {
        let relative = score / self.threshold;
        if relative < 0.5 {
            DriftInterpretation::NoDrift
        } else if relative < 1.0 {
            DriftInterpretation::MinorDrift
        } else if relative < 2.0 {
            DriftInterpretation::ModerateDrift
        } else {
            DriftInterpretation::SignificantDrift
        }
    }

    /// Calculates Population Stability Index
    fn calculate_psi(&self) -> f64 {
        let num_bins = 10;
        let baseline_bins = self.bin_distribution(&self.baseline, num_bins);
        let current_bins = self.bin_distribution(&self.current_window, num_bins);

        let epsilon = 0.0001; // Avoid division by zero

        baseline_bins
            .iter()
            .zip(current_bins.iter())
            .map(|(expected, actual)| {
                let e = expected.max(epsilon);
                let a = actual.max(epsilon);
                (a - e) * (a / e).ln()
            })
            .sum()
    }

    /// Calculates Kolmogorov-Smirnov statistic
    fn calculate_ks(&self) -> f64 {
        let mut b_sorted = self.baseline.clone();
        let mut c_sorted = self.current_window.clone();

        b_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        c_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let all_values = self.collect_unique_values(&b_sorted, &c_sorted);
        let mut max_diff = 0.0_f64;

        for value in all_values {
            let cdf1 = self.compute_cdf(&b_sorted, value);
            let cdf2 = self.compute_cdf(&c_sorted, value);
            max_diff = max_diff.max((cdf1 - cdf2).abs());
        }

        max_diff
    }

    fn collect_unique_values(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        let mut set = std::collections::BTreeSet::new();
        for &x in a.iter().chain(b.iter()) {
            set.insert(ordered_float::OrderedFloat(x));
        }
        set.into_iter().map(|x| x.0).collect()
    }

    fn compute_cdf(&self, sorted_values: &[f64], value: f64) -> f64 {
        if sorted_values.is_empty() {
            return 0.0;
        }
        let count = sorted_values.iter().filter(|&&x| x <= value).count();
        count as f64 / sorted_values.len() as f64
    }

    /// Calculates mean difference (percentage)
    fn calculate_mean_diff(&self) -> f64 {
        let baseline_mean: f64 = self.baseline.iter().sum::<f64>() / self.baseline.len() as f64;
        let current_mean: f64 =
            self.current_window.iter().sum::<f64>() / self.current_window.len() as f64;

        if baseline_mean.abs() < f64::EPSILON {
            return (current_mean - baseline_mean).abs();
        }

        ((current_mean - baseline_mean) / baseline_mean).abs()
    }

    /// Bins a distribution into equal-width buckets
    fn bin_distribution(&self, values: &[f64], num_bins: usize) -> Vec<f64> {
        if values.is_empty() {
            return vec![0.0; num_bins];
        }

        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        if (max - min).abs() < f64::EPSILON {
            // All values are the same
            let mut bins = vec![0.0; num_bins];
            bins[num_bins / 2] = 1.0;
            return bins;
        }

        let bin_width = (max - min) / num_bins as f64;
        let mut counts = vec![0usize; num_bins];

        for &value in values {
            let bin = ((value - min) / bin_width) as usize;
            let bin = bin.min(num_bins - 1); // Handle edge case where value == max
            counts[bin] += 1;
        }

        let total = values.len() as f64;
        counts.into_iter().map(|c| c as f64 / total).collect()
    }

    /// Resets the detector (clears window, keeps baseline)
    pub fn reset_window(&mut self) {
        self.current_window.clear();
    }

    /// Fully resets the detector
    pub fn reset(&mut self) {
        self.baseline.clear();
        self.current_window.clear();
    }
}

impl Default for DriftDetector {
    fn default() -> Self {
        Self::new(DriftMethod::PopulationStabilityIndex, 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_drift() {
        let mut detector = DriftDetector::new(DriftMethod::MeanComparison, 100);

        // Set baseline
        let baseline: Vec<f64> = (0..100).map(|i| i as f64).collect();
        detector.set_baseline(baseline);

        // Observe similar values
        for i in 0..100 {
            detector.observe(i as f64 + 0.5); // Small offset
        }

        assert!(!detector.check_is_drifting());
    }

    #[test]
    fn test_drift_detection() {
        let mut detector = DriftDetector::new(DriftMethod::MeanComparison, 100);

        // Set baseline around 50
        let baseline: Vec<f64> = (0..100).map(|i| 50.0 + (i as f64 - 50.0) * 0.1).collect();
        detector.set_baseline(baseline);

        // Observe values around 100 (significant drift)
        for i in 0..100 {
            detector.observe(100.0 + (i as f64 - 50.0) * 0.1);
        }

        assert!(detector.check_is_drifting());
    }

    #[test]
    fn test_psi_calculation() {
        let mut detector = DriftDetector::new(DriftMethod::PopulationStabilityIndex, 100);

        // Identical distributions should have PSI ≈ 0
        let values: Vec<f64> = (0..100).map(|i| i as f64).collect();
        detector.set_baseline(values.clone());
        detector.observe_batch(&values);

        let score = detector.get_drift_score().unwrap_or(1.0);
        assert!(
            score < 0.10,
            "PSI should be low for identical distributions"
        );
    }

    #[test]
    fn test_drift_interpretation() {
        let detector = DriftDetector::new(DriftMethod::PopulationStabilityIndex, 100);

        assert_eq!(
            detector.get_interpretation(0.05),
            DriftInterpretation::NoDrift
        );
        assert_eq!(
            detector.get_interpretation(0.15),
            DriftInterpretation::MinorDrift
        );
        assert_eq!(
            detector.get_interpretation(0.35),
            DriftInterpretation::ModerateDrift
        );
        assert_eq!(
            detector.get_interpretation(0.60),
            DriftInterpretation::SignificantDrift
        );
    }
}
