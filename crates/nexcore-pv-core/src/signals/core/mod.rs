//! Core infrastructure for signal detection.
//!
//! This module contains shared types, newtypes, error handling, and statistical
//! helper functions used across all signal detection algorithms.

pub mod error;
pub mod newtypes;
pub mod stats;
pub mod types;

pub use error::SignalError;
pub use newtypes::{ChiSquare, Ebgm, Ic, MetricError, Prr, Ror};
pub use stats::{
    CHI_SQUARE_CRITICAL_05, LN_2, Z_95, apply_continuity_correction, chi_square_p_value,
    chi_square_statistic, chi_square_yates_corrected, digamma, gamma, log_gamma,
    log_ratio_standard_error, log2, normal_cdf, normal_quantile, z_score_for_confidence,
};
pub use types::{ContingencyTable, DisproportionalityResult, SignalCriteria, SignalResult};
