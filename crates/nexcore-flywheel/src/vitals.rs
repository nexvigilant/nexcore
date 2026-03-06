//! Flywheel health vitals — the 15-field system snapshot.
//!
//! ## T1 Primitive Grounding: ς (State) + Σ (Sum) + π (Persistence)

use serde::{Deserialize, Serialize};

/// The 15-field flywheel health snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlywheelVitals {
    // ── Loop 1: Rim Integrity ────────────────────────────────────────────
    pub value_density: f64,
    pub churn_rate: f64,
    pub switching_cost_index: f64,

    // ── Loop 2: Momentum Conservation ────────────────────────────────────
    pub knowledge_base_growth: f64,
    pub execution_velocity: f64,
    pub momentum: f64,

    // ── Loop 3: Friction Dissipation ─────────────────────────────────────
    pub automation_coverage: f64,
    pub manual_touchpoints: u32,
    pub overhead_ratio: f64,

    // ── Loop 4: Gyroscopic Stability ─────────────────────────────────────
    pub mission_alignment_score: f64,
    pub scope_creep_incidents: u32,
    pub pivot_resistance: f64,

    // ── Loop 5: Elastic Equilibrium ──────────────────────────────────────
    pub contributor_load: f64,
    pub fatigue_cycle_count: u32,
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
