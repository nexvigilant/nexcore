//! Flywheel diagnostic thresholds.
//!
//! Default constants for loop health classification. These are the minimum
//! values that define critical boundaries for each autonomous loop.
//!
//! ## T1 Primitive Grounding: ∂ (Boundary) + κ (Comparison)

use serde::{Deserialize, Serialize};

/// Diagnostic threshold constants for the five autonomous loops.
///
/// Each field defines a boundary condition that governs loop behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlywheelThresholds {
    // ── Loop 1: Rim Integrity ────────────────────────────────────────────
    /// Below this community size, Loop 1 cannot self-sustain.
    pub min_community_size: u32,

    /// Maximum churn rate before rim disintegrates.
    pub max_churn_rate: f64,

    /// Margin percentage where tensile ≈ centrifugal (critical zone).
    pub rim_critical_margin: f64,

    // ── Loop 2: Momentum Conservation ────────────────────────────────────
    /// Below this momentum, Loop 4 (gyroscopic) doesn't activate.
    pub min_momentum_for_stability: f64,

    // ── Loop 3: Friction Dissipation ─────────────────────────────────────
    /// Above this overhead ratio, Loop 3 dominates.
    pub max_overhead_ratio: f64,

    /// Drag coefficient for aerodynamic (cubic) drag term.
    pub drag_coefficient: f64,

    /// Net drain below this is "acceptable".
    pub friction_acceptable_threshold: f64,

    /// Net drain below this is "warning" (above acceptable).
    pub friction_warning_threshold: f64,

    // ── Loop 4: Gyroscopic Stability ─────────────────────────────────────
    /// Stability ratio above this = stable (below = precessing).
    pub gyroscopic_stable_ratio: f64,

    // ── Loop 5: Elastic Equilibrium ──────────────────────────────────────
    /// Above this fatigue cycle count, failure is imminent.
    pub max_fatigue_cycles: u32,

    /// Default elastic modulus for strain computation.
    pub default_elastic_modulus: f64,
}

impl Default for FlywheelThresholds {
    fn default() -> Self {
        Self {
            min_community_size: 100,
            max_churn_rate: 0.15,
            rim_critical_margin: 0.1,
            min_momentum_for_stability: 50.0,
            max_overhead_ratio: 0.4,
            drag_coefficient: 0.001,
            friction_acceptable_threshold: 10.0,
            friction_warning_threshold: 50.0,
            gyroscopic_stable_ratio: 2.0,
            max_fatigue_cycles: 1000,
            default_elastic_modulus: 1.0,
        }
    }
}
