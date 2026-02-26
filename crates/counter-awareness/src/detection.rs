//! # Detection Probability Model
//!
//! ## Mathematical Formalization
//!
//! Single-sensor detection probability:
//! ```text
//! P_detect(sensor, target, range) =
//!     (1 - exp(-SNR)) × A(range)
//!
//! where:
//!   S_residual = S_raw × ∏(1 - E(counter_i, primitive_j))
//!   SNR        = max(S_residual_i) / noise_floor   [max over sensor's primitives]
//!   A(range)   = exp(-α × range / max_range)       [atmospheric attenuation]
//!   α          = attenuation coefficient (band-dependent)
//! ```
//!
//! The exponential saturation model `1 - exp(-SNR)` is used instead of a linear clamp
//! because it gives a smooth, physically-motivated curve: detection rises steeply when
//! residual signature first clears the noise floor, then saturates toward 1.0 as SNR
//! grows large. Countermeasures always reduce detection even when residual remains above
//! the noise floor.
//!
//! ## Lex Primitiva Grounding
//! `DetectionAssessment` → N (Quantity) × κ (Comparison) — numerical probability compared against threshold

use serde::{Deserialize, Serialize};

use crate::matrix::EffectivenessMatrix;
use crate::primitives::{CounterPrimitive, SensingPrimitive, SensorSystem, SpectralBand};

/// Atmospheric attenuation coefficients by spectral band.
///
/// α values model how quickly signal strength degrades with range.
/// Higher α = faster degradation = shorter effective detection range.
///
/// Tier: T2-C (composed of frequency + distance + intensity)
fn attenuation_coefficient(band: &SpectralBand) -> f64 {
    match band {
        SpectralBand::Visible => 1.0,      // Moderate — affected by haze, smoke
        SpectralBand::Infrared => 0.8,     // Lower — penetrates some obscurants
        SpectralBand::NearInfrared => 0.9, // Between visible and IR
        SpectralBand::Microwave => 0.3,    // Low — penetrates clouds, rain
        SpectralBand::Ultraviolet => 1.5,  // High — scattered by atmosphere
        SpectralBand::Multispectral => 0.7, // Weighted average across bands
    }
}

/// Range attenuation factor.
///
/// ```text
/// A(range, max_range, α) = exp(-α × range / max_range)
/// ```
///
/// Returns 1.0 at range=0, decays exponentially toward 0.
fn range_attenuation(range_m: f64, max_range_m: f64, alpha: f64) -> f64 {
    if max_range_m <= 0.0 {
        return 0.0;
    }
    (-alpha * range_m / max_range_m).exp()
}

/// Result of a detection probability assessment.
///
/// Tier: T3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionAssessment {
    /// Sensor that performed the assessment
    pub sensor_name: String,
    /// Raw signature strength before countermeasures [0.0, 1.0]
    pub raw_signature: f64,
    /// Residual signature after countermeasures [0.0, 1.0]
    pub residual_signature: f64,
    /// Range attenuation factor [0.0, 1.0]
    pub range_factor: f64,
    /// Final detection probability [0.0, 1.0]
    pub detection_probability: f64,
    /// Per-primitive breakdown
    pub primitive_contributions: Vec<PrimitiveContribution>,
}

/// Contribution of a single sensing primitive to detection.
///
/// Tier: T2-C
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveContribution {
    pub primitive: SensingPrimitive,
    /// Raw contribution [0.0, 1.0]
    pub raw: f64,
    /// After countermeasures [0.0, 1.0]
    pub residual: f64,
    /// Reduction achieved [0.0, 1.0]
    pub reduction: f64,
}

/// Compute single-sensor detection probability.
///
/// ## Algorithm
/// 1. For each sensing primitive the sensor relies on:
///    - Compute residual = raw × ∏\(1 - M\[primitive\]\[counter\]\)
/// 2. Aggregate residuals across primitives (max — weakest link model)
/// 3. Compare against noise floor
/// 4. Apply range attenuation
///
/// ## Why MAX aggregation?
/// Detection succeeds if ANY primitive yields a detectable signature.
/// A target invisible on reflection but bright on emission is still detected
/// by a sensor that uses both primitives.
pub fn compute_detection(
    sensor: &SensorSystem,
    counters: &[CounterPrimitive],
    matrix: &EffectivenessMatrix,
    range_m: f64,
    raw_signature: f64,
) -> DetectionAssessment {
    let alpha = attenuation_coefficient(&sensor.spectral_band);
    let range_factor = range_attenuation(range_m, sensor.max_range_m, alpha);

    let mut contributions = Vec::new();
    let mut max_residual: f64 = 0.0;

    for primitive in &sensor.primary_primitives {
        let residual_fraction = matrix.residual_fraction(*primitive, counters);
        let raw_contrib = raw_signature;
        let residual_contrib = raw_contrib * residual_fraction;

        contributions.push(PrimitiveContribution {
            primitive: *primitive,
            raw: raw_contrib,
            residual: residual_contrib,
            reduction: 1.0 - residual_fraction,
        });

        if residual_contrib > max_residual {
            max_residual = residual_contrib;
        }
    }

    // Detection probability via exponential signal-to-noise model:
    //   P = (1 - exp(-SNR)) × range_factor
    // This gives a smooth curve: higher SNR → higher detection,
    // but countermeasures that reduce residual always reduce detection
    // even when signal remains above noise floor.
    let snr = if sensor.noise_floor > 0.0 {
        max_residual / sensor.noise_floor
    } else {
        max_residual * 100.0 // Very sensitive sensor
    };
    let signal_contribution = 1.0 - (-snr).exp();

    let detection_probability = (signal_contribution * range_factor).clamp(0.0, 1.0);

    DetectionAssessment {
        sensor_name: sensor.name.clone(),
        raw_signature,
        residual_signature: max_residual,
        range_factor,
        detection_probability,
        primitive_contributions: contributions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::{EnergyMode, LatencyClass};

    fn test_eo_sensor() -> SensorSystem {
        SensorSystem {
            name: "EO Camera".into(),
            energy_mode: EnergyMode::Passive,
            spectral_band: SpectralBand::Visible,
            latency_class: LatencyClass::RealTime,
            primary_primitives: vec![
                SensingPrimitive::Reflection,
                SensingPrimitive::Contrast,
                SensingPrimitive::Boundary,
            ],
            max_range_m: 5000.0,
            noise_floor: 0.05,
        }
    }

    #[test]
    fn no_countermeasures_high_detection() {
        let sensor = test_eo_sensor();
        let matrix = EffectivenessMatrix::default_physics();
        let result = compute_detection(&sensor, &[], &matrix, 1000.0, 0.8);
        assert!(
            result.detection_probability > 0.5,
            "Unprotected target should be detectable: {}",
            result.detection_probability
        );
    }

    #[test]
    fn full_countermeasures_reduce_detection() {
        let sensor = test_eo_sensor();
        let matrix = EffectivenessMatrix::default_physics();

        let unprotected = compute_detection(&sensor, &[], &matrix, 1000.0, 0.8);
        let protected = compute_detection(
            &sensor,
            &[
                CounterPrimitive::Absorption,
                CounterPrimitive::Homogenization,
                CounterPrimitive::Diffusion,
            ],
            &matrix,
            1000.0,
            0.8,
        );

        assert!(
            protected.detection_probability < unprotected.detection_probability,
            "Countermeasures should reduce detection: {} vs {}",
            protected.detection_probability,
            unprotected.detection_probability,
        );
    }

    #[test]
    fn range_reduces_detection() {
        let sensor = test_eo_sensor();
        let matrix = EffectivenessMatrix::default_physics();

        let close = compute_detection(&sensor, &[], &matrix, 100.0, 0.8);
        let far = compute_detection(&sensor, &[], &matrix, 4500.0, 0.8);

        assert!(
            far.detection_probability < close.detection_probability,
            "Greater range should reduce detection: {} vs {}",
            far.detection_probability,
            close.detection_probability,
        );
    }

    #[test]
    fn zero_range_max_detection() {
        let sensor = test_eo_sensor();
        let matrix = EffectivenessMatrix::default_physics();
        let result = compute_detection(&sensor, &[], &matrix, 0.0, 1.0);
        assert!(
            (result.range_factor - 1.0).abs() < 1e-10,
            "Range factor at zero should be 1.0"
        );
    }
}
