//! Core types for signal detection.
//!
//! # Codex Compliance
//!
//! - **Tier**: T2-C (Cross-Domain Composite)
//! - **Grounding**: All types ground to T1 primitives (u32, f64) or T2-P enums.
//! - **Quantification**: Methods are enumerated via `SignalMethod`.

use serde::{Deserialize, Serialize};
use std::fmt;

/// 2x2 contingency table for disproportionality analysis.
///
/// ```text
///                    Event    No Event
/// Drug               a        b          (a+b)
/// No Drug            c        d          (c+d)
///                   (a+c)    (b+d)        N
/// ```
///
/// # Tier: T2-C
/// Composite structure grounding to T1 (u64).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContingencyTable {
    /// Drug + Event (target cell)
    pub a: u64,
    /// Drug + No Event
    pub b: u64,
    /// No Drug + Event
    pub c: u64,
    /// No Drug + No Event
    pub d: u64,
}

impl ContingencyTable {
    /// Create a new contingency table.
    #[must_use]
    pub const fn new(a: u64, b: u64, c: u64, d: u64) -> Self {
        Self { a, b, c, d }
    }

    /// Total count (N).
    #[must_use]
    pub const fn total(&self) -> u64 {
        self.a + self.b + self.c + self.d
    }

    /// Drug reports (a + b).
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

    /// Non-event reports (b + d).
    #[must_use]
    pub const fn non_event_reports(&self) -> u64 {
        self.b + self.d
    }

    /// Expected count under independence assumption.
    #[must_use]
    pub fn expected_count(&self) -> f64 {
        let n = self.total() as f64;
        if n == 0.0 {
            return 0.0;
        }
        self.drug_reports() as f64 * self.event_reports() as f64 / n
    }

    /// Check if table is valid (all values non-negative and total > 0).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.total() > 0
    }

    /// Row 1 total (drug exposed).
    #[must_use]
    pub const fn row1(&self) -> u64 {
        self.a + self.b
    }

    /// Row 2 total (not drug exposed).
    #[must_use]
    pub const fn row2(&self) -> u64 {
        self.c + self.d
    }

    /// Column 1 total (event occurred).
    #[must_use]
    pub const fn col1(&self) -> u64 {
        self.a + self.c
    }

    /// Column 2 total (event not occurred).
    #[must_use]
    pub const fn col2(&self) -> u64 {
        self.b + self.d
    }

    /// Calculate expected count for cell 'a' (Drug+, Event+)
    /// E = (row1 * col1) / total
    #[must_use]
    pub fn expected_a(&self) -> f64 {
        let n = self.total() as f64;
        if n == 0.0 {
            return 0.0;
        }
        (self.row1() as f64 * self.col1() as f64) / n
    }
}

pub use nexcore_signal_types::{SignalMethod, SignalResult};

/// Signal detection criteria thresholds.
///
/// # Tier: T2-C
/// Configuration composite grounding to T1 (f64, u32).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SignalCriteria {
    /// Minimum PRR threshold (default: 2.0)
    pub prr_threshold: f64,
    /// Minimum ROR threshold (default: 2.0)
    pub ror_threshold: f64,
    /// Minimum ROR lower CI threshold (default: 1.0)
    pub ror_lower_ci_threshold: f64,
    /// Minimum IC025 threshold (default: 0.0)
    pub ic025_threshold: f64,
    /// Minimum EBGM threshold (default: 2.0)
    pub ebgm_threshold: f64,
    /// Minimum EB05 threshold (default: 2.0)
    pub eb05_threshold: f64,
    /// Chi-square threshold for significance (default: 3.841 for p<0.05)
    pub chi_square_threshold: f64,
    /// Minimum case count (default: 3)
    pub min_cases: u32,
}

impl SignalCriteria {
    /// Evans criteria (most commonly used).
    ///
    /// - PRR >= 2.0
    /// - Chi-square >= 3.841 (p < 0.05, df=1)
    /// - Minimum 3 cases
    #[must_use]
    pub const fn evans() -> Self {
        Self {
            prr_threshold: 2.0,
            ror_threshold: 2.0,
            ror_lower_ci_threshold: 1.0,
            ic025_threshold: 0.0,
            ebgm_threshold: 2.0,
            eb05_threshold: 2.0,
            chi_square_threshold: 3.841, // CRITICAL: Exact value, NOT 4.0
            min_cases: 3,
        }
    }

    /// WHO-UMC criteria.
    #[must_use]
    pub const fn who_umc() -> Self {
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

    /// FDA FAERS criteria.
    #[must_use]
    pub const fn fda_faers() -> Self {
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

    /// Sensitive criteria (lower thresholds for early detection).
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

/// Combined result from all disproportionality methods.
///
/// # Tier: T3
/// Domain-specific composite.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisproportionalityResult {
    /// Algorithm method name
    pub method: SignalMethod,
    /// Point estimate
    pub point_estimate: f64,
    /// Lower CI
    pub lower_ci: f64,
    /// Upper CI
    pub upper_ci: f64,
    /// Chi-square (if applicable)
    pub chi_square: Option<f64>,
    /// Signal detected
    pub is_signal: bool,
}

impl From<SignalResult> for DisproportionalityResult {
    fn from(r: SignalResult) -> Self {
        Self {
            method: r.method,
            point_estimate: r.point_estimate,
            lower_ci: r.lower_ci,
            upper_ci: r.upper_ci,
            chi_square: r.chi_square,
            is_signal: r.is_signal,
        }
    }
}
