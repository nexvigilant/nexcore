//! # Thermodynamic Module (Conservation Laws 2 + First Law)
//!
//! ## Core Equations
//!
//! | Equation | Formula | Module |
//! |----------|---------|--------|
//! | Gibbs Free Energy | ΔG = ΔH - TΔS | `binding` |
//! | Dissociation | Kd = exp(ΔG/RT) | `binding` |
//! | Arrhenius | k = A·exp(-Ea/RT) | `kinetics` |
//! | **First Law (Closed)** | ΔU = Q - W | `energy_balance` |
//! | **First Law (Open)** | dE/dt = Q̇ - Ẇ + Σṁh | `energy_balance` |
//!
//! ## Safety Implications
//!
//! - Off-target binding: DeltaG favorable for non-intended targets
//! - Selectivity: Delta_DeltaG between targets
//! - Energy balance: System accumulation indicates processing bottlenecks
//!
//! ## Submodules
//!
//! - **binding** - Gibbs free energy, dissociation constants, selectivity
//! - **kinetics** - Residence time, association/dissociation rates, Arrhenius
//! - **energy_balance** - First Law (closed/open), system classification

pub mod binding;
pub mod energy_balance;
pub mod kinetics;

// Re-export commonly used items
pub use binding::{
    R_J_MOL_K, R_KJ_MOL_K, STANDARD_TEMP_K, calculate_association_constant,
    calculate_binding_entropy, calculate_dissociation_constant, calculate_gibbs_free_energy,
    calculate_selectivity, is_spontaneous_binding,
};
pub use energy_balance::{
    CaseProcessingBalance, ClosedSystemBalance, MassFlowStream, OpenSystemBalance, SystemType,
    first_law_closed, first_law_open, steady_state_heat_work_balance,
};
pub use kinetics::{
    calculate_arrhenius_rate, calculate_koff_from_kd_kon, calculate_kon_from_kd_koff,
    calculate_residence_time,
};
