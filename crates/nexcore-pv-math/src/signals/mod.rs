//! Signal detection algorithms: frequentist, Bayesian, chi-square, Fisher.

pub mod bayesian;
pub mod chi_square;
pub mod disproportionality;
pub mod fisher;

pub use bayesian::{
    MgpsPriors, calculate_ebgm, calculate_ebgm_with_priors, calculate_ic, eb05, ebgm_only, ic_only,
    ic025,
};
pub use chi_square::{ChiSquare, calculate_chi_square};
pub use disproportionality::{calculate_prr, calculate_ror, prr_only, ror_only};
pub use fisher::{FisherResult, fisher_exact_test, log_hypergeometric_prob};
