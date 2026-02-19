//! # Multi-Sensor Fusion & Optimal Countermeasure Selection
//!
//! ## Fusion Model
//!
//! Independent fusion (any-sensor-detects = detected):
//! ```text
//! P_fused = 1 - ∏(1 - P_detect(sensor_i, target, range))
//! ```
//!
//! This is pessimistic for the target: the adversary needs to defeat ALL sensors.
//!
//! ## Optimal Countermeasure Selection
//!
//! Given:
//! - Threat sensor suite S = {s_1, ..., s_n}
//! - Available countermeasures C = {c_1, ..., c_m}
//! - Weight budget W (mass constraint in kg)
//!
//! Minimize: P_fused(S, target_with_countermeasures)
//! Subject to: Σ w_i × x_i ≤ W, x_i ∈ {0,1}
//!
//! Solved via branch-and-bound for small instance sizes.
//!
//! ## Lex Primitiva Grounding
//! - `FusionResult` → Σ (Sum) — combining sensor detections
//! - `OptimalLoadout` → κ (Comparison) × ρ (Recursion) — recursive search with comparison

use serde::{Deserialize, Serialize};

use crate::detection::{DetectionAssessment, compute_detection};
use crate::matrix::EffectivenessMatrix;
use crate::primitives::{CounterPrimitive, Countermeasure, SensorSystem};

/// Result of multi-sensor fused detection assessment.
///
/// Tier: T2-C (composed of Σ + N)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionResult {
    /// Per-sensor detection assessments
    pub sensor_assessments: Vec<DetectionAssessment>,
    /// Fused detection probability [0.0, 1.0]
    pub fused_probability: f64,
    /// Whether the target is considered detected (P > threshold)
    pub detected: bool,
    /// Detection threshold used
    pub threshold: f64,
}

/// Compute fused detection probability across multiple sensors.
///
/// ```text
/// P_fused = 1 - ∏(1 - P_i)
/// ```
pub fn compute_fusion(
    sensors: &[SensorSystem],
    counters: &[CounterPrimitive],
    matrix: &EffectivenessMatrix,
    range_m: f64,
    raw_signature: f64,
    detection_threshold: f64,
) -> FusionResult {
    let assessments: Vec<DetectionAssessment> = sensors
        .iter()
        .map(|s| compute_detection(s, counters, matrix, range_m, raw_signature))
        .collect();

    let pass_through: f64 = assessments
        .iter()
        .map(|a| 1.0 - a.detection_probability)
        .product();

    let fused = 1.0 - pass_through;

    FusionResult {
        sensor_assessments: assessments,
        fused_probability: fused,
        detected: fused >= detection_threshold,
        threshold: detection_threshold,
    }
}

/// A selected loadout of countermeasures with computed effectiveness.
///
/// Tier: T3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimalLoadout {
    /// Selected countermeasure indices
    pub selected: Vec<usize>,
    /// Total weight of selected countermeasures
    pub total_weight_kg: f64,
    /// Total power draw of selected countermeasures
    pub total_power_w: f64,
    /// Fused detection probability with this loadout
    pub fused_probability: f64,
    /// Active counter-primitives from selected countermeasures
    pub active_counters: Vec<CounterPrimitive>,
}

/// Find optimal countermeasure loadout via exhaustive search.
///
/// For N countermeasures, evaluates 2^N combinations.
/// Practical for N ≤ 20 (1M combinations).
///
/// ## Algorithm
/// ```text
/// for each subset S ⊆ countermeasures:
///   if weight(S) ≤ budget:
///     compute P_fused with counters(S)
///     if P_fused < best:
///       best = S
/// ```
pub fn optimize_loadout(
    sensors: &[SensorSystem],
    countermeasures: &[Countermeasure],
    matrix: &EffectivenessMatrix,
    weight_budget_kg: f64,
    range_m: f64,
    raw_signature: f64,
) -> OptimalLoadout {
    let n = countermeasures.len();
    // Cap at 20 countermeasures to avoid combinatorial explosion
    let max_n = n.min(20);

    let mut best_prob = f64::MAX;
    let mut best_mask: u32 = 0;

    for mask in 0..(1u32 << max_n) {
        // Check weight constraint
        let total_weight: f64 = (0..max_n)
            .filter(|i| mask & (1 << i) != 0)
            .map(|i| countermeasures[i].weight_kg)
            .sum();

        if total_weight > weight_budget_kg {
            continue;
        }

        // Collect active counter-primitives
        let counters: Vec<CounterPrimitive> = (0..max_n)
            .filter(|i| mask & (1 << i) != 0)
            .flat_map(|i| countermeasures[i].primary_counters.clone())
            .collect();

        // Compute fused detection
        let fusion = compute_fusion(sensors, &counters, matrix, range_m, raw_signature, 0.5);

        if fusion.fused_probability < best_prob {
            best_prob = fusion.fused_probability;
            best_mask = mask;
        }
    }

    // Reconstruct best loadout
    let selected: Vec<usize> = (0..max_n).filter(|i| best_mask & (1 << i) != 0).collect();

    let total_weight: f64 = selected.iter().map(|&i| countermeasures[i].weight_kg).sum();
    let total_power: f64 = selected.iter().map(|&i| countermeasures[i].power_w).sum();
    let active_counters: Vec<CounterPrimitive> = selected
        .iter()
        .flat_map(|&i| countermeasures[i].primary_counters.clone())
        .collect();

    OptimalLoadout {
        selected,
        total_weight_kg: total_weight,
        total_power_w: total_power,
        fused_probability: best_prob,
        active_counters,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{EnergyMode, LatencyClass, SpectralBand};

    fn test_sensor_suite() -> Vec<SensorSystem> {
        vec![
            SensorSystem {
                name: "EO Camera".into(),
                energy_mode: EnergyMode::Passive,
                spectral_band: SpectralBand::Visible,
                latency_class: LatencyClass::RealTime,
                primary_primitives: vec![
                    crate::primitives::SensingPrimitive::Reflection,
                    crate::primitives::SensingPrimitive::Contrast,
                ],
                max_range_m: 5000.0,
                noise_floor: 0.05,
            },
            SensorSystem {
                name: "FLIR".into(),
                energy_mode: EnergyMode::Passive,
                spectral_band: SpectralBand::Infrared,
                latency_class: LatencyClass::RealTime,
                primary_primitives: vec![
                    crate::primitives::SensingPrimitive::Emission,
                    crate::primitives::SensingPrimitive::Contrast,
                ],
                max_range_m: 3000.0,
                noise_floor: 0.08,
            },
        ]
    }

    fn test_countermeasures() -> Vec<Countermeasure> {
        vec![
            Countermeasure {
                name: "RAM Coating".into(),
                energy_mode: EnergyMode::Passive,
                primary_counters: vec![CounterPrimitive::Absorption],
                weight_kg: 2.0,
                power_w: 0.0,
                effectiveness: vec![0.85],
            },
            Countermeasure {
                name: "Thermal Insulation".into(),
                energy_mode: EnergyMode::Passive,
                primary_counters: vec![CounterPrimitive::ThermalEquilibrium],
                weight_kg: 1.5,
                power_w: 0.0,
                effectiveness: vec![0.80],
            },
            Countermeasure {
                name: "Adaptive Camo".into(),
                energy_mode: EnergyMode::Active,
                primary_counters: vec![
                    CounterPrimitive::Homogenization,
                    CounterPrimitive::Diffusion,
                ],
                weight_kg: 3.0,
                power_w: 50.0,
                effectiveness: vec![0.85, 0.80],
            },
        ]
    }

    #[test]
    fn fusion_higher_than_individual() {
        let sensors = test_sensor_suite();
        let matrix = EffectivenessMatrix::default_physics();
        let result = compute_fusion(&sensors, &[], &matrix, 1000.0, 0.8, 0.5);

        for assessment in &result.sensor_assessments {
            assert!(
                result.fused_probability >= assessment.detection_probability,
                "Fused should be >= individual"
            );
        }
    }

    #[test]
    fn optimizer_respects_weight_budget() {
        let sensors = test_sensor_suite();
        let cms = test_countermeasures();
        let matrix = EffectivenessMatrix::default_physics();

        let loadout = optimize_loadout(&sensors, &cms, &matrix, 4.0, 1000.0, 0.8);
        assert!(
            loadout.total_weight_kg <= 4.0,
            "Should respect weight budget: {} > 4.0",
            loadout.total_weight_kg
        );
    }

    #[test]
    fn optimizer_reduces_detection() {
        let sensors = test_sensor_suite();
        let cms = test_countermeasures();
        let matrix = EffectivenessMatrix::default_physics();

        let unprotected = compute_fusion(&sensors, &[], &matrix, 1000.0, 0.8, 0.5);
        let loadout = optimize_loadout(&sensors, &cms, &matrix, 10.0, 1000.0, 0.8);

        assert!(
            loadout.fused_probability <= unprotected.fused_probability,
            "Optimizer should reduce or maintain detection probability"
        );
    }
}
