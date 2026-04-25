//! Four control loops — one per compound. Each ticks at its own cadence,
//! reads sensor / state inputs, runs the compound's update step, and writes
//! a fresh snapshot to shared state.
//!
//! Loops in v0.1 use mocked sensor inputs — replace `mock_*` calls with
//! real hardware bindings when available.

use crate::state::{
    ControlSnapshot, HumanInterfaceSnapshot, PerceptionSnapshot, PowerSnapshot, StationState,
};
use std::sync::Arc;
use std::time::Duration;
use suit_perception::vestibular::{BarometricData, InertialState, MagnetometerData};
use suit_perception::proprioceptive::BodyState;
use tracing::info;

const PERCEPTION_TICK_MS: u64 = 50; // 20 Hz
const POWER_TICK_MS: u64 = 100; //   10 Hz
const CONTROL_TICK_MS: u64 = 100; //   10 Hz
const HI_TICK_MS: u64 = 200; //         5 Hz

/// Perception loop — sensor fusion at 20 Hz.
pub async fn run_perception(state: Arc<StationState>) {
    let mut engine = suit_perception::perception_engine::PerceptionEngine::new(
        "model/intent.bin".to_string(),
    );
    let mut tick = 0u64;
    let mut interval = tokio::time::interval(Duration::from_millis(PERCEPTION_TICK_MS));
    loop {
        interval.tick().await;
        tick += 1;

        let world = engine.update(
            &InertialState::default(),
            &MagnetometerData { field: [0.0; 3] },
            &BarometricData {
                pressure: 1013.25,
                temperature: 20.0,
            },
            &BodyState {
                joint_angles: vec![],
                joint_velocities: vec![],
                heart_rate: 60,
                spo2: 98,
                foot_pressure: vec![],
            },
            &None,
        );

        let snap = PerceptionSnapshot {
            tick,
            heading_rad: world.attitude[2], // yaw
            altitude_m: world.position[2],
            intent: "Standing",
        };
        *state.perception.write().await = snap;
        if tick % 20 == 0 {
            info!(tick, "perception_loop");
        }
    }
}

/// Power loop — SOC / load prioritization at 10 Hz.
pub async fn run_power(state: Arc<StationState>) {
    let mut engine = suit_power::engine::PowerEngine::new();
    let forecast = suit_power::mission::MissionForecast {
        load_forecast: vec![],
    };
    let mut tick = 0u64;
    let mut interval = tokio::time::interval(Duration::from_millis(POWER_TICK_MS));
    loop {
        interval.tick().await;
        tick += 1;

        // Mock: 400 V bus, 10 A draw, 25 °C, 8 kW available.
        let (soc, _shed) = engine.update(400.0, 10.0, 25.0, 8000.0, &forecast);

        let snap = PowerSnapshot {
            tick,
            soc_pct: soc.soc,
            health: soc.health,
            current_tier: load_tier_label(engine.prioritizer.current_tier as u8),
            power_state: power_state_label(engine.sequencer.state as u8),
        };
        *state.power.write().await = snap;
        if tick % 10 == 0 {
            info!(tick, soc = soc.soc, "power_loop");
        }
    }
}

/// Control loop — flight target translation at 10 Hz, reads perception state.
pub async fn run_control(state: Arc<StationState>) {
    let mut tick = 0u64;
    let mut interval = tokio::time::interval(Duration::from_millis(CONTROL_TICK_MS));
    loop {
        interval.tick().await;
        tick += 1;

        // Read latest perception snapshot for telemetry; v0.1 emits a fixed
        // command (no real Intent classifier hooked up yet).
        let _peek = state.perception.read().await.clone();
        let snap = ControlSnapshot {
            tick,
            target_vector: [0.0, 0.0, 1.0], // hover target
        };
        *state.control.write().await = snap;
        if tick % 10 == 0 {
            info!(tick, "control_loop");
        }
    }
}

/// Human-interface loop — safety + thermal at 5 Hz, polls watchdog.
pub async fn run_human_interface(state: Arc<StationState>) {
    let mut controller = suit_safety::e_stop::EStopController {
        watchdog: MockWatchdog::default(),
    };
    let mut tick = 0u64;
    let mut interval = tokio::time::interval(Duration::from_millis(HI_TICK_MS));
    loop {
        interval.tick().await;
        tick += 1;

        controller.poll();

        let snap = HumanInterfaceSnapshot {
            tick,
            estop_status: "armed",
            thermal_action: "Nominal",
            watchdog_kicks: tick,
        };
        *state.human_interface.write().await = snap;
        if tick % 5 == 0 {
            info!(tick, "human_interface_loop");
        }
    }
}

#[derive(Default)]
struct MockWatchdog;

impl suit_safety::hardware_watchdog::HardwareWatchdog for MockWatchdog {
    fn kick(&mut self) {}
    fn trigger_emergency_stop(&mut self) {}
}

fn load_tier_label(t: u8) -> &'static str {
    match t {
        0 => "Comms",
        1 => "Compute",
        2 => "Actuation",
        3 => "Critical",
        _ => "Unknown",
    }
}

fn power_state_label(s: u8) -> &'static str {
    match s {
        0 => "Nominal",
        1 => "Caution",
        2 => "Critical",
        3 => "Emergency",
        _ => "Unknown",
    }
}
