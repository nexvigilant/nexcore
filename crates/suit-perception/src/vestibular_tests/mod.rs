use crate::vestibular::*;

#[test]
fn test_tmr_imu_voting() {
    let tmr_data = TmrImuData {
        sensor_0: ([1.0, 0.0, 0.0], [0.1, 0.0, 0.0]),
        sensor_1: ([1.1, 0.0, 0.0], [0.1, 0.0, 0.0]),
        sensor_2: ([0.9, 0.0, 0.0], [0.1, 0.0, 0.0]),
    };
    let (accel, gyro) = tmr_data.vote();
    assert!((accel[0] - 1.0).abs() < 1e-6);
    assert!((gyro[0] - 0.1).abs() < 1e-6);
}

struct MockMag;
impl Magnetometer for MockMag {
    fn read_raw(&mut self) -> Result<MagnetometerData, nexcore_error::NexError> {
        Ok(MagnetometerData {
            field: [100.0, 100.0, 100.0],
        })
    }
}

#[test]
fn test_mag_calibration() {
    let mut mag = MockMag;
    let raw = mag.read_raw().unwrap();
    let cal = MagnetometerCalibration {
        hard_iron_bias: [10.0, 10.0, 10.0],
        soft_iron_matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };
    let calibrated = mag.calibrate(raw, &cal);
    assert!((calibrated.field[0] - 90.0).abs() < 1e-6);
}

struct MockBaro;
impl Barometer for MockBaro {
    fn read_sensor_data(&mut self) -> Result<BarometricData, nexcore_error::NexError> {
        Ok(BarometricData {
            pressure: 1013.25,
            temperature: 20.0,
        })
    }
}

#[test]
fn test_baro_altitude() {
    let baro = MockBaro;
    let data = BarometricData {
        pressure: 1013.25,
        temperature: 20.0,
    };
    let altitude = baro.estimate_altitude(&data, 1013.25);
    assert!((altitude - 0.0).abs() < 1e-6);
}
