//! # Capability 9: Infrastructure & Energy Grid (Compute Quota Act)
//!
//! Implementation of the Compute Quota Act as a core structural
//! capability within the HUD domain. This capability manages the
//! "Power Plants" (Compute Quotas) and "Fuel Reserves" (Memory Pools)
//! of the Union.
//!
//! Matches 1:1 to the US Department of Energy (DOE) mandate to ensure
//! the security and prosperity of the Union by addressing its energy
//! and environmental challenges through transformative science and technology.
//!
//! ## DOE Agency Mappings
//! - **EIA (Energy Info Admin):** Tracks and reports real-time resource utilization.
//! - **Office of Science:** Optimizes algorithm efficiency to reduce "Energy Waste".
//! - **NNSA (Nuclear Security):** Safeguards high-stakes compute reserves for formal proofs.
//! - **FERC (Regulatory Commission):** Manages the "Grid" (Inter-domain resource transfer).

use crate::primitives::governance::{Treasury, Verdict};
use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};

/// T3: ComputeQuotaAct - Capability 9 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeQuotaAct {
    pub id: String,
    pub grid_stable: bool,
}

/// T2-P: EnergyUsage - The quantified consumption of compute.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct EnergyUsage(pub f64);

/// T2-C: GridStatus - The health of the Union's energy reserves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridStatus {
    pub compute_load: f64,       // 0.0 - 1.0
    pub memory_utilization: f64, // 0.0 - 1.0
    pub reserve_margin: f64,
}

impl ComputeQuotaAct {
    pub fn new() -> Self {
        Self {
            id: "CAP-009".into(),
            grid_stable: true,
        }
    }

    /// Assess the stability of the energy grid.
    pub fn assess_grid(&self, _treasury: &Treasury) -> Measured<GridStatus> {
        // Simulation of EIA reporting logic
        let status = GridStatus {
            compute_load: 0.45, // Assume 45% load for simulation
            memory_utilization: 0.30,
            reserve_margin: 0.55,
        };

        let confidence = if status.reserve_margin > 0.2 {
            Confidence::new(0.95)
        } else {
            Confidence::new(0.5)
        };

        Measured::uncertain(status, confidence)
    }

    /// Authorize a "High-Energy" executive action.
    /// Ensures NNSA-grade security for the Union's reserves.
    pub fn authorize_surge(&self, status: &GridStatus, cost: &Treasury) -> Verdict {
        if status.compute_load > 0.9 || status.reserve_margin < 0.1 {
            // Grid at risk of Brownout/Panic
            Verdict::Rejected
        } else if cost.compute_quota > 1000 {
            // High stakes energy spend requires flagging
            Verdict::Flagged
        } else {
            Verdict::Permitted
        }
    }
}
