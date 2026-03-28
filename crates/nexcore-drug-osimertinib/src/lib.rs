//! # nexcore-drug-osimertinib
//!
//! Osimertinib (Tagrisso) — third-generation EGFR TKI from AstraZeneca.
//! Static drug safety model implementing [`DrugAnalysis`].

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod catalog;

use nexcore_drug::{
    Drug, SignalEntry,
    analysis::{DefaultDrugAnalysis, DrugAnalysis, SignalComparison},
};

/// Osimertinib drug safety model.
///
/// # Examples
///
/// ```
/// use nexcore_drug_osimertinib::Osimertinib;
/// use nexcore_drug::analysis::DrugAnalysis;
///
/// let o = Osimertinib::new();
/// assert_eq!(o.drug().generic_name, "osimertinib");
/// assert!(!o.drug().has_boxed_warning());
/// assert!(o.signal_count() >= 4);
/// ```
pub struct Osimertinib {
    drug: Drug,
}

impl Osimertinib {
    /// Construct a new `Osimertinib` instance with the canonical drug data.
    pub fn new() -> Self {
        Self {
            drug: catalog::drug(),
        }
    }

    /// Total number of safety signals in the portfolio.
    pub fn signal_count(&self) -> usize {
        self.drug.signal_count()
    }
}

impl Default for Osimertinib {
    fn default() -> Self {
        Self::new()
    }
}

impl DrugAnalysis for Osimertinib {
    fn drug(&self) -> &Drug {
        &self.drug
    }

    fn signal_portfolio(&self) -> &[SignalEntry] {
        &self.drug.safety_signals
    }

    fn strongest_signal(&self) -> Option<&SignalEntry> {
        self.drug.strongest_signal()
    }

    fn on_label_signals(&self) -> Vec<&SignalEntry> {
        self.drug.on_label_signals()
    }

    fn off_label_signals(&self) -> Vec<&SignalEntry> {
        self.drug.off_label_signals()
    }

    fn compare_signals(&self, other: &dyn DrugAnalysis) -> Vec<SignalComparison> {
        DefaultDrugAnalysis::new(&self.drug).compare_signals(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn osimertinib_loads() {
        let o = Osimertinib::new();
        assert_eq!(o.drug().generic_name, "osimertinib");
        assert_eq!(o.drug().brand_names[0], "Tagrisso");
    }

    #[test]
    fn osimertinib_no_boxed_warning() {
        let o = Osimertinib::new();
        assert!(!o.drug().has_boxed_warning());
    }

    #[test]
    fn osimertinib_signal_count() {
        let o = Osimertinib::new();
        assert!(o.signal_count() >= 4);
    }

    #[test]
    fn osimertinib_ild_is_strongest_signal() {
        let o = Osimertinib::new();
        let s = o.strongest_signal().expect("has signals");
        assert!(
            s.event.contains("lung") || s.event.contains("pneumonitis"),
            "strongest signal should be ILD/pneumonitis"
        );
    }

    #[test]
    fn osimertinib_owner_is_astrazeneca() {
        let o = Osimertinib::new();
        assert_eq!(o.drug().owner.as_deref(), Some("AstraZeneca PLC"));
    }
}
