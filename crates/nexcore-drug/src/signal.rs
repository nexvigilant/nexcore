//! Pharmacovigilance signal types.
//!
//! `SignalEntry` carries the full disproportionality data for a single
//! adverse event: the 2×2 contingency table, computed PRR/ROR/IC statistics,
//! case count, label status, and overall verdict.
//!
//! `ContingencyTable` stores the four cells (a, b, c, d) of the standard
//! FAERS disproportionality layout:
//!
//! ```text
//!              Drug of interest   All other drugs
//! Event          a                b
//! All other      c                d
//! ```

use serde::{Deserialize, Serialize};

/// Overall pharmacovigilance signal strength verdict.
///
/// Determined by combining disproportionality statistics (PRR, ROR, IC),
/// case counts, and clinical context. Mirrors `SignalVerdict` in
/// `nexcore-pharma` for cross-crate consistency.
///
/// # Examples
///
/// ```
/// use nexcore_drug::SignalVerdict;
///
/// assert_eq!(SignalVerdict::Strong.to_string(), "Strong");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalVerdict {
    /// Consistent, strong disproportionality across multiple metrics
    Strong,
    /// Moderate signal, warrants further evaluation
    Moderate,
    /// Weak signal, monitor but no immediate action required
    Weak,
    /// Below detection threshold, likely background noise
    Noise,
}

impl std::fmt::Display for SignalVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Strong => "Strong",
            Self::Moderate => "Moderate",
            Self::Weak => "Weak",
            Self::Noise => "Noise",
        };
        f.write_str(s)
    }
}

/// Standard 2×2 contingency table for FAERS disproportionality analysis.
///
/// Cells follow the convention used in PRR/ROR/IC computations:
///
/// | Cell | Meaning |
/// |------|---------|
/// | `a`  | Reports of the event WITH the drug of interest |
/// | `b`  | Reports of the event WITH all other drugs |
/// | `c`  | Reports of all other events WITH the drug of interest |
/// | `d`  | Reports of all other events WITH all other drugs |
///
/// # Examples
///
/// ```
/// use nexcore_drug::ContingencyTable;
///
/// let table = ContingencyTable { a: 50, b: 1000, c: 200, d: 5_000_000 };
/// assert_eq!(table.total_reports(), 50 + 1000 + 200 + 5_000_000);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContingencyTable {
    /// Cases: event WITH drug
    pub a: u64,
    /// Non-cases: event WITHOUT drug (all other drugs)
    pub b: u64,
    /// Controls: other events WITH drug
    pub c: u64,
    /// Background: other events WITHOUT drug
    pub d: u64,
}

impl ContingencyTable {
    /// Total report count across all four cells.
    pub fn total_reports(&self) -> u64 {
        self.a
            .saturating_add(self.b)
            .saturating_add(self.c)
            .saturating_add(self.d)
    }

    /// Expected count under the null hypothesis of no association.
    ///
    /// `E = (a + b) × (a + c) / N`
    pub fn expected(&self) -> f64 {
        let n = self.total_reports() as f64;
        if n == 0.0 {
            return 0.0;
        }
        let row_total = (self.a + self.b) as f64;
        let col_total = (self.a + self.c) as f64;
        row_total * col_total / n
    }
}

/// Disproportionality signal entry for a single adverse event–drug pair.
///
/// Carries the full contingency table alongside the derived statistics
/// (PRR, ROR, IC), enabling downstream re-computation or audit.
///
/// # Examples
///
/// ```
/// use nexcore_drug::{ContingencyTable, SignalEntry, SignalVerdict};
///
/// let entry = SignalEntry {
///     event: "Pancreatitis".to_string(),
///     contingency: ContingencyTable { a: 120, b: 8000, c: 500, d: 5_000_000 },
///     prr: 3.02,
///     ror: 3.04,
///     ic: 1.56,
///     cases: 120,
///     on_label: true,
///     verdict: SignalVerdict::Strong,
/// };
/// assert!(entry.is_elevated());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEntry {
    /// MedDRA preferred term or verbatim adverse event description
    pub event: String,
    /// Full 2×2 contingency table for audit and re-computation
    pub contingency: ContingencyTable,
    /// Proportional Reporting Ratio
    pub prr: f64,
    /// Reporting Odds Ratio
    pub ror: f64,
    /// Information Component (Bayesian disproportionality measure)
    pub ic: f64,
    /// Number of cases (cell `a`) in the reporting database
    pub cases: u64,
    /// Whether this event appears in the current product label
    pub on_label: bool,
    /// Overall signal strength verdict
    pub verdict: SignalVerdict,
}

impl SignalEntry {
    /// Returns `true` if the signal meets at least the `Moderate` threshold.
    pub fn is_elevated(&self) -> bool {
        matches!(
            self.verdict,
            SignalVerdict::Strong | SignalVerdict::Moderate
        )
    }

    /// Returns `true` if the signal is off-label and at least moderate —
    /// the standard criterion for a potential new safety signal requiring
    /// regulatory triage.
    pub fn requires_triage(&self) -> bool {
        !self.on_label && self.is_elevated()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(verdict: SignalVerdict, on_label: bool) -> SignalEntry {
        SignalEntry {
            event: "nausea".to_string(),
            contingency: ContingencyTable {
                a: 50,
                b: 1_000,
                c: 200,
                d: 5_000_000,
            },
            prr: 2.5,
            ror: 2.6,
            ic: 1.2,
            cases: 50,
            on_label,
            verdict,
        }
    }

    #[test]
    fn signal_verdict_display() {
        assert_eq!(SignalVerdict::Strong.to_string(), "Strong");
        assert_eq!(SignalVerdict::Moderate.to_string(), "Moderate");
        assert_eq!(SignalVerdict::Weak.to_string(), "Weak");
        assert_eq!(SignalVerdict::Noise.to_string(), "Noise");
    }

    #[test]
    fn signal_verdict_serializes_round_trip() {
        for v in [
            SignalVerdict::Strong,
            SignalVerdict::Moderate,
            SignalVerdict::Weak,
            SignalVerdict::Noise,
        ] {
            let json =
                serde_json::to_string(&v).expect("serialization cannot fail on valid enum variant");
            let parsed: SignalVerdict =
                serde_json::from_str(&json).expect("deserialization cannot fail on valid JSON");
            assert_eq!(v, parsed);
        }
    }

    #[test]
    fn contingency_table_total_reports() {
        let t = ContingencyTable {
            a: 50,
            b: 1_000,
            c: 200,
            d: 5_000_000,
        };
        assert_eq!(t.total_reports(), 5_001_250);
    }

    #[test]
    fn contingency_table_expected_nonzero() {
        let t = ContingencyTable {
            a: 50,
            b: 1_000,
            c: 200,
            d: 5_000_000,
        };
        let e = t.expected();
        assert!(e > 0.0, "expected count should be positive");
    }

    #[test]
    fn contingency_table_expected_zero_when_empty() {
        let t = ContingencyTable {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
        };
        assert_eq!(t.expected(), 0.0);
    }

    #[test]
    fn signal_entry_is_elevated_strong() {
        let e = make_entry(SignalVerdict::Strong, true);
        assert!(e.is_elevated());
    }

    #[test]
    fn signal_entry_is_elevated_moderate() {
        let e = make_entry(SignalVerdict::Moderate, false);
        assert!(e.is_elevated());
    }

    #[test]
    fn signal_entry_not_elevated_weak() {
        let e = make_entry(SignalVerdict::Weak, false);
        assert!(!e.is_elevated());
    }

    #[test]
    fn signal_entry_requires_triage_off_label_moderate() {
        let e = make_entry(SignalVerdict::Moderate, false);
        assert!(e.requires_triage());
    }

    #[test]
    fn signal_entry_no_triage_when_on_label() {
        let e = make_entry(SignalVerdict::Strong, true);
        assert!(!e.requires_triage());
    }

    #[test]
    fn signal_entry_no_triage_when_weak() {
        let e = make_entry(SignalVerdict::Weak, false);
        assert!(!e.requires_triage());
    }

    #[test]
    fn signal_entry_serializes_round_trip() {
        let e = make_entry(SignalVerdict::Strong, false);
        let json = serde_json::to_string(&e).expect("serialization cannot fail on valid struct");
        let parsed: SignalEntry =
            serde_json::from_str(&json).expect("deserialization cannot fail on valid JSON");
        assert_eq!(parsed.event, e.event);
        assert_eq!(parsed.cases, e.cases);
        assert_eq!(parsed.verdict, e.verdict);
    }
}
