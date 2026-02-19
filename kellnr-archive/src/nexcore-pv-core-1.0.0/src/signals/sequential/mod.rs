//! Sequential Testing Methods for Drug Safety Surveillance
//!
//! This module contains sequential analysis methods that allow for
//! early stopping and continuous monitoring of drug safety signals.
//! These methods control Type I error while enabling interim analysis.
//!
//! ## Methods
//!
//! - **SPRT** (Sequential Probability Ratio Test) - Wald's classical method
//! - **MaxSPRT** (Maximized SPRT) - FDA Sentinel Initiative method
//! - **CuSum** (Cumulative Sum) - Control chart approach
//! - **CSSP** (Continuous Sequential Sampling Procedure) - Chronic exposure monitoring
//!
//! ## Use Cases
//!
//! - Real-time safety monitoring
//! - Surveillance of newly approved drugs
//! - Post-marketing commitment studies
//! - Vaccine safety monitoring
//! - Chronic medication surveillance
//!
//! ## References
//!
//! - Wald A (1947). "Sequential Analysis." Wiley, New York.
//! - Kulldorff M, et al. (2011). "A maximized sequential probability ratio test."
//!   Sequential Analysis 30(1):58-78.

pub mod cssp;
pub mod cusum;
pub mod maxsprt;
pub mod sprt;

// Re-export main types and functions
pub use cssp::{
    CsspConfig, CsspDecision, CsspMonitor, CsspObservation, CsspResult, SpendingFunction,
    batch_cssp_parallel, calculate_cssp,
};
pub use cusum::{CuSumConfig, CuSumDirection, CuSumResult, batch_cusum_parallel, calculate_cusum};
pub use maxsprt::{
    MaxSprtConfig, MaxSprtDecision, MaxSprtResult, batch_maxsprt_parallel, calculate_maxsprt,
};
pub use sprt::{SprtConfig, SprtDecision, SprtMonitor, expected_sample_size};
