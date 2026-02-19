//! # Decay Kinetics (Half-Life)
//!
//! **T1 Components**: duration × quantity × ratio × frequency
//!
//! **Chemistry**: N(t) = N₀ × e^(-kt), t½ = ln(2)/k
//!
//! **Universal Pattern**: Quantity decreases exponentially over time.
//! Half-life is time for 50% reduction.
//!
//! **PV Application**: Signal persistence after intervention.

use thiserror::Error;

/// Errors for decay calculations.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum DecayError {
    /// Decay constant must be positive.
    #[error("Decay constant must be positive")]
    DecayConstantNotPositive,
    /// Half-life must be positive.
    #[error("Half-life must be positive")]
    HalfLifeNotPositive,
    /// Time must be non-negative.
    #[error("Time must be non-negative")]
    NegativeTime,
    /// Fraction must be in (0, 1].
    #[error("Fraction must be in range (0, 1]")]
    InvalidFraction,
}

/// Natural log of 2 (used in half-life calculations).
pub const LN_2: f64 = 0.693_147_180_559_945_3;

/// Decay kinetics configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct DecayKinetics {
    /// Half-life (time for 50% decay)
    pub half_life: f64,
}

impl DecayKinetics {
    /// Create new decay kinetics from half-life.
    pub fn from_half_life(half_life: f64) -> Result<Self, DecayError> {
        if half_life <= 0.0 {
            return Err(DecayError::HalfLifeNotPositive);
        }
        Ok(Self { half_life })
    }

    /// Create new decay kinetics from decay constant.
    pub fn from_decay_constant(k: f64) -> Result<Self, DecayError> {
        if k <= 0.0 {
            return Err(DecayError::DecayConstantNotPositive);
        }
        Ok(Self {
            half_life: LN_2 / k,
        })
    }

    /// Get decay constant k.
    pub fn decay_constant(&self) -> f64 {
        LN_2 / self.half_life
    }

    /// Calculate remaining amount after time t.
    pub fn remaining(&self, initial: f64, time: f64) -> Result<f64, DecayError> {
        if time < 0.0 {
            return Err(DecayError::NegativeTime);
        }
        let k = self.decay_constant();
        Ok(initial * (-k * time).exp())
    }

    /// Calculate time to reach a fraction of initial.
    pub fn time_to_fraction(&self, fraction: f64) -> Result<f64, DecayError> {
        if fraction <= 0.0 || fraction > 1.0 {
            return Err(DecayError::InvalidFraction);
        }
        let k = self.decay_constant();
        Ok(-fraction.ln() / k)
    }

    /// Time for 90% decay (10% remaining).
    pub fn time_to_10_percent(&self) -> f64 {
        // t = -ln(0.1) / k = ln(10) / k ≈ 3.32 half-lives
        self.half_life * 3.321_928_094_887_362
    }

    /// Time for 99% decay (1% remaining).
    pub fn time_to_1_percent(&self) -> f64 {
        // t = -ln(0.01) / k = ln(100) / k ≈ 6.64 half-lives
        self.half_life * 6.643_856_189_774_724
    }
}

/// Calculate first-order decay.
///
/// N(t) = N₀ × e^(-kt)
pub fn first_order_decay(initial: f64, decay_constant: f64, time: f64) -> Result<f64, DecayError> {
    if decay_constant <= 0.0 {
        return Err(DecayError::DecayConstantNotPositive);
    }
    if time < 0.0 {
        return Err(DecayError::NegativeTime);
    }
    Ok(initial * (-decay_constant * time).exp())
}

/// Calculate remaining amount after time.
///
/// Convenience function using half-life directly.
pub fn remaining_after_time(initial: f64, half_life: f64, time: f64) -> Result<f64, DecayError> {
    if half_life <= 0.0 {
        return Err(DecayError::HalfLifeNotPositive);
    }
    if time < 0.0 {
        return Err(DecayError::NegativeTime);
    }
    let k = LN_2 / half_life;
    Ok(initial * (-k * time).exp())
}

/// Calculate half-life from decay constant.
pub fn half_life_from_decay_constant(k: f64) -> Result<f64, DecayError> {
    if k <= 0.0 {
        return Err(DecayError::DecayConstantNotPositive);
    }
    Ok(LN_2 / k)
}

/// Calculate decay constant from half-life.
pub fn decay_constant_from_half_life(half_life: f64) -> Result<f64, DecayError> {
    if half_life <= 0.0 {
        return Err(DecayError::HalfLifeNotPositive);
    }
    Ok(LN_2 / half_life)
}

/// Calculate time to reach target fraction.
pub fn time_to_fraction(half_life: f64, fraction: f64) -> Result<f64, DecayError> {
    if half_life <= 0.0 {
        return Err(DecayError::HalfLifeNotPositive);
    }
    if fraction <= 0.0 || fraction > 1.0 {
        return Err(DecayError::InvalidFraction);
    }
    let k = LN_2 / half_life;
    Ok(-fraction.ln() / k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_half_life_decay() {
        // After 1 half-life, 50% remains
        let remaining = remaining_after_time(100.0, 30.0, 30.0).unwrap();
        assert!((remaining - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_two_half_lives() {
        // After 2 half-lives, 25% remains
        let remaining = remaining_after_time(100.0, 30.0, 60.0).unwrap();
        assert!((remaining - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_decay_constant_roundtrip() {
        let half_life = 30.0;
        let k = decay_constant_from_half_life(half_life).unwrap();
        let recovered = half_life_from_decay_constant(k).unwrap();
        assert!((recovered - half_life).abs() < 0.001);
    }

    #[test]
    fn test_time_to_fraction() {
        // Time to 50% = 1 half-life
        let time = time_to_fraction(30.0, 0.5).unwrap();
        assert!((time - 30.0).abs() < 0.1);
    }

    #[test]
    fn test_decay_kinetics_struct() {
        let kinetics = DecayKinetics::from_half_life(30.0).unwrap();
        let remaining = kinetics.remaining(100.0, 90.0).unwrap();
        // After 3 half-lives, 12.5% remains
        assert!((remaining - 12.5).abs() < 0.5);
    }

    #[test]
    fn test_time_to_10_percent() {
        let kinetics = DecayKinetics::from_half_life(30.0).unwrap();
        let time = kinetics.time_to_10_percent();
        // Should be ~3.32 half-lives = ~99.7
        assert!(time > 99.0 && time < 100.0);
    }
}
