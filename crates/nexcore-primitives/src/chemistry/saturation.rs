//! # Saturation Kinetics (Michaelis-Menten)
//!
//! **T1 Components**: maximum × cause × effect × ratio × transition
//!
//! **Chemistry**: v = Vmax × [S] / (Km + [S])
//!
//! **Universal Pattern**: Output asymptotically approaches maximum as input
//! increases. Systems have finite capacity.
//!
//! **PV Application**: Case processing throughput - systems saturate at capacity.

use nexcore_error::Error;

/// Errors for saturation calculations.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum SaturationError {
    /// Parameter must be non-negative.
    #[error("All parameters must be non-negative")]
    NegativeParameter,
    /// Half-saturation constant must be positive.
    #[error("Half-saturation constant (Km) must be positive")]
    ZeroHalfSaturation,
}

/// Saturation kinetics configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct SaturationKinetics {
    /// Maximum rate (Vmax)
    pub v_max: f64,
    /// Half-saturation constant (Km) - load at 50% capacity
    pub k_m: f64,
}

impl SaturationKinetics {
    /// Create new saturation kinetics model.
    pub fn new(v_max: f64, k_m: f64) -> Result<Self, SaturationError> {
        if v_max < 0.0 || k_m < 0.0 {
            return Err(SaturationError::NegativeParameter);
        }
        if k_m == 0.0 {
            return Err(SaturationError::ZeroHalfSaturation);
        }
        Ok(Self { v_max, k_m })
    }

    /// Calculate throughput at given load.
    pub fn throughput(&self, load: f64) -> Result<f64, SaturationError> {
        if load < 0.0 {
            return Err(SaturationError::NegativeParameter);
        }
        Ok((self.v_max * load) / (self.k_m + load))
    }

    /// Calculate utilization fraction (0.0 - 1.0).
    pub fn utilization(&self, load: f64) -> Result<f64, SaturationError> {
        if load < 0.0 {
            return Err(SaturationError::NegativeParameter);
        }
        Ok(load / (self.k_m + load))
    }

    /// Calculate load needed for target throughput.
    ///
    /// Returns None if target exceeds Vmax.
    pub fn required_load(&self, target: f64) -> Option<f64> {
        if target >= self.v_max || target < 0.0 {
            None
        } else {
            Some((self.k_m * target) / (self.v_max - target))
        }
    }
}

/// Calculate Michaelis-Menten rate.
///
/// v = Vmax × [S] / (Km + [S])
///
/// # Arguments
/// * `substrate` - Input load [S]
/// * `v_max` - Maximum rate
/// * `k_m` - Half-saturation constant
pub fn michaelis_menten_rate(substrate: f64, v_max: f64, k_m: f64) -> Result<f64, SaturationError> {
    if substrate < 0.0 || v_max < 0.0 || k_m < 0.0 {
        return Err(SaturationError::NegativeParameter);
    }
    if k_m == 0.0 {
        return Ok(if substrate > 0.0 { v_max } else { 0.0 });
    }
    Ok((v_max * substrate) / (k_m + substrate))
}

/// Calculate saturation fraction (0.0 - 1.0).
///
/// fraction = [S] / (Km + [S])
pub fn saturation_fraction(
    concentration: f64,
    half_saturation: f64,
) -> Result<f64, SaturationError> {
    if concentration < 0.0 {
        return Err(SaturationError::NegativeParameter);
    }
    if half_saturation <= 0.0 {
        return Err(SaturationError::ZeroHalfSaturation);
    }
    Ok(concentration / (half_saturation + concentration))
}

/// Calculate utilization at given load.
///
/// Alias for saturation_fraction with semantic naming.
pub fn utilization_at_load(load: f64, half_capacity: f64) -> Result<f64, SaturationError> {
    saturation_fraction(load, half_capacity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_michaelis_menten() {
        // v = (100 * 10) / (10 + 10) = 50
        let rate = michaelis_menten_rate(10.0, 100.0, 10.0).unwrap();
        assert!((rate - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_saturation_at_km() {
        // At [S] = Km, saturation = 0.5
        let frac = saturation_fraction(10.0, 10.0).unwrap();
        assert!((frac - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_kinetics_struct() {
        let kinetics = SaturationKinetics::new(1000.0, 200.0).unwrap();
        let throughput = kinetics.throughput(500.0).unwrap();
        // v = (1000 * 500) / (200 + 500) = 500000/700 ≈ 714.3
        assert!(throughput > 710.0 && throughput < 720.0);
    }

    #[test]
    fn test_required_load() {
        let kinetics = SaturationKinetics::new(1000.0, 200.0).unwrap();
        let load = kinetics.required_load(500.0);
        assert!(load.is_some());
        // Verify: v = (1000 * load) / (200 + load) = 500
        // 1000 * load = 500 * (200 + load)
        // 1000 * load = 100000 + 500 * load
        // 500 * load = 100000 -> load = 200
        assert!((load.unwrap() - 200.0).abs() < 0.1);
    }

    #[test]
    fn test_required_load_exceeds_vmax() {
        let kinetics = SaturationKinetics::new(1000.0, 200.0).unwrap();
        assert!(kinetics.required_load(1000.0).is_none());
        assert!(kinetics.required_load(1001.0).is_none());
    }
}
