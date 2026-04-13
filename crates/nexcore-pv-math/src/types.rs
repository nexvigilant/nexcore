//! Core types for PV math: contingency table and signal criteria.
//!
//! Re-exports `SignalMethod` and `SignalResult` from `nexcore-signal-types`
//! so consumers only need one import path.

use serde::{Deserialize, Serialize};

// Re-export canonical signal types — no duplication.
pub use nexcore_signal_types::{SignalMethod, SignalResult};

/// 2×2 contingency table for disproportionality analysis.
///
/// ```text
///                  Event    No Event
/// Drug             a        b          (a+b)
/// No Drug          c        d          (c+d)
///                 (a+c)    (b+d)        N
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TwoByTwoTable {
    /// Drug + Event (target cell)
    pub a: u64,
    /// Drug + No Event
    pub b: u64,
    /// No Drug + Event
    pub c: u64,
    /// No Drug + No Event
    pub d: u64,
}

impl TwoByTwoTable {
    /// Construct a new table.
    #[must_use]
    pub const fn new(a: u64, b: u64, c: u64, d: u64) -> Self {
        Self { a, b, c, d }
    }

    /// Total count N = a + b + c + d.
    #[must_use]
    pub const fn total(&self) -> u64 {
        self.a + self.b + self.c + self.d
    }

    /// Drug-exposed reports (a + b).
    #[must_use]
    pub const fn drug_reports(&self) -> u64 {
        self.a + self.b
    }

    /// Event reports (a + c).
    #[must_use]
    pub const fn event_reports(&self) -> u64 {
        self.a + self.c
    }

    /// Non-drug reports (c + d).
    #[must_use]
    pub const fn non_drug_reports(&self) -> u64 {
        self.c + self.d
    }

    /// Expected count for cell `a` under independence: E = (a+b)(a+c) / N.
    #[must_use]
    pub fn expected_count(&self) -> f64 {
        let n = self.total() as f64;
        if n == 0.0 {
            return 0.0;
        }
        self.drug_reports() as f64 * self.event_reports() as f64 / n
    }

    /// Returns `true` when the total is non-zero.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.total() > 0
    }
}

/// Signal-detection threshold configuration.
///
/// Defaults to Evans criteria (PRR/ROR ≥ 2.0, χ² ≥ 3.841, n ≥ 3).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SignalCriteria {
    /// Minimum PRR for a signal (Evans: 2.0).
    pub prr_threshold: f64,
    /// Minimum ROR for a signal (Evans: 2.0).
    pub ror_threshold: f64,
    /// Minimum ROR lower-CI for a signal (Evans: 1.0).
    pub ror_lower_ci_threshold: f64,
    /// Minimum IC025 for a signal (WHO-UMC: 0.0).
    pub ic025_threshold: f64,
    /// Minimum EBGM for a signal (Evans: 2.0).
    pub ebgm_threshold: f64,
    /// Minimum EB05 for a signal (Evans: 2.0).
    pub eb05_threshold: f64,
    /// Chi-square critical value (Evans: 3.841, i.e. p < 0.05, df = 1).
    pub chi_square_threshold: f64,
    /// Minimum case count (Evans: 3).
    pub min_cases: u32,
}

impl SignalCriteria {
    /// Evans criteria — the standard for most pharmacovigilance databases.
    ///
    /// CRITICAL: chi-square threshold is 3.841 (p < 0.05, df = 1), NOT 4.0.
    #[must_use]
    pub const fn evans() -> Self {
        Self {
            prr_threshold: 2.0,
            ror_threshold: 2.0,
            ror_lower_ci_threshold: 1.0,
            ic025_threshold: 0.0,
            ebgm_threshold: 2.0,
            eb05_threshold: 2.0,
            chi_square_threshold: 3.841,
            min_cases: 3,
        }
    }

    /// WHO-UMC criteria (same thresholds as Evans for disproportionality measures).
    #[must_use]
    pub const fn who_umc() -> Self {
        Self::evans()
    }

    /// Sensitive criteria — lower thresholds for early signal detection.
    #[must_use]
    pub const fn sensitive() -> Self {
        Self {
            prr_threshold: 1.5,
            ror_threshold: 1.5,
            ror_lower_ci_threshold: 0.5,
            ic025_threshold: -0.5,
            ebgm_threshold: 1.5,
            eb05_threshold: 1.5,
            chi_square_threshold: 2.706, // p < 0.10
            min_cases: 2,
        }
    }
}

impl Default for SignalCriteria {
    fn default() -> Self {
        Self::evans()
    }
}
