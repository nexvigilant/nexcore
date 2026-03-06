//! # NexVigilant Core — ghost — Ghost Mode for NexVigilant
//!
//! Privacy-by-design pseudonymization, redaction audit, and PII leak detection.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | ∂ Boundary | AnonymizationBoundary, GhostSensor, PiiLeakPattern |
//! | ς State | GhostMode, GhostConfig, AnonymizationLifecycle |
//! | μ Mapping | DataCategory, Pseudonymizer, ScrubAction |
//! | π Persistence | RedactionAudit, RedactionEntry |
//! | ∝ Irreversibility | Anonymized state (no reverse path) |
//! | σ Sequence | PiiScrubber pipeline, lifecycle transitions |
//! | κ Comparison | DataPrivacyPriority ordering |
//! | N Quantity | k-anonymity threshold, l-diversity |
//!
//! ## Architecture
//!
//! Foundation layer crate — no dependency on vigilance or guardian.
//! Higher layers depend on ghost, not the reverse.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod audit;
pub mod boundary;
pub mod config;
pub mod error;
pub mod grounding;
pub mod lifecycle;
pub mod mode;
pub mod priority;
pub mod pseudonymize;
pub mod scrubber;
pub mod sensor;

// Re-exports for ergonomic API surface
pub use audit::{RedactionAudit, RedactionEntry};
pub use boundary::AnonymizationBoundary;
pub use config::{CategoryPolicy, DataCategory, GhostConfig};
pub use error::{GhostError, Result};
pub use lifecycle::AnonymizationLifecycle;
pub use mode::GhostMode;
pub use priority::DataPrivacyPriority;
pub use pseudonymize::{HmacPseudonymizer, PseudonymHandle, Pseudonymizer};
pub use scrubber::{PiiScrubber, ScrubAction, ScrubResult};
pub use sensor::{GhostSensor, GhostSignal, PiiLeakPattern};
