//! Survival Analysis Methods for Pharmacovigilance
//!
//! This module provides survival analysis methods for time-to-event
//! data in drug safety, including non-parametric estimators and
//! regression models for hazard ratio estimation.
//!
//! ## Methods
//!
//! - **Kaplan-Meier** - Non-parametric survival curve estimation with
//!   Greenwood's variance formula and log-rank testing
//! - **Cox PH** - Proportional hazards regression for covariate-adjusted
//!   hazard ratio estimation using partial likelihood
//!
//! ## Use Cases
//!
//! - Comparing time-to-event between treatment and control groups
//! - Estimating drug-associated hazard ratios with confounding control
//! - Survival probability estimation at specific timepoints
//! - Statistical comparison of survival curves between groups
//!
//! ## References
//!
//! - Kaplan EL, Meier P (1958). "Nonparametric estimation from incomplete observations."
//!   JASA 53(282):457-481. DOI: [10.2307/2281868](https://doi.org/10.2307/2281868)
//!
//! - Cox DR (1972). "Regression models and life-tables."
//!   JRSS-B 34(2):187-220. DOI: [10.1111/j.2517-6161.1972.tb00899.x](https://doi.org/10.1111/j.2517-6161.1972.tb00899.x)

// Cox regression requires additional dependencies (nalgebra) - conditionally compiled
// TODO: Add "survival" feature to Cargo.toml when Cox regression is needed
// #[cfg(feature = "survival")]
// pub mod cox;

pub mod kaplan_meier;

// Re-export main types and functions
pub use kaplan_meier::{
    KaplanMeierResult, SurvivalObservation, SurvivalPoint, kaplan_meier, log_rank_test,
};

// #[cfg(feature = "survival")]
// pub use cox::{CoxConfig, CoxObservation, CoxResult, fit_cox, quick_hazard_ratio};
