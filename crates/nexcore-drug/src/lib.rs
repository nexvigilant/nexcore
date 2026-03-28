//! # nexcore-drug
//!
//! Drug entity domain models — identity, classification, indications,
//! safety signals, label status, and cross-drug analysis.
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol | Role |
//! |---------|-----------|--------|------|
//! | Drug struct | State | ς | Central mutable domain aggregate |
//! | DrugClass / SignalVerdict / LineOfTherapy / ComparisonResult | Sum | Σ | Variant classification |
//! | DrugAnalysis trait methods | Mapping | μ | Transform aggregate → view |
//! | Option fields (rxcui, owner, approval_year, …) | Void | ∅ | Strategic absence |
//! | ContingencyTable cells (a, b, c, d) | Quantity | N | Disproportionality magnitudes |
//! | on_label / off_label filter | Comparison | κ | Predicate selection |
//! | Serialize / Deserialize | Persistence | π | Cross-boundary transport |
//! | ::new() constructors | Existence | ∃ | Aggregate instantiation |
//! | Trait method chains | Causality | → | Query → result pipelines |
//!
//! ## Peer Architecture
//!
//! Drug and Company are **peer aggregates**. `nexcore-drug` does NOT
//! depend on `nexcore-pharma`. The owning manufacturer is stored as
//! `Option<String>`. A strategy crate composes the two peers when
//! cross-aggregate analysis is required.
//!
//! ## Modules
//!
//! - [`id`]: `DrugId` newtype (type-safe string identity)
//! - [`class`]: `DrugClass` enum (16 variants + `Other`)
//! - [`indication`]: `Indication`, `LineOfTherapy`
//! - [`signal`]: `SignalEntry`, `ContingencyTable`, `SignalVerdict`
//! - [`label`]: `LabelStatus`
//! - [`drug`]: `Drug` aggregate
//! - [`analysis`]: `DrugAnalysis` trait + `DefaultDrugAnalysis` implementation

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod analysis;
pub mod class;
pub mod drug;
pub mod id;
pub mod indication;
pub mod label;
pub mod signal;

pub use analysis::{DefaultDrugAnalysis, DrugAnalysis, SignalComparison};
pub use class::DrugClass;
pub use drug::Drug;
pub use id::DrugId;
pub use indication::{Indication, LineOfTherapy};
pub use label::LabelStatus;
pub use signal::{ContingencyTable, SignalEntry, SignalVerdict};
