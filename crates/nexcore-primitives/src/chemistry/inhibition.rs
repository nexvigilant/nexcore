//! # Competitive Inhibition
//!
//! **T1 Components**: signal × interference × threshold × ratio × dependency
//!
//! **Chemistry**: v = Vmax[S] / (Km(1 + [I]/Ki) + [S])
//!
//! **Universal Pattern**: Competing signals reduce effective sensitivity.
//! More interference = higher apparent threshold.
//!
//! **PV Application**: Signal interference patterns - background noise
//! competes with true signals, raising detection threshold.
//!
//! **Bond Application**: Too many pending bonds interfere with each other,
//! reducing effective throughput (Ki = interference constant).

use nexcore_error::Error;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors for inhibition calculations.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum InhibitionError {
    /// Parameters must be non-negative.
    #[error("All parameters must be non-negative")]
    NegativeParameter,
    /// Inhibition constant must be positive.
    #[error("Inhibition constant (Ki) must be positive")]
    InvalidKi,
    /// Half-saturation constant must be positive.
    #[error("Half-saturation constant (Km) must be positive")]
    InvalidKm,
}

/// Competitive inhibition system configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompetitiveInhibition {
    /// Maximum rate (Vmax)
    pub v_max: f64,
    /// Half-saturation constant (Km)
    pub k_m: f64,
    /// Inhibition constant (Ki) - affinity of inhibitor
    pub k_i: f64,
}

/// Inhibition strength classification.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InhibitionStrength {
    /// [I]/Ki < 0.1: Negligible interference
    Negligible,
    /// 0.1 ≤ [I]/Ki < 1: Mild interference
    Mild,
    /// 1 ≤ [I]/Ki < 5: Moderate interference
    Moderate,
    /// 5 ≤ [I]/Ki < 10: Strong interference
    Strong,
    /// [I]/Ki ≥ 10: Severe interference (system nearly blocked)
    Severe,
}

impl fmt::Display for InhibitionStrength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negligible => write!(f, "negligible"),
            Self::Mild => write!(f, "mild"),
            Self::Moderate => write!(f, "moderate"),
            Self::Strong => write!(f, "strong"),
            Self::Severe => write!(f, "severe"),
        }
    }
}

impl fmt::Display for CompetitiveInhibition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Inhib(Ki={:.1}, Km={:.1})", self.k_i, self.k_m)
    }
}

impl CompetitiveInhibition {
    /// Create new competitive inhibition model.
    pub fn new(v_max: f64, k_m: f64, k_i: f64) -> Result<Self, InhibitionError> {
        if v_max < 0.0 {
            return Err(InhibitionError::NegativeParameter);
        }
        if k_m <= 0.0 {
            return Err(InhibitionError::InvalidKm);
        }
        if k_i <= 0.0 {
            return Err(InhibitionError::InvalidKi);
        }
        Ok(Self { v_max, k_m, k_i })
    }

    /// Calculate rate with inhibitor present.
    pub fn rate(&self, substrate: f64, inhibitor: f64) -> Result<f64, InhibitionError> {
        if substrate < 0.0 || inhibitor < 0.0 {
            return Err(InhibitionError::NegativeParameter);
        }
        Ok(inhibited_rate(
            substrate, self.v_max, self.k_m, inhibitor, self.k_i,
        ))
    }

    /// Calculate apparent Km with inhibitor.
    ///
    /// Km_app = Km × (1 + [I]/Ki)
    #[must_use]
    pub fn apparent_km(&self, inhibitor: f64) -> f64 {
        self.k_m * (1.0 + inhibitor / self.k_i)
    }

    /// Calculate fractional inhibition (0.0 to 1.0).
    ///
    /// At saturating substrate, inhibition is minimal.
    /// This measures inhibition at low substrate.
    #[must_use]
    pub fn fractional_inhibition(&self, inhibitor: f64) -> f64 {
        // i = [I]/Ki / (1 + [I]/Ki)
        let ratio = inhibitor / self.k_i;
        ratio / (1.0 + ratio)
    }

    /// Classify inhibition strength.
    #[must_use]
    pub fn classify(&self, inhibitor: f64) -> InhibitionStrength {
        classify_inhibition(inhibitor, self.k_i)
    }

    /// Calculate inhibitor concentration for 50% inhibition (IC50).
    ///
    /// For competitive inhibition at [S] << Km: IC50 ≈ Ki
    /// At higher [S]: IC50 = Ki × (1 + [S]/Km)
    #[must_use]
    pub fn ic50(&self, substrate: f64) -> f64 {
        self.k_i * (1.0 + substrate / self.k_m)
    }
}

/// Calculate rate with competitive inhibition.
///
/// v = Vmax[S] / (Km(1 + [I]/Ki) + [S])
///
/// # Arguments
/// * `substrate` - Substrate concentration [S]
/// * `v_max` - Maximum rate
/// * `k_m` - Half-saturation constant
/// * `inhibitor` - Inhibitor concentration [I]
/// * `k_i` - Inhibition constant
///
/// # Returns
/// Reaction rate with inhibition
#[must_use]
pub fn inhibited_rate(substrate: f64, v_max: f64, k_m: f64, inhibitor: f64, k_i: f64) -> f64 {
    if substrate < 0.0 || v_max < 0.0 || k_m <= 0.0 || inhibitor < 0.0 || k_i <= 0.0 {
        return 0.0;
    }
    let apparent_km = k_m * (1.0 + inhibitor / k_i);
    (v_max * substrate) / (apparent_km + substrate)
}

/// Calculate apparent Km with inhibitor present.
///
/// Km_app = Km × (1 + [I]/Ki)
#[must_use]
pub fn apparent_km(k_m: f64, inhibitor: f64, k_i: f64) -> f64 {
    if k_i <= 0.0 {
        return k_m;
    }
    k_m * (1.0 + inhibitor / k_i)
}

/// Calculate inhibitor concentration needed for target fractional inhibition.
///
/// [I] = Ki × i / (1 - i)
/// where i is fractional inhibition (0.0 to 1.0)
pub fn inhibitor_for_fractional(k_i: f64, fraction: f64) -> Option<f64> {
    if fraction <= 0.0 || fraction >= 1.0 || k_i <= 0.0 {
        return None;
    }
    Some(k_i * fraction / (1.0 - fraction))
}

/// Classify inhibition strength based on [I]/Ki ratio.
#[must_use]
pub fn classify_inhibition(inhibitor: f64, k_i: f64) -> InhibitionStrength {
    if k_i <= 0.0 {
        return InhibitionStrength::Severe;
    }
    let ratio = inhibitor / k_i;
    if ratio < 0.1 {
        InhibitionStrength::Negligible
    } else if ratio < 1.0 {
        InhibitionStrength::Mild
    } else if ratio < 5.0 {
        InhibitionStrength::Moderate
    } else if ratio < 10.0 {
        InhibitionStrength::Strong
    } else {
        InhibitionStrength::Severe
    }
}

/// Calculate throughput reduction factor.
///
/// Returns fraction of uninhibited rate (0.0 to 1.0).
/// At [S] >> Km, this approaches 1.0 (competitive inhibition overcome).
#[must_use]
pub fn throughput_reduction(substrate: f64, k_m: f64, inhibitor: f64, k_i: f64) -> f64 {
    if k_m <= 0.0 || k_i <= 0.0 {
        return 0.0;
    }
    let uninhibited_denom = k_m + substrate;
    let inhibited_denom = k_m * (1.0 + inhibitor / k_i) + substrate;
    if inhibited_denom <= 0.0 {
        return 0.0;
    }
    uninhibited_denom / inhibited_denom
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inhibited_rate_no_inhibitor() {
        // Without inhibitor, should match standard MM
        let rate = inhibited_rate(10.0, 100.0, 10.0, 0.0, 5.0);
        // v = (100 * 10) / (10 + 10) = 50
        assert!((rate - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_inhibited_rate_with_inhibitor() {
        // With inhibitor [I] = Ki, apparent Km doubles
        let rate = inhibited_rate(10.0, 100.0, 10.0, 5.0, 5.0);
        // Km_app = 10 * (1 + 5/5) = 20
        // v = (100 * 10) / (20 + 10) = 33.3
        assert!((rate - 33.33).abs() < 0.1);
    }

    #[test]
    fn test_apparent_km() {
        let km_app = apparent_km(10.0, 5.0, 5.0);
        // Km_app = 10 * (1 + 1) = 20
        assert!((km_app - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_fractional_inhibition() {
        let inhib = CompetitiveInhibition::new(100.0, 10.0, 5.0).unwrap();
        // At [I] = Ki, fractional = 0.5
        let frac = inhib.fractional_inhibition(5.0);
        assert!((frac - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_classify_inhibition() {
        assert_eq!(
            classify_inhibition(0.05, 1.0),
            InhibitionStrength::Negligible
        );
        assert_eq!(classify_inhibition(0.5, 1.0), InhibitionStrength::Mild);
        assert_eq!(classify_inhibition(2.0, 1.0), InhibitionStrength::Moderate);
        assert_eq!(classify_inhibition(7.0, 1.0), InhibitionStrength::Strong);
        assert_eq!(classify_inhibition(15.0, 1.0), InhibitionStrength::Severe);
    }

    #[test]
    fn test_inhibitor_for_fractional() {
        // For 50% inhibition at [I] = Ki
        let conc = inhibitor_for_fractional(5.0, 0.5).unwrap();
        assert!((conc - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_ic50() {
        let inhib = CompetitiveInhibition::new(100.0, 10.0, 5.0).unwrap();
        // At [S] = 0: IC50 ≈ Ki = 5
        let ic50_low = inhib.ic50(0.0);
        assert!((ic50_low - 5.0).abs() < 0.001);
        // At [S] = Km: IC50 = Ki * 2 = 10
        let ic50_mid = inhib.ic50(10.0);
        assert!((ic50_mid - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_throughput_reduction() {
        // With [I] = Ki, apparent Km doubles
        let reduction = throughput_reduction(10.0, 10.0, 5.0, 5.0);
        // uninhibited: 10+10=20, inhibited: 20+10=30
        // reduction = 20/30 = 0.667
        assert!((reduction - 0.667).abs() < 0.01);
    }
}
