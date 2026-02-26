//! # Cooperative Binding (Hill Equation)
//!
//! **T1 Components**: signal × amplification × threshold × state × transition
//!
//! **Chemistry**: Y = I^nH / (K₀.₅^nH + I^nH)
//!
//! **Universal Pattern**: Multiple binding sites influence each other.
//! Positive cooperativity (nH > 1) amplifies response; negative (nH < 1) dampens.
//!
//! **PV Application**: Signal cascade amplification - multiple weak signals
//! combine to produce strong effect (or interfere to dampen).
//!
//! **Bond Application**: Multiple low-energy bonds cascade to trigger action.

use nexcore_error::Error;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors for cooperativity calculations.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum CooperativityError {
    /// Input must be non-negative.
    #[error("Input concentration must be non-negative")]
    NegativeInput,
    /// Half-saturation must be positive.
    #[error("Half-saturation constant (K₀.₅) must be positive")]
    InvalidHalfSaturation,
    /// Hill coefficient must be positive.
    #[error("Hill coefficient must be positive")]
    InvalidHillCoefficient,
}

/// Cooperative binding system configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CooperativeBinding {
    /// Half-saturation constant (K₀.₅)
    pub k_half: f64,
    /// Hill coefficient (nH)
    /// - nH = 1: No cooperativity (Michaelis-Menten)
    /// - nH > 1: Positive cooperativity (amplification)
    /// - nH < 1: Negative cooperativity (dampening)
    pub n_hill: f64,
}

/// Cooperativity classification.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CooperativityType {
    /// nH < 1: Signals interfere, dampening response
    Negative,
    /// nH = 1: No cooperativity (standard Michaelis-Menten)
    None,
    /// 1 < nH < 2: Mild amplification
    MildPositive,
    /// 2 ≤ nH < 4: Strong amplification
    StrongPositive,
    /// nH ≥ 4: Ultra-cooperative (switch-like)
    Ultrasensitive,
}

impl fmt::Display for CooperativityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negative => write!(f, "negative"),
            Self::None => write!(f, "non-cooperative"),
            Self::MildPositive => write!(f, "mild-positive"),
            Self::StrongPositive => write!(f, "strong-positive"),
            Self::Ultrasensitive => write!(f, "ultrasensitive"),
        }
    }
}

impl fmt::Display for CooperativeBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Hill(K\u{2080}\u{22C5}\u{2085}={:.1}, nH={:.2})",
            self.k_half, self.n_hill
        )
    }
}

impl CooperativeBinding {
    /// Create new cooperative binding model.
    pub fn new(k_half: f64, n_hill: f64) -> Result<Self, CooperativityError> {
        if k_half <= 0.0 {
            return Err(CooperativityError::InvalidHalfSaturation);
        }
        if n_hill <= 0.0 {
            return Err(CooperativityError::InvalidHillCoefficient);
        }
        Ok(Self { k_half, n_hill })
    }

    /// Calculate fractional response (0.0 - 1.0).
    pub fn response(&self, input: f64) -> Result<f64, CooperativityError> {
        if input < 0.0 {
            return Err(CooperativityError::NegativeInput);
        }
        Ok(hill_response(input, self.k_half, self.n_hill))
    }

    /// Classify cooperativity type.
    #[must_use]
    pub fn classify(&self) -> CooperativityType {
        classify_cooperativity(self.n_hill)
    }

    /// Calculate input needed for target response.
    ///
    /// Inverse Hill: I = K₀.₅ × (Y / (1 - Y))^(1/nH)
    pub fn input_for_response(&self, target: f64) -> Option<f64> {
        if target <= 0.0 || target >= 1.0 {
            return None;
        }
        let ratio = target / (1.0 - target);
        Some(self.k_half * ratio.powf(1.0 / self.n_hill))
    }

    /// Calculate effective amplification factor at midpoint.
    ///
    /// Measures steepness of transition relative to Michaelis-Menten.
    #[must_use]
    pub fn amplification_factor(&self) -> f64 {
        // Slope at midpoint is proportional to nH/4
        self.n_hill / 4.0
    }
}

/// Calculate Hill equation response.
///
/// Y = I^nH / (K₀.₅^nH + I^nH)
///
/// # Arguments
/// * `input` - Input concentration [I]
/// * `k_half` - Half-saturation constant (input at 50% response)
/// * `n_hill` - Hill coefficient (cooperativity factor)
///
/// # Returns
/// Fractional response (0.0 to 1.0)
#[must_use]
pub fn hill_response(input: f64, k_half: f64, n_hill: f64) -> f64 {
    if input <= 0.0 || n_hill <= 0.0 {
        return 0.0;
    }
    if k_half <= 0.0 {
        // lim(k→0⁺) of input^n / (k^n + input^n) = 1.0 for input > 0
        return 1.0;
    }
    let input_power = input.powf(n_hill);
    let k_power = k_half.powf(n_hill);
    input_power / (k_power + input_power)
}

/// Classify cooperativity based on Hill coefficient.
#[must_use]
pub fn classify_cooperativity(n_hill: f64) -> CooperativityType {
    if n_hill < 0.9 {
        CooperativityType::Negative
    } else if n_hill <= 1.1 {
        CooperativityType::None
    } else if n_hill < 2.0 {
        CooperativityType::MildPositive
    } else if n_hill < 4.0 {
        CooperativityType::StrongPositive
    } else {
        CooperativityType::Ultrasensitive
    }
}

/// Calculate effective Hill coefficient from dose-response data.
///
/// nH ≈ log(81) / log(EC90/EC10)
///
/// Where EC10 and EC90 are concentrations at 10% and 90% response.
pub fn infer_hill_coefficient(ec10: f64, ec90: f64) -> Result<f64, CooperativityError> {
    if ec10 <= 0.0 || ec90 <= 0.0 {
        return Err(CooperativityError::NegativeInput);
    }
    if ec90 <= ec10 {
        return Err(CooperativityError::InvalidHalfSaturation);
    }
    // nH = log(81) / log(EC90/EC10)
    Ok(81.0_f64.ln() / (ec90 / ec10).ln())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hill_response_at_k_half() {
        // At I = K₀.₅, response should be 0.5
        let response = hill_response(10.0, 10.0, 2.0);
        assert!((response - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_hill_no_cooperativity() {
        // nH = 1 should match Michaelis-Menten
        let hill = hill_response(10.0, 20.0, 1.0);
        // MM: 10 / (20 + 10) = 0.333...
        assert!((hill - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_hill_positive_cooperativity() {
        // nH > 1: steeper curve
        let response_nh1 = hill_response(15.0, 10.0, 1.0);
        let response_nh3 = hill_response(15.0, 10.0, 3.0);
        // Higher nH should give higher response above K₀.₅
        assert!(response_nh3 > response_nh1);
    }

    #[test]
    fn test_hill_negative_cooperativity() {
        // nH < 1: shallower curve
        let response_nh1 = hill_response(15.0, 10.0, 1.0);
        let response_nh05 = hill_response(15.0, 10.0, 0.5);
        // Lower nH should give lower response above K₀.₅
        assert!(response_nh05 < response_nh1);
    }

    #[test]
    fn test_cooperative_binding_struct() {
        let binding = CooperativeBinding::new(10.0, 2.5).unwrap();
        let response = binding.response(10.0).unwrap();
        assert!((response - 0.5).abs() < 0.001);
        assert_eq!(binding.classify(), CooperativityType::StrongPositive);
    }

    #[test]
    fn test_input_for_response() {
        let binding = CooperativeBinding::new(10.0, 2.0).unwrap();
        let input = binding.input_for_response(0.5).unwrap();
        assert!((input - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_classify_cooperativity() {
        assert_eq!(classify_cooperativity(0.5), CooperativityType::Negative);
        assert_eq!(classify_cooperativity(1.0), CooperativityType::None);
        assert_eq!(classify_cooperativity(1.5), CooperativityType::MildPositive);
        assert_eq!(
            classify_cooperativity(3.0),
            CooperativityType::StrongPositive
        );
        assert_eq!(
            classify_cooperativity(5.0),
            CooperativityType::Ultrasensitive
        );
    }

    #[test]
    fn test_infer_hill_coefficient() {
        // For nH = 1: EC90/EC10 = 81 -> log(81)/log(81) = 1
        let nh = infer_hill_coefficient(1.0, 81.0).unwrap();
        assert!((nh - 1.0).abs() < 0.01);
    }
}
