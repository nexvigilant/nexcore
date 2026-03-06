//! # nexcore-combinatorics
//!
//! Combinatorial algorithms derived from cross-domain transfer analysis
//! of Dudeney's "Amusements in Mathematics" (1917).
//!
//! Each algorithm traces to a specific puzzle and T1 primitive composition:
//!
//! | Algorithm | Puzzle | Primitives |
//! |-----------|--------|------------|
//! | Catalan numbers | #279 Barrels of Balsam | σ+∂+κ |
//! | Derangements | #267 Wrong Hats | σ+κ+N |
//! | Cycle decomposition | #238 Jampots | σ+μ+N |
//! | Josephus problem | #232 Catching Mice | σ+N+ρ |
//! | Grid paths | #253 Bank Holiday | σ+N+∂ |
//! | Linear extensions | #279+#253 combined | σ+∂+N+μ |

#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![forbid(unsafe_code)]

pub mod catalan;
pub mod cycle;
pub mod derangement;
pub mod grid_paths;
pub mod grounding;
pub mod josephus;
pub mod linear_extensions;

// Re-export primary functions
pub use catalan::catalan;
pub use cycle::{cycle_decomposition, min_transpositions};
pub use derangement::{derangement, derangement_probability};
pub use grid_paths::grid_paths;
pub use josephus::josephus;
pub use linear_extensions::count_linear_extensions_chains;
