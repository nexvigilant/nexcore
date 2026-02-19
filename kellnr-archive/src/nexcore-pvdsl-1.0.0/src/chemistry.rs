//! Inline chemistry-based capability calculations for PVDSL.
//!
//! These are the 5 chemistry equations mapped to capability metrics.
//! Inlined from `vigilance::capabilities` to avoid circular dependency.
//!
//! Each function returns a normalized f64 in [0.0, 1.0].

/// Arrhenius adoption potential.
///
/// Models learning barrier as activation energy.
/// * `barrier` - Learning barrier (1-10, higher = harder)
/// * `motivation` - Motivation factor (0.5-1.5)
/// * `resources` - Resource availability (0.5-1.5)
#[must_use]
pub fn arrhenius(barrier: f64, motivation: f64, resources: f64) -> f64 {
    let barrier = barrier.clamp(1.0, 10.0);
    let motivation = motivation.clamp(0.5, 1.5);
    let resources = resources.clamp(0.5, 1.5);
    // Arrhenius: k = A * exp(-Ea / RT)
    let rate = resources * (-barrier / (motivation * 8.314)).exp();
    rate.clamp(0.0, 1.0)
}

/// Michaelis-Menten capacity efficiency.
///
/// Models capacity saturation.
/// * `vmax` - Maximum capacity
/// * `demand` - Current demand/load
/// * `km` - Half-saturation constant
#[must_use]
pub fn michaelis_menten(vmax: f64, demand: f64, km: f64) -> f64 {
    if vmax <= 0.0 || km <= 0.0 {
        return 0.0;
    }
    let demand = demand.max(0.0);
    let efficiency = vmax * demand / (km + demand);
    (efficiency / vmax).clamp(0.0, 1.0)
}

/// Hill equation synergy coefficient.
///
/// Models cooperative binding / skill synergy.
/// * `n` - Hill coefficient (cooperativity)
/// * `skills` - Current skill count/level
/// * `threshold` - Half-maximum threshold
#[must_use]
pub fn hill(n: f64, skills: f64, threshold: f64) -> f64 {
    if threshold <= 0.0 || n <= 0.0 {
        return 0.0;
    }
    let skills = skills.max(0.0);
    let sn = skills.powf(n);
    let tn = threshold.powf(n);
    sn / (tn + sn)
}

/// Henderson-Hasselbalch stability score.
///
/// Models buffer capacity / stability.
/// * `stabilizing` - Stabilizing factors
/// * `destabilizing` - Destabilizing factors
#[must_use]
pub fn henderson_hasselbalch(stabilizing: f64, destabilizing: f64) -> f64 {
    if stabilizing <= 0.0 || destabilizing <= 0.0 {
        return 0.0;
    }
    // pH = pKa + log10([A-]/[HA])
    let raw = (stabilizing / destabilizing).log10();
    // Sigmoid normalization to [0, 1]
    1.0 / (1.0 + (-raw).exp())
}

/// Half-life decay freshness factor.
///
/// Models knowledge/skill decay over time.
/// * `elapsed` - Days since last update
/// * `half_life` - Half-life in days
#[must_use]
pub fn half_life(elapsed: f64, half_life: f64) -> f64 {
    if half_life <= 0.0 {
        return 0.0;
    }
    let elapsed = elapsed.max(0.0);
    (-std::f64::consts::LN_2 * elapsed / half_life).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrhenius_range() {
        let result = arrhenius(6.0, 1.2, 1.0);
        assert!(result >= 0.0 && result <= 1.0, "arrhenius={result}");
    }

    #[test]
    fn test_michaelis_menten_saturation() {
        // At Km, efficiency should be ~0.5
        let result = michaelis_menten(100.0, 50.0, 50.0);
        assert!((result - 0.5).abs() < 0.01, "mm={result}");
    }

    #[test]
    fn test_hill_cooperativity() {
        // At threshold with n=1, should be 0.5
        let result = hill(1.0, 5.0, 5.0);
        assert!((result - 0.5).abs() < 0.01, "hill={result}");
    }

    #[test]
    fn test_henderson_balanced() {
        // Equal stabilizing/destabilizing → 0.5
        let result = henderson_hasselbalch(1.0, 1.0);
        assert!((result - 0.5).abs() < 0.01, "hh={result}");
    }

    #[test]
    fn test_half_life_zero_elapsed() {
        // Zero elapsed → freshness = 1.0
        let result = half_life(0.0, 365.0);
        assert!((result - 1.0).abs() < 0.01, "hl={result}");
    }

    #[test]
    fn test_half_life_one_half_life() {
        // One half-life → freshness ≈ 0.5
        let result = half_life(365.0, 365.0);
        assert!((result - 0.5).abs() < 0.01, "hl={result}");
    }
}
