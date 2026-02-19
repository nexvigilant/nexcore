//! Omega Shrinkage Method for Drug-Drug Interaction (DDI) Detection
//!
//! Detects unexpected drug-drug interactions by comparing observed co-exposure
//! adverse event rates to expected rates based on individual drug profiles.
//!
//! # Theory
//!
//! For two drugs A and B, the Omega statistic measures interaction:
//!
//! ```text
//! Ω = log₂(O_AB / E_AB)
//!
//! where:
//!   O_AB = observed count of AE with both drugs
//!   E_AB = expected count assuming independence
//!        = (n_A × n_B × n_AE) / N²
//! ```
//!
//! With shrinkage to handle sparse data:
//!
//! ```text
//! Ω_shrunk = log₂((O_AB + 0.5) / (E_AB + 0.5))
//! ```
//!
//! # Signal Detection
//!
//! A DDI signal is detected when:
//! - Ω₀₂₅ > 0 (lower 2.5% credibility bound positive)
//! - At least 3 co-exposure cases
//!
//! # Use Cases
//!
//! - Post-market DDI surveillance
//! - Pharmacovigilance signal detection for polypharmacy
//! - Identifying synergistic toxicity
//!
//! # References
//!
//! - Norén GN, Sundberg R, Bate A, Edwards IR (2008). "A statistical methodology for
//!   drug-drug interaction surveillance." Statistics in Medicine 27(16):3057-3070.
//!   DOI: [10.1002/sim.3247](https://doi.org/10.1002/sim.3247)
//!
//! - Thakrar BT, Grundschober SB, Doessegger L (2007). "Detecting signals of drug-drug
//!   interactions in a spontaneous reports database." British Journal of Clinical
//!   Pharmacology 64(4):489-495. DOI: [10.1111/j.1365-2125.2007.02900.x](https://doi.org/10.1111/j.1365-2125.2007.02900.x)
//!
//! - Gosho M, Maruo K, Tada K, Hirakawa A (2017). "Utilization of chi-square statistics
//!   for screening adverse drug-drug interactions in spontaneous reporting systems."
//!   European Journal of Clinical Pharmacology 73(6):779-786.
//!   DOI: [10.1007/s00228-017-2233-3](https://doi.org/10.1007/s00228-017-2233-3)

use crate::signals::core::error::SignalError;
use crate::signals::core::stats::log2;
use serde::{Deserialize, Serialize};

/// Drug-drug interaction contingency data.
///
/// Represents a 2×2×2 table for drug A, drug B, and adverse event.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DDITable {
    /// Cases with drug A + drug B + adverse event
    pub n_ab_ae: u32,
    /// Cases with drug A + drug B (total co-exposures)
    pub n_ab: u32,
    /// Cases with drug A + adverse event (without B)
    pub n_a_ae: u32,
    /// Cases with drug B + adverse event (without A)
    pub n_b_ae: u32,
    /// Total cases with drug A
    pub n_a: u32,
    /// Total cases with drug B
    pub n_b: u32,
    /// Total cases with adverse event
    pub n_ae: u32,
    /// Total database size
    pub n_total: u32,
}

impl DDITable {
    /// Create a new DDI table.
    #[must_use]
    pub const fn new(
        n_ab_ae: u32,
        n_ab: u32,
        n_a_ae: u32,
        n_b_ae: u32,
        n_a: u32,
        n_b: u32,
        n_ae: u32,
        n_total: u32,
    ) -> Self {
        Self {
            n_ab_ae,
            n_ab,
            n_a_ae,
            n_b_ae,
            n_a,
            n_b,
            n_ae,
            n_total,
        }
    }

    /// Calculate expected count under independence assumption.
    ///
    /// E_AB = (n_A × n_B × n_AE) / N²
    #[must_use]
    pub fn expected_count(&self) -> f64 {
        if self.n_total == 0 {
            return 0.0;
        }
        let n = f64::from(self.n_total);
        f64::from(self.n_a) * f64::from(self.n_b) * f64::from(self.n_ae) / (n * n)
    }

    /// Check if table is valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.n_total > 0 && self.n_ab <= self.n_a.min(self.n_b) && self.n_ab_ae <= self.n_ab
    }
}

/// Configuration for Omega shrinkage calculation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OmegaConfig {
    /// Shrinkage constant (default: 0.5)
    pub shrinkage: f64,
    /// Minimum co-exposure cases for signal (default: 3)
    pub min_cases: u32,
    /// Confidence level for credibility interval (default: 0.95)
    pub confidence_level: f64,
}

impl Default for OmegaConfig {
    fn default() -> Self {
        Self {
            shrinkage: 0.5,
            min_cases: 3,
            confidence_level: 0.95,
        }
    }
}

impl OmegaConfig {
    /// Sensitive configuration for early signal detection.
    #[must_use]
    pub const fn sensitive() -> Self {
        Self {
            shrinkage: 0.5,
            min_cases: 2,
            confidence_level: 0.90,
        }
    }

    /// Conservative configuration for confirmed signals.
    #[must_use]
    pub const fn conservative() -> Self {
        Self {
            shrinkage: 0.5,
            min_cases: 5,
            confidence_level: 0.99,
        }
    }
}

/// Result of Omega shrinkage calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OmegaResult {
    /// Omega point estimate (log₂ scale)
    pub omega: f64,
    /// Lower credibility bound (e.g., Ω₀₂₅)
    pub omega_lower: f64,
    /// Upper credibility bound (e.g., Ω₉₇₅)
    pub omega_upper: f64,
    /// Observed count
    pub observed: u32,
    /// Expected count
    pub expected: f64,
    /// Whether a DDI signal is detected
    pub is_signal: bool,
    /// Observed/Expected ratio
    pub obs_exp_ratio: f64,
}

/// Calculate Omega statistic for drug-drug interaction detection.
///
/// # Arguments
///
/// * `table` - DDI contingency data
/// * `config` - Omega calculation configuration
///
/// # Returns
///
/// `OmegaResult` with Omega estimate, credibility intervals, and signal status.
///
/// # Complexity
///
/// - **Time**: O(1)
/// - **Space**: O(1)
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::pv::signals::bayesian::omega_shrinkage::{calculate_omega, DDITable, OmegaConfig};
///
/// let table = DDITable::new(
///     15,     // n_ab_ae: co-exposure + AE
///     100,    // n_ab: total co-exposures
///     50,     // n_a_ae: drug A + AE (without B)
///     40,     // n_b_ae: drug B + AE (without A)
///     500,    // n_a: total drug A
///     400,    // n_b: total drug B
///     200,    // n_ae: total AE
///     10000,  // n_total
/// );
///
/// let config = OmegaConfig::default();
/// let result = calculate_omega(&table, &config).unwrap();
///
/// if result.is_signal {
///     println!("DDI signal detected! Ω = {:.2}", result.omega);
/// }
/// ```
pub fn calculate_omega(table: &DDITable, config: &OmegaConfig) -> Result<OmegaResult, SignalError> {
    if !table.is_valid() {
        return Err(SignalError::invalid_table("Invalid DDI table"));
    }

    let observed = f64::from(table.n_ab_ae);
    let expected = table.expected_count();

    if expected <= 0.0 {
        return Err(SignalError::InvalidExpectedCount(expected));
    }

    // Shrinkage estimator
    let obs_shrunk = observed + config.shrinkage;
    let exp_shrunk = expected + config.shrinkage;

    // Omega = log2(O/E) with shrinkage
    let omega = log2(obs_shrunk / exp_shrunk);

    // Variance approximation using delta method
    // Var(log(O/E)) ≈ 1/O + 1/E for Poisson
    // With shrinkage: Var ≈ 1/(O+k) + 1/(E+k)
    let variance = 1.0 / obs_shrunk + 1.0 / exp_shrunk;
    let se = (variance / std::f64::consts::LN_2.powi(2)).sqrt(); // Convert to log2 scale

    // Credibility interval
    let z = z_for_confidence(config.confidence_level);
    let omega_lower = omega - z * se;
    let omega_upper = omega + z * se;

    // Signal detection: Ω_lower > 0 and minimum cases
    let is_signal = omega_lower > 0.0 && table.n_ab_ae >= config.min_cases;

    // O/E ratio
    let obs_exp_ratio = if expected > 0.0 {
        observed / expected
    } else {
        0.0
    };

    Ok(OmegaResult {
        omega,
        omega_lower,
        omega_upper,
        observed: table.n_ab_ae,
        expected,
        is_signal,
        obs_exp_ratio,
    })
}

/// Calculate interaction contrast Omega.
///
/// This variant explicitly accounts for individual drug effects:
///
/// ```text
/// Ω_interaction = log₂(P(AE|A,B) / P_expected)
///
/// where P_expected is based on additive or multiplicative models.
/// ```
///
/// # Arguments
///
/// * `table` - DDI contingency data
/// * `config` - Configuration
/// * `additive` - Use additive model (true) or multiplicative (false)
pub fn calculate_omega_interaction(
    table: &DDITable,
    config: &OmegaConfig,
    additive: bool,
) -> Result<OmegaResult, SignalError> {
    if !table.is_valid() {
        return Err(SignalError::invalid_table("Invalid DDI table"));
    }

    let n = f64::from(table.n_total);
    if n == 0.0 {
        return Err(SignalError::InsufficientData("Empty database".into()));
    }

    // Background AE rate
    let p_ae = f64::from(table.n_ae) / n;

    // Drug-specific AE rates
    let p_ae_a = if table.n_a > 0 {
        f64::from(table.n_a_ae) / f64::from(table.n_a)
    } else {
        p_ae
    };

    let p_ae_b = if table.n_b > 0 {
        f64::from(table.n_b_ae) / f64::from(table.n_b)
    } else {
        p_ae
    };

    // Expected rate under interaction model
    let expected_rate = if additive {
        // Additive: P(AE|A,B) = P(AE|A) + P(AE|B) - P(AE)
        (p_ae_a + p_ae_b - p_ae).max(0.0).min(1.0)
    } else {
        // Multiplicative: P(AE|A,B) = P(AE|A) × P(AE|B) / P(AE)
        if p_ae > 0.0 {
            (p_ae_a * p_ae_b / p_ae).min(1.0)
        } else {
            p_ae_a * p_ae_b
        }
    };

    // Expected count
    let expected = expected_rate * f64::from(table.n_ab);

    // Observed
    let observed = f64::from(table.n_ab_ae);

    // Shrinkage
    let obs_shrunk = observed + config.shrinkage;
    let exp_shrunk = expected + config.shrinkage;

    let omega = log2(obs_shrunk / exp_shrunk);

    // Variance and CI
    let variance = 1.0 / obs_shrunk + 1.0 / exp_shrunk;
    let se = (variance / std::f64::consts::LN_2.powi(2)).sqrt();
    let z = z_for_confidence(config.confidence_level);

    let omega_lower = omega - z * se;
    let omega_upper = omega + z * se;

    let is_signal = omega_lower > 0.0 && table.n_ab_ae >= config.min_cases;

    let obs_exp_ratio = if expected > 0.0 {
        observed / expected
    } else {
        0.0
    };

    Ok(OmegaResult {
        omega,
        omega_lower,
        omega_upper,
        observed: table.n_ab_ae,
        expected,
        is_signal,
        obs_exp_ratio,
    })
}

/// Get z-score for given confidence level.
fn z_for_confidence(confidence: f64) -> f64 {
    match () {
        () if (confidence - 0.90).abs() < 1e-9 => 1.645,
        () if (confidence - 0.95).abs() < 1e-9 => 1.96,
        () if (confidence - 0.99).abs() < 1e-9 => 2.576,
        () => crate::signals::core::stats::z_score_for_confidence(confidence),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_table() -> DDITable {
        DDITable::new(
            15,    // n_ab_ae
            100,   // n_ab
            50,    // n_a_ae
            40,    // n_b_ae
            500,   // n_a
            400,   // n_b
            200,   // n_ae
            10000, // n_total
        )
    }

    #[test]
    fn test_ddi_table_expected() {
        let table = create_test_table();
        let expected = table.expected_count();

        // E = (500 × 400 × 200) / 10000² = 40,000,000 / 100,000,000 = 0.4
        assert!((expected - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_omega_signal() {
        let table = create_test_table();
        let config = OmegaConfig::default();

        let result = calculate_omega(&table, &config).unwrap();

        // Observed (15) >> Expected (~0.4), strong signal
        assert!(result.omega > 0.0);
        assert!(result.is_signal);
        assert!(result.obs_exp_ratio > 10.0);
    }

    #[test]
    fn test_omega_no_signal() {
        // Create table with expected ≈ observed
        let table = DDITable::new(
            4,     // n_ab_ae
            100,   // n_ab
            200,   // n_a_ae
            200,   // n_b_ae
            1000,  // n_a
            1000,  // n_b
            400,   // n_ae
            10000, // n_total
        );
        let config = OmegaConfig::default();

        let result = calculate_omega(&table, &config).unwrap();

        // Expected = (1000 × 1000 × 400) / 100M = 4
        // Observed = 4, so Ω ≈ 0
        assert!(result.omega.abs() < 1.0);
    }

    #[test]
    fn test_omega_insufficient_cases() {
        let table = DDITable::new(
            2,     // n_ab_ae - below threshold
            50,    // n_ab
            10,    // n_a_ae
            10,    // n_b_ae
            100,   // n_a
            100,   // n_b
            50,    // n_ae
            10000, // n_total
        );
        let config = OmegaConfig::default(); // min_cases = 3

        let result = calculate_omega(&table, &config).unwrap();

        // Even if Ω_lower > 0, insufficient cases means no signal
        assert!(!result.is_signal);
    }

    #[test]
    fn test_omega_interaction_additive() {
        let table = create_test_table();
        let config = OmegaConfig::default();

        let result = calculate_omega_interaction(&table, &config, true).unwrap();

        assert!(result.omega.is_finite());
        assert!(result.expected > 0.0);
    }

    #[test]
    fn test_omega_interaction_multiplicative() {
        let table = create_test_table();
        let config = OmegaConfig::default();

        let result = calculate_omega_interaction(&table, &config, false).unwrap();

        assert!(result.omega.is_finite());
        assert!(result.expected > 0.0);
    }

    #[test]
    fn test_omega_config_sensitive() {
        let config = OmegaConfig::sensitive();
        assert_eq!(config.min_cases, 2);
        assert!((config.confidence_level - 0.90).abs() < 0.01);
    }

    #[test]
    fn test_invalid_table() {
        let table = DDITable::new(0, 0, 0, 0, 0, 0, 0, 0);
        let config = OmegaConfig::default();

        let result = calculate_omega(&table, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_credibility_interval() {
        let table = create_test_table();
        let config = OmegaConfig::default();

        let result = calculate_omega(&table, &config).unwrap();

        // CI should bracket the point estimate
        assert!(result.omega_lower < result.omega);
        assert!(result.omega_upper > result.omega);
    }
}
