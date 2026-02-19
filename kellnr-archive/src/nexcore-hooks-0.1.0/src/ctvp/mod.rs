//! Clinical Trial Validation Paradigm (CTVP) Framework
//!
//! A reproducible validation methodology mapping pharmaceutical trial phases
//! to software testing stages. Use this module to implement CTVP validation
//! for any capability in your system.
//!
//! # Quick Start
//!
//! 1. Define a capability using [`Capability`]
//! 2. Implement [`CapabilityValidator`] for your system
//! 3. Use [`ValidationPipeline`] to run all phases
//! 4. Monitor with [`DriftDetector`] for Phase 4
//!
//! # Example
//!
//! ```rust,ignore
//! use nexcore_hooks::ctvp::*;
//!
//! // Define capability
//! let cap = Capability::new("MCP Tool Adoption")
//!     .with_metric("CAR", MetricType::Rate)
//!     .with_threshold(0.80);
//!
//! // Track achievement
//! let mut tracker = CapabilityTracker::new(cap);
//! tracker.record(true, 1.0);  // Achieved
//! tracker.record(false, 0.0); // Not achieved
//!
//! // Check Phase 2 validation
//! if tracker.meets_threshold() {
//!     println!("Phase 2: VALIDATED");
//! }
//!
//! // Monitor for drift (Phase 4)
//! let detector = DriftDetector::new(0.80, 0.70);
//! if detector.is_drifting(tracker.car()) {
//!     println!("ALERT: CAR drifting below threshold");
//! }
//! ```

pub mod capability;
pub mod drift;
pub mod metrics;
pub mod phases;

pub use capability::{Capability, CapabilityTracker, MetricType};
pub use drift::{DriftAlert, DriftDetector, TrendDirection};
pub use metrics::{MetricSnapshot, ValidationMetrics};
pub use phases::{EvidenceQuality, Phase, PhaseStatus, ValidationEvidence};

/// CTVP Framework version
pub const CTVP_VERSION: &str = "1.0.0";

/// Minimum sessions required for Phase 2 validation (default)
pub const DEFAULT_MIN_SESSIONS: u32 = 10;

/// Default CAR threshold for Phase 2 validation
pub const DEFAULT_CAR_THRESHOLD: f64 = 0.80;

/// Default alert threshold (below which we alert)
pub const DEFAULT_ALERT_THRESHOLD: f64 = 0.70;
