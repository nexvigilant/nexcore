//! Shared station state — one snapshot per compound, updated atomically by
//! its corresponding control loop and read by MCP tool handlers.

use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Latest perception snapshot. Heading + altitude are the canonical handles
/// agents query; full WorldState lives only inside the loop.
#[derive(Debug, Clone, Serialize)]
pub struct PerceptionSnapshot {
    /// Tick counter (monotonic).
    pub tick: u64,
    /// Heading in radians, magnetic North.
    pub heading_rad: f32,
    /// Altitude AGL in meters.
    pub altitude_m: f32,
    /// Last classified intent label.
    pub intent: &'static str,
}

impl Default for PerceptionSnapshot {
    fn default() -> Self {
        Self {
            tick: 0,
            heading_rad: 0.0,
            altitude_m: 0.0,
            intent: "Unknown",
        }
    }
}

/// Latest power snapshot — SOC + current load tier + degradation state.
#[derive(Debug, Clone, Serialize)]
pub struct PowerSnapshot {
    /// Tick counter.
    pub tick: u64,
    /// State of charge percentage (0–100).
    pub soc_pct: f32,
    /// Battery health (0.0–1.0).
    pub health: f32,
    /// Active load tier (e.g. "Comms", "Compute").
    pub current_tier: &'static str,
    /// Power state (Nominal / Caution / Critical / Emergency).
    pub power_state: &'static str,
}

impl Default for PowerSnapshot {
    fn default() -> Self {
        Self {
            tick: 0,
            soc_pct: 100.0,
            health: 1.0,
            current_tier: "Comms",
            power_state: "Nominal",
        }
    }
}

/// Latest control snapshot — last flight command computed from perception.
#[derive(Debug, Clone, Serialize)]
pub struct ControlSnapshot {
    /// Tick counter.
    pub tick: u64,
    /// Commanded target vector [x, y, z].
    pub target_vector: [f32; 3],
}

impl Default for ControlSnapshot {
    fn default() -> Self {
        Self {
            tick: 0,
            target_vector: [0.0, 0.0, 0.0],
        }
    }
}

/// Latest human-interface snapshot — safety verdict + thermal action.
#[derive(Debug, Clone, Serialize)]
pub struct HumanInterfaceSnapshot {
    /// Tick counter.
    pub tick: u64,
    /// E-stop status: `armed` (default) or `triggered`.
    pub estop_status: &'static str,
    /// Thermal action label (e.g. "Nominal", "Throttle", "Shutdown").
    pub thermal_action: &'static str,
    /// Watchdog kicks since station start.
    pub watchdog_kicks: u64,
}

impl Default for HumanInterfaceSnapshot {
    fn default() -> Self {
        Self {
            tick: 0,
            estop_status: "armed",
            thermal_action: "Nominal",
            watchdog_kicks: 0,
        }
    }
}

/// Whole-station snapshot — combines all 4 compound snapshots in one struct.
#[derive(Debug, Clone, Serialize)]
pub struct StationSnapshot {
    /// Perception compound state.
    pub perception: PerceptionSnapshot,
    /// Power compound state.
    pub power: PowerSnapshot,
    /// Control compound state.
    pub control: ControlSnapshot,
    /// Human-interface compound state.
    pub human_interface: HumanInterfaceSnapshot,
    /// Total ticks across all compounds.
    pub total_ticks: u64,
    /// Compound count (always 4).
    pub compound_count: u32,
}

/// Mutable shared station state.
#[derive(Debug, Default)]
pub struct StationState {
    /// Perception compound snapshot, updated by perception loop.
    pub perception: RwLock<PerceptionSnapshot>,
    /// Power compound snapshot, updated by power loop.
    pub power: RwLock<PowerSnapshot>,
    /// Control compound snapshot, updated by control loop.
    pub control: RwLock<ControlSnapshot>,
    /// Human-interface compound snapshot, updated by HI loop.
    pub human_interface: RwLock<HumanInterfaceSnapshot>,
}

impl StationState {
    /// Construct a fresh shared state with default snapshots.
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Take a coherent point-in-time snapshot across all 4 compounds.
    /// Each lock is acquired in turn — readers are cheap, contention low.
    pub async fn snapshot(&self) -> StationSnapshot {
        let perception = self.perception.read().await.clone();
        let power = self.power.read().await.clone();
        let control = self.control.read().await.clone();
        let human_interface = self.human_interface.read().await.clone();
        let total_ticks =
            perception.tick + power.tick + control.tick + human_interface.tick;
        StationSnapshot {
            perception,
            power,
            control,
            human_interface,
            total_ticks,
            compound_count: 4,
        }
    }
}
