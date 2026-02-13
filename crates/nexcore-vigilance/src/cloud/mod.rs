//! # Cloud Provider Integrations
//!
//! Cloud provider integrations for nexcore.
//!
//! ## Features
//!
//! - **GCP Billing**: Track costs and billing accounts
//! - **Cost Tracking**: Monitor spending across projects

#![allow(missing_docs)] // Consolidated module - docs to be added incrementally

pub mod gcp;
pub mod secrets;

pub use gcp::{BillingAccount, CostSummary, CostTracker, ProjectBillingInfo};
pub use secrets::SecretClient;
