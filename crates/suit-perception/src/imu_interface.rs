//! # IMU Hardware Interface
//!
//! Typed abstraction over suit IMU hardware using `suit_sensors::imu`.
//! Supports TMR (Triple Modular Redundant) arrays with automatic calibration.

use nexcore_error::NexError as Error;
use suit_sensors::imu::{ImuCalibration, ImuReading};

/// Interface for physical IMU hardware.
///
/// Returns typed `ImuReading` from `suit-sensors` instead of raw f32 arrays.
pub trait HardwareImu {
    /// Reads the current 9-DOF IMU state from the sensor.
    fn read(&mut self) -> Result<ImuReading, Error>;
    /// Resets the IMU sensor.
    fn reset(&mut self) -> Result<(), Error>;
    /// Returns the unique hardware identifier.
    fn get_id(&self) -> u8;
    /// Returns the calibration data for this sensor, if available.
    fn calibration(&self) -> Option<&ImuCalibration>;
}

/// A collection of three redundant IMUs for TMR fault tolerance.
pub struct ImuArray<T: HardwareImu> {
    /// Array of sensors.
    pub sensors: [T; 3],
}

impl<T: HardwareImu> ImuArray<T> {
    /// Polls all sensors in the array and returns their readings.
    ///
    /// Each reading is automatically calibrated if the sensor provides
    /// calibration data via `HardwareImu::calibration()`.
    pub fn poll(&mut self) -> Result<crate::vestibular::TmrImuData, Error> {
        let r0 = self.read_calibrated(0)?;
        let r1 = self.read_calibrated(1)?;
        let r2 = self.read_calibrated(2)?;

        Ok(crate::vestibular::TmrImuData {
            sensor_0: reading_to_tuple(&r0),
            sensor_1: reading_to_tuple(&r1),
            sensor_2: reading_to_tuple(&r2),
        })
    }

    /// Polls all sensors and returns typed `ImuReading` values (no tuple conversion).
    pub fn poll_typed(&mut self) -> Result<[ImuReading; 3], Error> {
        let r0 = self.read_calibrated(0)?;
        let r1 = self.read_calibrated(1)?;
        let r2 = self.read_calibrated(2)?;
        Ok([r0, r1, r2])
    }

    fn read_calibrated(&mut self, idx: usize) -> Result<ImuReading, Error> {
        let raw = self.sensors[idx].read()?;
        match self.sensors[idx].calibration() {
            Some(cal) => Ok(suit_sensors::imu::calibrate(&raw, cal)),
            None => Ok(raw),
        }
    }
}

/// Convert an `ImuReading` to the legacy `([f32; 3], [f32; 3])` tuple format.
///
/// Returns `(accel_xyz, gyro_xyz)` for backward compatibility with `TmrImuData`.
fn reading_to_tuple(r: &ImuReading) -> ([f32; 3], [f32; 3]) {
    (
        [r.accel.x, r.accel.y, r.accel.z],
        [r.gyro.x, r.gyro.y, r.gyro.z],
    )
}
