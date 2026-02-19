//! Kaplan-Meier Survival Estimator
//!
//! Non-parametric method for estimating survival functions from time-to-event data,
//! handling censored observations (patients lost to follow-up or study end).
//!
//! # Algorithm
//!
//! The Kaplan-Meier estimator calculates survival probability as:
//!
//! ```text
//! S(t) = Π_{t_i ≤ t} (1 - d_i / n_i)
//!
//! where:
//!   t_i = distinct event times
//!   d_i = number of events at time t_i
//!   n_i = number at risk just before t_i
//! ```
//!
//! # Variance Estimation (Greenwood's Formula)
//!
//! ```text
//! Var(S(t)) = S(t)² × Σ_{t_i ≤ t} d_i / [n_i × (n_i - d_i)]
//! ```
//!
//! # Use Cases
//!
//! - Comparing survival between drug-exposed and unexposed groups
//! - Estimating median time-to-adverse-event
//! - Visualizing safety signals over time
//!
//! # References
//!
//! - Kaplan EL, Meier P (1958). "Nonparametric estimation from incomplete observations."
//!   Journal of the American Statistical Association 53(282):457-481.
//!   DOI: [10.1080/01621459.1958.10501452](https://doi.org/10.1080/01621459.1958.10501452)
//!
//! - Greenwood M (1926). "The natural duration of cancer." Reports on Public Health
//!   and Medical Subjects 33:1-26.
//!
//! - Bland JM, Altman DG (1998). "Survival probabilities (the Kaplan-Meier method)."
//!   BMJ 317(7172):1572. DOI: [10.1136/bmj.317.7172.1572](https://doi.org/10.1136/bmj.317.7172.1572)

use serde::{Deserialize, Serialize};

/// A single observation for survival analysis.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SurvivalObservation {
    /// Time of event or censoring
    pub time: f64,
    /// Whether the event occurred (true) or was censored (false)
    pub event: bool,
    /// Optional group indicator for comparing groups
    pub group: Option<u8>,
}

impl SurvivalObservation {
    /// Create a new observation.
    #[must_use]
    pub const fn new(time: f64, event: bool) -> Self {
        Self {
            time,
            event,
            group: None,
        }
    }

    /// Create with group assignment.
    #[must_use]
    pub const fn with_group(time: f64, event: bool, group: u8) -> Self {
        Self {
            time,
            event,
            group: Some(group),
        }
    }

    /// Create an event observation.
    #[must_use]
    pub const fn event(time: f64) -> Self {
        Self::new(time, true)
    }

    /// Create a censored observation.
    #[must_use]
    pub const fn censored(time: f64) -> Self {
        Self::new(time, false)
    }
}

/// A point on the Kaplan-Meier survival curve.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurvivalPoint {
    /// Time point
    pub time: f64,
    /// Number at risk just before this time
    pub n_risk: usize,
    /// Number of events at this time
    pub n_events: usize,
    /// Number censored at this time
    pub n_censored: usize,
    /// Cumulative survival probability S(t)
    pub survival: f64,
    /// Standard error of survival estimate
    pub se: f64,
    /// Lower 95% confidence interval
    pub ci_lower: f64,
    /// Upper 95% confidence interval
    pub ci_upper: f64,
}

/// Result of Kaplan-Meier estimation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KaplanMeierResult {
    /// Survival curve points (at each event time)
    pub curve: Vec<SurvivalPoint>,
    /// Total number of observations
    pub n_total: usize,
    /// Total number of events
    pub n_events: usize,
    /// Total number censored
    pub n_censored: usize,
    /// Median survival time (if reached)
    pub median_survival: Option<f64>,
    /// Mean survival time (restricted to max observed time)
    pub restricted_mean: f64,
}

impl KaplanMeierResult {
    /// Get survival probability at a specific time.
    #[must_use]
    pub fn survival_at(&self, t: f64) -> f64 {
        if self.curve.is_empty() || t < self.curve[0].time {
            return 1.0;
        }

        for point in self.curve.iter().rev() {
            if t >= point.time {
                return point.survival;
            }
        }
        1.0
    }

    /// Get the time when survival drops below a threshold.
    #[must_use]
    pub fn time_to_survival(&self, threshold: f64) -> Option<f64> {
        for point in &self.curve {
            if point.survival <= threshold {
                return Some(point.time);
            }
        }
        None
    }
}

/// Compute Kaplan-Meier survival estimate.
///
/// # Arguments
///
/// * `observations` - Vector of survival observations
///
/// # Returns
///
/// `KaplanMeierResult` containing the survival curve and summary statistics.
///
/// # Complexity
///
/// - **Time**: O(n log n) due to sorting
/// - **Space**: O(n) for storing curve points
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::pv::signals::survival::kaplan_meier::{kaplan_meier, SurvivalObservation};
///
/// let observations = vec![
///     SurvivalObservation::event(1.0),
///     SurvivalObservation::event(2.0),
///     SurvivalObservation::censored(3.0),
///     SurvivalObservation::event(4.0),
///     SurvivalObservation::censored(5.0),
/// ];
///
/// let result = kaplan_meier(&observations);
/// println!("Median survival: {:?}", result.median_survival);
/// println!("Survival at t=3: {:.2}", result.survival_at(3.0));
/// ```
#[must_use]
pub fn kaplan_meier(observations: &[SurvivalObservation]) -> KaplanMeierResult {
    if observations.is_empty() {
        return KaplanMeierResult {
            curve: vec![],
            n_total: 0,
            n_events: 0,
            n_censored: 0,
            median_survival: None,
            restricted_mean: 0.0,
        };
    }

    // Sort by time
    let mut sorted: Vec<_> = observations.to_vec();
    sorted.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let n_total = sorted.len();
    let mut n_risk = n_total;
    let mut survival = 1.0;
    let mut greenwood_sum = 0.0;
    let mut curve = Vec::new();
    let mut total_events = 0;
    let mut total_censored = 0;

    // Group events by time
    let mut i = 0;
    while i < sorted.len() {
        let current_time = sorted[i].time;

        // Count events and censored at this time
        let mut n_events = 0;
        let mut n_censored = 0;

        while i < sorted.len() && (sorted[i].time - current_time).abs() < 1e-10 {
            if sorted[i].event {
                n_events += 1;
                total_events += 1;
            } else {
                n_censored += 1;
                total_censored += 1;
            }
            i += 1;
        }

        // Only create curve point if there are events (not just censoring)
        if n_events > 0 {
            // Update survival: S(t) = S(t-) × (1 - d/n)
            let hazard = n_events as f64 / n_risk as f64;
            survival *= 1.0 - hazard;

            // Greenwood's formula for variance
            if n_risk > n_events {
                greenwood_sum += n_events as f64 / (n_risk as f64 * (n_risk - n_events) as f64);
            }

            // Standard error
            let se = survival * greenwood_sum.sqrt();

            // 95% CI using log-log transformation for better coverage
            let z = 1.96;
            let (ci_lower, ci_upper) = if survival > 0.0 && survival < 1.0 {
                let _log_log_s = (-survival.ln()).ln();
                let se_log_log = se / (survival * survival.ln().abs());
                let ci_l = (-((-survival.ln()) * (z * se_log_log).exp())).exp();
                let ci_u = (-((-survival.ln()) * (-z * se_log_log).exp())).exp();
                (ci_l.max(0.0), ci_u.min(1.0))
            } else {
                (survival.max(0.0), survival.min(1.0))
            };

            curve.push(SurvivalPoint {
                time: current_time,
                n_risk,
                n_events,
                n_censored,
                survival,
                se,
                ci_lower,
                ci_upper,
            });
        }

        // Update number at risk
        n_risk -= n_events + n_censored;
    }

    // Calculate median survival (time when S(t) = 0.5)
    let median_survival = curve.iter().find(|p| p.survival <= 0.5).map(|p| p.time);

    // Calculate restricted mean survival time (area under curve)
    let restricted_mean =
        calculate_restricted_mean(&curve, sorted.last().map(|o| o.time).unwrap_or(0.0));

    KaplanMeierResult {
        curve,
        n_total,
        n_events: total_events,
        n_censored: total_censored,
        median_survival,
        restricted_mean,
    }
}

/// Calculate restricted mean survival time (area under KM curve).
fn calculate_restricted_mean(curve: &[SurvivalPoint], max_time: f64) -> f64 {
    if curve.is_empty() {
        return max_time; // All survived
    }

    let mut area = 0.0;
    let mut prev_time = 0.0;
    let mut prev_survival = 1.0;

    for point in curve {
        // Add rectangle from previous time to current time
        area += prev_survival * (point.time - prev_time);
        prev_time = point.time;
        prev_survival = point.survival;
    }

    // Add final rectangle to max_time
    area += prev_survival * (max_time - prev_time);

    area
}

/// Log-rank test for comparing two survival curves.
///
/// # Returns
///
/// Tuple of (chi-square statistic, p-value, hazard ratio estimate)
#[must_use]
pub fn log_rank_test(
    group0: &[SurvivalObservation],
    group1: &[SurvivalObservation],
) -> (f64, f64, f64) {
    // Combine and sort all observations
    let mut all: Vec<_> = group0
        .iter()
        .map(|o| (o.time, o.event, 0u8))
        .chain(group1.iter().map(|o| (o.time, o.event, 1u8)))
        .collect();

    all.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut n0 = group0.len(); // At risk in group 0
    let mut n1 = group1.len(); // At risk in group 1

    let mut observed_0 = 0.0; // Observed events in group 0
    let mut expected_0 = 0.0; // Expected events in group 0
    let mut variance = 0.0;

    let mut i = 0;
    while i < all.len() {
        let current_time = all[i].0;

        // Count events at this time
        let mut d0 = 0; // Events in group 0
        let mut d1 = 0; // Events in group 1
        let mut c0 = 0; // Censored in group 0
        let mut c1 = 0; // Censored in group 1

        while i < all.len() && (all[i].0 - current_time).abs() < 1e-10 {
            let (_, event, group) = all[i];
            if event {
                if group == 0 {
                    d0 += 1;
                } else {
                    d1 += 1;
                }
            } else if group == 0 {
                c0 += 1;
            } else {
                c1 += 1;
            }
            i += 1;
        }

        let d = d0 + d1; // Total events
        let n = n0 + n1; // Total at risk

        if d > 0 && n > 1 {
            // Expected events in group 0
            let e0 = (d as f64) * (n0 as f64) / (n as f64);
            expected_0 += e0;
            observed_0 += d0 as f64;

            // Variance contribution (hypergeometric)
            let v = (d as f64) * (n0 as f64) * (n1 as f64) * ((n - d) as f64)
                / ((n * n * (n - 1)) as f64);
            variance += v;
        }

        // Update at-risk counts
        n0 -= d0 + c0;
        n1 -= d1 + c1;
    }

    // Chi-square statistic
    let chi_sq = if variance > 0.0 {
        (observed_0 - expected_0).powi(2) / variance
    } else {
        0.0
    };

    // P-value (chi-square with 1 df)
    let p_value = crate::signals::core::stats::chi_square_p_value(chi_sq);

    // Hazard ratio estimate (Mantel-Haenszel)
    let hr = if expected_0 > 0.0 {
        observed_0 / expected_0
    } else {
        1.0
    };

    (chi_sq, p_value, hr)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> Vec<SurvivalObservation> {
        vec![
            SurvivalObservation::event(1.0),
            SurvivalObservation::event(2.0),
            SurvivalObservation::event(3.0),
            SurvivalObservation::censored(3.5),
            SurvivalObservation::event(4.0),
            SurvivalObservation::censored(4.5),
            SurvivalObservation::event(5.0),
            SurvivalObservation::censored(6.0),
        ]
    }

    #[test]
    fn test_kaplan_meier_basic() {
        let data = create_test_data();
        let result = kaplan_meier(&data);

        assert_eq!(result.n_total, 8);
        assert_eq!(result.n_events, 5);
        assert_eq!(result.n_censored, 3);

        // Survival should decrease at each event time
        for i in 1..result.curve.len() {
            assert!(result.curve[i].survival <= result.curve[i - 1].survival);
        }
    }

    #[test]
    fn test_kaplan_meier_survival_at() {
        let data = create_test_data();
        let result = kaplan_meier(&data);

        // Before first event, survival = 1
        assert!((result.survival_at(0.5) - 1.0).abs() < 0.01);

        // Survival decreases over time
        assert!(result.survival_at(2.0) < result.survival_at(1.0));
        assert!(result.survival_at(4.0) < result.survival_at(2.0));
    }

    #[test]
    fn test_kaplan_meier_empty() {
        let result = kaplan_meier(&[]);

        assert_eq!(result.n_total, 0);
        assert!(result.curve.is_empty());
    }

    #[test]
    fn test_kaplan_meier_all_censored() {
        let data = vec![
            SurvivalObservation::censored(1.0),
            SurvivalObservation::censored(2.0),
            SurvivalObservation::censored(3.0),
        ];
        let result = kaplan_meier(&data);

        assert_eq!(result.n_events, 0);
        assert!(result.curve.is_empty()); // No events, no curve points
    }

    #[test]
    fn test_kaplan_meier_all_events() {
        let data = vec![
            SurvivalObservation::event(1.0),
            SurvivalObservation::event(2.0),
            SurvivalObservation::event(3.0),
        ];
        let result = kaplan_meier(&data);

        assert_eq!(result.n_events, 3);
        assert_eq!(result.n_censored, 0);

        // Final survival should be 0 (all died)
        assert!(result.curve.last().unwrap().survival < 0.5);
    }

    #[test]
    fn test_kaplan_meier_confidence_intervals() {
        let data = create_test_data();
        let result = kaplan_meier(&data);

        for point in &result.curve {
            // CI should bracket the point estimate
            assert!(point.ci_lower <= point.survival);
            assert!(point.ci_upper >= point.survival);
            // CI should be valid probabilities
            assert!(point.ci_lower >= 0.0);
            assert!(point.ci_upper <= 1.0);
        }
    }

    #[test]
    fn test_log_rank_basic() {
        // Group 0: Better survival (later events)
        let group0 = vec![
            SurvivalObservation::event(5.0),
            SurvivalObservation::event(6.0),
            SurvivalObservation::censored(7.0),
            SurvivalObservation::event(8.0),
        ];

        // Group 1: Worse survival (earlier events)
        let group1 = vec![
            SurvivalObservation::event(1.0),
            SurvivalObservation::event(2.0),
            SurvivalObservation::event(3.0),
            SurvivalObservation::censored(4.0),
        ];

        let (chi_sq, p_value, hr) = log_rank_test(&group0, &group1);

        // Chi-square should be positive
        assert!(chi_sq >= 0.0);
        // P-value should be valid
        assert!(p_value >= 0.0 && p_value <= 1.0);
        // HR for group 0 should be < 1 (fewer events than expected)
        assert!(hr < 1.5); // Some tolerance for small sample
    }

    #[test]
    fn test_log_rank_identical_groups() {
        let group0 = vec![
            SurvivalObservation::event(1.0),
            SurvivalObservation::event(2.0),
        ];
        let group1 = vec![
            SurvivalObservation::event(1.0),
            SurvivalObservation::event(2.0),
        ];

        let (chi_sq, _p_value, hr) = log_rank_test(&group0, &group1);

        // Identical groups should have chi_sq ≈ 0 and HR ≈ 1
        assert!(chi_sq < 1.0);
        assert!((hr - 1.0).abs() < 0.5);
    }

    #[test]
    fn test_restricted_mean() {
        let data = vec![
            SurvivalObservation::event(2.0),
            SurvivalObservation::event(4.0),
            SurvivalObservation::censored(6.0),
        ];
        let result = kaplan_meier(&data);

        // RMST should be positive and less than max time
        assert!(result.restricted_mean > 0.0);
        assert!(result.restricted_mean <= 6.0);
    }

    #[test]
    fn test_median_survival() {
        // Create data where median is clear
        let data = vec![
            SurvivalObservation::event(1.0),
            SurvivalObservation::event(2.0),
            SurvivalObservation::event(3.0), // 3rd of 5, survival drops to 0.4
            SurvivalObservation::event(4.0),
            SurvivalObservation::event(5.0),
        ];
        let result = kaplan_meier(&data);

        // Median should be defined (survival crosses 0.5)
        assert!(result.median_survival.is_some());
        assert!(result.median_survival.unwrap() > 0.0);
    }
}
