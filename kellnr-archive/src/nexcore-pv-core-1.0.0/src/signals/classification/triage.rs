//! Multi-Method Signal Triage Engine
//!
//! Combines results from multiple signal detection methods (PRR, ROR, IC, EBGM,
//! Chi², Fisher) into a unified priority score for signal management workflow.
//!
//! # Regulatory Basis
//!
//! GVP Module IX (Signal Management) requires systematic prioritization of
//! detected signals. This engine implements weighted multi-method consensus
//! scoring with configurable threshold profiles.
//!
//! # Primitive Grounding
//!
//! | Type | Primitives | Tier |
//! |------|-----------|------|
//! | `TriageScore` | N | T1 |
//! | `TriageCategory` | ∂, κ | T2-P |
//! | `MethodWeight` | μ, N | T2-P |
//! | `NormalizedSignal` | →, N | T2-P |
//! | `TriageEngine` | σ, μ, →, Σ, ∂ | T2-C |
//!
//! # Example
//!
//! ```rust
//! use nexcore_vigilance::pv::signals::classification::triage::{
//!     TriageEngine, SignalInput, TriageCategory,
//! };
//!
//! let engine = TriageEngine::default();
//! let input = SignalInput::builder()
//!     .prr(3.5)
//!     .ror(4.2)
//!     .ic025(1.8)
//!     .eb05(2.5)
//!     .chi_square(12.3)
//!     .case_count(15)
//!     .build();
//!
//! let result = engine.triage(&input);
//! assert!(matches!(result.category, TriageCategory::High | TriageCategory::Critical));
//! assert!(result.composite_score > 0.7);
//! ```

use serde::{Deserialize, Serialize};

// =============================================================================
// T1: TriageScore — Primitive N (Quantity)
// =============================================================================

/// Composite triage score normalized to [0.0, 1.0].
///
/// Tier: T1 (Quantity)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TriageScore(f64);

impl TriageScore {
    /// Create a triage score, clamped to [0.0, 1.0].
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Get the raw score value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl Default for TriageScore {
    fn default() -> Self {
        Self(0.0)
    }
}

// =============================================================================
// T2-P: TriageCategory — Primitives ∂ (Boundary) + κ (Comparison)
// =============================================================================

/// Signal triage priority category.
///
/// Tier: T2-P (Boundary × Comparison)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TriageCategory {
    /// No signal detected by any method
    NoSignal,
    /// Low priority — weak or single-method signal
    Low,
    /// Medium priority — moderate multi-method agreement
    Medium,
    /// High priority — strong multi-method consensus
    High,
    /// Critical — immediate evaluation required
    Critical,
}

impl TriageCategory {
    /// Whether this category requires expedited review.
    #[must_use]
    pub fn requires_expedited_review(self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }

    /// Suggested evaluation timeline in days.
    #[must_use]
    pub fn evaluation_timeline_days(self) -> u32 {
        match self {
            Self::Critical => 1,
            Self::High => 7,
            Self::Medium => 30,
            Self::Low => 90,
            Self::NoSignal => 0,
        }
    }
}

impl std::fmt::Display for TriageCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSignal => write!(f, "No Signal"),
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

// =============================================================================
// T2-P: MethodWeight — Primitives μ (Mapping) + N (Quantity)
// =============================================================================

/// Weight assigned to a signal detection method in composite scoring.
///
/// Tier: T2-P (Mapping × Quantity)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MethodWeight {
    /// Method identifier
    pub method: DetectionMethod,
    /// Weight in [0.0, 1.0]
    pub weight: f64,
}

/// Signal detection method identifier.
///
/// Tier: T1 (maps to σ Sequence as an enum over methods)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DetectionMethod {
    /// Proportional Reporting Ratio
    Prr,
    /// Reporting Odds Ratio
    Ror,
    /// Information Component (lower 95% CI)
    Ic025,
    /// Empirical Bayes Geometric Mean (lower 5th percentile)
    Eb05,
    /// Chi-squared test
    ChiSquare,
    /// Fisher exact test
    Fisher,
}

impl std::fmt::Display for DetectionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Prr => write!(f, "PRR"),
            Self::Ror => write!(f, "ROR"),
            Self::Ic025 => write!(f, "IC025"),
            Self::Eb05 => write!(f, "EB05"),
            Self::ChiSquare => write!(f, "χ²"),
            Self::Fisher => write!(f, "Fisher"),
        }
    }
}

// =============================================================================
// T2-P: NormalizedSignal — Primitives → (Causality) + N (Quantity)
// =============================================================================

/// A signal detection result normalized to [0.0, 1.0] for cross-method comparison.
///
/// Tier: T2-P (Causality × Quantity)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NormalizedSignal {
    /// Which method produced this
    pub method: DetectionMethod,
    /// Raw value from the detection method
    pub raw_value: f64,
    /// Normalized score in [0.0, 1.0]
    pub normalized: f64,
    /// Whether this method independently flags a signal
    pub is_signal: bool,
}

// =============================================================================
// Signal Input Builder
// =============================================================================

/// Raw signal detection results to be triaged.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignalInput {
    /// PRR value (signal if ≥ 2.0)
    pub prr: Option<f64>,
    /// ROR value (signal if lower CI > 1.0)
    pub ror: Option<f64>,
    /// ROR lower 95% confidence interval
    pub ror_lower_ci: Option<f64>,
    /// IC025 value (signal if > 0)
    pub ic025: Option<f64>,
    /// EB05 value (signal if ≥ 2.0)
    pub eb05: Option<f64>,
    /// Chi-square value (signal if ≥ 3.841)
    pub chi_square: Option<f64>,
    /// Fisher exact p-value (signal if < 0.05)
    pub fisher_p: Option<f64>,
    /// Number of observed cases (a cell in 2×2 table)
    pub case_count: Option<u32>,
    /// Drug name (for reporting)
    pub drug_name: Option<String>,
    /// Event name (for reporting)
    pub event_name: Option<String>,
}

impl SignalInput {
    /// Create a builder for signal input.
    #[must_use]
    pub fn builder() -> SignalInputBuilder {
        SignalInputBuilder::default()
    }
}

/// Builder for `SignalInput`.
#[derive(Debug, Clone, Default)]
pub struct SignalInputBuilder {
    input: SignalInput,
}

impl SignalInputBuilder {
    #[must_use]
    pub fn prr(mut self, value: f64) -> Self {
        self.input.prr = Some(value);
        self
    }

    #[must_use]
    pub fn ror(mut self, value: f64) -> Self {
        self.input.ror = Some(value);
        self
    }

    #[must_use]
    pub fn ror_lower_ci(mut self, value: f64) -> Self {
        self.input.ror_lower_ci = Some(value);
        self
    }

    #[must_use]
    pub fn ic025(mut self, value: f64) -> Self {
        self.input.ic025 = Some(value);
        self
    }

    #[must_use]
    pub fn eb05(mut self, value: f64) -> Self {
        self.input.eb05 = Some(value);
        self
    }

    #[must_use]
    pub fn chi_square(mut self, value: f64) -> Self {
        self.input.chi_square = Some(value);
        self
    }

    #[must_use]
    pub fn fisher_p(mut self, value: f64) -> Self {
        self.input.fisher_p = Some(value);
        self
    }

    #[must_use]
    pub fn case_count(mut self, value: u32) -> Self {
        self.input.case_count = Some(value);
        self
    }

    #[must_use]
    pub fn drug_name(mut self, name: impl Into<String>) -> Self {
        self.input.drug_name = Some(name.into());
        self
    }

    #[must_use]
    pub fn event_name(mut self, name: impl Into<String>) -> Self {
        self.input.event_name = Some(name.into());
        self
    }

    /// Build the signal input.
    #[must_use]
    pub fn build(self) -> SignalInput {
        self.input
    }
}

// =============================================================================
// T2-P: ThresholdProfile — Primitives ∂ (Boundary) + μ (Mapping)
// =============================================================================

/// Threshold profile for signal detection. Controls sensitivity.
///
/// Tier: T2-P (Boundary × Mapping)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdProfile {
    /// Name of this profile
    pub name: String,
    /// PRR threshold (default: 2.0)
    pub prr_threshold: f64,
    /// Chi² threshold (default: 3.841 for p<0.05)
    pub chi_square_threshold: f64,
    /// Minimum case count (default: 3)
    pub min_case_count: u32,
    /// ROR lower CI threshold (default: 1.0)
    pub ror_lower_ci_threshold: f64,
    /// IC025 threshold (default: 0.0)
    pub ic025_threshold: f64,
    /// EB05 threshold (default: 2.0)
    pub eb05_threshold: f64,
    /// Fisher p-value threshold (default: 0.05)
    pub fisher_p_threshold: f64,
}

impl ThresholdProfile {
    /// Default (standard) thresholds per regulatory guidance.
    #[must_use]
    pub fn standard() -> Self {
        Self {
            name: "Standard".into(),
            prr_threshold: 2.0,
            chi_square_threshold: 3.841,
            min_case_count: 3,
            ror_lower_ci_threshold: 1.0,
            ic025_threshold: 0.0,
            eb05_threshold: 2.0,
            fisher_p_threshold: 0.05,
        }
    }

    /// Strict thresholds for higher specificity.
    #[must_use]
    pub fn strict() -> Self {
        Self {
            name: "Strict".into(),
            prr_threshold: 3.0,
            chi_square_threshold: 6.635,
            min_case_count: 5,
            ror_lower_ci_threshold: 2.0,
            ic025_threshold: 1.0,
            eb05_threshold: 3.0,
            fisher_p_threshold: 0.01,
        }
    }

    /// Sensitive thresholds for higher recall.
    #[must_use]
    pub fn sensitive() -> Self {
        Self {
            name: "Sensitive".into(),
            prr_threshold: 1.5,
            chi_square_threshold: 2.706,
            min_case_count: 2,
            ror_lower_ci_threshold: 1.0,
            ic025_threshold: -0.5,
            eb05_threshold: 1.5,
            fisher_p_threshold: 0.10,
        }
    }
}

impl Default for ThresholdProfile {
    fn default() -> Self {
        Self::standard()
    }
}

// =============================================================================
// T2-C: TriageEngine — Primitives σ, μ, →, Σ, ∂
// =============================================================================

/// Multi-method signal triage engine.
///
/// Normalizes results from multiple detection methods, applies method weights,
/// computes a composite score, and classifies into triage categories.
///
/// Tier: T2-C (Sequence × Mapping × Causality × Sum × Boundary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageEngine {
    /// Method weights for composite scoring
    pub weights: Vec<MethodWeight>,
    /// Threshold profile for signal detection
    pub thresholds: ThresholdProfile,
    /// Category boundary thresholds [low, medium, high, critical]
    pub category_boundaries: [f64; 4],
}

impl Default for TriageEngine {
    fn default() -> Self {
        Self {
            weights: vec![
                MethodWeight {
                    method: DetectionMethod::Prr,
                    weight: 0.15,
                },
                MethodWeight {
                    method: DetectionMethod::Ror,
                    weight: 0.15,
                },
                MethodWeight {
                    method: DetectionMethod::Ic025,
                    weight: 0.25,
                },
                MethodWeight {
                    method: DetectionMethod::Eb05,
                    weight: 0.25,
                },
                MethodWeight {
                    method: DetectionMethod::ChiSquare,
                    weight: 0.10,
                },
                MethodWeight {
                    method: DetectionMethod::Fisher,
                    weight: 0.10,
                },
            ],
            thresholds: ThresholdProfile::standard(),
            category_boundaries: [0.2, 0.4, 0.65, 0.85],
        }
    }
}

/// Full triage result with breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageResult {
    /// Composite triage score
    pub composite_score: f64,
    /// Triage category
    pub category: TriageCategory,
    /// Number of methods that independently flag a signal
    pub methods_flagging: u32,
    /// Total methods evaluated
    pub methods_evaluated: u32,
    /// Method agreement ratio (0.0 - 1.0)
    pub agreement_ratio: f64,
    /// Per-method normalized results
    pub method_results: Vec<NormalizedSignal>,
    /// Whether minimum case count threshold is met
    pub case_count_met: bool,
    /// Human-readable summary
    pub summary: String,
}

impl TriageEngine {
    /// Create engine with custom weights and thresholds.
    #[must_use]
    pub fn new(weights: Vec<MethodWeight>, thresholds: ThresholdProfile) -> Self {
        Self {
            weights,
            thresholds,
            category_boundaries: [0.2, 0.4, 0.65, 0.85],
        }
    }

    /// Set category boundary thresholds.
    #[must_use]
    pub fn with_boundaries(mut self, boundaries: [f64; 4]) -> Self {
        self.category_boundaries = boundaries;
        self
    }

    /// Perform triage on signal detection results.
    #[must_use]
    pub fn triage(&self, input: &SignalInput) -> TriageResult {
        // Check minimum case count
        let case_count_met = input
            .case_count
            .map_or(true, |n| n >= self.thresholds.min_case_count);

        // Normalize each available method result
        let mut method_results = Vec::new();

        if let Some(prr) = input.prr {
            method_results.push(self.normalize_prr(prr));
        }
        if let Some(ror) = input.ror {
            let ror_ci = input.ror_lower_ci.unwrap_or(ror * 0.5);
            method_results.push(self.normalize_ror(ror, ror_ci));
        }
        if let Some(ic025) = input.ic025 {
            method_results.push(self.normalize_ic025(ic025));
        }
        if let Some(eb05) = input.eb05 {
            method_results.push(self.normalize_eb05(eb05));
        }
        if let Some(chi_sq) = input.chi_square {
            method_results.push(self.normalize_chi_square(chi_sq));
        }
        if let Some(fisher_p) = input.fisher_p {
            method_results.push(self.normalize_fisher(fisher_p));
        }

        let methods_evaluated = method_results.len() as u32;
        let methods_flagging = method_results.iter().filter(|r| r.is_signal).count() as u32;

        // Compute weighted composite score
        let composite_score = if method_results.is_empty() {
            0.0
        } else {
            self.compute_composite(&method_results)
        };

        // Apply case count gate: if not met, cap score
        let gated_score = if case_count_met {
            composite_score
        } else {
            composite_score * 0.5 // Halve score when insufficient cases
        };

        // Agreement boost: if many methods agree, boost confidence
        let agreement_ratio = if methods_evaluated > 0 {
            methods_flagging as f64 / methods_evaluated as f64
        } else {
            0.0
        };

        // Final score with agreement boost (up to 10% boost for full agreement)
        let final_score = (gated_score + agreement_ratio * 0.1).clamp(0.0, 1.0);

        // Classify
        let category = self.classify(final_score, methods_flagging);

        // Generate summary
        let summary = self.generate_summary(
            &input.drug_name,
            &input.event_name,
            &category,
            final_score,
            methods_flagging,
            methods_evaluated,
        );

        TriageResult {
            composite_score: final_score,
            category,
            methods_flagging,
            methods_evaluated,
            agreement_ratio,
            method_results,
            case_count_met,
            summary,
        }
    }

    // ── Normalization functions ─────────────────────────────────────────

    fn normalize_prr(&self, prr: f64) -> NormalizedSignal {
        let threshold = self.thresholds.prr_threshold;
        let is_signal = prr >= threshold;
        // Normalize: sigmoid-like transform centered at threshold
        let normalized = sigmoid_normalize(prr, threshold, 2.0);
        NormalizedSignal {
            method: DetectionMethod::Prr,
            raw_value: prr,
            normalized,
            is_signal,
        }
    }

    fn normalize_ror(&self, ror: f64, lower_ci: f64) -> NormalizedSignal {
        let threshold = self.thresholds.ror_lower_ci_threshold;
        let is_signal = lower_ci > threshold;
        let normalized = sigmoid_normalize(lower_ci, threshold, 1.5);
        NormalizedSignal {
            method: DetectionMethod::Ror,
            raw_value: ror,
            normalized,
            is_signal,
        }
    }

    fn normalize_ic025(&self, ic025: f64) -> NormalizedSignal {
        let threshold = self.thresholds.ic025_threshold;
        let is_signal = ic025 > threshold;
        // IC025 can be negative; shift normalization
        let normalized = sigmoid_normalize(ic025, threshold, 1.0);
        NormalizedSignal {
            method: DetectionMethod::Ic025,
            raw_value: ic025,
            normalized,
            is_signal,
        }
    }

    fn normalize_eb05(&self, eb05: f64) -> NormalizedSignal {
        let threshold = self.thresholds.eb05_threshold;
        let is_signal = eb05 >= threshold;
        let normalized = sigmoid_normalize(eb05, threshold, 2.0);
        NormalizedSignal {
            method: DetectionMethod::Eb05,
            raw_value: eb05,
            normalized,
            is_signal,
        }
    }

    fn normalize_chi_square(&self, chi_sq: f64) -> NormalizedSignal {
        let threshold = self.thresholds.chi_square_threshold;
        let is_signal = chi_sq >= threshold;
        let normalized = sigmoid_normalize(chi_sq, threshold, 4.0);
        NormalizedSignal {
            method: DetectionMethod::ChiSquare,
            raw_value: chi_sq,
            normalized,
            is_signal,
        }
    }

    fn normalize_fisher(&self, p_value: f64) -> NormalizedSignal {
        let threshold = self.thresholds.fisher_p_threshold;
        let is_signal = p_value < threshold;
        // Invert: lower p → higher signal score
        let normalized = if p_value <= 0.0 {
            1.0
        } else {
            (1.0 - (p_value / threshold).min(2.0) * 0.5).clamp(0.0, 1.0)
        };
        NormalizedSignal {
            method: DetectionMethod::Fisher,
            raw_value: p_value,
            normalized,
            is_signal,
        }
    }

    // ── Composite scoring ──────────────────────────────────────────────

    fn compute_composite(&self, results: &[NormalizedSignal]) -> f64 {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for result in results {
            let weight = self
                .weights
                .iter()
                .find(|w| w.method == result.method)
                .map_or(0.1, |w| w.weight);

            weighted_sum += result.normalized * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }

    // ── Classification ─────────────────────────────────────────────────

    fn classify(&self, score: f64, methods_flagging: u32) -> TriageCategory {
        let [low, med, high, crit] = self.category_boundaries;

        // Critical requires both high score AND multi-method agreement
        if score >= crit && methods_flagging >= 3 {
            TriageCategory::Critical
        } else if score >= high && methods_flagging >= 2 {
            TriageCategory::High
        } else if score >= med {
            TriageCategory::Medium
        } else if score >= low {
            TriageCategory::Low
        } else {
            TriageCategory::NoSignal
        }
    }

    // ── Summary generation ─────────────────────────────────────────────

    fn generate_summary(
        &self,
        drug: &Option<String>,
        event: &Option<String>,
        category: &TriageCategory,
        score: f64,
        flagging: u32,
        evaluated: u32,
    ) -> String {
        let pair = match (drug, event) {
            (Some(d), Some(e)) => format!("{d}-{e}"),
            (Some(d), None) => d.clone(),
            (None, Some(e)) => e.clone(),
            (None, None) => "Unknown pair".into(),
        };

        format!(
            "Signal Triage [{pair}]: {category} (score: {score:.3}, \
             {flagging}/{evaluated} methods flagging, \
             review within {} days)",
            category.evaluation_timeline_days()
        )
    }
}

// =============================================================================
// Normalization helper
// =============================================================================

/// Sigmoid normalization: maps a raw value to [0, 1] centered at threshold.
///
/// Values below threshold → 0..0.5, at threshold → 0.5, above → 0.5..1.0.
/// `steepness` controls the curve sharpness.
fn sigmoid_normalize(value: f64, threshold: f64, steepness: f64) -> f64 {
    if steepness <= 0.0 {
        return if value >= threshold { 1.0 } else { 0.0 };
    }
    let x = (value - threshold) / steepness;
    1.0 / (1.0 + (-x).exp())
}

// =============================================================================
// Batch triage
// =============================================================================

/// Triage multiple drug-event pairs and sort by priority.
#[must_use]
pub fn batch_triage(engine: &TriageEngine, inputs: &[SignalInput]) -> Vec<TriageResult> {
    let mut results: Vec<TriageResult> = inputs.iter().map(|i| engine.triage(i)).collect();
    // Sort descending by composite score (highest priority first)
    results.sort_by(|a, b| {
        b.composite_score
            .partial_cmp(&a.composite_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triage_critical_signal() {
        let engine = TriageEngine::default();
        let input = SignalInput::builder()
            .prr(5.0)
            .ror(6.0)
            .ror_lower_ci(3.5)
            .ic025(2.5)
            .eb05(4.0)
            .chi_square(25.0)
            .case_count(20)
            .drug_name("TestDrug")
            .event_name("TestEvent")
            .build();

        let result = engine.triage(&input);
        assert!(result.composite_score > 0.7);
        assert!(result.methods_flagging >= 3);
        assert!(matches!(
            result.category,
            TriageCategory::High | TriageCategory::Critical
        ));
        assert!(result.case_count_met);
        assert!(result.summary.contains("TestDrug"));
    }

    #[test]
    fn test_triage_no_signal() {
        let engine = TriageEngine::default();
        let input = SignalInput::builder()
            .prr(0.5)
            .ror(0.8)
            .ror_lower_ci(0.3)
            .ic025(-1.5)
            .eb05(0.5)
            .chi_square(0.5)
            .case_count(1)
            .build();

        let result = engine.triage(&input);
        assert!(result.composite_score < 0.4);
        assert_eq!(result.methods_flagging, 0);
        assert!(matches!(
            result.category,
            TriageCategory::NoSignal | TriageCategory::Low
        ));
    }

    #[test]
    fn test_triage_borderline() {
        let engine = TriageEngine::default();
        let input = SignalInput::builder()
            .prr(2.0)
            .ic025(0.1)
            .case_count(3)
            .build();

        let result = engine.triage(&input);
        assert!(result.methods_evaluated == 2);
        // PRR at threshold, IC025 barely positive — should be Low/Medium
        assert!(result.composite_score > 0.0);
    }

    #[test]
    fn test_triage_insufficient_cases() {
        let engine = TriageEngine::default();
        let input = SignalInput::builder()
            .prr(5.0)
            .ror(6.0)
            .ror_lower_ci(3.0)
            .ic025(2.0)
            .eb05(3.5)
            .case_count(1) // Below minimum of 3
            .build();

        let result = engine.triage(&input);
        // Score should be halved due to insufficient cases
        assert!(!result.case_count_met);
    }

    #[test]
    fn test_threshold_profiles() {
        let standard = ThresholdProfile::standard();
        let strict = ThresholdProfile::strict();
        let sensitive = ThresholdProfile::sensitive();

        assert!(strict.prr_threshold > standard.prr_threshold);
        assert!(sensitive.prr_threshold < standard.prr_threshold);
        assert!(strict.min_case_count > standard.min_case_count);
    }

    #[test]
    fn test_batch_triage_sorted() {
        let engine = TriageEngine::default();
        let inputs = vec![
            SignalInput::builder().prr(1.0).ic025(-0.5).build(),
            SignalInput::builder().prr(5.0).ic025(3.0).eb05(4.0).build(),
            SignalInput::builder().prr(2.5).ic025(0.5).build(),
        ];

        let results = batch_triage(&engine, &inputs);
        assert_eq!(results.len(), 3);
        // Verify sorted descending by score
        assert!(results[0].composite_score >= results[1].composite_score);
        assert!(results[1].composite_score >= results[2].composite_score);
    }

    #[test]
    fn test_sigmoid_normalize() {
        // At threshold → ~0.5
        let at_threshold = sigmoid_normalize(2.0, 2.0, 2.0);
        assert!((at_threshold - 0.5).abs() < 0.001);

        // Well above threshold → close to 1.0
        let high = sigmoid_normalize(10.0, 2.0, 2.0);
        assert!(high > 0.95);

        // Well below threshold → close to 0.0
        let low = sigmoid_normalize(-5.0, 2.0, 2.0);
        assert!(low < 0.05);
    }

    #[test]
    fn test_category_properties() {
        assert!(TriageCategory::Critical.requires_expedited_review());
        assert!(TriageCategory::High.requires_expedited_review());
        assert!(!TriageCategory::Medium.requires_expedited_review());
        assert!(!TriageCategory::Low.requires_expedited_review());

        assert_eq!(TriageCategory::Critical.evaluation_timeline_days(), 1);
        assert_eq!(TriageCategory::High.evaluation_timeline_days(), 7);
        assert_eq!(TriageCategory::Medium.evaluation_timeline_days(), 30);
    }

    #[test]
    fn test_empty_input() {
        let engine = TriageEngine::default();
        let input = SignalInput::default();
        let result = engine.triage(&input);
        assert_eq!(result.methods_evaluated, 0);
        assert_eq!(result.composite_score, 0.0);
        assert_eq!(result.category, TriageCategory::NoSignal);
    }

    #[test]
    fn test_single_method_high_value() {
        let engine = TriageEngine::default();
        let input = SignalInput::builder().prr(10.0).case_count(50).build();

        let result = engine.triage(&input);
        // Single method can't reach Critical (needs 3+ flagging)
        assert!(result.methods_flagging == 1);
        assert!(!matches!(result.category, TriageCategory::Critical));
    }

    #[test]
    fn test_strict_profile_reduces_signals() {
        let standard_engine = TriageEngine::default();
        let strict_engine =
            TriageEngine::new(standard_engine.weights.clone(), ThresholdProfile::strict());

        let input = SignalInput::builder()
            .prr(2.5) // Above standard (2.0) but below strict (3.0)
            .chi_square(4.0) // Above standard (3.841) but below strict (6.635)
            .case_count(4) // Above standard (3) but below strict (5)
            .build();

        let standard_result = standard_engine.triage(&input);
        let strict_result = strict_engine.triage(&input);

        // Standard should flag more methods than strict
        assert!(standard_result.methods_flagging >= strict_result.methods_flagging);
    }

    #[test]
    fn test_fisher_inversion() {
        let engine = TriageEngine::default();

        // Very significant p-value
        let input_sig = SignalInput::builder().fisher_p(0.001).build();
        let result_sig = engine.triage(&input_sig);

        // Non-significant p-value
        let input_ns = SignalInput::builder().fisher_p(0.5).build();
        let result_ns = engine.triage(&input_ns);

        assert!(result_sig.composite_score > result_ns.composite_score);
    }
}
