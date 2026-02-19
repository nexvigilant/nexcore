//! High-Dimensional Propensity Score (HDPS) for Confounding Control
//!
//! This module implements the HDPS algorithm for automated covariate selection
//! and propensity score estimation in pharmacoepidemiologic studies.
//!
//! # Algorithm Overview
//!
//! HDPS addresses confounding in observational studies by:
//! 1. Identifying empirically relevant covariates from high-dimensional data
//! 2. Selecting top-k covariates based on bias-reducing potential (Bross formula)
//! 3. Estimating propensity scores via logistic regression
//! 4. Supporting stratification, matching, or weighting adjustment
//!
//! # The Bross Formula
//!
//! For each covariate, the bias potential is estimated as:
//! ```text
//! bias = (RR_CD - 1) / RR_CD × (PC1 - PC0)
//! ```
//! Where:
//! - `RR_CD`: Relative risk of outcome given covariate
//! - `PC1`: Prevalence of covariate in exposed group
//! - `PC0`: Prevalence of covariate in unexposed group
//!
//! # Use Cases
//!
//! - Comparative effectiveness research
//! - Post-market drug safety surveillance
//! - Real-world evidence generation
//! - Observational study confounding adjustment
//!
//! # References
//!
//! - Schneeweiss S, Rassen JA, Glynn RJ, et al. (2009). "High-dimensional propensity score
//!   adjustment in studies of treatment effects using health care claims data."
//!   Epidemiology 20(4):512-522. DOI: [10.1097/EDE.0b013e3181a663cc](https://doi.org/10.1097/EDE.0b013e3181a663cc)
//!
//! - Rassen JA, Glynn RJ, Brookhart MA, Schneeweiss S (2011). "Covariate selection in
//!   high-dimensional propensity score analyses of treatment effects in small samples."
//!   American Journal of Epidemiology 173(12):1404-1413.
//!   DOI: [10.1093/aje/kwr001](https://doi.org/10.1093/aje/kwr001)
//!
//! - Schneeweiss S (2006). "Sensitivity analysis and external adjustment for unmeasured
//!   confounders in epidemiologic database studies of therapeutics." Pharmacoepidemiology
//!   and Drug Safety 15(5):291-303. DOI: [10.1002/pds.1200](https://doi.org/10.1002/pds.1200)
//!
//! - Bross IDJ (1966). "Spurious effects from an extraneous variable." Journal of Chronic
//!   Diseases 19(6):637-647. DOI: [10.1016/0021-9681(66)90062-2](https://doi.org/10.1016/0021-9681(66)90062-2)
//!
//! # Feature Flag
//!
//! This module requires the `propensity` feature flag due to nalgebra dependency:
//! ```toml
//! guardian-signals = { version = "...", features = ["propensity"] }
//! ```

use crate::signals::core::error::SignalError;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for HDPS calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HDPSConfig {
    /// Number of top covariates to select per data dimension (default: 200)
    pub top_k_per_dimension: usize,
    /// Total covariates to include in final propensity model (default: 500)
    pub total_covariates: usize,
    /// Minimum prevalence threshold for covariate inclusion (default: 0.01)
    pub min_prevalence: f64,
    /// Maximum prevalence threshold (default: 0.99)
    pub max_prevalence: f64,
    /// Minimum number of exposed/unexposed with covariate (default: 5)
    pub min_count: usize,
    /// Whether to use recurrence (count > 0) or binary (any occurrence) (default: false)
    pub use_recurrence: bool,
    /// Maximum iterations for logistic regression convergence (default: 100)
    pub max_iterations: usize,
    /// Convergence tolerance for logistic regression (default: 1e-6)
    pub convergence_tolerance: f64,
}

impl Default for HDPSConfig {
    fn default() -> Self {
        Self {
            top_k_per_dimension: 200,
            total_covariates: 500,
            min_prevalence: 0.01,
            max_prevalence: 0.99,
            min_count: 5,
            use_recurrence: false,
            max_iterations: 100,
            convergence_tolerance: 1e-6,
        }
    }
}

impl HDPSConfig {
    /// Create a new configuration with specified parameters.
    #[must_use]
    pub const fn new(
        top_k_per_dimension: usize,
        total_covariates: usize,
        min_prevalence: f64,
    ) -> Self {
        Self {
            top_k_per_dimension,
            total_covariates,
            min_prevalence,
            max_prevalence: 0.99,
            min_count: 5,
            use_recurrence: false,
            max_iterations: 100,
            convergence_tolerance: 1e-6,
        }
    }

    /// Aggressive configuration for smaller samples.
    #[must_use]
    pub const fn small_sample() -> Self {
        Self {
            top_k_per_dimension: 50,
            total_covariates: 100,
            min_prevalence: 0.02,
            max_prevalence: 0.98,
            min_count: 3,
            use_recurrence: false,
            max_iterations: 200,
            convergence_tolerance: 1e-8,
        }
    }

    /// Conservative configuration for large datasets.
    #[must_use]
    pub const fn large_sample() -> Self {
        Self {
            top_k_per_dimension: 500,
            total_covariates: 1000,
            min_prevalence: 0.005,
            max_prevalence: 0.995,
            min_count: 10,
            use_recurrence: true,
            max_iterations: 100,
            convergence_tolerance: 1e-6,
        }
    }
}

/// Raw covariate data from a single data dimension (e.g., diagnoses, procedures).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CovariateData {
    /// Unique identifier for this data dimension (e.g., "dx_codes", "rx_codes")
    pub dimension_id: String,
    /// Covariate identifier (e.g., ICD code, NDC code)
    pub covariate_id: String,
    /// Binary indicator for each subject (true = covariate present)
    pub values: Vec<bool>,
}

impl CovariateData {
    /// Create new covariate data.
    #[must_use]
    pub fn new(dimension_id: String, covariate_id: String, values: Vec<bool>) -> Self {
        Self {
            dimension_id,
            covariate_id,
            values,
        }
    }

    /// Calculate prevalence in a subset of subjects.
    #[must_use]
    pub fn prevalence_in(&self, indices: &[usize]) -> f64 {
        if indices.is_empty() {
            return 0.0;
        }
        let count = indices
            .iter()
            .filter(|&&i| self.values.get(i).copied().unwrap_or(false))
            .count();
        count as f64 / indices.len() as f64
    }

    /// Calculate prevalence across all subjects.
    #[must_use]
    pub fn prevalence(&self) -> f64 {
        if self.values.is_empty() {
            return 0.0;
        }
        let count = self.values.iter().filter(|&&v| v).count();
        count as f64 / self.values.len() as f64
    }
}

/// A covariate selected for the propensity model with bias metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedCovariate {
    /// Original dimension ID
    pub dimension_id: String,
    /// Covariate identifier
    pub covariate_id: String,
    /// Estimated bias reduction potential (absolute value)
    pub bias_score: f64,
    /// Prevalence in exposed group
    pub prevalence_exposed: f64,
    /// Prevalence in unexposed group
    pub prevalence_unexposed: f64,
    /// Relative risk of outcome given covariate (RR_CD)
    pub rr_outcome: f64,
    /// Rank among all covariates (1 = highest bias potential)
    pub rank: usize,
}

/// Result of HDPS calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HDPSResult {
    /// Selected covariates ranked by bias-reducing potential
    pub selected_covariates: Vec<SelectedCovariate>,
    /// Propensity scores for each subject (P(exposure=1 | covariates))
    pub propensity_scores: Vec<f64>,
    /// C-statistic (AUC) for propensity model discrimination
    pub c_statistic: f64,
    /// Number of subjects
    pub n_subjects: usize,
    /// Number of exposed subjects
    pub n_exposed: usize,
    /// Number of unexposed subjects
    pub n_unexposed: usize,
    /// Model coefficients (intercept + covariate coefficients)
    pub coefficients: Vec<f64>,
    /// Whether the model converged
    pub converged: bool,
    /// Number of iterations used
    pub iterations: usize,
}

impl HDPSResult {
    /// Get propensity score deciles for stratification.
    #[must_use]
    pub fn propensity_deciles(&self) -> Vec<(f64, f64)> {
        let mut sorted: Vec<f64> = self.propensity_scores.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n = sorted.len();
        if n == 0 {
            return vec![];
        }

        (0..10)
            .map(|i| {
                let lower_idx = (i * n) / 10;
                let upper_idx = ((i + 1) * n) / 10 - 1;
                (sorted[lower_idx], sorted[upper_idx.min(n - 1)])
            })
            .collect()
    }

    /// Calculate standardized mean differences for balance assessment.
    ///
    /// Returns a map of covariate_id -> SMD value.
    /// SMD < 0.1 generally indicates good balance.
    #[must_use]
    pub fn balance_metrics(
        &self,
        covariates: &[CovariateData],
        exposure: &[bool],
    ) -> HashMap<String, f64> {
        let exposed_idx: Vec<usize> = exposure
            .iter()
            .enumerate()
            .filter(|(_, e)| **e)
            .map(|(i, _)| i)
            .collect();
        let unexposed_idx: Vec<usize> = exposure
            .iter()
            .enumerate()
            .filter(|(_, e)| !**e)
            .map(|(i, _)| i)
            .collect();

        covariates
            .iter()
            .map(|cov| {
                let p1 = cov.prevalence_in(&exposed_idx);
                let p0 = cov.prevalence_in(&unexposed_idx);
                // SMD for binary variables: (p1 - p0) / sqrt((p1(1-p1) + p0(1-p0)) / 2)
                let var_pooled = (p1 * (1.0 - p1) + p0 * (1.0 - p0)) / 2.0;
                let smd = if var_pooled > 0.0 {
                    (p1 - p0).abs() / var_pooled.sqrt()
                } else {
                    0.0
                };
                (cov.covariate_id.clone(), smd)
            })
            .collect()
    }
}

/// Calculate HDPS propensity scores.
///
/// This function implements the full HDPS algorithm:
/// 1. Screen covariates by prevalence
/// 2. Calculate bias potential using Bross formula
/// 3. Select top-k covariates per dimension
/// 4. Fit propensity model via logistic regression
///
/// # Arguments
///
/// * `covariates` - All candidate covariates from multiple data dimensions
/// * `exposure` - Binary exposure indicator for each subject
/// * `outcome` - Binary outcome indicator for each subject (used for covariate selection)
/// * `config` - HDPS configuration
///
/// # Returns
///
/// * `Ok(HDPSResult)` - Selected covariates and propensity scores
/// * `Err(SignalError)` - If data is invalid or estimation fails
///
/// # Complexity
///
/// TIME: O(n_covariates * n_subjects + n_selected * iterations)
/// SPACE: O(n_subjects + n_selected)
pub fn calculate_hdps(
    covariates: &[CovariateData],
    exposure: &[bool],
    outcome: &[bool],
    config: &HDPSConfig,
) -> Result<HDPSResult, SignalError> {
    // Validate inputs
    let n_subjects = exposure.len();
    if n_subjects == 0 {
        return Err(SignalError::InsufficientData("No subjects provided".into()));
    }
    if outcome.len() != n_subjects {
        return Err(SignalError::invalid_covariate_matrix(format!(
            "Outcome length {} != exposure length {}",
            outcome.len(),
            n_subjects
        )));
    }
    if covariates.is_empty() {
        return Err(SignalError::insufficient_covariates(
            "No covariates provided",
        ));
    }

    // Validate covariate dimensions
    for cov in covariates {
        if cov.values.len() != n_subjects {
            return Err(SignalError::invalid_covariate_matrix(format!(
                "Covariate {} has {} values, expected {}",
                cov.covariate_id,
                cov.values.len(),
                n_subjects
            )));
        }
    }

    // Get exposed/unexposed indices
    let exposed_idx: Vec<usize> = exposure
        .iter()
        .enumerate()
        .filter(|(_, e)| **e)
        .map(|(i, _)| i)
        .collect();
    let unexposed_idx: Vec<usize> = exposure
        .iter()
        .enumerate()
        .filter(|(_, e)| !**e)
        .map(|(i, _)| i)
        .collect();

    let n_exposed = exposed_idx.len();
    let n_unexposed = unexposed_idx.len();

    if n_exposed == 0 || n_unexposed == 0 {
        return Err(SignalError::InsufficientData(
            "Need both exposed and unexposed subjects".into(),
        ));
    }

    // Step 1: Screen and score covariates (in parallel)
    let scored_covariates: Vec<_> = covariates
        .par_iter()
        .filter_map(|cov| score_covariate(cov, &exposed_idx, &unexposed_idx, outcome, config))
        .collect();

    if scored_covariates.is_empty() {
        return Err(SignalError::insufficient_covariates(
            "No covariates passed screening criteria",
        ));
    }

    // Step 2: Group by dimension and select top-k per dimension
    let mut by_dimension: HashMap<String, Vec<(usize, f64)>> = HashMap::new();
    for (idx, (dim_id, _, bias_score, _, _, _)) in scored_covariates.iter().enumerate() {
        by_dimension
            .entry(dim_id.clone())
            .or_default()
            .push((idx, *bias_score));
    }

    // Select top-k per dimension
    let mut selected_indices: Vec<usize> = Vec::new();
    for covs in by_dimension.values_mut() {
        covs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        selected_indices.extend(
            covs.iter()
                .take(config.top_k_per_dimension)
                .map(|(i, _)| *i),
        );
    }

    // Step 3: Re-rank globally and take top total_covariates
    let mut global_ranked: Vec<_> = selected_indices
        .iter()
        .map(|&i| (i, scored_covariates[i].2))
        .collect();
    global_ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let final_indices: Vec<usize> = global_ranked
        .iter()
        .take(config.total_covariates)
        .map(|(i, _)| *i)
        .collect();

    // Build selected covariates with ranks
    let selected_covariates: Vec<SelectedCovariate> = final_indices
        .iter()
        .enumerate()
        .map(|(rank, &idx)| {
            let (dim_id, cov_id, bias_score, p_exp, p_unexp, rr_outcome) = &scored_covariates[idx];
            SelectedCovariate {
                dimension_id: dim_id.clone(),
                covariate_id: cov_id.clone(),
                bias_score: *bias_score,
                prevalence_exposed: *p_exp,
                prevalence_unexposed: *p_unexp,
                rr_outcome: *rr_outcome,
                rank: rank + 1,
            }
        })
        .collect();

    if selected_covariates.is_empty() {
        return Err(SignalError::insufficient_covariates(
            "No covariates selected after ranking",
        ));
    }

    // Step 4: Build design matrix for selected covariates
    let design_matrix =
        build_design_matrix(covariates, &final_indices, &scored_covariates, n_subjects);

    // Step 5: Fit logistic regression for propensity scores
    let (coefficients, converged, iterations) = logistic_regression(
        &design_matrix,
        exposure,
        config.max_iterations,
        config.convergence_tolerance,
    )?;

    // Step 6: Calculate propensity scores
    let propensity_scores = calculate_propensity_scores(&design_matrix, &coefficients);

    // Step 7: Calculate C-statistic
    let c_statistic = calculate_c_statistic(&propensity_scores, exposure);

    Ok(HDPSResult {
        selected_covariates,
        propensity_scores,
        c_statistic,
        n_subjects,
        n_exposed,
        n_unexposed,
        coefficients,
        converged,
        iterations,
    })
}

/// Score a single covariate using the Bross formula.
///
/// Returns None if covariate doesn't meet screening criteria.
fn score_covariate(
    cov: &CovariateData,
    exposed_idx: &[usize],
    unexposed_idx: &[usize],
    outcome: &[bool],
    config: &HDPSConfig,
) -> Option<(String, String, f64, f64, f64, f64)> {
    // Calculate prevalence in each group
    let p_exposed = cov.prevalence_in(exposed_idx);
    let p_unexposed = cov.prevalence_in(unexposed_idx);

    // Screen by overall prevalence
    let p_overall = cov.prevalence();
    if p_overall < config.min_prevalence || p_overall > config.max_prevalence {
        return None;
    }

    // Screen by minimum count
    let count_exposed = exposed_idx
        .iter()
        .filter(|&&i| cov.values.get(i).copied().unwrap_or(false))
        .count();
    let count_unexposed = unexposed_idx
        .iter()
        .filter(|&&i| cov.values.get(i).copied().unwrap_or(false))
        .count();
    if count_exposed < config.min_count || count_unexposed < config.min_count {
        return None;
    }

    // Calculate RR_CD (relative risk of outcome given covariate)
    let with_cov: Vec<usize> = (0..cov.values.len()).filter(|&i| cov.values[i]).collect();
    let without_cov: Vec<usize> = (0..cov.values.len()).filter(|&i| !cov.values[i]).collect();

    let outcome_rate_with = if with_cov.is_empty() {
        0.0
    } else {
        with_cov
            .iter()
            .filter(|&&i| outcome.get(i).copied().unwrap_or(false))
            .count() as f64
            / with_cov.len() as f64
    };
    let outcome_rate_without = if without_cov.is_empty() {
        0.0
    } else {
        without_cov
            .iter()
            .filter(|&&i| outcome.get(i).copied().unwrap_or(false))
            .count() as f64
            / without_cov.len() as f64
    };

    // RR_CD with continuity correction to avoid division by zero
    let rr_outcome = if outcome_rate_without > 0.0 {
        outcome_rate_with / outcome_rate_without
    } else if outcome_rate_with > 0.0 {
        outcome_rate_with / 0.001 // Cap at high RR
    } else {
        1.0 // No bias if no outcome in either group
    };

    // Bross formula: bias = (RR_CD - 1) / RR_CD * (PC1 - PC0)
    let bias_score = if rr_outcome > 0.0 {
        ((rr_outcome - 1.0) / rr_outcome * (p_exposed - p_unexposed)).abs()
    } else {
        0.0
    };

    Some((
        cov.dimension_id.clone(),
        cov.covariate_id.clone(),
        bias_score,
        p_exposed,
        p_unexposed,
        rr_outcome,
    ))
}

/// Build design matrix for logistic regression.
///
/// Returns a row-major matrix: n_subjects x (1 + n_covariates)
/// First column is intercept (all 1s).
fn build_design_matrix(
    all_covariates: &[CovariateData],
    selected_indices: &[usize],
    scored_covariates: &[(String, String, f64, f64, f64, f64)],
    n_subjects: usize,
) -> Vec<Vec<f64>> {
    // Find original covariate data for each selected index
    let mut matrix: Vec<Vec<f64>> = vec![vec![0.0; 1 + selected_indices.len()]; n_subjects];

    // Set intercept column
    for row in &mut matrix {
        row[0] = 1.0;
    }

    // Fill covariate columns
    for (col_idx, &scored_idx) in selected_indices.iter().enumerate() {
        let (dim_id, cov_id, _, _, _, _) = &scored_covariates[scored_idx];
        // Find matching covariate in original data
        if let Some(cov) = all_covariates
            .iter()
            .find(|c| c.dimension_id == *dim_id && c.covariate_id == *cov_id)
        {
            for (row_idx, &val) in cov.values.iter().enumerate() {
                matrix[row_idx][col_idx + 1] = if val { 1.0 } else { 0.0 };
            }
        }
    }

    matrix
}

/// Logistic regression via IRLS (Iteratively Reweighted Least Squares).
///
/// This is a simple implementation without regularization.
/// For production use with high-dimensional data, consider L1/L2 regularization.
fn logistic_regression(
    design_matrix: &[Vec<f64>],
    outcome: &[bool],
    max_iterations: usize,
    tolerance: f64,
) -> Result<(Vec<f64>, bool, usize), SignalError> {
    let n = design_matrix.len();
    if n == 0 {
        return Err(SignalError::InsufficientData("Empty design matrix".into()));
    }
    let p = design_matrix[0].len();
    if p == 0 {
        return Err(SignalError::invalid_covariate_matrix(
            "No features in design matrix",
        ));
    }

    // Initialize coefficients to zero
    let mut beta: Vec<f64> = vec![0.0; p];

    // Convert outcome to f64
    let y: Vec<f64> = outcome.iter().map(|&o| if o { 1.0 } else { 0.0 }).collect();

    let mut converged = false;
    let mut iterations = 0;

    for iter in 0..max_iterations {
        iterations = iter + 1;

        // Calculate predicted probabilities: p = sigmoid(X * beta)
        let mut probs: Vec<f64> = Vec::with_capacity(n);
        for row in design_matrix {
            let linear: f64 = row.iter().zip(&beta).map(|(x, b)| x * b).sum();
            probs.push(sigmoid(linear));
        }

        // Calculate gradient: X'(y - p)
        let mut gradient: Vec<f64> = vec![0.0; p];
        for i in 0..n {
            let residual = y[i] - probs[i];
            for j in 0..p {
                gradient[j] += design_matrix[i][j] * residual;
            }
        }

        // Calculate Hessian diagonal approximation: -X'WX ≈ -Σ p(1-p) x_j²
        // Use diagonal approximation for efficiency
        let mut hessian_diag: Vec<f64> = vec![0.0; p];
        for i in 0..n {
            let w = probs[i] * (1.0 - probs[i]) + 1e-10; // Small constant for stability
            for j in 0..p {
                hessian_diag[j] -= w * design_matrix[i][j] * design_matrix[i][j];
            }
        }

        // Newton step: beta_new = beta - H^{-1} * gradient
        let mut max_change: f64 = 0.0;
        for j in 0..p {
            if hessian_diag[j].abs() > 1e-10 {
                let delta = -gradient[j] / hessian_diag[j];
                // Damping for stability
                let clamped_delta = delta.max(-5.0).min(5.0);
                beta[j] += clamped_delta;
                max_change = max_change.max(clamped_delta.abs());
            }
        }

        // Check convergence
        if max_change < tolerance {
            converged = true;
            break;
        }
    }

    Ok((beta, converged, iterations))
}

/// Sigmoid function with numerical stability.
fn sigmoid(x: f64) -> f64 {
    if x >= 0.0 {
        let exp_neg_x = (-x).exp();
        1.0 / (1.0 + exp_neg_x)
    } else {
        let exp_x = x.exp();
        exp_x / (1.0 + exp_x)
    }
}

/// Calculate propensity scores from design matrix and coefficients.
fn calculate_propensity_scores(design_matrix: &[Vec<f64>], coefficients: &[f64]) -> Vec<f64> {
    design_matrix
        .iter()
        .map(|row| {
            let linear: f64 = row.iter().zip(coefficients).map(|(x, b)| x * b).sum();
            sigmoid(linear)
        })
        .collect()
}

/// Calculate C-statistic (concordance / AUC) for propensity model.
fn calculate_c_statistic(propensity_scores: &[f64], exposure: &[bool]) -> f64 {
    let exposed: Vec<f64> = propensity_scores
        .iter()
        .zip(exposure)
        .filter(|(_, e)| **e)
        .map(|(&p, _)| p)
        .collect();
    let unexposed: Vec<f64> = propensity_scores
        .iter()
        .zip(exposure)
        .filter(|(_, e)| !**e)
        .map(|(&p, _)| p)
        .collect();

    if exposed.is_empty() || unexposed.is_empty() {
        return 0.5; // No discrimination possible
    }

    // Count concordant pairs
    let mut concordant = 0_usize;
    let mut ties = 0_usize;
    let total = exposed.len() * unexposed.len();

    for &p1 in &exposed {
        for &p0 in &unexposed {
            if p1 > p0 {
                concordant += 1;
            } else if (p1 - p0).abs() < 1e-10 {
                ties += 1;
            }
        }
    }

    // C = (concordant + 0.5 * ties) / total
    (concordant as f64 + 0.5 * ties as f64) / total as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> (Vec<CovariateData>, Vec<bool>, Vec<bool>) {
        // 20 subjects: 10 exposed, 10 unexposed
        let exposure: Vec<bool> = (0..20).map(|i| i < 10).collect();

        // Outcome: higher in exposed (8/10 vs 2/10)
        let outcome: Vec<bool> = vec![
            true, true, true, true, true, true, true, true, false, false, // exposed
            true, true, false, false, false, false, false, false, false, false, // unexposed
        ];

        // Confounder 1: associated with both exposure and outcome
        let cov1 = CovariateData::new(
            "dx".into(),
            "diabetes".into(),
            vec![
                true, true, true, true, true, true, true, false, false, false, true, false, false,
                false, false, false, false, false, false, false,
            ],
        );

        // Confounder 2: also associated
        let cov2 = CovariateData::new(
            "dx".into(),
            "hypertension".into(),
            vec![
                true, true, true, true, true, false, false, false, false, false, false, false,
                false, false, false, false, false, false, false, false,
            ],
        );

        // Noise variable: not associated with outcome
        let cov3 = CovariateData::new(
            "rx".into(),
            "aspirin".into(),
            vec![
                true, false, true, false, true, false, true, false, true, false, true, false, true,
                false, true, false, true, false, true, false,
            ],
        );

        (vec![cov1, cov2, cov3], exposure, outcome)
    }

    #[test]
    fn test_covariate_prevalence() {
        let cov = CovariateData::new(
            "test".into(),
            "var1".into(),
            vec![true, true, false, false, false],
        );

        assert!((cov.prevalence() - 0.4).abs() < 0.01);

        let indices = vec![0, 1, 2];
        assert!((cov.prevalence_in(&indices) - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_hdps_basic() {
        let (covariates, exposure, outcome) = create_test_data();
        let config = HDPSConfig {
            top_k_per_dimension: 10,
            total_covariates: 10,
            min_prevalence: 0.05,
            max_prevalence: 0.95,
            min_count: 2,
            ..Default::default()
        };

        let result = calculate_hdps(&covariates, &exposure, &outcome, &config);
        assert!(result.is_ok(), "HDPS should succeed: {:?}", result.err());

        let result = result.unwrap();
        assert!(!result.selected_covariates.is_empty());
        assert_eq!(result.propensity_scores.len(), 20);
        assert!(result.c_statistic >= 0.5 && result.c_statistic <= 1.0);
    }

    #[test]
    fn test_hdps_empty_input() {
        let config = HDPSConfig::default();

        // Empty covariates
        let result = calculate_hdps(&[], &[true, false], &[true, false], &config);
        assert!(result.is_err());

        // Empty exposure
        let cov = CovariateData::new("test".into(), "var".into(), vec![]);
        let result = calculate_hdps(&[cov], &[], &[], &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_hdps_dimension_mismatch() {
        let config = HDPSConfig::default();
        let cov = CovariateData::new("test".into(), "var".into(), vec![true, false, true]);
        let exposure = vec![true, false]; // 2 subjects
        let outcome = vec![true, false];

        let result = calculate_hdps(&[cov], &exposure, &outcome, &config);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SignalError::InvalidCovariateMatrix(_)
        ));
    }

    #[test]
    fn test_hdps_propensity_deciles() {
        let (covariates, exposure, outcome) = create_test_data();
        let config = HDPSConfig {
            top_k_per_dimension: 10,
            total_covariates: 10,
            min_prevalence: 0.05,
            max_prevalence: 0.95,
            min_count: 2,
            ..Default::default()
        };

        let result = calculate_hdps(&covariates, &exposure, &outcome, &config).unwrap();
        let deciles = result.propensity_deciles();

        assert_eq!(deciles.len(), 10);
        // Each decile should have lower <= upper
        for (lower, upper) in &deciles {
            assert!(lower <= upper);
        }
    }

    #[test]
    fn test_sigmoid() {
        assert!((sigmoid(0.0) - 0.5).abs() < 0.001);
        assert!(sigmoid(10.0) > 0.999);
        assert!(sigmoid(-10.0) < 0.001);
        // Numerical stability for extreme values
        assert!(sigmoid(100.0).is_finite());
        assert!(sigmoid(-100.0).is_finite());
    }

    #[test]
    fn test_c_statistic_perfect() {
        // Perfect discrimination
        let scores = vec![0.9, 0.8, 0.7, 0.3, 0.2, 0.1];
        let exposure = vec![true, true, true, false, false, false];

        let c = calculate_c_statistic(&scores, &exposure);
        assert!(
            (c - 1.0).abs() < 0.01,
            "C-statistic should be ~1.0 for perfect discrimination"
        );
    }

    #[test]
    fn test_c_statistic_random() {
        // No discrimination
        let scores = vec![0.5, 0.5, 0.5, 0.5, 0.5, 0.5];
        let exposure = vec![true, true, true, false, false, false];

        let c = calculate_c_statistic(&scores, &exposure);
        assert!(
            (c - 0.5).abs() < 0.01,
            "C-statistic should be ~0.5 for no discrimination"
        );
    }

    #[test]
    fn test_selected_covariate_ranking() {
        let (covariates, exposure, outcome) = create_test_data();
        let config = HDPSConfig {
            top_k_per_dimension: 10,
            total_covariates: 10,
            min_prevalence: 0.05,
            max_prevalence: 0.95,
            min_count: 2,
            ..Default::default()
        };

        let result = calculate_hdps(&covariates, &exposure, &outcome, &config).unwrap();

        // Check ranks are sequential
        for (i, cov) in result.selected_covariates.iter().enumerate() {
            assert_eq!(cov.rank, i + 1);
        }

        // Check bias scores are in descending order
        for i in 1..result.selected_covariates.len() {
            assert!(
                result.selected_covariates[i - 1].bias_score
                    >= result.selected_covariates[i].bias_score,
                "Covariates should be sorted by bias score"
            );
        }
    }
}
