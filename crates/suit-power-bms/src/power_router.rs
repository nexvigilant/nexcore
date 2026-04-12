//! Federated power routing across 4 energy tiers.
//!
//! Maps to the `power-load-router` microgram decision tree.
//! Routes demand by magnitude, duration, locality, and bus health.

use serde::{Deserialize, Serialize};

/// Energy tier in the federated battery architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnergyTier {
    /// Main NMC pack — bulk energy, steady-state loads.
    MainBus,
    /// Main bus derated — module offline, 50% max power.
    MainBusDerated,
    /// Ultracapacitor bank — burst transients ≤5s.
    UltracapBank,
    /// Per-limb LFP buffer — local demand ≤2kW.
    PerLimbLfp,
    /// Auxiliary LFP — life-critical only (HUD, comms, BRS).
    AuxLfp,
}

/// Main bus health state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BusStatus {
    /// All modules online, full capacity.
    Nominal,
    /// One or more modules offline, reduced capacity.
    Degraded,
    /// Total bus failure — aux takes over.
    Failed,
}

/// Limb location for local power routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Limb {
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
    Torso,
    Distributed,
}

/// Power demand request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerDemand {
    /// Instantaneous demand in kilowatts.
    pub demand_kw: f64,
    /// Expected duration in seconds.
    pub duration_s: f64,
    /// Which limb/zone needs power.
    pub locality: Limb,
    /// Main bus health.
    pub bus_status: BusStatus,
    /// Ultracap state of charge (0.0–1.0).
    pub ultracap_soc: f64,
    /// Target limb buffer SoC (0.0–1.0).
    pub limb_buffer_soc: f64,
}

/// Power routing decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerRoute {
    /// Selected energy tier.
    pub source: EnergyTier,
    /// Fallback tier if primary fails.
    pub fallback: EnergyTier,
    /// Maximum available power from this tier (kW).
    pub max_available_kw: f64,
    /// Rationale for the routing decision.
    pub rationale: &'static str,
}

impl PowerDemand {
    /// Route this demand to the appropriate energy tier.
    pub fn route(&self) -> PowerRoute {
        // Gate 1: Bus failure → emergency aux
        if self.bus_status == BusStatus::Failed {
            return PowerRoute {
                source: EnergyTier::AuxLfp,
                fallback: EnergyTier::AuxLfp,
                max_available_kw: 0.5,
                rationale: "Main bus FAILED — aux LFP takes life-critical loads only",
            };
        }

        // Gate 2: Bus degraded → derated operation
        if self.bus_status == BusStatus::Degraded {
            return PowerRoute {
                source: EnergyTier::MainBusDerated,
                fallback: EnergyTier::AuxLfp,
                max_available_kw: 15.0,
                rationale: "Main bus degraded — derate to 50% max power",
            };
        }

        // Gate 3: Burst demand → ultracap if available
        if self.duration_s <= 5.0 && self.demand_kw > 5.0 {
            if self.ultracap_soc > 0.2 {
                return PowerRoute {
                    source: EnergyTier::UltracapBank,
                    fallback: EnergyTier::MainBus,
                    max_available_kw: 10.0,
                    rationale: "Short burst >5kW — ultracaps handle transient",
                };
            }
            return PowerRoute {
                source: EnergyTier::MainBus,
                fallback: EnergyTier::AuxLfp,
                max_available_kw: 30.0,
                rationale: "Burst demand but ultracaps depleted — main bus direct",
            };
        }

        // Gate 4: Local limb demand ≤2kW with buffer available
        if self.locality != Limb::Distributed
            && self.demand_kw <= 2.0
            && self.limb_buffer_soc > 0.15
        {
            return PowerRoute {
                source: EnergyTier::PerLimbLfp,
                fallback: EnergyTier::MainBus,
                max_available_kw: 2.0,
                rationale: "Local demand ≤2kW — per-limb buffer handles without bus current",
            };
        }

        // Default: main bus steady state
        PowerRoute {
            source: EnergyTier::MainBus,
            fallback: EnergyTier::UltracapBank,
            max_available_kw: 30.0,
            rationale: "Steady-state distributed demand — main NMC pack",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demand(kw: f64, dur: f64, loc: Limb, bus: BusStatus, ucap: f64, limb: f64) -> PowerDemand {
        PowerDemand {
            demand_kw: kw,
            duration_s: dur,
            locality: loc,
            bus_status: bus,
            ultracap_soc: ucap,
            limb_buffer_soc: limb,
        }
    }

    #[test]
    fn test_bus_failure_routes_to_aux() {
        let d = demand(1.0, 10.0, Limb::Distributed, BusStatus::Failed, 0.8, 0.5);
        assert_eq!(d.route().source, EnergyTier::AuxLfp);
    }

    #[test]
    fn test_degraded_routes_to_derated() {
        let d = demand(5.0, 10.0, Limb::Distributed, BusStatus::Degraded, 0.8, 0.5);
        assert_eq!(d.route().source, EnergyTier::MainBusDerated);
    }

    #[test]
    fn test_burst_routes_to_ultracap() {
        let d = demand(8.0, 2.0, Limb::Distributed, BusStatus::Nominal, 0.8, 0.5);
        assert_eq!(d.route().source, EnergyTier::UltracapBank);
    }

    #[test]
    fn test_burst_empty_ultracap_routes_to_main() {
        let d = demand(8.0, 2.0, Limb::Distributed, BusStatus::Nominal, 0.1, 0.5);
        assert_eq!(d.route().source, EnergyTier::MainBus);
    }

    #[test]
    fn test_local_limb_routes_to_buffer() {
        let d = demand(1.5, 10.0, Limb::LeftArm, BusStatus::Nominal, 0.8, 0.6);
        assert_eq!(d.route().source, EnergyTier::PerLimbLfp);
    }

    #[test]
    fn test_local_limb_empty_buffer_routes_to_main() {
        let d = demand(1.5, 10.0, Limb::LeftArm, BusStatus::Nominal, 0.8, 0.1);
        assert_eq!(d.route().source, EnergyTier::MainBus);
    }

    #[test]
    fn test_steady_state_routes_to_main() {
        let d = demand(5.0, 60.0, Limb::Distributed, BusStatus::Nominal, 0.8, 0.5);
        assert_eq!(d.route().source, EnergyTier::MainBus);
    }
}
