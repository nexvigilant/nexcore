//! # Epidemiological Measurement Primitives
//!
//! Standard epidemiological metrics for pharmacovigilance signal characterization.
//! These are direct domain mappings (not cross-domain transfers) — they formalize
//! the fundamental measures every PV system must compute.
//!
//! ## Type Inventory
//!
//! | Type | What It Measures | T1 Grounding | Priority |
//! |------|------------------|--------------|----------|
//! | `IncidenceRate` | Cases per person-time at risk | ν (Frequency) | P1 |
//! | `Prevalence` | Proportion with condition | ν (Frequency) | P1 |
//! | `AttributableRisk` | Risk difference (exposed − unexposed) | κ (Comparison) | P0 |
//! | `PopulationAttributableFraction` | Public health impact proportion | × (Product) | P0 |
//! | `DoseResponseCurve` | Hill equation: EC50, Emax, slope | ρ+∂ (Recursion+Boundary) | P1 |
//! | `SignalToNoiseRatio` | Detection method discrimination | κ (Comparison) | P1 |
//! | `Sensitivity` | True positive rate | κ (Comparison) | P1 |
//! | `Specificity` | True negative rate | κ (Comparison) | P1 |
//! | `PositivePredictiveValue` | P(true signal │ detected) | κ (Comparison) | P1 |
//! | `NegativePredictiveValue` | P(true noise │ not detected) | κ (Comparison) | P1 |
//!
//! ## Tier Classification
//!
//! - **T2-P**: IncidenceRate, Prevalence, AttributableRisk, SNR, Sensitivity,
//!             Specificity, PPV, NPV (single T1 dominant)
//! - **T2-C**: PopulationAttributableFraction, DoseResponseCurve (multi-T1)

use crate::lex_primitiva::{GroundsTo, LexPrimitiva, PrimitiveComposition};
use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// 1. IncidenceRate — Cases per person-time at risk
//    T1: ν (Frequency)
//    Priority: P1 (Signal Integrity)
// ============================================================================

/// Number of new cases per person-time at risk.
///
/// # Lex Primitiva
/// - **ν (Frequency)** dominant: rate of occurrence per exposure unit
///
/// # Formula
/// IR = new_cases / person_time_at_risk
///
/// # Units
/// Typically expressed as cases per 1,000 patient-years (PY).
///
/// Tier: T2-P (ν)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IncidenceRate {
    /// Raw rate (cases per person-time unit)
    rate: f64,
    /// Numerator: new cases observed
    cases: u64,
    /// Denominator: person-time at risk
    person_time: f64,
}

impl IncidenceRate {
    /// Compute incidence rate from case count and person-time.
    ///
    /// Returns `None` if person-time is zero or negative.
    #[must_use]
    pub fn new(cases: u64, person_time: f64) -> Option<Self> {
        if person_time <= 0.0 || !person_time.is_finite() {
            return None;
        }

        Some(Self {
            rate: cases as f64 / person_time,
            cases,
            person_time,
        })
    }

    /// Get raw rate (per 1 person-time unit).
    #[inline]
    #[must_use]
    pub fn rate(&self) -> f64 {
        self.rate
    }

    /// Rate per 1,000 patient-years (standard PV reporting unit).
    #[inline]
    #[must_use]
    pub fn per_1000_py(&self) -> f64 {
        self.rate * 1000.0
    }

    /// Rate per 10,000 patient-years.
    #[inline]
    #[must_use]
    pub fn per_10000_py(&self) -> f64 {
        self.rate * 10000.0
    }

    /// Case count.
    #[inline]
    #[must_use]
    pub fn cases(&self) -> u64 {
        self.cases
    }

    /// Person-time at risk.
    #[inline]
    #[must_use]
    pub fn person_time(&self) -> f64 {
        self.person_time
    }

    /// Compare two incidence rates (rate ratio).
    /// Returns `None` if comparator rate is zero.
    #[must_use]
    pub fn rate_ratio(&self, comparator: &Self) -> Option<f64> {
        if comparator.rate < f64::EPSILON {
            None
        } else {
            Some(self.rate / comparator.rate)
        }
    }
}

impl GroundsTo for IncidenceRate {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Frequency])
    }
}

impl fmt::Display for IncidenceRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "IR={:.2}/1000PY ({} cases / {:.1} PY)",
            self.per_1000_py(),
            self.cases,
            self.person_time
        )
    }
}

// ============================================================================
// 2. Prevalence — Proportion with condition at a point in time
//    T1: ν (Frequency)
//    Priority: P1 (Signal Integrity)
// ============================================================================

/// Proportion of a population with a condition at a specific point in time.
///
/// # Lex Primitiva
/// - **ν (Frequency)** dominant: proportion as rate
///
/// # Formula
/// Prevalence = affected / total_population
///
/// # Variants
/// - Point prevalence: at a specific moment
/// - Period prevalence: during a time window
///
/// Tier: T2-P (ν)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Prevalence {
    /// Proportion [0.0, 1.0]
    proportion: f64,
    /// Affected count
    affected: u64,
    /// Total population
    total: u64,
}

impl Prevalence {
    /// Compute prevalence from affected count and total population.
    ///
    /// Returns `None` if total is zero.
    #[must_use]
    pub fn new(affected: u64, total: u64) -> Option<Self> {
        if total == 0 {
            return None;
        }

        Some(Self {
            proportion: affected as f64 / total as f64,
            affected,
            total,
        })
    }

    /// Get the proportion [0.0, 1.0].
    #[inline]
    #[must_use]
    pub fn proportion(&self) -> f64 {
        self.proportion
    }

    /// Express as percentage.
    #[inline]
    #[must_use]
    pub fn percentage(&self) -> f64 {
        self.proportion * 100.0
    }

    /// Express per 100,000 (standard epidemiological rate).
    #[inline]
    #[must_use]
    pub fn per_100k(&self) -> f64 {
        self.proportion * 100_000.0
    }

    /// Affected count.
    #[inline]
    #[must_use]
    pub fn affected(&self) -> u64 {
        self.affected
    }

    /// Total population.
    #[inline]
    #[must_use]
    pub fn total(&self) -> u64 {
        self.total
    }

    /// Is this a rare condition (< 1%)?
    #[inline]
    #[must_use]
    pub fn is_rare(&self) -> bool {
        self.proportion < 0.01
    }

    /// Is this a common condition (>= 10%)?
    #[inline]
    #[must_use]
    pub fn is_common(&self) -> bool {
        self.proportion >= 0.10
    }
}

impl GroundsTo for Prevalence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Frequency])
    }
}

impl fmt::Display for Prevalence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Prevalence={:.2}% ({}/{}, {:.1}/100k)",
            self.percentage(),
            self.affected,
            self.total,
            self.per_100k()
        )
    }
}

// ============================================================================
// 3. AttributableRisk — Risk difference (absolute risk increase)
//    T1: κ (Comparison)
//    Priority: P0 (Patient Safety)
// ============================================================================

/// Absolute risk difference between exposed and unexposed groups.
/// The foundational metric for NNH calculation.
///
/// # Lex Primitiva
/// - **κ (Comparison)** dominant: difference between two risk measures
///
/// # Formula
/// AR = risk_exposed − risk_unexposed
///
/// # Relationship
/// NNH = 1 / AR (when AR > 0)
///
/// Tier: T2-P (κ)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AttributableRisk {
    /// Absolute risk difference
    difference: f64,
    /// Risk in exposed group [0.0, 1.0]
    risk_exposed: f64,
    /// Risk in unexposed group [0.0, 1.0]
    risk_unexposed: f64,
}

impl AttributableRisk {
    /// Compute attributable risk from group risks.
    ///
    /// Returns `None` if risks are outside [0.0, 1.0].
    #[must_use]
    pub fn from_risks(risk_exposed: f64, risk_unexposed: f64) -> Option<Self> {
        if !(0.0..=1.0).contains(&risk_exposed) || !(0.0..=1.0).contains(&risk_unexposed) {
            return None;
        }

        Some(Self {
            difference: risk_exposed - risk_unexposed,
            risk_exposed,
            risk_unexposed,
        })
    }

    /// Compute from a 2x2 contingency table.
    /// RE = a/(a+b), RU = c/(c+d).
    #[must_use]
    pub fn from_contingency(a: u64, b: u64, c: u64, d: u64) -> Option<Self> {
        let exp_total = a + b;
        let unexp_total = c + d;
        if exp_total == 0 || unexp_total == 0 {
            return None;
        }

        Self::from_risks(a as f64 / exp_total as f64, c as f64 / unexp_total as f64)
    }

    /// Get absolute risk difference.
    #[inline]
    #[must_use]
    pub fn difference(&self) -> f64 {
        self.difference
    }

    /// Risk in exposed group.
    #[inline]
    #[must_use]
    pub fn risk_exposed(&self) -> f64 {
        self.risk_exposed
    }

    /// Risk in unexposed group.
    #[inline]
    #[must_use]
    pub fn risk_unexposed(&self) -> f64 {
        self.risk_unexposed
    }

    /// Relative risk (risk ratio).
    #[must_use]
    pub fn relative_risk(&self) -> Option<f64> {
        if self.risk_unexposed < f64::EPSILON {
            None
        } else {
            Some(self.risk_exposed / self.risk_unexposed)
        }
    }

    /// Is this a harmful exposure (AR > 0)?
    #[inline]
    #[must_use]
    pub fn is_harmful(&self) -> bool {
        self.difference > f64::EPSILON
    }

    /// Is this a protective exposure (AR < 0)?
    #[inline]
    #[must_use]
    pub fn is_protective(&self) -> bool {
        self.difference < -f64::EPSILON
    }

    /// NNH derived from this AR (1/AR when harmful).
    #[must_use]
    pub fn nnh(&self) -> Option<f64> {
        if self.difference.abs() < f64::EPSILON {
            None
        } else {
            Some(1.0 / self.difference)
        }
    }
}

impl GroundsTo for AttributableRisk {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
    }
}

impl fmt::Display for AttributableRisk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = if self.is_harmful() {
            "harmful"
        } else if self.is_protective() {
            "protective"
        } else {
            "neutral"
        };
        write!(
            f,
            "AR={:+.4} ({}, RE={:.4}, RU={:.4})",
            self.difference, label, self.risk_exposed, self.risk_unexposed
        )
    }
}

// ============================================================================
// 4. PopulationAttributableFraction — Public health impact
//    T1: × (Product)
//    Priority: P0 (Patient Safety)
// ============================================================================

/// Proportion of disease in the total population attributable to the exposure.
/// Answers: "If we removed this drug, how much would the AE rate drop?"
///
/// # Lex Primitiva
/// - **× (Product)** dominant: prevalence × relative_risk interaction
///
/// # Formula
/// PAF = P_e × (RR − 1) / (P_e × (RR − 1) + 1)
/// where P_e = prevalence of exposure, RR = relative risk
///
/// # Interpretation
/// - PAF = 0.30: 30% of cases in the population are attributable to this drug
/// - PAF = 0.01: 1% of cases — low public health impact
///
/// Tier: T2-C (× + ν + κ)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PopulationAttributableFraction {
    /// PAF value [0.0, 1.0]
    fraction: f64,
    /// Prevalence of exposure in population
    exposure_prevalence: f64,
    /// Relative risk
    relative_risk: f64,
}

impl PopulationAttributableFraction {
    /// Compute PAF from exposure prevalence and relative risk.
    ///
    /// Returns `None` if inputs are invalid.
    #[must_use]
    pub fn new(exposure_prevalence: f64, relative_risk: f64) -> Option<Self> {
        if !(0.0..=1.0).contains(&exposure_prevalence) || relative_risk < 0.0 {
            return None;
        }

        let numerator = exposure_prevalence * (relative_risk - 1.0);
        let denominator = numerator + 1.0;

        if denominator.abs() < f64::EPSILON {
            return None;
        }

        let fraction = (numerator / denominator).clamp(0.0, 1.0);

        Some(Self {
            fraction,
            exposure_prevalence,
            relative_risk,
        })
    }

    /// Get the PAF value [0.0, 1.0].
    #[inline]
    #[must_use]
    pub fn fraction(&self) -> f64 {
        self.fraction
    }

    /// Express as percentage.
    #[inline]
    #[must_use]
    pub fn percentage(&self) -> f64 {
        self.fraction * 100.0
    }

    /// Exposure prevalence.
    #[inline]
    #[must_use]
    pub fn exposure_prevalence(&self) -> f64 {
        self.exposure_prevalence
    }

    /// Relative risk used.
    #[inline]
    #[must_use]
    pub fn relative_risk(&self) -> f64 {
        self.relative_risk
    }

    /// Is this a significant public health impact (PAF > 10%)?
    #[inline]
    #[must_use]
    pub fn is_significant(&self) -> bool {
        self.fraction > 0.10
    }

    /// Estimated preventable cases given total case count.
    #[inline]
    #[must_use]
    pub fn preventable_cases(&self, total_cases: u64) -> f64 {
        total_cases as f64 * self.fraction
    }
}

impl GroundsTo for PopulationAttributableFraction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,
            LexPrimitiva::Frequency,
            LexPrimitiva::Comparison,
        ])
    }
}

impl fmt::Display for PopulationAttributableFraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PAF={:.1}% (P_e={:.3}, RR={:.2})",
            self.percentage(),
            self.exposure_prevalence,
            self.relative_risk
        )
    }
}

// ============================================================================
// 5. DoseResponseCurve — Hill equation parameters
//    T1: ρ (Recursion) + ∂ (Boundary)
//    Priority: P1 (Signal Integrity)
// ============================================================================

/// Sigmoidal dose-response relationship modeled by the Hill equation.
///
/// # Lex Primitiva
/// - **ρ (Recursion)** dominant: self-similar cooperative binding
/// - **∂ (Boundary)**: EC50 is the half-maximal boundary
///
/// # Hill Equation
/// E = Emax × D^n / (EC50^n + D^n)
///
/// where:
/// - E = effect at dose D
/// - Emax = maximum achievable effect
/// - EC50 = dose producing 50% of Emax
/// - n = Hill coefficient (cooperativity)
///
/// # Interpretation
/// - n = 1: Standard hyperbolic (Michaelis-Menten)
/// - n > 1: Positive cooperativity (steep transition)
/// - n < 1: Negative cooperativity (gradual transition)
///
/// Tier: T2-C (ρ + ∂)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DoseResponseCurve {
    /// Maximum effect (Emax)
    emax: f64,
    /// Half-maximal effective concentration
    ec50: f64,
    /// Hill coefficient (cooperativity)
    hill_coefficient: f64,
}

impl DoseResponseCurve {
    /// Create a dose-response curve from Hill parameters.
    ///
    /// Returns `None` if parameters are non-positive or non-finite.
    #[must_use]
    pub fn new(emax: f64, ec50: f64, hill_coefficient: f64) -> Option<Self> {
        if emax <= 0.0
            || ec50 <= 0.0
            || hill_coefficient <= 0.0
            || !emax.is_finite()
            || !ec50.is_finite()
            || !hill_coefficient.is_finite()
        {
            return None;
        }

        Some(Self {
            emax,
            ec50,
            hill_coefficient,
        })
    }

    /// Standard Michaelis-Menten (Hill coefficient = 1).
    #[must_use]
    pub fn michaelis_menten(emax: f64, ec50: f64) -> Option<Self> {
        Self::new(emax, ec50, 1.0)
    }

    /// Compute effect at a given dose.
    #[must_use]
    pub fn effect_at_dose(&self, dose: f64) -> f64 {
        if dose <= 0.0 {
            return 0.0;
        }

        let d_n = dose.powf(self.hill_coefficient);
        let ec50_n = self.ec50.powf(self.hill_coefficient);

        self.emax * d_n / (ec50_n + d_n)
    }

    /// Fraction of maximum effect at a given dose [0.0, 1.0].
    #[must_use]
    pub fn fractional_effect(&self, dose: f64) -> f64 {
        self.effect_at_dose(dose) / self.emax
    }

    /// Dose required to achieve a target fraction of Emax.
    ///
    /// Inverts the Hill equation: D = EC50 × (f/(1-f))^(1/n)
    #[must_use]
    pub fn dose_for_fraction(&self, fraction: f64) -> Option<f64> {
        if !(f64::EPSILON..1.0).contains(&fraction) {
            return None;
        }

        let ratio = fraction / (1.0 - fraction);
        Some(self.ec50 * ratio.powf(1.0 / self.hill_coefficient))
    }

    /// Maximum effect.
    #[inline]
    #[must_use]
    pub fn emax(&self) -> f64 {
        self.emax
    }

    /// Half-maximal effective concentration.
    #[inline]
    #[must_use]
    pub fn ec50(&self) -> f64 {
        self.ec50
    }

    /// Hill coefficient.
    #[inline]
    #[must_use]
    pub fn hill_coefficient(&self) -> f64 {
        self.hill_coefficient
    }

    /// Is this a cooperative response (steep)?
    #[inline]
    #[must_use]
    pub fn is_cooperative(&self) -> bool {
        self.hill_coefficient > 1.0
    }

    /// Is this anti-cooperative (gradual)?
    #[inline]
    #[must_use]
    pub fn is_anti_cooperative(&self) -> bool {
        self.hill_coefficient < 1.0
    }
}

impl GroundsTo for DoseResponseCurve {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Recursion, LexPrimitiva::Boundary])
    }
}

impl fmt::Display for DoseResponseCurve {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DRC(Emax={:.2}, EC50={:.2}, n={:.2})",
            self.emax, self.ec50, self.hill_coefficient
        )
    }
}

// ============================================================================
// 6. SignalToNoiseRatio — Detection method discrimination
//    T1: κ (Comparison)
//    Priority: P1 (Signal Integrity)
// ============================================================================

/// Ratio of true signal strength to background noise in detection.
///
/// # Lex Primitiva
/// - **κ (Comparison)** dominant: signal vs noise comparison
///
/// # Formula
/// SNR = signal_power / noise_power
/// SNR_dB = 10 × log10(signal_power / noise_power)
///
/// # PV Interpretation
/// - SNR > 10: Clear signal, high confidence detection
/// - SNR 3–10: Moderate, warrants investigation
/// - SNR < 3: Weak, close to noise floor
///
/// Tier: T2-P (κ)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SignalToNoiseRatio {
    /// Linear ratio (signal/noise)
    ratio: f64,
    /// Signal power (observed disproportionality)
    signal_power: f64,
    /// Noise power (expected background rate)
    noise_power: f64,
}

impl SignalToNoiseRatio {
    /// Compute SNR from signal and noise power.
    ///
    /// Returns `None` if noise is zero.
    #[must_use]
    pub fn new(signal_power: f64, noise_power: f64) -> Option<Self> {
        if noise_power.abs() < f64::EPSILON || !signal_power.is_finite() || !noise_power.is_finite()
        {
            return None;
        }

        Some(Self {
            ratio: signal_power / noise_power,
            signal_power,
            noise_power,
        })
    }

    /// Compute from PRR (signal = PRR - 1.0, noise = 1/sqrt(n)).
    /// PRR of 1.0 = no signal. Cases = cell a count.
    #[must_use]
    pub fn from_prr(prr: f64, cases: u64) -> Option<Self> {
        if cases == 0 {
            return None;
        }
        let signal = (prr - 1.0).abs();
        let noise = 1.0 / (cases as f64).sqrt();
        Self::new(signal, noise)
    }

    /// Get the linear ratio.
    #[inline]
    #[must_use]
    pub fn ratio(&self) -> f64 {
        self.ratio
    }

    /// Get SNR in decibels.
    #[inline]
    #[must_use]
    pub fn decibels(&self) -> f64 {
        if self.ratio <= 0.0 {
            return f64::NEG_INFINITY;
        }
        10.0 * self.ratio.log10()
    }

    /// Signal power.
    #[inline]
    #[must_use]
    pub fn signal_power(&self) -> f64 {
        self.signal_power
    }

    /// Noise power.
    #[inline]
    #[must_use]
    pub fn noise_power(&self) -> f64 {
        self.noise_power
    }

    /// Is the signal clear (SNR > 10)?
    #[inline]
    #[must_use]
    pub fn is_clear(&self) -> bool {
        self.ratio > 10.0
    }

    /// Is the signal moderate (3 ≤ SNR ≤ 10)?
    #[inline]
    #[must_use]
    pub fn is_moderate(&self) -> bool {
        self.ratio >= 3.0 && self.ratio <= 10.0
    }

    /// Is the signal weak (SNR < 3)?
    #[inline]
    #[must_use]
    pub fn is_weak(&self) -> bool {
        self.ratio < 3.0
    }
}

impl GroundsTo for SignalToNoiseRatio {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
    }
}

impl fmt::Display for SignalToNoiseRatio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let quality = if self.is_clear() {
            "clear"
        } else if self.is_moderate() {
            "moderate"
        } else {
            "weak"
        };
        write!(
            f,
            "SNR={:.2} ({:.1}dB, {})",
            self.ratio,
            self.decibels(),
            quality
        )
    }
}

// ============================================================================
// 7–10. Diagnostic Test Performance: Sensitivity, Specificity, PPV, NPV
//    T1: κ (Comparison)
//    Priority: P1 (Signal Integrity)
// ============================================================================

/// Confusion matrix for signal detection algorithm evaluation.
///
/// ```text
///                   Truth: Signal    Truth: Noise
/// Detected:  Yes  |     TP         |     FP        |
/// Detected:  No   |     FN         |     TN        |
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConfusionMatrix {
    /// True positives (correctly detected signals)
    pub tp: u64,
    /// False positives (noise detected as signal)
    pub fp: u64,
    /// False negatives (missed signals)
    pub fn_: u64,
    /// True negatives (correctly identified noise)
    pub tn: u64,
}

impl ConfusionMatrix {
    /// Create a confusion matrix.
    #[must_use]
    pub const fn new(tp: u64, fp: u64, fn_: u64, tn: u64) -> Self {
        Self { tp, fp, fn_, tn }
    }

    /// Total observations.
    #[inline]
    #[must_use]
    pub fn total(&self) -> u64 {
        self.tp + self.fp + self.fn_ + self.tn
    }

    /// Compute sensitivity (true positive rate).
    #[must_use]
    pub fn sensitivity(&self) -> Option<Sensitivity> {
        Sensitivity::from_matrix(self)
    }

    /// Compute specificity (true negative rate).
    #[must_use]
    pub fn specificity(&self) -> Option<Specificity> {
        Specificity::from_matrix(self)
    }

    /// Compute positive predictive value.
    #[must_use]
    pub fn ppv(&self) -> Option<PositivePredictiveValue> {
        PositivePredictiveValue::from_matrix(self)
    }

    /// Compute negative predictive value.
    #[must_use]
    pub fn npv(&self) -> Option<NegativePredictiveValue> {
        NegativePredictiveValue::from_matrix(self)
    }

    /// Compute all four metrics at once.
    #[must_use]
    pub fn all_metrics(
        &self,
    ) -> Option<(
        Sensitivity,
        Specificity,
        PositivePredictiveValue,
        NegativePredictiveValue,
    )> {
        Some((
            self.sensitivity()?,
            self.specificity()?,
            self.ppv()?,
            self.npv()?,
        ))
    }
}

impl fmt::Display for ConfusionMatrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CM(TP={}, FP={}, FN={}, TN={})",
            self.tp, self.fp, self.fn_, self.tn
        )
    }
}

/// True positive rate: P(detected | true signal).
/// Tier: T2-P (κ)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Sensitivity(f64);

impl Sensitivity {
    /// Compute from confusion matrix: TP / (TP + FN).
    #[must_use]
    pub fn from_matrix(cm: &ConfusionMatrix) -> Option<Self> {
        let denom = cm.tp + cm.fn_;
        if denom == 0 {
            return None;
        }
        Some(Self(cm.tp as f64 / denom as f64))
    }

    /// Create from raw value [0.0, 1.0].
    #[must_use]
    pub fn new(value: f64) -> Option<Self> {
        if (0.0..=1.0).contains(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Get the value.
    #[inline]
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl GroundsTo for Sensitivity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
    }
}

impl fmt::Display for Sensitivity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sensitivity={:.1}%", self.0 * 100.0)
    }
}

/// True negative rate: P(not detected | true noise).
/// Tier: T2-P (κ)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Specificity(f64);

impl Specificity {
    /// Compute from confusion matrix: TN / (TN + FP).
    #[must_use]
    pub fn from_matrix(cm: &ConfusionMatrix) -> Option<Self> {
        let denom = cm.tn + cm.fp;
        if denom == 0 {
            return None;
        }
        Some(Self(cm.tn as f64 / denom as f64))
    }

    /// Create from raw value [0.0, 1.0].
    #[must_use]
    pub fn new(value: f64) -> Option<Self> {
        if (0.0..=1.0).contains(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Get the value.
    #[inline]
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl GroundsTo for Specificity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
    }
}

impl fmt::Display for Specificity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Specificity={:.1}%", self.0 * 100.0)
    }
}

/// P(true signal | detected). Depends on prevalence.
/// Tier: T2-P (κ)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PositivePredictiveValue(f64);

impl PositivePredictiveValue {
    /// Compute from confusion matrix: TP / (TP + FP).
    #[must_use]
    pub fn from_matrix(cm: &ConfusionMatrix) -> Option<Self> {
        let denom = cm.tp + cm.fp;
        if denom == 0 {
            return None;
        }
        Some(Self(cm.tp as f64 / denom as f64))
    }

    /// Compute from Bayes' theorem: (Se × Prev) / (Se × Prev + (1-Sp) × (1-Prev)).
    #[must_use]
    pub fn from_bayes(sensitivity: f64, specificity: f64, prevalence: f64) -> Option<Self> {
        let numerator = sensitivity * prevalence;
        let denominator = numerator + (1.0 - specificity) * (1.0 - prevalence);
        if denominator.abs() < f64::EPSILON {
            return None;
        }
        Some(Self(numerator / denominator))
    }

    /// Create from raw value [0.0, 1.0].
    #[must_use]
    pub fn new(value: f64) -> Option<Self> {
        if (0.0..=1.0).contains(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Get the value.
    #[inline]
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl GroundsTo for PositivePredictiveValue {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
    }
}

impl fmt::Display for PositivePredictiveValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PPV={:.1}%", self.0 * 100.0)
    }
}

/// P(true noise | not detected). Depends on prevalence.
/// Tier: T2-P (κ)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NegativePredictiveValue(f64);

impl NegativePredictiveValue {
    /// Compute from confusion matrix: TN / (TN + FN).
    #[must_use]
    pub fn from_matrix(cm: &ConfusionMatrix) -> Option<Self> {
        let denom = cm.tn + cm.fn_;
        if denom == 0 {
            return None;
        }
        Some(Self(cm.tn as f64 / denom as f64))
    }

    /// Compute from Bayes' theorem: (Sp × (1-Prev)) / (Sp × (1-Prev) + (1-Se) × Prev).
    #[must_use]
    pub fn from_bayes(sensitivity: f64, specificity: f64, prevalence: f64) -> Option<Self> {
        let numerator = specificity * (1.0 - prevalence);
        let denominator = numerator + (1.0 - sensitivity) * prevalence;
        if denominator.abs() < f64::EPSILON {
            return None;
        }
        Some(Self(numerator / denominator))
    }

    /// Create from raw value [0.0, 1.0].
    #[must_use]
    pub fn new(value: f64) -> Option<Self> {
        if (0.0..=1.0).contains(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Get the value.
    #[inline]
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl GroundsTo for NegativePredictiveValue {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
    }
}

impl fmt::Display for NegativePredictiveValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NPV={:.1}%", self.0 * 100.0)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    mod incidence_tests {
        use super::*;

        #[test]
        fn basic_rate() {
            let ir = IncidenceRate::new(50, 10000.0);
            assert!(ir.is_some());
            let ir = ir.unwrap_or_else(|| unreachable!());
            assert!((ir.rate() - 0.005).abs() < f64::EPSILON);
            assert!((ir.per_1000_py() - 5.0).abs() < f64::EPSILON);
            assert_eq!(ir.cases(), 50);
        }

        #[test]
        fn rate_ratio() {
            let exposed = IncidenceRate::new(50, 10000.0).unwrap_or_else(|| unreachable!());
            let background = IncidenceRate::new(10, 10000.0).unwrap_or_else(|| unreachable!());
            let rr = exposed.rate_ratio(&background);
            assert!(rr.is_some());
            assert!((rr.unwrap_or(0.0) - 5.0).abs() < f64::EPSILON);
        }

        #[test]
        fn zero_person_time() {
            assert!(IncidenceRate::new(10, 0.0).is_none());
            assert!(IncidenceRate::new(10, -5.0).is_none());
        }
    }

    mod prevalence_tests {
        use super::*;

        #[test]
        fn basic_prevalence() {
            let p = Prevalence::new(500, 100_000);
            assert!(p.is_some());
            let p = p.unwrap_or_else(|| unreachable!());
            assert!((p.proportion() - 0.005).abs() < f64::EPSILON);
            assert!((p.percentage() - 0.5).abs() < f64::EPSILON);
            assert!((p.per_100k() - 500.0).abs() < f64::EPSILON);
            assert!(p.is_rare());
            assert!(!p.is_common());
        }

        #[test]
        fn common_condition() {
            let p = Prevalence::new(15000, 100_000);
            assert!(p.is_some());
            let p = p.unwrap_or_else(|| unreachable!());
            assert!(p.is_common());
        }

        #[test]
        fn zero_total() {
            assert!(Prevalence::new(5, 0).is_none());
        }
    }

    mod attributable_risk_tests {
        use super::*;

        #[test]
        fn harmful_exposure() {
            let ar = AttributableRisk::from_risks(0.15, 0.05);
            assert!(ar.is_some());
            let ar = ar.unwrap_or_else(|| unreachable!());
            assert!((ar.difference() - 0.10).abs() < f64::EPSILON);
            assert!(ar.is_harmful());
            let nnh = ar.nnh();
            assert!(nnh.is_some());
            assert!((nnh.unwrap_or(0.0) - 10.0).abs() < f64::EPSILON);
        }

        #[test]
        fn protective_exposure() {
            let ar = AttributableRisk::from_risks(0.02, 0.10);
            assert!(ar.is_some());
            let ar = ar.unwrap_or_else(|| unreachable!());
            assert!(ar.is_protective());
        }

        #[test]
        fn from_contingency() {
            let ar = AttributableRisk::from_contingency(15, 85, 5, 895);
            assert!(ar.is_some());
            let ar = ar.unwrap_or_else(|| unreachable!());
            assert!(ar.is_harmful());
            let rr = ar.relative_risk();
            assert!(rr.is_some());
            assert!(rr.unwrap_or(0.0) > 1.0);
        }

        #[test]
        fn invalid_risks() {
            assert!(AttributableRisk::from_risks(1.5, 0.5).is_none());
            assert!(AttributableRisk::from_risks(0.5, -0.1).is_none());
        }
    }

    mod paf_tests {
        use super::*;

        #[test]
        fn significant_paf() {
            // 30% exposed, RR=3.0 → PAF = 0.3*2/(0.3*2+1) = 0.6/1.6 = 0.375
            let paf = PopulationAttributableFraction::new(0.30, 3.0);
            assert!(paf.is_some());
            let paf = paf.unwrap_or_else(|| unreachable!());
            assert!((paf.fraction() - 0.375).abs() < 0.001);
            assert!(paf.is_significant());
        }

        #[test]
        fn preventable_cases() {
            let paf =
                PopulationAttributableFraction::new(0.30, 3.0).unwrap_or_else(|| unreachable!());
            let prevented = paf.preventable_cases(1000);
            assert!((prevented - 375.0).abs() < 1.0);
        }

        #[test]
        fn low_impact() {
            // 1% exposed, RR=1.5 → PAF = 0.01*0.5/(0.01*0.5+1) = 0.005/1.005 ≈ 0.005
            let paf = PopulationAttributableFraction::new(0.01, 1.5);
            assert!(paf.is_some());
            let paf = paf.unwrap_or_else(|| unreachable!());
            assert!(!paf.is_significant());
        }

        #[test]
        fn invalid_inputs() {
            assert!(PopulationAttributableFraction::new(1.5, 2.0).is_none());
            assert!(PopulationAttributableFraction::new(0.3, -1.0).is_none());
        }
    }

    mod dose_response_tests {
        use super::*;

        #[test]
        fn hill_equation() {
            let drc = DoseResponseCurve::new(100.0, 10.0, 1.0);
            assert!(drc.is_some());
            let drc = drc.unwrap_or_else(|| unreachable!());

            // At EC50, effect should be 50% of Emax
            assert!((drc.effect_at_dose(10.0) - 50.0).abs() < 0.1);
            assert!((drc.fractional_effect(10.0) - 0.5).abs() < 0.01);
        }

        #[test]
        fn cooperative_binding() {
            let drc = DoseResponseCurve::new(100.0, 10.0, 3.0);
            assert!(drc.is_some());
            let drc = drc.unwrap_or_else(|| unreachable!());
            assert!(drc.is_cooperative());

            // At EC50, still 50% (by definition)
            assert!((drc.fractional_effect(10.0) - 0.5).abs() < 0.01);
        }

        #[test]
        fn dose_inversion() {
            let drc = DoseResponseCurve::new(100.0, 10.0, 1.0).unwrap_or_else(|| unreachable!());
            let dose = drc.dose_for_fraction(0.5);
            assert!(dose.is_some());
            assert!((dose.unwrap_or(0.0) - 10.0).abs() < 0.1);
        }

        #[test]
        fn zero_dose() {
            let drc = DoseResponseCurve::new(100.0, 10.0, 1.0).unwrap_or_else(|| unreachable!());
            assert!((drc.effect_at_dose(0.0)).abs() < f64::EPSILON);
        }

        #[test]
        fn invalid_params() {
            assert!(DoseResponseCurve::new(0.0, 10.0, 1.0).is_none());
            assert!(DoseResponseCurve::new(100.0, 0.0, 1.0).is_none());
            assert!(DoseResponseCurve::new(100.0, 10.0, 0.0).is_none());
        }
    }

    mod snr_tests {
        use super::*;

        #[test]
        fn clear_signal() {
            let snr = SignalToNoiseRatio::new(20.0, 1.0);
            assert!(snr.is_some());
            let snr = snr.unwrap_or_else(|| unreachable!());
            assert!(snr.is_clear());
            assert!((snr.ratio() - 20.0).abs() < f64::EPSILON);
            assert!(snr.decibels() > 10.0);
        }

        #[test]
        fn weak_signal() {
            let snr = SignalToNoiseRatio::new(2.0, 1.0);
            assert!(snr.is_some());
            let snr = snr.unwrap_or_else(|| unreachable!());
            assert!(snr.is_weak());
        }

        #[test]
        fn from_prr() {
            // PRR=5.0, 100 cases → signal=4.0, noise=0.1 → SNR=40
            let snr = SignalToNoiseRatio::from_prr(5.0, 100);
            assert!(snr.is_some());
            let snr = snr.unwrap_or_else(|| unreachable!());
            assert!(snr.is_clear());
        }

        #[test]
        fn zero_noise() {
            assert!(SignalToNoiseRatio::new(5.0, 0.0).is_none());
        }
    }

    mod diagnostic_tests {
        use super::*;

        fn sample_matrix() -> ConfusionMatrix {
            // TP=90, FP=10, FN=5, TN=895
            ConfusionMatrix::new(90, 10, 5, 895)
        }

        #[test]
        fn sensitivity() {
            let se = sample_matrix().sensitivity();
            assert!(se.is_some());
            let se = se.unwrap_or_else(|| unreachable!());
            // 90 / (90+5) = 0.9474
            assert!((se.value() - 90.0 / 95.0).abs() < 0.001);
        }

        #[test]
        fn specificity() {
            let sp = sample_matrix().specificity();
            assert!(sp.is_some());
            let sp = sp.unwrap_or_else(|| unreachable!());
            // 895 / (895+10) = 0.9890
            assert!((sp.value() - 895.0 / 905.0).abs() < 0.001);
        }

        #[test]
        fn ppv() {
            let ppv = sample_matrix().ppv();
            assert!(ppv.is_some());
            let ppv = ppv.unwrap_or_else(|| unreachable!());
            // 90 / (90+10) = 0.90
            assert!((ppv.value() - 0.90).abs() < f64::EPSILON);
        }

        #[test]
        fn npv() {
            let npv = sample_matrix().npv();
            assert!(npv.is_some());
            let npv = npv.unwrap_or_else(|| unreachable!());
            // 895 / (895+5) = 0.99444
            assert!((npv.value() - 895.0 / 900.0).abs() < 0.001);
        }

        #[test]
        fn all_metrics() {
            let metrics = sample_matrix().all_metrics();
            assert!(metrics.is_some());
        }

        #[test]
        fn ppv_from_bayes() {
            // Se=0.95, Sp=0.99, Prev=0.01
            let ppv = PositivePredictiveValue::from_bayes(0.95, 0.99, 0.01);
            assert!(ppv.is_some());
            let ppv = ppv.unwrap_or_else(|| unreachable!());
            // PPV = (0.95*0.01) / (0.95*0.01 + 0.01*0.99) = 0.0095 / 0.0194 ≈ 0.49
            assert!(ppv.value() > 0.45 && ppv.value() < 0.55);
        }

        #[test]
        fn npv_from_bayes() {
            let npv = NegativePredictiveValue::from_bayes(0.95, 0.99, 0.01);
            assert!(npv.is_some());
            let npv = npv.unwrap_or_else(|| unreachable!());
            // Very high NPV when prevalence is low
            assert!(npv.value() > 0.99);
        }
    }

    mod grounding_tests {
        use super::*;

        #[test]
        fn all_types_grounded() {
            let _ir = IncidenceRate::primitive_composition();
            let _prev = Prevalence::primitive_composition();
            let _ar = AttributableRisk::primitive_composition();
            let _paf = PopulationAttributableFraction::primitive_composition();
            let _drc = DoseResponseCurve::primitive_composition();
            let _snr = SignalToNoiseRatio::primitive_composition();
            let _se = Sensitivity::primitive_composition();
            let _sp = Specificity::primitive_composition();
            let _ppv = PositivePredictiveValue::primitive_composition();
            let _npv = NegativePredictiveValue::primitive_composition();
        }
    }
}
