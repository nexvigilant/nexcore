//! # Cardio Bridge
//!
//! Inter-crate pipeline: Cardiovascular → Circulatory.
//!
//! Converts cardiovascular `PumpResult` output into circulatory `BloodCell`
//! input, and maps `CardiacVitals` to circulatory `BloodPressure`.
//!
//! ```text
//! Cardiovascular::PumpResult → BloodCell
//! Cardiovascular::CardiacVitals → BloodPressure
//! ```

use nexcore_cardiovascular::{CardiacVitals, PumpResult};

use crate::{BloodCell, BloodPressure};

/// Convert a cardiovascular `PumpResult` into a circulatory `BloodCell`.
///
/// **Biological mapping**: Aortic output — blood ejected from the left
/// ventricle enters the systemic circulation as data-carrying cells.
pub fn pump_result_to_cell(result: &PumpResult) -> BloodCell {
    BloodCell::data(
        "cardiovascular",
        serde_json::json!({
            "stroke_volume": result.stroke_volume,
            "beat_number": result.beat_number,
            "cardiac_output": result.cardiac_output.value(),
        }),
    )
}

/// Convert `CardiacVitals` to circulatory `BloodPressure`.
///
/// Maps cardiac output to total capacity and stroke volume to available.
///
/// **Biological mapping**: Arterial pressure measurement — systemic
/// blood pressure reflects cardiac output against vascular resistance.
pub fn vitals_to_blood_pressure(vitals: &CardiacVitals) -> BloodPressure {
    // cardiac_output = stroke_volume * heart_rate / 60
    // total represents max capacity; available represents utilized fraction
    let total = (vitals.cardiac_output.value() * 100.0) as usize;
    let available =
        (vitals.cardiac_output.value() * vitals.stroke_volume.min(1.0) * 100.0) as usize;
    BloodPressure::new(total.max(1), available)
}

/// Compute the throughput value from a pump result.
///
/// Returns a scalar throughput representing cardiac output flow rate.
/// This value can be traced through the full circulation chain.
pub fn pump_throughput(result: &PumpResult) -> f64 {
    result.cardiac_output.value()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_cardiovascular::Heart;

    #[test]
    fn test_pump_result_to_cell() {
        let mut heart = Heart::new();
        let result = heart.pump(80.0);

        let cell = pump_result_to_cell(&result);

        assert_eq!(cell.source, "cardiovascular");
        assert_eq!(cell.kind, crate::CellKind::Data);
        assert!(!cell.processed);
        // Payload should contain pump data
        assert!(cell.payload.get("stroke_volume").is_some());
        assert!(cell.payload.get("beat_number").is_some());
        assert!(cell.payload.get("cardiac_output").is_some());
    }

    #[test]
    fn test_vitals_to_blood_pressure() {
        let mut heart = Heart::new();
        let _ = heart.pump(80.0);
        let resistance = nexcore_cardiovascular::Resistance::new(1.0);
        let vitals = heart.vitals(resistance);

        let pressure = vitals_to_blood_pressure(&vitals);

        assert!(pressure.total > 0);
        assert!(pressure.available <= pressure.total);
    }

    #[test]
    fn test_pump_throughput() {
        let mut heart = Heart::new();
        let result = heart.pump(80.0);

        let throughput = pump_throughput(&result);
        assert!(throughput > 0.0, "Throughput should be positive after pump");
    }

    #[test]
    fn test_cell_payload_valid_json() {
        let mut heart = Heart::new();
        let result = heart.pump(50.0);
        let cell = pump_result_to_cell(&result);

        // Verify payload is valid JSON with expected fields
        let sv = cell.payload["stroke_volume"].as_f64();
        assert!(sv.is_some(), "stroke_volume should be a number");
        assert!(sv.is_some_and(|v| v > 0.0));
    }

    #[test]
    fn test_higher_preload_higher_throughput() {
        let mut heart_low = Heart::new();
        let mut heart_high = Heart::new();

        let result_low = heart_low.pump(30.0);
        let result_high = heart_high.pump(100.0);

        assert!(
            pump_throughput(&result_high) > pump_throughput(&result_low),
            "Higher preload should produce higher throughput"
        );
    }
}
