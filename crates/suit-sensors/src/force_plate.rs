//! Force plate sensor abstraction.
//!
//! Multi-axis ground reaction force measurement for gait analysis,
//! balance assessment, and jump/landing detection.
//!
//! ## T1 Grounding
//! - `N` (quantity) — force magnitude in Newtons
//! - `λ` (location) — center of pressure position
//! - `→` (causality) — ground reaction forces cause postural response

use suit_primitives::Vector3;

/// Single force plate reading.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ForcePlateReading {
    /// Ground reaction force vector (N) — vertical (z), anterior-posterior (y), medial-lateral (x).
    pub force: Vector3,
    /// Moment vector (N·m) about the plate origin.
    pub moment: Vector3,
    /// Center of pressure position (m) on the plate surface.
    pub cop: CenterOfPressure,
    /// Monotonic timestamp (microseconds since boot).
    pub timestamp_us: u64,
}

/// Center of pressure on a force plate.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct CenterOfPressure {
    /// X position (m) — medial-lateral.
    pub x: f32,
    /// Y position (m) — anterior-posterior.
    pub y: f32,
}

/// Force plate configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ForcePlateConfig {
    /// Full-scale range per axis (N).
    pub max_force_n: f32,
    /// Sample rate (Hz). Typical: 1000-2000 Hz.
    pub sample_rate_hz: u16,
    /// Plate dimensions (m): width × length.
    pub plate_width_m: f32,
    /// Plate length (m).
    pub plate_length_m: f32,
    /// Which foot this plate is under.
    pub foot: Foot,
}

/// Foot assignment for bilateral force plates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Foot {
    /// Left foot plate.
    Left,
    /// Right foot plate.
    Right,
}

/// Compute vertical force magnitude from a reading.
pub fn vertical_force(reading: &ForcePlateReading) -> f32 {
    reading.force.z
}

/// Detect stance phase: vertical force exceeds threshold.
pub fn is_stance(reading: &ForcePlateReading, threshold_n: f32) -> bool {
    reading.force.z > threshold_n
}

/// Compute bilateral weight distribution (0.0 = all left, 1.0 = all right).
pub fn weight_distribution(left: &ForcePlateReading, right: &ForcePlateReading) -> f32 {
    let total = left.force.z + right.force.z;
    if total < 1.0 {
        0.5 // airborne or negligible — return neutral
    } else {
        right.force.z / total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stance_detection() {
        let reading = ForcePlateReading {
            force: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 750.0,
            },
            moment: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            cop: CenterOfPressure { x: 0.0, y: 0.0 },
            timestamp_us: 0,
        };
        assert!(is_stance(&reading, 20.0));
        assert!(!is_stance(&reading, 800.0));
    }

    #[test]
    fn test_weight_distribution_balanced() {
        let left = ForcePlateReading {
            force: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 400.0,
            },
            moment: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            cop: CenterOfPressure { x: 0.0, y: 0.0 },
            timestamp_us: 0,
        };
        let right = ForcePlateReading {
            force: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 400.0,
            },
            moment: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            cop: CenterOfPressure { x: 0.0, y: 0.0 },
            timestamp_us: 0,
        };
        assert!((weight_distribution(&left, &right) - 0.5).abs() < 1e-6);
    }
}
