// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Live data adapter — maps Guardian, Immunity, and session metrics
//! into [`CascadeInput`] for real-time VDAG evaluation.
//!
//! ## Mapping Table
//!
//! | Loop | Live Source | Mapping |
//! |------|-----------|---------|
//! | Rim (structural integrity) | Guardian sensors/actuators | tensile = sensors, centrifugal = unresolved signals |
//! | Momentum (inertia) | Session velocity | inertia = tool_calls, omega = commits/hour |
//! | Friction (parasitic drain) | Manual processes | manual = human touchpoints, automation = hook coverage |
//! | Gyroscopic (stability) | Guardian iteration history | momentum_l = iterations, torque = threat signals |
//! | Elastic (fatigue) | Session count / immunity | stress = session rate, fatigue = sessions since rest |
//!
//! ## T1 Primitive Grounding: μ(Mapping) + ς(State) + →(Causality)

use crate::loops::{
    CascadeInput, ElasticInput, FrictionInput, GyroscopicInput, MomentumInput, RimInput,
};
use serde::{Deserialize, Serialize};

/// Live system metrics collected from MCP tools.
///
/// Each field maps to a real observable: Guardian status, Immunity status,
/// or session-level telemetry. No synthetic data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveMetrics {
    // -- Guardian surface --
    /// Number of registered Guardian sensors (from `guardian_status`).
    pub sensor_count: u32,
    /// Number of registered Guardian actuators (from `guardian_status`).
    pub actuator_count: u32,
    /// Guardian iteration count (from `guardian_status`).
    pub guardian_iterations: u32,
    /// Signals detected in last homeostasis tick.
    pub signals_detected: u32,
    /// Actions taken in last homeostasis tick.
    pub actions_taken: u32,

    // -- Immunity surface --
    /// Total antibodies in the registry (from `immunity_status`).
    pub antibody_count: u32,
    /// Critical-severity antibodies.
    pub critical_antibodies: u32,

    // -- Session surface --
    /// Tool calls in current session.
    pub tool_calls: u32,
    /// Commits in current session.
    pub commits: u32,
    /// Files modified in current session.
    pub files_modified: u32,
    /// Total sessions in brain.db.
    pub total_sessions: u32,
    /// Sessions in last 24 hours.
    pub sessions_last_24h: u32,

    // -- Automation surface --
    /// Active hooks count.
    pub active_hooks: u32,
    /// Total skills count.
    pub skill_count: u32,
    /// Automation coverage ratio (0.0-1.0): hooks / (hooks + manual processes).
    pub automation_coverage: f64,
}

impl Default for LiveMetrics {
    fn default() -> Self {
        Self {
            sensor_count: 0,
            actuator_count: 0,
            guardian_iterations: 0,
            signals_detected: 0,
            actions_taken: 0,
            antibody_count: 0,
            critical_antibodies: 0,
            tool_calls: 0,
            commits: 0,
            files_modified: 0,
            total_sessions: 0,
            sessions_last_24h: 0,
            active_hooks: 0,
            skill_count: 0,
            automation_coverage: 0.0,
        }
    }
}

impl LiveMetrics {
    /// Convert live metrics into a [`CascadeInput`] for VDAG evaluation.
    ///
    /// The mapping is deterministic and documented per-field.
    pub fn to_cascade_input(&self) -> CascadeInput {
        CascadeInput {
            rim: self.map_rim(),
            momentum: self.map_momentum(),
            friction: self.map_friction(),
            gyroscopic: self.map_gyroscopic(),
            elastic: self.map_elastic(),
        }
    }

    /// Rim Integrity: structural health of the value network.
    ///
    /// - tensile_strength = sensor_count × actuator_count (detection × response capacity)
    /// - centrifugal_force = signals_detected - actions_taken (unresolved load)
    fn map_rim(&self) -> RimInput {
        let tensile_strength = f64::from(self.sensor_count) * f64::from(self.actuator_count);
        // Unresolved signals create centrifugal force (outward pressure)
        let unresolved = self.signals_detected.saturating_sub(self.actions_taken);
        let centrifugal_force = f64::from(unresolved).max(1.0); // Floor at 1.0 to avoid division edge cases

        RimInput {
            tensile_strength,
            centrifugal_force,
        }
    }

    /// Momentum: session velocity and forward progress.
    ///
    /// - inertia = tool_calls (mass of work done)
    /// - omega = commits + files_modified (rate of tangible output)
    /// - friction_drain = 0 (friction loop handles this separately)
    fn map_momentum(&self) -> MomentumInput {
        MomentumInput {
            inertia: f64::from(self.tool_calls).max(1.0),
            omega: f64::from(self.commits + self.files_modified).max(0.1),
            friction_drain: 0.0, // Friction loop handles drain independently
        }
    }

    /// Friction: parasitic drag from manual processes.
    ///
    /// - manual_processes = (1.0 - automation_coverage) × 10 (inverse of automation)
    /// - human_touchpoints = sessions_last_24h (context switches)
    /// - velocity = tool_calls (throughput)
    /// - automation_coverage = direct from metrics
    fn map_friction(&self) -> FrictionInput {
        let manual = (1.0 - self.automation_coverage.clamp(0.0, 1.0)) * 10.0;

        FrictionInput {
            manual_processes: manual,
            human_touchpoints: f64::from(self.sessions_last_24h).max(1.0),
            velocity: f64::from(self.tool_calls).max(1.0),
            automation_coverage: self.automation_coverage.clamp(0.0, 1.0),
        }
    }

    /// Gyroscopic Stability: mission alignment under perturbation.
    ///
    /// - momentum_l = guardian_iterations × antibody_count (accumulated stability mass)
    /// - perturbation_torque = critical_antibodies × 10 + signals_detected (threat pressure)
    /// - critical_momentum scales with system maturity: sqrt(total_sessions) clamped to [10, 200]
    ///
    /// CALIBRATION: critical_momentum grows as the system matures — a young system
    /// needs less stability mass to resist perturbation than a large one.
    fn map_gyroscopic(&self) -> GyroscopicInput {
        let stability_mass =
            f64::from(self.guardian_iterations.max(1)) * f64::from(self.antibody_count.max(1));
        let threat_pressure =
            f64::from(self.critical_antibodies) * 10.0 + f64::from(self.signals_detected);

        // CALIBRATION: critical_momentum = sqrt(sessions), clamped [10, 200]
        let critical_momentum = (f64::from(self.total_sessions).sqrt()).clamp(10.0, 200.0);

        GyroscopicInput {
            momentum_l: stability_mass,
            perturbation_torque: threat_pressure.max(1.0),
            critical_momentum,
        }
    }

    /// Elastic Equilibrium: fatigue and adaptive capacity.
    ///
    /// - stress = sessions_last_24h × 10 (session load pressure)
    /// - yield_point scales with automation: 80 + (automation_coverage × 120)
    ///   Range: [80, 200]. More automation = higher yield tolerance.
    /// - fatigue_cycles = total_sessions (cumulative wear)
    /// - fatigue_limit scales with hooks+skills: base 500 + (hooks + skills)
    ///   A richer ecosystem can sustain more sessions before fatigue.
    ///
    /// CALIBRATION: yield_point and fatigue_limit grow with system capability,
    /// not fixed constants. A system with 100 hooks tolerates more than one with 10.
    fn map_elastic(&self) -> ElasticInput {
        // CALIBRATION: yield_point = 80 + automation × 120, range [80, 200]
        let yield_point = 80.0 + self.automation_coverage.clamp(0.0, 1.0) * 120.0;

        // CALIBRATION: fatigue_limit = 500 + hooks + skills, minimum 500
        let fatigue_limit = 500 + self.active_hooks + self.skill_count;

        ElasticInput {
            stress: f64::from(self.sessions_last_24h) * 10.0,
            yield_point,
            fatigue_cycles: self.total_sessions,
            fatigue_limit,
        }
    }
}

// ============================================================================
// VDAG → Microgram Bridge
// ============================================================================

/// Result of the full live evaluation pipeline.
///
/// Contains both the VDAG graded result and the integer reality_score
/// expected by the `reality-gate` microgram.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveEvaluation {
    /// The full VDAG graded cascade result.
    pub graded: crate::vdag::GradedCascadeResult,
    /// Reality score scaled to 0-100 for microgram consumption.
    /// `reality_score = (reality.score * 100.0).round() as u32`
    pub reality_score: u32,
}

impl LiveMetrics {
    /// Run the full pipeline: LiveMetrics → CascadeInput → VDAG → LiveEvaluation.
    ///
    /// The `reality_score` field is scaled to 0-100 for direct use with
    /// the `reality-gate` microgram: `mcg run reality-gate.yaml -i '{"reality_score": N}'`
    pub fn evaluate(
        &self,
        thresholds: &crate::thresholds::FlywheelThresholds,
        goal: &crate::vdag::FlywheelGoal,
    ) -> LiveEvaluation {
        let input = self.to_cascade_input();
        let graded = crate::vdag::evaluate(&input, thresholds, goal);
        let reality_score = (graded.reality.score * 100.0).round() as u32;
        LiveEvaluation {
            graded,
            reality_score,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_metrics() -> LiveMetrics {
        LiveMetrics {
            sensor_count: 13,
            actuator_count: 3,
            guardian_iterations: 5,
            signals_detected: 11,
            actions_taken: 11,
            antibody_count: 14,
            critical_antibodies: 4,
            tool_calls: 25,
            commits: 2,
            files_modified: 5,
            total_sessions: 390,
            sessions_last_24h: 3,
            active_hooks: 105,
            skill_count: 256,
            automation_coverage: 0.85,
        }
    }

    #[test]
    fn cascade_input_from_live_metrics() {
        let metrics = sample_metrics();
        let input = metrics.to_cascade_input();

        // Rim: 13 sensors × 3 actuators = 39 tensile, 0 unresolved → floor 1.0
        assert!((input.rim.tensile_strength - 39.0).abs() < f64::EPSILON);
        assert!((input.rim.centrifugal_force - 1.0).abs() < f64::EPSILON);

        // Momentum: inertia = 25 tool calls, omega = 2+5 = 7
        assert!((input.momentum.inertia - 25.0).abs() < f64::EPSILON);
        assert!((input.momentum.omega - 7.0).abs() < f64::EPSILON);

        // Friction: manual = (1-0.85)*10 ≈ 1.5, touchpoints = 3
        assert!((input.friction.manual_processes - 1.5).abs() < 1e-10);
        assert!((input.friction.human_touchpoints - 3.0).abs() < f64::EPSILON);
        assert!((input.friction.automation_coverage - 0.85).abs() < f64::EPSILON);

        // Gyroscopic: momentum_l = 5*14 = 70, torque = 4*10+11 = 51
        assert!((input.gyroscopic.momentum_l - 70.0).abs() < f64::EPSILON);
        assert!((input.gyroscopic.perturbation_torque - 51.0).abs() < f64::EPSILON);
        // critical_momentum = sqrt(390) ≈ 19.75, clamped to [10, 200]
        assert!((input.gyroscopic.critical_momentum - 390_f64.sqrt()).abs() < 1e-10);

        // Elastic: stress = 3*10 = 30, fatigue_cycles = 390
        assert!((input.elastic.stress - 30.0).abs() < f64::EPSILON);
        assert_eq!(input.elastic.fatigue_cycles, 390);
        // yield_point = 80 + 0.85 * 120 = 182.0
        assert!((input.elastic.yield_point - 182.0).abs() < f64::EPSILON);
        // fatigue_limit = 500 + 105 + 256 = 861
        assert_eq!(input.elastic.fatigue_limit, 861);
    }

    #[test]
    fn default_metrics_produce_valid_input() {
        let input = LiveMetrics::default().to_cascade_input();
        // Should not panic and should produce floor values
        assert!(input.rim.centrifugal_force >= 1.0);
        assert!(input.momentum.inertia >= 1.0);
        assert!(input.momentum.omega >= 0.1);
    }

    #[test]
    fn high_load_metrics() {
        let metrics = LiveMetrics {
            sensor_count: 13,
            actuator_count: 3,
            guardian_iterations: 100,
            signals_detected: 50,
            actions_taken: 10, // 40 unresolved
            antibody_count: 20,
            critical_antibodies: 10,
            tool_calls: 200,
            commits: 15,
            files_modified: 40,
            total_sessions: 500,
            sessions_last_24h: 12, // Heavy usage
            active_hooks: 105,
            skill_count: 256,
            automation_coverage: 0.3, // Low automation
        };
        let input = metrics.to_cascade_input();

        // 40 unresolved signals → high centrifugal force
        assert!((input.rim.centrifugal_force - 40.0).abs() < f64::EPSILON);
        // Low automation → high manual friction
        assert!((input.friction.manual_processes - 7.0).abs() < f64::EPSILON);
        // High session load → high elastic stress
        assert!((input.elastic.stress - 120.0).abs() < f64::EPSILON);
        // yield_point = 80 + 0.3 * 120 = 116.0
        assert!((input.elastic.yield_point - 116.0).abs() < f64::EPSILON);
        // stress(120) > yield_point(116) → will classify as YieldExceeded
        assert!(input.elastic.stress > input.elastic.yield_point);
    }

    #[test]
    fn evaluate_produces_microgram_compatible_score() {
        let metrics = sample_metrics();
        let thresholds = crate::thresholds::FlywheelThresholds::default();
        let goal = crate::vdag::FlywheelGoal::default();
        let eval = metrics.evaluate(&thresholds, &goal);

        // reality_score is 0-100 integer
        assert!(eval.reality_score <= 100);
        // Should match the graded score scaled up
        let expected = (eval.graded.reality.score * 100.0).round() as u32;
        assert_eq!(eval.reality_score, expected);
        // Healthy metrics should produce an executable result
        assert!(eval.graded.reality.executable);
    }

    #[test]
    fn serialization_roundtrip() {
        let metrics = sample_metrics();
        let json = serde_json::to_string(&metrics);
        assert!(json.is_ok());
        let parsed: Result<LiveMetrics, _> = serde_json::from_str(&json.unwrap_or_default());
        assert!(parsed.is_ok());
    }
}
