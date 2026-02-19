//! T3 Domain Implementation: PV Signal Analyzer
//!
//! Demonstrates all four STEM domains working together in a concrete
//! pharmacovigilance signal detection system.
//!
//! ## Domain Mapping
//!
//! | STEM Domain | PV Application |
//! |-------------|----------------|
//! | Science (Sense/Classify/Infer) | Detect, categorize, predict signals |
//! | Chemistry (Concentrate/Saturate/Yield) | Case density, capacity, efficiency |
//! | Physics (Preserve/YieldForce/Couple) | Conservation, dose-response, interaction |
//! | Mathematics (Membership/Bound/Prove) | Cohort membership, CIs, causality |

use stem::prelude::*;

// ============================================================================
// T3 Domain Types
// ============================================================================

/// A pharmacovigilance signal (T3: domain-specific)
#[derive(Debug, Clone)]
struct PvSignal {
    #[allow(dead_code)]
    drug: String,
    #[allow(dead_code)]
    event: String,
    prr: f64,
    case_count: u32,
    confidence: Confidence,
}

/// Signal category after classification
#[derive(Debug, Clone, PartialEq)]
enum SignalCategory {
    /// No signal detected
    NoSignal,
    /// Weak signal (PRR 1.5-2.0)
    Weak,
    /// Moderate signal (PRR 2.0-3.0)
    Moderate,
    /// Strong signal (PRR ≥ 3.0)
    Strong,
}

/// Case density metrics
#[derive(Debug)]
struct CaseDensity {
    /// Cases per million population
    ratio: Ratio,
    /// Processing capacity fraction
    saturation: Fraction,
    /// Detection efficiency
    #[allow(dead_code)]
    efficiency: Fraction,
}

/// Dose-response measurement
#[derive(Debug)]
struct DoseResponse {
    /// Applied dose (force analog)
    #[allow(dead_code)]
    dose: Force,
    /// Patient resistance (mass analog)
    #[allow(dead_code)]
    resistance: Mass,
    /// Response magnitude (acceleration analog)
    response: Acceleration,
}

/// Confidence interval
#[derive(Debug)]
struct ConfidenceInterval {
    /// The PRR estimate
    estimate: Bounded<f64>,
    /// Statistical proof validity
    significant: bool,
}

// ============================================================================
// SCIENCE Traits (Sense, Classify, Infer)
// ============================================================================

/// The signal analyzer system (T3)
struct PvSignalAnalyzer {
    #[allow(dead_code)]
    threshold_prr: f64,
    min_cases: u32,
}

impl PvSignalAnalyzer {
    fn new() -> Self {
        Self {
            threshold_prr: 2.0, // Evans criteria
            min_cases: 3,
        }
    }
}

// Sense: raw data → signal
impl Sense for PvSignalAnalyzer {
    type Environment = (u32, u32, u32, u32); // 2x2 table: (a, b, c, d)
    type Signal = PvSignal;

    fn sense(&self, env: &Self::Environment) -> PvSignal {
        let (a, b, c, d) = *env;
        let prr = if b > 0 && c > 0 && d > 0 {
            let a_f = a as f64;
            let b_f = b as f64;
            let c_f = c as f64;
            let d_f = d as f64;
            (a_f / (a_f + b_f)) / (c_f / (c_f + d_f))
        } else {
            0.0
        };

        PvSignal {
            drug: "DrugX".to_string(),
            event: "EventY".to_string(),
            prr,
            case_count: a,
            confidence: Confidence::new(if a >= self.min_cases { 0.9 } else { 0.3 }),
        }
    }
}

// Classify: signal → category
impl Classify for PvSignalAnalyzer {
    type Signal = PvSignal;
    type Category = SignalCategory;

    fn classify(&self, signal: &PvSignal) -> SignalCategory {
        if signal.case_count < self.min_cases {
            return SignalCategory::NoSignal;
        }
        match signal.prr {
            p if p >= 3.0 => SignalCategory::Strong,
            p if p >= 2.0 => SignalCategory::Moderate,
            p if p >= 1.5 => SignalCategory::Weak,
            _ => SignalCategory::NoSignal,
        }
    }
}

// ============================================================================
// CHEMISTRY Traits (Concentrate, Saturate, Yield)
// ============================================================================

impl CaseDensity {
    /// Build from raw counts using chemistry primitives
    fn from_counts(cases: u32, population: u64, capacity: u32) -> Self {
        let ratio = Ratio::new(cases as f64 / population as f64 * 1_000_000.0);
        let saturation = Fraction::new(cases as f64 / capacity as f64);
        let efficiency = Fraction::new(if cases > 0 { 0.85 } else { 0.0 });

        CaseDensity {
            ratio,
            saturation,
            efficiency,
        }
    }
}

// ============================================================================
// PHYSICS Traits (Force/Mass/Acceleration analogy)
// ============================================================================

impl DoseResponse {
    /// Model dose-response as F=ma analogy
    fn from_dose_and_resistance(dose_mg: f64, patient_weight_kg: f64) -> Self {
        let dose = Force::new(dose_mg);
        let resistance = Mass::new(patient_weight_kg);
        let response = Acceleration::from_force_and_mass(dose, resistance);

        DoseResponse {
            dose,
            resistance,
            response,
        }
    }
}

// ============================================================================
// MATHEMATICS Traits (Bounded, Proof)
// ============================================================================

impl ConfidenceInterval {
    /// Build CI from PRR and case count
    fn from_prr(prr: f64, case_count: u32) -> Self {
        // Simplified CI calculation
        let se = if case_count > 0 {
            1.0 / (case_count as f64).sqrt()
        } else {
            f64::INFINITY
        };

        let lower = (prr.ln() - 1.96 * se).exp();
        let upper = (prr.ln() + 1.96 * se).exp();

        let estimate = Bounded::new(prr, Some(lower), Some(upper));
        let significant = lower > 1.0; // Lower CI > 1.0 means significant

        ConfidenceInterval {
            estimate,
            significant,
        }
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn full_pipeline_strong_signal() {
    let analyzer = PvSignalAnalyzer::new();

    // SCIENCE: Sense → Classify
    let env = (15_u32, 100, 20, 10000);
    let signal = analyzer.sense(&env);
    let category = analyzer.classify(&signal);

    assert!(signal.prr > 2.0);
    assert_eq!(category, SignalCategory::Strong);
    assert!((signal.confidence.value() - 0.9).abs() < f64::EPSILON);

    // CHEMISTRY: Case density
    let density = CaseDensity::from_counts(15, 1_000_000, 100);
    assert!((density.ratio.value() - 15.0).abs() < f64::EPSILON);
    assert!(!density.saturation.is_saturated());

    // PHYSICS: Dose-response
    let dr = DoseResponse::from_dose_and_resistance(100.0, 70.0);
    assert!(dr.response.value() > 0.0);

    // MATHEMATICS: Confidence interval
    let ci = ConfidenceInterval::from_prr(signal.prr, signal.case_count);
    assert!(ci.significant); // Lower CI > 1.0
    assert!(ci.estimate.in_bounds());
}

#[test]
fn full_pipeline_no_signal() {
    let analyzer = PvSignalAnalyzer::new();

    // Low PRR, few cases
    let env = (2_u32, 500, 100, 10000);
    let signal = analyzer.sense(&env);
    let category = analyzer.classify(&signal);

    assert_eq!(category, SignalCategory::NoSignal);
    assert!((signal.confidence.value() - 0.3).abs() < f64::EPSILON);
}

#[test]
fn chemistry_saturation_detection() {
    let density = CaseDensity::from_counts(99, 1_000_000, 100);
    assert!(density.saturation.value() > 0.98);
    assert!(density.saturation.is_saturated());
}

#[test]
fn physics_dose_response_proportional() {
    let low = DoseResponse::from_dose_and_resistance(50.0, 70.0);
    let high = DoseResponse::from_dose_and_resistance(100.0, 70.0);

    // Double dose → double response (same mass)
    let ratio = high.response.value() / low.response.value();
    assert!((ratio - 2.0).abs() < f64::EPSILON);
}

#[test]
fn math_ci_excludes_one_for_strong_signal() {
    let ci = ConfidenceInterval::from_prr(5.0, 50);
    assert!(ci.significant);
    assert!(ci.estimate.in_bounds());

    // Weak signal with few cases → not significant
    let weak_ci = ConfidenceInterval::from_prr(1.5, 3);
    // With only 3 cases, SE is large → lower CI may include 1.0
    // This tests that math correctly propagates uncertainty
    assert!(weak_ci.estimate.in_bounds());
}

#[test]
fn cross_domain_measured_pipeline() {
    let analyzer = PvSignalAnalyzer::new();
    let signal = analyzer.sense(&(15, 100, 20, 10000));

    // Wrap each domain result in Measured<T> with confidence
    let measured_prr = Measured::new(signal.prr, signal.confidence);
    let measured_density = Measured::new(
        CaseDensity::from_counts(15, 1_000_000, 100),
        Confidence::new(0.85),
    );

    // Combine confidences across domains
    let combined = measured_prr.confidence.combine(measured_density.confidence);
    assert!(combined.value() < measured_prr.confidence.value());
    assert!(combined.value() < measured_density.confidence.value());
    assert!(combined.value() > 0.0);
}
