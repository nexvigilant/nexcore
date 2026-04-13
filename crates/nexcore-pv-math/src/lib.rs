//! # nexcore-pv-math
//!
//! WASM-compilable PV signal detection and causality assessment mathematics.
//!
//! Zero networking dependencies — no tokio, no axum, no reqwest, no mio.
//! Compiles to `wasm32-unknown-unknown` for browser-side PV computation.
//!
//! ## Signal Detection
//!
//! | Method  | Type         | Signal Criterion                        |
//! |---------|-------------|------------------------------------------|
//! | PRR     | Frequentist  | PRR ≥ 2.0, χ² ≥ 3.841, n ≥ 3           |
//! | ROR     | Frequentist  | ROR ≥ 2.0, lower CI ≥ 1.0, n ≥ 3       |
//! | IC      | Bayesian     | IC025 > 0.0, n ≥ 3                      |
//! | EBGM    | Bayesian     | EBGM ≥ 2.0, EB05 ≥ 2.0, n ≥ 3          |
//! | χ²      | Frequentist  | p < 0.05 (statistic ≥ 3.841)            |
//! | Fisher  | Exact        | one-tailed p < 0.05                     |
//!
//! ## Causality Assessment
//!
//! | Algorithm | Use Case          | Range      |
//! |-----------|-------------------|------------|
//! | Naranjo   | General ADR       | −4 to +13  |
//! | WHO-UMC   | Global PV standard | 6 categories |
//!
//! ## Example
//!
//! ```rust
//! use nexcore_pv_math::{TwoByTwoTable, SignalCriteria, calculate_prr};
//!
//! let table = TwoByTwoTable::new(10, 90, 100, 9800);
//! let result = calculate_prr(&table, &SignalCriteria::evans()).unwrap();
//! assert!(result.is_signal);
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

pub mod causality;
pub mod error;
pub mod signals;
pub mod stats;
pub mod types;

// ---------------------------------------------------------------------------
// Top-level re-exports — the public API surface
// ---------------------------------------------------------------------------

// Core types
pub use types::{SignalCriteria, TwoByTwoTable};

// nexcore-signal-types pass-throughs (consumers get everything from one crate)
pub use nexcore_signal_types::{SignalMethod, SignalResult};

// Signal detection
pub use signals::bayesian::{
    MgpsPriors, calculate_ebgm, calculate_ebgm_with_priors, calculate_ic, eb05, ebgm_only, ic_only,
    ic025,
};
pub use signals::chi_square::{ChiSquare, calculate_chi_square};
pub use signals::disproportionality::{calculate_prr, calculate_ror, prr_only, ror_only};
pub use signals::fisher::{FisherResult, fisher_exact_test};

// Causality
pub use causality::{
    NaranjoInput, NaranjoResult, NaranjoScore, WhoUmcCategory, WhoUmcResult, calculate_naranjo,
    calculate_naranjo_quick, calculate_who_umc_quick,
};

// Error type
pub use error::PvMathError;
