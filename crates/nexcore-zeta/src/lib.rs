//! # nexcore-zeta
//!
//! Riemann Zeta function and Dirichlet L-function computation.
//!
//! ## Evaluation Strategies
//!
//! | Region | Method |
//! |--------|--------|
//! | Re(s) > 1 | Dirichlet series + Euler–Maclaurin |
//! | Critical strip | Dirichlet eta relation |
//! | Non-positive integers | Bernoulli closed form |
//! | Re(s) < 0 (non-integer) | Functional equation |
//! | Critical line | Riemann–Siegel Z function |
//!
//! ## Key Functions
//!
//! - [`zeta::zeta`] — main ζ(s) dispatcher
//! - [`riemann_siegel::riemann_siegel_z`] — Z(t) on the critical line
//! - [`zeros::find_zeros_bracket`] — locate zeros via sign changes
//! - [`zeros::verify_rh_to_height`] — verify RH up to height T
//! - [`l_functions::dirichlet_l`] — Dirichlet L-function L(s, χ)
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_zeta::zeta::zeta;
//! use stem_complex::Complex;
//! use std::f64::consts::PI;
//!
//! // ζ(2) = π²/6
//! let result = zeta(Complex::from(2.0)).unwrap();
//! assert!((result.re - PI * PI / 6.0).abs() < 1e-4);
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod adversarial;
pub mod anomaly;
pub mod batch;
pub mod cayley;
pub mod cmv;
pub mod convergence;
pub mod error;
pub mod explicit;
pub mod fingerprint;
pub mod grounding;
pub mod inverse;
pub mod killip_nenciu;
pub mod l_functions;
pub mod lmfdb;
pub mod operator;
pub mod pipeline;
pub mod prediction;
pub mod riemann_siegel;
pub mod scaling;
pub mod special;
pub mod statistics;
pub mod subseries;
pub mod zeros;
pub mod zeta;

pub use adversarial::{
    CounterexampleCandidate, DensityBand, DensityProfile, ExclusionAnalysis, SensitivityCurve,
    SensitivityPoint, analyze_exclusions, density_profile, sensitivity_curve,
};
pub use anomaly::{
    AnomalyDetail, AnomalyReport, PhaseModel, SpikeFeature, VerblunskyBaseline, build_baseline,
    detect_anomaly, inject_off_cl_zero,
};
pub use batch::{
    BatchConfig, BatchEntry, BatchReport, BatchStatistics, run_telescope_batch,
    run_telescope_batch_raw,
};
pub use cayley::{
    CayleyAnomalyReport, CayleyDeviation, CayleyTransform, cayley_anomaly_detect, cayley_transform,
};
pub use cmv::{CmvReconstruction, CmvStructure, reconstruct_cmv};
pub use convergence::{
    BootstrapResult, ConvergenceAnalysis, ExtendedConvergenceAnalysis, analyze_convergence,
    analyze_convergence_extended,
};
pub use error::ZetaError;
pub use explicit::{
    AdaptiveTruncation, ResidualByHeight, adaptive_truncation, convergence_series, explicit_psi,
    explicit_psi_comparison, residual_by_height,
};
pub use fingerprint::{
    FingerprintClass, FingerprintComparison, FingerprintDistance, SpectralFingerprint,
    compare_across_zero_sets, compare_fingerprints, fingerprint_l_function_zeros,
    fingerprint_zeros,
};
pub use inverse::{JacobiReconstruction, OperatorStructure, reconstruct_jacobi};
pub use killip_nenciu::{DualGueVerdict, KillipNenciuTest, compare_gue_tests, killip_nenciu_test};
pub use l_functions::DirichletCharacter;
pub use lmfdb::{
    LmfdbCatalog, LmfdbLfunction, LmfdbZeroSet, embedded_riemann_zeros, embedded_riemann_zeros_n,
    parse_lmfdb_api_response, parse_lmfdb_catalog, parse_lmfdb_labeled, parse_lmfdb_zeros,
};
pub use operator::{
    OperatorFit, OperatorHuntReport, berry_keating_xp, cmv_truncated_operator, hunt_operators,
    xp_plus_potential,
};
pub use pipeline::{TelescopeConfig, TelescopeReport, run_telescope};
pub use prediction::{
    PredictionAccuracy, VerblunskyModel, estimate_t_for_nth_zero, fit_verblunsky_model,
    predict_next_zeros, validate_prediction,
};
pub use scaling::{
    ScalingArm, ScalingComparison, ScalingLaw, ScalingPoint, compare_scaling_laws, fit_scaling_law,
    predict_confidence,
};
pub use statistics::{GueComparison, compare_to_gue};
pub use subseries::{
    CouplingExtrapolation, SubseriesAnalysis, analyze_subseries, extrapolate_coupling,
};
pub use zeros::{RhVerification, ZetaZero};
