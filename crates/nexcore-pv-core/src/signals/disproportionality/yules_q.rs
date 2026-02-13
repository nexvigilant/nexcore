//! Yule's Q Coefficient for Disproportionality Analysis
//!
//! Yule's Q is a measure of association for 2×2 contingency tables,
//! normalized to range [-1, +1]. It's the normalized form of the odds ratio.
//!
//! # Formula
//!
//! ```text
//! Q = (ad - bc) / (ad + bc)
//!   = (OR - 1) / (OR + 1)
//!
//! where OR = (a×d) / (b×c) is the odds ratio
//! ```
//!
//! # Interpretation
//!
//! | Q Value | Interpretation |
//! |---------|----------------|
//! | +1.0 | Perfect positive association |
//! | +0.5 to +1.0 | Strong positive association |
//! | +0.25 to +0.5 | Moderate positive association |
//! | -0.25 to +0.25 | Weak or no association |
//! | -1.0 | Perfect negative association |
//!
//! # Advantages over ROR/PRR
//!
//! - **Bounded**: Always in [-1, +1], unlike ROR which can be infinite
//! - **Symmetric**: Q(A,B) = -Q(B,A) for complementary associations
//! - **Robust**: Less sensitive to extreme cell counts
//!
//! # References
//!
//! - Yule GU (1900). "On the association of attributes in statistics."
//!   Philosophical Transactions of the Royal Society A 194:257-319.
//!   DOI: [10.1098/rsta.1900.0019](https://doi.org/10.1098/rsta.1900.0019)
//!
//! - Yule GU (1912). "On the methods of measuring association between two attributes."
//!   Journal of the Royal Statistical Society 75(6):579-652.
//!   DOI: [10.2307/2340126](https://doi.org/10.2307/2340126)
//!
//! - Agresti A (2002). Categorical Data Analysis, 2nd ed. Wiley.
//!   ISBN: 978-0-471-36093-3

use crate::signals::core::error::SignalError;
use crate::signals::core::types::{ContingencyTable, SignalCriteria};
use serde::{Deserialize, Serialize};

/// Result of Yule's Q calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YulesQResult {
    /// Yule's Q coefficient [-1, +1]
    pub q: f64,
    /// Standard error of Q
    pub se: f64,
    /// Lower 95% confidence interval
    pub lower_ci: f64,
    /// Upper 95% confidence interval
    pub upper_ci: f64,
    /// Whether a signal is detected (Q > threshold, CI excludes 0)
    pub is_signal: bool,
    /// Association strength category
    pub strength: AssociationStrength,
    /// Case count (a)
    pub case_count: u64,
}

/// Strength of association based on Q value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssociationStrength {
    /// Q > 0.5: Strong positive
    StrongPositive,
    /// 0.25 < Q ≤ 0.5: Moderate positive
    ModeratePositive,
    /// 0 < Q ≤ 0.25: Weak positive
    WeakPositive,
    /// -0.25 ≤ Q ≤ 0: None/negligible
    None,
    /// -0.5 ≤ Q < -0.25: Weak negative
    WeakNegative,
    /// -1 ≤ Q < -0.5: Moderate negative
    ModerateNegative,
    /// Q < -0.5: Strong negative (protective)
    StrongNegative,
}

impl AssociationStrength {
    /// Classify Q value into strength category.
    #[must_use]
    pub fn classify(q: f64) -> Self {
        if q > 0.5 {
            Self::StrongPositive
        } else if q > 0.25 {
            Self::ModeratePositive
        } else if q > 0.0 {
            Self::WeakPositive
        } else if q > -0.25 {
            Self::None
        } else if q > -0.5 {
            Self::WeakNegative
        } else {
            Self::ModerateNegative
        }
    }

    /// Whether this represents a positive (risk) association.
    #[must_use]
    pub const fn is_positive(&self) -> bool {
        matches!(
            self,
            Self::StrongPositive | Self::ModeratePositive | Self::WeakPositive
        )
    }

    /// Whether this represents a protective (negative) association.
    #[must_use]
    pub const fn is_protective(&self) -> bool {
        matches!(
            self,
            Self::StrongNegative | Self::ModerateNegative | Self::WeakNegative
        )
    }
}

/// Calculate Yule's Q coefficient from contingency table.
///
/// # Arguments
///
/// * `table` - 2×2 contingency table
/// * `criteria` - Signal detection thresholds
///
/// # Returns
///
/// `YulesQResult` with Q coefficient, confidence interval, and signal status.
///
/// # Complexity
///
/// - **Time**: O(1)
/// - **Space**: O(1)
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::pv::signals::{ContingencyTable, SignalCriteria};
/// use nexcore_vigilance::pv::signals::disproportionality::yules_q::calculate_yules_q;
///
/// let table = ContingencyTable::new(10, 90, 100, 9800);
/// let criteria = SignalCriteria::evans();
/// let result = calculate_yules_q(&table, &criteria).unwrap();
///
/// println!("Q = {:.3}, Strength: {:?}", result.q, result.strength);
/// ```
pub fn calculate_yules_q(
    table: &ContingencyTable,
    criteria: &SignalCriteria,
) -> Result<YulesQResult, SignalError> {
    if !table.is_valid() {
        return Err(SignalError::invalid_table("Invalid contingency table"));
    }

    let a = table.a as f64;
    let b = table.b as f64;
    let c = table.c as f64;
    let d = table.d as f64;

    // Apply continuity correction if any cell is zero
    let (a, b, c, d) = if a == 0.0 || b == 0.0 || c == 0.0 || d == 0.0 {
        (a + 0.5, b + 0.5, c + 0.5, d + 0.5)
    } else {
        (a, b, c, d)
    };

    let ad = a * d;
    let bc = b * c;

    // Q = (ad - bc) / (ad + bc)
    let denominator = ad + bc;
    let q = if denominator > 0.0 {
        (ad - bc) / denominator
    } else {
        0.0
    };

    // Standard error using delta method
    // SE(Q) = (1 - Q²) × sqrt(1/a + 1/b + 1/c + 1/d) / 2
    let q_sq = q * q;
    let sum_inv = 1.0 / a + 1.0 / b + 1.0 / c + 1.0 / d;
    let se = (1.0 - q_sq) * sum_inv.sqrt() / 2.0;

    // 95% CI
    let z = 1.96;
    let lower_ci = (q - z * se).max(-1.0);
    let upper_ci = (q + z * se).min(1.0);

    // Signal detection: Q > threshold and lower CI > 0
    let q_threshold = 0.25; // Moderate association threshold
    let is_signal = q > q_threshold && lower_ci > 0.0 && table.a >= u64::from(criteria.min_cases);

    let strength = AssociationStrength::classify(q);

    Ok(YulesQResult {
        q,
        se,
        lower_ci,
        upper_ci,
        is_signal,
        strength,
        case_count: table.a,
    })
}

/// Convert odds ratio to Yule's Q.
///
/// Q = (OR - 1) / (OR + 1)
#[must_use]
pub fn or_to_q(or: f64) -> f64 {
    if or < 0.0 {
        return f64::NAN;
    }
    if or == f64::INFINITY {
        return 1.0;
    }
    (or - 1.0) / (or + 1.0)
}

/// Convert Yule's Q to odds ratio.
///
/// OR = (1 + Q) / (1 - Q)
#[must_use]
pub fn q_to_or(q: f64) -> f64 {
    if q <= -1.0 {
        return 0.0;
    }
    if q >= 1.0 {
        return f64::INFINITY;
    }
    (1.0 + q) / (1.0 - q)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yules_q_basic() {
        let table = ContingencyTable::new(10, 90, 100, 9800);
        let criteria = SignalCriteria::evans();

        let result = calculate_yules_q(&table, &criteria).unwrap();

        // Q should be positive (a×d > b×c)
        // 10 × 9800 = 98000, 90 × 100 = 9000
        assert!(result.q > 0.0);
        assert!(result.q <= 1.0);
    }

    #[test]
    fn test_yules_q_strong_signal() {
        // Strong association: many cases, few non-cases
        let table = ContingencyTable::new(50, 50, 10, 9890);
        let criteria = SignalCriteria::evans();

        let result = calculate_yules_q(&table, &criteria).unwrap();

        assert!(result.q > 0.5);
        assert_eq!(result.strength, AssociationStrength::StrongPositive);
        assert!(result.is_signal);
    }

    #[test]
    fn test_yules_q_no_association() {
        // No association: proportional cells
        let table = ContingencyTable::new(10, 90, 100, 900);
        let criteria = SignalCriteria::evans();

        let result = calculate_yules_q(&table, &criteria).unwrap();

        // Q should be close to 0
        assert!(result.q.abs() < 0.3);
    }

    #[test]
    fn test_yules_q_protective() {
        // Protective association: drug associated with FEWER events
        let table = ContingencyTable::new(5, 495, 100, 400);
        let criteria = SignalCriteria::evans();

        let result = calculate_yules_q(&table, &criteria).unwrap();

        // Q should be negative
        assert!(result.q < 0.0);
        assert!(result.strength.is_protective());
    }

    #[test]
    fn test_yules_q_zero_cell() {
        // Zero cell should trigger continuity correction
        let table = ContingencyTable::new(0, 100, 50, 9850);
        let criteria = SignalCriteria::evans();

        let result = calculate_yules_q(&table, &criteria);
        assert!(result.is_ok());
    }

    #[test]
    fn test_or_to_q() {
        // OR = 1 → Q = 0
        assert!((or_to_q(1.0) - 0.0).abs() < 0.001);

        // OR = 2 → Q = 1/3
        assert!((or_to_q(2.0) - 1.0 / 3.0).abs() < 0.001);

        // OR = ∞ → Q = 1
        assert!((or_to_q(f64::INFINITY) - 1.0).abs() < 0.001);

        // OR = 0 → Q = -1
        assert!((or_to_q(0.0) - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_q_to_or() {
        // Q = 0 → OR = 1
        assert!((q_to_or(0.0) - 1.0).abs() < 0.001);

        // Q = 0.5 → OR = 3
        assert!((q_to_or(0.5) - 3.0).abs() < 0.001);

        // Q = 1 → OR = ∞
        assert!(q_to_or(1.0).is_infinite());

        // Q = -1 → OR = 0
        assert!((q_to_or(-1.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_roundtrip() {
        for or in [0.5, 1.0, 2.0, 5.0, 10.0] {
            let q = or_to_q(or);
            let or_back = q_to_or(q);
            assert!(
                (or - or_back).abs() < 0.001,
                "Roundtrip failed for OR={}",
                or
            );
        }
    }

    #[test]
    fn test_ci_bounds() {
        let table = ContingencyTable::new(10, 90, 100, 9800);
        let criteria = SignalCriteria::evans();

        let result = calculate_yules_q(&table, &criteria).unwrap();

        assert!(result.lower_ci >= -1.0);
        assert!(result.upper_ci <= 1.0);
        assert!(result.lower_ci < result.q);
        assert!(result.upper_ci > result.q);
    }
}
