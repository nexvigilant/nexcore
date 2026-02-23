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

pub mod error;
pub mod explicit;
pub mod grounding;
pub mod l_functions;
pub mod riemann_siegel;
pub mod special;
pub mod zeros;
pub mod zeta;

pub use error::ZetaError;
pub use explicit::{convergence_series, explicit_psi, explicit_psi_comparison};
pub use l_functions::DirichletCharacter;
pub use zeros::{RhVerification, ZetaZero};
