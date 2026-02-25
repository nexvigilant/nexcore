//! Core types for the Preemptive Pharmacovigilance system.
//!
//! Defines the fundamental domain types used across all three tiers
//! of signal detection: Reactive, Predictive, and Preemptive.

#![allow(
    clippy::doc_markdown,
    reason = "Domain abbreviations and product terms are intentional nomenclature"
)]

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Seriousness (ICH E2A)
// ---------------------------------------------------------------------------

/// ICH E2A seriousness categories for adverse events.
///
/// Tier: T2-P (maps to N Quantity via severity score)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Seriousness {
    /// Non-serious adverse event (severity score = 1)
    NonSerious,
    /// Results in hospitalization or prolongation (severity score = 2)
    Hospitalization,
    /// Results in persistent or significant disability (severity score = 3)
    Disability,
    /// Life-threatening adverse event (severity score = 4)
    LifeThreatening,
    /// Results in death (severity score = 5)
    Fatal,
}

impl Seriousness {
    /// Returns the severity score S (1-5) per ICH E2A.
    #[must_use]
    pub fn severity_score(self) -> f64 {
        match self {
            Self::NonSerious => 1.0,
            Self::Hospitalization => 2.0,
            Self::Disability => 3.0,
            Self::LifeThreatening => 4.0,
            Self::Fatal => 5.0,
        }
    }

    /// Returns the irreversibility factor (0.0 - 1.0).
    ///
    /// Higher values indicate outcomes that cannot be reversed.
    #[must_use]
    pub fn irreversibility_factor(self) -> f64 {
        match self {
            Self::NonSerious => 0.0,
            Self::Hospitalization => 0.3,
            Self::Disability => 0.7,
            Self::LifeThreatening => 0.8,
            Self::Fatal => 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Safety Lambda
// ---------------------------------------------------------------------------

/// Safety lambda multiplier that lowers detection threshold for severe events.
///
/// Tier: T2-P (maps to boundary + quantity)
///
/// Applied as: `theta_preemptive = theta_detection * lambda_safety`
/// Lower lambda = more sensitive detection for severe outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SafetyLambda(f64);

impl SafetyLambda {
    /// Derives the safety lambda from seriousness.
    ///
    /// - Fatal / Life-threatening: 0.3
    /// - Hospitalization+: 0.5
    /// - Otherwise: 0.7
    #[must_use]
    pub fn from_seriousness(seriousness: Seriousness) -> Self {
        let lambda = match seriousness {
            Seriousness::Fatal | Seriousness::LifeThreatening => 0.3,
            Seriousness::Hospitalization | Seriousness::Disability => 0.5,
            Seriousness::NonSerious => 0.7,
        };
        Self(lambda)
    }

    /// Returns the raw lambda value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }

    /// Applies the safety lambda to a detection threshold.
    ///
    /// Returns `theta_detection * lambda_safety`.
    #[must_use]
    pub fn apply(self, theta_detection: f64) -> f64 {
        theta_detection * self.0
    }
}

// ---------------------------------------------------------------------------
// Drug-Event Pair
// ---------------------------------------------------------------------------

/// A drug-event pair identifying a specific drug and adverse event combination.
///
/// Tier: T2-P (product type: string x string)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DrugEventPair {
    /// Drug identifier (e.g., generic name, NDC, or RxNorm code)
    pub drug_id: String,
    /// Adverse event identifier (e.g., MedDRA PT or SOC)
    pub event_id: String,
}

impl DrugEventPair {
    /// Creates a new drug-event pair.
    #[must_use]
    pub fn new(drug_id: impl Into<String>, event_id: impl Into<String>) -> Self {
        Self {
            drug_id: drug_id.into(),
            event_id: event_id.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Reporting Counts (2x2 contingency table)
// ---------------------------------------------------------------------------

/// Reporting counts from a 2x2 contingency table for signal detection.
///
/// Tier: T2-P (product type: N x N x N x N)
///
/// ```text
///                 | Event+ | Event- | Total
/// Drug+           |   a    |   b    | a + b
/// Drug-           |   c    |   d    | c + d
/// Total           | a + c  | b + d  | N
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReportingCounts {
    /// Cases with both the drug and event (cell a)
    pub a: f64,
    /// Cases with the drug but not the event (cell b)
    pub b: f64,
    /// Cases with the event but not the drug (cell c)
    pub c: f64,
    /// Cases with neither the drug nor event (cell d)
    pub d: f64,
}

impl ReportingCounts {
    /// Creates new reporting counts from the four cells.
    #[must_use]
    pub fn new(a: f64, b: f64, c: f64, d: f64) -> Self {
        Self { a, b, c, d }
    }

    /// Total number of reports (N = a + b + c + d).
    #[must_use]
    pub fn total(&self) -> f64 {
        self.a + self.b + self.c + self.d
    }

    /// Expected count under independence: E(a) = (a+b)(a+c) / N.
    #[must_use]
    pub fn expected(&self) -> f64 {
        let n = self.total();
        if n == 0.0 {
            return 0.0;
        }
        (self.a + self.b) * (self.a + self.c) / n
    }
}

// ---------------------------------------------------------------------------
// Decision Outcomes
// ---------------------------------------------------------------------------

/// Three-tier decision outcome from the preemptive PV system.
///
/// Tier: T2-C (sum type: variant selection based on tier evaluation)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Decision {
    /// Tier 1: Continue monitoring with current data.
    Monitor {
        /// The reactive signal strength S(d,e).
        signal_strength: f64,
    },
    /// Tier 2: Predictive signal detected; enhanced surveillance warranted.
    Predict {
        /// The preemptive signal potential Psi.
        psi: f64,
        /// Trajectory direction (positive = escalating).
        trajectory: f64,
    },
    /// Tier 3: Preemptive intervention recommended.
    Intervene {
        /// The intervention index Pi.
        pi: f64,
        /// The irreversibility-weighted severity Omega.
        omega: f64,
        /// Recommended intervention strength [I].
        recommended_intervention_strength: f64,
    },
}

impl Decision {
    /// Returns the tier number (1, 2, or 3).
    #[must_use]
    pub fn tier(&self) -> u8 {
        match self {
            Self::Monitor { .. } => 1,
            Self::Predict { .. } => 2,
            Self::Intervene { .. } => 3,
        }
    }

    /// Returns true if the decision recommends intervention.
    #[must_use]
    pub fn requires_intervention(&self) -> bool {
        matches!(self, Self::Intervene { .. })
    }
}

// ---------------------------------------------------------------------------
// Temporal Reporting Data
// ---------------------------------------------------------------------------

/// A time-series data point for reporting rate analysis.
///
/// Tier: T2-P (product type: time x rate)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReportingDataPoint {
    /// Time in arbitrary units (e.g., months since launch)
    pub time: f64,
    /// Reporting rate at this time point
    pub rate: f64,
}

impl ReportingDataPoint {
    /// Creates a new data point.
    #[must_use]
    pub fn new(time: f64, rate: f64) -> Self {
        Self { time, rate }
    }
}

// ---------------------------------------------------------------------------
// Intervention Result
// ---------------------------------------------------------------------------

/// Result of applying a competitive inhibition intervention model.
///
/// Tier: T2-C (product of inhibited rate + reduction metrics)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InterventionResult {
    /// The inhibited (reduced) harm rate after intervention
    pub inhibited_rate: f64,
    /// The original (uninhibited) harm rate
    pub original_rate: f64,
    /// Rate reduction as a fraction (0.0 - 1.0)
    pub reduction_fraction: f64,
    /// Rate reduction as a percentage (0.0 - 100.0)
    pub reduction_percentage: f64,
}

// ---------------------------------------------------------------------------
// Gibbs Parameters
// ---------------------------------------------------------------------------

/// Parameters for the Gibbs free energy signal feasibility calculation.
///
/// Tier: T2-P (product type: enthalpy x temperature x entropy)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GibbsParams {
    /// Pharmacological harm enthalpy: how strongly the mechanism predicts the AE.
    /// Higher values indicate stronger mechanistic plausibility (0.0 - 10.0 typical).
    pub delta_h_mechanism: f64,
    /// Market temperature in patient-years of exposure.
    /// Higher values indicate more opportunity for signal emergence.
    pub t_exposure: f64,
    /// Information entropy of the evidence base.
    /// Higher values indicate greater uncertainty / more scattered evidence.
    pub delta_s_information: f64,
}

impl GibbsParams {
    /// Creates new Gibbs parameters.
    #[must_use]
    pub fn new(delta_h_mechanism: f64, t_exposure: f64, delta_s_information: f64) -> Self {
        Self {
            delta_h_mechanism,
            t_exposure,
            delta_s_information,
        }
    }
}

// ---------------------------------------------------------------------------
// Noise Floor Parameters
// ---------------------------------------------------------------------------

/// Parameters for the noise floor correction (Nernst-inspired sigmoid).
///
/// Tier: T2-P (product: baseline + stimulated rates + sensitivity)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NoiseParams {
    /// Stimulated reporting rate (e.g., from media attention or regulatory action)
    pub r_stimulated: f64,
    /// Baseline organic reporting rate
    pub r_baseline: f64,
    /// Sensitivity parameter k controlling sigmoid steepness (default: 5.0)
    pub k: f64,
}

impl NoiseParams {
    /// Creates new noise parameters with default k = 5.0.
    #[must_use]
    pub fn new(r_stimulated: f64, r_baseline: f64) -> Self {
        Self {
            r_stimulated,
            r_baseline,
            k: 5.0,
        }
    }

    /// Creates new noise parameters with custom k.
    #[must_use]
    pub fn with_k(r_stimulated: f64, r_baseline: f64, k: f64) -> Self {
        Self {
            r_stimulated,
            r_baseline,
            k,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seriousness_severity_scores() {
        assert!((Seriousness::NonSerious.severity_score() - 1.0).abs() < f64::EPSILON);
        assert!((Seriousness::Hospitalization.severity_score() - 2.0).abs() < f64::EPSILON);
        assert!((Seriousness::Disability.severity_score() - 3.0).abs() < f64::EPSILON);
        assert!((Seriousness::LifeThreatening.severity_score() - 4.0).abs() < f64::EPSILON);
        assert!((Seriousness::Fatal.severity_score() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn seriousness_irreversibility_factors() {
        assert!((Seriousness::NonSerious.irreversibility_factor() - 0.0).abs() < f64::EPSILON);
        assert!((Seriousness::Hospitalization.irreversibility_factor() - 0.3).abs() < f64::EPSILON);
        assert!((Seriousness::Disability.irreversibility_factor() - 0.7).abs() < f64::EPSILON);
        assert!((Seriousness::LifeThreatening.irreversibility_factor() - 0.8).abs() < f64::EPSILON);
        assert!((Seriousness::Fatal.irreversibility_factor() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn safety_lambda_from_seriousness() {
        let fatal = SafetyLambda::from_seriousness(Seriousness::Fatal);
        assert!((fatal.value() - 0.3).abs() < f64::EPSILON);

        let lt = SafetyLambda::from_seriousness(Seriousness::LifeThreatening);
        assert!((lt.value() - 0.3).abs() < f64::EPSILON);

        let hosp = SafetyLambda::from_seriousness(Seriousness::Hospitalization);
        assert!((hosp.value() - 0.5).abs() < f64::EPSILON);

        let dis = SafetyLambda::from_seriousness(Seriousness::Disability);
        assert!((dis.value() - 0.5).abs() < f64::EPSILON);

        let ns = SafetyLambda::from_seriousness(Seriousness::NonSerious);
        assert!((ns.value() - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn safety_lambda_apply() {
        let lambda = SafetyLambda::from_seriousness(Seriousness::Fatal);
        let result = lambda.apply(2.0);
        assert!((result - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn drug_event_pair_creation() {
        let pair = DrugEventPair::new("ibuprofen", "GI_bleeding");
        assert_eq!(pair.drug_id, "ibuprofen");
        assert_eq!(pair.event_id, "GI_bleeding");
    }

    #[test]
    fn reporting_counts_total() {
        let counts = ReportingCounts::new(15.0, 100.0, 20.0, 10000.0);
        assert!((counts.total() - 10135.0).abs() < f64::EPSILON);
    }

    #[test]
    fn reporting_counts_expected() {
        let counts = ReportingCounts::new(15.0, 100.0, 20.0, 10000.0);
        let expected = (15.0 + 100.0) * (15.0 + 20.0) / 10135.0;
        assert!((counts.expected() - expected).abs() < 1e-10);
    }

    #[test]
    fn reporting_counts_zero_total() {
        let counts = ReportingCounts::new(0.0, 0.0, 0.0, 0.0);
        assert!((counts.expected() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn decision_tier_numbers() {
        let monitor = Decision::Monitor {
            signal_strength: 1.5,
        };
        assert_eq!(monitor.tier(), 1);
        assert!(!monitor.requires_intervention());

        let predict = Decision::Predict {
            psi: 2.0,
            trajectory: 0.5,
        };
        assert_eq!(predict.tier(), 2);
        assert!(!predict.requires_intervention());

        let intervene = Decision::Intervene {
            pi: 3.0,
            omega: 7.2,
            recommended_intervention_strength: 10.0,
        };
        assert_eq!(intervene.tier(), 3);
        assert!(intervene.requires_intervention());
    }

    #[test]
    fn gibbs_params_creation() {
        let params = GibbsParams::new(5.0, 1000.0, 0.01);
        assert!((params.delta_h_mechanism - 5.0).abs() < f64::EPSILON);
        assert!((params.t_exposure - 1000.0).abs() < f64::EPSILON);
        assert!((params.delta_s_information - 0.01).abs() < f64::EPSILON);
    }

    #[test]
    fn noise_params_default_k() {
        let params = NoiseParams::new(100.0, 50.0);
        assert!((params.k - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn noise_params_custom_k() {
        let params = NoiseParams::with_k(100.0, 50.0, 10.0);
        assert!((params.k - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn reporting_data_point_creation() {
        let dp = ReportingDataPoint::new(3.0, 15.5);
        assert!((dp.time - 3.0).abs() < f64::EPSILON);
        assert!((dp.rate - 15.5).abs() < f64::EPSILON);
    }

    #[test]
    fn types_serialize_roundtrip() {
        let pair = DrugEventPair::new("aspirin", "tinnitus");
        let json = serde_json::to_string(&pair);
        assert!(json.is_ok());
        let json_str = json.unwrap_or_default();
        let deserialized: Result<DrugEventPair, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
    }
}
