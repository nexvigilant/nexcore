//! Fisher's Exact Test for 2×2 contingency tables.
//!
//! Use when any cell count is < 5 (chi-square approximation fails)
//! or when dealing with rare adverse events.

use crate::types::TwoByTwoTable;

/// Result of Fisher's Exact Test.
#[derive(Debug, Clone)]
pub struct FisherResult {
    /// One-tailed p-value (over-representation direction).
    pub p_value_one_tailed: f64,
    /// Two-tailed p-value.
    pub p_value_two_tailed: f64,
    /// `true` when one-tailed p < 0.05.
    pub is_signal: bool,
}

/// Calculate log hypergeometric probability for a 2×2 table.
#[must_use]
pub fn log_hypergeometric_prob(table: &TwoByTwoTable) -> f64 {
    let n = table.total() as f64;
    let a = table.a as f64;
    let b = table.b as f64;
    let c = table.c as f64;
    let d = table.d as f64;

    ln_factorial(a + b) + ln_factorial(c + d) + ln_factorial(a + c) + ln_factorial(b + d)
        - ln_factorial(a)
        - ln_factorial(b)
        - ln_factorial(c)
        - ln_factorial(d)
        - ln_factorial(n)
}

/// Fisher's Exact Test for a 2×2 contingency table.
#[must_use]
pub fn fisher_exact_test(table: &TwoByTwoTable) -> FisherResult {
    let r1 = table.a + table.b;
    let r2 = table.c + table.d;
    let c1 = table.a + table.c;
    let observed_log_prob = log_hypergeometric_prob(table);
    let max_a = r1.min(c1);

    // One-tailed: sum probabilities for a ≥ observed
    let mut p_one = 0.0_f64;
    for a in table.a..=max_a {
        let b = r1 - a;
        let c = c1 - a;
        let d = r2.saturating_sub(c);
        p_one += log_hypergeometric_prob(&TwoByTwoTable::new(a, b, c, d)).exp();
    }

    // Two-tailed: sum all tables at least as extreme as observed
    let min_a = c1.saturating_sub(r2);
    let mut p_two = 0.0_f64;
    for a in min_a..=max_a {
        let b = r1 - a;
        let c = c1 - a;
        let d = r2.saturating_sub(c);
        let lp = log_hypergeometric_prob(&TwoByTwoTable::new(a, b, c, d));
        if lp <= observed_log_prob + 1e-10 {
            p_two += lp.exp();
        }
    }

    FisherResult {
        p_value_one_tailed: p_one.min(1.0),
        p_value_two_tailed: p_two.min(1.0),
        is_signal: p_one < 0.05,
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn ln_factorial(n: f64) -> f64 {
    if n <= 1.0 {
        return 0.0;
    }
    lgamma(n + 1.0)
}

/// Lanczos approximation for ln Γ(x).
fn lgamma(x: f64) -> f64 {
    const P: [f64; 8] = [
        676.520_368_121_885_1,
        -1259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    const G: f64 = 7.0;
    if x < 0.5 {
        return (std::f64::consts::PI / (std::f64::consts::PI * x).sin()).ln() - lgamma(1.0 - x);
    }
    let x = x - 1.0;
    let mut a = 0.999_999_999_999_809_9_f64;
    for (i, &coef) in P.iter().enumerate() {
        a += coef / (x + i as f64 + 1.0);
    }
    let t = x + G + 0.5;
    0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + a.ln()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fisher_rare_event_is_signal() {
        let table = TwoByTwoTable::new(3, 7, 1, 89);
        let result = fisher_exact_test(&table);
        assert!(result.p_value_one_tailed < 0.05);
        assert!(result.is_signal);
    }

    #[test]
    fn fisher_balanced_not_signal() {
        let table = TwoByTwoTable::new(5, 5, 5, 5);
        let result = fisher_exact_test(&table);
        assert!(!result.is_signal);
    }
}
