//! # Pharmacokinetics (PK) Module
//!
//! Pure functions for pharmacokinetic calculations implementing
//! Conservation Laws 1 (mass balance), 7 (steady state), 8 (ionization),
//! and 9 (saturation kinetics).
//!
//! ## Submodules
//!
//! - **auc** - Area under curve calculations, clearance, half-life
//! - **steady_state** - Steady-state concentrations, accumulation
//! - **mass_balance** - Conservation law 1, drug accounting
//! - **ionization** - Henderson-Hasselbalch, pH partitioning
//! - **saturation** - Michaelis-Menten kinetics, receptor occupancy

pub mod auc;
pub mod ionization;
pub mod mass_balance;
pub mod saturation;
pub mod steady_state;
pub mod types;

pub use types::*;

// Re-export commonly used items
pub use auc::{
    calculate_auc_linear, calculate_auc_log_linear, calculate_clearance_from_auc,
    calculate_half_life_from_clearance,
};
pub use ionization::{
    calculate_fraction_unionized, calculate_ionization_ratio, calculate_ph_partition,
};
pub use mass_balance::{
    MassBalanceResult, calculate_cumulative_elimination, calculate_remaining_drug,
    check_mass_balance,
};
pub use saturation::{
    calculate_michaelis_menten_rate, calculate_receptor_occupancy, calculate_saturation_fraction,
    is_linear_kinetics,
};
pub use steady_state::{
    HALF_LIVES_TO_STEADY_STATE, calculate_accumulation_factor, calculate_loading_dose,
    calculate_steady_state_concentration, calculate_time_to_steady_state,
};
