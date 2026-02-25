//! # Signal Core
//!
//! Foundation types and traits for the signal detection pipeline.

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
//! Every crate in the `signal-*` family depends on `signal-core`.
//!
//! ## Architecture (T1 Primitive Grounding)
//!
//! | T1 Primitive | Manifestation |
//! |---|---|
//! | **Sequence (σ)** | Pipeline stage order: Ingest → Normalize → Detect → Alert |
//! | **Mapping (μ)** | Transformation between raw and standardized event models |
//! | **State (ς)** | Encapsulated context in `AlertState` and `ContingencyTable` |
//! | **Recursion (ρ)** | Problem reduction via hierarchical signal aggregation |
//!
//! ## Quick Start
//!
//! ```rust
//! use crate::core::{DrugEventPair, ContingencyTable, SignalStrength};
//!
//! let pair = DrugEventPair::new("aspirin", "bleeding");
//! let table = ContingencyTable { a: 15, b: 100, c: 20, d: 10_000 };
//! let strength = table.prr().map(SignalStrength::from_prr);
//! ```

use nexcore_chrono::DateTime;
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};

// ─── Error Types ────────────────────────────────────────────

/// Unified error type for the signal detection pipeline.
#[non_exhaustive]
#[derive(Debug, nexcore_error::Error)]
pub enum SignalError {
    /// Data ingestion stage failure.
    #[error("ingestion failed: {0}")]
    Ingestion(String),
    /// Normalization stage failure.
    #[error("normalization failed: {0}")]
    Normalization(String),
    /// Signal detection stage failure.
    #[error("detection failed: {0}")]
    Detection(String),
    /// Threshold evaluation failure.
    #[error("threshold error: {0}")]
    Threshold(String),
    /// Persistence layer failure.
    #[error("storage error: {0}")]
    Storage(String),
    /// Validation stage failure.
    #[error("validation error: {0}")]
    Validation(String),
    /// Division by zero in contingency table computation.
    #[error("division by zero in contingency table")]
    DivisionByZero,
    /// Insufficient case count for reliable detection.
    #[error("insufficient data: need at least {needed} cases, got {got}")]
    InsufficientData {
        /// Minimum required cases.
        needed: u64,
        /// Actual cases provided.
        got: u64,
    },
}

/// Convenience alias for `Result<T, SignalError>`.
pub type Result<T> = std::result::Result<T, SignalError>;

// ─── T2-P: Newtypes (Cross-Domain Primitives) ──────────────

/// Proportional Reporting Ratio — no naked f64.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Prr(pub f64);

/// Reporting Odds Ratio.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Ror(pub f64);

/// Information Component (Bayesian).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Ic(pub f64);

/// Empirical Bayesian Geometric Mean.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Ebgm(pub f64);

/// Chi-square statistic.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ChiSquare(pub f64);

/// Confidence interval bounds.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    /// Lower bound of the interval.
    pub lower: f64,
    /// Upper bound of the interval.
    pub upper: f64,
    /// Confidence level (e.g., 0.95 for 95%).
    pub level: f64,
}

/// Signal strength classification.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SignalStrength {
    /// No signal detected.
    None,
    /// Weak signal — warrants monitoring.
    Weak,
    /// Moderate signal — warrants investigation.
    Moderate,
    /// Strong signal — warrants action.
    Strong,
    /// Critical signal — immediate action required.
    Critical,
}

impl SignalStrength {
    /// Classify from PRR value using Evans thresholds.
    pub fn from_prr(prr: f64) -> Self {
        match prr {
            x if x < 1.5 => Self::None,
            x if x < 2.0 => Self::Weak,
            x if x < 3.0 => Self::Moderate,
            x if x < 5.0 => Self::Strong,
            _ => Self::Critical,
        }
    }
}

// ─── T2-C: Composites ──────────────────────────────────────

/// 2×2 contingency table — the atom of disproportionality analysis.
///
/// ```text
///              Event+    Event-
/// Drug+     |   a    |    b    |
/// Drug-     |   c    |    d    |
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContingencyTable {
    /// Drug+ Event+ (cases of interest).
    pub a: u64,
    /// Drug+ Event- (drug exposure without event).
    pub b: u64,
    /// Drug- Event+ (event without drug).
    pub c: u64,
    /// Drug- Event- (background).
    pub d: u64,
}

impl ContingencyTable {
    /// Create a new 2x2 contingency table.
    #[must_use]
    pub fn new(a: u64, b: u64, c: u64, d: u64) -> Self {
        Self { a, b, c, d }
    }

    /// Total reports in the table.
    pub fn total(&self) -> u64 {
        self.a
            .saturating_add(self.b)
            .saturating_add(self.c)
            .saturating_add(self.d)
    }

    /// Proportional Reporting Ratio.
    /// PRR = (a / (a+b)) / (c / (c+d))
    #[allow(
        clippy::as_conversions,
        clippy::cast_precision_loss,
        reason = "u64->f64 cast is intentional for statistical computation; values bounded by dataset size"
    )]
    pub fn prr(&self) -> Option<f64> {
        let drug_total = self.a.saturating_add(self.b) as f64;
        let ref_total = self.c.saturating_add(self.d) as f64;
        if drug_total == 0.0 || ref_total == 0.0 {
            return None;
        }
        let num = self.a as f64 / drug_total;
        let denom = self.c as f64 / ref_total;
        if denom == 0.0 {
            None
        } else {
            Some(num / denom)
        }
    }

    /// Reporting Odds Ratio.
    /// ROR = (a*d) / (b*c)
    #[allow(
        clippy::as_conversions,
        clippy::cast_precision_loss,
        reason = "u64->f64 cast is intentional for statistical computation; saturating mul prevents overflow"
    )]
    pub fn ror(&self) -> Option<f64> {
        let denom = self.b.saturating_mul(self.c) as f64;
        if denom == 0.0 {
            None
        } else {
            Some(self.a.saturating_mul(self.d) as f64 / denom)
        }
    }

    /// Chi-square statistic (Yates correction).
    #[allow(
        clippy::as_conversions,
        clippy::cast_precision_loss,
        reason = "u64->f64 cast is intentional for statistical computation; saturating ops prevent overflow"
    )]
    pub fn chi_square(&self) -> f64 {
        let n = self.total() as f64;
        let ad = self.a as f64 * self.d as f64;
        let bc = self.b as f64 * self.c as f64;
        let num = n * (ad - bc).abs().powi(2);
        let denom = self.a.saturating_add(self.b) as f64
            * self.c.saturating_add(self.d) as f64
            * self.a.saturating_add(self.c) as f64
            * self.b.saturating_add(self.d) as f64;
        if denom == 0.0 { 0.0 } else { num / denom }
    }
}

/// A drug-event pair identifier.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DrugEventPair {
    /// Drug name or substance identifier.
    pub drug: String,
    /// Adverse event term (preferably `MedDRA` PT).
    pub event: String,
}

impl DrugEventPair {
    /// Create a new drug-event pair from any string-like types.
    pub fn new(drug: impl Into<String>, event: impl Into<String>) -> Self {
        Self {
            drug: drug.into(),
            event: event.into(),
        }
    }
}

// ─── T3: Domain-Specific Types ──────────────────────────────

/// A raw adverse event report (pre-normalization).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawReport {
    /// Unique report identifier.
    pub id: String,
    /// Drug names as reported (pre-normalization).
    pub drug_names: Vec<String>,
    /// Event terms as reported (pre-normalization).
    pub event_terms: Vec<String>,
    /// Date the report was submitted.
    pub report_date: Option<DateTime>,
    /// Originating data source.
    pub source: ReportSource,
    /// Arbitrary metadata from the source system.
    pub metadata: serde_json::Value,
}

/// Source of a report.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportSource {
    /// FDA Adverse Event Reporting System.
    Faers,
    /// EU pharmacovigilance database.
    Eudravigilance,
    /// WHO global ICSR database.
    Vigibase,
    /// Spontaneous (voluntary) report.
    Spontaneous,
    /// Report from a clinical trial.
    ClinicalTrial,
    /// Published literature case report.
    Literature,
    /// Unrecognized source (preserves original label).
    Unknown(String),
}

/// A normalized event after standardization.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedEvent {
    /// Unique identifier for this normalized event.
    pub id: NexId,
    /// Standardized drug name.
    pub drug: String,
    /// Standardized event term.
    pub event: String,
    /// `MedDRA` Preferred Term (if mapped).
    pub meddra_pt: Option<String>,
    /// `MedDRA` System Organ Class (if mapped).
    pub meddra_soc: Option<String>,
    /// Date of the original report.
    pub report_date: DateTime,
    /// Originating data source.
    pub source: ReportSource,
}

/// Full detection result for a drug-event pair.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    /// The drug-event pair analyzed.
    pub pair: DrugEventPair,
    /// Contingency table used for computation.
    pub table: ContingencyTable,
    /// Proportional Reporting Ratio (if computable).
    pub prr: Option<Prr>,
    /// Reporting Odds Ratio (if computable).
    pub ror: Option<Ror>,
    /// Information Component (Bayesian, if computable).
    pub ic: Option<Ic>,
    /// Empirical Bayesian Geometric Mean (if computable).
    pub ebgm: Option<Ebgm>,
    /// Chi-square statistic.
    pub chi_square: ChiSquare,
    /// Overall signal strength classification.
    pub strength: SignalStrength,
    /// Timestamp of detection.
    pub detected_at: DateTime,
}

impl DetectionResult {
    /// Create a new detection result.
    ///
    /// Provides a stable constructor for use outside the defining crate, where
    /// `#[non_exhaustive]` prevents direct struct literal construction.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pair: DrugEventPair,
        table: ContingencyTable,
        prr: Option<Prr>,
        ror: Option<Ror>,
        ic: Option<Ic>,
        ebgm: Option<Ebgm>,
        chi_square: ChiSquare,
        strength: SignalStrength,
        detected_at: nexcore_chrono::DateTime,
    ) -> Self {
        Self {
            pair,
            table,
            prr,
            ror,
            ic,
            ebgm,
            chi_square,
            strength,
            detected_at,
        }
    }
}

/// Alert lifecycle states.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertState {
    /// Newly created alert, not yet reviewed.
    New,
    /// Alert is under active review.
    UnderReview,
    /// Signal confirmed as genuine.
    Confirmed,
    /// Escalated to senior reviewer or regulatory body.
    Escalated,
    /// Alert closed (no further action).
    Closed,
    /// Determined to be a false positive.
    FalsePositive,
}

/// A signal alert.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique alert identifier.
    pub id: NexId,
    /// Detection result that triggered this alert.
    pub detection: DetectionResult,
    /// Current lifecycle state.
    pub state: AlertState,
    /// When the alert was created.
    pub created_at: DateTime,
    /// When the alert was last modified.
    pub updated_at: DateTime,
    /// Reviewer notes and comments.
    pub notes: Vec<String>,
}

// ─── Pipeline Traits ────────────────────────────────────────

/// Ingests raw data into `RawReport` streams.
pub trait Ingest {
    /// Ingest from a data source, yielding raw reports.
    fn ingest(&self) -> Result<Vec<RawReport>>;
}

/// Normalizes raw reports to standardized events.
pub trait Normalize {
    /// Normalize a raw report into zero or more standardized events.
    fn normalize(&self, report: &RawReport) -> Result<Vec<NormalizedEvent>>;
}

/// Detects signals from a set of normalized events.
pub trait Detect {
    /// Run signal detection across normalized events.
    fn detect(&self, events: &[NormalizedEvent]) -> Result<Vec<DetectionResult>>;
}

/// Applies thresholds to detection results.
pub trait Threshold {
    /// Returns `true` if the detection result exceeds the threshold.
    fn apply(&self, result: &DetectionResult) -> bool;
}

/// Generates alerts from detection results that pass thresholds.
pub trait Alertable {
    /// Create an alert from a detection result.
    fn alert(&self, result: &DetectionResult) -> Result<Alert>;
}

/// Stores and retrieves detection results and alerts.
pub trait Store {
    /// Persist a detection result.
    fn save_result(&mut self, result: &DetectionResult) -> Result<()>;
    /// Persist an alert.
    fn save_alert(&mut self, alert: &Alert) -> Result<()>;
    /// Retrieve alerts, optionally filtered by state.
    fn get_alerts(&self, state: Option<AlertState>) -> Result<Vec<Alert>>;
}

/// Validates detection results for quality and consistency.
pub trait Validate {
    /// Validate a detection result, returning a report of checks.
    fn validate(&self, result: &DetectionResult) -> Result<ValidationReport>;
}

/// Generates reports from detection results.
pub trait Report {
    /// Generate a human-readable report from detection results.
    fn report(&self, results: &[DetectionResult]) -> Result<String>;
}

/// Validation report for a detection result.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// The drug-event pair that was validated.
    pub pair: DrugEventPair,
    /// Whether all checks passed.
    pub passed: bool,
    /// Individual validation checks performed.
    pub checks: Vec<ValidationCheck>,
}

/// Individual validation check.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    /// Check name (e.g., "`minimum_case_count`").
    pub name: String,
    /// Whether this check passed.
    pub passed: bool,
    /// Human-readable result message.
    pub message: String,
}

// ─── Threshold Configuration ────────────────────────────────

/// Evans criteria thresholds for signal detection.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    /// Minimum PRR value.
    pub prr_min: f64,
    /// Minimum chi-square statistic.
    pub chi_square_min: f64,
    /// Minimum case count (cell `a`).
    pub case_count_min: u64,
    /// Minimum ROR lower confidence interval bound.
    pub ror_lower_ci_min: f64,
    /// Minimum IC 2.5th percentile (IC025).
    pub ic025_min: f64,
    /// Minimum EB05 (EBGM 5th percentile).
    pub eb05_min: f64,
}

impl Default for ThresholdConfig {
    /// Evans default thresholds.
    fn default() -> Self {
        Self {
            prr_min: 2.0,
            chi_square_min: 3.841,
            case_count_min: 3,
            ror_lower_ci_min: 1.0,
            ic025_min: 0.0,
            eb05_min: 2.0,
        }
    }
}

impl ThresholdConfig {
    /// Create a config overriding only the three most commonly tuned parameters;
    /// all other fields keep their Evans defaults.
    ///
    /// Useful from outside the crate where struct update syntax is blocked by
    /// `#[non_exhaustive]`.
    #[must_use]
    pub fn with_mins(prr_min: f64, chi_square_min: f64, case_count_min: u64) -> Self {
        Self {
            prr_min,
            chi_square_min,
            case_count_min,
            ..Self::default()
        }
    }

    /// Strict thresholds (fewer false positives).
    pub fn strict() -> Self {
        Self {
            prr_min: 3.0,
            chi_square_min: 6.635,
            case_count_min: 5,
            ror_lower_ci_min: 2.0,
            ic025_min: 1.0,
            eb05_min: 3.0,
        }
    }

    /// Sensitive thresholds (fewer false negatives).
    pub fn sensitive() -> Self {
        Self {
            prr_min: 1.5,
            chi_square_min: 2.706,
            case_count_min: 2,
            ror_lower_ci_min: 1.0,
            ic025_min: -0.5,
            eb05_min: 1.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contingency_table_prr() {
        let t = ContingencyTable {
            a: 15,
            b: 100,
            c: 20,
            d: 10_000,
        };
        let prr = t.prr().expect("should compute PRR");
        assert!(prr > 2.0, "PRR={prr} should exceed Evans threshold");
    }

    #[test]
    fn contingency_table_ror() {
        let t = ContingencyTable {
            a: 15,
            b: 100,
            c: 20,
            d: 10_000,
        };
        let ror = t.ror().expect("should compute ROR");
        assert!(ror > 1.0, "ROR={ror} should indicate signal");
    }

    #[test]
    fn signal_strength_classification() {
        assert_eq!(SignalStrength::from_prr(0.5), SignalStrength::None);
        assert_eq!(SignalStrength::from_prr(1.8), SignalStrength::Weak);
        assert_eq!(SignalStrength::from_prr(2.5), SignalStrength::Moderate);
        assert_eq!(SignalStrength::from_prr(4.0), SignalStrength::Strong);
        assert_eq!(SignalStrength::from_prr(10.0), SignalStrength::Critical);
    }

    #[test]
    fn threshold_configs() {
        let evans = ThresholdConfig::default();
        let strict = ThresholdConfig::strict();
        let sensitive = ThresholdConfig::sensitive();
        assert!(strict.prr_min > evans.prr_min);
        assert!(sensitive.prr_min < evans.prr_min);
    }
}
