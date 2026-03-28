//! # nexcore-drug-upadacitinib
//!
//! Upadacitinib (Rinvoq) — selective JAK1 inhibitor from AbbVie.
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

/// Upadacitinib drug safety model.
///
/// # Examples
///
/// ```
/// use nexcore_drug_upadacitinib::Upadacitinib;
/// use nexcore_drug::analysis::DrugAnalysis;
///
/// let u = Upadacitinib::new();
/// assert_eq!(u.drug().generic_name, "upadacitinib");
/// assert!(u.drug().has_boxed_warning());
/// assert!(u.signal_count() >= 5);
/// ```
pub struct Upadacitinib {
    drug: Drug,
}

impl Upadacitinib {
    /// Construct a new `Upadacitinib` instance with the canonical drug data.
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

impl Default for Upadacitinib {
    fn default() -> Self {
        Self::new()
    }
}

impl DrugAnalysis for Upadacitinib {
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
    fn upadacitinib_loads() {
        let u = Upadacitinib::new();
        assert_eq!(u.drug().generic_name, "upadacitinib");
        assert_eq!(u.drug().brand_names[0], "Rinvoq");
    }

    #[test]
    fn upadacitinib_has_boxed_warning() {
        let u = Upadacitinib::new();
        assert!(u.drug().has_boxed_warning());
    }

    #[test]
    fn upadacitinib_signal_count() {
        let u = Upadacitinib::new();
        assert!(u.signal_count() >= 5);
    }

    #[test]
    fn upadacitinib_serious_infection_is_strongest() {
        let u = Upadacitinib::new();
        let s = u.strongest_signal().expect("has signals");
        assert!(
            s.event.to_lowercase().contains("infection"),
            "strongest signal should be serious infection"
        );
    }

    #[test]
    fn upadacitinib_all_signals_on_label() {
        let u = Upadacitinib::new();
        assert!(u.off_label_signals().is_empty());
    }
}
