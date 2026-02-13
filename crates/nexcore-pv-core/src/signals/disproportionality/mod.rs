//! Frequentist disproportionality methods for signal detection.
//!
//! This module contains ratio-based measures that compare observed vs expected
//! drug-event associations using classical statistical methods.
//!
//! ## Methods
//!
//! - **PRR** (Proportional Reporting Ratio) - Evans et al. 2001
//! - **ROR** (Reporting Odds Ratio) - Case-control approach
//! - **Yule's Q** - Normalized odds ratio [-1, +1]
//! - **NPRR** - Normalized PRR for cross-event comparison
//!
//! ## Signal Criteria
//!
//! Most methods use Evans criteria:
//! - PRR/ROR >= 2.0
//! - Chi-square >= 3.841 (p < 0.05)
//! - Minimum 3 cases

pub mod nprr;
pub mod prr;
pub mod ror;
pub mod yules_q;

// Re-export main functions at module level
pub use nprr::{NPRRExtended, calculate_nprr, calculate_nprr_extended};
pub use prr::{calculate_prr, prr_only};
pub use ror::{calculate_ror, ror_only};
pub use yules_q::{AssociationStrength, YulesQResult, calculate_yules_q, or_to_q, q_to_or};
