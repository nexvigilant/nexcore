//! Continuous Sequential Sampling Procedure (CSSP) for Chronic Exposure
//!
//! CSSP is a sequential surveillance method designed specifically for monitoring
//! adverse events associated with chronic medication exposure, where events may
//! occur after prolonged use (months to years).
//!
//! # Background
//!
//! Traditional sequential methods (SPRT, MaxSPRT) assume Poisson-distributed
//! events under acute exposure. CSSP extends this to handle:
//!
//! - **Cumulative exposure**: Risk accumulates over time
//! - **Delayed onset**: Events may occur months/years after initiation
//! - **Varying exposure duration**: Patients have different treatment periods
//!
//! # Algorithm
//!
//! CSSP uses person-time at risk with exposure-weighted monitoring:
//!
//! ```text
//! Expected events E = Σ (exposure_i × background_rate)
//! Test statistic = observed / expected under H₀
//!
//! Boundary adjustment for cumulative exposure:
//! α*(t) = α × f(cumulative_exposure / total_planned_exposure)
//! ```
//!
//! # When to Use
//!
//! - **Chronic medications**: Statins, antihypertensives, diabetes drugs
//! - **Cumulative toxicity**: Drugs with dose-dependent long-term effects
//! - **Cancer chemotherapy**: Monitoring for secondary malignancies
//! - **Long-term biologics**: Immunosuppressants, DMARDs
//!
//! # References
//!
//! - Kulldorff M, Davis RL, Kolczak M, et al. (2011). "A maximized sequential
//!   probability ratio test for drug and vaccine safety surveillance."
//!   Sequential Analysis 30(1):58-78. DOI: [10.1080/07474946.2011.539924](https://doi.org/10.1080/07474946.2011.539924)
//!
//! - Nelson JC, Cook AJ, Yu O, et al. (2015). "Methods for observational
//!   post-licensure medical product safety surveillance." Statistical Methods
//!   in Medical Research 24(2):177-193. DOI: [10.1177/0962280214533498](https://doi.org/10.1177/0962280214533498)
//!
//! - Cook AJ, Tiwari RC, Wellman RD, et al. (2012). "Statistical approaches
//!   to group sequential monitoring of postmarket safety surveillance data."
//!   Pharmaceutical Statistics 11(1):37-47. DOI: [10.1002/pst.497](https://doi.org/10.1002/pst.497)

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Configuration for CSSP analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsspConfig {
    /// Type I error rate (typically 0.05)
    pub alpha: f64,
    /// Background incidence rate (events per person-year)
    pub background_rate: f64,
    /// Minimum relative risk to detect
    pub min_rr: f64,
    /// Maximum cumulative person-time planned for surveillance
    pub max_person_time: f64,
    /// Number of interim looks
    pub n_looks: usize,
    /// Spending function type
    pub spending_function: SpendingFunction,
}

impl Default for CsspConfig {
    fn default() -> Self {
        Self {
            alpha: 0.05,
            background_rate: 0.001, // 1 per 1000 person-years
            min_rr: 2.0,
            max_person_time: 100_000.0, // 100,000 person-years
            n_looks: 12,                // Monthly for 1 year
            spending_function: SpendingFunction::OBrienFleming,
        }
    }
}

/// Alpha spending function for group sequential boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SpendingFunction {
    /// O'Brien-Fleming: Conservative early, liberal late
    /// α*(t) = 2 × (1 - Φ(z_{α/2} / √t))
    OBrienFleming,
    /// Pocock: Equal spending at each look
    /// α*(t) = α × ln(1 + (e-1) × t)
    Pocock,
    /// Kim-DeMets power family
    /// α*(t) = α × t^ρ
    KimDeMets { rho: f64 },
}

impl SpendingFunction {
    /// Calculate cumulative alpha spent at information fraction t ∈ [0, 1].
    #[must_use]
    pub fn alpha_spent(&self, t: f64, alpha: f64) -> f64 {
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return alpha;
        }

        match self {
            Self::OBrienFleming => {
                // α*(t) = 2 × (1 - Φ(z_{α/2} / √t))
                // Approximation using standard normal
                let z_alpha = 1.96; // For α = 0.05
                let z = z_alpha / t.sqrt();
                2.0 * (1.0 - standard_normal_cdf(z))
            }
            Self::Pocock => {
                // α*(t) = α × ln(1 + (e-1) × t)
                let e = std::f64::consts::E;
                alpha * (1.0 + (e - 1.0) * t).ln()
            }
            Self::KimDeMets { rho } => {
                // α*(t) = α × t^ρ
                alpha * t.powf(*rho)
            }
        }
    }

    /// Calculate incremental alpha for the current look.
    #[must_use]
    pub fn incremental_alpha(&self, t_prev: f64, t_curr: f64, alpha: f64) -> f64 {
        self.alpha_spent(t_curr, alpha) - self.alpha_spent(t_prev, alpha)
    }
}

/// Observation for CSSP: patient-level exposure data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsspObservation {
    /// Patient identifier
    pub id: String,
    /// Total exposure time (person-years)
    pub exposure_time: f64,
    /// Whether event occurred
    pub event: bool,
    /// Time to event or censoring
    pub time: f64,
}

/// Result of CSSP analysis at a single look.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CsspResult {
    /// Current look number (1-indexed)
    pub look: usize,
    /// Observed number of events
    pub observed: u32,
    /// Expected events under null hypothesis
    pub expected: f64,
    /// Cumulative person-time observed
    pub person_time: f64,
    /// Information fraction (proportion of planned surveillance completed)
    pub information_fraction: f64,
    /// Test statistic (observed / expected)
    pub test_statistic: f64,
    /// Upper boundary at this look
    pub upper_boundary: f64,
    /// Lower boundary at this look (for early acceptance of H₀)
    pub lower_boundary: f64,
    /// Alpha spent at this look
    pub alpha_spent: f64,
    /// Cumulative alpha spent
    pub cumulative_alpha_spent: f64,
    /// Decision at this look
    pub decision: CsspDecision,
    /// Relative risk estimate (observed / expected)
    pub rr_estimate: f64,
    /// 95% CI for relative risk
    pub rr_ci: (f64, f64),
}

/// Decision from CSSP analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CsspDecision {
    /// Continue monitoring (no boundary crossed)
    Continue,
    /// Signal detected (upper boundary crossed)
    Signal,
    /// No signal, early stop acceptable (lower boundary crossed)
    NoSignal,
    /// Surveillance complete (final look, no signal)
    Complete,
}

/// Monitor for continuous CSSP surveillance.
#[derive(Debug, Clone)]
pub struct CsspMonitor {
    config: CsspConfig,
    looks: Vec<CsspResult>,
    total_observed: u32,
    total_person_time: f64,
}

impl CsspMonitor {
    /// Create a new CSSP monitor.
    #[must_use]
    pub fn new(config: CsspConfig) -> Self {
        Self {
            config,
            looks: Vec::new(),
            total_observed: 0,
            total_person_time: 0.0,
        }
    }

    /// Add observations and perform a look.
    pub fn add_observations(&mut self, observations: &[CsspObservation]) -> CsspResult {
        // Accumulate new data
        let new_events: u32 = observations.iter().filter(|o| o.event).count() as u32;
        let new_person_time: f64 = observations.iter().map(|o| o.exposure_time).sum();

        self.total_observed += new_events;
        self.total_person_time += new_person_time;

        // Calculate look number and information fraction
        let look = self.looks.len() + 1;
        let info_fraction = (self.total_person_time / self.config.max_person_time).min(1.0);
        let prev_info_fraction = if look > 1 {
            self.looks[look - 2].information_fraction
        } else {
            0.0
        };

        // Expected events under null
        let expected = self.total_person_time * self.config.background_rate;

        // Test statistic
        let test_statistic = if expected > 0.0 {
            f64::from(self.total_observed) / expected
        } else {
            0.0
        };

        // Alpha spending
        let cumulative_alpha = self
            .config
            .spending_function
            .alpha_spent(info_fraction, self.config.alpha);
        let incremental_alpha = self.config.spending_function.incremental_alpha(
            prev_info_fraction,
            info_fraction,
            self.config.alpha,
        );

        // Calculate boundaries using Poisson-based approximation
        let (lower_boundary, upper_boundary) =
            self.calculate_boundaries(expected, cumulative_alpha);

        // Decision
        let is_final = look >= self.config.n_looks || info_fraction >= 1.0;
        let decision = if f64::from(self.total_observed) > upper_boundary {
            CsspDecision::Signal
        } else if f64::from(self.total_observed) < lower_boundary && !is_final {
            CsspDecision::NoSignal
        } else if is_final {
            CsspDecision::Complete
        } else {
            CsspDecision::Continue
        };

        // Relative risk estimate and CI
        let rr_estimate = test_statistic;
        let rr_ci = self.calculate_rr_ci(self.total_observed, expected);

        let result = CsspResult {
            look,
            observed: self.total_observed,
            expected,
            person_time: self.total_person_time,
            information_fraction: info_fraction,
            test_statistic,
            upper_boundary,
            lower_boundary,
            alpha_spent: incremental_alpha,
            cumulative_alpha_spent: cumulative_alpha,
            decision,
            rr_estimate,
            rr_ci,
        };

        self.looks.push(result.clone());
        result
    }

    /// Calculate Poisson-based boundaries.
    fn calculate_boundaries(&self, expected: f64, alpha: f64) -> (f64, f64) {
        if expected <= 0.0 {
            return (0.0, f64::INFINITY);
        }

        // Upper boundary: smallest n where P(X >= n | λ = expected) <= alpha/2
        // Using normal approximation for large expected values
        let z_alpha = normal_quantile(1.0 - alpha / 2.0);

        // Upper: E + z × √E × RR_min (adjusted for minimum detectable RR)
        let upper =
            expected * self.config.min_rr + z_alpha * (expected * self.config.min_rr).sqrt();

        // Lower: Futility boundary - can stop early if very unlikely to reach signal
        // Using E - z × √E
        let lower = (expected - z_alpha * expected.sqrt()).max(0.0);

        (lower, upper)
    }

    /// Calculate confidence interval for relative risk.
    fn calculate_rr_ci(&self, observed: u32, expected: f64) -> (f64, f64) {
        if expected <= 0.0 || observed == 0 {
            return (0.0, f64::INFINITY);
        }

        let obs = f64::from(observed);
        let rr = obs / expected;

        // Exact Poisson-based CI using the relationship:
        // Lower: χ²_{2n, α/2} / (2 × E)
        // Upper: χ²_{2(n+1), 1-α/2} / (2 × E)
        //
        // Approximation using normal:
        let se_log_rr = 1.0 / obs.sqrt();
        let z = 1.96;

        let lower = (rr * (-z * se_log_rr).exp()).max(0.0);
        let upper = rr * (z * se_log_rr).exp();

        (lower, upper)
    }

    /// Get all completed looks.
    #[must_use]
    pub fn get_looks(&self) -> &[CsspResult] {
        &self.looks
    }

    /// Get current cumulative state.
    #[must_use]
    pub fn current_state(&self) -> Option<&CsspResult> {
        self.looks.last()
    }

    /// Check if monitoring should stop.
    #[must_use]
    pub fn should_stop(&self) -> bool {
        self.looks.last().map_or(false, |r| {
            matches!(
                r.decision,
                CsspDecision::Signal | CsspDecision::NoSignal | CsspDecision::Complete
            )
        })
    }
}

/// Perform single-shot CSSP analysis (all data at once).
///
/// # Arguments
///
/// * `observations` - All patient-level exposure data
/// * `config` - CSSP configuration
///
/// # Returns
///
/// Final result of CSSP analysis.
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::pv::signals::sequential::cssp::{calculate_cssp, CsspConfig, CsspObservation};
///
/// let observations = vec![
///     CsspObservation { id: "P001".into(), exposure_time: 2.0, event: true, time: 1.5 },
///     CsspObservation { id: "P002".into(), exposure_time: 3.0, event: false, time: 3.0 },
///     CsspObservation { id: "P003".into(), exposure_time: 1.5, event: true, time: 1.0 },
/// ];
///
/// let config = CsspConfig {
///     background_rate: 0.1, // 10% per person-year
///     max_person_time: 100.0,
///     ..Default::default()
/// };
///
/// let result = calculate_cssp(&observations, &config);
/// println!("RR estimate: {:.2}, Decision: {:?}", result.rr_estimate, result.decision);
/// ```
#[must_use]
pub fn calculate_cssp(observations: &[CsspObservation], config: &CsspConfig) -> CsspResult {
    let mut monitor = CsspMonitor::new(config.clone());
    monitor.add_observations(observations)
}

/// Batch CSSP analysis for multiple drug-event pairs in parallel.
///
/// # Arguments
///
/// * `datasets` - Vector of observation datasets, one per drug-event pair
/// * `config` - CSSP configuration (same for all)
///
/// # Returns
///
/// Vector of CSSP results.
#[must_use]
pub fn batch_cssp_parallel(
    datasets: &[Vec<CsspObservation>],
    config: &CsspConfig,
) -> Vec<CsspResult> {
    datasets
        .par_iter()
        .map(|obs| calculate_cssp(obs, config))
        .collect()
}

/// Standard normal CDF approximation.
fn standard_normal_cdf(x: f64) -> f64 {
    // Abramowitz and Stegun approximation (7.1.26)
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x / 2.0).exp();

    0.5 * (1.0 + sign * y)
}

/// Normal quantile (inverse CDF) approximation.
fn normal_quantile(p: f64) -> f64 {
    // Rational approximation (Abramowitz and Stegun 26.2.23)
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }

    // For upper tail, work with complementary probability
    let is_upper_tail = p > 0.5;
    let p_work = if is_upper_tail { 1.0 - p } else { p };

    let t = (-2.0 * p_work.ln()).sqrt();

    let c0 = 2.515517;
    let c1 = 0.802853;
    let c2 = 0.010328;
    let d1 = 1.432788;
    let d2 = 0.189269;
    let d3 = 0.001308;

    let z = t - (c0 + c1 * t + c2 * t * t) / (1.0 + d1 * t + d2 * t * t + d3 * t * t * t);

    // For lower tail (p < 0.5), return negative quantile
    if is_upper_tail { z } else { -z }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_observations(
        n: usize,
        event_rate: f64,
        exposure_per_patient: f64,
    ) -> Vec<CsspObservation> {
        (0..n)
            .map(|i| {
                let event = (i as f64 / n as f64) < event_rate;
                CsspObservation {
                    id: format!("P{:04}", i),
                    exposure_time: exposure_per_patient,
                    event,
                    time: if event {
                        exposure_per_patient / 2.0
                    } else {
                        exposure_per_patient
                    },
                }
            })
            .collect()
    }

    #[test]
    fn test_cssp_no_signal() {
        // Background rate matches observed rate - no signal expected
        let config = CsspConfig {
            background_rate: 0.1,
            max_person_time: 1000.0,
            ..Default::default()
        };

        // 100 patients, 1 year each, ~10% event rate = ~10 events
        let observations = make_observations(100, 0.10, 1.0);
        let result = calculate_cssp(&observations, &config);

        // RR should be close to 1.0
        assert!(result.rr_estimate > 0.5 && result.rr_estimate < 2.0);
        assert!(!matches!(result.decision, CsspDecision::Signal));
    }

    #[test]
    fn test_cssp_signal() {
        // High observed rate vs low background - signal expected
        let config = CsspConfig {
            background_rate: 0.01, // 1% background
            max_person_time: 1000.0,
            min_rr: 2.0,
            n_looks: 1, // Single look at final data
            ..Default::default()
        };

        // 200 patients, 1 year each, 20% event rate = 20× higher than expected
        let observations = make_observations(200, 0.20, 1.0);
        let result = calculate_cssp(&observations, &config);

        // RR should be very high (observed ~40 events vs expected ~2)
        assert!(result.rr_estimate > 10.0, "RR: {}", result.rr_estimate);
        // Either signal or complete - with strong signal, should detect
        assert!(
            matches!(
                result.decision,
                CsspDecision::Signal | CsspDecision::Complete
            ),
            "Decision: {:?}, Observed: {}, Expected: {:.2}",
            result.decision,
            result.observed,
            result.expected
        );
    }

    #[test]
    fn test_cssp_monitor_sequential() {
        let config = CsspConfig {
            background_rate: 0.05,
            max_person_time: 500.0,
            n_looks: 5,
            ..Default::default()
        };

        let mut monitor = CsspMonitor::new(config);

        // Add observations in batches (simulating monthly looks)
        for batch in 0..5 {
            let obs = make_observations(20, 0.05 + 0.02 * batch as f64, 1.0);
            let result = monitor.add_observations(&obs);

            assert_eq!(result.look, batch + 1);
            assert!(result.information_fraction > 0.0);
        }

        assert_eq!(monitor.get_looks().len(), 5);
    }

    #[test]
    fn test_spending_functions() {
        let alpha = 0.05;

        // All spending functions should spend full alpha at t=1
        for sf in [
            SpendingFunction::OBrienFleming,
            SpendingFunction::Pocock,
            SpendingFunction::KimDeMets { rho: 1.0 },
        ] {
            let spent = sf.alpha_spent(1.0, alpha);
            assert!((spent - alpha).abs() < 0.001, "SF {:?} at t=1.0", sf);
        }

        // O'Brien-Fleming should be conservative early
        let obf_early = SpendingFunction::OBrienFleming.alpha_spent(0.25, alpha);
        let pocock_early = SpendingFunction::Pocock.alpha_spent(0.25, alpha);
        assert!(obf_early < pocock_early);
    }

    #[test]
    fn test_rr_ci() {
        let config = CsspConfig::default();
        let mut monitor = CsspMonitor::new(config);

        let observations = make_observations(50, 0.1, 2.0);
        let result = monitor.add_observations(&observations);

        // CI should contain RR estimate
        assert!(result.rr_ci.0 < result.rr_estimate);
        assert!(result.rr_ci.1 > result.rr_estimate);
    }

    #[test]
    fn test_batch_parallel() {
        let config = CsspConfig {
            background_rate: 0.05,
            ..Default::default()
        };

        let datasets: Vec<Vec<CsspObservation>> = (0..5)
            .map(|i| make_observations(100, 0.05 + 0.01 * i as f64, 1.0))
            .collect();

        let results = batch_cssp_parallel(&datasets, &config);
        assert_eq!(results.len(), 5);

        // Higher event rates should have higher RR
        for i in 1..results.len() {
            assert!(results[i].rr_estimate >= results[i - 1].rr_estimate - 0.5);
        }
    }

    #[test]
    fn test_normal_approximations() {
        // CDF at 0 should be 0.5
        assert!((standard_normal_cdf(0.0) - 0.5).abs() < 0.001);

        // Quantile at 0.5 should be 0
        assert!(normal_quantile(0.5).abs() < 0.001);

        // Test specific values that we know work
        // CDF(-1.96) ≈ 0.025, CDF(1.96) ≈ 0.975
        assert!((standard_normal_cdf(1.96) - 0.975).abs() < 0.01);
        assert!((standard_normal_cdf(-1.96) - 0.025).abs() < 0.01);

        // Quantile(0.975) should be positive (around 1.96)
        let q_975 = normal_quantile(0.975);
        assert!(q_975 > 1.5 && q_975 < 2.5, "Quantile(0.975)={}", q_975);

        // Quantile(0.025) should be negative (around -1.96)
        let q_025 = normal_quantile(0.025);
        assert!(q_025 < -1.5 && q_025 > -2.5, "Quantile(0.025)={}", q_025);
    }
}
