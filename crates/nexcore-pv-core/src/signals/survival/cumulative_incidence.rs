//! Cumulative Incidence (1 - Kaplan-Meier)
//!
//! Complementary perspective on survival data: rather than estimating the probability
//! of survival, estimates the probability that the event has occurred by time t.
//!
//! # Use Cases
//!
//! - Adverse event time-to-onset characterization (PV perspective)
//! - Clinical trialists think in survival probability; safety scientists think in cumulative incidence
//! - The Observatory epidemiology explorer renders both perspectives
//!
//! # Relationship to Kaplan-Meier
//!
//! ```text
//! CI(t) = 1 - S(t)
//! SE_CI(t) = SE_KM(t)  (complement preserves SE)
//! CI_lower_cum = 1 - CI_upper_km  (bounds flip)
//! CI_upper_cum = 1 - CI_lower_km  (bounds flip)
//! ```

use nexcore_constants::{Confidence, Measured};
use serde::{Deserialize, Serialize};

use super::kaplan_meier::{SurvivalObservation, SurvivalPoint};

/// A single point on the cumulative incidence curve.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CumulativeIncidencePoint {
    /// Time point
    pub time: f64,
    /// Cumulative incidence: 1.0 - S(t)
    pub incidence: f64,
    /// Standard error (same as KM SE — complement preserves SE)
    pub se: f64,
    /// CI lower bound: 1.0 - km_ci_upper (bounds flip)
    pub ci_lower: f64,
    /// CI upper bound: 1.0 - km_ci_lower (bounds flip)
    pub ci_upper: f64,
    /// Number at risk
    pub n_risk: usize,
    /// Events at this time
    pub n_events: usize,
    /// Censored at this time
    pub n_censored: usize,
}

/// Complete cumulative incidence result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CumulativeIncidenceResult {
    /// Points on the cumulative incidence curve
    pub points: Vec<CumulativeIncidencePoint>,
    /// Final cumulative incidence (at last observed time)
    pub total_incidence: f64,
    /// Total number of events
    pub event_count: usize,
    /// Total number of censored observations
    pub censored_count: usize,
    /// Total observations
    pub n_total: usize,
}

/// Cumulative incidence with Measured<T> confidence at each point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasuredCumulativeIncidence {
    /// Raw result (unchanged)
    pub raw: CumulativeIncidenceResult,
    /// Each incidence point with confidence
    pub measured_points: Vec<Measured<f64>>,
    /// Overall confidence
    pub overall_confidence: Confidence,
}

/// Convert a KM SurvivalPoint to a CumulativeIncidencePoint.
///
/// Key: CI bounds FLIP when complementing.
fn km_to_ci(pt: &SurvivalPoint) -> CumulativeIncidencePoint {
    CumulativeIncidencePoint {
        time: pt.time,
        incidence: 1.0 - pt.survival,
        se: pt.se, // Complement preserves SE
        // CRITICAL: Bounds flip when complementing
        ci_lower: 1.0 - pt.ci_upper,
        ci_upper: 1.0 - pt.ci_lower,
        n_risk: pt.n_risk,
        n_events: pt.n_events,
        n_censored: pt.n_censored,
    }
}

/// Compute cumulative incidence from time-to-event observations.
///
/// Internally computes Kaplan-Meier, then complements.
/// Infallible: KM estimator always produces a result.
pub fn cumulative_incidence(observations: &[SurvivalObservation]) -> CumulativeIncidenceResult {
    let km = super::kaplan_meier::kaplan_meier(observations);

    let points: Vec<CumulativeIncidencePoint> = km.curve.iter().map(km_to_ci).collect();

    let total_incidence = points.last().map_or(0.0, |p| p.incidence);

    CumulativeIncidenceResult {
        points,
        total_incidence,
        event_count: km.n_events,
        censored_count: km.n_censored,
        n_total: km.n_total,
    }
}

/// Compute cumulative incidence with Measured<T> confidence.
///
/// Confidence mapping inherits from KM Greenwood SE (same data, complementary probability).
/// Infallible: wraps infallible KM estimator.
pub fn cumulative_incidence_measured(
    observations: &[SurvivalObservation],
) -> MeasuredCumulativeIncidence {
    let km = super::kaplan_meier::kaplan_meier(observations);

    let points: Vec<CumulativeIncidencePoint> = km.curve.iter().map(km_to_ci).collect();
    let total_incidence = points.last().map_or(0.0, |p| p.incidence);

    let measured_points: Vec<Measured<f64>> = km
        .curve
        .iter()
        .zip(points.iter())
        .map(|(km_pt, ci_pt)| {
            // CALIBRATION: Inherits KM confidence mapping (same SE)
            let conf = Confidence::new((1.0_f64 - 2.0 * km_pt.se).clamp(0.05, 0.99));
            Measured::new(ci_pt.incidence, conf)
        })
        .collect();

    let overall_confidence = measured_points
        .iter()
        .map(|m| m.confidence)
        .min_by(|a, b| a.cmp_total(*b))
        .unwrap_or(Confidence::UNCERTAIN);

    let raw = CumulativeIncidenceResult {
        points,
        total_incidence,
        event_count: km.n_events,
        censored_count: km.n_censored,
        n_total: km.n_total,
    };

    MeasuredCumulativeIncidence {
        raw,
        measured_points,
        overall_confidence,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> Vec<SurvivalObservation> {
        vec![
            SurvivalObservation::event(1.0),
            SurvivalObservation::event(2.0),
            SurvivalObservation::censored(3.0),
            SurvivalObservation::event(3.5),
            SurvivalObservation::event(4.0),
            SurvivalObservation::censored(4.5),
            SurvivalObservation::event(5.0),
            SurvivalObservation::censored(6.0),
        ]
    }

    #[test]
    fn test_complementarity() {
        let obs = sample_data();
        let km = super::super::kaplan_meier::kaplan_meier(&obs);
        let ci = cumulative_incidence(&obs);

        assert_eq!(km.curve.len(), ci.points.len());

        for (km_pt, ci_pt) in km.curve.iter().zip(ci.points.iter()) {
            let complement = 1.0 - km_pt.survival;
            assert!(
                (ci_pt.incidence - complement).abs() < 1e-10,
                "CI({}) = {:.6} != 1.0 - S({}) = {:.6}",
                ci_pt.time,
                ci_pt.incidence,
                km_pt.time,
                complement
            );
        }
    }

    #[test]
    fn test_ci_bounds_flip() {
        let obs = sample_data();
        let km = super::super::kaplan_meier::kaplan_meier(&obs);
        let ci = cumulative_incidence(&obs);

        for (km_pt, ci_pt) in km.curve.iter().zip(ci.points.iter()) {
            assert!(
                (ci_pt.ci_lower - (1.0 - km_pt.ci_upper)).abs() < 1e-10,
                "CI lower flip failed at t={}: ci_lower={:.6} != 1-km_ci_upper={:.6}",
                ci_pt.time,
                ci_pt.ci_lower,
                1.0 - km_pt.ci_upper
            );
            assert!(
                (ci_pt.ci_upper - (1.0 - km_pt.ci_lower)).abs() < 1e-10,
                "CI upper flip failed at t={}: ci_upper={:.6} != 1-km_ci_lower={:.6}",
                ci_pt.time,
                ci_pt.ci_upper,
                1.0 - km_pt.ci_lower
            );
        }
    }

    #[test]
    fn test_ci_se_matches_km() {
        let obs = sample_data();
        let km = super::super::kaplan_meier::kaplan_meier(&obs);
        let ci = cumulative_incidence(&obs);

        for (km_pt, ci_pt) in km.curve.iter().zip(ci.points.iter()) {
            assert!(
                (ci_pt.se - km_pt.se).abs() < 1e-10,
                "SE mismatch at t={}: ci_se={:.6} != km_se={:.6}",
                ci_pt.time,
                ci_pt.se,
                km_pt.se
            );
        }
    }

    #[test]
    fn test_all_censored_ci_zero() {
        let obs = vec![
            SurvivalObservation::censored(1.0),
            SurvivalObservation::censored(2.0),
            SurvivalObservation::censored(3.0),
        ];
        let ci = cumulative_incidence(&obs);
        assert!(ci.points.is_empty(), "All censored → no curve points");
        assert!(
            (ci.total_incidence - 0.0).abs() < 1e-10,
            "All censored → incidence stays 0"
        );
    }

    #[test]
    fn test_all_events_ci_approaches_one() {
        let obs = vec![
            SurvivalObservation::event(1.0),
            SurvivalObservation::event(2.0),
            SurvivalObservation::event(3.0),
        ];
        let ci = cumulative_incidence(&obs);
        assert!(
            ci.total_incidence > 0.5,
            "All events → CI approaches 1.0: {:.4}",
            ci.total_incidence
        );
    }

    #[test]
    fn test_measured_ci_confidence_matches_km() {
        let obs = sample_data();
        let km_measured = super::super::measured::kaplan_meier_measured(&obs);
        let ci_measured = cumulative_incidence_measured(&obs);

        assert_eq!(
            km_measured.measured_curve.len(),
            ci_measured.measured_points.len()
        );

        for (km_m, ci_m) in km_measured
            .measured_curve
            .iter()
            .zip(ci_measured.measured_points.iter())
        {
            assert!(
                (km_m.confidence.value() - ci_m.confidence.value()).abs() < 1e-10,
                "Measured confidence should match: km={:.4} != ci={:.4}",
                km_m.confidence.value(),
                ci_m.confidence.value()
            );
        }
    }

    #[test]
    fn test_measured_ci_confidence_range() {
        let result = cumulative_incidence_measured(&sample_data());
        for m in &result.measured_points {
            assert!(
                m.confidence.value() >= 0.05 && m.confidence.value() <= 0.99,
                "Confidence {:.4} outside [0.05, 0.99]",
                m.confidence.value()
            );
        }
    }
}
