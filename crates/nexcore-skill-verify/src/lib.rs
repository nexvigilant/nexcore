//! Skill verification infrastructure for Claude Code skills.
//!
//! Provides a Rust-based framework for verifying skill compliance,
//! replacing the Python VerifyBase system.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

mod check;
mod context;
mod error;
pub mod grounding;
mod reporter;
mod result;
mod verifier;

pub mod checks;

pub use check::{BoxedCheck, Check, FnCheck};
pub use context::VerifyContext;
pub use error::VerifyError;
pub use reporter::{Report, ReportFormat, ReportSummary};
pub use result::{CheckOutcome, CheckResult};
pub use verifier::Verifier;
