//! # Energy Prelude
//!
//! Convenience re-exports for the most common energy management types.
//!
//! ## Usage
//!
//! ```rust
//! use nexcore_energy::prelude::*;
//!
//! let mut pool = TokenPool::new(100_000);
//! pool.spend_productive(5_000);
//! let state = snapshot(&pool, 10.0);
//! assert!(state.energy_charge > 0.0);
//! ```

// Core decision types
pub use crate::decide;
pub use crate::snapshot;

// Pool management
pub use crate::TokenPool;

// Classification
pub use crate::EnergySystem;
pub use crate::Regime;
pub use crate::Strategy;
pub use crate::WasteClass;

// Operation planning
pub use crate::Operation;
pub use crate::OperationBuilder;

// Monitoring
pub use crate::EnergyState;
pub use crate::RecyclingRate;

// Temporal metabolism (Time = ADP)
pub use crate::temporal::AttentionCostCurve;
pub use crate::temporal::CrossSessionSynthase;
pub use crate::temporal::TemporalConservation;
pub use crate::temporal::TemporalMetrics;
pub use crate::temporal::TemporalVelocityTracker;

// Threshold constants
pub use crate::ADP_WEIGHT;
pub use crate::EC_ANABOLIC;
pub use crate::EC_CATABOLIC;
pub use crate::EC_HOMEOSTATIC;
pub use crate::MIN_OPUS_COUPLING;
pub use crate::MIN_SONNET_COUPLING;
