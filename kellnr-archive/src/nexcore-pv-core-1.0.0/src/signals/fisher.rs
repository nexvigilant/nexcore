//! # Fisher's Exact Test
//!
//! Exact test for 2x2 contingency tables.
//!
//! Fisher's Exact Test is essential for pharmacovigilance when:
//! - Any cell count is < 5 (chi-square approximation fails)
//! - Dealing with rare adverse events
//! - Need mathematically precise p-values

use crate::types::ContingencyTable;

/// Fisher's Exact Test result
#[derive(Debug, Clone)]
pub struct FisherResult {
    /// One-tailed p-value (for over-representation)
    pub p_value_one_tailed: f64,
    /// Two-tailed p-value
    pub p_value_two_tailed: f64,
    /// Whether the test indicates a signal at alpha=0.05
    pub is_signal: bool,
}

/// Calculate log of hypergeometric probability for a 2x2 table.
#[must_use]
pub fn log_hypergeometric_prob(table: &ContingencyTable) -> f64 {
    let n = table.total() as f64;
    let a = table.a as f64;
    let b = table.b as f64;
    let c = table.c as f64;
    let d = table.d as f64;

    ln_factorial(a + b) + ln_factorial(c + d) + ln_factorial(a + c) + ln_factorial(b + d)
        - (ln_factorial(a) + ln_factorial(b) + ln_factorial(c) + ln_factorial(d) + ln_factorial(n))
}

/// Calculate Fisher's Exact Test for a 2x2 contingency table.
#[must_use]
pub fn fisher_exact_test(table: &ContingencyTable) -> FisherResult {
    let r1 = table.a + table.b;
    let r2 = table.c + table.d;
    let c1 = table.a + table.c;
    let observed_log_prob = log_hypergeometric_prob(table);
    let mut p_one_tailed = 0.0;
    let max_a = r1.min(c1);

    for a in table.a..=max_a {
        let b = r1 - a;
        let c = c1 - a;
        let d = r2.saturating_sub(c);
        let t = ContingencyTable::new(a, b, c, d);
        p_one_tailed += log_hypergeometric_prob(&t).exp();
    }

    let mut p_two_tailed = 0.0;
    let min_a = c1.saturating_sub(r2);

    for a in min_a..=max_a {
        let b = r1 - a;
        let c = c1 - a;
        let d = r2.saturating_sub(c);
        let t = ContingencyTable::new(a, b, c, d);
        let log_prob = log_hypergeometric_prob(&t);
        if log_prob <= observed_log_prob + 1e-10 {
            p_two_tailed += log_prob.exp();
        }
    }

    FisherResult {
        p_value_one_tailed: p_one_tailed.min(1.0),
        p_value_two_tailed: p_two_tailed.min(1.0),
        is_signal: p_one_tailed < 0.05,
    }
}

fn ln_factorial(n: f64) -> f64 {
    if n <= 1.0 {
        return 0.0;
    }
    lgamma(n + 1.0)
}

fn lgamma(x: f64) -> f64 {
    const P: [f64; 8] = [
        676.5203681218851,
        -1259.1392167224028,
        771.3234287776531,
        -176.6150291621406,
        12.507343278686905,
        -0.13857109526572012,
        9.984_369_578_019_572e-6,
        1.5056327351493116e-7,
    ];
    const G: f64 = 7.0;
    if x < 0.5 {
        return (std::f64::consts::PI / (std::f64::consts::PI * x).sin()).ln() - lgamma(1.0 - x);
    }
    let x = x - 1.0;
    let mut a = 0.999_999_999_999_809_9;
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
    fn test_fisher_rare_event() {
        let table = ContingencyTable::new(3, 7, 1, 89);
        let result = fisher_exact_test(&table);
        assert!(result.p_value_one_tailed < 0.05);
        assert!(result.is_signal);
    }

    #[test]
    fn test_fisher_no_signal() {
        let table = ContingencyTable::new(5, 5, 5, 5);
        let result = fisher_exact_test(&table);
        assert!(!result.is_signal);
    }
}
