//! Error types for signal detection algorithms.

use thiserror::Error;

/// Errors that can occur during signal detection.
#[derive(Debug, Error)]
pub enum SignalError {
    /// Contingency table values are invalid.
    #[error("Invalid contingency table: {0}")]
    InvalidContingencyTable(String),

    /// Zero or negative expected count.
    #[error("Invalid expected count: {0}")]
    InvalidExpectedCount(f64),

    /// Mathematical operation failed.
    #[error("Math error: {0}")]
    MathError(String),

    /// Insufficient data for calculation.
    #[error("Insufficient data: {0}")]
    InsufficientData(String),

    // =========================================================================
    // Temporal Analysis Errors (Phase 1)
    // =========================================================================
    /// Weibull distribution fitting failed to converge.
    #[error("Weibull fit failed: {0}")]
    WeibullFitError(String),

    /// Time data contains invalid values (negative, NaN, or infinite).
    #[error("Invalid time data: {0}")]
    InvalidTimeData(String),

    // =========================================================================
    // Propensity Score Errors (Phase 1)
    // =========================================================================
    /// Insufficient covariates for HDPS calculation.
    #[error("Insufficient covariates: {0}")]
    InsufficientCovariates(String),

    /// Covariate matrix is invalid (dimension mismatch, singular, etc.).
    #[error("Invalid covariate matrix: {0}")]
    InvalidCovariateMatrix(String),

    /// Propensity score estimation failed to converge.
    #[error("Propensity estimation failed: {0}")]
    PropensityEstimationError(String),
}

impl SignalError {
    /// Create an invalid contingency table error.
    pub fn invalid_table(msg: impl Into<String>) -> Self {
        Self::InvalidContingencyTable(msg.into())
    }

    /// Create a math error.
    pub fn math_error(msg: impl Into<String>) -> Self {
        Self::MathError(msg.into())
    }

    /// Create a Weibull fit error.
    pub fn weibull_fit(msg: impl Into<String>) -> Self {
        Self::WeibullFitError(msg.into())
    }

    /// Create an invalid time data error.
    pub fn invalid_time(msg: impl Into<String>) -> Self {
        Self::InvalidTimeData(msg.into())
    }

    /// Create an insufficient covariates error.
    pub fn insufficient_covariates(msg: impl Into<String>) -> Self {
        Self::InsufficientCovariates(msg.into())
    }

    /// Create an invalid covariate matrix error.
    pub fn invalid_covariate_matrix(msg: impl Into<String>) -> Self {
        Self::InvalidCovariateMatrix(msg.into())
    }
}
