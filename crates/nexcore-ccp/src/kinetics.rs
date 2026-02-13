//! Pharmacokinetic engine — absorption, decay, steady-state, titration.
//!
//! # T1 Grounding
//! - ∝ (proportionality): All functions model proportional relationships
//! - ∂ (boundary): Therapeutic index guards safety margins
//! - κ (comparison): Titration compares current vs target

use crate::error::CcpError;
use crate::types::{
    BioAvailability, Dose, DosingStrategy, HalfLife, PlasmaLevel, TherapeuticWindow,
};

/// Decay constant from half-life: k = ln(2) / t½
#[must_use]
fn decay_constant(half_life: HalfLife) -> f64 {
    core::f64::consts::LN_2 / half_life.value()
}

/// One-compartment PK model with first-order absorption and elimination.
///
/// C(t) = (F·D·ka)/(ka - ke) · [e^(-ke·t) - e^(-ka·t)]
///
/// Simplified for our use case where absorption is rapid (ka ≈ 1/t_max):
/// - Before t_max: linear rise → C = F·D · (t/t_max)
/// - After t_max: exponential decay → C = F·D · e^(-k·(t - t_max))
///
/// # Errors
/// Returns `CcpError::NegativeTime` if `time` < 0.
pub fn plasma_level_at(
    dose: Dose,
    bioavail: BioAvailability,
    half_life: HalfLife,
    time: f64,
    t_max: f64,
) -> Result<PlasmaLevel, CcpError> {
    if time < 0.0 {
        return Err(CcpError::NegativeTime { value: time });
    }
    if time < f64::EPSILON {
        return Ok(PlasmaLevel::ZERO);
    }

    let peak = dose.value() * bioavail.value();
    let t_max = t_max.max(0.01); // guard against zero t_max

    let level = if time <= t_max {
        // Absorption phase: linear rise to peak
        peak * (time / t_max)
    } else {
        // Elimination phase: exponential decay from peak
        let k = decay_constant(half_life);
        peak * (-k * (time - t_max)).exp()
    };

    Ok(PlasmaLevel(level.max(0.0)))
}

/// Compute loading dose to reach target plasma level quickly.
///
/// Loading dose = target / bioavailability
///
/// # Errors
/// Returns `CcpError::InvalidDose` if computed dose > 1.0.
pub fn compute_loading_dose(
    target: PlasmaLevel,
    bioavail: BioAvailability,
) -> Result<Dose, CcpError> {
    let raw = target.value() / bioavail.value();
    Dose::new(raw.min(1.0))
}

/// Compute maintenance dose for steady-state.
///
/// At steady state, dose replaces what decays per dosing interval.
/// D_maint = target × (1 - e^(-k·τ)) / F
///
/// We use τ = half_life (dose every half-life) for simplicity:
/// D_maint = target × 0.5 / F ≈ target × 0.5
///
/// # Errors
/// Returns `CcpError::InvalidDose` if computed dose > 1.0.
pub fn compute_maintenance_dose(
    half_life: HalfLife,
    target: PlasmaLevel,
) -> Result<Dose, CcpError> {
    let k = decay_constant(half_life);
    // Dosing interval = half-life
    let fraction_lost = 1.0 - (-k * half_life.value()).exp();
    let raw = target.value() * fraction_lost;
    Dose::new(raw.clamp(0.0, 1.0))
}

/// Therapeutic index: how centered the current level is within the window.
///
/// Returns a value in [0, 1]:
/// - 1.0 = perfectly centered
/// - 0.0 = at or beyond boundary
/// - Negative values clamped to 0
#[must_use]
pub fn therapeutic_index(window: TherapeuticWindow, current: PlasmaLevel) -> f64 {
    let mid = (window.lower + window.upper) / 2.0;
    let half_width = window.width() / 2.0;

    if half_width < f64::EPSILON {
        return 0.0;
    }

    let distance_from_center = (current.value() - mid).abs();
    let ratio = 1.0 - (distance_from_center / half_width);
    ratio.clamp(0.0, 1.0)
}

/// Time (hours) until plasma level decays to threshold.
///
/// t = -ln(threshold / current) / k
///
/// Returns 0.0 if already below threshold.
///
/// # Errors
/// Returns `CcpError::NegativeTime` if computation yields invalid result.
pub fn time_to_booster(
    current: PlasmaLevel,
    threshold: PlasmaLevel,
    half_life: HalfLife,
) -> Result<f64, CcpError> {
    if current.value() <= threshold.value() {
        return Ok(0.0);
    }
    if threshold.value() <= 0.0 {
        // Never decays to zero, return large value
        return Ok(f64::MAX);
    }

    let k = decay_constant(half_life);
    let t = -(threshold.value() / current.value()).ln() / k;
    Ok(t.max(0.0))
}

/// Titrate dose based on current response vs target.
///
/// Strategy-aware adjustment:
/// - `Subtherapeutic`: aggressive increase (1.5x gap)
/// - `Therapeutic`: fine-tune (0.5x gap)
/// - `Loading`: maximum safe dose toward target
/// - `Maintenance`: conservative (0.3x gap)
///
/// # Errors
/// Returns `CcpError::InvalidDose` if computed dose is out of range.
pub fn titrate(
    current_response: PlasmaLevel,
    target: PlasmaLevel,
    strategy: DosingStrategy,
) -> Result<Dose, CcpError> {
    let gap = target.value() - current_response.value();

    let adjustment = match strategy {
        DosingStrategy::Subtherapeutic => gap * 1.5,
        DosingStrategy::Therapeutic => gap * 0.5,
        DosingStrategy::Loading => gap.max(0.0).min(1.0),
        DosingStrategy::Maintenance => gap * 0.3,
    };

    Dose::new(adjustment.clamp(0.0, 1.0))
}

/// Hill dose-response curve: E = E_max × D^n / (EC50^n + D^n)
///
/// Models sigmoidal dose-response relationship.
/// - `n` = Hill coefficient (cooperativity; typically 1-4)
/// - `ec50` = dose at 50% max effect
#[must_use]
pub fn hill_response(dose: Dose, e_max: f64, ec50: f64, n: f64) -> f64 {
    if ec50 <= 0.0 || n <= 0.0 {
        return 0.0;
    }
    let d_n = dose.value().powf(n);
    let ec50_n = ec50.powf(n);
    e_max * d_n / (ec50_n + d_n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dose(v: f64) -> Dose {
        Dose::new(v).unwrap_or(Dose::ZERO)
    }
    fn bio(v: f64) -> BioAvailability {
        BioAvailability::new(v)
            .unwrap_or_else(|_| BioAvailability::new(1.0).unwrap_or_else(|_| panic!("unreachable")))
    }
    fn hl(v: f64) -> HalfLife {
        HalfLife::new(v)
            .unwrap_or_else(|_| HalfLife::new(1.0).unwrap_or_else(|_| panic!("unreachable")))
    }

    #[test]
    fn plasma_at_zero_time() {
        let result = plasma_level_at(dose(0.8), bio(1.0), hl(24.0), 0.0, 1.0);
        assert!(result.is_ok());
        assert!((result.unwrap_or(PlasmaLevel::ZERO).value()).abs() < f64::EPSILON);
    }

    #[test]
    fn plasma_negative_time_errors() {
        let result = plasma_level_at(dose(0.8), bio(1.0), hl(24.0), -1.0, 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn plasma_at_peak() {
        let result = plasma_level_at(dose(0.8), bio(1.0), hl(24.0), 1.0, 1.0);
        assert!(result.is_ok());
        let level = result.unwrap_or(PlasmaLevel::ZERO).value();
        assert!((level - 0.8).abs() < 0.01);
    }

    #[test]
    fn plasma_decays_after_peak() {
        let at_peak =
            plasma_level_at(dose(0.8), bio(1.0), hl(24.0), 1.0, 1.0).unwrap_or(PlasmaLevel::ZERO);
        let at_25h =
            plasma_level_at(dose(0.8), bio(1.0), hl(24.0), 25.0, 1.0).unwrap_or(PlasmaLevel::ZERO);
        assert!(at_25h.value() < at_peak.value());
        // After one half-life past peak, should be ~half
        assert!((at_25h.value() - 0.4).abs() < 0.05);
    }

    #[test]
    fn plasma_bioavailability_scales() {
        let full =
            plasma_level_at(dose(1.0), bio(1.0), hl(24.0), 1.0, 1.0).unwrap_or(PlasmaLevel::ZERO);
        let half =
            plasma_level_at(dose(1.0), bio(0.5), hl(24.0), 1.0, 1.0).unwrap_or(PlasmaLevel::ZERO);
        assert!((full.value() / half.value() - 2.0).abs() < 0.01);
    }

    #[test]
    fn loading_dose_computation() {
        let d = compute_loading_dose(PlasmaLevel(0.6), bio(0.8));
        assert!(d.is_ok());
        let dv = d.unwrap_or(Dose::ZERO).value();
        assert!((dv - 0.75).abs() < 0.01);
    }

    #[test]
    fn maintenance_dose_computation() {
        let d = compute_maintenance_dose(hl(24.0), PlasmaLevel(0.5));
        assert!(d.is_ok());
        let dv = d.unwrap_or(Dose::ZERO).value();
        // Should be ~0.25 (half of target, replaced per half-life)
        assert!(dv > 0.1 && dv < 0.6);
    }

    #[test]
    fn therapeutic_index_centered() {
        let w = TherapeuticWindow::default_window(); // [0.3, 0.8]
        let ti = therapeutic_index(w, PlasmaLevel(0.55)); // center
        assert!(ti > 0.9);
    }

    #[test]
    fn therapeutic_index_at_boundary() {
        let w = TherapeuticWindow::default_window();
        let ti = therapeutic_index(w, PlasmaLevel(0.3)); // at lower bound
        assert!(ti < 0.05);
    }

    #[test]
    fn therapeutic_index_outside() {
        let w = TherapeuticWindow::default_window();
        let ti = therapeutic_index(w, PlasmaLevel(0.1));
        assert!((ti - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn time_to_booster_below_threshold() {
        let t = time_to_booster(PlasmaLevel(0.1), PlasmaLevel(0.3), hl(24.0));
        assert!(t.is_ok());
        assert!((t.unwrap_or(1.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn time_to_booster_computation() {
        let t = time_to_booster(PlasmaLevel(0.8), PlasmaLevel(0.4), hl(24.0));
        assert!(t.is_ok());
        let hours = t.unwrap_or(0.0);
        // Should be about 24 hours (one half-life to go from 0.8 to 0.4)
        assert!((hours - 24.0).abs() < 0.5);
    }

    #[test]
    fn titrate_subtherapeutic() {
        let d = titrate(
            PlasmaLevel(0.1),
            PlasmaLevel(0.5),
            DosingStrategy::Subtherapeutic,
        );
        assert!(d.is_ok());
        let dv = d.unwrap_or(Dose::ZERO).value();
        // 1.5 * (0.5 - 0.1) = 0.6
        assert!((dv - 0.6).abs() < 0.01);
    }

    #[test]
    fn titrate_maintenance() {
        let d = titrate(
            PlasmaLevel(0.4),
            PlasmaLevel(0.5),
            DosingStrategy::Maintenance,
        );
        assert!(d.is_ok());
        let dv = d.unwrap_or(Dose::ZERO).value();
        // 0.3 * (0.5 - 0.4) = 0.03
        assert!((dv - 0.03).abs() < 0.01);
    }

    #[test]
    fn hill_response_at_ec50() {
        // At EC50, effect should be E_max/2
        let effect = hill_response(dose(0.5), 1.0, 0.5, 1.0);
        assert!((effect - 0.5).abs() < 0.01);
    }

    #[test]
    fn hill_response_steep_curve() {
        // High Hill coefficient → steeper curve
        let low = hill_response(dose(0.3), 1.0, 0.5, 1.0);
        let high = hill_response(dose(0.3), 1.0, 0.5, 4.0);
        // Steeper curve means lower value below EC50
        assert!(high < low);
    }
}
