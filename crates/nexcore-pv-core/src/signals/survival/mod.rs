//! Survival Analysis Methods for Pharmacovigilance
//!
//! This module provides survival analysis methods for time-to-event
//! data in drug safety, including non-parametric estimators,
//! regression models for hazard ratio estimation, cumulative incidence,
//! and Measured<T> confidence-propagating wrappers.
//!
//! ## Methods
//!
//! - **Kaplan-Meier** - Non-parametric survival curve estimation with
//!   Greenwood's variance formula and log-rank testing
//! - **Cox PH** - Proportional hazards regression for covariate-adjusted
//!   hazard ratio estimation using partial likelihood
//! - **Cumulative Incidence** - Complement of KM: CI(t) = 1 - S(t)
//! - **Measured Wrappers** - Confidence-propagating outputs via Measured<T>
//!
//! ## Use Cases
//!
//! - Comparing time-to-event between treatment and control groups
//! - Estimating drug-associated hazard ratios with confounding control
//! - Survival probability estimation at specific timepoints
//! - Statistical comparison of survival curves between groups
//! - Adverse event time-to-onset characterization (cumulative incidence)
//!
//! ## References
//!
//! - Kaplan EL, Meier P (1958). "Nonparametric estimation from incomplete observations."
//!   JASA 53(282):457-481. DOI: [10.2307/2281868](https://doi.org/10.2307/2281868)
//!
//! - Cox DR (1972). "Regression models and life-tables."
//!   JRSS-B 34(2):187-220. DOI: [10.1111/j.2517-6161.1972.tb00899.x](https://doi.org/10.1111/j.2517-6161.1972.tb00899.x)

// Cox regression — self-contained, no external dependencies (hand-rolled Newton-Raphson).
// Previously commented out with incorrect "requires nalgebra" note — verified clean 2026-02-24.
pub mod cox;

pub mod kaplan_meier;

pub mod cumulative_incidence;

pub mod measured;

// Reference validation tests (Freireich leukemia dataset)
#[cfg(test)]
mod reference_validation;

// Re-export main types and functions
pub use kaplan_meier::{
    KaplanMeierResult, SurvivalObservation, SurvivalPoint, kaplan_meier, log_rank_test,
};

pub use cox::{
    CoxCoefficient, CoxConfig, CoxObservation, CoxResult, TieMethod, fit_cox, quick_hazard_ratio,
};

pub use cumulative_incidence::{
    CumulativeIncidencePoint, CumulativeIncidenceResult, MeasuredCumulativeIncidence,
    cumulative_incidence, cumulative_incidence_measured,
};

pub use measured::{
    MeasuredCoxResult, MeasuredHazardRatio, MeasuredKaplanMeier, MeasuredLogRank, cox_measured,
    hazard_ratio_measured, kaplan_meier_measured, log_rank_measured,
};
