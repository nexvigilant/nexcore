#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![forbid(unsafe_code)]
//! # nexcore-trial
//!
//! Universal experimentation framework derived from FDA clinical trial methodology.
//! Implements TRIAL (T-R-I-A-L) protocol: Target, Regiment, Interim, Assay, Lifecycle.

pub mod adaptation;
pub mod blinding;
pub mod endpoint;
pub mod error;
pub mod grounding;
pub mod interim;
pub mod multiplicity;
pub mod power;
pub mod protocol;
pub mod randomize;
pub mod report;
pub mod safety;
pub mod types;

pub use adaptation::evaluate_adaptation;
pub use blinding::verify_blinding;
pub use endpoint::{compute_nnt, evaluate_two_means, evaluate_two_proportions};
pub use error::TrialError;
pub use interim::{
    evaluate_interim, lan_demets_alpha_spent, obrien_fleming_boundary,
    posterior_probability_superiority,
};
pub use multiplicity::{
    benjamini_hochberg_adjust, bonferroni_adjust, hochberg_adjust, holm_adjust,
};
pub use power::{sample_size_survival, sample_size_two_mean, sample_size_two_proportion};
pub use protocol::register_protocol;
pub use randomize::{block_randomize, randomization_hash, simple_randomize, stratified_randomize};
pub use report::{determine_verdict, generate_report};
pub use safety::{check_safety_boundary, safety_event_rate};
pub use types::*;

// Lex Primitiva grounding
pub use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
pub use nexcore_lex_primitiva::tier::Tier;
pub use nexcore_primitives::{Confidence, Measured};
