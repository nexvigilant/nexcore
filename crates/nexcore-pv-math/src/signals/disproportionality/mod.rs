//! Frequentist disproportionality methods: PRR and ROR.

pub mod prr;
pub mod ror;

pub use prr::{calculate_prr, prr_only};
pub use ror::{calculate_ror, ror_only};
