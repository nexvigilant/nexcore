//! # nexcore-pharma
//!
//! Pharmaceutical company domain models — products, pipelines, safety
//! profiles, and competitive analysis.
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol | Role |
//! |---------|-----------|--------|------|
//! | Company / Product structs | State | ς | Mutable domain aggregates |
//! | TherapeuticArea / Phase / SignalVerdict / CommType | Sum | Σ | Variant classification |
//! | CompanyAnalysis trait methods | Mapping | μ | Transform aggregate → view |
//! | Option fields (ticker, rxcui, …) | Void | ∅ | Strategic absence |
//! | Result at API edges | Boundary | ∂ | Error propagation gates |
//! | Filter / match in trait methods | Comparison | κ | Predicate selection |
//! | prr / ror / cases | Quantity | N | Disproportionality magnitudes |
//! | Serialize / Deserialize | Persistence | π | Cross-boundary transport |
//! | ::new() constructors | Existence | ∃ | Aggregate instantiation |
//! | Trait method chains | Causality | → | Query → result pipelines |
//!
//! ## Modules
//!
//! - [`id`]: `CompanyId` newtype (type-safe string identity)
//! - [`therapeutic`]: `TherapeuticArea` enum (13 variants)
//! - [`product`]: `Product`, `SafetyProfile`, `SignalSummary`, `SignalVerdict`
//! - [`pipeline`]: `PipelineCandidate`, `Phase`
//! - [`safety_comm`]: `SafetyCommunication`, `CommType`
//! - [`company`]: `Company` aggregate
//! - [`analysis`]: `CompanyAnalysis` trait + `DefaultAnalysis` implementation

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod analysis;
pub mod company;
pub mod id;
pub mod pipeline;
pub mod product;
pub mod safety_comm;
pub mod therapeutic;

pub use analysis::CompanyAnalysis;
pub use company::Company;
pub use id::CompanyId;
pub use pipeline::{Phase, PipelineCandidate};
pub use product::{Product, SafetyProfile, SignalSummary, SignalVerdict};
pub use safety_comm::{CommType, SafetyCommunication};
pub use therapeutic::TherapeuticArea;
