//! # Clinical Trial Validation Paradigm (CTVP)
//!
//! A pharmaceutical-grade software validation framework that maps drug trial
//! phases to software testing stages.
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_vigilance::ctvp::prelude::*;
//!
//! // Define a capability to validate
//! let cap = Capability::new("User Authentication")
//!     .with_threshold(0.95)
//!     .with_min_observations(100);
//!
//! // Track observations
//! let mut tracker = CapabilityTracker::new(cap);
//! tracker.record(true, 1.0);  // Success
//! tracker.record(false, 0.0); // Failure
//!
//! // Check validation status
//! if tracker.meets_threshold() {
//!     println!("Phase 2: VALIDATED");
//! }
//!
//! // Monitor for drift (Phase 4)
//! let mut drift = DriftDetector::new(0.95, 0.90);
//! drift.record(tracker.car());
//! println!("{}", drift.report());
//! ```
//!
//! ## The Five Phases
//!
//! | Phase | Pharma | Software | Question |
//! |-------|--------|----------|----------|
//! | 0 | Preclinical | Unit tests, mocks | Does mechanism work? |
//! | 1 | Safety | Fault injection | Does it fail gracefully? |
//! | 2 | Efficacy | Real data, SLOs | Does it achieve its purpose? |
//! | 3 | Confirmation | Canary, A/B | Does it work at scale? |
//! | 4 | Surveillance | Drift detection | Does it keep working? |
//!
//! ## Core Principle
//!
//! > "Mock testing is testing theater—it validates that your simulation of
//! > reality works, not that your code works in reality."

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod capability;
pub mod config;
pub mod drift;
pub mod metrics;
pub mod phases;
pub mod registry;

/// Prelude for convenient imports
pub mod prelude {
    pub use super::capability::{Capability, CapabilityTracker, MetricType, Observation};
    pub use super::config::CtvpConfig;
    pub use super::drift::{DriftAlert, DriftDetector, TrendDirection};
    pub use super::metrics::{MetricSnapshot, ValidationMetrics};
    pub use super::phases::{
        EvidenceQuality, Phase, PhaseStatus, ValidationEvidence, ValidationSummary,
    };
    pub use super::registry::CtvpRegistry;
}

/// CTVP Framework version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default CAR threshold for Phase 2 validation
pub const DEFAULT_CAR_THRESHOLD: f64 = 0.80;

/// Default minimum observations before validation
pub const DEFAULT_MIN_OBSERVATIONS: u32 = 10;

/// Default alert threshold
pub const DEFAULT_ALERT_THRESHOLD: f64 = 0.70;

/// Returns current Unix timestamp
///
/// # Returns
/// Current time as f64 seconds since epoch
pub fn now() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}
