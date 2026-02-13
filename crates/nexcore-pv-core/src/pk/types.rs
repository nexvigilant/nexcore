//! # PK Type System (Cartouche + Horus Patterns)
//!
//! Validated newtypes for pharmacokinetic parameters.
//! All domain constraints are enforced at construction (Horus: parse-don't-validate).
//! Functions consuming these types become infallible where possible.
//!
//! ## Tier Classification
//!
//! | Type | Tier | Grounding |
//! |------|------|-----------|
//! | Scalar newtypes | T2-P | Cartouche (∂ Boundary) |
//! | TimeConcProfile | T2-C | Horus (∂+σ Boundary+Sequence) |
//! | PkError | T3 | Domain error envelope |

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============================================================================
// Unified PK Error
// ============================================================================

/// Unified error type for all PK calculations.
///
/// Replaces the per-module error types (AucError, MassBalanceError,
/// SaturationError, SteadyStateError) with a single boundary error.
#[derive(Debug, Error, PartialEq)]
pub enum PkError {
    /// Value must be non-negative (>= 0).
    #[error("Value must be non-negative (got {value} for {param})")]
    Negative {
        /// Parameter name
        param: &'static str,
        /// The invalid value
        value: f64,
    },

    /// Value must be strictly positive (> 0).
    #[error("Value must be positive (got {value} for {param})")]
    NotPositive {
        /// Parameter name
        param: &'static str,
        /// The invalid value
        value: f64,
    },

    /// Value must be finite.
    #[error("Value must be finite for {param}")]
    NotFinite {
        /// Parameter name
        param: &'static str,
    },

    /// Value must be within a bounded range.
    #[error("Value must be in [{min}, {max}] (got {value} for {param})")]
    OutOfRange {
        /// Parameter name
        param: &'static str,
        /// The invalid value
        value: f64,
        /// Minimum allowed
        min: f64,
        /// Maximum allowed
        max: f64,
    },

    /// Time-concentration profile requires at least 2 points.
    #[error("Profile must have >= 2 points (got {len})")]
    InsufficientPoints {
        /// Actual number of points
        len: usize,
    },

    /// Time points must be strictly ascending.
    #[error("Time points must be strictly ascending")]
    TimeNotAscending,

    /// Concentration values must be non-negative.
    #[error("Concentration values must be non-negative")]
    NegativeConcentration,

    /// Mass conservation violation: eliminated exceeds dose.
    #[error("Mass violation: eliminated ({eliminated}) exceeds dose ({dose})")]
    MassViolation {
        /// Amount eliminated
        eliminated: f64,
        /// Initial dose
        dose: f64,
    },
}

// Deprecated type aliases for backward compatibility
/// Deprecated: use [`PkError`] instead.
pub type AucError = PkError;
/// Deprecated: use [`PkError`] instead.
pub type MassBalanceError = PkError;
/// Deprecated: use [`PkError`] instead.
pub type SteadyStateError = PkError;

// ============================================================================
// Newtype Macro
// ============================================================================

/// Generate a validated PK newtype over f64.
///
/// Variants:
/// - `non_negative`: value >= 0.0, finite
/// - `positive`: value > 0.0, finite
/// - `range(min, max)`: min <= value <= max, finite
/// - `min_bound(min)`: value >= min, finite
macro_rules! pk_newtype {
    // Non-negative: >= 0.0, finite
    ($name:ident, non_negative, $param:literal, $doc:literal) => {
        #[doc = $doc]
        ///
        /// Tier: T2-P (Cartouche newtype, ∂ Boundary)
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
        pub struct $name(pub(crate) f64);

        impl $name {
            /// Create a new validated value.
            ///
            /// # Errors
            /// Returns `PkError` if value is negative or non-finite.
            pub fn new(value: f64) -> Result<Self, PkError> {
                if !value.is_finite() {
                    return Err(PkError::NotFinite { param: $param });
                }
                if value < 0.0 {
                    return Err(PkError::Negative {
                        param: $param,
                        value,
                    });
                }
                Ok(Self(value))
            }

            /// Extract the inner value.
            #[must_use]
            pub const fn value(self) -> f64 {
                self.0
            }
        }

        impl From<$name> for f64 {
            fn from(v: $name) -> f64 {
                v.0
            }
        }
    };

    // Positive: > 0.0, finite
    ($name:ident, positive, $param:literal, $doc:literal) => {
        #[doc = $doc]
        ///
        /// Tier: T2-P (Cartouche newtype, ∂ Boundary)
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
        pub struct $name(pub(crate) f64);

        impl $name {
            /// Create a new validated value.
            ///
            /// # Errors
            /// Returns `PkError` if value is not positive or non-finite.
            pub fn new(value: f64) -> Result<Self, PkError> {
                if !value.is_finite() {
                    return Err(PkError::NotFinite { param: $param });
                }
                if value <= 0.0 {
                    return Err(PkError::NotPositive {
                        param: $param,
                        value,
                    });
                }
                Ok(Self(value))
            }

            /// Extract the inner value.
            #[must_use]
            pub const fn value(self) -> f64 {
                self.0
            }
        }

        impl From<$name> for f64 {
            fn from(v: $name) -> f64 {
                v.0
            }
        }
    };

    // Range: [min, max], finite
    ($name:ident, range($min:expr, $max:expr), $param:literal, $doc:literal) => {
        #[doc = $doc]
        ///
        /// Tier: T2-P (Cartouche newtype, ∂ Boundary)
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
        pub struct $name(pub(crate) f64);

        impl $name {
            /// Create a new validated value.
            ///
            /// # Errors
            /// Returns `PkError` if value is outside the valid range or non-finite.
            pub fn new(value: f64) -> Result<Self, PkError> {
                if !value.is_finite() {
                    return Err(PkError::NotFinite { param: $param });
                }
                if !($min..=$max).contains(&value) {
                    return Err(PkError::OutOfRange {
                        param: $param,
                        value,
                        min: $min,
                        max: $max,
                    });
                }
                Ok(Self(value))
            }

            /// Extract the inner value.
            #[must_use]
            pub const fn value(self) -> f64 {
                self.0
            }
        }

        impl From<$name> for f64 {
            fn from(v: $name) -> f64 {
                v.0
            }
        }
    };

    // Min-bound: >= min, finite
    ($name:ident, min_bound($min:expr), $param:literal, $doc:literal) => {
        #[doc = $doc]
        ///
        /// Tier: T2-P (Cartouche newtype, ∂ Boundary)
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
        pub struct $name(pub(crate) f64);

        impl $name {
            /// Create a new validated value.
            ///
            /// # Errors
            /// Returns `PkError` if value is below minimum or non-finite.
            pub fn new(value: f64) -> Result<Self, PkError> {
                if !value.is_finite() {
                    return Err(PkError::NotFinite { param: $param });
                }
                if value < $min {
                    return Err(PkError::OutOfRange {
                        param: $param,
                        value,
                        min: $min,
                        max: f64::INFINITY,
                    });
                }
                Ok(Self(value))
            }

            /// Extract the inner value.
            #[must_use]
            pub const fn value(self) -> f64 {
                self.0
            }
        }

        impl From<$name> for f64 {
            fn from(v: $name) -> f64 {
                v.0
            }
        }
    };
}

// ============================================================================
// Scalar Newtypes (12)
// ============================================================================

pk_newtype!(
    Dose,
    non_negative,
    "dose",
    "Administered drug dose (mass units)."
);
pk_newtype!(
    Concentration,
    non_negative,
    "concentration",
    "Drug concentration (mass/volume)."
);
pk_newtype!(
    Clearance,
    positive,
    "clearance",
    "Systemic clearance (volume/time)."
);
pk_newtype!(
    Volume,
    positive,
    "volume_of_distribution",
    "Volume of distribution (volume units)."
);
pk_newtype!(
    Bioavailability,
    range(0.0, 1.0),
    "bioavailability",
    "Fraction of dose reaching systemic circulation (0.0-1.0)."
);
pk_newtype!(
    HalfLife,
    positive,
    "half_life",
    "Elimination half-life (time units)."
);
pk_newtype!(
    DosingInterval,
    positive,
    "dosing_interval",
    "Time between doses (time units)."
);
pk_newtype!(
    Auc,
    positive,
    "auc",
    "Area under the concentration-time curve (concentration*time)."
);
pk_newtype!(
    Vmax,
    non_negative,
    "vmax",
    "Maximum reaction velocity (mass/time)."
);
pk_newtype!(
    Km,
    positive,
    "km",
    "Michaelis constant / half-saturation concentration."
);
pk_newtype!(
    Kd,
    positive,
    "kd",
    "Dissociation constant for receptor binding."
);
pk_newtype!(
    AccumulationFactor,
    min_bound(1.0),
    "accumulation_factor",
    "Ratio of steady-state to single-dose concentration (>= 1.0)."
);

// ============================================================================
// Composite: TimeConcProfile (T2-C)
// ============================================================================

/// T2-C: Validated time-concentration profile (Horus ∂+σ Boundary+Sequence).
///
/// Invariants enforced at construction:
/// - At least 2 data points
/// - All values are finite
/// - Time points are strictly ascending
/// - Concentration values are non-negative
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeConcProfile {
    /// Validated (time, concentration) pairs, sorted by time ascending.
    points: Vec<(f64, f64)>,
}

impl TimeConcProfile {
    /// Create from a vector of (time, concentration) pairs.
    ///
    /// # Errors
    /// Returns `PkError` if any invariant is violated.
    pub fn new(points: Vec<(f64, f64)>) -> Result<Self, PkError> {
        if points.len() < 2 {
            return Err(PkError::InsufficientPoints { len: points.len() });
        }

        for &(t, c) in &points {
            if !t.is_finite() {
                return Err(PkError::NotFinite { param: "time" });
            }
            if !c.is_finite() {
                return Err(PkError::NotFinite {
                    param: "concentration",
                });
            }
            if c < 0.0 {
                return Err(PkError::NegativeConcentration);
            }
        }

        // Check strictly ascending times
        for i in 1..points.len() {
            if points[i].0 <= points[i - 1].0 {
                return Err(PkError::TimeNotAscending);
            }
        }

        Ok(Self { points })
    }

    /// Create from parallel time and concentration slices (backward compat).
    ///
    /// # Errors
    /// Returns `PkError` if slices differ in length or invariants are violated.
    pub fn from_parallel_slices(times: &[f64], concentrations: &[f64]) -> Result<Self, PkError> {
        if times.len() != concentrations.len() {
            return Err(PkError::InsufficientPoints {
                len: times.len().min(concentrations.len()),
            });
        }

        let points: Vec<(f64, f64)> = times
            .iter()
            .copied()
            .zip(concentrations.iter().copied())
            .collect();
        Self::new(points)
    }

    /// Access the validated data points.
    #[must_use]
    pub fn points(&self) -> &[(f64, f64)] {
        &self.points
    }

    /// Number of data points.
    #[must_use]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Profile is never empty (min 2 points), but satisfy clippy.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Scalar newtype construction ---

    #[test]
    fn test_dose_valid() {
        assert!(Dose::new(0.0).is_ok());
        assert!(Dose::new(500.0).is_ok());
    }

    #[test]
    fn test_dose_negative() {
        let err = Dose::new(-1.0).unwrap_err();
        assert!(matches!(err, PkError::Negative { param: "dose", .. }));
    }

    #[test]
    fn test_dose_infinite() {
        let err = Dose::new(f64::INFINITY).unwrap_err();
        assert!(matches!(err, PkError::NotFinite { param: "dose" }));
    }

    #[test]
    fn test_dose_nan() {
        let err = Dose::new(f64::NAN).unwrap_err();
        assert!(matches!(err, PkError::NotFinite { param: "dose" }));
    }

    #[test]
    fn test_clearance_positive() {
        assert!(Clearance::new(5.0).is_ok());
        let err = Clearance::new(0.0).unwrap_err();
        assert!(matches!(err, PkError::NotPositive { .. }));
        let err = Clearance::new(-1.0).unwrap_err();
        assert!(matches!(err, PkError::NotPositive { .. }));
    }

    #[test]
    fn test_bioavailability_range() {
        assert!(Bioavailability::new(0.0).is_ok());
        assert!(Bioavailability::new(0.5).is_ok());
        assert!(Bioavailability::new(1.0).is_ok());
        let err = Bioavailability::new(1.5).unwrap_err();
        assert!(matches!(err, PkError::OutOfRange { .. }));
        let err = Bioavailability::new(-0.1).unwrap_err();
        assert!(matches!(err, PkError::OutOfRange { .. }));
    }

    #[test]
    fn test_accumulation_factor_min() {
        assert!(AccumulationFactor::new(1.0).is_ok());
        assert!(AccumulationFactor::new(2.0).is_ok());
        let err = AccumulationFactor::new(0.5).unwrap_err();
        assert!(matches!(err, PkError::OutOfRange { .. }));
    }

    #[test]
    fn test_value_extraction() {
        let d = Dose::new(42.0).unwrap_or_else(|_| Dose(0.0));
        assert!((d.value() - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_f64_from_dose() {
        let d = Dose::new(99.0).unwrap_or_else(|_| Dose(0.0));
        let raw: f64 = d.into();
        assert!((raw - 99.0).abs() < f64::EPSILON);
    }

    // --- TimeConcProfile ---

    #[test]
    fn test_profile_valid() {
        let profile = TimeConcProfile::new(vec![(0.0, 0.0), (1.0, 10.0), (2.0, 5.0)]);
        assert!(profile.is_ok());
        let p = profile.unwrap_or_else(|_| TimeConcProfile {
            points: vec![(0.0, 0.0), (1.0, 0.0)],
        });
        assert_eq!(p.len(), 3);
    }

    #[test]
    fn test_profile_too_few_points() {
        let err = TimeConcProfile::new(vec![(0.0, 0.0)]).unwrap_err();
        assert!(matches!(err, PkError::InsufficientPoints { len: 1 }));
    }

    #[test]
    fn test_profile_time_not_ascending() {
        let err = TimeConcProfile::new(vec![(0.0, 0.0), (2.0, 10.0), (1.0, 5.0)]).unwrap_err();
        assert!(matches!(err, PkError::TimeNotAscending));
    }

    #[test]
    fn test_profile_negative_concentration() {
        let err = TimeConcProfile::new(vec![(0.0, 0.0), (1.0, -5.0)]).unwrap_err();
        assert!(matches!(err, PkError::NegativeConcentration));
    }

    #[test]
    fn test_profile_from_parallel_slices() {
        let times = [0.0, 1.0, 2.0];
        let concs = [0.0, 10.0, 5.0];
        let profile = TimeConcProfile::from_parallel_slices(&times, &concs);
        assert!(profile.is_ok());
    }

    #[test]
    fn test_profile_mismatched_slices() {
        let times = [0.0, 1.0];
        let concs = [0.0];
        let err = TimeConcProfile::from_parallel_slices(&times, &concs).unwrap_err();
        assert!(matches!(err, PkError::InsufficientPoints { .. }));
    }

    #[test]
    fn test_profile_duplicate_times() {
        let err = TimeConcProfile::new(vec![(0.0, 0.0), (1.0, 10.0), (1.0, 5.0)]).unwrap_err();
        assert!(matches!(err, PkError::TimeNotAscending));
    }

    #[test]
    fn test_profile_non_finite() {
        let err = TimeConcProfile::new(vec![(0.0, 0.0), (f64::NAN, 10.0)]).unwrap_err();
        assert!(matches!(err, PkError::NotFinite { .. }));
    }

    // --- serde round-trip ---

    #[test]
    fn test_dose_serde() {
        let d = Dose::new(42.0).unwrap_or_else(|_| Dose(0.0));
        let json = serde_json::to_string(&d).unwrap_or_default();
        let d2: Result<Dose, _> = serde_json::from_str(&json);
        assert!(d2.is_ok());
    }
}
