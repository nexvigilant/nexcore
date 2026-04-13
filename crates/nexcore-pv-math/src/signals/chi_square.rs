//! Chi-square tests for 2×2 contingency tables.
//!
//! Includes Pearson chi-square, Yates-corrected chi-square, and p-value.

use crate::stats::{chi_square_p_value, chi_square_statistic};
use crate::types::TwoByTwoTable;

/// Result of a chi-square test.
#[derive(Debug, Clone)]
pub struct ChiSquare {
    /// Pearson chi-square statistic.
    pub statistic: f64,
    /// Yates-corrected statistic.
    pub statistic_yates: f64,
    /// Two-tailed p-value (df = 1).
    pub p_value: f64,
    /// `true` when p < 0.05 (statistic ≥ 3.841).
    pub is_significant: bool,
}

/// Calculate Pearson chi-square and Yates-corrected chi-square for a 2×2 table.
#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn calculate_chi_square(table: &TwoByTwoTable) -> ChiSquare {
    let a = table.a as f64;
    let b = table.b as f64;
    let c = table.c as f64;
    let d = table.d as f64;

    let statistic = chi_square_statistic(a, b, c, d);
    let statistic_yates = yates_corrected(a, b, c, d);
    let p_value = chi_square_p_value(statistic);

    ChiSquare {
        statistic,
        statistic_yates,
        p_value,
        is_significant: statistic >= crate::stats::CHI_SQUARE_CRITICAL_05,
    }
}

/// Yates-corrected chi-square: N(|ad - bc| - N/2)² / (row1 × row2 × col1 × col2).
#[must_use]
#[allow(clippy::many_single_char_names)]
fn yates_corrected(a: f64, b: f64, c: f64, d: f64) -> f64 {
    let n = a + b + c + d;
    if n == 0.0 {
        return 0.0;
    }
    let denom = (a + b) * (c + d) * (a + c) * (b + d);
    if denom == 0.0 {
        return 0.0;
    }
    let ad_bc = (a * d - b * c).abs();
    let corrected = if ad_bc > n / 2.0 {
        ad_bc - n / 2.0
    } else {
        0.0
    };
    n * corrected.powi(2) / denom
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chi_square_strong_signal() {
        let table = TwoByTwoTable::new(10, 90, 100, 9800);
        let result = calculate_chi_square(&table);
        assert!(result.statistic > 40.0);
        assert!(result.is_significant);
        assert!(result.p_value < 0.05);
    }

    #[test]
    fn yates_is_less_than_pearson() {
        let table = TwoByTwoTable::new(10, 90, 100, 9800);
        let result = calculate_chi_square(&table);
        assert!(result.statistic_yates < result.statistic);
    }

    #[test]
    fn chi_square_no_association() {
        // Balanced table — no association
        let table = TwoByTwoTable::new(50, 50, 50, 50);
        let result = calculate_chi_square(&table);
        assert!(!result.is_significant);
    }
}
