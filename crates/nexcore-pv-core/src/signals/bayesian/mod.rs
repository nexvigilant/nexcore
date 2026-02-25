//! Bayesian methods for signal detection.
//!
//! This module contains Bayesian approaches that use prior distributions
//! to provide shrinkage estimation, which is particularly useful for
//! sparse data and multiple comparisons.
//!
//! ## Methods
//!
//! - **IC** (Information Component) - WHO-UMC log-ratio with shrinkage
//! - **BCPNN** (Bayesian Confidence Propagation Neural Network) - WHO-UMC method
//! - **EBGM** (Empirical Bayes Geometric Mean) - DuMouchel's MGPS
//! - **Omega-Shrinkage** - Drug-drug interaction (DDI) detection
//!
//! ## Signal Criteria
//!
//! - IC025 > 0 (lower 95% credibility interval above zero)
//! - EB05 >= 2.0 for EBGM
//! - Ω025 > 0 for DDI signals
//! - Minimum 3 cases

pub mod bcpnn;
pub mod ebgm;
pub mod ic;
pub mod measured;
pub mod omega_shrinkage;
pub mod update;

// Re-export main functions at module level
pub use bcpnn::{calculate_bcpnn, calculate_ic025, is_bcpnn_signal};
pub use ebgm::{MGPSPriors, calculate_ebgm, calculate_ebgm_with_priors, eb05, ebgm_only};
pub use ic::{calculate_ic, ic_only, ic025};
pub use measured::{bcpnn_measured, ebgm_measured, ic_measured, omega_measured};
pub use omega_shrinkage::{
    DDITable, OmegaConfig, OmegaResult, calculate_omega, calculate_omega_interaction,
};
pub use update::{
    BayesianUpdate, BetaParams, BinomialEvidence, ConjugateBetaBinomial, GammaParams,
    GammaPoissonMixture, PoissonEvidence,
};
