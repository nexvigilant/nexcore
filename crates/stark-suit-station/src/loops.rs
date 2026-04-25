//! Four control loops — one per compound. Each ticks at its own cadence,
//! reads sensor / state inputs, runs the compound's update step, and writes
//! a fresh snapshot to shared state.
//!
//! Loops in v0.1 use mocked sensor inputs — replace `mock_*` calls with
//! real hardware bindings when available.

use crate::bms::{BmsError, BmsSource};
use crate::perception::{PerceptionError, PerceptionSource};
use crate::state::{
    ControlSnapshot, HumanInterfaceSnapshot, PerceptionSnapshot, PowerSnapshot, StationState,
};
use std::sync::Arc;
use std::time::Duration;
use suit_perception::proprioceptive::BodyState;
use suit_perception::vestibular::{BarometricData, InertialState, MagnetometerData};
use tracing::{info, warn};

const PERCEPTION_TICK_MS: u64 = 50; // 20 Hz
const POWER_TICK_MS: u64 = 100; //   10 Hz
const CONTROL_TICK_MS: u64 = 100; //   10 Hz
const HI_TICK_MS: u64 = 200; //         5 Hz

/// Perception loop — pulls one fused frame per tick from a `PerceptionSource`
/// at 20 Hz, runs the fusion engine, writes a snapshot. v0.6: source is
/// parameterized; default backend is `MockPerceptionSource`. Loop never
/// panics — every error path preserves the last-known snapshot.
pub async fn run_perception(state: Arc<StationState>, source: Arc<dyn PerceptionSource>) {
    let mut engine = suit_perception::perception_engine::PerceptionEngine::new(
        "model/intent.bin".to_string(),
    );
    let mut tick = 0u64;
    let mut interval = tokio::time::interval(Duration::from_millis(PERCEPTION_TICK_MS));
    loop {
        interval.tick().await;
        tick += 1;

        match source.poll().await {
            Ok(frame) => {
                let world = engine.update(
                    &InertialState {
                        acceleration: frame.accel_mps2,
                        angular_velocity: frame.gyro_radps,
                        ..Default::default()
                    },
                    &MagnetometerData { field: frame.mag_ut },
                    &BarometricData {
                        pressure: frame.pressure_hpa,
                        temperature: frame.temp_c,
                    },
                    &BodyState {
                        joint_angles: vec![],
                        joint_velocities: vec![],
                        heart_rate: frame.heart_rate_bpm as u8,
                        spo2: frame.spo2_pct,
                        foot_pressure: vec![],
                    },
                    &None,
                );

                let snap = PerceptionSnapshot {
                    tick,
                    heading_rad: world.attitude[2],
                    altitude_m: world.position[2],
                    intent: "Standing",
                };
                *state.perception.write().await = snap;
                if tick % 20 == 0 {
                    info!(tick, hr = frame.heart_rate_bpm, "perception_loop");
                }
            }
            Err(PerceptionError::Exhausted | PerceptionError::ReplayExhausted) => {
                warn!(tick, "perception source exhausted — halting loop");
                break;
            }
            Err(other) => {
                warn!(tick, error = %other, "perception unexpected error — halting loop");
                break;
            }
        }
    }
}

/// Power loop — pulls one frame from a `BmsSource` per tick at 10 Hz.
///
/// v0.2: source is parameterized; default backend is `MockBmsSource`. The
/// loop never panics — every error path either preserves the last-known
/// snapshot or marks `power_state` as `Fault`.
pub async fn run_power(state: Arc<StationState>, source: Arc<dyn BmsSource>) {
    let mut tick = 0u64;
    let mut interval = tokio::time::interval(Duration::from_millis(POWER_TICK_MS));
    loop {
        interval.tick().await;
        tick += 1;

        match source.poll().await {
            Ok(frame) => {
                let snap = PowerSnapshot {
                    tick,
                    soc_pct: frame.soc_pct,
                    health: frame.soh_pct / 100.0,
                    current_tier: frame.tier.label(),
                    power_state: classify_power_state(frame.cell_temp_c, frame.soc_pct),
                };
                *state.power.write().await = snap;
                if tick % 10 == 0 {
                    info!(tick, soc = frame.soc_pct, "power_loop");
                }
            }
            Err(BmsError::Timeout) => {
                warn!(tick, "bms timeout — degrading to last known");
                state.power.write().await.tick = tick;
            }
            Err(BmsError::Fault(code)) => {
                warn!(tick, code, "bms fault — marking power_state Fault");
                let mut snap = state.power.write().await;
                snap.tick = tick;
                snap.power_state = "Fault";
            }
            Err(BmsError::Exhausted | BmsError::ReplayExhausted) => {
                warn!(tick, "bms source exhausted — halting power loop");
                break;
            }
            Err(other) => {
                warn!(tick, error = %other, "bms unexpected error — halting power loop");
                break;
            }
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

/// Classify power state from BMS frame readings.
fn classify_power_state(cell_temp_c: f32, soc_pct: f32) -> &'static str {
    if cell_temp_c >= 60.0 || soc_pct <= 5.0 {
        "Emergency"
    } else if cell_temp_c >= 45.0 || soc_pct <= 15.0 {
        "Critical"
    } else if cell_temp_c >= 35.0 || soc_pct <= 30.0 {
        "Caution"
    } else {
        "Nominal"
    }
}
