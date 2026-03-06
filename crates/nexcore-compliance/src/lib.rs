//! # NexVigilant Core — Compliance Engine
//!
//! Regulatory compliance verification for pharmacovigilance systems.
//! Integrates government data sources (SAM.gov) with OSCAL-inspired
//! compliance assessment DSL.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                 NEXCORE COMPLIANCE ENGINE                    │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                              │
//! │  DATA SOURCES                                                │
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐                      │
//! │  │ SAM.gov │  │  FDA    │  │  ICH    │                      │
//! │  │Exclus.  │  │Guidelines│ │Controls │                      │
//! │  └────┬────┘  └────┬────┘  └────┬────┘                      │
//! │       │            │            │                            │
//! │       ▼            ▼            ▼                            │
//! │  ┌─────────────────────────────────────────────┐            │
//! │  │          COMPLIANCE DSL (OSCAL-INSPIRED)    │            │
//! │  │  • Control definitions                      │            │
//! │  │  • Assessment specifications                │            │
//! │  │  • Finding generation                       │            │
//! │  └────────────────────┬────────────────────────┘            │
//! │                       │                                      │
//! │                       ▼                                      │
//! │  ┌─────────────────────────────────────────────┐            │
//! │  │          GUARDIAN INTEGRATION               │            │
//! │  │  • Risk scoring                             │            │
//! │  │  • Compliance signals                       │            │
//! │  └─────────────────────────────────────────────┘            │
//! │                                                              │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Modules
//!
//! - [`sam`] - SAM.gov Exclusions API client
//! - [`sec`] - SEC EDGAR filings API client
//! - [`oscal`] - OSCAL-inspired type definitions
//! - [`dsl`] - Compliance assessment DSL

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
pub mod dsl;
pub mod grounding;
pub mod oscal;
pub mod sam;
pub mod sec;

/// Re-export core types for convenience.
pub use dsl::{Assessment, ComplianceResult, Finding, FindingSeverity};
pub use oscal::{Control, ControlCatalog, ControlStatus};
pub use sam::{Exclusion, ExclusionClassification, ExclusionType, SamClient};
pub use sec::{CompanyFacts, CompanySubmissions, Filing, SecClient};
