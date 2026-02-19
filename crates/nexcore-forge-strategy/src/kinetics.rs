//! Kinetics module — Arrhenius rates and Hill cooperativity for game decisions.
//!
//! Merged from ferro-forge-engine. T1 Primitives: N(Quantity) + ∂(Boundary) + →(Causality) + ν(Frequency)

use serde::{Deserialize, Serialize};

/// Arrhenius rate constant: k = A × e^(-Ea/RT)
///
/// Maps to signal detection threshold (0.92 confidence).
/// Used as the decision gate — should we proceed with an action?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrheniusGate {
    /// Pre-exponential factor (base sensitivity)
    pub sensitivity: f64,
    /// Activation energy in kJ/mol (threshold barrier)
    pub barrier: f64,
    /// Temperature (scaling factor, higher = more permissive)
    pub temperature: f64,
    /// Computed rate constant
    pub rate: f64,
}

impl ArrheniusGate {
    const GAS_CONSTANT: f64 = 8.314e-3; // kJ/(mol·K)

    /// Compute the Arrhenius rate for a given barrier and temperature.
    pub fn compute(sensitivity: f64, barrier: f64, temperature: f64) -> Self {
        let rate = if temperature > 0.0 {
            sensitivity * (-barrier / (Self::GAS_CONSTANT * temperature)).exp()
        } else {
            0.0
        };
        Self {
            sensitivity,
            barrier,
            temperature,
            rate,
        }
    }

    /// Should we proceed? Rate must exceed the abandon threshold.
    pub fn should_proceed(&self, abandon_threshold: f64) -> bool {
        self.rate > abandon_threshold
    }

    /// Interpret the rate as a game action.
    pub fn interpretation(&self) -> &'static str {
        if self.rate > 0.5 {
            "Fast — low barrier, proceed aggressively"
        } else if self.rate > 0.1 {
            "Moderate — proceed with caution"
        } else if self.rate > 0.027 {
            "Slow — near abandon threshold, consider alternatives"
        } else {
            "Blocked — below abandon threshold, retreat"
        }
    }
}

/// Hill equation cooperativity: Y = I^nH / (K^nH + I^nH)
///
/// Multiple weak signals amplify each other above K₀.₅.
/// Used for the "ship decision" — do accumulated quality signals trigger release?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HillCascade {
    /// Hill coefficient (cooperativity)
    pub n_hill: f64,
    /// Half-saturation constant
    pub k_half: f64,
    /// Accumulated signal strength
    pub total_signal: f64,
    /// Computed response (0.0 - 1.0)
    pub response: f64,
    /// Cooperativity classification
    pub cooperativity: CooperativityType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CooperativityType {
    Negative,
    Standard,
    MildPositive,
    StrongPositive,
    Ultrasensitive,
}

impl std::fmt::Display for CooperativityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Negative => write!(f, "Negative (dampening)"),
            Self::Standard => write!(f, "Standard (no cooperativity)"),
            Self::MildPositive => write!(f, "Mild positive"),
            Self::StrongPositive => write!(f, "Strong positive"),
            Self::Ultrasensitive => write!(f, "Ultrasensitive (switch)"),
        }
    }
}

impl HillCascade {
    /// Compute Hill response for accumulated signals.
    pub fn compute(n_hill: f64, k_half: f64, signals: &[f64]) -> Self {
        let total_signal: f64 = signals.iter().sum();
        let response = if total_signal <= 0.0 || k_half <= 0.0 {
            0.0
        } else {
            let i_pow = total_signal.powf(n_hill);
            let k_pow = k_half.powf(n_hill);
            i_pow / (k_pow + i_pow)
        };

        let cooperativity = if n_hill < 0.9 {
            CooperativityType::Negative
        } else if n_hill <= 1.1 {
            CooperativityType::Standard
        } else if n_hill < 2.0 {
            CooperativityType::MildPositive
        } else if n_hill < 4.0 {
            CooperativityType::StrongPositive
        } else {
            CooperativityType::Ultrasensitive
        };

        Self {
            n_hill,
            k_half,
            total_signal,
            response,
            cooperativity,
        }
    }

    /// Should we ship? Response must meet or exceed threshold.
    pub fn should_ship(&self, threshold: f64) -> bool {
        self.response >= threshold
    }

    /// Amplification factor vs standard (nH=1).
    pub fn amplification(&self) -> f64 {
        if self.total_signal <= 0.0 || self.k_half <= 0.0 {
            return 1.0;
        }
        let standard = self.total_signal / (self.k_half + self.total_signal);
        if standard > 0.0 {
            self.response / standard
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrhenius_gate() {
        let gate = ArrheniusGate::compute(1.0, 8.314, 350.0);
        assert!(gate.rate > 0.0);
        assert!(gate.rate < 1.0);
        assert!(gate.should_proceed(0.027));
    }

    #[test]
    fn test_arrhenius_blocked() {
        let gate = ArrheniusGate::compute(1.0, 100.0, 300.0);
        assert!(gate.rate < 0.027);
        assert!(!gate.should_proceed(0.027));
        assert_eq!(
            gate.interpretation(),
            "Blocked — below abandon threshold, retreat"
        );
    }

    #[test]
    fn test_hill_at_half_saturation() {
        let cascade = HillCascade::compute(2.5, 1.0, &[0.5, 0.5]);
        assert!((cascade.response - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_hill_above_threshold() {
        let cascade = HillCascade::compute(2.5, 1.0, &[0.6, 0.6, 0.6]);
        assert!(cascade.response > 0.8);
        assert!(cascade.should_ship(0.8));
        assert_eq!(cascade.cooperativity, CooperativityType::StrongPositive);
    }

    #[test]
    fn test_hill_below_threshold() {
        let cascade = HillCascade::compute(2.5, 1.0, &[0.3]);
        assert!(cascade.response < 0.5);
        assert!(!cascade.should_ship(0.8));
    }

    #[test]
    fn test_amplification_above_k_half() {
        let cascade = HillCascade::compute(2.5, 1.0, &[0.6, 0.6, 0.6]);
        assert!(cascade.amplification() > 1.0);
    }
}
