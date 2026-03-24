//! Flywheel health vitals — the 15-field system snapshot.
//!
//! ## T1 Primitive Grounding: ς (State) + Σ (Sum) + π (Persistence)

use serde::{Deserialize, Serialize};

/// The 15-field flywheel health snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlywheelVitals {
    // ── Loop 1: Rim Integrity ────────────────────────────────────────────
    /// Value delivered per unit effort.
    pub value_density: f64,
    /// Rate of customer/user loss.
    pub churn_rate: f64,
    /// How expensive it is to switch away.
    pub switching_cost_index: f64,

    // ── Loop 2: Momentum Conservation ────────────────────────────────────
    /// Knowledge asset growth rate.
    pub knowledge_base_growth: f64,
    /// Speed of task completion.
    pub execution_velocity: f64,
    /// Angular momentum (L = I × ω).
    pub momentum: f64,

    // ── Loop 3: Friction Dissipation ─────────────────────────────────────
    /// Fraction of processes automated (0.0–1.0).
    pub automation_coverage: f64,
    /// Count of manual human touchpoints.
    pub manual_touchpoints: u32,
    /// Overhead as fraction of productive work.
    pub overhead_ratio: f64,

    // ── Loop 4: Gyroscopic Stability ─────────────────────────────────────
    /// Alignment with stated mission (0.0–1.0).
    pub mission_alignment_score: f64,
    /// Count of scope creep events.
    pub scope_creep_incidents: u32,
    /// Resistance to perturbation (higher = more stable).
    pub pivot_resistance: f64,

    // ── Loop 5: Elastic Equilibrium ──────────────────────────────────────
    /// Load on primary contributor (0.0–1.0).
    pub contributor_load: f64,
    /// Cumulative fatigue cycles.
    pub fatigue_cycle_count: u32,
    /// Days needed to recover from overload.
    pub recovery_time_days: f64,
}

impl Default for FlywheelVitals {
    fn default() -> Self {
        Self {
            value_density: 0.0,
            churn_rate: 0.0,
            switching_cost_index: 0.0,
            knowledge_base_growth: 0.0,
            execution_velocity: 0.0,
            momentum: 0.0,
            automation_coverage: 0.0,
            manual_touchpoints: 0,
            overhead_ratio: 0.0,
            mission_alignment_score: 0.0,
            scope_creep_incidents: 0,
            pivot_resistance: 0.0,
            contributor_load: 0.0,
            fatigue_cycle_count: 0,
            recovery_time_days: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_vitals_all_zero() {
        let v = FlywheelVitals::default();
        assert!((v.value_density).abs() < f64::EPSILON);
        assert!((v.momentum).abs() < f64::EPSILON);
        assert_eq!(v.manual_touchpoints, 0);
        assert_eq!(v.fatigue_cycle_count, 0);
    }
    #[test]
    fn vitals_has_15_fields() {
        let json = serde_json::to_value(FlywheelVitals::default()).expect("ser");
        let obj = json.as_object().expect("object");
        assert_eq!(obj.len(), 15);
    }
    #[test]
    fn vitals_serialization_roundtrip() {
        let mut v = FlywheelVitals::default();
        v.value_density = 0.85;
        v.momentum = 120.0;
        v.fatigue_cycle_count = 42;
        let json = serde_json::to_string(&v).expect("ser");
        let back: FlywheelVitals = serde_json::from_str(&json).expect("de");
        assert!((back.value_density - 0.85).abs() < f64::EPSILON);
        assert!((back.momentum - 120.0).abs() < f64::EPSILON);
        assert_eq!(back.fatigue_cycle_count, 42);
    }
    #[test]
    fn vitals_clone() {
        let mut v = FlywheelVitals::default();
        v.churn_rate = 0.05;
        let v2 = v.clone();
        assert!((v2.churn_rate - 0.05).abs() < f64::EPSILON);
    }
}
