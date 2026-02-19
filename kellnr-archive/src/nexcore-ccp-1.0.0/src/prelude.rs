//! Convenience re-exports for the CCP pharmacokinetic engine.
//!
//! ```
//! use nexcore_ccp::prelude::*;
//! ```

pub use crate::episode::{Episode, Intervention};
pub use crate::error::CcpError;
pub use crate::interactions::{InteractionEffect, compute_interaction, detect_dependency_risk};
pub use crate::kinetics::{
    compute_loading_dose, compute_maintenance_dose, hill_response, plasma_level_at,
    therapeutic_index, time_to_booster, titrate,
};
pub use crate::quality::{QualityComponents, QualityRating, QualityScore, score_episode};
pub use crate::state_machine::{PhaseTransition, can_transition, execute_transition};
pub use crate::types::{
    BioAvailability, Dose, DosingStrategy, HalfLife, InteractionType, Phase, PlasmaLevel,
    TherapeuticWindow,
};
