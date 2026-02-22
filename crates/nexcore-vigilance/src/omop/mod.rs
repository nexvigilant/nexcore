//! OHDSI OMOP CDM v5.4 — Module Root
//!
//! Exposes core OMOP Common Data Model types for use across NexVigilant:
//!
//! - [`tables`]: Core clinical tables (Person, DrugExposure, ConditionOccurrence, etc.)
//! - [`vocabulary`]: Vocabulary tables (Concept, ConceptRelationship, ConceptAncestor, etc.)
//! - [`mapping`]: FAERS → OMOP mapping trait and mapper
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_vigilance::omop::tables::Person;
//! use nexcore_vigilance::omop::vocabulary::Concept;
//! use nexcore_vigilance::omop::mapping::{FaersToOmopMapper, OmopMappable};
//! ```
//!
//! ## OMOP CDM v5.4 Key Facts
//!
//! - All identifiers are `i64` (SQL BIGINT)
//! - Dates use `chrono::NaiveDate`; datetimes use `chrono::NaiveDateTime`
//! - Nullable columns are `Option<T>`
//! - Standard concepts have `domain_id`, `vocabulary_id`, `concept_class_id`
//! - The drug vocabulary standard is RxNorm (Ingredient level)
//! - The condition vocabulary standard is SNOMED-CT
//! - The measurement vocabulary standard is LOINC

pub mod mapping;
pub mod tables;
pub mod vocabulary;

// Convenience re-exports for the most-used types.
pub use mapping::{FaersToOmopMapper, OmopMappable};
pub use tables::{
    ConditionOccurrence, Death, DeviceExposure, DrugExposure, Measurement, Observation,
    ObservationPeriod, Person, ProcedureOccurrence, VisitOccurrence,
};
pub use vocabulary::{Concept, ConceptAncestor, ConceptRelationship, Domain, Vocabulary};
