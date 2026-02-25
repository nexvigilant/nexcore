//! # nexcore-qbr — Quantitative Benefit-Risk
//!
//! Computable, confidence-bounded, methodologically transparent
//! benefit-risk assessment for pharmaceutical development.
//!
//! ## Four Forms
//!
//! 1. **Simple Ratio**: `signal_strength(benefit) / signal_strength(risk)`
//! 2. **Bayesian (EBGM)**: `EBGM_benefit / EBGM_risk` with credibility intervals
//! 3. **Composite Weighted**: `Σ(w_i × signal(benefit_i)) / Σ(w_j × signal(risk_j))`
//! 4. **Therapeutic Window**: `∫(Hill_efficacy - Hill_toxicity) dd`
//!
//! ## Key Properties
//!
//! - **Measured<T> throughout**: Every output carries propagated confidence.
//! - **Reuses existing signal detection**: ContingencyTable + 6 algorithms unchanged.
//! - **Sovereign**: Zero external dependencies. Simpson's rule implemented from scratch.
//! - **GroundsTo T1**: All types ground to Lex Primitiva.
//!
//! ## Grounding
//!
//! QBR → κ(Comparison) + N(Quantity) + →(Causality) + ∂(Boundary)
//!
//! Tier: T3-D (Domain Composite)

#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![forbid(unsafe_code)]

pub mod bayesian;
pub mod composite;
pub mod compute;
pub mod error;
pub mod grounding;
pub mod integration;
pub mod signal;
pub mod simple;
pub mod therapeutic_window;
pub mod types;

// ═══════════════════════════════════════════════════════════════════════════
// PUBLIC API RE-EXPORTS
// ═══════════════════════════════════════════════════════════════════════════

pub use compute::compute_qbr;
pub use error::QbrError;
pub use types::{
    BenefitRiskInput, HillCurveParams, IntegrationBounds, QBR, QbrMethodDetails, QbrSignalMethod,
};

// Form-specific entry points
pub use bayesian::compute_bayesian;
pub use composite::compute_composite;
pub use simple::compute_simple;
pub use therapeutic_window::compute_therapeutic_window;

// Integration utility
pub use integration::simpson_integrate;
