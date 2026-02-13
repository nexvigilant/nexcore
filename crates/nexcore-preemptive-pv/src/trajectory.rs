//! Temporal Trajectory with Hill Amplification.
//!
//! Tier: T2-C (maps to Frequency `nu` + Sequence `sigma` + Causality `->` + Comparison `kappa`)
//!
//! Computes the temporal trajectory Gamma of a reporting rate signal:
//!
//! ```text
//! Gamma(d, e, t) = dR/dt + alpha * d^2R/dt^2
//! ```
//!
//! Where:
//! - `dR/dt` = velocity (first derivative) of reporting rate
//! - `d^2R/dt^2` = acceleration (second derivative) of reporting rate
//! - `alpha` = weight for acceleration term (default 0.5)
//!
//! Hill amplification models cooperative binding:
//!
//! ```text
//! Gamma_amp = Gamma^nH / (K_half^nH + Gamma^nH)
//! ```
//!
//! With `nH > 1` for cooperative amplification of escalating signals.

use crate::types::ReportingDataPoint;

/// Default weight for the acceleration term in trajectory calculation.
pub const DEFAULT_ALPHA: f64 = 0.5;

/// Default Hill coefficient for cooperative amplification.
pub const DEFAULT_HILL_COEFFICIENT: f64 = 2.0;

/// Default half-maximal constant for Hill amplification.
pub const DEFAULT_K_HALF: f64 = 1.0;

/// Computes the raw trajectory Gamma from a time series of reporting rates.
///
/// Uses finite differences to estimate velocity (dR/dt) and acceleration (d^2R/dt^2):
///
/// ```text
/// Gamma = dR/dt + alpha * d^2R/dt^2
/// ```
///
/// Requires at least 3 data points for acceleration estimation.
/// With 2 data points, only velocity is used.
/// With 0-1 data points, returns 0.0.
///
/// # Arguments
///
/// * `data` - Time-series reporting rate data points (must be sorted by time).
/// * `alpha` - Weight for the acceleration term.
#[must_use]
pub fn gamma(data: &[ReportingDataPoint], alpha: f64) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }

    let velocity = compute_velocity(data);

    if data.len() < 3 {
        return velocity;
    }

    let acceleration = compute_acceleration(data);
    velocity + alpha * acceleration
}

/// Computes the raw trajectory with default alpha.
#[must_use]
pub fn gamma_default(data: &[ReportingDataPoint]) -> f64 {
    gamma(data, DEFAULT_ALPHA)
}

/// Applies Hill amplification to the trajectory value.
///
/// ```text
/// Gamma_amp = Gamma^nH / (K_half^nH + Gamma^nH)
/// ```
///
/// For Gamma <= 0 (decelerating signal), returns 0.0.
///
/// # Arguments
///
/// * `gamma_value` - The raw trajectory value.
/// * `n_h` - Hill coefficient (> 1 for cooperative amplification).
/// * `k_half` - Half-maximal concentration constant.
#[must_use]
pub fn hill_amplify(gamma_value: f64, n_h: f64, k_half: f64) -> f64 {
    if gamma_value <= 0.0 || k_half <= 0.0 {
        return 0.0;
    }

    let g_n = gamma_value.powf(n_h);
    let k_n = k_half.powf(n_h);
    g_n / (k_n + g_n)
}

/// Computes the Hill-amplified trajectory from raw data with default parameters.
///
/// Combines `gamma_default()` and `hill_amplify()` with default constants.
#[must_use]
pub fn gamma_amplified(data: &[ReportingDataPoint]) -> f64 {
    let raw = gamma_default(data);
    hill_amplify(raw, DEFAULT_HILL_COEFFICIENT, DEFAULT_K_HALF)
}

/// Computes the velocity (first derivative) using the last two data points.
fn compute_velocity(data: &[ReportingDataPoint]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let n = data.len();
    let dt = data[n - 1].time - data[n - 2].time;
    if dt.abs() < f64::EPSILON {
        return 0.0;
    }
    (data[n - 1].rate - data[n - 2].rate) / dt
}

/// Computes the acceleration (second derivative) using the last three data points.
fn compute_acceleration(data: &[ReportingDataPoint]) -> f64 {
    if data.len() < 3 {
        return 0.0;
    }
    let n = data.len();

    let dt1 = data[n - 2].time - data[n - 3].time;
    let dt2 = data[n - 1].time - data[n - 2].time;

    if dt1.abs() < f64::EPSILON || dt2.abs() < f64::EPSILON {
        return 0.0;
    }

    let v1 = (data[n - 2].rate - data[n - 3].rate) / dt1;
    let v2 = (data[n - 1].rate - data[n - 2].rate) / dt2;

    let dt_avg = (dt1 + dt2) / 2.0;
    if dt_avg.abs() < f64::EPSILON {
        return 0.0;
    }

    (v2 - v1) / dt_avg
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_data(points: &[(f64, f64)]) -> Vec<ReportingDataPoint> {
        points
            .iter()
            .map(|&(t, r)| ReportingDataPoint::new(t, r))
            .collect()
    }

    #[test]
    fn gamma_empty_data() {
        let data: Vec<ReportingDataPoint> = vec![];
        assert!((gamma(&data, 0.5) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn gamma_single_point() {
        let data = make_data(&[(1.0, 5.0)]);
        assert!((gamma(&data, 0.5) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn gamma_two_points_velocity_only() {
        // velocity = (10 - 5) / (2 - 1) = 5.0
        let data = make_data(&[(1.0, 5.0), (2.0, 10.0)]);
        let result = gamma(&data, 0.5);
        assert!((result - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn gamma_constant_rate() {
        // Constant rate -> velocity = 0, acceleration = 0
        let data = make_data(&[(1.0, 5.0), (2.0, 5.0), (3.0, 5.0)]);
        let result = gamma(&data, 0.5);
        assert!(result.abs() < f64::EPSILON);
    }

    #[test]
    fn gamma_linear_increase() {
        // Linear: rate = 5*t, velocity = 5, acceleration = 0
        let data = make_data(&[(1.0, 5.0), (2.0, 10.0), (3.0, 15.0)]);
        let result = gamma(&data, 0.5);
        // velocity = 5, accel = 0, gamma = 5 + 0.5*0 = 5
        assert!((result - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn gamma_accelerating_signal() {
        // Quadratic-like: rates increasing faster
        let data = make_data(&[(1.0, 2.0), (2.0, 5.0), (3.0, 11.0)]);
        // v1 = (5-2)/1 = 3, v2 = (11-5)/1 = 6
        // velocity = 6, accel = (6-3)/1 = 3
        // gamma = 6 + 0.5 * 3 = 7.5
        let result = gamma(&data, 0.5);
        assert!((result - 7.5).abs() < f64::EPSILON);
    }

    #[test]
    fn gamma_decelerating_signal() {
        // Rates increasing but slowing down
        let data = make_data(&[(1.0, 2.0), (2.0, 8.0), (3.0, 11.0)]);
        // v1 = (8-2)/1 = 6, v2 = (11-8)/1 = 3
        // velocity = 3, accel = (3-6)/1 = -3
        // gamma = 3 + 0.5 * (-3) = 1.5
        let result = gamma(&data, 0.5);
        assert!((result - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn hill_amplify_zero_gamma() {
        assert!((hill_amplify(0.0, 2.0, 1.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hill_amplify_negative_gamma() {
        assert!((hill_amplify(-1.0, 2.0, 1.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hill_amplify_at_k_half() {
        // When gamma = K_half, result = K^n / (K^n + K^n) = 0.5
        let result = hill_amplify(1.0, 2.0, 1.0);
        assert!((result - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn hill_amplify_high_gamma() {
        // When gamma >> K_half, result approaches 1.0
        let result = hill_amplify(100.0, 2.0, 1.0);
        assert!(result > 0.99);
    }

    #[test]
    fn hill_amplify_low_gamma() {
        // When gamma << K_half, result approaches 0.0
        let result = hill_amplify(0.01, 2.0, 1.0);
        assert!(result < 0.01);
    }

    #[test]
    fn hill_amplify_cooperative_n_h_3() {
        // Higher nH = steeper transition
        let result_low = hill_amplify(0.5, 3.0, 1.0);
        let result_high = hill_amplify(2.0, 3.0, 1.0);
        // With nH=3: 0.5^3/(1+0.5^3) = 0.125/1.125 ~ 0.111
        assert!((result_low - 0.125 / 1.125).abs() < 1e-10);
        // 2^3/(1+2^3) = 8/9 ~ 0.889
        assert!((result_high - 8.0 / 9.0).abs() < 1e-10);
    }

    #[test]
    fn hill_amplify_zero_k_half() {
        assert!((hill_amplify(5.0, 2.0, 0.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn gamma_amplified_escalating() {
        let data = make_data(&[(1.0, 2.0), (2.0, 5.0), (3.0, 11.0)]);
        let result = gamma_amplified(&data);
        // raw gamma = 7.5, hill(7.5, 2, 1) = 56.25 / (1 + 56.25) = 0.9825...
        let expected_raw: f64 = 7.5;
        let expected_hill = expected_raw.powi(2) / (1.0 + expected_raw.powi(2));
        assert!((result - expected_hill).abs() < 1e-10);
    }
}
