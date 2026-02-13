//! Multiple testing correction methods.
//!
//! This module contains methods for adjusting p-values when performing
//! multiple comparisons in large-scale signal detection.
//!
//! ## Methods
//!
//! - **Benjamini-Hochberg** - FDR (False Discovery Rate) control
//! - **Bonferroni** - FWER (Family-Wise Error Rate) control
//! - **Holm** - Step-down Bonferroni (more powerful)
//! - **Šidák** - Assumes independence
//!
//! ## When to Use Which
//!
//! | Method | Use When | Control |
//! |--------|----------|---------|
//! | Benjamini-Hochberg | Large-scale screening, discovery | FDR |
//! | Bonferroni | High-stakes, few tests | FWER |
//! | Holm | Want FWER but more power than Bonferroni | FWER |
//! | Šidák | Tests are independent | FWER |
//!
//! ## Example
//!
//! ```rust
//! use nexcore_vigilance::pv::signals::adjustment::benjamini_hochberg::bh_adjust;
//! use nexcore_vigilance::pv::signals::adjustment::bonferroni::bonferroni_adjust;
//!
//! let p_values = vec![0.001, 0.008, 0.039, 0.041, 0.05];
//!
//! // FDR control (less conservative, good for screening)
//! let bh = bh_adjust(&p_values, 0.05);
//! println!("BH rejects {} hypotheses", bh.n_rejected);
//!
//! // FWER control (more conservative, for confirmatory)
//! let bonf = bonferroni_adjust(&p_values, 0.05);
//! println!("Bonferroni rejects {} hypotheses", bonf.n_rejected);
//! ```

pub mod benjamini_hochberg;
pub mod bonferroni;

// Re-export main functions for convenience
pub use benjamini_hochberg::{BHResult, bh_adjust, bh_reject, compute_q_values};
pub use bonferroni::{
    BonferroniMethod, BonferroniResult, bonferroni_adjust, bonferroni_threshold, holm_adjust,
    sidak_adjust,
};
