//! # nexcore-drug-pembrolizumab
//!
//! Pembrolizumab (Keytruda) — PD-1 checkpoint inhibitor from Merck.
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

/// Pembrolizumab drug safety model.
///
/// # Examples
///
/// ```
/// use nexcore_drug_pembrolizumab::Pembrolizumab;
/// use nexcore_drug::analysis::DrugAnalysis;
///
/// let p = Pembrolizumab::new();
/// assert_eq!(p.drug().generic_name, "pembrolizumab");
/// assert!(!p.drug().has_boxed_warning());
/// assert!(p.signal_count() >= 5);
/// ```
pub struct Pembrolizumab {
    drug: Drug,
}

impl Pembrolizumab {
    /// Construct a new `Pembrolizumab` instance with the canonical drug data.
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

impl Default for Pembrolizumab {
    fn default() -> Self {
        Self::new()
    }
}

impl DrugAnalysis for Pembrolizumab {
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
    fn pembrolizumab_loads() {
        let p = Pembrolizumab::new();
        assert_eq!(p.drug().generic_name, "pembrolizumab");
        assert_eq!(p.drug().brand_names[0], "Keytruda");
    }

    #[test]
    fn pembrolizumab_no_boxed_warning() {
        let p = Pembrolizumab::new();
        assert!(!p.drug().has_boxed_warning());
    }

    #[test]
    fn pembrolizumab_signal_count() {
        let p = Pembrolizumab::new();
        assert!(p.signal_count() >= 5);
    }

    #[test]
    fn pembrolizumab_all_signals_on_label() {
        let p = Pembrolizumab::new();
        assert!(p.off_label_signals().is_empty());
    }

    #[test]
    fn pembrolizumab_owner_is_merck() {
        let p = Pembrolizumab::new();
        assert_eq!(p.drug().owner.as_deref(), Some("Merck & Co., Inc."));
    }
}
