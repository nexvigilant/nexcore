//! # Suit Safety System
//!
//! Safety-critical core: Ballistic recovery, fire suppression, E-stops, and voting logic.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod e_stop;
pub mod logger;
pub mod recovery;
pub mod suppression;
pub mod voter;

/// Re-export of safety types.
pub mod prelude {
    pub use crate::e_stop::EStopController;
    pub use crate::recovery::{BallisticSystem, RecoveryState};
    pub use crate::suppression::FireSuppression;
}
