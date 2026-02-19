//! # Capability 21: Energy Act (Compute Production)
//!
//! Implementation of the Energy Act as a core structural
//! capability within the HUD domain. This capability manages the
//! "Compute Generation" and "Power Distribution" of the Union.
//!
//! Matches 1:1 to the US Department of Energy (DOE) mandate to
//! ensure America’s security and prosperity by addressing its
//! energy, environmental and nuclear challenges through
//! transformative science and technology solutions.
//!
//! ## DOE Agency Mappings
//! - **NNSA (Nuclear):** Maintains the "Atomic" stability of T1 primitives.
//! - **EIA (Statistics):** Reports on compute consumption and efficiency.
//! - **Grid (Distribution):** Allocates compute resources to the HUD tenancy modules.

use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};

/// T3: EnergyAct - Capability 21 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyAct {
    /// The unique capability identifier.
    pub id: String,
    /// Whether the core compute reactor is stable.
    pub reactor_stable: bool,
}

/// T2-P: ComputePower - The quantified generation of "Power" (Gas/Ops).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ComputePower(pub f64);

/// T2-C: GridStatus - The current state of the Union's energy grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridStatus {
    /// Total compute generation capacity.
    pub generation: f64,
    /// Current compute consumption level.
    pub consumption: f64,
    /// Remaining compute margin before exhaustion.
    pub reserve_margin: f64,
}

impl EnergyAct {
    /// Creates a new instance of the EnergyAct.
    pub fn new() -> Self {
        Self {
            id: "CAP-021".into(),
            reactor_stable: true,
        }
    }

    /// Generate compute power for the Union.
    pub fn generate_power(&self, fuel_units: u64) -> ComputePower {
        // Implementation of power generation (Token/Gas conversion)
        ComputePower(fuel_units as f64 * 0.95) // Efficiency factor
    }

    /// Audit the Grid for stability.
    pub fn audit_grid(&self, current_load: f64) -> Measured<GridStatus> {
        let status = GridStatus {
            generation: 100.0,
            consumption: current_load,
            reserve_margin: 100.0 - current_load,
        };

        let confidence = if status.reserve_margin > 20.0 {
            Confidence::new(1.0)
        } else {
            Confidence::new(0.6)
        };

        Measured::uncertain(status, confidence)
    }
}
