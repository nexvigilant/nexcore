//! # Energy Bridge
//!
//! Inter-crate pipeline: Energy → Cardiovascular.
//!
//! Converts energy `Regime` and energy charge into cardiovascular demand
//! signals that drive heart rate regulation and pump preload.
//!
//! ```text
//! Energy::Regime → demand (f64) → Heart::regulate_rate → Heart::pump → PumpResult
//! ```

use nexcore_energy::Regime;

use crate::{Heart, PumpResult};

/// Map an energy regime to a cardiac demand factor (0.0–1.0).
///
/// **Biological mapping**: Sympathetic/parasympathetic tone — metabolic state
/// drives autonomic nervous system which modulates heart rate.
/// Anabolic (growth) = high demand, Crisis = minimal.
pub fn regime_to_demand(regime: &Regime) -> f64 {
    match regime {
        Regime::Anabolic => 1.0,
        Regime::Homeostatic => 0.6,
        Regime::Catabolic => 0.3,
        Regime::Crisis => 0.1,
    }
}

/// Map energy charge (0.0–1.0) to pump preload factor.
///
/// **Biological mapping**: Venous return — higher energy availability
/// means more substrate returning to the heart for pumping.
pub fn energy_charge_to_preload(energy_charge: f64) -> f64 {
    // EC is already 0.0–1.0; scale to a physiological preload range.
    // 100.0 baseline gives Heart::frank_starling meaningful input.
    energy_charge.clamp(0.0, 1.0) * 100.0
}

/// Execute a pump cycle driven by the current energy regime.
///
/// Adjusts heart rate to match metabolic demand, then pumps with
/// a preload proportional to the demand level.
///
/// **Biological mapping**: Exercise response — metabolic demand increases
/// heart rate AND stroke volume simultaneously.
pub fn energy_driven_pump(heart: &mut Heart, regime: &Regime) -> PumpResult {
    let demand = regime_to_demand(regime);
    heart.regulate_rate(demand);
    heart.pump(demand * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regime_to_demand_ordering() {
        let anabolic = regime_to_demand(&Regime::Anabolic);
        let homeostatic = regime_to_demand(&Regime::Homeostatic);
        let catabolic = regime_to_demand(&Regime::Catabolic);
        let crisis = regime_to_demand(&Regime::Crisis);

        assert!(anabolic > homeostatic);
        assert!(homeostatic > catabolic);
        assert!(catabolic > crisis);
        assert!(crisis > 0.0);
    }

    #[test]
    fn test_energy_charge_to_preload() {
        assert!((energy_charge_to_preload(1.0) - 100.0).abs() < f64::EPSILON);
        assert!((energy_charge_to_preload(0.5) - 50.0).abs() < f64::EPSILON);
        assert!((energy_charge_to_preload(0.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_energy_charge_clamping() {
        assert!((energy_charge_to_preload(1.5) - 100.0).abs() < f64::EPSILON);
        assert!((energy_charge_to_preload(-0.5) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_energy_driven_pump_anabolic() {
        let mut heart = Heart::new();
        let result = energy_driven_pump(&mut heart, &Regime::Anabolic);
        assert!(result.stroke_volume > 0.0, "Anabolic pump should produce output");
        assert!(result.beat_number > 0);
    }

    #[test]
    fn test_energy_driven_pump_crisis() {
        let mut heart = Heart::new();
        let result = energy_driven_pump(&mut heart, &Regime::Crisis);
        // Crisis demand is 0.1 → preload 10.0 → small stroke volume
        assert!(result.stroke_volume > 0.0, "Crisis should still pump something");
    }

    #[test]
    fn test_anabolic_produces_more_than_crisis() {
        let mut heart_a = Heart::new();
        let mut heart_c = Heart::new();

        let anabolic_result = energy_driven_pump(&mut heart_a, &Regime::Anabolic);
        let crisis_result = energy_driven_pump(&mut heart_c, &Regime::Crisis);

        assert!(
            anabolic_result.stroke_volume > crisis_result.stroke_volume,
            "Anabolic should pump more than crisis"
        );
    }
}
