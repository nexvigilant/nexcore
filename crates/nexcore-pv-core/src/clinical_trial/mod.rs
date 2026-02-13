//! # Clinical Trial Endpoints
//!
//! FDA guidance translation for proving drug effectiveness.
//!
//! Based on:
//! - FDA Multiple Endpoints Guidance (2024)
//! - FDA Safety Reporting Final Guidance (December 2025)
//! - ICH E9 Statistical Principles for Clinical Trials
//!
//! ## Endpoint Hierarchy (FDA)
//!
//! 1. **Primary endpoints** - Required for approval; demonstrate substantial evidence of effectiveness
//! 2. **Secondary endpoints** - Support primary or demonstrate additional clinical effects
//! 3. **Exploratory endpoints** - Hypothesis-generating, not for approval claims
//!
//! ## Type Mapping (Primitive Codex)
//!
//! | FDA Concept | Tier | Rust Type |
//! |-------------|------|-----------|
//! | Effect size | T2-P | `EffectSize(f64)` |
//! | P-value | T2-P | `PValue(f64)` |
//! | CI bounds | T2-P | `ConfidenceInterval` |
//! | Endpoint | T2-C | `Endpoint<T>` |
//! | Hierarchy | T3 | `EndpointHierarchy` |

pub mod effectiveness;
pub mod endpoints;
pub mod safety_reporting;
pub mod types;

pub use effectiveness::{EffectivenessAssessment, SubstantialEvidence};
pub use endpoints::{
    Endpoint, EndpointHierarchy, EndpointResult, EndpointTier, PrimaryEndpoint, SecondaryEndpoint,
};
pub use safety_reporting::{SafetyReport, SafetyReportingTimeline, SeriousAdverseEvent};
pub use types::{ConfidenceInterval, EffectSize, PValue, StatisticalSignificance};
