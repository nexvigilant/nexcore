//! Newtype wrappers for signal detection metrics.
//!
//! These provide stronger type safety by preventing accidental mixing of
//! different disproportionality measures. Each type enforces:
//!
//! - Domain validation (non-negative for ratios, bounded ranges)
//! - Unit-specific operations (only valid comparisons)
//! - Explicit conversions to raw f64
//!
//! # Example
//!
//! ```
//! use nexcore_vigilance::pv::signals::newtypes::{Prr, Ror, Ic, Ebgm};
//!
//! let prr = Prr::new(3.5).unwrap();
//! let ror = Ror::new(4.2).unwrap();
//!
//! // Type system prevents: prr == ror (different types!)
//! // Must explicitly compare raw values if needed:
//! assert!(prr.value() < ror.value());
//! ```

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt;

/// Error for invalid metric values.
#[derive(Debug, Clone, PartialEq)]
pub enum MetricError {
    /// Value is negative (invalid for ratio-based metrics)
    Negative(f64),
    /// Value is NaN
    NaN,
    /// Value is infinite (unconstrained)
    Infinite,
}

impl fmt::Display for MetricError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negative(v) => write!(f, "Metric value cannot be negative: {v}"),
            Self::NaN => write!(f, "Metric value cannot be NaN"),
            Self::Infinite => write!(f, "Metric value cannot be infinite"),
        }
    }
}

impl std::error::Error for MetricError {}

/// Macro to generate newtype wrappers for ratio-based metrics.
macro_rules! ratio_newtype {
    (
        $(#[$meta:meta])*
        $name:ident, $threshold_name:ident, $default_threshold:expr
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy)]
        pub struct $name(f64);

        impl $name {
            /// Create a new metric value.
            ///
            /// # Errors
            ///
            /// Returns `MetricError` if value is negative, NaN, or infinite.
            #[inline]
            pub fn new(value: f64) -> Result<Self, MetricError> {
                if value.is_nan() {
                    Err(MetricError::NaN)
                } else if value.is_infinite() {
                    Err(MetricError::Infinite)
                } else if value < 0.0 {
                    Err(MetricError::Negative(value))
                } else {
                    Ok(Self(value))
                }
            }

            /// Create a new metric value, clamping to valid range.
            ///
            /// NaN becomes 0.0, negative becomes 0.0, infinite becomes `f64::MAX`.
            #[inline]
            #[must_use]
            pub fn new_clamped(value: f64) -> Self {
                if value.is_nan() || value < 0.0 {
                    Self(0.0)
                } else if value.is_infinite() {
                    Self(f64::MAX)
                } else {
                    Self(value)
                }
            }

            /// Create from raw value without validation.
            ///
            /// # Safety (Logical)
            ///
            /// Caller must ensure value is non-negative, finite, and not NaN.
            #[inline]
            #[must_use]
            pub const fn new_unchecked(value: f64) -> Self {
                Self(value)
            }

            /// Get the raw f64 value.
            #[inline]
            #[must_use]
            pub const fn value(self) -> f64 {
                self.0
            }

            /// Check if this metric exceeds the standard signal threshold.
            #[inline]
            #[must_use]
            pub fn exceeds_threshold(self) -> bool {
                self.0 >= $default_threshold
            }

            /// Check if this metric exceeds a custom threshold.
            #[inline]
            #[must_use]
            pub fn exceeds(self, threshold: f64) -> bool {
                self.0 >= threshold
            }

            /// Zero value (no signal).
            pub const ZERO: Self = Self(0.0);

            /// Standard signal threshold.
            pub const $threshold_name: f64 = $default_threshold;
        }

        impl Default for $name {
            fn default() -> Self {
                Self::ZERO
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:.4}", self.0)
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                (self.0 - other.0).abs() < f64::EPSILON
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.0.partial_cmp(&other.0)
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_f64(self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                struct MetricVisitor;

                impl Visitor<'_> for MetricVisitor {
                    type Value = f64;

                    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                        formatter.write_str("a non-negative finite number")
                    }

                    fn visit_f64<E: de::Error>(self, value: f64) -> Result<f64, E> {
                        Ok(value)
                    }

                    fn visit_i64<E: de::Error>(self, value: i64) -> Result<f64, E> {
                        Ok(value as f64)
                    }

                    fn visit_u64<E: de::Error>(self, value: u64) -> Result<f64, E> {
                        Ok(value as f64)
                    }
                }

                let value = deserializer.deserialize_f64(MetricVisitor)?;
                $name::new(value).map_err(de::Error::custom)
            }
        }

        impl From<$name> for f64 {
            fn from(metric: $name) -> f64 {
                metric.0
            }
        }
    };
}

ratio_newtype!(
    /// Proportional Reporting Ratio (PRR).
    ///
    /// PRR = P(Event|Drug) / P(Event|No Drug)
    ///
    /// Signal threshold: >= 2.0 (Evans criteria)
    Prr, SIGNAL_THRESHOLD, 2.0
);

ratio_newtype!(
    /// Reporting Odds Ratio (ROR).
    ///
    /// ROR = (a*d) / (b*c)
    ///
    /// Signal threshold: >= 2.0 with lower CI >= 1.0
    Ror, SIGNAL_THRESHOLD, 2.0
);

ratio_newtype!(
    /// Empirical Bayes Geometric Mean (EBGM).
    ///
    /// EBGM uses MGPS shrinkage estimation.
    ///
    /// Signal threshold: >= 2.0 with EB05 >= 2.0
    Ebgm, SIGNAL_THRESHOLD, 2.0
);

/// Information Component (IC).
///
/// IC = log2(observed / expected)
///
/// Unlike ratio metrics, IC can be negative (when observed < expected).
/// Signal threshold: IC025 > 0
#[derive(Debug, Clone, Copy)]
pub struct Ic(f64);

impl Ic {
    /// Create a new IC value.
    ///
    /// # Errors
    ///
    /// Returns `MetricError` if value is NaN or infinite.
    #[inline]
    pub fn new(value: f64) -> Result<Self, MetricError> {
        if value.is_nan() {
            Err(MetricError::NaN)
        } else if value.is_infinite() {
            Err(MetricError::Infinite)
        } else {
            Ok(Self(value))
        }
    }

    /// Create a new IC value, clamping to valid range.
    #[inline]
    #[must_use]
    pub fn new_clamped(value: f64) -> Self {
        if value.is_nan() {
            Self(0.0)
        } else if value.is_infinite() {
            Self(if value > 0.0 { f64::MAX } else { f64::MIN })
        } else {
            Self(value)
        }
    }

    /// Create from raw value without validation.
    #[inline]
    #[must_use]
    pub const fn new_unchecked(value: f64) -> Self {
        Self(value)
    }

    /// Get the raw f64 value.
    #[inline]
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Check if IC025 (lower bound) indicates a signal.
    ///
    /// For IC, signal detection uses the lower credibility bound > 0.
    #[inline]
    #[must_use]
    pub fn is_signal_lower_bound(self) -> bool {
        self.0 > 0.0
    }

    /// Zero value (no disproportionality).
    pub const ZERO: Self = Self(0.0);

    /// Standard signal threshold for IC025.
    pub const SIGNAL_THRESHOLD: f64 = 0.0;
}

impl Default for Ic {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Ic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

impl PartialEq for Ic {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < f64::EPSILON
    }
}

impl PartialOrd for Ic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Serialize for Ic {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

impl<'de> Deserialize<'de> for Ic {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = f64::deserialize(deserializer)?;
        Ic::new(value).map_err(de::Error::custom)
    }
}

impl From<Ic> for f64 {
    fn from(metric: Ic) -> f64 {
        metric.0
    }
}

/// Chi-square statistic for significance testing.
///
/// Used with PRR for Evans criteria (threshold: 3.841 for p < 0.05).
#[derive(Debug, Clone, Copy)]
pub struct ChiSquare(f64);

impl ChiSquare {
    /// Chi-square critical value for p < 0.05 with df = 1.
    /// CRITICAL: Exact value is 3.841, NOT 4.0.
    pub const CRITICAL_05: f64 = 3.841;

    /// Create a new chi-square value.
    ///
    /// # Errors
    ///
    /// Returns `MetricError` if value is negative, NaN, or infinite.
    #[inline]
    pub fn new(value: f64) -> Result<Self, MetricError> {
        if value.is_nan() {
            Err(MetricError::NaN)
        } else if value.is_infinite() {
            Err(MetricError::Infinite)
        } else if value < 0.0 {
            Err(MetricError::Negative(value))
        } else {
            Ok(Self(value))
        }
    }

    /// Get the raw f64 value.
    #[inline]
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Check if statistically significant at p < 0.05.
    #[inline]
    #[must_use]
    pub fn is_significant(self) -> bool {
        self.0 >= Self::CRITICAL_05
    }

    /// Zero value.
    pub const ZERO: Self = Self(0.0);
}

impl Default for ChiSquare {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for ChiSquare {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

impl PartialEq for ChiSquare {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < f64::EPSILON
    }
}

impl PartialOrd for ChiSquare {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Serialize for ChiSquare {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

impl<'de> Deserialize<'de> for ChiSquare {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = f64::deserialize(deserializer)?;
        ChiSquare::new(value).map_err(de::Error::custom)
    }
}

impl From<ChiSquare> for f64 {
    fn from(metric: ChiSquare) -> f64 {
        metric.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prr_creation() {
        let prr = Prr::new(3.5).unwrap();
        assert!((prr.value() - 3.5).abs() < f64::EPSILON);

        // Negative should fail
        assert!(Prr::new(-1.0).is_err());

        // NaN should fail
        assert!(Prr::new(f64::NAN).is_err());

        // Infinite should fail
        assert!(Prr::new(f64::INFINITY).is_err());
    }

    #[test]
    fn test_prr_clamped() {
        let prr = Prr::new_clamped(-1.0);
        assert!((prr.value() - 0.0).abs() < f64::EPSILON);

        let prr = Prr::new_clamped(f64::NAN);
        assert!((prr.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_prr_threshold() {
        let below = Prr::new(1.5).unwrap();
        let above = Prr::new(2.5).unwrap();

        assert!(!below.exceeds_threshold());
        assert!(above.exceeds_threshold());
    }

    #[test]
    fn test_ic_negative_valid() {
        // IC can be negative (observed < expected)
        let ic = Ic::new(-2.5).unwrap();
        assert!((ic.value() - (-2.5)).abs() < f64::EPSILON);
        assert!(!ic.is_signal_lower_bound());

        let ic_positive = Ic::new(1.5).unwrap();
        assert!(ic_positive.is_signal_lower_bound());
    }

    #[test]
    fn test_chi_square_significance() {
        let low = ChiSquare::new(2.0).unwrap();
        let high = ChiSquare::new(5.0).unwrap();
        let critical = ChiSquare::new(3.841).unwrap();

        assert!(!low.is_significant());
        assert!(high.is_significant());
        assert!(critical.is_significant());
    }

    #[test]
    fn test_serialization() {
        let prr = Prr::new(3.5).unwrap();
        let json = serde_json::to_string(&prr).unwrap();
        assert_eq!(json, "3.5");

        let deserialized: Prr = serde_json::from_str(&json).unwrap();
        assert_eq!(prr, deserialized);
    }

    #[test]
    fn test_type_safety() {
        // This test documents that different metric types cannot be compared
        // directly (compile-time safety)
        let prr = Prr::new(3.5).unwrap();
        let ror = Ror::new(3.5).unwrap();

        // Cannot directly compare: prr == ror (different types!)
        // Must explicitly convert:
        assert!((prr.value() - ror.value()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_partial_ord() {
        let prr1 = Prr::new(2.0).unwrap();
        let prr2 = Prr::new(3.0).unwrap();

        assert!(prr1 < prr2);
        assert!(prr2 > prr1);
    }
}
