use serde::{Deserialize, Serialize};

/// Represents the suit's internal sense of balance and absolute position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InertialState {
    /// Acceleration in X, Y, Z (m/s^2)
    pub acceleration: [f32; 3],
    /// Angular velocity in roll, pitch, yaw (rad/s)
    pub angular_velocity: [f32; 3],
    /// Absolute heading relative to magnetic North (radians)
    pub heading: f32,
    /// Altitude Above Ground Level (meters)
    pub altitude_agl: f32,
    /// Latitude and Longitude from GNSS RTK
    pub position_gps: Option<(f64, f64)>,
}

/// Triple Modular Redundant IMU sensor data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmrImuData {
    /// Sensor 0 raw data (accel, gyro)
    pub sensor_0: ([f32; 3], [f32; 3]),
    /// Sensor 1 raw data (accel, gyro)
    pub sensor_1: ([f32; 3], [f32; 3]),
    /// Sensor 2 raw data (accel, gyro)
    pub sensor_2: ([f32; 3], [f32; 3]),
}

impl TmrImuData {
    /// Simple median voting for TMR fault tolerance.
    pub fn vote(&self) -> ([f32; 3], [f32; 3]) {
        (
            median_vec3(self.sensor_0.0, self.sensor_1.0, self.sensor_2.0),
            median_vec3(self.sensor_0.1, self.sensor_1.1, self.sensor_2.1),
        )
    }
}

/// Raw magnetometer data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagnetometerData {
    /// Magnetic field vector (x, y, z) in Gauss/Micro-tesla
    pub field: [f32; 3],
}

/// Calibration parameters for hard-iron (bias) and soft-iron (matrix) compensation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagnetometerCalibration {
    /// Hard-iron bias (offset)
    pub hard_iron_bias: [f32; 3],
    /// Soft-iron correction matrix (3x3)
    pub soft_iron_matrix: [[f32; 3]; 3],
}

/// Trait for magnetometer hardware drivers.
pub trait Magnetometer {
    /// Reads raw magnetic field data.
    fn read_raw(&mut self) -> Result<MagnetometerData, nexcore_error::NexError>;
    /// Applies hard/soft iron calibration.
    fn calibrate(&self, raw: MagnetometerData, cal: &MagnetometerCalibration) -> MagnetometerData {
        let mut corrected = [0.0; 3];
        // Apply hard-iron: (raw - bias)
        let centered = [
            raw.field[0] - cal.hard_iron_bias[0],
            raw.field[1] - cal.hard_iron_bias[1],
            raw.field[2] - cal.hard_iron_bias[2],
        ];
        // Apply soft-iron matrix: matrix * centered
        for i in 0..3 {
            for j in 0..3 {
                corrected[i] += cal.soft_iron_matrix[i][j] * centered[j];
            }
        }
        MagnetometerData { field: corrected }
    }
}

fn median_vec3(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    [
        median(a[0], b[0], c[0]),
        median(a[1], b[1], c[1]),
        median(a[2], b[2], c[2]),
    ]
}

fn median(a: f32, b: f32, c: f32) -> f32 {
    if (a <= b && b <= c) || (c <= b && b <= a) {
        b
    } else if (b <= a && a <= c) || (c <= a && a <= b) {
        a
    } else {
        c
    }
}

/// Raw barometric pressure data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarometricData {
    /// Pressure in hectopascals (hPa)
    pub pressure: f32,
    /// Temperature in Celsius (needed for pressure-to-altitude compensation)
    pub temperature: f32,
}

/// Trait for barometer hardware drivers.
pub trait Barometer {
    /// Reads raw pressure and temperature data.
    fn read_sensor_data(&mut self) -> Result<BarometricData, nexcore_error::NexError>;
    /// Estimates altitude (meters) given the current pressure and baseline sea-level pressure.
    fn estimate_altitude(&self, data: &BarometricData, sea_level_hpa: f32) -> f32 {
        // Simple barometric formula: h = 44330 * (1 - (p/p0)^(1/5.255))
        44330.0 * (1.0 - (data.pressure / sea_level_hpa).powf(1.0 / 5.255))
    }
}
