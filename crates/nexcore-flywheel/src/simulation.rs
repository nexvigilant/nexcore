// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Simulation engine — time-step evolution of flywheel state.
//!
//! Evolves `FlywheelState` forward through discrete time steps, applying
//! the governing equations and loop cascade at each step.
//!
//! ## T1 Primitive Grounding: σ (Sequence) + ν (Frequency) + ς (State)
//!
//! ## Design
//!
//! The engine is deterministic: same initial state + same scenario = same trajectory.
//! No randomness, no external I/O. Pure state evolution.

use crate::equations::{self, FlywheelPhysics, PhysicsInput};
use crate::loops::{
    self, CascadeInput, CascadeResult, ElasticInput, FrictionInput, GyroscopicInput, MomentumInput,
    RimInput, SystemState,
};
use crate::thresholds::FlywheelThresholds;
use serde::{Deserialize, Serialize};

/// Complete flywheel state at a single point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlywheelState {
    /// Current time step index.
    pub step: u64,
    /// Elapsed simulation time.
    pub time: f64,
    /// Physics snapshot (governing equations).
    pub physics: FlywheelPhysics,
    /// Loop cascade result.
    pub cascade: CascadeResult,
    /// Physics input (mutable — scenario can modify).
    pub physics_input: PhysicsInput,
    /// Loop cascade input (derived from physics + scenario).
    pub cascade_input: CascadeInput,
}

/// A recorded trajectory — the full simulation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trajectory {
    /// Configuration used.
    pub config: SimulationConfig,
    /// Every state snapshot in sequence.
    pub states: Vec<FlywheelState>,
    /// Final system state.
    pub final_state: SystemState,
    /// Step at which Loop 4 (gyroscopic stability) first activated, if ever.
    pub gyroscopic_activation_step: Option<u64>,
    /// Step at which system first entered Failed state, if ever.
    pub failure_step: Option<u64>,
}

/// Simulation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Time step size (dt).
    pub dt: f64,
    /// Total number of steps to simulate.
    pub steps: u64,
    /// Flywheel thresholds for loop evaluation.
    pub thresholds: FlywheelThresholds,
    /// Scenario: external forces applied per step.
    pub scenario: Scenario,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            dt: 1.0,
            steps: 100,
            thresholds: FlywheelThresholds::default(),
            scenario: Scenario::SteadyState,
        }
    }
}

/// Predefined scenarios (spec §Phase 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Scenario {
    /// Cold start: low mass, low omega, building up.
    ColdStart,
    /// Steady state: constant inputs, observe natural dynamics.
    SteadyState,
    /// Stress test: increasing perturbation torque over time.
    StressTest {
        /// Perturbation increment per step.
        perturbation_rate: f64,
    },
    /// Growth: mass and omega increase over time.
    Growth {
        /// Mass growth rate per step.
        mass_rate: f64,
        /// Omega growth rate per step.
        omega_rate: f64,
    },
    /// Fatigue: constant load, cycling stress to test Loop 5.
    Fatigue {
        /// Cycles added per step.
        cycles_per_step: u32,
    },
    /// Friction spike: automation coverage drops suddenly at a given step.
    FrictionSpike {
        /// Step at which the spike occurs.
        spike_step: u64,
        /// Automation coverage after spike (0.0-1.0).
        post_spike_coverage: f64,
    },
}

/// Derive CascadeInput from PhysicsInput + scenario state.
fn derive_cascade_input(
    physics: &FlywheelPhysics,
    physics_input: &PhysicsInput,
    fatigue_cycles: u32,
    automation_coverage: f64,
    perturbation_torque: f64,
) -> CascadeInput {
    // Rim: tensile_strength from yield_stress, centrifugal from stress
    let rim = RimInput {
        tensile_strength: physics_input.yield_stress,
        centrifugal_force: physics.stress,
    };

    // Momentum: directly from physics
    let momentum = MomentumInput {
        inertia: physics.inertia,
        omega: physics_input.omega,
        friction_drain: 0.0, // cascade computes this from friction loop
    };

    // Friction: map physics to operational overhead
    let friction = FrictionInput {
        manual_processes: (1.0 - automation_coverage) * 10.0,
        human_touchpoints: (1.0 - automation_coverage) * 10.0,
        velocity: physics_input.omega,
        automation_coverage,
    };

    // Gyroscopic: momentum vs perturbation
    let gyroscopic = GyroscopicInput {
        momentum_l: physics.momentum,
        perturbation_torque,
        critical_momentum: 50.0,
    };

    // Elastic: stress vs yield, fatigue tracking
    let elastic = ElasticInput {
        stress: physics.stress,
        yield_point: physics_input.yield_stress,
        fatigue_cycles,
        fatigue_limit: 1000,
    };

    CascadeInput {
        rim,
        momentum,
        friction,
        gyroscopic,
        elastic,
    }
}

/// Initial state for a scenario.
fn initial_physics_input(scenario: &Scenario) -> PhysicsInput {
    match scenario {
        Scenario::ColdStart => PhysicsInput {
            mass: 10.0,
            radius: 0.5,
            omega: 0.5,
            density: 1.0,
            drag_coefficient: 0.001,
            yield_stress: 1000.0,
        },
        Scenario::SteadyState => PhysicsInput::default(),
        Scenario::StressTest { .. } => PhysicsInput::default(),
        Scenario::Growth { .. } => PhysicsInput {
            mass: 20.0,
            radius: 1.0,
            omega: 1.0,
            density: 1.0,
            drag_coefficient: 0.001,
            yield_stress: 1000.0,
        },
        Scenario::Fatigue { .. } => PhysicsInput::default(),
        Scenario::FrictionSpike { .. } => PhysicsInput {
            mass: 100.0,
            radius: 1.0,
            omega: 5.0,
            density: 1.0,
            drag_coefficient: 0.001,
            yield_stress: 1000.0,
        },
    }
}

/// Run a complete simulation, producing a trajectory.
#[must_use]
pub fn run(config: &SimulationConfig) -> Trajectory {
    let mut states = Vec::with_capacity(config.steps as usize);
    let mut physics_input = initial_physics_input(&config.scenario);
    let mut l_previous = 0.0;
    let mut fatigue_cycles: u32 = 0;
    let mut automation_coverage = 0.8;
    let mut perturbation_torque = 10.0;
    let mut gyroscopic_activation_step = None;
    let mut failure_step = None;

    for step in 0..config.steps {
        let time = step as f64 * config.dt;

        // Apply scenario modifications
        match &config.scenario {
            Scenario::ColdStart => {
                // Gradually increase mass and omega
                physics_input.mass += 1.0 * config.dt;
                physics_input.omega += 0.1 * config.dt;
            }
            Scenario::SteadyState => {}
            Scenario::StressTest { perturbation_rate } => {
                perturbation_torque += perturbation_rate * config.dt;
            }
            Scenario::Growth {
                mass_rate,
                omega_rate,
            } => {
                physics_input.mass += mass_rate * config.dt;
                physics_input.omega += omega_rate * config.dt;
            }
            Scenario::Fatigue { cycles_per_step } => {
                fatigue_cycles = fatigue_cycles.saturating_add(*cycles_per_step);
            }
            Scenario::FrictionSpike {
                spike_step,
                post_spike_coverage,
            } => {
                if step >= *spike_step {
                    automation_coverage = *post_spike_coverage;
                }
            }
        }

        // Compute physics
        let physics = equations::evaluate(&physics_input, l_previous, config.dt);
        l_previous = physics.momentum;

        // Derive and run cascade
        let cascade_input = derive_cascade_input(
            &physics,
            &physics_input,
            fatigue_cycles,
            automation_coverage,
            perturbation_torque,
        );
        let cascade = loops::cascade(&cascade_input, &config.thresholds);

        // Track milestones
        if gyroscopic_activation_step.is_none()
            && matches!(
                cascade.gyroscopic.state,
                crate::loops::gyroscopic::GyroscopicState::Stable
                    | crate::loops::gyroscopic::GyroscopicState::Precessing
            )
        {
            gyroscopic_activation_step = Some(step);
        }
        if failure_step.is_none() && cascade.system_state == SystemState::Failed {
            failure_step = Some(step);
        }

        states.push(FlywheelState {
            step,
            time,
            physics,
            cascade,
            physics_input: physics_input.clone(),
            cascade_input,
        });
    }

    let final_state = states
        .last()
        .map(|s| s.cascade.system_state)
        .unwrap_or(SystemState::Failed);

    Trajectory {
        config: config.clone(),
        states,
        final_state,
        gyroscopic_activation_step,
        failure_step,
    }
}

/// Summary statistics from a trajectory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectorySummary {
    pub steps: u64,
    pub final_state: SystemState,
    pub peak_energy: f64,
    pub peak_momentum: f64,
    pub peak_stress_ratio: f64,
    pub gyroscopic_activation_step: Option<u64>,
    pub failure_step: Option<u64>,
    pub time_in_thriving: u64,
    pub time_in_stressed: u64,
    pub time_in_critical: u64,
    pub time_in_failed: u64,
}

/// Extract summary statistics from a trajectory.
#[must_use]
pub fn summarize(trajectory: &Trajectory) -> TrajectorySummary {
    let mut peak_energy = 0.0_f64;
    let mut peak_momentum = 0.0_f64;
    let mut peak_stress_ratio = 0.0_f64;
    let mut time_in_thriving = 0_u64;
    let mut time_in_stressed = 0_u64;
    let mut time_in_critical = 0_u64;
    let mut time_in_failed = 0_u64;

    for state in &trajectory.states {
        peak_energy = peak_energy.max(state.physics.energy);
        peak_momentum = peak_momentum.max(state.physics.momentum);
        peak_stress_ratio = peak_stress_ratio.max(state.physics.stress_ratio);
        match state.cascade.system_state {
            SystemState::Thriving => time_in_thriving += 1,
            SystemState::Stressed => time_in_stressed += 1,
            SystemState::Critical => time_in_critical += 1,
            SystemState::Failed => time_in_failed += 1,
        }
    }

    TrajectorySummary {
        steps: trajectory.states.len() as u64,
        final_state: trajectory.final_state,
        peak_energy,
        peak_momentum,
        peak_stress_ratio,
        gyroscopic_activation_step: trajectory.gyroscopic_activation_step,
        failure_step: trajectory.failure_step,
        time_in_thriving,
        time_in_stressed,
        time_in_critical,
        time_in_failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steady_state_stays_stable() {
        let config = SimulationConfig {
            dt: 1.0,
            steps: 50,
            scenario: Scenario::SteadyState,
            ..Default::default()
        };
        let trajectory = run(&config);
        let summary = summarize(&trajectory);
        assert_eq!(summary.final_state, SystemState::Thriving);
        assert_eq!(summary.time_in_failed, 0);
    }

    #[test]
    fn cold_start_builds_momentum() {
        let config = SimulationConfig {
            dt: 1.0,
            steps: 100,
            scenario: Scenario::ColdStart,
            ..Default::default()
        };
        let trajectory = run(&config);
        let _summary = summarize(&trajectory);
        // Energy should increase over time
        let first_energy = trajectory
            .states
            .first()
            .map(|s| s.physics.energy)
            .unwrap_or(0.0);
        let last_energy = trajectory
            .states
            .last()
            .map(|s| s.physics.energy)
            .unwrap_or(0.0);
        assert!(
            last_energy > first_energy,
            "Energy should grow: first={first_energy}, last={last_energy}"
        );
    }

    #[test]
    fn stress_test_eventually_degrades() {
        let config = SimulationConfig {
            dt: 1.0,
            steps: 200,
            scenario: Scenario::StressTest {
                perturbation_rate: 5.0,
            },
            ..Default::default()
        };
        let trajectory = run(&config);
        let summary = summarize(&trajectory);
        // With increasing perturbation, gyroscopic should eventually fail
        assert!(summary.time_in_stressed + summary.time_in_critical + summary.time_in_failed > 0);
    }

    #[test]
    fn fatigue_causes_failure() {
        let config = SimulationConfig {
            dt: 1.0,
            steps: 120,
            scenario: Scenario::Fatigue {
                cycles_per_step: 10,
            },
            ..Default::default()
        };
        let trajectory = run(&config);
        let summary = summarize(&trajectory);
        // 120 steps × 10 cycles = 1200 > fatigue_limit of 1000
        assert!(summary.failure_step.is_some(), "Should fail from fatigue");
        assert_eq!(summary.final_state, SystemState::Failed);
    }

    #[test]
    fn growth_increases_energy() {
        let config = SimulationConfig {
            dt: 1.0,
            steps: 50,
            scenario: Scenario::Growth {
                mass_rate: 2.0,
                omega_rate: 0.1,
            },
            ..Default::default()
        };
        let trajectory = run(&config);
        let summary = summarize(&trajectory);
        assert!(summary.peak_energy > 0.0);
        let first = trajectory
            .states
            .first()
            .map(|s| s.physics.energy)
            .unwrap_or(0.0);
        assert!(summary.peak_energy > first);
    }

    #[test]
    fn friction_spike_degrades_momentum() {
        let config = SimulationConfig {
            dt: 1.0,
            steps: 50,
            scenario: Scenario::FrictionSpike {
                spike_step: 10,
                post_spike_coverage: 0.0,
            },
            ..Default::default()
        };
        let trajectory = run(&config);
        let summary = summarize(&trajectory);
        // After spike, friction should increase significantly
        assert!(summary.time_in_stressed + summary.time_in_critical > 0);
    }

    #[test]
    fn gyroscopic_activates_with_momentum() {
        let config = SimulationConfig {
            dt: 1.0,
            steps: 50,
            scenario: Scenario::SteadyState,
            ..Default::default()
        };
        let trajectory = run(&config);
        // Default steady state has I=100, ω=1 → L=100 > critical=50
        assert!(trajectory.gyroscopic_activation_step.is_some());
    }

    #[test]
    fn trajectory_serializable() {
        let config = SimulationConfig {
            dt: 1.0,
            steps: 5,
            scenario: Scenario::SteadyState,
            ..Default::default()
        };
        let trajectory = run(&config);
        let json = serde_json::to_string(&trajectory);
        assert!(json.is_ok());
    }

    #[test]
    fn summary_counts_match_steps() {
        let config = SimulationConfig {
            dt: 1.0,
            steps: 30,
            scenario: Scenario::SteadyState,
            ..Default::default()
        };
        let trajectory = run(&config);
        let summary = summarize(&trajectory);
        let total = summary.time_in_thriving
            + summary.time_in_stressed
            + summary.time_in_critical
            + summary.time_in_failed;
        assert_eq!(total, summary.steps);
    }

    #[test]
    fn empty_run() {
        let config = SimulationConfig {
            dt: 1.0,
            steps: 0,
            scenario: Scenario::SteadyState,
            ..Default::default()
        };
        let trajectory = run(&config);
        assert!(trajectory.states.is_empty());
        assert_eq!(trajectory.final_state, SystemState::Failed);
    }
}
