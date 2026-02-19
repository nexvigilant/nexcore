//! # Capability 8: Agricultural Data Pipeline (FAERS-ETL)
//!
//! Implementation of the Agricultural Data Act as a core structural
//! capability within the HUD domain. This capability manages the
//! "Data Harvesting" and "Crop Yield" (Signal Counts) of the Union.
//!
//! Matches 1:1 to the US Department of Agriculture (USDA) mandate for
//! developing and executing federal laws related to farming, forestry,
//! and food.
//!
//! ## USDA Agency Mappings
//! - **ARS (Research Service):** Identifies potential signal "Crops" from raw FAERS data.
//! - **NRCS (Conservation Service):** Handles data deduplication and "Soil Quality" (Data Integrity).
//! - **FSIS (Inspection Service):** Validates serious reports (SAE) for immediate processing.
//! - **FAS (Foreign Service):** Manages data imports from external registries (EudraVigilance, etc.).

use crate::primitives::governance::Verdict;
use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};

/// T3: AgriculturalDataAct - Capability 8 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgriculturalDataAct {
    pub id: String,
    pub harvest_active: bool,
}

/// T2-P: CropType - The type of data being harvested.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CropType {
    /// Raw adverse event reports.
    RawReports,
    /// Refined signal candidates.
    SignalCrops,
    /// Validated safety findings.
    PrimeHarvest,
}

/// T2-C: HarvestYield - The output of an ETL cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvestYield {
    pub crop_type: CropType,
    pub quantity: u64,
    pub purity_score: Confidence,
}

impl AgriculturalDataAct {
    pub fn new() -> Self {
        Self {
            id: "CAP-008".into(),
            harvest_active: true,
        }
    }

    /// Execute a "Data Harvest" (ETL Cycle).
    /// Returns a Measured<HarvestYield> ensuring the yield quality is quantified.
    pub fn execute_harvest(&self, input_size: u64, raw_purity: f64) -> Measured<HarvestYield> {
        // Simulation of USDA Processing Logic
        let yield_quantity = (input_size as f64 * raw_purity) as u64;
        let confidence = Confidence::new(raw_purity);

        let yield_data = HarvestYield {
            crop_type: CropType::RawReports,
            quantity: yield_quantity,
            purity_score: confidence,
        };

        Measured::uncertain(yield_data, confidence)
    }

    /// Inspect a harvest for "Pests" (Data Corruption or False Signals).
    pub fn inspect_yield(&self, harvest: &HarvestYield) -> Verdict {
        if harvest.purity_score.value() < 0.7 {
            // High "Pest" density detected
            Verdict::Rejected
        } else {
            Verdict::Permitted
        }
    }
}
