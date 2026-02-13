//! Vessel hierarchy — arteries, capillaries, veins.
//!
//! ## Biological Model
//!
//! Vessels form a branching tree from heart → arteries → capillaries → veins → heart.
//! Each vessel type has different properties:
//! - **Arteries**: High pressure, elastic, outbound (service mesh egress)
//! - **Capillaries**: Exchange points, thin walls (direct data access)
//! - **Veins**: Low pressure, valved, return (service mesh ingress)
//!
//! ## Vessel Compliance (Elasticity)
//!
//! Healthy vessels expand under pressure (compliance).
//! Atherosclerosis = reduced compliance = rigid pipes = higher pressure.
//!
//! ## T1 Grounding: ∂ (Boundary) dominant
//!
//! Vessels ARE boundaries — they define the channel walls through which
//! data flows. Arteries have thick boundaries (high pressure tolerance),
//! capillaries have thin boundaries (exchange), veins have valved boundaries.

use serde::{Deserialize, Serialize};

use crate::types::{FlowRate, Pressure, Resistance, VesselType, Volume};

/// A single vessel in the circulatory network.
///
/// # Tier: T2-C (∂ · N · → · ς)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vessel {
    /// Vessel identifier (name/path)
    pub id: String,
    /// Type classification
    pub vessel_type: VesselType,
    /// Maximum capacity
    pub capacity: Volume,
    /// Current fill level
    pub current_volume: Volume,
    /// Compliance (elasticity, 0.0–1.0)
    pub compliance: f64,
    /// Resistance to flow
    pub resistance: Resistance,
    /// Whether the vessel is occluded (0.0 = clear, 1.0 = blocked)
    pub occlusion: f64,
}

impl Vessel {
    /// Create a new vessel.
    pub fn new(id: impl Into<String>, vessel_type: VesselType) -> Self {
        let (capacity, compliance, resistance) = match vessel_type {
            VesselType::Aorta => (Volume::new(100.0), 0.9, Resistance::new(0.1)),
            VesselType::Artery => (Volume::new(50.0), 0.8, Resistance::new(0.3)),
            VesselType::Arteriole => (Volume::new(10.0), 0.5, Resistance::new(0.8)),
            VesselType::Capillary => (Volume::new(2.0), 0.2, Resistance::new(1.0)),
            VesselType::Venule => (Volume::new(10.0), 0.6, Resistance::new(0.5)),
            VesselType::Vein => (Volume::new(80.0), 0.7, Resistance::new(0.2)),
            VesselType::VenaCava => (Volume::new(150.0), 0.8, Resistance::new(0.1)),
        };

        Self {
            id: id.into(),
            vessel_type,
            capacity,
            current_volume: Volume::new(0.0),
            compliance,
            resistance,
            occlusion: 0.0,
        }
    }

    /// Effective resistance accounting for occlusion.
    ///
    /// R_eff = R_base / (1 - occlusion). As occlusion → 1.0, resistance → ∞.
    pub fn effective_resistance(&self) -> Resistance {
        let denom = 1.0 - self.occlusion.clamp(0.0, 0.99);
        Resistance::new(self.resistance.value() / denom)
    }

    /// Current utilization (0.0 to 1.0+).
    pub fn utilization(&self) -> f64 {
        self.current_volume.utilization(self.capacity.value())
    }

    /// Whether this vessel is critically occluded (>80%).
    pub fn is_critically_occluded(&self) -> bool {
        self.occlusion > 0.8
    }

    /// Compute pressure drop across this vessel.
    ///
    /// ΔP = flow × resistance (Ohm's law analog: V = IR)
    pub fn pressure_drop(&self, flow: FlowRate) -> Pressure {
        Pressure::new(flow.value() * self.effective_resistance().value())
    }

    /// Receive flow into this vessel.
    ///
    /// Returns how much was actually accepted (may be less if at capacity).
    pub fn receive(&mut self, amount: f64) -> f64 {
        let available = self.capacity.value() - self.current_volume.value();
        // Compliance allows slight over-fill
        let effective_capacity = available * (1.0 + self.compliance);
        let accepted = amount.min(effective_capacity).max(0.0);
        self.current_volume = Volume::new(self.current_volume.value() + accepted);
        accepted
    }

    /// Release flow from this vessel.
    ///
    /// Returns how much was actually released.
    pub fn release(&mut self, amount: f64) -> f64 {
        let available = self.current_volume.value();
        let released = amount.min(available).max(0.0);
        self.current_volume = Volume::new(self.current_volume.value() - released);
        released
    }
}

/// A network of connected vessels forming a vascular bed.
///
/// # Tier: T3 (∂ · σ · μ · → · N · ς)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VascularBed {
    /// Name of this vascular bed
    pub name: String,
    /// Vessels in this bed (ordered: arterial → capillary → venous)
    pub vessels: Vec<Vessel>,
}

impl VascularBed {
    /// Create a new vascular bed.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vessels: Vec::new(),
        }
    }

    /// Add a vessel to the bed.
    pub fn add_vessel(&mut self, vessel: Vessel) {
        self.vessels.push(vessel);
    }

    /// Create a standard organ vascular bed (artery → capillary → vein).
    pub fn standard_organ(name: impl Into<String>) -> Self {
        let name = name.into();
        let mut bed = Self::new(name.clone());
        bed.add_vessel(Vessel::new(format!("{}_artery", name), VesselType::Artery));
        bed.add_vessel(Vessel::new(
            format!("{}_arteriole", name),
            VesselType::Arteriole,
        ));
        bed.add_vessel(Vessel::new(
            format!("{}_capillary", name),
            VesselType::Capillary,
        ));
        bed.add_vessel(Vessel::new(format!("{}_venule", name), VesselType::Venule));
        bed.add_vessel(Vessel::new(format!("{}_vein", name), VesselType::Vein));
        bed
    }

    /// Total resistance of the bed (series: R_total = sum of all R).
    pub fn total_resistance(&self) -> Resistance {
        let total: f64 = self
            .vessels
            .iter()
            .map(|v| v.effective_resistance().value())
            .sum();
        Resistance::new(total)
    }

    /// Total volume in the bed.
    pub fn total_volume(&self) -> Volume {
        let total: f64 = self.vessels.iter().map(|v| v.current_volume.value()).sum();
        Volume::new(total)
    }

    /// Total capacity of the bed.
    pub fn total_capacity(&self) -> Volume {
        let total: f64 = self.vessels.iter().map(|v| v.capacity.value()).sum();
        Volume::new(total)
    }

    /// Average utilization across all vessels.
    pub fn avg_utilization(&self) -> f64 {
        if self.vessels.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.vessels.iter().map(|v| v.utilization()).sum();
        sum / self.vessels.len() as f64
    }

    /// Find vessels that are critically occluded.
    pub fn find_occlusions(&self) -> Vec<&Vessel> {
        self.vessels
            .iter()
            .filter(|v| v.is_critically_occluded())
            .collect()
    }
}

/// Capillary exchange result — what was delivered and collected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapillaryExchange {
    /// Items delivered to tissue
    pub delivered: f64,
    /// Waste collected from tissue
    pub collected: f64,
    /// Net exchange (delivered - collected)
    pub net: f64,
}

/// Perform capillary exchange between blood and tissue.
///
/// Nutrients flow out (hydrostatic pressure > osmotic pressure at arterial end).
/// Waste flows in (osmotic pressure > hydrostatic at venous end).
pub fn capillary_exchange(
    arterial_pressure: Pressure,
    venous_pressure: Pressure,
    osmotic_pressure: f64,
) -> CapillaryExchange {
    // Starling forces: net filtration = (Pc - Pi) - (πc - πi)
    // Simplified: delivery ∝ arterial_pressure - osmotic, collection ∝ osmotic - venous_pressure
    let delivered = (arterial_pressure.value() - osmotic_pressure).max(0.0);
    let collected = (osmotic_pressure - venous_pressure.value()).max(0.0);
    CapillaryExchange {
        delivered,
        collected,
        net: delivered - collected,
    }
}
