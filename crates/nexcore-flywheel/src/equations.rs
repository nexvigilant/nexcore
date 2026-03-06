// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Governing equations — the physics foundation.
//!
//! Every equation maps to a measurable strategic quantity.
//! All functions are pure: input → output, no side effects.
//!
//! ## T1 Primitive Grounding: N (Quantity) + → (Causality) + ∂ (Boundary)
//!
//! ## Appendix A Reference
//!
//! | Equation       | Mechanical              | Strategic                    |
//! |----------------|-------------------------|------------------------------|
//! | E = ½Iω²      | Stored kinetic energy   | Platform value               |
//! | I = mr²        | Moment of inertia (rim) | Ecosystem weight × reach²    |
//! | σ = ρω²r²     | Centrifugal stress      | Scaling stress               |
//! | L = Iω         | Angular momentum        | Organizational momentum      |
//! | F_drag ∝ ω³   | Aerodynamic drag        | Overhead at velocity         |
//! | τ = dL/dt      | Torque                  | Rate of momentum change      |

use serde::{Deserialize, Serialize};

/// All governing equation results in one snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlywheelPhysics {
    /// Stored kinetic energy: E = ½Iω²
    pub energy: f64,
    /// Moment of inertia: I = mr²
    pub inertia: f64,
    /// Angular momentum: L = Iω
    pub momentum: f64,
    /// Centrifugal stress: σ = ρω²r²
    pub stress: f64,
    /// Aerodynamic drag: F_drag = C_d × ω³
    pub drag: f64,
    /// Torque: τ = dL/dt (requires two snapshots)
    pub torque: f64,
    /// Stress ratio: σ / σ_yield (>1.0 = failure)
    pub stress_ratio: f64,
}

/// Input parameters for the governing equations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsInput {
    /// Total ecosystem mass (users, content, data weight).
    pub mass: f64,
    /// Radius — distance of value from core (reach).
    pub radius: f64,
    /// Angular velocity — execution throughput.
    pub omega: f64,
    /// Material density (for stress calculation).
    pub density: f64,
    /// Drag coefficient (aerodynamic overhead scaling).
    pub drag_coefficient: f64,
    /// Yield stress — structural failure threshold.
    pub yield_stress: f64,
}

impl Default for PhysicsInput {
    fn default() -> Self {
        Self {
            mass: 100.0,
            radius: 1.0,
            omega: 1.0,
            density: 1.0,
            drag_coefficient: 0.001,
            yield_stress: 1000.0,
        }
    }
}

/// E = ½Iω² — Stored kinetic energy (platform value).
///
/// Scales quadratically with velocity. Doubling execution speed
/// quadruples stored value.
#[must_use]
pub fn kinetic_energy(inertia: f64, omega: f64) -> f64 {
    0.5 * inertia * omega * omega
}

/// I = mr² — Moment of inertia (ecosystem weight × reach²).
///
/// Mass at the rim (far from core) stores more energy per unit ω.
/// Strategic: users far from your core (broad reach) contribute
/// disproportionately to stored value.
#[must_use]
pub fn moment_of_inertia(mass: f64, radius: f64) -> f64 {
    mass * radius * radius
}

/// L = Iω — Angular momentum (organizational momentum).
///
/// Conservation law: L stays constant unless external torque applied.
/// Strategic: momentum persists without effort once established.
#[must_use]
pub fn angular_momentum(inertia: f64, omega: f64) -> f64 {
    inertia * omega
}

/// σ = ρω²r² — Centrifugal stress (scaling stress).
///
/// The constraint on E = ½Iω²: you cannot spin arbitrarily fast
/// without structural failure. σ must stay below σ_yield.
#[must_use]
pub fn centrifugal_stress(density: f64, omega: f64, radius: f64) -> f64 {
    density * omega * omega * radius * radius
}

/// F_drag = C_d × ω³ — Aerodynamic drag (overhead at velocity).
///
/// Cubic scaling: 2× velocity → 8× overhead. This is why fast-growing
/// organizations hit a "sound barrier" — overhead grows faster than throughput.
#[must_use]
pub fn aerodynamic_drag(drag_coefficient: f64, omega: f64) -> f64 {
    drag_coefficient * omega * omega * omega
}

/// τ = ΔL/Δt — Torque (rate of momentum change).
///
/// Approximated from two momentum values over a time interval.
/// Positive τ = accelerating, negative τ = decelerating.
#[must_use]
pub fn torque(l_current: f64, l_previous: f64, dt: f64) -> f64 {
    if dt.abs() < f64::EPSILON {
        return 0.0;
    }
    (l_current - l_previous) / dt
}

/// Compute all governing equations from a single input set.
///
/// The `torque` field requires a previous momentum value; pass 0.0 and dt=0.0
/// for a single-snapshot evaluation (torque will be 0.0).
#[must_use]
pub fn evaluate(input: &PhysicsInput, l_previous: f64, dt: f64) -> FlywheelPhysics {
    let inertia = moment_of_inertia(input.mass, input.radius);
    let energy = kinetic_energy(inertia, input.omega);
    let momentum = angular_momentum(inertia, input.omega);
    let stress = centrifugal_stress(input.density, input.omega, input.radius);
    let drag = aerodynamic_drag(input.drag_coefficient, input.omega);
    let tau = torque(momentum, l_previous, dt);
    let stress_ratio = if input.yield_stress.abs() < f64::EPSILON {
        if stress.abs() < f64::EPSILON {
            0.0
        } else {
            f64::MAX
        }
    } else {
        stress / input.yield_stress
    };

    FlywheelPhysics {
        energy,
        inertia,
        momentum,
        stress,
        drag,
        torque: tau,
        stress_ratio,
    }
}

/// The core design tension: MAXIMIZE E SUBJECT TO σ < σ_yield.
///
/// Returns the maximum safe ω for a given configuration.
/// Derived from σ = ρω²r² < σ_yield → ω < sqrt(σ_yield / (ρr²)).
#[must_use]
pub fn max_safe_omega(density: f64, radius: f64, yield_stress: f64) -> f64 {
    if density.abs() < f64::EPSILON || radius.abs() < f64::EPSILON {
        return f64::MAX;
    }
    let denom = density * radius * radius;
    if denom.abs() < f64::EPSILON {
        return f64::MAX;
    }
    (yield_stress / denom).sqrt()
}

/// Maximum storable energy at safe operating limits.
/// E_max = ½ × m × r² × ω_max²
#[must_use]
pub fn max_safe_energy(mass: f64, radius: f64, density: f64, yield_stress: f64) -> f64 {
    let omega_max = max_safe_omega(density, radius, yield_stress);
    if omega_max == f64::MAX {
        return f64::MAX;
    }
    let i = moment_of_inertia(mass, radius);
    kinetic_energy(i, omega_max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn energy_scales_quadratically() {
        let e1 = kinetic_energy(100.0, 1.0);
        let e2 = kinetic_energy(100.0, 2.0);
        assert!((e2 / e1 - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn inertia_scales_with_radius_squared() {
        let i1 = moment_of_inertia(10.0, 1.0);
        let i2 = moment_of_inertia(10.0, 3.0);
        assert!((i2 / i1 - 9.0).abs() < f64::EPSILON);
    }

    #[test]
    fn momentum_conservation() {
        let l = angular_momentum(100.0, 5.0);
        assert!((l - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stress_scales_with_omega_squared() {
        let s1 = centrifugal_stress(1.0, 1.0, 1.0);
        let s2 = centrifugal_stress(1.0, 3.0, 1.0);
        assert!((s2 / s1 - 9.0).abs() < f64::EPSILON);
    }

    #[test]
    fn drag_scales_cubically() {
        let d1 = aerodynamic_drag(1.0, 1.0);
        let d2 = aerodynamic_drag(1.0, 2.0);
        assert!((d2 / d1 - 8.0).abs() < f64::EPSILON);
    }

    #[test]
    fn torque_from_momentum_change() {
        let tau = torque(600.0, 500.0, 1.0);
        assert!((tau - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn torque_zero_dt() {
        assert!((torque(600.0, 500.0, 0.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn evaluate_default() {
        let physics = evaluate(&PhysicsInput::default(), 0.0, 0.0);
        assert!((physics.inertia - 100.0).abs() < f64::EPSILON);
        assert!((physics.energy - 50.0).abs() < f64::EPSILON);
        assert!((physics.momentum - 100.0).abs() < f64::EPSILON);
        assert!(physics.stress_ratio < 1.0);
    }

    #[test]
    fn max_safe_omega_finite() {
        let omega = max_safe_omega(1.0, 1.0, 1000.0);
        // ω < sqrt(1000 / (1 × 1)) = sqrt(1000) ≈ 31.62
        assert!((omega - 1000.0_f64.sqrt()).abs() < 0.01);
    }

    #[test]
    fn max_safe_omega_zero_density() {
        assert_eq!(max_safe_omega(0.0, 1.0, 1000.0), f64::MAX);
    }

    #[test]
    fn stress_ratio_above_one_is_failure() {
        let input = PhysicsInput {
            mass: 100.0,
            radius: 10.0,
            omega: 100.0,
            density: 1.0,
            drag_coefficient: 0.001,
            yield_stress: 100.0,
        };
        let physics = evaluate(&input, 0.0, 0.0);
        assert!(
            physics.stress_ratio > 1.0,
            "Should exceed yield: {}",
            physics.stress_ratio
        );
    }

    #[test]
    fn max_safe_energy_computable() {
        let e = max_safe_energy(100.0, 1.0, 1.0, 1000.0);
        assert!(e.is_finite());
        assert!(e > 0.0);
    }

    #[test]
    fn negative_torque_means_decelerating() {
        let tau = torque(400.0, 500.0, 1.0);
        assert!(tau < 0.0);
    }

    #[test]
    fn evaluate_with_torque() {
        let input = PhysicsInput::default();
        let physics = evaluate(&input, 80.0, 1.0);
        // L = 100, L_prev = 80, dt = 1 → τ = 20
        assert!((physics.torque - 20.0).abs() < f64::EPSILON);
    }
}
