//! IMU (Inertial Measurement Unit) sensor abstraction.
//!
//! Covers 3-axis accelerometer, 3-axis gyroscope, and optional magnetometer.
//! Raw readings at configurable sample rates (100-1000 Hz).
//!
//! ## T1 Grounding
//! - `ς` (state) — raw sensor readings are instantaneous state
//! - `ν` (frequency) — sample rate is the measurement frequency
//! - `λ` (location) — each IMU has a body-fixed mounting position

use suit_primitives::Vector3;

/// Raw 9-DOF IMU reading at a single instant.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ImuReading {
    /// Linear acceleration (m/s²) in sensor frame.
    pub accel: Vector3,
    /// Angular velocity (rad/s) in sensor frame.
    pub gyro: Vector3,
    /// Magnetic field (µT) in sensor frame. `None` if magnetometer absent.
    pub mag: Option<Vector3>,
    /// Sensor temperature (°C) for thermal compensation.
    pub temperature_c: f32,
    /// Monotonic timestamp (microseconds since boot).
    pub timestamp_us: u64,
}

/// IMU configuration parameters.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImuConfig {
    /// Accelerometer full-scale range in g (±2, ±4, ±8, ±16).
    pub accel_range_g: u8,
    /// Gyroscope full-scale range in dps (±250, ±500, ±1000, ±2000).
    pub gyro_range_dps: u16,
    /// Output data rate in Hz.
    pub sample_rate_hz: u16,
    /// Mounting position on the body (body-frame offset from center of mass).
    pub mount_position: Vector3,
    /// Mounting orientation as Euler angles (roll, pitch, yaw in radians).
    pub mount_orientation: Vector3,
}

/// Calibration data for bias and scale correction.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImuCalibration {
    /// Accelerometer bias (m/s²).
    pub accel_bias: Vector3,
    /// Accelerometer scale factor (dimensionless, near 1.0).
    pub accel_scale: Vector3,
    /// Gyroscope bias (rad/s).
    pub gyro_bias: Vector3,
    /// Magnetometer hard-iron offset (µT).
    pub mag_hard_iron: Vector3,
    /// Temperature at which calibration was performed (°C).
    pub cal_temperature_c: f32,
}

/// Apply calibration to a raw IMU reading.
pub fn calibrate(raw: &ImuReading, cal: &ImuCalibration) -> ImuReading {
    ImuReading {
        accel: Vector3 {
            x: (raw.accel.x - cal.accel_bias.x) * cal.accel_scale.x,
            y: (raw.accel.y - cal.accel_bias.y) * cal.accel_scale.y,
            z: (raw.accel.z - cal.accel_bias.z) * cal.accel_scale.z,
        },
        gyro: Vector3 {
            x: raw.gyro.x - cal.gyro_bias.x,
            y: raw.gyro.y - cal.gyro_bias.y,
            z: raw.gyro.z - cal.gyro_bias.z,
        },
        mag: raw.mag.map(|m| Vector3 {
            x: m.x - cal.mag_hard_iron.x,
            y: m.y - cal.mag_hard_iron.y,
            z: m.z - cal.mag_hard_iron.z,
        }),
        temperature_c: raw.temperature_c,
        timestamp_us: raw.timestamp_us,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calibrate_removes_bias() {
        let raw = ImuReading {
            accel: Vector3 {
                x: 0.1,
                y: 0.2,
                z: 9.91,
            },
            gyro: Vector3 {
                x: 0.01,
                y: -0.02,
                z: 0.005,
            },
            mag: None,
            temperature_c: 25.0,
            timestamp_us: 1000,
        };
        let cal = ImuCalibration {
            accel_bias: Vector3 {
                x: 0.1,
                y: 0.2,
                z: 0.1,
            },
            accel_scale: Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            gyro_bias: Vector3 {
                x: 0.01,
                y: -0.02,
                z: 0.005,
            },
            mag_hard_iron: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            cal_temperature_c: 25.0,
        };
        let result = calibrate(&raw, &cal);
        assert!((result.accel.x).abs() < 1e-6);
        assert!((result.accel.y).abs() < 1e-6);
        assert!((result.accel.z - 9.81).abs() < 0.01);
        assert!((result.gyro.x).abs() < 1e-6);
    }
}
