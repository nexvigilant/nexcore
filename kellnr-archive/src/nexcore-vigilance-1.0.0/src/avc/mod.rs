//! # Autonomous Vigilance Company (AVC)
//!
//! The core autonomous system grounded to 9 T1 primitives:
//! κ(Comparison) + σ(Sequence) + ∂(Boundary) + ρ(Recursion) +
//! ς(State) + μ(Mapping) + π(Persistence) + →(Causality) + N(Quantity)
//!
//! ## Architecture
//!
//! ```text
//! σ(sense) → κ(compare) → →(decide) → π(audit) → ρ(learn)
//!     ↑                                              │
//!     └──────────────── ς(homeostasis) ←─────────────┘
//! ```
//!
//! ## Modules
//!
//! - **types**: T1/T2-P/T2-C type taxonomy with `GroundsTo` implementations
//! - **engine**: T3 AVC system with the autonomous SENSE→COMPARE→DECIDE→ACT→LEARN loop
//!
//! ## Example
//!
//! ```rust
//! use nexcore_vigilance::avc::{Avc, Baseline, Event, Source, Threshold};
//!
//! let mut avc = Avc::new();
//!
//! // Configure domain
//! let baseline = Baseline::from_values(&[10.0, 11.0, 9.0, 10.5]);
//! avc.set_baseline("pharma", baseline);
//! if let Ok(thresh) = Threshold::new(0.0, 5.0) {
//!     avc.set_threshold("pharma", thresh);
//! }
//!
//! // Process event through full cycle
//! let event = Event::new("pharma", 20.0, Source("fda".into()));
//! let result = avc.process(event);
//! assert!(result.is_ok());
//! ```

pub mod engine;
pub mod types;

// Re-export T1 types
pub use types::{Boundary, Comparison, Frequency, OperationalState, TimeSeries};

// Re-export T2-P types
pub use types::{Baseline, ClientId, Domain, SignalId, Source, Threshold};

// Re-export T2-C types
pub use types::{
    AuditRecord, ClientConfig, Decision, Detection, Event, Feedback, Outcome, PvAction,
};

// Re-export T3 engine
pub use engine::{ActionResult, Avc, AvcMetrics};

// Re-export error and utilities
pub use types::{AvcError, HumanCommand, fnv1a_hash};

// Re-export trust boundaries
pub use types::{
    ADJUSTMENT_FACTOR, CRITICAL_BOUNDARY, HEALTH_ALERT, HEALTH_DEGRADED, HEALTH_NOMINAL,
    LEARNING_BATCH_SIZE, MAX_FALSE_NEGATIVE_RATE, MAX_FALSE_POSITIVE_RATE,
};
