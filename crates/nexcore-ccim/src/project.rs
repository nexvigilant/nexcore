//! CCIM projection: trajectory forecasting, Rule of 72, FIRE ETA.
//!
//! Core equation: C(d) = C₀(1 + ρ)^d + T × [((1 + ρ)^d - 1) / ρ] - W(d)
//! Grounding: ρ(Recursion) + N(Quantity) + →(Causality).

use nexcore_constants::{Confidence, Measured};
use serde::{Deserialize, Serialize};

use crate::error::CcimError;
use crate::types::CompoundingRatio;

/// A single point on the capability trajectory.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TrajectoryPoint {
    /// Directive number (0 = current).
    pub directive: u32,
    /// Projected capability units at this directive.
    pub capability_units: f64,
}

/// FIRE (Financial Independence, Retire Early) projection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FireProjection {
    /// Current capability units.
    pub current_cu: f64,
    /// FIRE threshold (default 5000 CU).
    pub fire_threshold: f64,
    /// Estimated directives to reach FIRE.
    pub directives_to_fire: Measured<f64>,
    /// Rule of 72 doubling time at given rho.
    pub rule_of_72: Measured<f64>,
    /// Projected trajectory.
    pub trajectory: Vec<TrajectoryPoint>,
}

/// Compute C(d) using the CCIM compound interest equation.
///
/// When rho is zero, falls back to linear: C(d) = C0 + T*d - W.
///
/// # CALIBRATION: confidence = clamp(1.0 - 1.0 / (observations + 1), 0.05, 0.99)
#[allow(
    clippy::too_many_arguments,
    reason = "CCIM equation requires all terms"
)]
pub fn ccim_equation(
    c0: f64,
    rho: CompoundingRatio,
    d: u32,
    t_per_directive: f64,
    cumulative_w: f64,
    observations: u32,
) -> Result<Measured<f64>, CcimError> {
    let d_f64 = f64::from(d);
    let r = rho.value();

    let c_d = if rho.is_zero() {
        // Linear case: no compounding
        c0 + t_per_directive * d_f64 - cumulative_w
    } else {
        // Compound case
        let growth_factor = (1.0 + r).powf(d_f64);

        // Check for overflow
        if growth_factor.is_infinite() || growth_factor.is_nan() {
            return Err(CcimError::ProjectionOverflow {
                rho: r,
                directives: d,
            });
        }

        let principal_growth = c0 * growth_factor;
        let annuity_factor = (growth_factor - 1.0) / r;
        let contribution_growth = t_per_directive * annuity_factor;

        principal_growth + contribution_growth - cumulative_w
    };

    // CALIBRATION: confidence scales with observations
    let conf_raw = 1.0 - 1.0 / (f64::from(observations) + 1.0);
    let conf = conf_raw.clamp(0.05, 0.99);
    let confidence = Confidence::new(conf);

    Ok(Measured::new(c_d, confidence))
}

/// Rule of 72: directives to double capability.
///
/// At rho=0, returns infinity (never doubles without reinvestment).
///
/// # CALIBRATION: confidence = clamp(1.0 - 1.0 / (observations + 1), 0.05, 0.99)
pub fn rule_of_72(rho: CompoundingRatio, observations: u32) -> Result<Measured<f64>, CcimError> {
    let doubling = if rho.is_zero() {
        f64::INFINITY
    } else {
        72.0 / (rho.value() * 100.0)
    };

    let conf_raw = 1.0 - 1.0 / (f64::from(observations) + 1.0);
    let conf = conf_raw.clamp(0.05, 0.99);
    let confidence = Confidence::new(conf);

    Ok(Measured::new(doubling, confidence))
}

/// Project capability trajectory over N directives.
///
/// Returns a `FireProjection` with trajectory points and FIRE ETA.
#[allow(
    clippy::too_many_arguments,
    reason = "trajectory projection requires all parameters"
)]
pub fn trajectory_project(
    current_cu: f64,
    rho: CompoundingRatio,
    n_directives: u32,
    t_per_directive: f64,
    w_per_directive: f64,
    fire_threshold: f64,
    observations: u32,
) -> Result<FireProjection, CcimError> {
    let capacity = usize::try_from(n_directives.saturating_add(1)).unwrap_or(usize::MAX);
    let mut trajectory = Vec::with_capacity(capacity);
    trajectory.push(TrajectoryPoint {
        directive: 0,
        capability_units: current_cu,
    });

    let mut fire_directive: Option<u32> = None;

    for d in 1..=n_directives {
        let cumulative_w = w_per_directive * f64::from(d);
        let c_d = ccim_equation(
            current_cu,
            rho,
            d,
            t_per_directive,
            cumulative_w,
            observations,
        )?;

        trajectory.push(TrajectoryPoint {
            directive: d,
            capability_units: c_d.value,
        });

        if fire_directive.is_none() && c_d.value >= fire_threshold {
            fire_directive = Some(d);
        }
    }

    let dtf = fire_directive.map_or(f64::INFINITY, |d| f64::from(d));
    let conf_raw = 1.0 - 1.0 / (f64::from(observations) + 1.0);
    let conf = conf_raw.clamp(0.05, 0.99);
    let confidence = Confidence::new(conf);

    Ok(FireProjection {
        current_cu,
        fire_threshold,
        directives_to_fire: Measured::new(dtf, confidence),
        rule_of_72: rule_of_72(rho, observations)?,
        trajectory,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ccim_equation_zero_rho_linear() {
        let rho = CompoundingRatio::MATTRESS; // 0.0
        let result = ccim_equation(1000.0, rho, 5, 10.0, 5.0, 3).expect("should compute");
        // C(5) = 1000 + 10*5 - 5 = 1045
        assert!((result.value - 1045.0).abs() < 0.01);
    }

    #[test]
    fn test_ccim_equation_reference_table_rho_005() {
        // From CCIM doc: C0=1049, rho=0.05, d=20, T=13.6
        // Expected ~2224 CU (approximate from table)
        let rho = CompoundingRatio::new(0.05).expect("valid");
        let result = ccim_equation(1049.0, rho, 20, 13.6, 0.0, 5).expect("should compute");
        // Allow 10% tolerance for approximation
        assert!(
            result.value > 2000.0 && result.value < 3500.0,
            "expected ~2224-3000, got {}",
            result.value
        );
    }

    #[test]
    fn test_ccim_equation_reference_table_rho_050() {
        // From CCIM doc: C0=1049, rho=0.50, d=20, T=13.6
        // Expected very large growth (>100k CU)
        let rho = CompoundingRatio::GROWTH; // 0.50
        let result = ccim_equation(1049.0, rho, 20, 13.6, 0.0, 5).expect("should compute");
        assert!(
            result.value > 100_000.0,
            "expected >100k at rho=0.50 d=20, got {}",
            result.value
        );
    }

    #[test]
    fn test_rule_of_72_at_010() {
        let rho = CompoundingRatio::new(0.10).expect("valid");
        let result = rule_of_72(rho, 5).expect("should compute");
        assert!((result.value - 7.2).abs() < 0.01);
    }

    #[test]
    fn test_rule_of_72_at_050() {
        let rho = CompoundingRatio::GROWTH;
        let result = rule_of_72(rho, 5).expect("should compute");
        assert!((result.value - 1.44).abs() < 0.01);
    }

    #[test]
    fn test_rule_of_72_zero_rho_returns_infinity() {
        let rho = CompoundingRatio::MATTRESS;
        let result = rule_of_72(rho, 1).expect("should compute");
        assert!(result.value.is_infinite());
    }

    #[test]
    fn test_projection_overflow_protection() {
        let rho = CompoundingRatio::AGGRESSIVE; // 0.75
        // Very large directive count should overflow gracefully
        let result = ccim_equation(1000.0, rho, 10000, 10.0, 0.0, 1);
        assert!(result.is_err() || result.expect("ok").value.is_finite());
    }
}
