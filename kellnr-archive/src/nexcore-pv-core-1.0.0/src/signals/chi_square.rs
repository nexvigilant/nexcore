//! # Chi-Square Statistical Tests
//!
//! Chi-square tests for 2x2 contingency tables, including Yates correction and p-value calculation.

use crate::types::ContingencyTable;
use std::f64::consts::PI;

/// Calculate Pearson's chi-square statistic for a 2x2 table.
///
/// χ² = Σ (O - E)² / E
#[must_use]
pub fn calculate_chi_square(table: &ContingencyTable) -> f64 {
    let n = table.total() as f64;
    if n == 0.0 {
        return 0.0;
    }

    let row1 = table.row1() as f64;
    let row2 = table.row2() as f64;
    let col1 = table.col1() as f64;
    let col2 = table.col2() as f64;

    if row1 == 0.0 || row2 == 0.0 || col1 == 0.0 || col2 == 0.0 {
        return 0.0;
    }

    let e_a = (row1 * col1) / n;
    let e_b = (row1 * col2) / n;
    let e_c = (row2 * col1) / n;
    let e_d = (row2 * col2) / n;

    let a = table.a as f64;
    let b = table.b as f64;
    let c = table.c as f64;
    let d = table.d as f64;

    (a - e_a).powi(2) / e_a
        + (b - e_b).powi(2) / e_b
        + (c - e_c).powi(2) / e_c
        + (d - e_d).powi(2) / e_d
}

/// Calculate Yates-corrected chi-square for 2x2 table.
///
/// Applies continuity correction for small sample sizes.
/// χ²_Yates = N(|ad - bc| - N/2)² / (row1 * row2 * col1 * col2)
#[must_use]
pub fn calculate_yates_corrected_chi_square(table: &ContingencyTable) -> f64 {
    let n = table.total() as f64;
    if n == 0.0 {
        return 0.0;
    }

    let row1 = table.row1() as f64;
    let row2 = table.row2() as f64;
    let col1 = table.col1() as f64;
    let col2 = table.col2() as f64;

    let denom = row1 * row2 * col1 * col2;
    if denom == 0.0 {
        return 0.0;
    }

    let a = table.a as f64;
    let b = table.b as f64;
    let c = table.c as f64;
    let d = table.d as f64;

    let ad_bc = (a * d - b * c).abs();
    let numerator = n * (0.0_f64.max(ad_bc - n / 2.0).powi(2));

    numerator / denom
}

/// Calculate p-value for chi-square statistic with 1 degree of freedom.
///
/// Uses Wilson-Hilferty approximation for chi-square CDF.
#[must_use]
pub fn calculate_chi_square_p_value(chi_square: f64) -> f64 {
    if chi_square <= 0.0 {
        return 1.0;
    }

    // Degrees of freedom is 1 for 2x2 tables in signal detection
    let df = 1.0;

    // Wilson-Hilferty transformation to approximate normal
    let z =
        ((chi_square / df).powf(1.0 / 3.0) - (1.0 - 2.0 / (9.0 * df))) / (2.0 / (9.0 * df)).sqrt();

    // Standard normal CDF approximation (Abramowitz & Stegun)
    if z < 0.0 {
        1.0 - standard_normal_cdf(-z)
    } else {
        1.0 - standard_normal_cdf(z)
    }
}

/// Approximate standard normal CDF using Abramowitz & Stegun 26.2.17.
fn standard_normal_cdf(z: f64) -> f64 {
    if z < -8.0 {
        return 0.0;
    }
    if z > 8.0 {
        return 1.0;
    }

    let b1 = 0.319381530;
    let b2 = -0.356563782;
    let b3 = 1.781477937;
    let b4 = -1.821255978;
    let b5 = 1.330274429;
    let p = 0.2316419;

    let t = 1.0 / (1.0 + p * z.abs());
    let pdf = (-z * z / 2.0).exp() / (2.0 * PI).sqrt();
    let cdf =
        1.0 - pdf * (b1 * t + b2 * t.powi(2) + b3 * t.powi(3) + b4 * t.powi(4) + b5 * t.powi(5));

    if z >= 0.0 { cdf } else { 1.0 - cdf }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chi_square_basic() {
        let table = ContingencyTable::new(10, 90, 100, 9800);
        let chi_sq = calculate_chi_square(&table);
        assert!(chi_sq > 40.0); // Strong signal
    }

    #[test]
    fn test_yates_correction() {
        let table = ContingencyTable::new(10, 90, 100, 9800);
        let chi_sq = calculate_chi_square(&table);
        let yates = calculate_yates_corrected_chi_square(&table);
        assert!(yates < chi_sq); // Yates should reduce the statistic
    }

    #[test]
    fn test_p_value() {
        // Chi-square of 3.841 should be approx p=0.05
        let p = calculate_chi_square_p_value(3.841);
        assert!((p - 0.05).abs() < 0.01);

        // Very high chi-square should be approx p=0
        let p_low = calculate_chi_square_p_value(100.0);
        assert!(p_low < 0.001);
    }
}
