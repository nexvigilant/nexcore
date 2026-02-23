#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! # Homeostasis Machine — Memory
//!
//! Incident memory and response playbooks for the Homeostasis Machine.
//!
//! This crate provides the **immune memory** layer — the system's ability to
//! remember past incidents, match new events against historical patterns, and
//! recommend proven response playbooks.
//!
//! ## Architecture
//!
//! ```text
//! New Incident → MemoryStore::find_similar() → Past Incidents
//!                                            → MemoryStore::match_playbooks() → Playbook
//!                                            → Execute Playbook → Record Outcome
//! ```
//!
//! ## Modules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`incident`] | Incident types, signatures, severity, similarity scoring |
//! | [`playbook`] | Response playbooks — named action sequences for known patterns |
//! | [`store`] | Memory store — persistence, lookup, statistics |
//!
//! ## T1 Grounding
//!
//! - π (Persistence) — incidents and playbooks persist across control loops
//! - μ (Mapping) — signatures map patterns to responses
//! - κ (Comparison) — similarity scoring drives pattern recognition
//! - σ (Sequence) — playbook steps execute in order
//! - → (Causality) — incidents record cause-effect chains

pub mod incident;
pub mod memory;
pub mod playbook;
pub mod store;

pub mod composites;
pub mod grounding;
pub mod prelude;
pub mod primitives;
pub mod transfer;

// Convenience re-exports.
pub use incident::{Incident, IncidentSeverity, IncidentSignature};
pub use memory::IncidentMemory;
pub use playbook::{Playbook, PlaybookMatch, PlaybookStep};
pub use store::{MemoryConfig, MemoryError, MemoryStats, MemoryStore, SimilarIncident};
