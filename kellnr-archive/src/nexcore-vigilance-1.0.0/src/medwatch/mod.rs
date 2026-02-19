//! # nexcore-medwatch
//!
//! FDA `MedWatch` Form 3500/3500B types and validation for voluntary adverse event reporting.
//!
//! This crate provides Rust types for the FDA `MedWatch` reporting system:
//! - **Form 3500**: Health Professional Voluntary Reporting
//! - **Form 3500B**: Consumer/Patient Voluntary Reporting
//!
//! ## Overview
//!
//! `MedWatch` is FDA's safety reporting program for human medical products including:
//! - Drugs (prescription and OTC)
//! - Biologics (vaccines, blood products)
//! - Medical devices
//! - Cosmetics
//! - Special nutritional products
//!
//! ## Example
//!
//! ```rust
//! use nexcore_vigilance::medwatch::{Form3500, PatientInformation, validate_form3500};
//!
//! // Create a partial form for validation
//! let patient = PatientInformation {
//!     patient_identifier: "JD001".to_string(),
//!     ..Default::default()
//! };
//! ```
//!
//! ## Form Reference
//!
//! - Form 3500: OMB No. 0910-0291 (Health Professionals)
//! - Form 3500B: OMB No. 0910-0291 (Consumers)

mod common;
mod error;
mod form3500;
mod form3500b;
mod validation;

pub use common::*;
pub use error::*;
pub use form3500::*;
pub use form3500b::*;
pub use validation::*;

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
