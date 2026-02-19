//! # Capability Assessment Framework
//!
//! Chemistry-derived equations for quantifying organizational capabilities.
//!
//! ## Equations Mapped
//!
//! | Chemistry | Capability Metric |
//! |-----------|-------------------|
//! | Arrhenius | Adoption potential (activation barrier) |
//! | Michaelis-Menten | Capacity efficiency (saturation) |
//! | Hill | Synergy coefficient (cooperativity) |
//! | Henderson-Hasselbalch | Stability buffer (resilience) |
//! | Half-life | Freshness factor (decay) |
//!
//! ## Skill Quality Index (SQI)
//!
//! ```text
//! SQI = (Adoption×0.20 + Capacity×0.25 + Synergy×0.20 + Stability×0.20 + Freshness×0.15) × 10
//! ```

pub mod arrhenius;
pub mod capacity;
pub mod decay;
pub mod hill;
pub mod sqi;
pub mod stability;
pub mod types;

pub use arrhenius::{AdoptionPotential, LearningBarrier};
pub use capacity::{CapacityEfficiency, SaturationPoint};
pub use decay::{CapabilityHalfLife, FreshnessFactor};
pub use hill::{CooperativityType, SynergyCoefficient};
pub use sqi::{CapabilityAssessment, SkillQualityIndex, SqiRating};
pub use stability::{BufferRatio, StabilityScore};
pub use types::{Capability, CapabilityMetrics, CapabilityType};
