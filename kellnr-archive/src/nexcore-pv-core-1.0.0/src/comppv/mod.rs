//! # Comprehensive Pharmacovigilance (CompPV) Conservation Laws
//!
//! This module implements the 11 Conservation Laws that govern pharmacovigilance
//! signal detection and drug safety assessment. Each law represents a fundamental
//! physical, chemical, or biological constraint that must be satisfied.
//!
//! ## The 11 Conservation Laws
//!
//! | Law | Name | Mathematical Form |
//! |-----|------|-------------------|
//! | 1 | Drug Mass Balance | D(t) = D₀ - ∫CL·C(τ)dτ |
//! | 2 | Thermodynamic Binding | ΔG = -RT ln(Ka) < 0 |
//! | 3 | Receptor State Conservation | R_total = R_free + R_bound + R_desensitized |
//! | 4 | Pathway Flux Conservation | Σ J_in = Σ J_out |
//! | 5 | Enzyme Regeneration | dE/dt = k_syn - k_deg - k_inact[I] |
//! | 6 | ADME Rate Conservation | dA/dt = Rate_in - Rate_out |
//! | 7 | Steady-State Equilibrium | C_ss = (F·Dose)/(CL·τ) |
//! | 8 | Ionization State | log([A⁻]/[HA]) = pH - pKa |
//! | 9 | Saturation Kinetics | v = Vmax·[S]/(Km + [S]) |
//! | 10 | Entropy Increase | ΔS_total ≥ 0 |
//! | 11 | Genetic Conservation | DNA_before = DNA_after |
//!
//! ## Architecture
//!
//! - **types.rs** - Core enums and structs (ConservationLaw, LawValidationResult)
//! - **specs.rs** - Formal specifications with validation tolerances
//! - **validators.rs** - State structs and validation functions for each law
//! - Calculation functions are reused from the `pk` and `thermodynamic` modules

pub mod atoms;
pub mod molecules;
pub mod specs;
pub mod types;
pub mod validators;

pub use specs::CONSERVATION_LAW_SPECS;
pub use types::*;
pub use validators::*;
