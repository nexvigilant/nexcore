//! Bayesian disproportionality methods: IC and EBGM.

pub mod ebgm;
pub mod ic;

pub use ebgm::{MgpsPriors, calculate_ebgm, calculate_ebgm_with_priors, eb05, ebgm_only};
pub use ic::{calculate_ic, ic_only, ic025};
