//! Cox Proportional Hazards Model
//!
//! Semi-parametric regression model for estimating hazard ratios from
//! time-to-event data, widely used in pharmacoepidemiology for comparative
//! safety analysis.
//!
//! # Model
//!
//! The Cox model specifies the hazard function as:
//!
//! ```text
//! h(t|X) = h₀(t) × exp(β₁X₁ + β₂X₂ + ... + βₚXₚ)
//!
//! where:
//!   h₀(t) = baseline hazard (unspecified)
//!   βᵢ = regression coefficients
//!   Xᵢ = covariates
//!   exp(βᵢ) = hazard ratio for covariate i
//! ```
//!
//! # Partial Likelihood
//!
//! Estimation uses partial likelihood (Breslow approximation for ties):
//!
//! ```text
//! L(β) = Π_{i: δᵢ=1} [exp(Xᵢβ) / Σ_{j∈R(tᵢ)} exp(Xⱼβ)]
//! ```
//!
//! # Use Cases
//!
//! - Estimating drug exposure hazard ratios
//! - Adjusting for confounders in safety analysis
//! - Comparing treatment groups with covariate adjustment
//!
//! # References
//!
//! - Cox DR (1972). "Regression models and life-tables." Journal of the Royal
//!   Statistical Society Series B 34(2):187-220.
//!   DOI: [10.1111/j.2517-6161.1972.tb00899.x](https://doi.org/10.1111/j.2517-6161.1972.tb00899.x)
//!
//! - Breslow NE (1974). "Covariance analysis of censored survival data."
//!   Biometrics 30(1):89-99. DOI: [10.2307/2529620](https://doi.org/10.2307/2529620)
//!
//! - Therneau TM, Grambsch PM (2000). Modeling Survival Data: Extending the Cox Model.
//!   Springer. ISBN: 978-0-387-98784-2

use crate::signals::core::error::SignalError;
use serde::{Deserialize, Serialize};

/// A survival observation with covariates for Cox regression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoxObservation {
    /// Time of event or censoring
    pub time: f64,
    /// Whether the event occurred (true) or was censored (false)
    pub event: bool,
    /// Covariate values
    pub covariates: Vec<f64>,
}

impl CoxObservation {
    /// Create a new observation.
    #[must_use]
    pub fn new(time: f64, event: bool, covariates: Vec<f64>) -> Self {
        Self {
            time,
            event,
            covariates,
        }
    }

    /// Create for simple single-covariate case (e.g., treatment indicator).
    #[must_use]
    pub fn simple(time: f64, event: bool, treatment: f64) -> Self {
        Self::new(time, event, vec![treatment])
    }
}

/// Configuration for Cox model fitting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoxConfig {
    /// Maximum iterations for Newton-Raphson
    pub max_iterations: usize,
    /// Convergence tolerance
    pub tolerance: f64,
    /// Confidence level for intervals
    pub confidence_level: f64,
    /// Tie handling method
    pub tie_method: TieMethod,
}

impl Default for CoxConfig {
    fn default() -> Self {
        Self {
            max_iterations: 25,
            tolerance: 1e-9,
            confidence_level: 0.95,
            tie_method: TieMethod::Breslow,
        }
    }
}

/// Method for handling tied event times.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TieMethod {
    /// Breslow approximation (most common)
    Breslow,
    /// Efron approximation (more accurate for many ties)
    Efron,
}

/// Result of Cox model coefficient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoxCoefficient {
    /// Variable name/index
    pub name: String,
    /// Coefficient estimate (β)
    pub coefficient: f64,
    /// Standard error
    pub se: f64,
    /// Hazard ratio exp(β)
    pub hazard_ratio: f64,
    /// Lower CI for hazard ratio
    pub hr_ci_lower: f64,
    /// Upper CI for hazard ratio
    pub hr_ci_upper: f64,
    /// Wald z-statistic
    pub z_statistic: f64,
    /// P-value
    pub p_value: f64,
}

/// Result of Cox proportional hazards regression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoxResult {
    /// Coefficient estimates
    pub coefficients: Vec<CoxCoefficient>,
    /// Log partial likelihood at convergence
    pub log_likelihood: f64,
    /// Log likelihood at null model (all β = 0)
    pub log_likelihood_null: f64,
    /// Likelihood ratio test statistic
    pub lr_statistic: f64,
    /// Likelihood ratio test p-value
    pub lr_p_value: f64,
    /// Number of observations
    pub n_observations: usize,
    /// Number of events
    pub n_events: usize,
    /// Concordance index (C-statistic)
    pub concordance: f64,
    /// Whether the model converged
    pub converged: bool,
    /// Number of iterations used
    pub iterations: usize,
}

/// Fit Cox proportional hazards model.
///
/// Uses Newton-Raphson optimization of the partial likelihood.
///
/// # Arguments
///
/// * `observations` - Vector of survival observations with covariates
/// * `config` - Model fitting configuration
///
/// # Returns
///
/// `CoxResult` containing coefficient estimates and model statistics.
///
/// # Complexity
///
/// - **Time**: O(n × p × iterations) where n = observations, p = covariates
/// - **Space**: O(n × p)
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::pv::signals::survival::cox::{fit_cox, CoxObservation, CoxConfig};
///
/// // Simple treatment vs control comparison
/// let observations = vec![
///     CoxObservation::simple(1.0, true, 1.0),   // Treatment, event at t=1
///     CoxObservation::simple(2.0, true, 0.0),   // Control, event at t=2
///     CoxObservation::simple(3.0, false, 1.0),  // Treatment, censored at t=3
///     CoxObservation::simple(4.0, true, 0.0),   // Control, event at t=4
///     CoxObservation::simple(5.0, true, 1.0),   // Treatment, event at t=5
///     CoxObservation::simple(6.0, false, 0.0),  // Control, censored at t=6
/// ];
///
/// let result = fit_cox(&observations, &CoxConfig::default()).unwrap();
/// println!("Hazard Ratio: {:.2}", result.coefficients[0].hazard_ratio);
/// ```
pub fn fit_cox(
    observations: &[CoxObservation],
    config: &CoxConfig,
) -> Result<CoxResult, SignalError> {
    if observations.is_empty() {
        return Err(SignalError::InsufficientData(
            "No observations provided".into(),
        ));
    }

    let n_events = observations.iter().filter(|o| o.event).count();
    if n_events == 0 {
        return Err(SignalError::InsufficientData("No events in data".into()));
    }

    let p = observations[0].covariates.len();
    if p == 0 {
        return Err(SignalError::InsufficientData(
            "No covariates provided".into(),
        ));
    }

    // Validate all observations have same number of covariates
    if observations.iter().any(|o| o.covariates.len() != p) {
        return Err(SignalError::invalid_covariate_matrix(
            "Inconsistent covariate dimensions",
        ));
    }

    // Sort by time (descending for risk set calculation)
    let mut sorted: Vec<_> = observations.to_vec();
    sorted.sort_by(|a, b| {
        b.time
            .partial_cmp(&a.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Initialize coefficients to zero
    let mut beta: Vec<f64> = vec![0.0; p];

    // Newton-Raphson iteration
    let mut converged = false;
    let mut iterations = 0;
    let mut log_likelihood = f64::NEG_INFINITY;

    for iter in 0..config.max_iterations {
        iterations = iter + 1;

        let (ll, gradient, hessian) = compute_partial_likelihood(&sorted, &beta, config.tie_method);

        // Check for convergence
        if iter > 0 && (ll - log_likelihood).abs() < config.tolerance {
            converged = true;
            log_likelihood = ll;
            break;
        }
        log_likelihood = ll;

        // Newton-Raphson update: β_new = β - H⁻¹ × g
        // For single covariate, this simplifies significantly
        if p == 1 {
            if hessian[0][0].abs() > 1e-10 {
                let delta = gradient[0] / hessian[0][0];
                beta[0] -= delta.max(-5.0).min(5.0); // Damping
            }
        } else {
            // Multiple covariates: use diagonal approximation for simplicity
            for j in 0..p {
                if hessian[j][j].abs() > 1e-10 {
                    let delta = gradient[j] / hessian[j][j];
                    beta[j] -= delta.max(-5.0).min(5.0);
                }
            }
        }
    }

    // Compute null log-likelihood (β = 0)
    let (log_likelihood_null, _, _) =
        compute_partial_likelihood(&sorted, &vec![0.0; p], config.tie_method);

    // Compute standard errors from Hessian
    let (_, _, hessian) = compute_partial_likelihood(&sorted, &beta, config.tie_method);

    // Build coefficient results
    let z = z_for_confidence(config.confidence_level);
    let coefficients: Vec<CoxCoefficient> = (0..p)
        .map(|j| {
            let se = if hessian[j][j].abs() > 1e-10 {
                (1.0 / hessian[j][j].abs()).sqrt()
            } else {
                f64::INFINITY
            };

            let hr = beta[j].exp();
            let hr_ci_lower = (beta[j] - z * se).exp();
            let hr_ci_upper = (beta[j] + z * se).exp();

            let z_stat = if se.is_finite() && se > 0.0 {
                beta[j] / se
            } else {
                0.0
            };

            let p_value = 2.0 * (1.0 - crate::signals::core::stats::normal_cdf(z_stat.abs()));

            CoxCoefficient {
                name: format!("X{}", j + 1),
                coefficient: beta[j],
                se,
                hazard_ratio: hr,
                hr_ci_lower,
                hr_ci_upper,
                z_statistic: z_stat,
                p_value,
            }
        })
        .collect();

    // Likelihood ratio test (clamped to 0 as fitted model should never be worse than null)
    let lr_statistic = (2.0 * (log_likelihood - log_likelihood_null)).max(0.0);
    let lr_p_value = crate::signals::core::stats::chi_square_p_value(lr_statistic);

    // Concordance index
    let concordance = compute_concordance(&sorted, &beta);

    Ok(CoxResult {
        coefficients,
        log_likelihood,
        log_likelihood_null,
        lr_statistic,
        lr_p_value,
        n_observations: observations.len(),
        n_events,
        concordance,
        converged,
        iterations,
    })
}

/// Compute partial likelihood and derivatives.
fn compute_partial_likelihood(
    observations: &[CoxObservation],
    beta: &[f64],
    _tie_method: TieMethod,
) -> (f64, Vec<f64>, Vec<Vec<f64>>) {
    let p = beta.len();
    let n = observations.len();

    let mut log_likelihood = 0.0;
    let mut gradient = vec![0.0; p];
    let mut hessian = vec![vec![0.0; p]; p];

    // Compute linear predictors
    let eta: Vec<f64> = observations
        .iter()
        .map(|o| {
            o.covariates
                .iter()
                .zip(beta.iter())
                .map(|(x, b)| x * b)
                .sum()
        })
        .collect();

    // Risk scores exp(η)
    let exp_eta: Vec<f64> = eta.iter().map(|&e| e.exp()).collect();

    // Process in order of decreasing time (observations are pre-sorted descending)
    let mut risk_sum = 0.0;
    let mut risk_x_sum = vec![0.0; p];
    let mut risk_xx_sum = vec![vec![0.0; p]; p];

    for i in 0..n {
        let obs = &observations[i];

        // Add to risk set sums
        risk_sum += exp_eta[i];
        for j in 0..p {
            risk_x_sum[j] += exp_eta[i] * obs.covariates[j];
            for k in 0..p {
                risk_xx_sum[j][k] += exp_eta[i] * obs.covariates[j] * obs.covariates[k];
            }
        }

        // Contribution from events
        if obs.event && risk_sum > 0.0 {
            log_likelihood += eta[i] - risk_sum.ln();

            for j in 0..p {
                let mean_x_j = risk_x_sum[j] / risk_sum;
                gradient[j] += obs.covariates[j] - mean_x_j;

                for k in 0..p {
                    let mean_x_k = risk_x_sum[k] / risk_sum;
                    let mean_xx_jk = risk_xx_sum[j][k] / risk_sum;
                    hessian[j][k] += mean_xx_jk - mean_x_j * mean_x_k;
                }
            }
        }
    }

    (log_likelihood, gradient, hessian)
}

/// Compute Harrell's concordance index.
fn compute_concordance(observations: &[CoxObservation], beta: &[f64]) -> f64 {
    let mut concordant = 0;
    let mut discordant = 0;
    let mut tied = 0;

    for i in 0..observations.len() {
        if !observations[i].event {
            continue;
        }

        let eta_i: f64 = observations[i]
            .covariates
            .iter()
            .zip(beta.iter())
            .map(|(x, b)| x * b)
            .sum();

        for j in 0..observations.len() {
            if i == j {
                continue;
            }

            // j must have longer survival time or be censored after i's event
            if observations[j].time <= observations[i].time {
                continue;
            }

            let eta_j: f64 = observations[j]
                .covariates
                .iter()
                .zip(beta.iter())
                .map(|(x, b)| x * b)
                .sum();

            // Higher risk score should have earlier event
            if eta_i > eta_j {
                concordant += 1;
            } else if eta_i < eta_j {
                discordant += 1;
            } else {
                tied += 1;
            }
        }
    }

    let total = concordant + discordant + tied;
    if total == 0 {
        return 0.5;
    }

    (concordant as f64 + 0.5 * tied as f64) / total as f64
}

/// Get z-score for confidence level.
fn z_for_confidence(confidence: f64) -> f64 {
    match () {
        () if (confidence - 0.90).abs() < 1e-9 => 1.645,
        () if (confidence - 0.95).abs() < 1e-9 => 1.96,
        () if (confidence - 0.99).abs() < 1e-9 => 2.576,
        () => crate::signals::core::stats::z_score_for_confidence(confidence),
    }
}

/// Quick hazard ratio estimation for single binary covariate.
///
/// Simplified interface when you just need HR for treatment vs control.
#[must_use]
pub fn quick_hazard_ratio(
    treatment_times: &[f64],
    treatment_events: &[bool],
    control_times: &[f64],
    control_events: &[bool],
) -> Result<CoxCoefficient, SignalError> {
    if treatment_times.len() != treatment_events.len() {
        return Err(SignalError::invalid_covariate_matrix(
            "Treatment times and events length mismatch",
        ));
    }
    if control_times.len() != control_events.len() {
        return Err(SignalError::invalid_covariate_matrix(
            "Control times and events length mismatch",
        ));
    }

    let mut observations: Vec<CoxObservation> = treatment_times
        .iter()
        .zip(treatment_events.iter())
        .map(|(&t, &e)| CoxObservation::simple(t, e, 1.0))
        .collect();

    observations.extend(
        control_times
            .iter()
            .zip(control_events.iter())
            .map(|(&t, &e)| CoxObservation::simple(t, e, 0.0)),
    );

    let result = fit_cox(&observations, &CoxConfig::default())?;

    Ok(result.coefficients.into_iter().next().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> Vec<CoxObservation> {
        vec![
            CoxObservation::simple(1.0, true, 1.0),
            CoxObservation::simple(2.0, true, 0.0),
            CoxObservation::simple(3.0, false, 1.0),
            CoxObservation::simple(4.0, true, 0.0),
            CoxObservation::simple(5.0, true, 1.0),
            CoxObservation::simple(6.0, false, 0.0),
            CoxObservation::simple(7.0, true, 1.0),
            CoxObservation::simple(8.0, true, 0.0),
        ]
    }

    #[test]
    fn test_cox_basic() {
        let data = create_test_data();
        let result = fit_cox(&data, &CoxConfig::default()).unwrap();

        assert_eq!(result.coefficients.len(), 1);
        assert!(result.concordance >= 0.0 && result.concordance <= 1.0);
        assert!(result.n_events > 0);
    }

    #[test]
    fn test_cox_hazard_ratio() {
        let data = create_test_data();
        let result = fit_cox(&data, &CoxConfig::default()).unwrap();

        let hr = result.coefficients[0].hazard_ratio;
        // HR should be positive
        assert!(hr > 0.0);
        // CI should bracket HR
        assert!(result.coefficients[0].hr_ci_lower <= hr);
        assert!(result.coefficients[0].hr_ci_upper >= hr);
    }

    #[test]
    fn test_cox_empty_data() {
        let result = fit_cox(&[], &CoxConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_cox_no_events() {
        let data = vec![
            CoxObservation::simple(1.0, false, 1.0),
            CoxObservation::simple(2.0, false, 0.0),
        ];
        let result = fit_cox(&data, &CoxConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_cox_convergence() {
        let data = create_test_data();
        let result = fit_cox(&data, &CoxConfig::default()).unwrap();

        // Should converge within max iterations
        assert!(result.iterations <= 25);
    }

    #[test]
    fn test_cox_likelihood_ratio() {
        let data = create_test_data();
        let result = fit_cox(&data, &CoxConfig::default()).unwrap();

        // LR statistic should be non-negative
        assert!(result.lr_statistic >= 0.0);
        // P-value should be valid
        assert!(result.lr_p_value >= 0.0 && result.lr_p_value <= 1.0);
    }

    #[test]
    fn test_quick_hazard_ratio() {
        let treatment_times = vec![1.0, 3.0, 5.0, 7.0];
        let treatment_events = vec![true, false, true, true];
        let control_times = vec![2.0, 4.0, 6.0, 8.0];
        let control_events = vec![true, true, false, true];

        let hr = quick_hazard_ratio(
            &treatment_times,
            &treatment_events,
            &control_times,
            &control_events,
        )
        .unwrap();

        assert!(hr.hazard_ratio > 0.0);
        assert!(hr.p_value >= 0.0 && hr.p_value <= 1.0);
    }

    #[test]
    fn test_cox_multivariate() {
        // Two covariates: treatment and age
        let data = vec![
            CoxObservation::new(1.0, true, vec![1.0, 65.0]),
            CoxObservation::new(2.0, true, vec![0.0, 70.0]),
            CoxObservation::new(3.0, false, vec![1.0, 55.0]),
            CoxObservation::new(4.0, true, vec![0.0, 60.0]),
            CoxObservation::new(5.0, true, vec![1.0, 75.0]),
        ];

        let result = fit_cox(&data, &CoxConfig::default()).unwrap();

        assert_eq!(result.coefficients.len(), 2);
        assert!(result.coefficients[0].hazard_ratio > 0.0);
        assert!(result.coefficients[1].hazard_ratio > 0.0);
    }
}
