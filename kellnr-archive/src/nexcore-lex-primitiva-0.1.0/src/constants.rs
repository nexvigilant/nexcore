//! # Mathematical Constants (Terminal Layer)
//!
//! All Lex Primitiva ultimately ground to these mathematical constants.
//!
//! ## The Two Root Constants
//!
//! - **ZERO (0)**: Absence constant (additive identity, empty set, falsity, origin)
//! - **ONE (1)**: Existence constant (multiplicative identity, witness, truth, unit)
//!
//! ## Tier: T1-Universal
//!
//! These constants are the terminal nodes of the primitive dependency graph.

use crate::symbols::*;
use serde::{Deserialize, Serialize};
use std::f64::consts::{E, LN_2, PI};

/// Golden ratio: φ = (1 + √5) / 2
///
/// Appears in: Recursion (Fibonacci limit), self-similar structures.
pub const PHI: f64 = 1.618_033_988_749_895;

/// Boltzmann constant in J/K.
///
/// Appears in: Irreversibility (entropy), Persistence (thermal decay).
pub const K_BOLTZMANN: f64 = 1.380_649e-23;

/// Standard statistical significance level (α = 0.05).
///
/// Appears in: Comparison (decision rules), Boundary (thresholds).
pub const ALPHA_SIGNIFICANCE: f64 = 0.05;

/// Chi-square critical value for p < 0.05, df = 1.
///
/// Appears in: Comparison, Boundary.
pub const CHI_SQUARE_CRITICAL: f64 = 3.841;

/// Nyquist sampling factor.
///
/// Appears in: Frequency (sampling theorem).
pub const NYQUIST_FACTOR: u32 = 2;

/// First infinite ordinal (sentinel value).
///
/// Appears in: Sequence (transfinite), Quantity (cardinality of ℕ).
pub const OMEGA_SENTINEL: u64 = u64::MAX;

/// A mathematical constant that grounds a primitive.
///
/// Each constant has a value, symbol, name, and the primitives it grounds.
/// Note: This type uses `&'static str` for compile-time constant definitions,
/// so it only implements Serialize (not Deserialize).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct MathConstant {
    /// The numeric value (if finite).
    pub value: Option<f64>,
    /// The mathematical symbol.
    pub symbol: &'static str,
    /// Human-readable name.
    pub name: &'static str,
    /// Brief description of its role.
    pub description: &'static str,
    /// Is this constant finite?
    pub is_finite: bool,
}

impl MathConstant {
    /// Zero: The absence constant.
    pub const ZERO: Self = Self {
        value: Some(0.0),
        symbol: CONST_ZERO,
        name: "Zero",
        description: "Additive identity, empty set cardinality, falsity, origin",
        is_finite: true,
    };

    /// One: The existence constant.
    pub const ONE: Self = Self {
        value: Some(1.0),
        symbol: CONST_ONE,
        name: "One",
        description: "Multiplicative identity, witness minimum, truth, unit",
        is_finite: true,
    };

    /// Pi: The cycle constant.
    pub const PI: Self = Self {
        value: Some(PI),
        symbol: CONST_PI,
        name: "Pi",
        description: "Ratio of circumference to diameter, half-cycle in radians",
        is_finite: true,
    };

    /// Euler's number: The growth constant.
    pub const E: Self = Self {
        value: Some(E),
        symbol: CONST_E,
        name: "Euler's Number",
        description: "Base of natural logarithm, continuous growth rate",
        is_finite: true,
    };

    /// Golden ratio: The self-similarity constant.
    pub const PHI: Self = Self {
        value: Some(PHI),
        symbol: CONST_PHI,
        name: "Golden Ratio",
        description: "Fibonacci limit, self-similar proportion",
        is_finite: true,
    };

    /// Infinity: The unbounded constant.
    pub const INFINITY: Self = Self {
        value: None,
        symbol: CONST_INFINITY,
        name: "Infinity",
        description: "Unbounded magnitude, cardinal of naturals",
        is_finite: false,
    };

    /// Omega: The first transfinite ordinal.
    pub const OMEGA: Self = Self {
        value: None,
        symbol: CONST_OMEGA,
        name: "Omega",
        description: "First infinite ordinal, limit of finite ordinals",
        is_finite: false,
    };

    /// Natural log of 2: The bit constant.
    pub const LN_2: Self = Self {
        value: Some(LN_2),
        symbol: CONST_LN2,
        name: "Natural Log of 2",
        description: "Nats per bit, information unit conversion",
        is_finite: true,
    };

    /// Boltzmann constant: The entropy constant.
    pub const K_BOLTZMANN: Self = Self {
        value: Some(K_BOLTZMANN),
        symbol: CONST_KB,
        name: "Boltzmann Constant",
        description: "Entropy-temperature relation, thermal energy per kelvin",
        is_finite: true,
    };

    /// Epsilon: The infinitesimal constant.
    pub const EPSILON: Self = Self {
        value: Some(f64::EPSILON),
        symbol: CONST_EPSILON,
        name: "Epsilon",
        description: "Arbitrarily small positive, limit definition",
        is_finite: true,
    };

    /// Returns all 10 fundamental constants.
    #[must_use]
    pub const fn all() -> [Self; 10] {
        [
            Self::ZERO,
            Self::ONE,
            Self::PI,
            Self::E,
            Self::PHI,
            Self::INFINITY,
            Self::OMEGA,
            Self::LN_2,
            Self::K_BOLTZMANN,
            Self::EPSILON,
        ]
    }

    /// Returns the two root constants (0 and 1).
    #[must_use]
    pub const fn roots() -> [Self; 2] {
        [Self::ZERO, Self::ONE]
    }

    /// Get the numeric value, returning infinity if unbounded.
    #[must_use]
    pub fn numeric_value(&self) -> f64 {
        self.value.unwrap_or(f64::INFINITY)
    }
}

impl std::fmt::Display for MathConstant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            Some(v) => write!(f, "{} = {:.6}", self.symbol, v),
            None => write!(f, "{} (unbounded)", self.symbol),
        }
    }
}

/// Trichotomy result for comparison operations.
///
/// Grounds to: {-1, 0, +1}.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i8)]
pub enum Trichotomy {
    /// Less than: -1
    Less = -1,
    /// Equal: 0
    Equal = 0,
    /// Greater than: +1
    Greater = 1,
}

impl Trichotomy {
    /// Convert from i8.
    #[must_use]
    pub const fn from_i8(value: i8) -> Option<Self> {
        match value {
            -1 => Some(Self::Less),
            0 => Some(Self::Equal),
            1 => Some(Self::Greater),
            _ => None,
        }
    }

    /// Convert to i8.
    #[must_use]
    pub const fn as_i8(self) -> i8 {
        self as i8
    }
}

impl From<std::cmp::Ordering> for Trichotomy {
    fn from(ord: std::cmp::Ordering) -> Self {
        match ord {
            std::cmp::Ordering::Less => Self::Less,
            std::cmp::Ordering::Equal => Self::Equal,
            std::cmp::Ordering::Greater => Self::Greater,
        }
    }
}

impl From<Trichotomy> for std::cmp::Ordering {
    fn from(tri: Trichotomy) -> Self {
        match tri {
            Trichotomy::Less => Self::Less,
            Trichotomy::Equal => Self::Equal,
            Trichotomy::Greater => Self::Greater,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_values() {
        assert!((MathConstant::ZERO.numeric_value() - 0.0).abs() < f64::EPSILON);
        assert!((MathConstant::ONE.numeric_value() - 1.0).abs() < f64::EPSILON);
        assert!((MathConstant::PI.numeric_value() - PI).abs() < f64::EPSILON);
        assert!((MathConstant::E.numeric_value() - E).abs() < f64::EPSILON);
        assert!((MathConstant::PHI.numeric_value() - PHI).abs() < 1e-10);
    }

    #[test]
    fn test_infinity_constants() {
        assert!(!MathConstant::INFINITY.is_finite);
        assert!(!MathConstant::OMEGA.is_finite);
        assert!(MathConstant::INFINITY.numeric_value().is_infinite());
    }

    #[test]
    fn test_all_constants() {
        let all = MathConstant::all();
        assert_eq!(all.len(), 10);
    }

    #[test]
    fn test_roots() {
        let roots = MathConstant::roots();
        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0].symbol, CONST_ZERO);
        assert_eq!(roots[1].symbol, CONST_ONE);
    }

    #[test]
    fn test_trichotomy() {
        assert_eq!(Trichotomy::Less.as_i8(), -1);
        assert_eq!(Trichotomy::Equal.as_i8(), 0);
        assert_eq!(Trichotomy::Greater.as_i8(), 1);

        assert_eq!(Trichotomy::from_i8(-1), Some(Trichotomy::Less));
        assert_eq!(Trichotomy::from_i8(0), Some(Trichotomy::Equal));
        assert_eq!(Trichotomy::from_i8(1), Some(Trichotomy::Greater));
        assert_eq!(Trichotomy::from_i8(2), None);
    }

    #[test]
    fn test_trichotomy_ordering_conversion() {
        use std::cmp::Ordering;

        assert_eq!(Trichotomy::from(Ordering::Less), Trichotomy::Less);
        assert_eq!(Trichotomy::from(Ordering::Equal), Trichotomy::Equal);
        assert_eq!(Trichotomy::from(Ordering::Greater), Trichotomy::Greater);

        assert_eq!(Ordering::from(Trichotomy::Less), Ordering::Less);
        assert_eq!(Ordering::from(Trichotomy::Equal), Ordering::Equal);
        assert_eq!(Ordering::from(Trichotomy::Greater), Ordering::Greater);
    }

    #[test]
    fn test_display() {
        let pi = MathConstant::PI;
        let display = format!("{pi}");
        assert!(display.contains(CONST_PI));
        assert!(display.contains("3.14"));

        let inf = MathConstant::INFINITY;
        let display = format!("{inf}");
        assert!(display.contains(CONST_INFINITY));
        assert!(display.contains("unbounded"));
    }
}
