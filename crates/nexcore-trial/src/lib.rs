#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![forbid(unsafe_code)]
//! # nexcore-trial
//!
//! Universal experimentation framework derived from FDA clinical trial methodology.
//! Implements TRIAL (T-R-I-A-L) protocol: Target, Regiment, Interim, Assay, Lifecycle.

pub mod error;
pub mod power;
pub mod protocol;
pub mod types;

pub use error::TrialError;
pub use power::{sample_size_survival, sample_size_two_mean, sample_size_two_proportion};
pub use protocol::register_protocol;
pub use types::*;
