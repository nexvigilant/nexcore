//! Propensity score methods for confounding control.
//!
//! This module contains methods for controlling confounding
//! in observational pharmacoepidemiology studies using propensity scores.
//!
//! ## Methods
//!
//! - **HDPS** (High-Dimensional Propensity Score) - Automated covariate selection
//!   using the Bross formula to identify empirically relevant confounders
//!
//! ## Use Cases
//!
//! - Observational study confounding adjustment
//! - Comparative safety analysis
//! - Real-world evidence generation
//! - Post-market surveillance studies
//!
//! ## Feature Flag
//!
//! The HDPS implementation requires the `propensity` feature flag due to
//! linear algebra dependencies:
//!
//! ```toml
//! guardian-signals = { version = "...", features = ["propensity"] }
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use nexcore_vigilance::pv::signals::propensity::hdps::{calculate_hdps, HDPSConfig, CovariateData};
//!
//! // Create covariate data from claims/EHR
//! let diabetes = CovariateData::new(
//!     "dx".into(),
//!     "E11".into(), // ICD-10 diabetes
//!     vec![true, false, true, ...], // presence per subject
//! );
//!
//! // Calculate HDPS-adjusted propensity scores
//! let config = HDPSConfig::default();
//! let result = calculate_hdps(&[diabetes], &exposure, &outcome, &config)?;
//!
//! // Use propensity scores for matching, stratification, or weighting
//! println!("C-statistic: {:.3}", result.c_statistic);
//! for decile in result.propensity_deciles() {
//!     println!("Decile: [{:.3}, {:.3}]", decile.0, decile.1);
//! }
//! ```

pub mod hdps;

// Re-export main types for convenience
pub use hdps::{CovariateData, HDPSConfig, HDPSResult, SelectedCovariate, calculate_hdps};
