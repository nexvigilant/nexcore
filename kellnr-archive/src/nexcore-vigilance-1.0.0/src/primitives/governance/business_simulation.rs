//! # Business Simulation Engine (Djed Typestate Pattern)
//!
//! Implementation of the NEX-SIM-001 enterprise growth simulation.
//! Uses typestate pattern for compile-time phase transition enforcement.
//!
//! ## Djed Pattern Applied
//!
//! The `GrowthPhase` enum is replaced with zero-sized marker types.
//! `SimulationParameters<S>` is generic over the phase state.
//! Phase transitions consume `self` and return the next state.
//! Terminal state (`MarketLeadership`) has no `advance()` method —
//! calling it on a terminal state is a compile error.
//!
//! ## Tier Classification
//!
//! | Type | Tier | Grounding |
//! |------|------|-----------|
//! | Phase markers | T1 | ς State |
//! | SimulationParameters\<S\> | T3 | ς+σ State+Sequence |
//! | GrowthPhaseSnapshot | T2-P | κ Comparison (for serde) |

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

// ============================================================================
// Sealed Trait + GrowthPhaseState
// ============================================================================

mod private {
    pub trait Sealed {}
}

/// Trait for growth phase state markers. Sealed — cannot be implemented outside this module.
pub trait GrowthPhaseState: private::Sealed {
    /// Human-readable name of this phase.
    fn name() -> &'static str;
    /// Ordinal position (0-based).
    fn ordinal() -> u8;
    /// Whether this is a terminal (non-advanceable) state.
    fn is_terminal() -> bool;
}

// ============================================================================
// Zero-Sized Phase Markers
// ============================================================================

/// Phase 0: Foundation — initial bootstrap state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Foundation;

/// Phase 1: Capability Build — constructing core capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityBuild;

/// Phase 2: Market Entry — first customer-facing operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarketEntry;

/// Phase 3: Scaling — growth beyond initial market.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scaling;

/// Phase 4: Market Leadership — terminal state, peak maturity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarketLeadership;

// Seal all markers
impl private::Sealed for Foundation {}
impl private::Sealed for CapabilityBuild {}
impl private::Sealed for MarketEntry {}
impl private::Sealed for Scaling {}
impl private::Sealed for MarketLeadership {}

// Implement GrowthPhaseState for each marker
impl GrowthPhaseState for Foundation {
    fn name() -> &'static str {
        "Foundation"
    }
    fn ordinal() -> u8 {
        0
    }
    fn is_terminal() -> bool {
        false
    }
}

impl GrowthPhaseState for CapabilityBuild {
    fn name() -> &'static str {
        "CapabilityBuild"
    }
    fn ordinal() -> u8 {
        1
    }
    fn is_terminal() -> bool {
        false
    }
}

impl GrowthPhaseState for MarketEntry {
    fn name() -> &'static str {
        "MarketEntry"
    }
    fn ordinal() -> u8 {
        2
    }
    fn is_terminal() -> bool {
        false
    }
}

impl GrowthPhaseState for Scaling {
    fn name() -> &'static str {
        "Scaling"
    }
    fn ordinal() -> u8 {
        3
    }
    fn is_terminal() -> bool {
        false
    }
}

impl GrowthPhaseState for MarketLeadership {
    fn name() -> &'static str {
        "MarketLeadership"
    }
    fn ordinal() -> u8 {
        4
    }
    fn is_terminal() -> bool {
        true
    }
}

// ============================================================================
// Serde Snapshot (for storage/serialization)
// ============================================================================

/// Serde-friendly enum for persisting/transmitting growth phase.
///
/// Use this for serialization; use the typestate markers for state machine logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GrowthPhaseSnapshot {
    /// Ordinal 0
    Foundation = 0,
    /// Ordinal 1
    CapabilityBuild = 1,
    /// Ordinal 2
    MarketEntry = 2,
    /// Ordinal 3
    Scaling = 3,
    /// Ordinal 4
    MarketLeadership = 4,
}

// Backward compatibility: re-export the snapshot as GrowthPhase
/// Backward-compatible alias for serialization-friendly growth phase.
pub type GrowthPhase = GrowthPhaseSnapshot;

// ============================================================================
// SimulationParameters<S> (Typestate FSM)
// ============================================================================

/// T3: SimulationParameters — generic over growth phase state.
///
/// Phase transitions consume `self` and return a new `SimulationParameters`
/// in the next state. Terminal state (`MarketLeadership`) has no `advance()`.
#[derive(Debug, Clone)]
pub struct SimulationParameters<S: GrowthPhaseState> {
    /// Simulation identifier
    pub simulation_id: String,
    /// Whether in bootstrap mode (auto-disabled at MarketEntry→Scaling transition)
    pub bootstrap_mode: bool,
    /// Revenue model descriptor
    pub revenue_model: String,
    /// Zero-sized phase marker
    _phase: PhantomData<S>,
}

impl SimulationParameters<Foundation> {
    /// Create a new simulation in Foundation phase.
    pub fn new_default() -> Self {
        Self {
            simulation_id: "NEX-SIM-001".into(),
            bootstrap_mode: true,
            revenue_model: "subscription_freemium_enterprise".into(),
            _phase: PhantomData,
        }
    }

    /// Advance: Foundation → CapabilityBuild
    pub fn advance(self) -> SimulationParameters<CapabilityBuild> {
        SimulationParameters {
            simulation_id: self.simulation_id,
            bootstrap_mode: self.bootstrap_mode,
            revenue_model: self.revenue_model,
            _phase: PhantomData,
        }
    }
}

impl SimulationParameters<CapabilityBuild> {
    /// Advance: CapabilityBuild → MarketEntry
    pub fn advance(self) -> SimulationParameters<MarketEntry> {
        SimulationParameters {
            simulation_id: self.simulation_id,
            bootstrap_mode: self.bootstrap_mode,
            revenue_model: self.revenue_model,
            _phase: PhantomData,
        }
    }
}

impl SimulationParameters<MarketEntry> {
    /// Advance: MarketEntry → Scaling (disables bootstrap mode)
    pub fn advance(self) -> SimulationParameters<Scaling> {
        SimulationParameters {
            simulation_id: self.simulation_id,
            bootstrap_mode: false, // Bootstrap ends at scaling
            revenue_model: self.revenue_model,
            _phase: PhantomData,
        }
    }
}

impl SimulationParameters<Scaling> {
    /// Advance: Scaling → MarketLeadership
    pub fn advance(self) -> SimulationParameters<MarketLeadership> {
        SimulationParameters {
            simulation_id: self.simulation_id,
            bootstrap_mode: self.bootstrap_mode,
            revenue_model: self.revenue_model,
            _phase: PhantomData,
        }
    }
}

// MarketLeadership: NO advance() method — compile error if attempted.

// ============================================================================
// Snapshot Adapter (typestate ↔ serde)
// ============================================================================

/// Serde-friendly flat simulation state for storage in Union and persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSnapshot {
    /// Simulation identifier
    pub simulation_id: String,
    /// Current phase (as enum for serde)
    pub phase: GrowthPhaseSnapshot,
    /// Whether in bootstrap mode
    pub bootstrap_mode: bool,
    /// Revenue model descriptor
    pub revenue_model: String,
}

impl<S: GrowthPhaseState> SimulationParameters<S> {
    /// Convert typestate parameters to a serde-friendly snapshot.
    pub fn to_snapshot(&self) -> SimulationSnapshot {
        let phase = match S::ordinal() {
            0 => GrowthPhaseSnapshot::Foundation,
            1 => GrowthPhaseSnapshot::CapabilityBuild,
            2 => GrowthPhaseSnapshot::MarketEntry,
            3 => GrowthPhaseSnapshot::Scaling,
            _ => GrowthPhaseSnapshot::MarketLeadership,
        };
        SimulationSnapshot {
            simulation_id: self.simulation_id.clone(),
            phase,
            bootstrap_mode: self.bootstrap_mode,
            revenue_model: self.revenue_model.clone(),
        }
    }
}

/// Type-erased simulation for runtime dispatch (deserialization target).
pub enum AnySimulation {
    /// Foundation phase
    Foundation(SimulationParameters<Foundation>),
    /// CapabilityBuild phase
    CapabilityBuild(SimulationParameters<CapabilityBuild>),
    /// MarketEntry phase
    MarketEntry(SimulationParameters<MarketEntry>),
    /// Scaling phase
    Scaling(SimulationParameters<Scaling>),
    /// MarketLeadership phase
    MarketLeadership(SimulationParameters<MarketLeadership>),
}

impl AnySimulation {
    /// Reconstruct a typed simulation from a snapshot.
    pub fn from_snapshot(snap: &SimulationSnapshot) -> Self {
        match snap.phase {
            GrowthPhaseSnapshot::Foundation => AnySimulation::Foundation(SimulationParameters {
                simulation_id: snap.simulation_id.clone(),
                bootstrap_mode: snap.bootstrap_mode,
                revenue_model: snap.revenue_model.clone(),
                _phase: PhantomData,
            }),
            GrowthPhaseSnapshot::CapabilityBuild => {
                AnySimulation::CapabilityBuild(SimulationParameters {
                    simulation_id: snap.simulation_id.clone(),
                    bootstrap_mode: snap.bootstrap_mode,
                    revenue_model: snap.revenue_model.clone(),
                    _phase: PhantomData,
                })
            }
            GrowthPhaseSnapshot::MarketEntry => AnySimulation::MarketEntry(SimulationParameters {
                simulation_id: snap.simulation_id.clone(),
                bootstrap_mode: snap.bootstrap_mode,
                revenue_model: snap.revenue_model.clone(),
                _phase: PhantomData,
            }),
            GrowthPhaseSnapshot::Scaling => AnySimulation::Scaling(SimulationParameters {
                simulation_id: snap.simulation_id.clone(),
                bootstrap_mode: snap.bootstrap_mode,
                revenue_model: snap.revenue_model.clone(),
                _phase: PhantomData,
            }),
            GrowthPhaseSnapshot::MarketLeadership => {
                AnySimulation::MarketLeadership(SimulationParameters {
                    simulation_id: snap.simulation_id.clone(),
                    bootstrap_mode: snap.bootstrap_mode,
                    revenue_model: snap.revenue_model.clone(),
                    _phase: PhantomData,
                })
            }
        }
    }

    /// Convert back to snapshot.
    pub fn to_snapshot(&self) -> SimulationSnapshot {
        match self {
            AnySimulation::Foundation(s) => s.to_snapshot(),
            AnySimulation::CapabilityBuild(s) => s.to_snapshot(),
            AnySimulation::MarketEntry(s) => s.to_snapshot(),
            AnySimulation::Scaling(s) => s.to_snapshot(),
            AnySimulation::MarketLeadership(s) => s.to_snapshot(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_lifecycle() {
        let sim = SimulationParameters::<Foundation>::new_default();
        assert!(sim.bootstrap_mode);
        assert_eq!(Foundation::ordinal(), 0);

        let sim = sim.advance(); // → CapabilityBuild
        assert!(sim.bootstrap_mode);
        assert_eq!(CapabilityBuild::ordinal(), 1);

        let sim = sim.advance(); // → MarketEntry
        assert!(sim.bootstrap_mode);
        assert_eq!(MarketEntry::ordinal(), 2);

        let sim = sim.advance(); // → Scaling (bootstrap disabled)
        assert!(!sim.bootstrap_mode);
        assert_eq!(Scaling::ordinal(), 3);

        let sim = sim.advance(); // → MarketLeadership (terminal)
        assert!(!sim.bootstrap_mode);
        assert_eq!(MarketLeadership::ordinal(), 4);
        assert!(MarketLeadership::is_terminal());

        // sim.advance() would be a compile error here — no advance() on MarketLeadership
    }

    #[test]
    fn test_snapshot_round_trip() {
        let sim = SimulationParameters::<Foundation>::new_default();
        let snap = sim.to_snapshot();
        assert_eq!(snap.phase, GrowthPhaseSnapshot::Foundation);
        assert!(snap.bootstrap_mode);

        // Advance through all phases and verify snapshots
        let sim2 = sim.advance().advance(); // → MarketEntry
        let snap2 = sim2.to_snapshot();
        assert_eq!(snap2.phase, GrowthPhaseSnapshot::MarketEntry);
    }

    #[test]
    fn test_any_simulation_round_trip() {
        let snap = SimulationSnapshot {
            simulation_id: "NEX-SIM-001".into(),
            phase: GrowthPhaseSnapshot::Scaling,
            bootstrap_mode: false,
            revenue_model: "enterprise".into(),
        };

        let any = AnySimulation::from_snapshot(&snap);
        let snap2 = any.to_snapshot();
        assert_eq!(snap2.phase, GrowthPhaseSnapshot::Scaling);
        assert!(!snap2.bootstrap_mode);
    }

    #[test]
    fn test_phase_names() {
        assert_eq!(Foundation::name(), "Foundation");
        assert_eq!(CapabilityBuild::name(), "CapabilityBuild");
        assert_eq!(MarketEntry::name(), "MarketEntry");
        assert_eq!(Scaling::name(), "Scaling");
        assert_eq!(MarketLeadership::name(), "MarketLeadership");
    }

    #[test]
    fn test_terminal_detection() {
        assert!(!Foundation::is_terminal());
        assert!(!CapabilityBuild::is_terminal());
        assert!(!MarketEntry::is_terminal());
        assert!(!Scaling::is_terminal());
        assert!(MarketLeadership::is_terminal());
    }

    #[test]
    fn test_growth_phase_snapshot_ordering() {
        assert!(GrowthPhaseSnapshot::Foundation < GrowthPhaseSnapshot::CapabilityBuild);
        assert!(GrowthPhaseSnapshot::CapabilityBuild < GrowthPhaseSnapshot::MarketEntry);
        assert!(GrowthPhaseSnapshot::MarketEntry < GrowthPhaseSnapshot::Scaling);
        assert!(GrowthPhaseSnapshot::Scaling < GrowthPhaseSnapshot::MarketLeadership);
    }

    #[test]
    fn test_backward_compat_growth_phase_alias() {
        // GrowthPhase is now a type alias for GrowthPhaseSnapshot
        let phase: GrowthPhase = GrowthPhase::Foundation;
        assert_eq!(phase, GrowthPhaseSnapshot::Foundation);
    }
}
