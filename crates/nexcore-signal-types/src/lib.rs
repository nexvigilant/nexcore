//! Grounded signal types and methods.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

pub mod grounding;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Enumeration of supported signal detection methods.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SignalMethod {
    /// Proportional Reporting Ratio
    Prr = 1,
    /// Reporting Odds Ratio
    Ror = 2,
    /// Information Component (Bayesian)
    Ic = 3,
    /// Empirical Bayes Geometric Mean
    Ebgm = 4,
    /// Bayesian Confidence Propagation Neural Network
    Bcpnn = 5,
    /// Normalized PRR
    Nprr = 6,
    /// Yule's Q
    YulesQ = 7,
    /// Fisher's Exact Test
    Fisher = 8,
    /// Chi-Square Test
    ChiSquare = 9,
}

impl fmt::Display for SignalMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Prr => write!(f, "PRR"),
            Self::Ror => write!(f, "ROR"),
            Self::Ic => write!(f, "IC"),
            Self::Ebgm => write!(f, "EBGM"),
            Self::Bcpnn => write!(f, "BCPNN"),
            Self::Nprr => write!(f, "NPRR"),
            Self::YulesQ => write!(f, "Yule's Q"),
            Self::Fisher => write!(f, "Fisher"),
            Self::ChiSquare => write!(f, "Chi-Square"),
        }
    }
}

/// Result from a signal detection algorithm.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalResult {
    /// Algorithm method name
    pub method: SignalMethod,
    /// Point estimate (PRR, ROR, IC, or EBGM)
    pub point_estimate: f64,
    /// Lower confidence/credibility interval
    pub lower_ci: f64,
    /// Upper confidence/credibility interval
    pub upper_ci: f64,
    /// Chi-square statistic (for frequentist methods)
    pub chi_square: Option<f64>,
    /// Whether a signal is detected
    pub is_signal: bool,
    /// Case count (a)
    pub case_count: u64,
    /// Total reports (N)
    pub total_reports: u64,
}

impl SignalResult {
    /// Create a new signal result (legacy compatibility constructor).
    #[must_use]
    pub fn new(point_estimate: f64, lower_ci: f64, upper_ci: f64, is_signal: bool) -> Self {
        Self {
            method: SignalMethod::Prr,
            point_estimate,
            lower_ci,
            upper_ci,
            chi_square: None,
            is_signal,
            case_count: 0,
            total_reports: 0,
        }
    }

    /// Create a null result (no signal, zero values).
    #[must_use]
    pub fn null(method: SignalMethod, case_count: u64, total_reports: u64) -> Self {
        Self {
            method,
            point_estimate: 0.0,
            lower_ci: 0.0,
            upper_ci: 0.0,
            chi_square: None,
            is_signal: false,
            case_count,
            total_reports,
        }
    }
}
