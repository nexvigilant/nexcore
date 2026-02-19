#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]
//! # nexcore-trial
//!
//! Universal experimentation framework derived from FDA clinical trial methodology.
//! Implements TRIAL (T-R-I-A-L) protocol: Target, Regiment, Interim, Assay, Lifecycle.

pub mod error;
pub mod types;

pub use error::TrialError;
pub use types::*;
