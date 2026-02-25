//! # nexcore-fhir — Focused FHIR R4 Types for Pharmacovigilance
//!
//! NexVigilant-native FHIR R4 resource types. Zero external FHIR dependencies.
//! Covers the ~15 resources relevant to pharmacovigilance signal detection.
//!
//! ## Resources
//!
//! | Resource | PV Use |
//! |----------|--------|
//! | AdverseEvent | Core — one ICSR maps to one AdverseEvent |
//! | Patient | Subject demographics |
//! | Medication | Drug identification (RxNorm, NDC) |
//! | MedicationAdministration | Drug exposure timing + dosage |
//! | Condition | Pre-existing conditions, comorbidities |
//! | Observation | Lab results, vital signs |
//! | Encounter | Healthcare visit context |
//! | Organization | Reporter / manufacturer |
//! | Practitioner | Reporter identity |
//! | Bundle | Batch container for FHIR resources |
//!
//! ## Adapter
//!
//! `adapter::adverse_event_to_signal` converts a FHIR AdverseEvent into a
//! `SignalInput` compatible with nexcore-vigilance disproportionality analysis.
//!
//! Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![allow(
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    reason = "FHIR resource DTOs intentionally track a closed, PV-focused subset of R4 fields"
)]

/// FHIR R4 base data types (Coding, CodeableConcept, Reference, etc.).
pub mod base;

/// FHIR R4 resource definitions — PV-focused subset.
pub mod resources;

/// FHIR AdverseEvent → NexVigilant signal detection adapter.
pub mod adapter;

/// T1 primitive grounding for FHIR resource types.
pub mod grounding;

// Convenience re-exports
pub use adapter::{SignalInput, adverse_event_to_signal, adverse_events_to_signals};
pub use base::{
    CodeableConcept, Coding, Identifier, MEDDRA_SYSTEM, RXNORM_SYSTEM, Reference, SNOMED_SYSTEM,
};
pub use resources::{
    AdverseEvent, Bundle, Condition, Encounter, Medication, MedicationAdministration, Observation,
    Organization, Patient, Practitioner,
};
