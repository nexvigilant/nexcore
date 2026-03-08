//! # Signal Detection Algorithms
//!
//! Comprehensive pharmacovigilance signal detection algorithms.
//!
//! This module is consolidated from the standalone `nexcore-signals` crate.
//!
//! ## Module Organization
//!
//! - **`core`** - Shared types, newtypes, errors, and statistical helpers
//! - **`disproportionality`** - Frequentist methods (PRR, ROR)
//! - **`bayesian`** - Bayesian methods (IC, BCPNN, EBGM)
//! - **`sequential`** - Sequential testing (SPRT, MaxSPRT, CuSum)
//! - **`batch`** - High-performance parallel processing
//! - **`temporal`** - Time-based analysis (Weibull TTO, ARIMA)
//! - **`propensity`** - Propensity methods (HDPS)
//! - **`adjustment`** - Multiple testing correction
//! - **`classification`** - Severity assessment
//! - **`survival`** - Survival analysis (Kaplan-Meier, Cox)
//!
//! ## Signal Thresholds (Evans criteria)
//!
//! - PRR >= 2.0
//! - Chi-square >= 3.841 (p < 0.05)
//! - Minimum 3 cases

use serde::{Deserialize, Serialize};

// =============================================================================
// SUBMODULES
// =============================================================================

/// Core infrastructure: types, newtypes, errors, statistics
pub mod core;

/// Frequentist disproportionality methods (PRR, ROR)
pub mod disproportionality;

/// Bayesian signal detection methods (IC, BCPNN, EBGM)
pub mod bayesian;

/// Sequential testing methods (SPRT, MaxSPRT, CuSum)
pub mod sequential;

/// High-performance batch processing
pub mod batch;

/// Temporal analysis (Weibull TTO, ARIMA)
pub mod temporal;

/// Propensity score methods (HDPS)
pub mod propensity;

/// Multiple testing correction
pub mod adjustment;

/// Severity classification
pub mod classification;

/// Survival analysis
pub mod survival;

/// Theory of Vigilance (ToV) universal signal detection
pub mod tov;

/// ToV Signal Equation (S = U × R × T) - Codex-compliant types (§20-§23)
pub mod signal_equation;

/// Signal detection atoms (vigilance-specific L1 primitives)
pub mod atoms;

/// Safe signal detector (vigilance-specific L2 molecule)
pub mod safe_detector;

/// Fisher exact test
pub mod fisher;

/// Chi-square calculations (compatibility)
pub mod chi_square;

/// Grounded signal detection — Uncertain<T>-returning wrappers
pub mod grounded_signals;

// =============================================================================
// TOP-LEVEL RE-EXPORTS
// =============================================================================

// Core types
pub use core::error::SignalError;
pub use core::newtypes;
pub use core::stats::{
    chi_square_p_value, chi_square_statistic, chi_square_yates_corrected, z_score_for_confidence,
};
pub use core::types::{
    ContingencyTable, DisproportionalityResult, SignalCriteria, SignalMethod, SignalResult,
};

// =============================================================================
// MODULE ALIASES (Backwards Compatibility)
// =============================================================================

/// PRR module alias
pub mod prr {
    //! PRR (Proportional Reporting Ratio)
    pub use super::disproportionality::prr::*;
}

/// ROR module alias
pub mod ror {
    //! ROR (Reporting Odds Ratio)
    pub use super::disproportionality::ror::*;
}

/// IC module alias
pub mod ic {
    //! IC (Information Component)
    pub use super::bayesian::ic::*;
}

/// BCPNN module alias
pub mod bcpnn {
    //! BCPNN (Bayesian Confidence Propagation Neural Network)
    pub use super::bayesian::bcpnn::*;
}

/// EBGM module alias
pub mod ebgm {
    //! EBGM (Empirical Bayes Geometric Mean)
    pub use super::bayesian::ebgm::*;
}

/// GPS is an alias for EBGM (same algorithm)
pub mod gps {
    //! GPS (Gamma Poisson Shrinker) - alias for EBGM
    pub use super::bayesian::ebgm::*;
}

/// SPRT module alias
pub mod sprt {
    //! SPRT (Sequential Probability Ratio Test)
    pub use super::sequential::sprt::*;
}

/// MaxSPRT module alias
pub mod maxsprt {
    //! MaxSPRT (Maximized SPRT)
    pub use super::sequential::maxsprt::*;
}

/// CuSum module alias
pub mod cusum {
    //! CuSum (Cumulative Sum)
    pub use super::sequential::cusum::*;
}

/// Weibull TTO module alias
pub mod weibull {
    //! Weibull Time-to-Onset analysis
    pub use super::temporal::weibull::*;
}

/// HDPS module alias
pub mod hdps {
    //! HDPS (High-Dimensional Propensity Score)
    pub use super::propensity::hdps::*;
}

/// CSSP module alias
pub mod cssp {
    //! CSSP (Continuous Sequential Sampling Procedure)
    pub use super::sequential::cssp::*;
}

/// ARIMA module alias
pub mod arima {
    //! ARIMA time-series analysis
    pub use super::temporal::arima::*;
}

/// Kaplan-Meier module alias
pub mod km {
    //! Kaplan-Meier survival estimator
    pub use super::survival::kaplan_meier::*;
}

// =============================================================================
// DIRECT RE-EXPORTS
// =============================================================================

// SPRT types
pub use sequential::sprt::{SprtConfig, SprtDecision, SprtMonitor};

// MaxSPRT types and functions
pub use sequential::maxsprt::{
    MaxSprtConfig, MaxSprtDecision, MaxSprtResult, batch_maxsprt_parallel, calculate_maxsprt,
};

// CuSum types and functions
pub use sequential::cusum::{
    CuSumConfig, CuSumDirection, CuSumResult, batch_cusum_parallel, calculate_cusum,
};

// EBGM types
pub use bayesian::ebgm::{MGPSPriors, calculate_ebgm_with_priors};

// Batch processing
pub use batch::{
    BatchContingencyTables, BatchResult, CompleteSignalResult, batch_chi_square_p_values,
    batch_chi_square_p_values_sequential, batch_complete_parallel,
    batch_ebgm_custom_priors_parallel, batch_ebgm_parallel, batch_ic_parallel, batch_prr_parallel,
    batch_prr_vectorized, batch_ror_parallel, build_contingency_tables,
    build_contingency_tables_parallel,
};

// Temporal analysis types and functions
pub use temporal::{
    WeibullShape, WeibullTTOConfig, WeibullTTOResult, batch_weibull_parallel, fit_weibull_tto,
};

// Propensity score types and functions
pub use propensity::{CovariateData, HDPSConfig, HDPSResult, SelectedCovariate, calculate_hdps};

// Multiple testing adjustment
pub use adjustment::{
    BHResult, BonferroniMethod, BonferroniResult, bh_adjust, bh_reject, bonferroni_adjust,
    bonferroni_threshold, compute_q_values, holm_adjust, sidak_adjust,
};

// Omega shrinkage for DDI detection
pub use bayesian::omega_shrinkage::{
    DDITable, OmegaConfig, OmegaResult, calculate_omega, calculate_omega_interaction,
};

// Severity classification
pub use classification::{
    SeverityAssessment, SeverityCategory, SeverityCriteria, SeverityLevel, assess_severity,
    batch_severity_score, full_assessment,
};

// Additional disproportionality methods
pub use disproportionality::{
    AssociationStrength, NPRRExtended, YulesQResult, calculate_nprr, calculate_yules_q, or_to_q,
    q_to_or,
};

// CSSP sequential surveillance
pub use sequential::cssp::{
    CsspConfig, CsspDecision, CsspMonitor, CsspObservation, CsspResult, SpendingFunction,
    batch_cssp_parallel, calculate_cssp,
};

// ARIMA time series analysis
pub use temporal::arima::{
    ArimaAnomalyResult, ArimaConfig, ArimaForecast, ArimaResult, batch_arima_parallel,
    detect_anomalies, fit_arima, forecast_arima,
};

// Survival analysis - Kaplan-Meier
pub use survival::kaplan_meier::kaplan_meier as compute_kaplan_meier;
pub use survival::{KaplanMeierResult, SurvivalObservation, SurvivalPoint, log_rank_test};

// Top-level convenience re-exports for common functions
pub use bayesian::bcpnn::calculate_bcpnn;
pub use bayesian::ebgm::calculate_ebgm;
pub use bayesian::ic::calculate_ic;
pub use disproportionality::prr::calculate_prr;
pub use disproportionality::ror::calculate_ror;
pub use fisher::{FisherResult, fisher_exact_test};

// ToV Signal Equation types (§20-§23) - Codex compliant
pub use signal_equation::{
    RecognitionPresence, SignalComponentError, SignalEquationResult, SignalStrength,
    TemporalWindow, UnrepeatablePattern,
};

// =============================================================================
// FDR / MULTIPLE TESTING CONFIGURATION
// =============================================================================

/// Method for multiple testing correction.
///
/// Used to select which adjustment procedure to apply when correcting
/// p-values across multiple drug-event pair evaluations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdjustmentMethod {
    /// Benjamini-Hochberg — controls FDR (default for screening)
    BenjaminiHochberg,
    /// Bonferroni — controls FWER (conservative)
    Bonferroni,
    /// Holm step-down — controls FWER (more powerful than Bonferroni)
    Holm,
    /// Šidák — controls FWER (assumes independence)
    Sidak,
    /// No correction applied
    None,
}

impl Default for AdjustmentMethod {
    fn default() -> Self {
        Self::BenjaminiHochberg
    }
}

/// Configuration for signal evaluation with optional FDR correction.
///
/// When `fdr_correction` is enabled, frequentist methods (PRR, ROR) have their
/// p-values adjusted for multiple comparisons. Bayesian methods (BCPNN, EBGM,
/// IC, Omega) are NOT adjusted — they have built-in shrinkage that implicitly
/// controls false discoveries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalEvaluationConfig {
    /// Apply FDR correction to frequentist methods
    pub fdr_correction: bool,
    /// FDR level (default 0.05)
    pub fdr_level: f64,
    /// Which adjustment method to use
    pub adjustment_method: AdjustmentMethod,
}

impl Default for SignalEvaluationConfig {
    fn default() -> Self {
        Self {
            fdr_correction: false,
            fdr_level: 0.05,
            adjustment_method: AdjustmentMethod::BenjaminiHochberg,
        }
    }
}

impl SignalEvaluationConfig {
    /// Configuration for batch processing — FDR on by default.
    ///
    /// When screening 100K+ drug-event pairs, multiplicity correction
    /// is the pharmacostatistically correct default.
    #[must_use]
    pub fn batch_default() -> Self {
        Self {
            fdr_correction: true,
            fdr_level: 0.05,
            adjustment_method: AdjustmentMethod::BenjaminiHochberg,
        }
    }
}

// =============================================================================
// CONSTANTS
// =============================================================================

/// Evans criteria: PRR threshold
pub const EVANS_PRR_THRESHOLD: f64 = 2.0;
/// Evans criteria: Chi-square threshold for p < 0.05 (1 df)
pub const EVANS_CHI_SQUARE_THRESHOLD: f64 = 3.841;
/// Evans criteria: Minimum case count
pub const EVANS_MIN_CASES: u32 = 3;
/// Z-score for 95% confidence interval
pub const Z_95: f64 = 1.96;

// FDR batch processing
pub use batch::parallel::{BatchAdjustmentMetadata, BatchFdrResults, batch_complete_with_fdr};

// =============================================================================
// CONVENIENCE FUNCTIONS
// =============================================================================

/// Evaluate signal using all methods and return combined result.
///
/// Returns results from all 5 disproportionality algorithms:
/// PRR, ROR, IC, BCPNN, and EBGM.
pub fn evaluate_signal_all(
    table: &ContingencyTable,
    criteria: &SignalCriteria,
) -> Result<Vec<DisproportionalityResult>, SignalError> {
    let prr_result = disproportionality::prr::calculate_prr(table, criteria)?;
    let ror_result = disproportionality::ror::calculate_ror(table, criteria)?;
    let ic_result = bayesian::ic::calculate_ic(table, criteria)?;
    let bcpnn_result = bayesian::bcpnn::calculate_bcpnn(table, criteria)?;
    let ebgm_result = bayesian::ebgm::calculate_ebgm(table, criteria)?;

    Ok(vec![
        prr_result.into(),
        ror_result.into(),
        ic_result.into(),
        bcpnn_result.into(),
        ebgm_result.into(),
    ])
}

/// Evaluate signal using PRR, ROR, IC, EBGM and return complete result.
///
/// This is the simplified version returning `crate::types::CompleteSignalResult`
/// for compatibility with vigilance-specific modules.
#[must_use]
pub fn evaluate_signal_complete(
    table: &crate::types::ContingencyTable,
    _criteria: &crate::thresholds::SignalCriteria,
) -> crate::types::CompleteSignalResult {
    // Use the new algorithm implementations
    let table_core = ContingencyTable::new(table.a, table.b, table.c, table.d);
    let criteria_core = SignalCriteria::evans(); // Default criteria

    // Calculate each algorithm
    let prr = disproportionality::prr::calculate_prr(&table_core, &criteria_core)
        .unwrap_or_else(|_| SignalResult::null(SignalMethod::Prr, table.a, table_core.total()));

    let ror = disproportionality::ror::calculate_ror(&table_core, &criteria_core)
        .unwrap_or_else(|_| SignalResult::null(SignalMethod::Ror, table.a, table_core.total()));

    let ic = bayesian::ic::calculate_ic(&table_core, &criteria_core)
        .unwrap_or_else(|_| SignalResult::null(SignalMethod::Ic, table.a, table_core.total()));

    let ebgm = bayesian::ebgm::calculate_ebgm(&table_core, &criteria_core)
        .unwrap_or_else(|_| SignalResult::null(SignalMethod::Ebgm, table.a, table_core.total()));

    let chi_square = core::stats::chi_square_statistic(
        table.a as f64,
        table.b as f64,
        table.c as f64,
        table.d as f64,
    );

    crate::types::CompleteSignalResult {
        prr,
        ror,
        ic,
        ebgm,
        chi_square,
        n: table.a,
    }
}
