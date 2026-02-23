//! # Prelude
//!
//! Convenient re-exports for `use nexcore_homeostasis_memory::prelude::*`.
//!
//! Provides all public types from the crate in a single import.

pub use crate::incident::{Incident, IncidentSeverity, IncidentSignature};
pub use crate::playbook::{Playbook, PlaybookMatch, PlaybookStep};
pub use crate::store::{MemoryConfig, MemoryError, MemoryStats, MemoryStore, SimilarIncident};

pub use crate::composites::{CompositeDescriptor, composite_inventory};
pub use crate::primitives::{CratePrimitiveManifest, manifest};
pub use crate::transfer::{TransferMapping, transfer_confidence, transfer_mappings, transfers_for_type};
